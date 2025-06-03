use anyhow::{Context, Result, anyhow};
use log::{debug, warn};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::launcher;

#[derive(Debug, Clone)]
pub struct JavaInstallation {
    pub path: PathBuf,
    pub major_version: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub struct JavaManager {
    pub installations: HashMap<u32, JavaInstallation>,
}

impl JavaManager {
    pub fn new() -> Self {
        Self {
            installations: HashMap::new(),
        }
    }

    /// Initialize the Java manager by scanning for Java installations
    pub fn initialize(&mut self) {
        debug!("Scanning for Java installations...");
        self.scan_java_installations();

        if self.installations.is_empty() {
            warn!(
                "No Java installations found! Please ensure Java is installed and available in PATH or JAVA_HOME"
            );
        } else {
            debug!("Found {} Java installation(s)", self.installations.len());
            for (major, installation) in &self.installations {
                debug!("Java {}: {}", major, installation.path.display());
            }
        }
    }

    /// Get the required Java version for a specific Minecraft version
    pub fn get_required_java_version(minecraft_version: &str) -> u32 {
        // Parse the Minecraft version to determine required Java version
        if let Some(version_parts) = parse_minecraft_version(minecraft_version) {
            let (major, minor) = version_parts;

            match (major, minor) {
                // Minecraft 1.21+ requires Java 21
                (1, minor) if minor >= 21 => 21,
                // Minecraft 1.18+ requires Java 17
                (1, minor) if minor >= 18 => 17,
                // Minecraft 1.17 requires Java 16
                (1, 17) => 16,
                // Minecraft 1.16 can use Java 8 or 11, prefer 11
                (1, 16) => 11,
                // Minecraft 1.15 and below require Java 8
                (1, minor) if minor <= 15 => 8,
                // Default to Java 17 for unknown versions
                _ => 17,
            }
        } else {
            // Default to Java 17 for unparseable versions
            17
        }
    }

    /// Get the best Java installation for a Minecraft version
    pub fn get_java_for_minecraft(&self, minecraft_version: &str) -> Result<&JavaInstallation> {
        let required_version = Self::get_required_java_version(minecraft_version);

        // First, try to find the exact required version
        if let Some(installation) = self.installations.get(&required_version) {
            debug!("Using Java {required_version} for Minecraft {minecraft_version}");
            return Ok(installation);
        }

        // If exact version not found, try to find a compatible higher version
        let mut compatible_versions: Vec<_> = self
            .installations
            .iter()
            .filter(|(major, _)| **major >= required_version)
            .collect();

        compatible_versions.sort_by_key(|(major, _)| *major);

        if let Some((major, installation)) = compatible_versions.first() {
            warn!(
                "Required Java {required_version} not found for Minecraft {minecraft_version}, using Java {major} instead"
            );
            return Ok(installation);
        }

        // If no compatible version found, use the highest available version
        let mut all_versions: Vec<_> = self.installations.iter().collect();
        all_versions.sort_by_key(|(major, _)| *major);

        if let Some((major, installation)) = all_versions.last() {
            warn!(
                "No compatible Java version found for Minecraft {minecraft_version} (requires Java {required_version}), using Java {major} - this may not work!"
            );
            return Ok(installation);
        }

        Err(anyhow!(
            "No Java installations found! Please install Java {} or higher for Minecraft {}",
            required_version,
            minecraft_version
        ))
    }

    /// Scan for Java installations in common locations
    fn scan_java_installations(&mut self) {
        // Check JAVA_HOME first
        if let Ok(java_home) = env::var("JAVA_HOME") {
            let java_path = PathBuf::from(java_home).join("bin").join(if cfg!(windows) {
                "java.exe"
            } else {
                "java"
            });

            if let Ok(installation) =
                launcher::java::JavaManager::probe_java_installation(&java_path)
            {
                debug!("Found Java via JAVA_HOME: {}", installation.path.display());
                self.installations
                    .insert(installation.major_version, installation);
            }
        }

        // Check system PATH
        let java_executable = if cfg!(windows) { "java.exe" } else { "java" };
        if let Ok(installation) =
            launcher::java::JavaManager::probe_java_installation_by_name(java_executable)
        {
            if let std::collections::hash_map::Entry::Vacant(e) =
                self.installations.entry(installation.major_version)
            {
                debug!("Found Java via PATH: {}", installation.path.display());
                e.insert(installation);
            }
        }

        // Check common installation directories
        self.scan_common_java_directories();
    }

