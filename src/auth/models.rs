use serde::{Deserialize, Serialize};

/// The result of a successful authentication process
#[derive(Debug)]
pub struct AuthResult {
    pub access_token: String,
    pub profile: MinecraftProfile,
}

/// Represents a Minecraft player profile
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub skins: Option<Vec<Skin>>,
    pub capes: Option<Vec<Cape>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Skin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cape {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: String,
}

// JSON structures for Xbox Live authentication
#[derive(Serialize)]
pub struct XboxLiveRequest {
    pub properties: XboxLiveProperties,
    #[serde(rename = "RelyingParty")]
    pub relying_party: String,
    #[serde(rename = "TokenType")]
    pub token_type: String,
}

#[derive(Serialize)]
pub struct XboxLiveProperties {
    #[serde(rename = "AuthMethod")]
    pub auth_method: String,
    #[serde(rename = "SiteName")]
    pub site_name: String,
    #[serde(rename = "RpsTicket")]
    pub rps_ticket: String,
}

#[derive(Deserialize)]
pub struct XboxLiveResponse {
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "DisplayClaims")]
    pub display_claims: XboxDisplayClaims,
}

#[derive(Deserialize)]
pub struct XboxDisplayClaims {
    #[serde(rename = "xui")]
    pub xui: Vec<XboxUserInfo>,
}

#[derive(Deserialize)]
pub struct XboxUserInfo {
    #[serde(rename = "uhs")]
    pub uhs: String,
}

// XSTS Request structure
#[derive(Serialize)]
pub struct XstsRequest {
    pub properties: XstsProperties,
    #[serde(rename = "RelyingParty")]
    pub relying_party: String,
    #[serde(rename = "TokenType")]
    pub token_type: String,
}

#[derive(Serialize)]
pub struct XstsProperties {
    #[serde(rename = "SandboxId")]
    pub sandbox_id: String,
    #[serde(rename = "UserTokens")]
    pub user_tokens: Vec<String>,
}

// Minecraft authentication structures
#[derive(Serialize)]
pub struct MinecraftAuthRequest {
    #[serde(rename = "identityToken")]
    pub identity_token: String,
}

#[derive(Deserialize)]
pub struct MinecraftAuthResponse {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "expires_in")]
    pub expires_in: u64,
}

// Entitlement response from Minecraft services
#[derive(Deserialize, Debug)]
pub struct EntitlementResponse {
    pub items: Vec<Entitlement>,
}

#[derive(Deserialize, Debug)]
pub struct Entitlement {
    pub name: String,
}
