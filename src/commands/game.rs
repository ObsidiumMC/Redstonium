use crate::cli::{SortOrder, VersionTypeFilter};
use crate::launcher;
use tracing::{error, info};

/// Options for listing Minecraft versions
#[derive(Debug)]
pub struct ListVersionsOptions {
    pub types: Vec<VersionTypeFilter>,
    pub releases_only: bool,
    pub snapshots_only: bool,
    pub limit: usize,
    pub filter: Option<String>,
    pub show_installed: bool,
    pub sort: SortOrder,
}

/// Lists available Minecraft versions.
///
/// # Errors
///
/// Returns an error if fetching the version manifest or processing versions fails.
pub async fn list_versions(
    launcher: &launcher::Launcher,
    options: ListVersionsOptions,
) -> crate::error::Result<()> {
    // ...existing code from main.rs...
    info!("Fetching available Minecraft versions...");
    let manifest = launcher.file_manager.get_version_manifest().await?;

    let mut versions = manifest.versions.clone();

    // Handle filtering based on the new options
    if !options.types.is_empty() {
        // Filter by specific types provided
        versions.retain(|v| {
            options.types.iter().any(|filter_type| match filter_type {
                VersionTypeFilter::Release => {
                    matches!(v.version_type, launcher::VersionType::Release)
                }
                VersionTypeFilter::Snapshot => {
                    matches!(v.version_type, launcher::VersionType::Snapshot)
                }
                VersionTypeFilter::OldBeta => {
                    matches!(v.version_type, launcher::VersionType::OldBeta)
                }
                VersionTypeFilter::OldAlpha => {
                    matches!(v.version_type, launcher::VersionType::OldAlpha)
                }
            })
        });
    } else if options.releases_only {
        // Backward compatibility: filter only releases
        versions.retain(|v| matches!(v.version_type, launcher::VersionType::Release));
    } else if options.snapshots_only {
        // Filter only snapshots
        versions.retain(|v| matches!(v.version_type, launcher::VersionType::Snapshot));
    }

    // Apply text filter if provided
    if let Some(filter_pattern) = &options.filter {
        let pattern = filter_pattern.to_lowercase();
        versions.retain(|v| v.id.to_lowercase().contains(&pattern));
    }

    // Sort versions according to the specified order
    match options.sort {
        SortOrder::NewestFirst => {
            // Already sorted newest first in the manifest, no change needed
        }
        SortOrder::OldestFirst => {
            versions.reverse();
        }
        SortOrder::Alphabetical => {
            versions.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }

    // Apply limit
    versions.truncate(options.limit);

    // Show hint about filtering options if using default settings
    let is_using_defaults = options.types.is_empty()
        && !options.releases_only
        && !options.snapshots_only
        && options.filter.is_none()
        && options.limit == 10
        && !options.show_installed
        && matches!(options.sort, SortOrder::NewestFirst);

    if is_using_defaults {
        info!(
            "💡 Tip: Use 'Rustified list --help' to see filtering options like --types, --filter, --sort, and more!"
        );
    }

    info!("Available Minecraft versions:");
    info!("Latest release: {}", manifest.latest.release);
    info!("Latest snapshot: {}", manifest.latest.snapshot);

    if let Some(filter_pattern) = &options.filter {
        info!("Filtered by: \"{filter_pattern}\"");
    }

    if !options.types.is_empty() {
        let type_names: Vec<String> = options.types.iter().map(|t| format!("{t:?}")).collect();
        info!("Types: {}", type_names.join(", "));
    } else if options.releases_only {
        info!("Showing only: Release versions");
    } else if options.snapshots_only {
        info!("Showing only: Snapshot versions");
    }

    if !is_using_defaults {
        info!("Sort order: {:?}", options.sort);
    }
    info!(
        "Showing {} of {} total versions",
        versions.len().min(options.limit),
        manifest.versions.len()
    );
    info!("");

    for version in &versions {
        let installed_marker =
            if options.show_installed && launcher.minecraft_dir.is_version_installed(&version.id) {
                "✓ "
            } else if options.show_installed {
                "  "
            } else {
                ""
            };

        info!(
            "{}{} ({:?}) - {}",
            installed_marker, version.id, version.version_type, version.release_time
        );
    }

    Ok(())
}

/// Prepares the specified Minecraft version by downloading necessary files and assets.
///
/// # Errors
///
/// Returns an error if resolving the version alias, fetching version info, creating directories,
/// or downloading game files, libraries, or assets fails.
pub async fn prepare_game(
    launcher: &launcher::Launcher,
    version: &str,
) -> crate::error::Result<()> {
    // ...existing code from main.rs...
    let resolved_version = super::game::resolve_version_alias(launcher, version).await?;

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

/// Launches the specified Minecraft instance, handling authentication and preparation.
///
/// # Errors
///
/// Returns an error if the instance does not exist, the Minecraft version is invalid,
/// authentication fails, preparation fails, or launching the game fails.
pub async fn launch_game(
    launcher: &launcher::Launcher,
    instance_name: &str,
    _skip_verification: bool,
) -> crate::error::Result<()> {
    let (instance_config, version) = {
        let instance_manager = launcher.instance_manager.lock().await;
        if let Some(config) = instance_manager.get_instance(instance_name) {
            let config_clone = config.clone();
            let version = config.version.clone();
            (Some(config_clone), version)
        } else {
            return Err(crate::error::InstanceError::not_found(format!(
                "Instance '{instance_name}' does not exist. Use 'Rustified instance list' to see available instances or 'Rustified instance create' to create one."
            )).into());
        }
    };

    let resolved_version = super::game::resolve_version_alias(launcher, &version).await?;

    // Validate Minecraft version before authentication
    if let Err(e) = launcher
        .file_manager
        .get_version_info(&resolved_version)
        .await
    {
        error!("Invalid Minecraft version: {resolved_version} : {e}");
        return Err(crate::error::GameError::invalid_version(format!(
            "Instance '{instance_name}' uses an invalid Minecraft version ('{resolved_version}'). Use 'Rustified list' to see valid versions."
        )).into());
    }

    // Update last used timestamp
    {
        let mut instance_manager = launcher.instance_manager.lock().await;
        instance_manager.update_last_used(instance_name).await?;
    }

    info!("Launching Minecraft {resolved_version} with instance '{instance_name}'...");

    // Authenticate first
    info!("Starting authentication process...");
    let auth_result = match crate::auth::authenticate().await {
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

/// Resolves a version alias (like "latest-release" or "latest-snapshot") to a concrete Minecraft version string.
///
/// # Errors
///
/// Returns an error if fetching the version manifest fails.
pub async fn resolve_version_alias(
    launcher: &launcher::Launcher,
    version: &str,
) -> crate::error::Result<String> {
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
