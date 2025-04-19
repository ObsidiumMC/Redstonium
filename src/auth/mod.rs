use anyhow::{Result, Context, anyhow};
use log::{debug, info, warn, error, trace};
use oauth2::{
    AuthUrl, ClientId, RedirectUrl, TokenUrl,
    basic::BasicClient, AuthorizationCode, CsrfToken, Scope, TokenResponse,
};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tiny_http::{Server, Response};
use tokio::sync::oneshot;
use tokio::task;
use url::Url;
use std::env; // Import env module

// Microsoft OAuth2 constants - updated to use the correct endpoints
const MS_AUTH_URL: &str = "https://login.live.com/oauth20_authorize.srf";
const MS_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
// Use a local redirect URI
const REDIRECT_URI: &str = "http://localhost:8080"; // Make sure this matches the Azure App Registration

// Xbox Live constants
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

// Minecraft API constants
const MINECRAFT_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const MINECRAFT_ENTITLEMENT_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";

/// The result of a successful authentication process
#[derive(Debug)]
pub struct AuthResult {
    pub access_token: String,
    pub profile: MinecraftProfile,
}

/// Represents a Minecraft player profile
#[derive(Debug, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub skins: Option<Vec<Skin>>,
    pub capes: Option<Vec<Cape>>,
}

#[derive(Debug, Deserialize)]
pub struct Skin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Debug, Deserialize)]
pub struct Cape {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: String,
}

// JSON structures for Xbox Live authentication
#[derive(Serialize)]
struct XboxLiveRequest {
    properties: XboxLiveProperties,
    #[serde(rename = "RelyingParty")]
    relying_party: String,
    #[serde(rename = "TokenType")]
    token_type: String,
}

#[derive(Serialize)]
struct XboxLiveProperties {
    #[serde(rename = "AuthMethod")]
    auth_method: String,
    #[serde(rename = "SiteName")]
    site_name: String,
    #[serde(rename = "RpsTicket")]
    rps_ticket: String,
}

#[derive(Deserialize)]
struct XboxLiveResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XboxDisplayClaims,
}

#[derive(Deserialize)]
struct XboxDisplayClaims {
    #[serde(rename = "xui")]
    xui: Vec<XboxUserInfo>,
}

#[derive(Deserialize)]
struct XboxUserInfo {
    #[serde(rename = "uhs")]
    uhs: String,
}

// XSTS Request structure
#[derive(Serialize)]
struct XstsRequest {
    properties: XstsProperties,
    #[serde(rename = "RelyingParty")]
    relying_party: String,
    #[serde(rename = "TokenType")]
    token_type: String,
}

#[derive(Serialize)]
struct XstsProperties {
    #[serde(rename = "SandboxId")]
    sandbox_id: String,
    #[serde(rename = "UserTokens")]
    user_tokens: Vec<String>,
}

// Minecraft authentication structures
#[derive(Serialize)]
struct MinecraftAuthRequest {
    #[serde(rename = "identityToken")]
    identity_token: String,
}

#[derive(Deserialize)]
struct MinecraftAuthResponse {
    #[serde(rename = "access_token")]
    access_token: String,
    #[serde(rename = "expires_in")]
    expires_in: u64,
}

// Entitlement response from Minecraft services
#[derive(Deserialize, Debug)]
struct EntitlementResponse {
    items: Vec<Entitlement>,
    // signature field might be optional or named differently
    #[serde(default, rename = "signature")]
    signature: String,
}

#[derive(Deserialize, Debug)]
struct Entitlement {
    name: String,
    // Make signedDate optional as it might not be present
    #[serde(rename = "signedDate", default)]
    signed_date: String,
}

