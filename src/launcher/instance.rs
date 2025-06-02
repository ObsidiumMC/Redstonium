use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::launcher::minecraft_dir::MinecraftDir;

const MAX_INSTANCE_NAME_LEN: usize = 64;

/// Configuration for a Minecraft instance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceConfig {
    /// Instance name
    pub name: String,
    /// Minecraft version to use
    pub version: String,
    /// Instance description
    pub description: Option<String>,
    /// Creation timestamp
    pub created: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// Instance-specific settings
    pub settings: InstanceSettings,
    /// Mods configuration
    pub mods: ModsConfig,
}

/// Instance-specific settings
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InstanceSettings {
    /// Java arguments (additional to default)
    pub java_args: Vec<String>,
    /// Game arguments (additional to default)
    pub game_args: Vec<String>,
    /// Memory allocation in MB
    pub memory_mb: Option<u32>,
    /// Enable JVM debugging
    pub debug: bool,
    /// Custom server to connect to on launch
    pub server: Option<ServerConfig>,
}

/// Server configuration for quick connect
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: Option<u16>,
}

/// Mods configuration for the instance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModsConfig {
    /// Mod loader type (vanilla, forge, fabric, quilt)
    pub loader: ModLoader,
    /// Loader version
    pub loader_version: Option<String>,
    /// List of installed mods
    pub mods: Vec<ModInfo>,
}

/// Supported mod loaders
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
}

/// Information about an installed mod
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub name: String,
    pub version: String,
    pub file_name: String,
    pub enabled: bool,
}

/// Instance manager for handling multiple Minecraft instances
pub struct InstanceManager {
    minecraft_dir: MinecraftDir,
    instances_dir: PathBuf,
    instances: HashMap<String, InstanceConfig>,
}

impl InstanceManager {
    /// Create a new instance manager
    pub async fn new(minecraft_dir: MinecraftDir) -> Result<Self> {
        let instances_dir = minecraft_dir.base_path.join("instances");

        // Ensure instances directory exists
        if !instances_dir.exists() {
            fs::create_dir_all(&instances_dir)
                .await
                .context("Failed to create instances directory")?;
        }

        let mut manager = Self {
            minecraft_dir,
            instances_dir,
            instances: HashMap::new(),
        };

        // Load existing instances
        manager.load_instances().await?;

        Ok(manager)
    }

    /// Load all instances from disk
    async fn load_instances(&mut self) -> Result<()> {
        let mut entries = fs::read_dir(&self.instances_dir)
            .await
            .context("Failed to read instances directory")?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let config_path = path.join("instance.json");
                if config_path.exists() {
                    match self.load_instance_config(&config_path).await {
                        Ok(config) => {
                            debug!("Loaded instance: {}", config.name);
                            self.instances.insert(config.name.clone(), config);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to load instance config at {}: {}",
                                config_path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        info!("Loaded {} instances", self.instances.len());
        Ok(())
    }

    /// Load instance configuration from file
    async fn load_instance_config(&self, config_path: &Path) -> Result<InstanceConfig> {
        let content = fs::read_to_string(config_path)
            .await
            .context("Failed to read instance config")?;

        let config: InstanceConfig =
            serde_json::from_str(&content).context("Failed to parse instance config")?;

        Ok(config)
    }

    /// Save instance configuration to disk
    async fn save_instance_config(&self, config: &InstanceConfig) -> Result<()> {
        let instance_dir = self.get_instance_dir(&config.name);
        let config_path = instance_dir.join("instance.json");

        // Ensure instance directory exists
        if !instance_dir.exists() {
            fs::create_dir_all(&instance_dir)
                .await
                .context("Failed to create instance directory")?;
        }

        let content =
            serde_json::to_string_pretty(config).context("Failed to serialize instance config")?;

        fs::write(&config_path, content)
            .await
            .context("Failed to write instance config")?;

        Ok(())
    }

    /// Create a new instance (with version validation)
    pub async fn create_instance(
        &mut self,
        name: String,
        version: String,
        description: Option<String>,
        file_manager: &crate::launcher::FileManager,
    ) -> Result<()> {
        // Check if instance already exists
        if self.instances.contains_key(&name) {
            return Err(anyhow::anyhow!("Instance '{}' already exists", name));
        }

        // Validate instance name (alphanumeric, hyphens, underscores only)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(anyhow::anyhow!(
                "Instance name can only contain letters, numbers, hyphens, and underscores"
            ));
        }

        // Enforce a maximum instance name length (e.g., 64 chars)
        if name.len() > MAX_INSTANCE_NAME_LEN {
            return Err(anyhow::anyhow!(
                "Instance name is too long ({} characters). Maximum allowed is {} characters.",
                name.len(), MAX_INSTANCE_NAME_LEN
            ));
        }

        // Validate version exists in manifest
        let manifest = file_manager.get_version_manifest().await?;
        let valid_version = manifest.versions.iter().any(|v| v.id == version);
        if !valid_version {
            return Err(anyhow::anyhow!(
                "Minecraft version '{}' does not exist. Use 'rustified list' to see valid versions.",
                version
            ));
        }

        let config = InstanceConfig {
            name: name.clone(),
            version,
            description,
            created: chrono::Utc::now(),
            last_used: None,
            settings: InstanceSettings::default(),
            mods: ModsConfig::default(),
        };

        // Save to disk
        self.save_instance_config(&config).await?;

        // Create the game directory structure
        self.ensure_instance_directory(&name)
            .context("Failed to create instance game directories")?;

        // Add to memory
        self.instances.insert(name.clone(), config);

        info!("Created instance: {name} with game directories");
        Ok(())
    }

