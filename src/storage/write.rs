use super::Trait;
use crate::opensea::types::{Collection, Event};
use anyhow::Result;
use sqlx::postgres::PgQueryResult;
use sqlx::{Acquire, PgConnection};

// ============ ASSET ============
pub async fn write_asset(conn: &mut PgConnection, asset: &super::Asset) -> Result<PgQueryResult> {
    sqlx::query!(
        r#"
       insert into asset(
        name,
        collection_slug,
        token_id,
        image_url,
        owner,
        traits,
        unique_traits,
        traits_3_combination_overlap,
        traits_4_combination_overlap,
        traits_5_combination_overlap,
        traits_3_combination_overlap_ids,
        traits_4_combination_overlap_ids,
        traits_5_combination_overlap_ids
       )
       values
           ($1, $2, $3, $4,$5, $6, $7, $8, $9, $10, $11, $12, $13);
       "#,
        asset.name,
        asset.collection_slug.to_lowercase(),
        asset.token_id as i32,
        asset.image_url,
        asset.owner,
        &asset.traits,
        asset.unique_traits,
        asset.traits_3_combination_overlap,
        asset.traits_4_combination_overlap,
        asset.traits_5_combination_overlap,
        &asset.traits_3_combination_overlap_ids,
        &asset.traits_4_combination_overlap_ids,
        &asset.traits_5_combination_overlap_ids,
    )
    .execute(conn)
    .await
    .map_err(|e| e.into())
}

// ============ COLLECTION ============
pub async fn write_collection(
    conn: &mut PgConnection,
    collection: &Collection,
    avg_trait_rarity: f64,
) -> Result<PgQueryResult> {
    sqlx::query!(
        r#"
       insert into collection(
            slug,
            name,
            address,
            total_supply,
            rarity_cutoff,
            floor_price,
            avg_trait_rarity,
            banner_image_url,
            daily_volume,
            daily_sales,
            daily_avg_price,
            weekly_avg_price,
            monthly_avg_price,
            nr_owners
       )
       values
           ($1, $2, $3, $4, $5,$6, $7, $8, $9, $10, $11, $12, $13, $14);
       "#,
        collection.slug.to_lowercase(),
        collection.name.clone().unwrap_or_default(),
        collection.primary_asset_contracts[0].address.to_lowercase(),
        collection.stats.total_supply as i32,
        (avg_trait_rarity * 2.0) / 100f64,
        collection.stats.floor_price,
        avg_trait_rarity,
        collection.banner_image_url,
        collection.stats.daily_volume,
        collection.stats.daily_sales,
        collection.stats.daily_avg_price,
        collection.stats.weekly_avg_price,
        collection.stats.monthly_avg_price,
        collection.stats.nr_owners,
    )
    .execute(conn)
    .await
    .map_err(|e| e.into())
}

pub async fn update_collection_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    new_floor: f64,
) -> Result<PgQueryResult> {
    sqlx::query!(
        r#"
        update  collection
            set
            floor_price = $1
        where slug = $2
       "#,
        new_floor,
        collection_slug
    )
    .execute(conn)
    .await
    .map_err(|e| e.into())
}

pub async fn update_collection_rarity_cutoff(
    conn: &mut PgConnection,
    collection_slug: &str,
    rarity_cutoff: f64,
) -> Result<PgQueryResult> {
    sqlx::query!(
        r#"
        update collection
            set
            rarity_cutoff = $1
        where slug= $2
       "#,
        rarity_cutoff,
        collection_slug
    )
    .execute(conn)
    .await
    .map_err(|e| e.into())
}

// ============ TRAITS ============
pub async fn write_traits(conn: &mut PgConnection, collection: &Collection) -> Result<()> {
    let traits: Vec<Trait> = collection
        .traits
        .clone()
        .into_iter()
        .map(|(k, v)| {
            v.into_iter()
                .map(|(n, c)| Trait {
                    collection_slug: collection.slug.clone().to_lowercase(),
                    trait_type: k.clone().to_lowercase(),
                    trait_name: n.to_lowercase(),
                    trait_count: c as i32,
                })
                .collect::<Vec<Trait>>()
        })
        .collect::<Vec<Vec<Trait>>>()
        .into_iter()
        .flatten()
        .collect();

    let mut txn = conn.begin().await?;
    for t in traits {
        sqlx::query!(
            r#"
        insert into trait(
               collection_slug,
               trait_type,
               trait_name,
               trait_count
        )
        values
            ($1, $2, $3, $4)
        "#,
            t.collection_slug,
            t.trait_type,
            t.trait_name,
            t.trait_count
        )
        .execute(&mut txn)
        .await?;
    }
    txn.commit().await.map_err(|e| e.into())
}

// ============ EVENTS ============
pub async fn write_sale(
    conn: &mut PgConnection,
    sale: &Event,
    collection_slug: &str,
) -> Result<()> {
    let token_id = if sale.asset.is_some() {
        sale.asset.as_ref().unwrap().token_id as i32
    } else {
        // Bundles dont have an asset, we ignore bundle sales
        return Ok(());
    };
    sqlx::query!(
        r#"
       insert into sale(
        collection_slug,
        token_id,
        price,
        timestamp
       )
       values
           ($1, $2, $3, $4);
       "#,
        collection_slug.to_lowercase(),
        token_id,
        sale.total_price.clone().unwrap().parse::<f64>().unwrap(),
        sale.created_date.timestamp() as i32,
    )
    .execute(conn)
    .await?;
    Ok(())
}

// ============ LISTINGS ============
pub async fn write_listing(
    conn: &mut PgConnection,
    collection_slug: &str,
    update_type: &str,
    token_id: i32,
    price: Option<f64>,
    timestamp: i32,
) -> Result<()> {
    sqlx::query!(
        r#"
       insert into LISTING(
        collection_slug,
        token_id,
        price,
        timestamp,
        update_type
       )
       values
           ($1, $2, $3, $4, $5);
       "#,
        collection_slug.to_lowercase(),
        token_id,
        price,
        timestamp, //Utc::now().timestamp() as i32,
        update_type
    )
    .execute(conn)
    .await?;
    Ok(())
}
