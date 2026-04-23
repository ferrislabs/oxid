pub mod errors;
pub mod state;

use std::error::Error;

use args::Args;
use clap::Parser;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let args = Args::parse();

    println!("Args: {:?}", args);

    Ok(())
}
