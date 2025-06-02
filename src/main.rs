#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(dead_code)]

mod auth;
pub mod cli;
pub mod commands;
mod launcher;
mod logger;

use crate::cli::{Cli, Commands};
use clap::Parser;
use dotenvy::dotenv;
use log::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize the logger with custom format
    logger::setup_logger();

    let cli = Cli::parse();

    info!("Rustified Minecraft Launcher v0.2.0");
    info!("=====================================");

    // Initialize the launcher
    info!("Initializing launcher...");
    let launcher = match launcher::Launcher::new().await {
        Ok(launcher) => {
            info!("✓ Launcher initialized successfully");
            launcher
        }
        Err(e) => {
            error!("Failed to initialize launcher: {e}");
            return Err(e);
        }
    };

    match cli.command {
        Commands::List {
            releases_only,
            limit,
        } => {
            commands::game::list_versions(&launcher, releases_only, limit).await?;
        }
        Commands::Launch {
            instance,
            skip_verification,
        } => {
            commands::game::launch_game(&launcher, &instance, skip_verification).await?;
        }
        Commands::Prepare { version } => {
            commands::game::prepare_game(&launcher, &version).await?;
        }
        Commands::Auth { action } => {
            commands::auth::handle_auth_command(action).await?;
        }
        Commands::Instance { action } => {
            commands::instance::handle_instance_command(&launcher, action).await?;
        }
        Commands::Java { action } => {
            commands::java::handle_java_command(&launcher, action);
        }
    }

    Ok(())
}
