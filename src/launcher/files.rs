use crate::error::{FileManagerError, Result, ResultExt};
use log::{debug, info, warn};
use reqwest::Client;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

use crate::launcher::minecraft_dir::MinecraftDir;
use crate::launcher::version::{AssetManifest, Library, VersionInfo, VersionManifest};

const VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
const RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net";

pub struct FileManager {
    client: Client,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch the version manifest from Mojang
    pub async fn get_version_manifest(&self) -> Result<VersionManifest> {
        info!("Fetching version manifest from {VERSION_MANIFEST_URL}");

        let response = self
            .client
            .get(VERSION_MANIFEST_URL)
            .send()
            .await
            .context("Failed to fetch version manifest")?;

        if !response.status().is_success() {
            return Err(FileManagerError::download_failed(format!(
                "Failed to fetch version manifest: HTTP {}",
                response.status()
            ))
            .into());
        }

        let manifest: VersionManifest = response
            .json()
            .await
            .context("Failed to parse version manifest JSON")?;

        info!(
            "Successfully fetched version manifest with {} versions",
            manifest.versions.len()
        );
        Ok(manifest)
    }

    /// Get version info for a specific version
    pub async fn get_version_info(&self, version_id: &str) -> Result<VersionInfo> {
        info!("Getting version info for {version_id}");

        // First get the version manifest to find the URL
        let manifest = self.get_version_manifest().await?;

        let version_entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| {
                FileManagerError::version_not_found(&format!(
                    "Version {version_id} not found in manifest"
                ))
            })?;

        info!("Fetching version info from {}", version_entry.url);

        let response = self
            .client
            .get(&version_entry.url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch version info for {version_id}"))?;

        if !response.status().is_success() {
            return Err(FileManagerError::download_failed(format!(
                "Failed to fetch version info: HTTP {}",
                response.status()
            ))
            .into());
        }

        let version_info: VersionInfo = response
            .json()
            .await
            .with_context(|| format!("Failed to parse version info JSON for {version_id}"))?;

        // Check minimum launcher version if present
        if let Some(min_version) = version_info.minimum_launcher_version {
            const LAUNCHER_VERSION: u32 = 1; // Our launcher version
            if LAUNCHER_VERSION < min_version {
                warn!(
                    "This launcher version ({LAUNCHER_VERSION}) may be incompatible with Minecraft {version_id} (requires version {min_version})"
                );
            }
        }

        info!("Successfully fetched version info for {version_id}");
        Ok(version_info)
    }

    /// Download the main game JAR file
    pub async fn download_game_jar(
        &self,
        version_info: &VersionInfo,
        minecraft_dir: &MinecraftDir,
    ) -> Result<()> {
        let jar_path = minecraft_dir.version_jar_path(&version_info.id);
        let json_path = minecraft_dir.version_json_path(&version_info.id);

        // Save version JSON first
        let version_json = serde_json::to_string_pretty(version_info)
            .context("Failed to serialize version info")?;

        fs::write(&json_path, version_json)
            .await
            .with_context(|| format!("Failed to write version JSON to {}", json_path.display()))?;

        // Download JAR if not already present and valid
        if self
            .is_file_valid(&jar_path, &version_info.downloads.client.sha1)
            .await?
        {
            info!("Game JAR already exists and is valid");
        } else {
            info!("Downloading game JAR for {}", version_info.id);

            self.download_file_with_verification(
                &version_info.downloads.client.url,
                &jar_path,
                &version_info.downloads.client.sha1,
                version_info.downloads.client.size,
            )
            .await
            .with_context(|| format!("Failed to download game JAR for {}", version_info.id))?;

            info!("✓ Game JAR downloaded successfully");
        }

        Ok(())
    }

    /// Download all required libraries
    pub async fn download_libraries(
        &self,
        version_info: &VersionInfo,
        minecraft_dir: &MinecraftDir,
    ) -> Result<()> {
        info!("Downloading libraries for {}", version_info.id);

        let mut total_libraries = 0;
        let mut downloaded_libraries = 0;
        let mut skipped_libraries = 0;

        for library in &version_info.libraries {
            if !library.should_use() {
                debug!("Skipping library {} (platform rules)", library.name);
                skipped_libraries += 1;
                continue;
            }

            total_libraries += 1;

            if library.is_native_library() {
                downloaded_libraries += self
                    .download_native_library(library, version_info, minecraft_dir)
                    .await?;
                continue;
            }

            downloaded_libraries += self
                .download_regular_library(library, minecraft_dir)
                .await?;

            downloaded_libraries += self
                .download_legacy_native(library, version_info, minecraft_dir)
                .await?;
        }

        info!(
            "✓ Libraries processed: {downloaded_libraries} downloaded, {skipped_libraries} skipped, {total_libraries} total"
        );
        Ok(())
    }

