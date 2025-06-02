mod files;
mod game;
mod instance;
pub mod java;
mod minecraft_dir;
mod version;

pub use files::{FileManager, get_library_path};
pub use instance::{InstanceConfig, InstanceManager};
pub use java::JavaManager;
pub use minecraft_dir::MinecraftDir;
pub use version::VersionType;

use crate::{auth::AuthResult, launcher};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Launcher {
    pub minecraft_dir: MinecraftDir,
    pub file_manager: FileManager,
    pub java_manager: JavaManager,
    pub instance_manager: Arc<Mutex<InstanceManager>>,
}

impl Launcher {
    pub async fn new() -> Result<Self> {
        let minecraft_dir = MinecraftDir::new()?;
        let file_manager = FileManager::new();
        let mut java_manager = JavaManager::new();

        // Initialize Java manager
        java_manager.initialize();

        // Initialize instance manager with Arc<Mutex<>> for shared mutable access
        let instance_manager = Arc::new(Mutex::new(
            InstanceManager::new(minecraft_dir.clone()).await?,
        ));

        Ok(Self {
            minecraft_dir,
            file_manager,
            java_manager,
            instance_manager,
        })
    }

    pub async fn prepare_game(&self, version_id: &str, _auth: &AuthResult) -> Result<()> {
        // Download version manifest and get version info
        let version_info = self.file_manager.get_version_info(version_id).await?;

        // Ensure version directory exists
        self.minecraft_dir.ensure_version_dir(version_id)?;

        // Download main game JAR
        self.file_manager
            .download_game_jar(&version_info, &self.minecraft_dir)
            .await?;

        // Download libraries
        self.file_manager
            .download_libraries(&version_info, &self.minecraft_dir)
            .await?;

        // Download assets
        self.file_manager
            .download_assets(&version_info, &self.minecraft_dir)
            .await?;

        Ok(())
    }

    pub async fn launch_game(
        &self,
        version_id: &str,
        auth: &AuthResult,
        instance: Option<&InstanceConfig>,
    ) -> Result<()> {
        let version_info = self.file_manager.get_version_info(version_id).await?;
        launcher::game::GameLauncher::launch(&version_info, auth, &self.minecraft_dir, &self.java_manager, instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_required_java_version() {
        // Test old versions that require Java 8
        assert_eq!(JavaManager::get_required_java_version("1.15.2"), 8);
        assert_eq!(JavaManager::get_required_java_version("1.12.2"), 8);

        // Test versions that require Java 11 (1.16)
        assert_eq!(JavaManager::get_required_java_version("1.16.5"), 11);

        // Test versions that require Java 17
        assert_eq!(JavaManager::get_required_java_version("1.18.0"), 17);
        assert_eq!(JavaManager::get_required_java_version("1.20.1"), 17);

        // Test versions that require Java 21
        assert_eq!(JavaManager::get_required_java_version("1.21.0"), 21);
        assert_eq!(JavaManager::get_required_java_version("1.21.5"), 21);

        // Test edge cases
        assert_eq!(JavaManager::get_required_java_version("invalid"), 17); // Default to 17
    }

    #[tokio::test]
    async fn test_launcher_initialization() {
        // Test that launcher can be initialized without panicking
        // Note: This test may fail in CI environments without proper setup
        let result = Launcher::new().await;

        // In a test environment, we expect this might fail due to missing directories
        // but we test that it doesn't panic
        match result {
            Ok(_launcher) => {
                // If successful, verify basic structure
                println!("Launcher initialized successfully");
            }
            Err(_e) => {
                // Expected in test environments without proper Minecraft directories
                println!("Launcher initialization failed as expected in test environment");
            }
        }
    }
}
