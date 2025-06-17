use crate::cli::AuthCommands;
use tracing::info;

/// Handles authentication-related commands.
///
/// # Errors
///
/// Returns an error if authentication storage operations fail or if authentication fails.
pub async fn handle_auth_command(action: AuthCommands) -> crate::error::Result<()> {
    let storage = crate::auth::storage::AuthStorage::new()?;

    match action {
        AuthCommands::Status => {
            if let Some(cached_auth) = storage.load_auth().await? {
                info!("✓ Authentication: Valid");
                info!("  Player: {}", cached_auth.profile.name);
                info!("  UUID: {}", cached_auth.profile.id);
                // Don't log the token for security
            } else {
                info!("❌ No valid authentication found");
                info!("  Run 'Redstonium launch <instance>' to authenticate");
            }
        }
        AuthCommands::Clear => {
            storage.clear_cache().await?;
            info!("✓ Authentication cache cleared");
        }
        AuthCommands::Refresh => {
            info!("Clearing cache and forcing re-authentication...");
            storage.clear_cache().await?;
            let auth_result = crate::auth::authenticate().await?;
            info!(
                "✓ Re-authentication successful for {}",
                auth_result.profile.name
            );
        }
    }

    Ok(())
}
