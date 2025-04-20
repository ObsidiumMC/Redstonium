// Microsoft OAuth2 constants - updated to use the correct endpoints
pub const MS_AUTH_URL: &str = "https://login.live.com/oauth20_authorize.srf";
pub const MS_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
// Use a local redirect URI
pub const REDIRECT_URI: &str = "http://localhost:8080"; // Make sure this matches the Azure App Registration

// Xbox Live constants
pub const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
pub const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

// Minecraft API constants
pub const MINECRAFT_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
pub const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
pub const MINECRAFT_ENTITLEMENT_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
