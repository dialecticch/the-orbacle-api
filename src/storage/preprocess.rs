use super::read::read_collection;
use super::Asset;
use crate::opensea::types::Asset as OpenseaAsset;
use anyhow::Result;
use itertools::Itertools;
use sqlx::PgConnection;

pub async fn process_assets(
    conn: &mut PgConnection,
    os_assets: Vec<OpenseaAsset>,
    collection_slug: &str,
) -> Result<Vec<Asset>> {
    let collection = read_collection(conn, collection_slug).await?;

    let mut assets: Vec<Asset> = vec![];
    let mut max_score = f64::MIN;
    for asset in os_assets {
        let trait_names = asset
            .traits
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|t| t.value.to_lowercase())
            .collect::<Vec<String>>();

        let mut rarity_score = 1f64;
        asset
            .traits
            .clone()
            .unwrap_or_default()
            .iter()
            .filter(|t| t.value.to_lowercase() == "serial")
            .map(|t| {
                rarity_score = rarity_score
                    * (t.trait_count.unwrap_or(collection.total_supply as u64) as f64
                        / collection.total_supply as f64)
            })
            .for_each(drop);

        if rarity_score > max_score {
            max_score = rarity_score
        }

        let unique_traits = asset
            .traits
            .clone()
            .unwrap_or_default()
            .iter()
            .filter(|t| t.trait_count.unwrap_or_default() == 1)
            .count();

        assets.push(Asset {
            name: asset.name,
            collection_slug: collection_slug.to_string(),
            token_id: asset.token_id as i32,
            image_url: asset.image_url,
            owner: asset.owner.address,
            traits: trait_names,
            rarity_score: rarity_score,
            unique_traits: unique_traits as i32,
            unique_3_trait_combinations: 0i32,
            unique_4_trait_combinations: 0i32,
            unique_5_trait_combinations: 0i32,
        });
    }
    println!("base processing done");

    let mut res = vec![];

    let chunks: Vec<_> = assets.chunks(16).collect();
    let mut handlers = vec![];
    for c in chunks {
        let collection = assets.clone();
        let list = c.clone().to_vec();
        handlers.push(std::thread::spawn(move || {
            return compute_combinations(list, collection, max_score).unwrap();
        }))
    }

    for h in handlers {
        res.extend(h.join().unwrap())
    }

    Ok(res)
}

fn compute_combinations(
    assets: Vec<Asset>,
    collection: Vec<Asset>,
    max_score: f64,
) -> Result<Vec<Asset>> {
    let mut res = vec![];
    for mut asset in assets {
        println!("{}", asset.token_id);
        asset.rarity_score = max_score / asset.rarity_score;
        let mut unique_3 = 0;
        let mut unique_4 = 0;
        let mut unique_5 = 0;
        for vpair in asset.traits.iter().combinations(5) {
            unique_5 += collection
                .iter()
                .filter(|a| {
                    a.traits.contains(vpair[0])
                        && a.traits.contains(vpair[1])
                        && a.traits.contains(vpair[2])
                        && a.traits.contains(vpair[3])
                        && a.traits.contains(vpair[4])
                })
                .count();
        }

        for vpair in asset.traits.iter().combinations(3) {
            unique_3 += collection
                .iter()
                .filter(|a| {
                    a.traits.contains(vpair[0])
                        && a.traits.contains(vpair[1])
                        && a.traits.contains(vpair[2])
                })
                .count();
        }

        for vpair in asset.traits.iter().combinations(4) {
            unique_4 += collection
                .iter()
                .filter(|a| {
                    a.traits.contains(vpair[0])
                        && a.traits.contains(vpair[1])
                        && a.traits.contains(vpair[2])
                        && a.traits.contains(vpair[3])
                })
                .count();
        }

        asset.unique_5_trait_combinations = unique_5 as i32;
        asset.unique_3_trait_combinations = unique_3 as i32;
        asset.unique_4_trait_combinations = unique_4 as i32;

        res.push(asset.clone());
    }

    Ok(res)
}
