use common::CoreError;

pub fn map_sqlx_error(error: sqlx::Error) -> CoreError {
    match &error {
        sqlx::Error::RowNotFound => CoreError::NotFound,
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            CoreError::Conflict(db_err.message().to_string())
        }
        _ => CoreError::Database(error.to_string()),
    }
}
