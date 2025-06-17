//! Rustified Minecraft Launcher
//!
//! A command-line Minecraft launcher written in Rust.

#![deny(clippy::too_many_lines, clippy::panic)]

mod auth;
pub mod cli;
pub mod commands;
pub mod error;
mod launcher;
mod logger;

use crate::cli::{Cli, Commands};
use clap::Parser;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> crate::error::Result<()> {
    // Initialize the logger with custom format
    logger::init();

    let cli = Cli::parse();

    info!("Rustified Minecraft Launcher v{}", env!("CARGO_PKG_VERSION"));
    info!("===================================");

    // Initialize the launcher
    debug!("Initializing launcher...");
    let launcher = match launcher::Launcher::new().await {
        Ok(launcher) => {
            debug!("✓ Launcher initialized successfully");
            launcher
        }
        Err(e) => {
            error!("Failed to initialize launcher: {e}");
            return Err(e);
        }
    };

    match cli.command {
        Commands::List {
            types,
            releases_only,
            snapshots_only,
            limit,
            filter,
            show_installed,
            sort,
        } => {
            let options = commands::game::ListVersionsOptions {
                types,
                releases_only,
                snapshots_only,
                limit,
                filter,
                show_installed,
                sort,
            };
            commands::game::list_versions(&launcher, options).await?;
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
