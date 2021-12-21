use super::{Asset, Collection, Trait};
use anyhow::Result;
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
