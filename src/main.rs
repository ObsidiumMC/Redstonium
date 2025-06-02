#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(dead_code)]

mod auth;
mod launcher;

use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use env_logger::{Builder, Env};
use log::{LevelFilter, error, info};
use std::io::Write;
use time::{format_description::FormatItem, macros::format_description};

const LOG_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");

#[derive(Parser)]
#[command(name = "rustified")]
#[command(about = "A Minecraft CLI launcher written in Rust")]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available Minecraft versions
    List {
        /// Show only release versions
        #[arg(long)]
        releases_only: bool,
        /// Maximum number of versions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Launch a Minecraft instance
    Launch {
        /// Instance to launch
        instance: String,
        /// Skip file verification (faster launch)
        #[arg(long)]
        skip_verification: bool,
    },
    /// Prepare (download) a Minecraft version without launching
    Prepare {
        /// Version to prepare
        version: String,
    },
    /// Authentication management
    Auth {
        #[command(subcommand)]
        action: AuthCommands,
    },
    /// Instance management
    Instance {
        #[command(subcommand)]
        action: InstanceCommands,
    },
    /// Java runtime management
    Java {
        #[command(subcommand)]
        action: JavaCommands,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Check authentication status
    Status,
    /// Clear cached authentication
    Clear,
    /// Force re-authentication
    Refresh,
}

#[derive(Subcommand)]
enum InstanceCommands {
    /// List all instances
    List,
    /// Create a new instance
    Create {
        /// Instance name
        name: String,
        /// Minecraft version
        version: String,
        /// Instance description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete an instance
    Delete {
        /// Instance name
        name: String,
    },
    /// Show instance details
    Info {
        /// Instance name
        name: String,
    },
    /// Set instance memory allocation
    Memory {
        /// Instance name
        name: String,
        /// Memory in MB
        memory: u32,
    },
}

#[derive(Subcommand)]
enum JavaCommands {
    /// List available Java installations
    List,
    /// Show recommended Java version for a Minecraft version
    Recommend {
        /// Minecraft version
        version: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize the logger with custom format
    setup_logger();

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
            list_versions(&launcher, releases_only, limit).await?;
        }
        Commands::Launch {
            instance,
            skip_verification,
        } => {
            launch_game(&launcher, &instance, skip_verification).await?;
        }
        Commands::Prepare { version } => {
            prepare_game(&launcher, &version).await?;
        }
        Commands::Auth { action } => {
            handle_auth_command(action).await?;
        }
        Commands::Instance { action } => {
            handle_instance_command(&launcher, action).await?;
        }
        Commands::Java { action } => {
            handle_java_command(&launcher, action);
        }
    }

    Ok(())
}

async fn list_versions(
    launcher: &launcher::Launcher,
    releases_only: bool,
    limit: usize,
) -> anyhow::Result<()> {
    info!("Fetching available Minecraft versions...");
    let manifest = launcher.file_manager.get_version_manifest().await?;

    let mut versions = manifest.versions.clone();

    if releases_only {
        versions.retain(|v| matches!(v.version_type, launcher::VersionType::Release));
    }

    versions.truncate(limit);

    info!("Available Minecraft versions:");
    info!("Latest release: {}", manifest.latest.release);
    info!("Latest snapshot: {}", manifest.latest.snapshot);
    info!("");

    for version in &versions {
        let installed_marker = if launcher.minecraft_dir.is_version_installed(&version.id) {
            "✓ "
        } else {
            "  "
        };

        info!(
            "{}{} ({:?}) - {}",
            installed_marker, version.id, version.version_type, version.release_time
        );
    }

    Ok(())
}

async fn prepare_game(launcher: &launcher::Launcher, version: &str) -> anyhow::Result<()> {
    let resolved_version = resolve_version_alias(launcher, version).await?;

    info!("Preparing Minecraft {resolved_version} (no authentication required)...");

    // Get version info and download files without authentication
    let version_info = launcher
        .file_manager
        .get_version_info(&resolved_version)
        .await?;

    // Ensure version directory exists
    launcher
        .minecraft_dir
        .ensure_version_dir(&resolved_version)?;

    // Download main game JAR
    launcher
        .file_manager
        .download_game_jar(&version_info, &launcher.minecraft_dir)
        .await?;

    // Download libraries
    launcher
        .file_manager
        .download_libraries(&version_info, &launcher.minecraft_dir)
        .await?;

    // Download assets
    launcher
        .file_manager
        .download_assets(&version_info, &launcher.minecraft_dir)
        .await?;

    info!("✓ Minecraft {resolved_version} prepared successfully");
    Ok(())
}

async fn launch_game(
    launcher: &launcher::Launcher,
    instance_name: &str,
    _skip_verification: bool,
) -> anyhow::Result<()> {
    // Get the instance configuration and extract the version
    let (instance_config, version) = {
        let instance_manager = launcher.instance_manager.lock().await;
        if let Some(config) = instance_manager.get_instance(instance_name) {
            let config_clone = config.clone();
            let version = config.version.clone();
            (Some(config_clone), version)
        } else {
            return Err(anyhow::anyhow!(
                "Instance '{}' does not exist. Use 'rustified instance list' to see available instances or 'rustified instance create' to create one.",
                instance_name
            ));
        }
    };

    let resolved_version = resolve_version_alias(launcher, &version).await?;

    // Validate Minecraft version before authentication
    if let Err(e) = launcher
        .file_manager
        .get_version_info(&resolved_version)
        .await
    {
        error!("Invalid Minecraft version: {resolved_version} : {e}");
        return Err(anyhow::anyhow!(
            "Instance '{}' uses an invalid Minecraft version ('{}'). Use 'rustified list' to see valid versions.",
            instance_name,
            resolved_version
        ));
    }

    // Update last used timestamp
    {
        let mut instance_manager = launcher.instance_manager.lock().await;
        instance_manager.update_last_used(instance_name).await?;
    }

    info!("Launching Minecraft {resolved_version} with instance '{instance_name}'...");

    // Authenticate first
    info!("Starting authentication process...");
    let auth_result = match auth::authenticate().await {
        Ok(result) => {
            info!("Authentication successful!");
            info!("Welcome, {}!", result.profile.name);
            result
        }
        Err(e) => {
            error!("Authentication failed: {e}");
            return Err(e);
        }
    };

    // Prepare the game (download if necessary)
    info!("Preparing game files...");
    launcher
        .prepare_game(&resolved_version, &auth_result)
        .await?;
    info!("✓ Game files prepared successfully");

    // Launch the game
    info!("Starting Minecraft {resolved_version}...");

    launcher
        .launch_game(&resolved_version, &auth_result, instance_config.as_ref())
        .await?;
    info!("✓ Minecraft exited");

    Ok(())
}

async fn resolve_version_alias(
    launcher: &launcher::Launcher,
    version: &str,
) -> anyhow::Result<String> {
    match version {
        "latest-release" | "latest" => {
            let manifest = launcher.file_manager.get_version_manifest().await?;
            Ok(manifest.latest.release)
        }
        "latest-snapshot" => {
            let manifest = launcher.file_manager.get_version_manifest().await?;
            Ok(manifest.latest.snapshot)
        }
        _ => Ok(version.to_string()),
    }
}

async fn handle_auth_command(action: AuthCommands) -> anyhow::Result<()> {
    let storage = auth::storage::AuthStorage::new()?;

    match action {
        AuthCommands::Status => {
            if let Some(cached_auth) = storage.load_auth().await? {
                info!("✓ Authentication: Valid");
                info!("  Player: {}", cached_auth.profile.name);
                info!("  UUID: {}", cached_auth.profile.id);
                // Don't log the token for security
            } else {
                info!("❌ No valid authentication found");
                info!("  Run 'rustified launch <instance>' to authenticate");
            }
        }
        AuthCommands::Clear => {
            storage.clear_cache().await?;
            info!("✓ Authentication cache cleared");
        }
        AuthCommands::Refresh => {
            info!("Clearing cache and forcing re-authentication...");
            storage.clear_cache().await?;
            let auth_result = auth::authenticate().await?;
            info!(
                "✓ Re-authentication successful for {}",
                auth_result.profile.name
            );
        }
    }

    Ok(())
}

async fn handle_instance_command(
    launcher: &launcher::Launcher,
    action: InstanceCommands,
) -> anyhow::Result<()> {
    match action {
        InstanceCommands::List => {
            let instance_manager = launcher.instance_manager.lock().await;
            let instances: Vec<_> = instance_manager
                .list_instances()
                .into_iter()
                .cloned()
                .collect();
            drop(instance_manager); // Release lock early

            if instances.is_empty() {
                info!(
                    "No instances found. Create one with: rustified instance create <name> <version>"
                );
            } else {
                info!("Available instances:");
                for instance in instances {
                    let last_used = if let Some(used) = instance.last_used {
                        format!(" (last used: {})", used.format("%Y-%m-%d %H:%M:%S"))
                    } else {
                        String::new()
                    };

                    let description = instance
                        .description
                        .as_ref()
                        .map(|d| format!(" - {d}"))
                        .unwrap_or_default();

                    info!(
                        "  {} (v{}){}{}",
                        instance.name, instance.version, description, last_used
                    );
                }
            }
        }
        InstanceCommands::Info { name } => {
            let instance_manager = launcher.instance_manager.lock().await;
            if let Some(instance) = instance_manager.get_instance(&name) {
                let instance = instance.clone(); // Clone to avoid borrow issues
                drop(instance_manager); // Release lock

                info!("Instance: {}", instance.name);
                info!("  Version: {}", instance.version);
                if let Some(desc) = &instance.description {
                    info!("  Description: {desc}");
                }
                info!(
                    "  Created: {}",
                    instance.created.format("%Y-%m-%d %H:%M:%S")
                );
                if let Some(used) = instance.last_used {
                    info!("  Last used: {}", used.format("%Y-%m-%d %H:%M:%S"));
                }
                info!("  Mod loader: {:?}", instance.mods.loader);
                if let Some(memory) = instance.settings.memory_mb {
                    info!("  Memory: {memory}MB");
                }
                if !instance.settings.java_args.is_empty() {
                    info!("  Java args: {}", instance.settings.java_args.join(" "));
                }
            } else {
                error!("Instance '{name}' does not exist");
                return Err(anyhow::anyhow!("Instance not found"));
            }
        }
        InstanceCommands::Create {
            name,
            version,
            description,
        } => {
            let mut instance_manager = launcher.instance_manager.lock().await;
            instance_manager
                .create_instance(name.clone(), version, description, &launcher.file_manager)
                .await?;
            info!("✓ Created instance '{name}'");
        }
        InstanceCommands::Delete { name } => {
            let mut instance_manager = launcher.instance_manager.lock().await;
            instance_manager.delete_instance(&name).await?;
            info!("✓ Deleted instance '{name}'");
        }
        InstanceCommands::Memory { name, memory } => {
            let mut instance_manager = launcher.instance_manager.lock().await;
            instance_manager.set_instance_memory(&name, memory).await?;
            info!("✓ Set memory for instance '{name}' to {memory}MB");
        }
    }
    Ok(())
}

fn handle_java_command(launcher: &launcher::Launcher, action: JavaCommands) {
    match action {
        JavaCommands::List => {
            let installations = &launcher.java_manager.installations;
            if installations.is_empty() {
                info!("No Java installations found. Try installing Java or setting JAVA_HOME.");
            } else {
                info!("Found {} Java installation(s):", installations.len());
                for (major, installation) in installations {
                    info!("  Java {}: {}", major, installation.path.display());
                }
            }
        }
        JavaCommands::Recommend { version } => {
            info!("Getting recommended Java version for Minecraft {version}...");
            let recommended = launcher::JavaManager::get_required_java_version(&version);
            info!("Recommended Java version: {recommended}");
        }
    }
}

/// Setup a custom logger with timestamps and colored output
fn setup_logger() {
    let env = Env::default().default_filter_or("info");

    Builder::from_env(env)
        .format(|buf, record| {
            let now = time::OffsetDateTime::now_utc();
            let timestamp = now
                .format(LOG_FORMAT)
                .unwrap_or_else(|_| String::from("timestamp-error"));

            let level_style = buf.default_level_style(record.level());

            writeln!(
                buf,
                "{} [{}] - {}",
                timestamp,
                level_style.value(record.level()),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}
