use super::read::{read_assets_with_traits, read_collection};
use super::Asset;
use crate::opensea::types::Asset as OpenseaAsset;
use crate::storage::establish_connection;
use anyhow::Result;
use futures::StreamExt;
use itertools::Itertools;
use sqlx::PgConnection;
use std::collections::HashSet;

pub async fn process_assets(
    conn: &mut PgConnection,
    os_assets: Vec<OpenseaAsset>,
    collection_slug: &str,
) -> Result<Vec<Asset>> {
    let collection = read_collection(conn, collection_slug).await?;

    println!("{:?}", collection.total_supply);

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

    Ok(assets)
}

pub async fn generate_overlaps(
    assets: Vec<Asset>,
    collection_slug: &str,
    ignored_trait_types_overlap: &Vec<String>,
) -> Result<Vec<Asset>> {
    let chunks: Vec<_> = assets.chunks(100).collect();

    let mut stream = futures::stream::iter(0..chunks.len())
        .map(|i| {
            let ignored = ignored_trait_types_overlap.clone();
            let list = chunks[i].clone().to_vec();
            _generate_overlaps(list.clone(), collection_slug.clone(), ignored.clone())
        })
        .buffer_unordered(5);

    let mut results = vec![];

    while let Some(result) = stream.next().await {
        match result {
            Ok(mut resp) => {
                results.append(&mut resp);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    Ok(results)
}

pub async fn _generate_overlaps(
    assets: Vec<Asset>,
    collection_slug: &str,
    ignored_trait_types_overlap: Vec<String>,
) -> Result<Vec<Asset>> {
    let mut res = vec![];
    let pool = establish_connection().await;
    let mut conn = pool.acquire().await.unwrap();
    for mut asset in assets {
        println!("{:?}", asset.token_id);
        let mut unique_3 = HashSet::<i32>::new();
        let mut unique_4 = HashSet::<i32>::new();
        let mut unique_5 = HashSet::<i32>::new();

        let asset_traits: Vec<_> = asset
            .traits
            .clone()
            .iter()
            .filter(|a| !ignored_trait_types_overlap.contains(&a))
            .cloned()
            .collect();

        for vpair in asset_traits.iter().combinations(3) {
            unique_3.extend(
                read_assets_with_traits(
                    &mut conn,
                    collection_slug,
                    vpair.into_iter().cloned().collect(),
                )
                .await?,
            );
        }

        for vpair in asset_traits.iter().combinations(4) {
            unique_4.extend(
                read_assets_with_traits(
                    &mut conn,
                    collection_slug,
                    vpair.into_iter().cloned().collect(),
                )
                .await?,
            );
        }

        for vpair in asset_traits.iter().combinations(5) {
            unique_5.extend(
                read_assets_with_traits(
                    &mut conn,
                    collection_slug,
                    vpair.into_iter().cloned().collect(),
                )
                .await?,
            );
        }

        asset.traits_3_combination_overlap = unique_3.len() as i32;
        asset.traits_4_combination_overlap = unique_4.len() as i32;
        asset.traits_5_combination_overlap = unique_5.len() as i32;
        asset.traits_3_combination_overlap_ids = unique_3.into_iter().collect::<Vec<_>>();
        asset.traits_4_combination_overlap_ids = unique_4.into_iter().collect::<Vec<_>>();
        asset.traits_5_combination_overlap_ids = unique_5.into_iter().collect::<Vec<_>>();

        res.push(asset.clone());
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::opensea::types::*;
    use crate::storage::establish_connection;
    #[tokio::test]
    async fn test_asset_process() {
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
        let pool = establish_connection().await;
        let mut conn = pool.acquire().await.unwrap();
        let a = process_assets(
            &mut conn,
            vec![asset1, asset2, asset3],
            "forgottenruneswizardscult",
        )
        .await
        .unwrap();
        println!("{}", serde_json::to_string_pretty(&a).unwrap());
        assert!(!a.is_empty());
    }
}