    /// Scan common Java installation directories
    fn scan_common_java_directories(&mut self) {
        let common_paths = if cfg!(windows) {
            vec![
                r"C:\Program Files\Java",
                r"C:\Program Files (x86)\Java",
                r"C:\Program Files\Eclipse Adoptium",
                r"C:\Program Files\AdoptOpenJDK",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "/Library/Java/JavaVirtualMachines",
                "/System/Library/Java/JavaVirtualMachines",
                "/usr/libexec/java_home",
            ]
        } else {
            vec!["/usr/lib/jvm", "/usr/java", "/opt/java", "/opt/jdk"]
        };

        for base_path in common_paths {
            if let Ok(entries) = std::fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        let java_path = entry.path().join("bin").join(if cfg!(windows) {
                            "java.exe"
                        } else {
                            "java"
                        });

                        if let Ok(installation) =
                            launcher::java::JavaManager::probe_java_installation(&java_path)
                        {
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                self.installations.entry(installation.major_version)
                            {
                                debug!(
                                    "Found Java in common directory: {}",
                                    installation.path.display()
                                );
                                e.insert(installation);
                            }
                        }
                    }
                }
            }
        }

        // Special handling for macOS java_home
        if cfg!(target_os = "macos") {
            self.scan_macos_java_home();
        }
    }

    /// Scan Java installations using macOS `java_home` utility
    fn scan_macos_java_home(&mut self) {
        let versions = ["8", "11", "16", "17", "21"];

        for version in &versions {
            if let Ok(output) = Command::new("/usr/libexec/java_home")
                .arg("-v")
                .arg(version)
                .output()
            {
                if output.status.success() {
                    let java_home = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let java_path = PathBuf::from(java_home).join("bin").join("java");

                    if let Ok(installation) =
                        launcher::java::JavaManager::probe_java_installation(&java_path)
                    {
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            self.installations.entry(installation.major_version)
                        {
                            debug!("Found Java via java_home: {}", installation.path.display());
                            e.insert(installation);
                        }
                    }
                }
            }
        }
    }

    /// Probe a Java installation by executable name
    fn probe_java_installation_by_name(executable: &str) -> Result<JavaInstallation> {
        if let Ok(output) = Command::new(executable).arg("-version").output() {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stderr);
                if let Some(version) = parse_java_version(&version_output) {
                    // Try to find the actual path
                    let path =
                        if let Ok(which_output) = Command::new("which").arg(executable).output() {
                            let path_str = String::from_utf8_lossy(&which_output.stdout)
                                .trim()
                                .to_string();
                            PathBuf::from(path_str)
                        } else {
                            PathBuf::from(executable)
                        };

                    return Ok(JavaInstallation {
                        path,
                        major_version: version.major,
                    });
                }
            }
        }

        Err(anyhow!("Failed to probe Java installation: {}", executable))
    }

    /// Probe a specific Java installation path
    fn probe_java_installation(java_path: &Path) -> Result<JavaInstallation> {
        if !java_path.exists() {
            return Err(anyhow!(
                "Java executable not found: {}",
                java_path.display()
            ));
        }

        let output = Command::new(java_path)
            .arg("-version")
            .output()
            .with_context(|| format!("Failed to execute Java: {}", java_path.display()))?;

        if !output.status.success() {
            return Err(anyhow!(
                "Java version check failed: {}",
                java_path.display()
            ));
        }

        let version_output = String::from_utf8_lossy(&output.stderr);
        let version = parse_java_version(&version_output)
            .ok_or_else(|| anyhow!("Failed to parse Java version: {}", version_output))?;

        Ok(JavaInstallation {
            path: java_path.to_path_buf(),
            major_version: version.major,
        })
    }
}