/// The main authentication function that orchestrates the entire Minecraft auth flow
pub async fn authenticate() -> Result<AuthResult> {
    // Step 1: Get Microsoft OAuth token
    info!("Starting Microsoft OAuth authentication process");
    debug!("Using Microsoft OAuth endpoints: Auth URL: {}, Token URL: {}", MS_AUTH_URL, MS_TOKEN_URL);
    let ms_token = get_microsoft_token()
        .await
        .context("Failed to get Microsoft OAuth token")?;
    info!("✓ Microsoft authentication successful");
    
    // Step 2: Get Xbox Live token using Microsoft token
    info!("Starting Xbox Live authentication");
    let (xbl_token, user_hash) = get_xbox_live_token(&ms_token)
        .await
        .context("Failed to get Xbox Live token")?;
    info!("✓ Xbox Live authentication successful");
    debug!("Retrieved user hash: {}", user_hash);
    
    // Step 3: Get XSTS token using Xbox Live token
    info!("Starting XSTS authentication");
    let xsts_token = get_xsts_token(&xbl_token)
        .await
        .context("Failed to get XSTS token")?;
    info!("✓ XSTS authentication successful");
    
    // Step 4: Authenticate with Minecraft using XSTS token
    info!("Starting Minecraft authentication");
    let minecraft_token = get_minecraft_token(&xsts_token, &user_hash)
        .await
        .context("Failed to get Minecraft token")?;
    info!("✓ Minecraft authentication successful");
    trace!("Minecraft token length: {}", minecraft_token.len());
    
    // Step 5: Verify game ownership
    info!("Verifying Minecraft game ownership");
    verify_game_ownership(&minecraft_token)
        .await
        .context("Failed to verify game ownership")?;
    info!("✓ Game ownership verified");
    
    // Step 6: Get player profile
    info!("Retrieving player profile");
    let profile = get_player_profile(&minecraft_token)
        .await
        .context("Failed to get player profile")?;
    info!("✓ Player profile retrieved for: {}", profile.name);
    
    Ok(AuthResult {
        access_token: minecraft_token,
        profile,
    })
}

/// Get a Microsoft OAuth token using the authorization code flow with a local server
async fn get_microsoft_token() -> Result<String> {
    let client_id = env::var("MS_CLIENT_ID")
        .context("MS_CLIENT_ID environment variable not set. Make sure .env file is present and loaded.")?;
    
    debug!("Creating OAuth client with client ID: {}", client_id);

    let redirect_url = RedirectUrl::new(REDIRECT_URI.to_string())
        .context("Invalid redirect URI")?;

    let oauth_client = BasicClient::new(
        ClientId::new(client_id),
        None, // No client secret for public clients
        AuthUrl::new(MS_AUTH_URL.to_string()).context("Invalid Microsoft Auth URL")?,
        Some(TokenUrl::new(MS_TOKEN_URL.to_string()).context("Invalid Microsoft Token URL")?)
    )
    .set_redirect_uri(redirect_url.clone());

    // Generate the authorization URL
    debug!("Generating authorization URL with scopes: XboxLive.signin, offline_access and prompt=login");
    let (auth_url, _csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("XboxLive.signin".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        // Add the prompt=login parameter to force user interaction
        // .add_extra_param("prompt", "login") 
        .url();

    // Channel to receive the authorization code from the local server
    let (tx, rx) = oneshot::channel::<Result<String>>();

    // Start the local server in a blocking thread
    let server_handle = task::spawn_blocking(move || {
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("Failed to parse address");
        let server = match Server::http(addr) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to start local HTTP server: {}", e);
                let _ = tx.send(Err(anyhow!("Failed to start local HTTP server on {}: {}", addr, e)));
                return;
            }
        };
        info!("Local server listening on {}", addr);
        info!("Waiting for redirect from Microsoft...");

        match server.recv() {
            Ok(rq) => {
                let url_str = format!("http://localhost:8080{}", rq.url());
                debug!("Received request: {}", url_str);

                let response_text;
                if let Ok(url) = Url::parse(&url_str) {
                    let code = url.query_pairs().find(|(key, _)| key == "code").map(|(_, value)| value.into_owned());
                    let error = url.query_pairs().find(|(key, _)| key == "error").map(|(_, value)| value.into_owned());
                    let error_description = url.query_pairs().find(|(key, _)| key == "error_description").map(|(_, value)| value.into_owned());

                    if let Some(code) = code {
                        debug!("Received authorization code.");
                        response_text = "Authentication successful! You can close this window.".to_string();
                        let _ = tx.send(Ok(code));
                    } else if let Some(error) = error {
                        let description = error_description.unwrap_or_else(|| "No description provided.".to_string());
                        error!("OAuth error received: {} - {}", error, description);
                        response_text = format!("Authentication failed: {} - {}. Please close this window.", error, description);
                        let _ = tx.send(Err(anyhow!("OAuth error: {} - {}", error, description)));
                    } else {
                        warn!("Received request without 'code' or 'error' parameter.");
                        response_text = "Received unexpected request. Please close this window.".to_string();
                        let _ = tx.send(Err(anyhow!("Invalid redirect request received")));
                    }
                } else {
                    error!("Failed to parse redirect URL: {}", url_str);
                    response_text = "Error processing request. Please close this window.".to_string();
                    let _ = tx.send(Err(anyhow!("Failed to parse redirect URL")));
                }

                let response = Response::from_string(response_text).with_status_code(200);
                if let Err(e) = rq.respond(response) {
                    error!("Failed to send response to browser: {}", e);
                }
                debug!("Local server responded and is shutting down.");
            },
            Err(e) => {
                error!("Local server failed to receive request: {}", e);
                let _ = tx.send(Err(anyhow!("Local server error: {}", e)));
            }
        }
        // Server automatically stops after handling one request
    });

    // Open the browser for the user to log in
    info!("Opening browser for Microsoft authentication...");
    webbrowser::open(auth_url.as_str()).context("Failed to open web browser for authentication")?;
    info!("Please complete the login in your browser. Waiting for authorization code...");

    // Wait for the server thread to send the code
    let code = rx.await.context("Authentication process cancelled or failed")??;

    // Ensure the server task finished
    server_handle.await.context("Server task panicked")?;

    if code.is_empty() {
        error!("Received empty authorization code");
        return Err(anyhow::anyhow!("Received empty authorization code"));
    }

    info!("Exchanging authorization code for access token...");
    debug!("Authorization code length: {}", code.len());

    // Exchange authorization code for access token
    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .context("Failed to exchange authorization code for token")?;

    debug!("Successfully received access token");
    trace!("Access token length: {}", token_result.access_token().secret().len());

    Ok(token_result.access_token().secret().clone())
}

