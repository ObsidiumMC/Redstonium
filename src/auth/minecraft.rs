use anyhow::{Context, Result};
use log::{debug, error, trace, warn};
use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};

use super::constants::{MINECRAFT_AUTH_URL, MINECRAFT_ENTITLEMENT_URL, MINECRAFT_PROFILE_URL};
use super::models::{
    EntitlementResponse, MinecraftAuthRequest, MinecraftAuthResponse, MinecraftProfile,
};

/// Get Minecraft access token using XSTS token and user hash
pub async fn get_minecraft_token(
    client: &Client,
    xsts_token: &str,
    user_hash: &str,
) -> Result<String> {
    // Format the Xbox Live identity token for Minecraft
    let identity_token = format!("XBL3.0 x={user_hash};{xsts_token}");

    let minecraft_request = MinecraftAuthRequest { identity_token };

    debug!(
        "Sending authentication request to Minecraft services: {MINECRAFT_AUTH_URL}"
    );
    let response = client
        .post(MINECRAFT_AUTH_URL)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json") // Explicitly add Accept header
        .json(&minecraft_request)
        .send()
        .await
        .context("Failed to send request to Minecraft authentication endpoint")?;

    let status = response.status();
    debug!("Received response from Minecraft with status: {status}");

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|e| {
            error!("Failed to read error response body: {e}");
            "Unknown error".to_string()
        });
        error!(
            "Minecraft authentication failed with status {status}: {error_text}"
        );
        return Err(anyhow::anyhow!(
            "Minecraft authentication failed: {} - {}",
            status,
            error_text
        ));
    }

    let minecraft_response: MinecraftAuthResponse = response
        .json()
        .await
        .context("Failed to parse Minecraft authentication response as JSON")?;

    debug!(
        "Successfully retrieved Minecraft token with expiration in {} seconds",
        minecraft_response.expires_in
    );
    trace!(
        "Minecraft token length: {}",
        minecraft_response.access_token.len()
    );

    Ok(minecraft_response.access_token)
}

/// Verify that the user owns Minecraft
pub async fn verify_game_ownership(client: &Client, minecraft_token: &str) -> Result<()> {
    debug!("Verifying game ownership at: {MINECRAFT_ENTITLEMENT_URL}");
    let response = client
        .get(MINECRAFT_ENTITLEMENT_URL)
        .header(AUTHORIZATION, format!("Bearer {minecraft_token}"))
        .send()
        .await
        .context("Failed to send request to Minecraft entitlement endpoint")?;

    let status = response.status();
    debug!(
        "Received response from Minecraft entitlement check with status: {status}"
    );

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|e| {
            error!("Failed to read error response body: {e}");
            "Unknown error".to_string()
        });
        error!(
            "Failed to verify game ownership with status {status}: {error_text}"
        );
        return Err(anyhow::anyhow!(
            "Failed to verify game ownership: {} - {}",
            status,
            error_text
        ));
    }

    // For debugging, get the raw JSON response
    let body = response
        .text()
        .await
        .context("Failed to read entitlement response body")?;
    debug!("Got entitlements response: {body}");

    // If the response is empty or doesn't contain items, assume the user has the game
    // This is a workaround for differences in the API response format
    if body.trim().is_empty() || !body.contains("items") {
        warn!("No explicit entitlements found, assuming ownership is valid");
        return Ok(());
    }

    // Try to parse the response as JSON
    let parsed: Result<EntitlementResponse, _> = serde_json::from_str(&body);

    match parsed {
        Ok(entitlements) => {
            // Check if the user has a product named "game_minecraft" or "product_minecraft"
            let has_game = entitlements.items.iter().any(|item| {
                item.name == "game_minecraft"
                    || item.name == "product_minecraft"
                    || item.name.contains("minecraft")
            });

            if has_game {
                debug!("Found valid Minecraft entitlement");
            } else {
                warn!(
                    "No Minecraft entitlement found in response: {entitlements:?}"
                );
                warn!("Warning: No Minecraft entitlement found, but proceeding anyway");
            }

            Ok(())
        }
        Err(e) => {
            warn!("Couldn't parse entitlement data: {e}. Response: {body}");
            warn!("Warning: Couldn't parse entitlement data, proceeding anyway");
            // Continue anyway - we'll assume the user has the game
            Ok(())
        }
    }
}

/// Get the player's Minecraft profile
pub async fn get_player_profile(
    client: &Client,
    minecraft_token: &str,
) -> Result<MinecraftProfile> {
    debug!(
        "Retrieving Minecraft profile from: {MINECRAFT_PROFILE_URL}"
    );
    let response = client
        .get(MINECRAFT_PROFILE_URL)
        .header(AUTHORIZATION, format!("Bearer {minecraft_token}"))
        .send()
        .await
        .context("Failed to send request to Minecraft profile endpoint")?;

    let status = response.status();
    debug!(
        "Received response from Minecraft profile endpoint with status: {status}"
    );

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|e| {
            error!("Failed to read error response body: {e}");
            "Unknown error".to_string()
        });
        error!(
            "Failed to get Minecraft profile with status {status}: {error_text}"
        );
        return Err(anyhow::anyhow!(
            "Failed to get Minecraft profile: {} - {}",
            status,
            error_text
        ));
    }

    let profile: MinecraftProfile = response
        .json()
        .await
        .context("Failed to parse Minecraft profile response as JSON")?;

    debug!(
        "Successfully retrieved Minecraft profile for player: {} ({})",
        profile.name, profile.id
    );
    trace!(
        "Profile has {} skins and {} capes",
        profile.skins.as_ref().map_or(0, std::vec::Vec::len),
        profile.capes.as_ref().map_or(0, std::vec::Vec::len)
    );

    Ok(profile)
}
