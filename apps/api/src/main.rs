pub mod auth;
pub mod errors;
pub mod handlers;
pub mod logger;
pub mod openapi;
pub mod response;
pub mod router;
pub mod state;

use std::{error::Error, sync::Arc};

use args::Args;
use clap::Parser;
use dotenvy::dotenv;
use server::{get_addr, run_server};

use crate::{logger::init_tracing_and_logging, router::router, state::state};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let args = Arc::new(Args::parse());
    init_tracing_and_logging(&args.log, "oxid", &args.observability)?;

    let app_state = state(args.clone()).await?;

    let router = router(app_state)?;
    let addr = get_addr(&args.server.host, args.server.port).await?;

    run_server(addr, router).await;

    Ok(())
}
