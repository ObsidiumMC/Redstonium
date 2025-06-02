use crate::launcher;
use log::info;

/// Lists available Minecraft versions.
///
/// # Errors
///
/// Returns an error if fetching the version manifest or processing versions fails.
pub async fn list_versions(
    launcher: &launcher::Launcher,
    releases_only: bool,
    limit: usize,
) -> anyhow::Result<()> {
    // ...existing code from main.rs...
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

/// Prepares the specified Minecraft version by downloading necessary files and assets.
///
/// # Errors
///
/// Returns an error if resolving the version alias, fetching version info, creating directories,
/// or downloading game files, libraries, or assets fails.
pub async fn prepare_game(launcher: &launcher::Launcher, version: &str) -> anyhow::Result<()> {
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
) -> anyhow::Result<()> {
    // ...existing code from main.rs...
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

    let resolved_version = super::game::resolve_version_alias(launcher, &version).await?;

    // Validate Minecraft version before authentication
    if let Err(e) = launcher
        .file_manager
        .get_version_info(&resolved_version)
        .await
    {
        log::error!("Invalid Minecraft version: {resolved_version} : {e}");
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
    let auth_result = match crate::auth::authenticate().await {
        Ok(result) => {
            info!("Authentication successful!");
            info!("Welcome, {}!", result.profile.name);
            result
        }
        Err(e) => {
            log::error!("Authentication failed: {e}");
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
