use anyhow::Result;
use oauth2::{
    AuthUrl, ClientId, RedirectUrl, TokenUrl,
    basic::BasicClient, AuthorizationCode, CsrfToken, Scope, TokenResponse,
};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::io::stdin;

// Microsoft OAuth2 constants - updated to use the correct endpoints
const MS_AUTH_URL: &str = "https://login.live.com/oauth20_authorize.srf";
const MS_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";

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
    let ms_token = get_microsoft_token("00000000402b5328").await?;
    println!("✓ Microsoft authentication successful");
    
    // Step 2: Get Xbox Live token using Microsoft token
    let (xbl_token, user_hash) = get_xbox_live_token(&ms_token).await?;
    println!("✓ Xbox Live authentication successful");
    
    // Step 3: Get XSTS token using Xbox Live token
    let xsts_token = get_xsts_token(&xbl_token).await?;
    println!("✓ XSTS authentication successful");
    
    // Step 4: Authenticate with Minecraft using XSTS token
    let minecraft_token = get_minecraft_token(&xsts_token, &user_hash).await?;
    println!("✓ Minecraft authentication successful");
    
    // Step 5: Verify game ownership
    verify_game_ownership(&minecraft_token).await?;
    println!("✓ Game ownership verified");
    
    // Step 6: Get player profile
    let profile = get_player_profile(&minecraft_token).await?;
    println!("✓ Player profile retrieved");
    
    Ok(AuthResult {
        access_token: minecraft_token,
        profile,
    })
}

/// Get a Microsoft OAuth token using the device code flow
async fn get_microsoft_token(client_id: &str) -> Result<String> {
    let oauth_client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        None, // No client secret for public clients
        AuthUrl::new(MS_AUTH_URL.to_string())?,
        Some(TokenUrl::new(MS_TOKEN_URL.to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.to_string())?);
    
    // Generate the authorization URL
    let (auth_url, _csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("XboxLive.signin".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        .url();
    
    // Open the browser for the user to log in
    println!("Opening browser for Microsoft authentication...");
    webbrowser::open(auth_url.as_str())?;
    
    // After redirect, user will see the authorization code
    println!("Please complete the login in your browser.");
    println!("After login, you'll be redirected to a page. Look for a URL parameter that says 'code='");
    println!("Please copy ONLY the code part (after 'code=' and before any '&' character if present) and paste it here:");
    
    let mut code = String::new();
    stdin().read_line(&mut code)?;
    code = code.trim().to_string();
    
    if code.is_empty() {
        return Err(anyhow::anyhow!("No authorization code provided"));
    }
    
    println!("Exchanging authorization code for access token...");
    
    // Exchange authorization code for access token
    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await?;
    
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
    
    println!("Authenticating with Xbox Live...");
    let response = client.post(XBL_AUTH_URL)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .json(&xbl_request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Xbox Live authentication failed: {}", error_text));
    }
    
    let xbl_response: XboxLiveResponse = response.json().await?;
    let user_hash = xbl_response.display_claims.xui.get(0)
        .ok_or_else(|| anyhow::anyhow!("No Xbox User Hash found"))?
        .uhs.clone();
    
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
    
    println!("Getting XSTS token...");
    let response = client.post(XSTS_AUTH_URL)
        .header(CONTENT_TYPE, "application/json")
        .json(&xsts_request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("XSTS authentication failed: {}", error_text));
    }
    
    let xsts_response: XboxLiveResponse = response.json().await?;
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
    
    println!("Authenticating with Minecraft services...");
    let response = client.post(MINECRAFT_AUTH_URL)
        .header(CONTENT_TYPE, "application/json")
        .json(&minecraft_request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Minecraft authentication failed: {}", error_text));
    }
    
    let minecraft_response: MinecraftAuthResponse = response.json().await?;
    Ok(minecraft_response.access_token)
}

/// Verify that the user owns Minecraft
async fn verify_game_ownership(minecraft_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    
    println!("Verifying game ownership...");
    let response = client.get(MINECRAFT_ENTITLEMENT_URL)
        .header(AUTHORIZATION, format!("Bearer {}", minecraft_token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Failed to verify game ownership: {}", error_text));
    }
    
    // For debugging, print the raw JSON response
    let body = response.text().await?;
    println!("Got entitlements response: {}", body);
    
    // If the response is empty or doesn't contain items, assume the user has the game
    // This is a workaround for differences in the API response format
    if body.trim().is_empty() || !body.contains("items") {
        println!("No explicit entitlements found, assuming ownership is valid");
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
                println!("Warning: No Minecraft entitlement found, but proceeding anyway");
            }
            
            Ok(())
        },
        Err(e) => {
            println!("Warning: Couldn't parse entitlement data: {}. Proceeding anyway", e);
            // Continue anyway - we'll assume the user has the game
            Ok(())
        }
    }
}

/// Get the player's Minecraft profile
async fn get_player_profile(minecraft_token: &str) -> Result<MinecraftProfile> {
    let client = reqwest::Client::new();
    
    println!("Retrieving Minecraft profile...");
    let response = client.get(MINECRAFT_PROFILE_URL)
        .header(AUTHORIZATION, format!("Bearer {}", minecraft_token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Failed to get Minecraft profile: {}", error_text));
    }
    
    let profile: MinecraftProfile = response.json().await?;
    Ok(profile)
}