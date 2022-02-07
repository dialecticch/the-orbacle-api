use super::Asset;
use crate::opensea::types::Asset as OpenseaAsset;
use crate::storage::read::read_traits_overlaping_tokens;
use anyhow::Result;
use chrono::Utc;
use futures::StreamExt;
use itertools::Itertools;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};

pub async fn generate_token_mapping(
    os_assets: Vec<OpenseaAsset>,
) -> Result<HashMap<String, Vec<i32>>> {
    println!("start");
    let mut map = HashMap::<String, Vec<i32>>::default();

    for asset in os_assets {
        let trait_list = asset
            .traits
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|t| t.trait_type.to_lowercase() != "serial")
            .collect::<Vec<_>>();

        let trait_ids = trait_list
            .iter()
            .map(|t| {
                format!(
                    "{}:{}",
                    &t.trait_type.to_lowercase(),
                    &t.value.to_lowercase()
                )
            })
            .collect::<Vec<String>>();

        for t in trait_ids {
            match map.get(&t) {
                Some(l) => {
                    let mut nl = l.clone();
                    nl.push(asset.token_id);
                    map.insert(t.clone(), nl);
                }
                None => {
                    map.insert(t.clone(), vec![]);
                }
            }
        }
    }

    println!("Unique Traits {:?}", map.keys().len());

    Ok(map)
}

