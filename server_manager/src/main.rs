#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use log::{error, LevelFilter};
use server_manager::core::exit_codes::{map_error_to_exit_code, EX_OK};
use server_manager::interface::cli;

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    if let Err(err) = cli::run().await {
        error!("Command execution failed: {:#}", err);
        let code = map_error_to_exit_code(&err);
        std::process::exit(code);
    }
    std::process::exit(EX_OK);
}
