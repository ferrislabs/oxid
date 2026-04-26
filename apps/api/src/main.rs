pub mod auth;
pub mod errors;
pub mod handlers;
pub mod internal_router;
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
use tracing::info;

use crate::{
    internal_router::internal_router, logger::init_tracing_and_logging, router::router,
    state::state,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let args = Arc::new(Args::parse());
    init_tracing_and_logging(&args.log, "oxid", &args.observability)?;

    let app_state = state(args.clone()).await?;

    let router = router(app_state)?;
    let internal_router = internal_router()?;

    let addr = get_addr(&args.server.host, args.server.port).await?;
    let internal_addr = get_addr(&args.server.host, args.server.internal_port).await?;

    info!(%addr, %internal_addr, "starting api and internal http servers");

    tokio::join!(
        run_server(addr, router),
        run_server(internal_addr, internal_router),
    );

    Ok(())
}
