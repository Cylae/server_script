use anyhow::Result;
use log::LevelFilter;
use server_manager::interface::cli;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    cli::run().await
}
