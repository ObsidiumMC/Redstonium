use crate::error::{AuthError, Result, ResultExt};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use time::{Duration, OffsetDateTime};
use tokio::fs;
use tracing::{debug, info, warn};

use super::AuthResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedAuth {
    pub access_token: String,
    pub profile: super::models::MinecraftProfile,
    pub expires_at: OffsetDateTime,
}

pub struct AuthStorage {
    cache_file_path: PathBuf,
}

impl AuthStorage {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        std::fs::create_dir_all(&cache_dir)
            .with_context(|| "Failed to create cache directory".to_string())?;

        let cache_file_path = cache_dir.join("auth_cache.json");

        Ok(Self { cache_file_path })
    }

    /// Get platform-specific cache directory
    fn get_cache_dir() -> Result<PathBuf> {
        match std::env::consts::OS {
            "windows" => {
                let appdata = std::env::var("APPDATA")
                    .with_context(|| "APPDATA environment variable not found".to_string())?;
                Ok(PathBuf::from(appdata).join("Redstonium").join("cache"))
            }
            "macos" => {
                let home = std::env::var("HOME")
                    .with_context(|| "HOME environment variable not found".to_string())?;
                Ok(PathBuf::from(home)
                    .join("Library")
                    .join("Caches")
                    .join("Redstonium"))
            }
            "linux" => {
                // Use XDG_CACHE_HOME if available, otherwise ~/.cache
                let cache_dir = std::env::var("XDG_CACHE_HOME").map_or_else(
                    |_| {
                        let home = std::env::var("HOME").unwrap_or_default();
                        PathBuf::from(home).join(".cache")
                    },
                    PathBuf::from,
                );
                Ok(cache_dir.join("Redstonium"))
            }
            _ => Err(AuthError::cache_error(format!(
                "Unsupported operating system: {}",
                std::env::consts::OS
            ))
            .into()),
        }
    }

    /// Save authentication result to cache
    pub async fn save_auth(&self, auth: &AuthResult) -> Result<()> {
        debug!("Saving authentication to cache");

        // Tokens typically expire in 24 hours, but we'll cache for 23 hours to be safe
        let expires_at = OffsetDateTime::now_utc() + Duration::hours(23);

        let cached_auth = CachedAuth {
            access_token: auth.access_token.clone(),
            profile: auth.profile.clone(),
            expires_at,
        };

        let json = serde_json::to_string_pretty(&cached_auth)
            .with_context(|| "Failed to serialize cached auth".to_string())?;

        fs::write(&self.cache_file_path, json)
            .await
            .with_context(|| "Failed to write auth cache file".to_string())?;

        info!(
            "✓ Authentication cached until {}",
            expires_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        );
        Ok(())
    }

    /// Load authentication result from cache if valid
    pub async fn load_auth(&self) -> Result<Option<AuthResult>> {
        if !self.cache_file_path.exists() {
            debug!("No auth cache file found");
            return Ok(None);
        }

        let content = fs::read_to_string(&self.cache_file_path)
            .await
            .with_context(|| "Failed to read auth cache file".to_string())?;

        let cached_auth: CachedAuth = serde_json::from_str(&content)
            .with_context(|| "Failed to parse cached auth".to_string())?;

        // Check if token is still valid
        let now = OffsetDateTime::now_utc();
        if now >= cached_auth.expires_at {
            warn!("Cached authentication has expired, requiring fresh login");
            // Clean up expired cache
            let _ = fs::remove_file(&self.cache_file_path).await;
            return Ok(None);
        }

        info!(
            "✓ Found valid cached authentication for {}",
            cached_auth.profile.name
        );
        debug!(
            "Cache expires at: {}",
            cached_auth
                .expires_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        );

        Ok(Some(AuthResult {
            access_token: cached_auth.access_token,
            profile: cached_auth.profile,
        }))
    }

    /// Clear cached authentication
    pub async fn clear_cache(&self) -> Result<()> {
        if self.cache_file_path.exists() {
            fs::remove_file(&self.cache_file_path)
                .await
                .with_context(|| "Failed to remove auth cache file".to_string())?;
            info!("✓ Authentication cache cleared");
        }
        Ok(())
    }
}
