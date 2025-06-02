use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: Option<u32>,
    pub downloads: Downloads,
    pub libraries: Vec<Library>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minecraftArguments", default)]
    pub minecraft_arguments: Option<String>,
    pub arguments: Option<Arguments>,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Downloads {
    pub client: DownloadInfo,
    pub server: Option<DownloadInfo>,
    #[serde(rename = "client_mappings")]
    pub client_mappings: Option<DownloadInfo>,
    #[serde(rename = "server_mappings")]
    pub server_mappings: Option<DownloadInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Library {
    pub name: String,
    pub downloads: LibraryDownloads,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<ExtractRules>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadInfo>,
    pub classifiers: Option<HashMap<String, DownloadInfo>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExtractRules {
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Arguments {
    pub game: Option<Vec<ArgumentValue>>,
    pub jvm: Option<Vec<ArgumentValue>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ArgumentValue {
    Simple(String),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValueType,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ArgumentValueType {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AssetManifest {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

impl Library {
    /// Check if this library should be used on the current platform
    pub fn should_use(&self) -> bool {
        if let Some(rules) = &self.rules {
            let mut allowed = false; // Start with disallow by default if there are rules

            for rule in rules {
                if Library::rule_matches(rule) {
                    allowed = rule.action == "allow";
                }
            }

            allowed
        } else {
            true // No rules means it's allowed for all platforms
        }
    }

    /// Check if a rule matches the current platform
    fn rule_matches(rule: &Rule) -> bool {
        if let Some(os_rule) = &rule.os {
            // Check OS name
            if let Some(name) = &os_rule.name {
                let current_os = match std::env::consts::OS {
                    "windows" => "windows",
                    "linux" => "linux",
                    "macos" => "osx",
                    _ => return false,
                };

                if name != current_os {
                    return false;
                }
            }

            // Check architecture
            if let Some(arch) = &os_rule.arch {
                let current_arch = match std::env::consts::ARCH {
                    "x86_64" => "x86_64",
                    "aarch64" => "arm64",
                    _ => return false,
                };

                if arch != current_arch {
                    return false;
                }
            }

            true
        } else {
            true // No OS rule means it matches all platforms
        }
    }
    /// Get the native classifier for the current platform (legacy format)
    pub fn get_native_classifier(&self) -> Option<String> {
        if let Some(natives) = &self.natives {
            let os_name = match std::env::consts::OS {
                "windows" => "windows",
                "linux" => "linux",
                "macos" => "osx",
                _ => return None,
            };

            natives.get(os_name).cloned()
        } else {
            None
        }
    }

    /// Check if this is a native library (modern format)
    pub fn is_native_library(&self) -> bool {
        self.name.contains(":natives-")
    }
}
