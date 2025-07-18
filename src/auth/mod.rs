use crate::error::{Result, ResultExt};
use reqwest::Client;
use tracing::{debug, info, trace, warn};

// Declare modules
mod constants;
mod microsoft;
mod minecraft;
mod models;
pub mod storage;
mod xbox;

pub use models::AuthResult;
use storage::AuthStorage;

pub async fn authenticate() -> Result<AuthResult> {
    // Initialize auth storage
    let auth_storage =
        AuthStorage::new().with_context(|| "Failed to initialize auth storage".to_string())?;

    // Try to load cached authentication first
    if let Some(cached_auth) = auth_storage.load_auth().await? {
        info!(
            "Using cached authentication for {}",
            cached_auth.profile.name
        );
        return Ok(cached_auth);
    }

    info!("No valid cached authentication found, starting fresh authentication");

    // Create a single reqwest client to be reused
    let client = Client::new();
    info!("Created shared HTTP client");

    // Step 1: Get Microsoft OAuth token
    info!("Starting Microsoft OAuth authentication process");
    debug!(
        "Using Microsoft OAuth endpoints: Auth URL: {}, Token URL: {}",
        constants::MS_AUTH_URL,
        constants::MS_TOKEN_URL
    );
    let ms_token = microsoft::get_microsoft_token()
        .await
        .with_context(|| "Failed to get Microsoft OAuth token".to_string())?;
    info!("✓ Microsoft authentication successful");

    // Step 2: Get Xbox Live token using Microsoft token
    info!("Starting Xbox Live authentication");
    let (xbl_token, user_hash) = xbox::get_xbox_live_token(&client, &ms_token)
        .await
        .with_context(|| "Failed to get Xbox Live token".to_string())?;
    info!("✓ Xbox Live authentication successful");
    debug!("Retrieved user hash: {user_hash}");

    // Step 3: Get XSTS token using Xbox Live token
    info!("Starting XSTS authentication");
    let xsts_token = xbox::get_xsts_token(&client, &xbl_token)
        .await
        .with_context(|| "Failed to get XSTS token".to_string())?;
    info!("✓ XSTS authentication successful");

    // Step 4: Authenticate with Minecraft using XSTS token
    info!("Starting Minecraft authentication");
    let minecraft_token = minecraft::get_minecraft_token(&client, &xsts_token, &user_hash)
        .await
        .with_context(|| "Failed to get Minecraft token".to_string())?;
    info!("✓ Minecraft authentication successful");
    trace!("Minecraft token length: {}", minecraft_token.len());

    // Step 5: Verify game ownership
    info!("Verifying Minecraft game ownership");
    minecraft::verify_game_ownership(&client, &minecraft_token)
        .await
        .with_context(|| "Failed to verify game ownership".to_string())?;
    info!("✓ Game ownership verified");

    // Step 6: Get player profile
    info!("Retrieving player profile");
    let profile = minecraft::get_player_profile(&client, &minecraft_token)
        .await
        .with_context(|| "Failed to get player profile".to_string())?;
    info!("✓ Player profile retrieved for: {}", profile.name);

    let auth_result = AuthResult {
        access_token: minecraft_token.clone(),
        profile,
    };

    // Cache the authentication result for future use
    if let Err(e) = auth_storage.save_auth(&auth_result).await {
        warn!("Failed to cache authentication: {e}");
        // Don't fail the authentication if caching fails
    }

    Ok(auth_result)
}
