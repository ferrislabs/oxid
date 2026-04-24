use clap::Parser;

use crate::{
    auth::AuthArgs, database::DatabaseArgs, log::LogArgs, observability::ObservabilityArgs,
    server::ServerArgs,
};

pub mod auth;
pub mod database;
pub mod log;
pub mod observability;
pub mod server;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[command(flatten)]
    pub log: LogArgs,

    #[command(flatten)]
    pub db: DatabaseArgs,

    #[command(flatten)]
    pub auth: AuthArgs,

    #[command(flatten)]
    pub server: ServerArgs,

    #[command(flatten)]
    pub observability: ObservabilityArgs,
}