pub async fn process_assets(
    pool: PgPool,
    os_assets: Vec<OpenseaAsset>,
    collection_slug: &str,
    ignored_trait_types_overlap: Vec<String>,
) -> Result<Vec<Asset>> {
    let mut assets: Vec<Asset> = vec![];
    for asset in os_assets {
        let trait_list = asset
            .traits
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|t| t.trait_type.to_lowercase() != "serial")
            .collect::<Vec<_>>();

        let trait_ids = trait_list
            .iter()
            .map(|t| {
                format!(
                    "{}:{}",
                    &t.trait_type.to_lowercase(),
                    &t.value.to_lowercase()
                )
            })
            .collect::<Vec<String>>();

        let unique_traits = asset
            .traits
            .clone()
            .unwrap_or_default()
            .iter()
            .filter(|t| t.trait_count.unwrap_or_default() == 1)
            .count();

        assets.push(Asset {
            name: asset.name.unwrap_or(format!(
                "{} #{}",
                asset.asset_contract.symbol.unwrap_or_default(),
                asset.token_id
            )),
            collection_slug: collection_slug.to_string(),
            token_id: asset.token_id as i32,
            image_url: asset.image_url,
            owner: asset.owner.address,
            traits: trait_ids,
            unique_traits: unique_traits as i32,
            traits_3_combination_overlap: 0i32,
            traits_4_combination_overlap: 0i32,
            traits_5_combination_overlap: 0i32,
            traits_3_combination_overlap_ids: vec![],
            traits_4_combination_overlap_ids: vec![],
            traits_5_combination_overlap_ids: vec![],
        });
    }
    println!("base processing done");

    let b = Utc::now();

    let mut res = vec![];
    let chunks: Vec<&[Asset]> = assets.chunks(assets.len() / 16).collect();

    let mut stream = futures::stream::iter(0..chunks.len())
        .map(|i| {
            compute_combinations(
                pool.clone(),
                collection_slug,
                chunks[i].clone().to_vec(),
                &ignored_trait_types_overlap,
            )
        })
        .buffer_unordered(16);

    while let Some(result) = stream.next().await {
        match result {
            Ok(mut resp) => {
                res.append(&mut resp);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    let a = Utc::now();
    println!("elapsed {:?}s", (a - b).num_seconds());

    Ok(res)
}

async fn compute_combinations(
    pool: PgPool,
    collection_slug: &str,
    assets: Vec<Asset>,
    ignored_trait_types_overlap: &[String],
) -> Result<Vec<Asset>> {
    let mut res = vec![];

    let mut conn = pool.acquire().await?;

    for mut asset in assets {
        let mut unique_3 = HashSet::<i32>::new();
        let mut unique_4 = HashSet::<i32>::new();
        let mut unique_5 = HashSet::<i32>::new();

        let asset_traits: Vec<_> = asset
            .traits
            .clone()
            .iter()
            .filter(|a| !ignored_trait_types_overlap.contains(a))
            .cloned()
            .collect();

        for vpair in asset_traits.iter().combinations(3) {
            let traits = vpair.into_iter().map(|t| t.to_string()).collect::<Vec<_>>();

            let ids = read_traits_overlaping_tokens(&mut conn, collection_slug, &traits).await?;
            unique_3.extend(ids);
        }

        for vpair in asset_traits.iter().combinations(4) {
            let traits = vpair.into_iter().map(|t| t.to_string()).collect::<Vec<_>>();

            let ids = read_traits_overlaping_tokens(&mut conn, collection_slug, &traits).await?;
            unique_4.extend(ids);
        }

        for vpair in asset_traits.iter().combinations(5) {
            let traits = vpair.into_iter().map(|t| t.to_string()).collect::<Vec<_>>();

            let ids = read_traits_overlaping_tokens(&mut conn, collection_slug, &traits).await?;
            unique_5.extend(ids);
        }

        asset.traits_3_combination_overlap = unique_3.len() as i32;
        asset.traits_4_combination_overlap = unique_4.len() as i32;
        asset.traits_5_combination_overlap = unique_5.len() as i32;
        asset.traits_3_combination_overlap_ids = unique_3.into_iter().collect::<Vec<_>>();
        asset.traits_4_combination_overlap_ids = unique_4.into_iter().collect::<Vec<_>>();
        asset.traits_5_combination_overlap_ids = unique_5.into_iter().collect::<Vec<_>>();

        res.push(asset.clone());
    }

    println!("post processing done");

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::opensea::types::*;
    fn get_assets() -> Vec<OpenseaAsset> {
        let asset1 = OpenseaAsset {
            name: Some(String::from("Test")),
            asset_contract: AssetContract::default(),
            token_id: 1i32,
            image_url: String::from("Test"),
            sell_orders: None,
            traits: Some(vec![
                Trait {
                    trait_type: String::from("background"),
                    value: String::from("black"),
                    trait_count: Some(100),
                },
                Trait {
                    trait_type: String::from("head"),
                    value: String::from("illuminatus"),
                    trait_count: Some(13),
                },
                Trait {
                    trait_type: String::from("body"),
                    value: String::from("Rainbow Suit"),
                    trait_count: Some(9),
                },
                Trait {
                    trait_type: String::from("familiar"),
                    value: String::from("Ancient Sphinx"),
                    trait_count: Some(9),
                },
                Trait {
                    trait_type: String::from("rune"),
                    value: String::from("Rune of Infinity"),
                    trait_count: Some(9),
                },
            ]),
            owner: Owner {
                address: String::from("addr"),
            },
        };

        let asset2 = OpenseaAsset {
            name: Some(String::from("Test")),
            asset_contract: AssetContract::default(),
            token_id: 2i32,
            image_url: String::from("Test"),
            sell_orders: None,
            traits: Some(vec![
                Trait {
                    trait_type: String::from("background"),
                    value: String::from("black"),
                    trait_count: Some(100),
                },
                Trait {
                    trait_type: String::from("head"),
                    value: String::from("great old one"),
                    trait_count: Some(26),
                },
                Trait {
                    trait_type: String::from("body"),
                    value: String::from("Rainbow Suit"),
                    trait_count: Some(18),
                },
                Trait {
                    trait_type: String::from("familiar"),
                    value: String::from("Ancient dog"),
                    trait_count: Some(18),
                },
                Trait {
                    trait_type: String::from("rune"),
                    value: String::from("Rune of Infinity"),
                    trait_count: Some(18),
                },
            ]),
            owner: Owner {
                address: String::from("addr"),
            },
        };

        let asset3 = OpenseaAsset {
            name: Some(String::from("Test")),
            asset_contract: AssetContract::default(),
            token_id: 3i32,
            image_url: String::from("Test"),
            sell_orders: None,
            traits: Some(vec![
                Trait {
                    trait_type: String::from("background"),
                    value: String::from("red"),
                    trait_count: Some(200),
                },
                Trait {
                    trait_type: String::from("head"),
                    value: String::from("great old one"),
                    trait_count: Some(130),
                },
                Trait {
                    trait_type: String::from("body"),
                    value: String::from("Rainbow Suit"),
                    trait_count: Some(90),
                },
                Trait {
                    trait_type: String::from("familiar"),
                    value: String::from("Ancient dog"),
                    trait_count: Some(90),
                },
                Trait {
                    trait_type: String::from("rune"),
                    value: String::from("Rune of Infinity"),
                    trait_count: Some(90),
                },
            ]),
            owner: Owner {
                address: String::from("addr"),
            },
        };
        vec![asset1, asset2, asset3]
    }
    #[tokio::test]
    async fn test_asset_process() {
        // let a = process_assets(
        //     get_assets(),
        //     "forgottenruneswizardscult",
        //     vec!["background".to_string()],
        // )
        // .await
        // .unwrap();
        // println!("{}", serde_json::to_string_pretty(&a).unwrap());
        // assert!(!a.is_empty());
    }

    #[tokio::test]
    async fn test_generate_token_mapping() {
        let a = generate_token_mapping(get_assets()).await.unwrap();
        println!("{}", serde_json::to_string_pretty(&a).unwrap());
        assert!(!a.is_empty());
    }
}