    // Helper for native libraries
    async fn download_native_library(
        &self,
        library: &Library,
        version_info: &VersionInfo,
        minecraft_dir: &MinecraftDir,
    ) -> Result<u32> {
        if let Some(artifact) = &library.downloads.artifact {
            let lib_path = get_library_path(&library.name);
            let full_path = minecraft_dir.library_path(&lib_path);

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await.with_context(|| {
                    format!(
                        "Failed to create native library directory: {}",
                        parent.display()
                    )
                })?;
            }

            if self.is_file_valid(&full_path, &artifact.sha1).await? {
                debug!(
                    "Native library {} already exists and is valid",
                    library.name
                );
                // Still need to extract if natives directory doesn't exist
                let natives_dir = minecraft_dir.natives_dir(&version_info.id);
                if !natives_dir.exists() {
                    self.extract_natives(&full_path, &natives_dir, library)
                        .await
                        .with_context(|| {
                            format!("Failed to extract natives from {}", library.name)
                        })?;
                }
                Ok(0)
            } else {
                debug!("Downloading native library: {}", library.name);

                self.download_file_with_verification(
                    &artifact.url,
                    &full_path,
                    &artifact.sha1,
                    artifact.size,
                )
                .await
                .with_context(|| format!("Failed to download native library: {}", library.name))?;

                // Extract natives from the JAR
                self.extract_natives(
                    &full_path,
                    &minecraft_dir.natives_dir(&version_info.id),
                    library,
                )
                .await
                .with_context(|| format!("Failed to extract natives from {}", library.name))?;

                Ok(1)
            }
        } else {
            Ok(0)
        }
    }

    // Helper for regular libraries
    async fn download_regular_library(
        &self,
        library: &Library,
        minecraft_dir: &MinecraftDir,
    ) -> Result<u32> {
        if let Some(artifact) = &library.downloads.artifact {
            let lib_path = get_library_path(&library.name);
            let full_path = minecraft_dir.library_path(&lib_path);

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await.with_context(|| {
                    format!("Failed to create library directory: {}", parent.display())
                })?;
            }

            if self.is_file_valid(&full_path, &artifact.sha1).await? {
                debug!("Library {} already exists and is valid", library.name);
                Ok(0)
            } else {
                debug!("Downloading library: {}", library.name);

                self.download_file_with_verification(
                    &artifact.url,
                    &full_path,
                    &artifact.sha1,
                    artifact.size,
                )
                .await
                .with_context(|| format!("Failed to download library: {}", library.name))?;

                Ok(1)
            }
        } else {
            Ok(0)
        }
    }

    // Helper for legacy natives
    async fn download_legacy_native(
        &self,
        library: &Library,
        version_info: &VersionInfo,
        minecraft_dir: &MinecraftDir,
    ) -> Result<u32> {
        if let (Some(classifiers), Some(native_classifier)) = (
            &library.downloads.classifiers,
            library.get_native_classifier(),
        ) {
            if let Some(native_download) = classifiers.get(&native_classifier) {
                let lib_path = get_library_path(&format!("{}:{}", library.name, native_classifier));
                let full_path = minecraft_dir.library_path(&lib_path);

                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent).await.with_context(|| {
                        format!(
                            "Failed to create native library directory: {}",
                            parent.display()
                        )
                    })?;
                }

                if !self
                    .is_file_valid(&full_path, &native_download.sha1)
                    .await?
                {
                    debug!(
                        "Downloading legacy native library: {}-{}",
                        library.name, native_classifier
                    );

                    self.download_file_with_verification(
                        &native_download.url,
                        &full_path,
                        &native_download.sha1,
                        native_download.size,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to download native library: {}-{}",
                            library.name, native_classifier
                        )
                    })?;

                    // Extract natives
                    self.extract_natives(
                        &full_path,
                        &minecraft_dir.natives_dir(&version_info.id),
                        library,
                    )
                    .await
                    .with_context(|| format!("Failed to extract natives from {}", library.name))?;
                    return Ok(1);
                }
            }
        }
        Ok(0)
    }

    /// Download game assets with concurrent processing
    pub async fn download_assets(
        &self,
        version_info: &VersionInfo,
        minecraft_dir: &MinecraftDir,
    ) -> Result<()> {
        const BATCH_SIZE: usize = 50;

        info!("Downloading assets for {}", version_info.id);

        // Download asset index
        let asset_index_path = minecraft_dir.asset_index_path(&version_info.asset_index.id);

        if let Some(parent) = asset_index_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("Failed to create asset indexes directory")?;
        }

        if self
            .is_file_valid(&asset_index_path, &version_info.asset_index.sha1)
            .await?
        {
            info!("Asset index already exists and is valid");
        } else {
            info!("Downloading asset index: {}", version_info.asset_index.id);

            self.download_file_with_verification(
                &version_info.asset_index.url,
                &asset_index_path,
                &version_info.asset_index.sha1,
                version_info.asset_index.size,
            )
            .await
            .context("Failed to download asset index")?;
        }

        // Parse asset index
        let asset_index_content = fs::read_to_string(&asset_index_path)
            .await
            .context("Failed to read asset index")?;

        let asset_manifest: AssetManifest = serde_json::from_str(&asset_index_content)
            .context("Failed to parse asset index JSON")?;

        // Download individual assets with concurrency
        let total_assets = asset_manifest.objects.len();
        info!("Processing {total_assets} assets with concurrent downloads...");

        // Process assets in batches to avoid overwhelming the server
        let mut downloaded_assets = 0;
        let mut skipped_assets = 0;

        let assets: Vec<_> = asset_manifest.objects.iter().collect();

        for batch in assets.chunks(BATCH_SIZE) {
            let download_futures = batch.iter().map(|(asset_name, asset_object)| {
                let asset_path = minecraft_dir.asset_path(&asset_object.hash);
                let asset_url = format!(
                    "{}/{}/{}",
                    RESOURCES_BASE_URL,
                    &asset_object.hash[..2],
                    &asset_object.hash
                );

                async move {
                    // Create asset directory if needed
                    if let Some(parent) = asset_path.parent() {
                        if let Err(e) = fs::create_dir_all(parent).await {
                            return Err(crate::error::RustifiedError::FileManager(
                                FileManagerError::filesystem_error(format!(
                                    "Failed to create asset directory {}: {}",
                                    parent.display(),
                                    e
                                )),
                            ));
                        }
                    }

                    // Check if file already exists and is valid
                    if self
                        .is_file_valid(&asset_path, &asset_object.hash)
                        .await
                        .unwrap_or(false)
                    {
                        return Ok(false); // File already exists
                    }

                    // Download the asset
                    self.download_file_with_verification(
                        &asset_url,
                        &asset_path,
                        &asset_object.hash,
                        asset_object.size,
                    )
                    .await
                    .with_context(|| format!("Failed to download asset: {asset_name}"))?;

                    Ok(true) // File was downloaded
                }
            });

            // Execute downloads concurrently
            let results = futures_util::future::join_all(download_futures).await;

            for result in results {
                match result {
                    Ok(true) => downloaded_assets += 1,
                    Ok(false) => skipped_assets += 1,
                    Err(e) => {
                        warn!("Asset download failed: {e}");
                        // Continue with other assets instead of failing completely
                    }
                }
            }

            // Progress update
            let processed = downloaded_assets + skipped_assets;
            if processed % 100 == 0 || processed == total_assets {
                info!("Asset progress: {processed}/{total_assets} processed");
            }
        }

        info!(
            "✓ Assets processed: {downloaded_assets} downloaded, {skipped_assets} skipped, {total_assets} total"
        );
        Ok(())
    }

    /// Download a file with SHA1 verification
    async fn download_file_with_verification(
        &self,
        url: &str,
        path: &Path,
        expected_sha1: &str,
        expected_size: u64,
    ) -> Result<()> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to start download from {url}"))?;

        if !response.status().is_success() {
            return Err(FileManagerError::download_failed(format!(
                "Download failed: HTTP {}",
                response.status()
            ))
            .into());
        }

        let mut file = fs::File::create(path)
            .await
            .with_context(|| format!("Failed to create file: {}", path.display()))?;

        let mut hasher = Sha1::new();

        // Read the response body in chunks
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("Failed to read response from {url}"))?;

        file.write_all(&bytes)
            .await
            .with_context(|| format!("Failed to write to file: {}", path.display()))?;

        hasher.update(&bytes);
        let downloaded = bytes.len() as u64;

        file.flush()
            .await
            .with_context(|| format!("Failed to flush file: {}", path.display()))?;

        // Verify size
        if downloaded != expected_size {
            return Err(FileManagerError::validation_failed(format!(
                "Size mismatch: expected {expected_size}, got {downloaded}"
            ))
            .into());
        }

        // Verify SHA1
        let actual_sha1 = format!("{:x}", hasher.finalize());
        if actual_sha1 != expected_sha1 {
            return Err(FileManagerError::validation_failed(format!(
                "SHA1 mismatch: expected {expected_sha1}, got {actual_sha1}"
            ))
            .into());
        }

        Ok(())
    }

    /// Check if a file exists and has the correct SHA1 hash
    async fn is_file_valid(&self, path: &Path, expected_sha1: &str) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let Ok(content) = fs::read(path).await else {
            return Ok(false);
        };

        let mut hasher = Sha1::new();
        hasher.update(&content);
        let actual_sha1 = format!("{:x}", hasher.finalize());

        Ok(actual_sha1 == expected_sha1)
    }

    /// Extract native libraries from JAR files
    async fn extract_natives(
        &self,
        jar_path: &Path,
        natives_dir: &Path,
        library: &Library,
    ) -> Result<()> {
        debug!(
            "Extracting natives from {} to {}",
            jar_path.display(),
            natives_dir.display()
        );

        // Create natives directory
        fs::create_dir_all(natives_dir).await.with_context(|| {
            format!(
                "Failed to create natives directory: {}",
                natives_dir.display()
            )
        })?;

        // Open the JAR file (which is actually a ZIP file)
        let file = File::open(jar_path)
            .with_context(|| format!("Failed to open JAR file: {}", jar_path.display()))?;

        let mut archive = ZipArchive::new(file).with_context(|| {
            format!("Failed to read JAR as ZIP archive: {}", jar_path.display())
        })?;

        // Extract files from the archive
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .with_context(|| format!("Failed to read file at index {i} from JAR"))?;

            let file_path = file.name().to_string(); // Clone the path to avoid borrow issues

            // Skip directories and META-INF files
            if file_path.ends_with('/') || file_path.starts_with("META-INF/") {
                continue;
            }

            // Check if we should exclude this file based on library extract rules
            if let Some(extract_rules) = &library.extract {
                if let Some(exclude_patterns) = &extract_rules.exclude {
                    if exclude_patterns
                        .iter()
                        .any(|pattern| file_path.contains(pattern))
                    {
                        debug!("Excluding file {file_path} from extraction");
                        continue;
                    }
                }
            }

            // Create the full output path
            let output_path = natives_dir.join(&file_path);

            // Create parent directories if needed
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            // Extract the file
            let mut output_file = std::fs::File::create(&output_path)
                .with_context(|| format!("Failed to create file: {}", output_path.display()))?;

            std::io::copy(&mut file, &mut output_file)
                .with_context(|| format!("Failed to extract file: {file_path}"))?;

            debug!("Extracted: {file_path}");
        }

        debug!("✓ Native extraction completed for {}", library.name);
        Ok(())
    }
}

