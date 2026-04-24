pub mod errors;
pub mod logger;
pub mod state;

use std::{error::Error, sync::Arc};

use args::Args;
use clap::Parser;
use dotenvy::dotenv;

use crate::logger::init_tracing_and_logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let args = Arc::new(Args::parse());

    init_tracing_and_logging(&args.log, "oxid", &args.observability)?;

    Ok(())
}
