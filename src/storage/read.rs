use super::*;
use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgConnection;

use std::collections::HashMap;

// ============ Collection ============
pub async fn read_collection(conn: &mut PgConnection, slug: &str) -> Result<Collection> {
    sqlx::query_as!(
        Collection,
        r#"
            select
                * 
            from
                collection c     
            where c.slug = $1
            
        "#,
        slug,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_all_collections(conn: &mut PgConnection) -> Result<Vec<String>> {
    sqlx::query_scalar!(
        r#"
            select
                slug 
            from
                collection
        "#,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.into())
}

// ============ Trait ============
pub async fn read_trait(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<Trait> {
    sqlx::query_as!(
        Trait,
        r#"
            select
                * 
            from
                trait t     
            where t.collection_slug = $1 and t.trait_name = $2
            
        "#,
        collection_slug,
        trait_name,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_traits_for_asset(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<HashMap<String, i32>> {
    let mut res = HashMap::new();
    let vals = sqlx::query!(
        r#"
            select
                t.*
            from
                trait as t join (
                    select * from asset where token_id = $1
                ) as a
                on
                    t.trait_name = any(a.traits)
            where t.collection_slug = $2
        "#,
        token_id,
        collection_slug,
    )
    .map(|r| (r.trait_name, r.trait_count))
    .fetch_all(&mut *conn)
    .await?;
    for (k, v) in vals {
        res.insert(k, v);
    }
    Ok(res)
}

// ============ Asset ============
pub async fn read_asset(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<Asset> {
    sqlx::query_as!(
        Asset,
        r#"
            select
                * 
            from
                asset a     
            where a.collection_slug = $1 and a.token_id = $2
            
        "#,
        collection_slug,
        token_id,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_assets_with_trait(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<Vec<Asset>> {
    sqlx::query_as!(
        Asset,
        r#"
            select
                * 
            from
                asset a     
            where a.collection_slug = $1 and  $2 = any(a.traits)
            
        "#,
        collection_slug,
        trait_name,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_assets_for_owner(
    conn: &mut PgConnection,
    collection_slug: &str,
    owner_address: &str,
) -> Result<Option<i64>> {
    sqlx::query_scalar!(
        r#"
            select
                count(*) 
            from
                asset a     
            where a.collection_slug = $1 and  a.owner = $2 
            
        "#,
        collection_slug,
        owner_address,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.into())
}

// ============ Sales ============
pub async fn read_sales_for_trait(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<Vec<SaleEvent>> {
    sqlx::query_as!(
        SaleEvent,
        r#"
            select
                *
            from
                sale
            where collection_slug = $1 and token_id = any(
                select
                    token_id
                from
                    asset a     
                where a.collection_slug = $1 and  $2 = any(a.traits)
            )
        "#,
        collection_slug,
        trait_name,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_latest_sale_for_collection(
    conn: &mut PgConnection,
    collection_slug: &str,
) -> Result<i32> {
    sqlx::query_scalar!(
        r#"
                select
                    distinct(timestamp)
                from
                    sale
                where collection_slug = $1 
                order by timestamp desc
            "#,
        collection_slug,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_highest_sale_for_collection(
    conn: &mut PgConnection,
    collection_slug: &str,
) -> Result<i32> {
    sqlx::query_scalar!(
        r#"
                select
                    token_id
                from
                    sale
                where collection_slug = $1 
                group by token_id, price order by price desc
            "#,
        collection_slug,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_sales_for_asset(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<Vec<SaleEvent>> {
    sqlx::query_as!(
        SaleEvent,
        r#"
            select
                *
            from
                sale
            where collection_slug = $1 and token_id = $2
        "#,
        collection_slug,
        token_id,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_avg_price_collection_at_ts(
    conn: &mut PgConnection,
    collection_slug: &str,
    timestamp: &NaiveDateTime,
) -> Result<Option<f64>> {
    let avg = sqlx::query_scalar!(
        r#"
            select
                avg(price) 
            from
                sale
            where collection_slug = $1 and timestamp < $2
        "#,
        collection_slug,
        timestamp.timestamp() as i32,
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(avg)
}

pub async fn read_avg_price_trait_at_ts(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
    timestamp: &NaiveDateTime,
) -> Result<Option<f64>> {
    sqlx::query_scalar!(
        r#"
            select
                avg(price)
            from
                sale
            where collection_slug = $1 and timestamp < $2 and token_id = any(
                select
                    token_id
                from
                    asset a     
                where a.collection_slug = $1 and  $3 = any(a.traits)
            )
        "#,
        collection_slug,
        timestamp.timestamp() as i32,
        trait_name,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.into())
}

// ============ Listings ============
pub async fn read_latests_listing_for_asset(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<Vec<Listing>> {
    sqlx::query_as!(
        Listing,
        r#"
            select
                *
            from
                listing
            where collection_slug = $1 and token_id = $2
            order by timestamp desc
        "#,
        collection_slug,
        token_id,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_latests_listing_for_collection(
    conn: &mut PgConnection,
    collection_slug: &str,
) -> Result<i32> {
    sqlx::query_scalar!(
        r#"
            select
                distinct(timestamp)
            from
                listing
            where collection_slug = $1 
            order by timestamp desc
        "#,
        collection_slug,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_listed_for_collection_at_ts(
    conn: &mut PgConnection,
    collection_slug: &str,
    timestamp: &NaiveDateTime,
) -> Result<Vec<Listing>> {
    sqlx::query_as!(
        Listing,
        r#"
        select 
            distinct on (token_id) *
        from 
            listing
        where collection_slug = $1 and timestamp < $2 and price is not null
        order by token_id, timestamp desc

        "#,
        collection_slug,
        timestamp.timestamp() as i32,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_listing_update_type_count_after_ts(
    conn: &mut PgConnection,
    collection_slug: &str,
    update_type: &str,
    timestamp: &NaiveDateTime,
) -> Result<Option<i64>> {
    sqlx::query_scalar!(
        r#"
            select
                count(distinct(token_id))
            from
                listing
            where collection_slug = $1 and update_type = $2 and timestamp > $3
        "#,
        collection_slug,
        update_type,
        timestamp.timestamp() as i32,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.into())
}

pub async fn read_trait_listings_at_ts(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
    ts: i32,
) -> Result<Vec<Listing>> {
    sqlx::query_as!(
        Listing,
        r#"
            select
                *
            from
            listing  
            where collection_slug = $1 and token_id = any( select
                token_id
                from
                    asset a     
                where a.collection_slug = $1 and  $2 = any(a.traits)) and timestamp < $3
            order by timestamp desc
        "#,
        collection_slug,
        trait_name,
        ts
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.into())
}