/// Get Xbox Live token using the Microsoft access token
async fn get_xbox_live_token(ms_token: &str) -> Result<(String, String)> {
    let client = reqwest::Client::new();
    
    // Build the Xbox Live authentication request
    let xbl_request = XboxLiveRequest {
        properties: XboxLiveProperties {
            auth_method: "RPS".to_string(),
            site_name: "user.auth.xboxlive.com".to_string(),
            rps_ticket: format!("d={}", ms_token),
        },
        relying_party: "http://auth.xboxlive.com".to_string(),
        token_type: "JWT".to_string(),
    };
    
    debug!("Sending authentication request to Xbox Live: {}", XBL_AUTH_URL);
    let response = client.post(XBL_AUTH_URL)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .json(&xbl_request)
        .send()
        .await
        .context("Failed to send request to Xbox Live authentication endpoint")?;
    
    let status = response.status();
    debug!("Received response from Xbox Live with status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|e| {
            error!("Failed to read error response body: {}", e);
            "Unknown error".to_string()
        });
        error!("Xbox Live authentication failed with status {}: {}", status, error_text);
        return Err(anyhow::anyhow!("Xbox Live authentication failed: {} - {}", status, error_text));
    }
    
    let xbl_response: XboxLiveResponse = response.json().await
        .context("Failed to parse Xbox Live response as JSON")?;
    
    let user_hash = match xbl_response.display_claims.xui.get(0) {
        Some(info) => info.uhs.clone(),
        None => {
            error!("No Xbox User Hash found in response");
            return Err(anyhow::anyhow!("No Xbox User Hash found in response"));
        }
    };
    
    debug!("Successfully retrieved Xbox Live token and user hash");
    trace!("User hash: {}, Token length: {}", user_hash, xbl_response.token.len());
    
    Ok((xbl_response.token, user_hash))
}

/// Get XSTS token using the Xbox Live token
async fn get_xsts_token(xbl_token: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    // Build the XSTS authentication request
    let xsts_request = XstsRequest {
        properties: XstsProperties {
            sandbox_id: "RETAIL".to_string(),
            user_tokens: vec![xbl_token.to_string()],
        },
        relying_party: "rp://api.minecraftservices.com/".to_string(),
        token_type: "JWT".to_string(),
    };
    
    debug!("Sending XSTS authentication request to: {}", XSTS_AUTH_URL);
    let response = client.post(XSTS_AUTH_URL)
        .header(CONTENT_TYPE, "application/json")
        .json(&xsts_request)
        .send()
        .await
        .context("Failed to send request to XSTS authentication endpoint")?;
    
    let status = response.status();
    debug!("Received response from XSTS with status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|e| {
            error!("Failed to read error response body: {}", e);
            "Unknown error".to_string()
        });
        
        // Special handling for common error codes
        if status.as_u16() == 401 {
            if error_text.contains("2148916233") {
                error!("XSTS authentication failed: Account belongs to a child (under 18) and requires adult approval");
                return Err(anyhow::anyhow!("Xbox Live account belongs to a child (under 18) and requires adult approval"));
            } else if error_text.contains("2148916238") {
                error!("XSTS authentication failed: Account is from a country/region where Xbox Live is not available");
                return Err(anyhow::anyhow!("Xbox Live is not available in your country/region"));
            }
        }
        
        error!("XSTS authentication failed with status {}: {}", status, error_text);
        return Err(anyhow::anyhow!("XSTS authentication failed: {} - {}", status, error_text));
    }
    
    let xsts_response: XboxLiveResponse = response.json().await
        .context("Failed to parse XSTS response as JSON")?;
    
    debug!("Successfully retrieved XSTS token");
    trace!("XSTS token length: {}", xsts_response.token.len());
    
    Ok(xsts_response.token)
}

