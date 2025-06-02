use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustified")]
#[command(about = "A Minecraft CLI launcher written in Rust")]
#[command(version = "0.2.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List available Minecraft versions
    List {
        /// Show only release versions
        #[arg(long)]
        releases_only: bool,
        /// Maximum number of versions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
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
