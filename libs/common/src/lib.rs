use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Clone, Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub issuer: String,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("resource not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub fn generate_timestamp() -> (DateTime<Utc>, Timestamp) {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);

    let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

    (now, timestamp)
}

pub fn generate_uuid_v7() -> Uuid {
    let (_, timestamp) = generate_timestamp();
    Uuid::new_v7(timestamp)
}
