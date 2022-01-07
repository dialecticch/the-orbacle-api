use super::read::read_collection;
use super::Asset;
use crate::opensea::types::Asset as OpenseaAsset;
use anyhow::Result;
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

        let trait_names = trait_list
            .iter()
            .map(|t| t.value.to_lowercase())
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
            traits: trait_names,
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

    let mut res = vec![];

    let chunks: Vec<_> = assets.chunks(16).collect();
    let mut handlers = vec![];
    for c in chunks {
        let collection = assets.clone();
        let list = c.to_vec();
        handlers.push(std::thread::spawn(move || {
            compute_combinations(list.clone(), collection).unwrap()
        }))
    }

    for h in handlers {
        res.extend(h.join().unwrap())
    }

    Ok(res)
}

fn compute_combinations(assets: Vec<Asset>, collection: Vec<Asset>) -> Result<Vec<Asset>> {
    let mut res = vec![];
    for mut asset in assets {
        let mut unique_3 = HashSet::<i32>::new();
        let mut unique_4 = HashSet::<i32>::new();
        let mut unique_5 = HashSet::<i32>::new();

        for vpair in asset.traits.iter().combinations(3) {
            unique_3.extend(
                collection
                    .iter()
                    .filter(|a| a.token_id != asset.token_id)
                    .filter(|a| {
                        a.traits.contains(vpair[0])
                            && a.traits.contains(vpair[1])
                            && a.traits.contains(vpair[2])
                    })
                    .map(|a| a.token_id)
                    .collect::<Vec<_>>(),
            );
        }

        for vpair in asset.traits.iter().combinations(4) {
            unique_4.extend(
                collection
                    .iter()
                    .filter(|a| a.token_id != asset.token_id)
                    .filter(|a| {
                        a.traits.contains(vpair[0])
                            && a.traits.contains(vpair[1])
                            && a.traits.contains(vpair[2])
                            && a.traits.contains(vpair[3])
                    })
                    .map(|a| a.token_id)
                    .collect::<Vec<_>>(),
            );
        }

        for vpair in asset.traits.iter().combinations(5) {
            unique_5.extend(
                collection
                    .iter()
                    .filter(|a| a.token_id != asset.token_id)
                    .filter(|a| {
                        a.traits.contains(vpair[0])
                            && a.traits.contains(vpair[1])
                            && a.traits.contains(vpair[2])
                            && a.traits.contains(vpair[3])
                            && a.traits.contains(vpair[4])
                    })
                    .map(|a| a.token_id)
                    .collect::<Vec<_>>(),
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
