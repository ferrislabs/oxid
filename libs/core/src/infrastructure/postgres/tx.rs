use common::CoreError;
use sqlx::{PgPool, Postgres, Transaction};

use crate::infrastructure::postgres::error::map_sqlx_error;

pub async fn with_tx<F, T>(pool: &PgPool, work: F) -> Result<T, CoreError>
where
    F: AsyncFnOnce(&mut Transaction<'static, Postgres>) -> Result<T, CoreError>,
{
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;

    match work(&mut tx).await {
        Ok(value) => {
            tx.commit().await.map_err(map_sqlx_error)?;
            Ok(value)
        }
        Err(err) => {
            let _ = tx.rollback().await;
            Err(err)
        }
    }
}