/// Parse Java version from version output
fn parse_java_version(version_output: &str) -> Option<JavaVersion> {
    // Java version output format varies, but we look for patterns like:
    // "1.8.0_333" (Java 8)
    // "11.0.16" (Java 11+)
    // "openjdk version "17.0.4""

    for line in version_output.lines() {
        if line.contains("version") {
            // Extract version string from quotes
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let version_str = &line[start + 1..start + 1 + end];

                    // Parse different version formats
                    if version_str.starts_with("1.") {
                        // Legacy format like "1.8.0_333"
                        let parts: Vec<&str> = version_str.split('.').collect();
                        if parts.len() >= 3 {
                            if let Ok(minor) = parts[1].parse::<u32>() {
                                return Some(JavaVersion {
                                    major: minor, // In "1.8", the actual version is 8
                                    minor: 0,
                                    patch: 0,
                                });
                            }
                        }
                    } else {
                        // Modern format like "17.0.4"
                        let parts: Vec<&str> = version_str.split('.').collect();
                        if !parts.is_empty() {
                            if let Ok(major) = parts[0].parse::<u32>() {
                                let minor = if parts.len() > 1 {
                                    parts[1].parse().unwrap_or(0)
                                } else {
                                    0
                                };
                                let patch = if parts.len() > 2 {
                                    parts[2].parse().unwrap_or(0)
                                } else {
                                    0
                                };

                                return Some(JavaVersion {
                                    major,
                                    minor,
                                    patch,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

#[allow(clippy::match_same_arms)]
/// Parse Minecraft version to extract major and minor version numbers
fn parse_minecraft_version(version: &str) -> Option<(u32, u32)> {
    // Handle versions like "1.20.4", "1.21", "24w14a" (snapshots)
    if version.contains('w') {
        // Snapshot format like "24w14a" - extract year and map to approximate release
        if let Some(year_str) = version.get(0..2) {
            if let Ok(year) = year_str.parse::<u32>() {
                // Map snapshot years to Minecraft versions (approximate)
                let major = 1;
                let minor = match year {
                    24 => 21, // 2024 snapshots are around 1.21
                    23 => 20, // 2023 snapshots are around 1.20
                    22 => 19, // 2022 snapshots are around 1.19
                    _ => 21,  // Default to recent version
                };
                return Some((major, minor));
            }
        }
    } else {
        // Regular version format
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                return Some((major, minor));
            }
        }
    }

    None
}

impl Default for JavaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_java_version() {
        let java8_output = r#"openjdk version "1.8.0_333""#;
        assert_eq!(
            parse_java_version(java8_output),
            Some(JavaVersion {
                major: 8,
                minor: 0,
                patch: 0
            })
        );

        let java17_output = r#"openjdk version "17.0.4""#;
        assert_eq!(
            parse_java_version(java17_output),
            Some(JavaVersion {
                major: 17,
                minor: 0,
                patch: 4
            })
        );

        let java21_output = r#"openjdk version "21.0.1""#;
        assert_eq!(
            parse_java_version(java21_output),
            Some(JavaVersion {
                major: 21,
                minor: 0,
                patch: 1
            })
        );
    }

    #[test]
    fn test_parse_minecraft_version() {
        assert_eq!(parse_minecraft_version("1.20.4"), Some((1, 20)));
        assert_eq!(parse_minecraft_version("1.21"), Some((1, 21)));
        assert_eq!(parse_minecraft_version("24w14a"), Some((1, 21)));
        assert_eq!(parse_minecraft_version("1.16.5"), Some((1, 16)));
    }

    #[test]
    fn test_required_java_version() {
        assert_eq!(JavaManager::get_required_java_version("1.21.5"), 21);
        assert_eq!(JavaManager::get_required_java_version("1.20.4"), 17);
        assert_eq!(JavaManager::get_required_java_version("1.18.2"), 17);
        assert_eq!(JavaManager::get_required_java_version("1.17.1"), 16);
        assert_eq!(JavaManager::get_required_java_version("1.16.5"), 11);
        assert_eq!(JavaManager::get_required_java_version("1.15.2"), 8);
        assert_eq!(JavaManager::get_required_java_version("24w14a"), 21);
    }
}