/// Helper function to convert library name to file path
/// Example: org.lwjgl:lwjgl:3.3.3 -> org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3.jar
/// Example: org.lwjgl:lwjgl-opengl:3.3.3:natives-macos-arm64 -> org/lwjgl/lwjgl-opengl/3.3.3/lwjgl-opengl-3.3.3-natives-macos-arm64.jar
pub fn get_library_path(library_name: &str) -> String {
    // Parse library name like "com.mojang:brigadier:1.0.18" or "org.lwjgl:lwjgl-freetype:3.3.3:natives-macos-arm64"
    let parts: Vec<&str> = library_name.split(':').collect();
    if parts.len() >= 4 {
        // Handle classifier format (e.g., org.lwjgl:lwjgl-freetype:3.3.3:natives-macos-arm64)
        let group = parts[0].replace('.', "/");
        let name = parts[1];
        let version = parts[2];
        let classifier = parts[3];

        format!("{group}/{name}/{version}/{name}-{version}-{classifier}.jar")
    } else if parts.len() >= 3 {
        // Standard format (e.g., com.mojang:brigadier:1.0.18)
        let group = parts[0].replace('.', "/");
        let name = parts[1];
        let version = parts[2];

        format!("{group}/{name}/{version}/{name}-{version}.jar")
    } else {
        // Fallback for malformed library names
        format!("{}.jar", library_name.replace(':', "/"))
    }
}

// Add these additional dependencies to Cargo.toml:
// futures-util = "0.3"
// sha1 = "0.10"

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}
