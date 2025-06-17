use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "Rustified")]
#[command(about = "A Minecraft CLI launcher written in Rust")]
#[command(version = "0.3.1")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List available Minecraft versions
    List {
        /// Filter by version types (can be used multiple times)
        #[arg(long, value_enum, action = clap::ArgAction::Append)]
        types: Vec<VersionTypeFilter>,
        /// Show only release versions (shorthand for --types release)
        #[arg(long, conflicts_with = "types")]
        releases_only: bool,
        /// Show only snapshot versions (shorthand for --types snapshot)
        #[arg(long, conflicts_with = "types")]
        snapshots_only: bool,
        /// Maximum number of versions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Filter versions by pattern (case-insensitive substring match)
        #[arg(long)]
        filter: Option<String>,
        /// Show installed status for each version
        #[arg(long)]
        show_installed: bool,
        /// Sort order for versions
        #[arg(long, value_enum, default_value = "newest-first")]
        sort: SortOrder,
    },
    /// Launch a Minecraft instance
    Launch {
        /// Instance to launch
        instance: String,
        /// Skip file verification (faster launch)
        #[arg(long)]
        skip_verification: bool,
    },
    /// Prepare (download) a Minecraft version without launching
    Prepare {
        /// Version to prepare
        version: String,
    },
    /// Authentication management
    Auth {
        #[command(subcommand)]
        action: AuthCommands,
    },
    /// Instance management
    Instance {
        #[command(subcommand)]
        action: InstanceCommands,
    },
    /// Java runtime management
    Java {
        #[command(subcommand)]
        action: JavaCommands,
    },
}

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Check authentication status
    Status,
    /// Clear cached authentication
    Clear,
    /// Force re-authentication
    Refresh,
}

#[derive(Subcommand)]
pub enum InstanceCommands {
    /// List all instances
    List,
    /// Create a new instance
    Create {
        /// Instance name
        name: String,
        /// Minecraft version
        version: String,
        /// Instance description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete an instance
    Delete {
        /// Instance name
        name: String,
    },
    /// Show instance details
    Info {
        /// Instance name
        name: String,
    },
    /// Set instance memory allocation
    Memory {
        /// Instance name
        name: String,
        /// Memory in MB
        memory: u32,
    },
}

#[derive(Subcommand)]
pub enum JavaCommands {
    /// List available Java installations
    List,
    /// Show recommended Java version for a Minecraft version
    Recommend {
        /// Minecraft version
        version: String,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum VersionTypeFilter {
    /// Release versions (stable)
    Release,
    /// Snapshot versions (development)
    Snapshot,
    /// Old beta versions
    OldBeta,
    /// Old alpha versions
    OldAlpha,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SortOrder {
    /// Newest versions first (default)
    NewestFirst,
    /// Oldest versions first
    OldestFirst,
    /// Alphabetical order
    Alphabetical,
}