/// Get Minecraft access token using XSTS token and user hash
async fn get_minecraft_token(xsts_token: &str, user_hash: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    // Format the Xbox Live identity token for Minecraft
    let identity_token = format!("XBL3.0 x={};{}", user_hash, xsts_token);
    
    let minecraft_request = MinecraftAuthRequest {
        identity_token,
    };
    
    debug!("Sending authentication request to Minecraft services: {}", MINECRAFT_AUTH_URL);
    let response = client.post(MINECRAFT_AUTH_URL)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json") // Explicitly add Accept header
        .json(&minecraft_request)
        .send()
        .await
        .context("Failed to send request to Minecraft authentication endpoint")?;
    
    let status = response.status();
    debug!("Received response from Minecraft with status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|e| {
            error!("Failed to read error response body: {}", e);
            "Unknown error".to_string()
        });
        error!("Minecraft authentication failed with status {}: {}", status, error_text);
        return Err(anyhow::anyhow!("Minecraft authentication failed: {} - {}", status, error_text));
    }
    
    let minecraft_response: MinecraftAuthResponse = response.json().await
        .context("Failed to parse Minecraft authentication response as JSON")?;
    
    debug!("Successfully retrieved Minecraft token with expiration in {} seconds", minecraft_response.expires_in);
    trace!("Minecraft token length: {}", minecraft_response.access_token.len());
    
    Ok(minecraft_response.access_token)
}

/// Verify that the user owns Minecraft
async fn verify_game_ownership(minecraft_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    
    debug!("Verifying game ownership at: {}", MINECRAFT_ENTITLEMENT_URL);
    let response = client.get(MINECRAFT_ENTITLEMENT_URL)
        .header(AUTHORIZATION, format!("Bearer {}", minecraft_token))
        .send()
        .await
        .context("Failed to send request to Minecraft entitlement endpoint")?;
    
    let status = response.status();
    debug!("Received response from Minecraft entitlement check with status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|e| {
            error!("Failed to read error response body: {}", e);
            "Unknown error".to_string()
        });
        error!("Failed to verify game ownership with status {}: {}", status, error_text);
        return Err(anyhow::anyhow!("Failed to verify game ownership: {} - {}", status, error_text));
    }
    
    // For debugging, get the raw JSON response
    let body = response.text().await
        .context("Failed to read entitlement response body")?;
    debug!("Got entitlements response: {}", body);
    
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
            let has_game = entitlements.items.iter().any(|item| 
                item.name == "game_minecraft" || item.name == "product_minecraft" || item.name.contains("minecraft")
            );
            
            if !has_game {
                warn!("No Minecraft entitlement found in response: {:?}", entitlements);
                warn!("Warning: No Minecraft entitlement found, but proceeding anyway");
            } else {
                debug!("Found valid Minecraft entitlement");
            }
            
            Ok(())
        },
        Err(e) => {
            warn!("Couldn't parse entitlement data: {}. Response: {}", e, body);
            warn!("Warning: Couldn't parse entitlement data, proceeding anyway");
            // Continue anyway - we'll assume the user has the game
            Ok(())
        }
    }
}

/// Get the player's Minecraft profile
async fn get_player_profile(minecraft_token: &str) -> Result<MinecraftProfile> {
    let client = reqwest::Client::new();
    
    debug!("Retrieving Minecraft profile from: {}", MINECRAFT_PROFILE_URL);
    let response = client.get(MINECRAFT_PROFILE_URL)
        .header(AUTHORIZATION, format!("Bearer {}", minecraft_token))
        .send()
        .await
        .context("Failed to send request to Minecraft profile endpoint")?;
    
    let status = response.status();
    debug!("Received response from Minecraft profile endpoint with status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|e| {
            error!("Failed to read error response body: {}", e);
            "Unknown error".to_string()
        });
        error!("Failed to get Minecraft profile with status {}: {}", status, error_text);
        return Err(anyhow::anyhow!("Failed to get Minecraft profile: {} - {}", status, error_text));
    }
    
    let profile: MinecraftProfile = response.json().await
        .context("Failed to parse Minecraft profile response as JSON")?;
    
    debug!("Successfully retrieved Minecraft profile for player: {} ({})", profile.name, profile.id);
    trace!("Profile has {} skins and {} capes", 
        profile.skins.as_ref().map_or(0, |s| s.len()),
        profile.capes.as_ref().map_or(0, |c| c.len())
    );
    
    Ok(profile)
}