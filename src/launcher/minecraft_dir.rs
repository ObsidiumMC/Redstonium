use anyhow::{Context, Result, anyhow};
use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub struct MinecraftDir {
    pub base_path: PathBuf,
}

impl MinecraftDir {
    pub fn new() -> Result<Self> {
        let base_path = Self::get_minecraft_dir()?;
        std::fs::create_dir_all(&base_path)?;

        Ok(Self { base_path })
    }

    /// Get the platform-specific .minecraft directory path
    fn get_minecraft_dir() -> Result<PathBuf> {
        match env::consts::OS {
            "windows" => {
                let appdata =
                    env::var("APPDATA").context("APPDATA environment variable not found")?;
                Ok(PathBuf::from(appdata).join(".minecraft"))
            }
            "macos" => {
                let home = env::var("HOME").context("HOME environment variable not found")?;
                Ok(PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("minecraft"))
            }
            "linux" => {
                let home = env::var("HOME").context("HOME environment variable not found")?;
                Ok(PathBuf::from(home).join(".minecraft"))
            }
            _ => Err(anyhow!("Unsupported operating system: {}", env::consts::OS)),
        }
    }

    /// Get the versions directory path
    pub fn versions_dir(&self) -> PathBuf {
        self.base_path.join("versions")
    }

    /// Get the libraries directory path
    pub fn libraries_dir(&self) -> PathBuf {
        self.base_path.join("libraries")
    }

    /// Get the assets directory path
    pub fn assets_dir(&self) -> PathBuf {
        self.base_path.join("assets")
    }

    /// Get the assets objects directory path
    pub fn assets_objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    /// Get the assets indexes directory path
    pub fn assets_indexes_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    /// Get the path for a specific version directory
    pub fn version_dir(&self, version_id: &str) -> PathBuf {
        self.versions_dir().join(version_id)
    }

    /// Get the path for a version's JAR file
    pub fn version_jar_path(&self, version_id: &str) -> PathBuf {
        self.version_dir(version_id)
            .join(format!("{version_id}.jar"))
    }

    /// Get the path for a version's JSON file
    pub fn version_json_path(&self, version_id: &str) -> PathBuf {
        self.version_dir(version_id)
            .join(format!("{version_id}.json"))
    }

    /// Get the path for an asset by its hash
    pub fn asset_path(&self, hash: &str) -> PathBuf {
        let prefix = &hash[..2];
        self.assets_objects_dir().join(prefix).join(hash)
    }

    /// Get the path for an asset index file
    pub fn asset_index_path(&self, asset_id: &str) -> PathBuf {
        self.assets_indexes_dir().join(format!("{asset_id}.json"))
    }

    /// Get the path for a library
    pub fn library_path(&self, library_path: &str) -> PathBuf {
        self.libraries_dir().join(library_path)
    }

    /// Get the natives directory for a version
    pub fn natives_dir(&self, version_id: &str) -> PathBuf {
        self.version_dir(version_id).join("natives")
    }

    /// Ensure a specific version directory exists
    pub fn ensure_version_dir(&self, version_id: &str) -> Result<()> {
        let version_dir = self.version_dir(version_id);
        let natives_dir = self.natives_dir(version_id);

        std::fs::create_dir_all(&version_dir).with_context(|| {
            format!(
                "Failed to create version directory: {}",
                version_dir.display()
            )
        })?;

        std::fs::create_dir_all(&natives_dir).with_context(|| {
            format!(
                "Failed to create natives directory: {}",
                natives_dir.display()
            )
        })?;

        Ok(())
    }

    /// Check if a version is installed (has both JAR and JSON files)
    pub fn is_version_installed(&self, version_id: &str) -> bool {
        let jar_path = self.version_jar_path(version_id);
        let json_path = self.version_json_path(version_id);

        jar_path.exists() && json_path.exists()
    }
}
