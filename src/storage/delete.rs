use anyhow::Result;
use sqlx::{Acquire, PgConnection};

// ============ COLLECTION ============
pub async fn purge_collection(conn: &mut PgConnection, collection: &str) -> Result<()> {
    let mut txn = conn.begin().await?;

    sqlx::query!(
        r#"
       delete from collection where slug = $1;
       "#,
        collection
    )
    .execute(&mut txn)
    .await?;

    sqlx::query!(
        r#"
       delete from trait where collection_slug = $1;
       "#,
        collection
    )
    .execute(&mut txn)
    .await?;

    sqlx::query!(
        r#"
       delete from asset where collection_slug = $1;
       "#,
        collection
    )
    .execute(&mut txn)
    .await?;

    sqlx::query!(
        r#"
       delete from sale where collection_slug = $1; 
       "#,
        collection
    )
    .execute(&mut txn)
    .await?;

    sqlx::query!(
        r#"
       delete from listing where collection_slug = $1; 
       "#,
        collection
    )
    .execute(&mut txn)
    .await?;

    txn.commit().await.map_err(|e| e.into())
}
