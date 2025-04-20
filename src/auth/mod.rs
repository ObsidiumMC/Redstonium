use anyhow::{Result, Context};
use log::{debug, info, trace};

// Declare modules
mod constants;
mod models;
mod microsoft;
mod xbox;
mod minecraft;

// Re-export necessary items
pub use models::{AuthResult, MinecraftProfile, Skin, Cape};

/// The main authentication function that orchestrates the entire Minecraft auth flow
pub async fn authenticate() -> Result<AuthResult> {
    // Step 1: Get Microsoft OAuth token
    info!("Starting Microsoft OAuth authentication process");
    debug!("Using Microsoft OAuth endpoints: Auth URL: {}, Token URL: {}", constants::MS_AUTH_URL, constants::MS_TOKEN_URL);
    let ms_token = microsoft::get_microsoft_token()
        .await
        .context("Failed to get Microsoft OAuth token")?;
    info!("✓ Microsoft authentication successful");
    
    // Step 2: Get Xbox Live token using Microsoft token
    info!("Starting Xbox Live authentication");
    let (xbl_token, user_hash) = xbox::get_xbox_live_token(&ms_token)
        .await
        .context("Failed to get Xbox Live token")?;
    info!("✓ Xbox Live authentication successful");
    debug!("Retrieved user hash: {}", user_hash);
    
    // Step 3: Get XSTS token using Xbox Live token
    info!("Starting XSTS authentication");
    let xsts_token = xbox::get_xsts_token(&xbl_token)
        .await
        .context("Failed to get XSTS token")?;
    info!("✓ XSTS authentication successful");
    
    // Step 4: Authenticate with Minecraft using XSTS token
    info!("Starting Minecraft authentication");
    let minecraft_token = minecraft::get_minecraft_token(&xsts_token, &user_hash)
        .await
        .context("Failed to get Minecraft token")?;
    info!("✓ Minecraft authentication successful");
    trace!("Minecraft token length: {}", minecraft_token.len());
    
    // Step 5: Verify game ownership
    info!("Verifying Minecraft game ownership");
    minecraft::verify_game_ownership(&minecraft_token)
        .await
        .context("Failed to verify game ownership")?;
    info!("✓ Game ownership verified");
    
    // Step 6: Get player profile
    info!("Retrieving player profile");
    let profile = minecraft::get_player_profile(&minecraft_token)
        .await
        .context("Failed to get player profile")?;
    info!("✓ Player profile retrieved for: {}", profile.name);
    
    Ok(AuthResult {
        access_token: minecraft_token,
        profile,
    })
}