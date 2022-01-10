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

pub async fn update_asset_overlap(
    conn: &mut PgConnection,
    asset: &super::Asset,
) -> Result<PgQueryResult> {
    sqlx::query!(
        r#"
       update asset
       set
        traits_3_combination_overlap = $3,
        traits_4_combination_overlap = $4,
        traits_5_combination_overlap = $5,
        traits_3_combination_overlap_ids = $6,
        traits_4_combination_overlap_ids = $7,
        traits_5_combination_overlap_ids = $8
    where collection_slug = $1 and token_id = $2
       
       "#,
        asset.collection_slug.to_lowercase(),
        asset.token_id as i32,
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
    multiplier: f64,
    ignored_trait_types_rarity: Vec<String>,
    ignored_trait_types_overlap: Vec<String>,
) -> Result<PgQueryResult> {
    sqlx::query!(
        r#"
       insert into collection(
            slug,
            name,
            address,
            ignored_trait_types_rarity,
            ignored_trait_types_overlap,
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
           ($1, $2, $3, $4, $5,$6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16);
       "#,
        collection.slug.to_lowercase(),
        collection.name.clone().unwrap_or_default(),
        collection.primary_asset_contracts[0].address.to_lowercase(),
        &ignored_trait_types_rarity,
        &ignored_trait_types_overlap,
        collection.stats.total_supply as i32,
        (avg_trait_rarity * multiplier) / collection.stats.total_supply,
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

pub async fn update_collection_info(
    conn: &mut PgConnection,
    collection_slug: &str,
    total_supply: f64,
    ignored_trait_types_rarity: Vec<String>,
    ignored_trait_types_overlap: Vec<String>,
    rarity_cutoff: f64,
) -> Result<PgQueryResult> {
    sqlx::query!(
        r#"
        update collection
            set
            ignored_trait_types_rarity = $1,
            ignored_trait_types_overlap = $2,
            rarity_cutoff = $3,
            total_supply = $4
        where slug= $5
       "#,
        &ignored_trait_types_rarity,
        &ignored_trait_types_overlap,
        rarity_cutoff,
        total_supply as i32,
        collection_slug
    )
    .execute(conn)
    .await
    .map_err(|e| e.into())
}

// ============ TRAITS ============
pub async fn write_traits(conn: &mut PgConnection, traits: Vec<super::Trait>) -> Result<()> {
    let mut txn = conn.begin().await?;
    for t in traits {
        sqlx::query!(
            r#"
        insert into trait(
               collection_slug,
               trait_id,
               trait_type,
               trait_name,
               trait_count
        )
        values
            ($1, $2, $3, $4, $5)
        "#,
            t.collection_slug,
            t.trait_id,
            t.trait_type,
            t.trait_name,
            t.trait_count
        )
        .execute(&mut txn)
        .await?;
    }
    txn.commit().await.map_err(|e| e.into())
}

pub async fn update_traits(
    conn: &mut PgConnection,
    collection_slug: &str,
    traits: Vec<super::Trait>,
) -> Result<()> {
    let mut txn = conn.begin().await?;

    sqlx::query!(
        r#"
        delete from trait where collection_slug = $1
    "#,
        &collection_slug,
    )
    .execute(&mut txn)
    .await?;

    for t in traits {
        sqlx::query!(
            r#"
        insert into trait(
               collection_slug,
               trait_id,
               trait_type,
               trait_name,
               trait_count
        )
        values
            ($1, $2, $3, $4, $5)
        "#,
            t.collection_slug,
            t.trait_id,
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
