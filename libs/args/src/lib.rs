use clap::Parser;

use crate::log::LogArgs;

pub mod auth;
pub mod database;
pub mod log;
pub mod server;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[command(flatten)]
    pub lgo: LogArgs,
}
