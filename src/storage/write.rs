use super::Trait;
use crate::opensea::types::{Asset, Collection, Event};
use anyhow::Result;
use chrono::prelude::Utc;
use sqlx::postgres::PgQueryResult;
use sqlx::{Acquire, PgConnection};

// ============ ASSET ============
pub async fn write_asset(
    conn: &mut PgConnection,
    asset: &Asset,
    collection_slug: &str,
) -> Result<PgQueryResult> {
    let trait_names = asset
        .traits
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.value.to_lowercase())
        .collect::<Vec<String>>();
    sqlx::query!(
        r#"
       insert into asset(
        collection_slug,
        token_id,
        traits
       )
       values
           ($1, $2, $3);
       "#,
        collection_slug.to_lowercase(),
        asset.token_id as i32,
        &trait_names,
    )
    .execute(conn)
    .await
    .map_err(|e| e.into())
}

// ============ COLLECTION ============
pub async fn write_collection(
    conn: &mut PgConnection,
    collection: &Collection,
) -> Result<PgQueryResult> {
    sqlx::query!(
        r#"
       insert into collection(
            slug,
            address,
            total_supply
       )
       values
           ($1, $2, $3);
       "#,
        collection.slug.to_lowercase(),
        collection.primary_asset_contracts[0].address.to_lowercase(),
        collection.stats.total_supply as i32,
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
        sale.total_price as f64,
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
    token_id: i32,
    price: Option<f64>,
) -> Result<()> {
    sqlx::query!(
        r#"
       insert into LISTING(
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
        price,
        Utc::now().timestamp() as i32,
    )
    .execute(conn)
    .await?;
    Ok(())
}
