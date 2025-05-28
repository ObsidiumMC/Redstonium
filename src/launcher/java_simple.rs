// Simple Java manager for debugging
use anyhow::{Result, anyhow};
use log::{info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct JavaInstallation {
    pub path: PathBuf,
    pub major_version: u32,
}

pub struct JavaManager {
    installations: HashMap<u32, JavaInstallation>,
}

impl JavaManager {
    pub fn new() -> Self {
        Self {
            installations: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        info!("Scanning for Java installations...");
        
        // Simple Java detection - just use system java for now
        if let Ok(output) = Command::new("java").arg("-version").output() {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stderr);
                if let Some(major_version) = Self::parse_java_version(&version_output) {
                    let installation = JavaInstallation {
                        path: PathBuf::from("java"),
                        major_version,
                    };
                    self.installations.insert(major_version, installation);
                    info!("Found Java {} in system PATH", major_version);
                } else {
                    warn!("Could not parse Java version from: {}", version_output);
                }
            }
        }
        
        if self.installations.is_empty() {
            warn!("No Java installations found!");
        }
        
        Ok(())
    }

    pub fn get_required_java_version(minecraft_version: &str) -> u32 {
        // Simple version mapping
        if minecraft_version.starts_with("1.21") { 21 }
        else if minecraft_version.starts_with("1.20") || minecraft_version.starts_with("1.19") || minecraft_version.starts_with("1.18") { 17 }
        else if minecraft_version.starts_with("1.17") { 16 }
        else if minecraft_version.starts_with("1.16") { 11 }
        else { 8 }
    }

    pub fn get_java_for_minecraft(&self, minecraft_version: &str) -> Result<&JavaInstallation> {
        let required_version = Self::get_required_java_version(minecraft_version);
        
        // Try to find exact version
        if let Some(installation) = self.installations.get(&required_version) {
            return Ok(installation);
        }

        // Find any compatible version (higher is OK)
        let mut compatible: Vec<_> = self.installations
            .iter()
            .filter(|(major, _)| **major >= required_version)
            .collect();
        
        if !compatible.is_empty() {
            compatible.sort_by_key(|(major, _)| *major);
            let (major, installation) = compatible[0];
            warn!("Using Java {} instead of required Java {} for Minecraft {}", 
                  major, required_version, minecraft_version);
            return Ok(installation);
        }

        // Use any available Java version as fallback
        if let Some((major, installation)) = self.installations.iter().next() {
            warn!("No compatible Java found, using Java {} for Minecraft {} (may not work!)", 
                  major, minecraft_version);
            return Ok(installation);
        }

        Err(anyhow!("No Java installation found!"))
    }

    fn parse_java_version(version_output: &str) -> Option<u32> {
        for line in version_output.lines() {
            if line.contains("version") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let version_str = &line[start + 1..start + 1 + end];
                        
                        if version_str.starts_with("1.") {
                            // Legacy format like "1.8.0_333"
                            let parts: Vec<&str> = version_str.split('.').collect();
                            if parts.len() >= 2 {
                                if let Ok(minor) = parts[1].parse::<u32>() {
                                    return Some(minor);
                                }
                            }
                        } else {
                            // Modern format like "17.0.4"
                            let parts: Vec<&str> = version_str.split('.').collect();
                            if !parts.is_empty() {
                                if let Ok(major) = parts[0].parse::<u32>() {
                                    return Some(major);
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