    /// Delete an instance
    pub async fn delete_instance(&mut self, name: &str) -> Result<()> {
        if !self.instances.contains_key(name) {
            return Err(anyhow::anyhow!("Instance '{}' does not exist", name));
        }

        // Remove from disk
        let instance_dir = self.get_instance_dir(name);
        if instance_dir.exists() {
            fs::remove_dir_all(&instance_dir)
                .await
                .context("Failed to delete instance directory")?;
        }

        // Remove from memory
        self.instances.remove(name);

        info!("Deleted instance: {name}");
        Ok(())
    }

    /// Get an instance configuration
    pub fn get_instance(&self, name: &str) -> Option<&InstanceConfig> {
        self.instances.get(name)
    }

    /// List all instances
    pub fn list_instances(&self) -> Vec<&InstanceConfig> {
        self.instances.values().collect()
    }

    /// Update an instance's last used timestamp
    pub async fn update_last_used(&mut self, name: &str) -> Result<()> {
        if let Some(config) = self.instances.get_mut(name) {
            config.last_used = Some(chrono::Utc::now());
            // Clone the config to avoid borrow checker issues
            let config_clone = config.clone();
            self.save_instance_config(&config_clone).await?;
        }
        Ok(())
    }

    /// Get the directory path for an instance
    pub fn get_instance_dir(&self, name: &str) -> PathBuf {
        self.instances_dir.join(name)
    }

    /// Set instance memory allocation
    pub async fn set_instance_memory(&mut self, name: &str, memory_mb: u32) -> Result<()> {
        // Set a reasonable upper bound for memory (e.g., 128 GB)
        const MAX_MEMORY_MB: u32 = 128 * 1024; // 131072 MB
        if memory_mb == 0 {
            return Err(anyhow::anyhow!("Memory must be greater than 0 MB"));
        }
        if memory_mb > MAX_MEMORY_MB {
            return Err(anyhow::anyhow!(
                "Memory value too large ({} MB). Maximum allowed is {} MB (128 GB)",
                memory_mb, MAX_MEMORY_MB
            ));
        }
        if let Some(config) = self.instances.get_mut(name) {
            config.settings.memory_mb = Some(memory_mb);
            // Clone the config to avoid borrow checker issues
            let config_clone = config.clone();
            self.save_instance_config(&config_clone).await?;
            info!("Set memory for instance '{name}' to {memory_mb}MB");
        } else {
            return Err(anyhow::anyhow!("Instance '{}' does not exist", name));
        }
        Ok(())
    }

    /// Create instance game directory and ensure it's properly set up
    pub fn ensure_instance_directory(&self, name: &str) -> Result<PathBuf> {
        let instance_dir = self.minecraft_dir.base_path.join("instances").join(name);

        // Create the instance directory structure
        std::fs::create_dir_all(&instance_dir).with_context(|| {
            format!(
                "Failed to create instance directory: {}",
                instance_dir.display()
            )
        })?;

        // Create subdirectories that Minecraft expects
        let subdirs = [
            "saves",
            "resourcepacks",
            "screenshots",
            "logs",
            "crash-reports",
        ];
        for subdir in &subdirs {
            let path = instance_dir.join(subdir);
            std::fs::create_dir_all(&path)
                .with_context(|| format!("Failed to create {subdir} directory"))?;
        }

        // Create options.txt with basic settings if it doesn't exist
        let options_file = instance_dir.join("options.txt");
        if !options_file.exists() {
            let default_options = "version:3343\nlang:en_us\n";
            std::fs::write(&options_file, default_options)
                .with_context(|| "Failed to create default options.txt")?;
        }

        Ok(instance_dir)
    }
}

impl Default for ModsConfig {
    fn default() -> Self {
        Self {
            loader: ModLoader::Vanilla,
            loader_version: None,
            mods: Vec::new(),
        }
    }
}
