use crate::cli::JavaCommands;
use log::info;

pub fn handle_java_command(launcher: &crate::launcher::Launcher, action: JavaCommands) {
    match action {
        JavaCommands::List => {
            let installations = &launcher.java_manager.installations;
            if installations.is_empty() {
                info!("No Java installations found. Try installing Java or setting JAVA_HOME.");
            } else {
                info!("Found {} Java installation(s):", installations.len());
                for (major, installation) in installations {
                    info!("  Java {}: {}", major, installation.path.display());
                }
            }
        }
        JavaCommands::Recommend { version } => {
            info!("Getting recommended Java version for Minecraft {version}...");
            let recommended = crate::launcher::JavaManager::get_required_java_version(&version);
            info!("Recommended Java version: {recommended}");
        }
    }
}
