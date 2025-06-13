use crate::cli::InstanceCommands;
use tracing::{error, info};

/// Handles all instance-related commands.
///
/// # Errors
/// Returns an error if instance operations (list, create, delete, info, memory) fail.
pub async fn handle_instance_command(
    launcher: &crate::launcher::Launcher,
    action: InstanceCommands,
) -> crate::error::Result<()> {
    match action {
        InstanceCommands::List => {
            let instance_manager = launcher.instance_manager.lock().await;
            let instances: Vec<_> = instance_manager
                .list_instances()
                .into_iter()
                .cloned()
                .collect();
            drop(instance_manager); // Release lock early

            if instances.is_empty() {
                info!(
                    "No instances found. Create one with: rustified instance create <name> <version>"
                );
            } else {
                info!("Available instances:");
                for instance in instances {
                    let last_used = if let Some(used) = instance.last_used {
                        format!(" (last used: {})", used.format("%Y-%m-%d %H:%M:%S"))
                    } else {
                        String::new()
                    };

                    let description = instance
                        .description
                        .as_ref()
                        .map(|d| format!(" - {d}"))
                        .unwrap_or_default();

                    info!(
                        "  {} (v{}){}{}",
                        instance.name, instance.version, description, last_used
                    );
                }
            }
        }
        InstanceCommands::Info { name } => {
            let instance_manager = launcher.instance_manager.lock().await;
            if let Some(instance) = instance_manager.get_instance(&name) {
                let instance = instance.clone(); // Clone to avoid borrow issues
                drop(instance_manager); // Release lock

                info!("Instance: {}", instance.name);
                info!("  Version: {}", instance.version);
                if let Some(desc) = &instance.description {
                    info!("  Description: {desc}");
                }
                info!(
                    "  Created: {}",
                    instance.created.format("%Y-%m-%d %H:%M:%S")
                );
                if let Some(used) = instance.last_used {
                    info!("  Last used: {}", used.format("%Y-%m-%d %H:%M:%S"));
                }
                info!("  Mod loader: {:?}", instance.mods.loader);
                if let Some(memory) = instance.settings.memory_mb {
                    info!("  Memory: {memory}MB");
                }
                if !instance.settings.java_args.is_empty() {
                    info!("  Java args: {}", instance.settings.java_args.join(" "));
                }
            } else {
                error!("Instance '{name}' does not exist");
                return Err(crate::error::InstanceError::not_found(
                    "Instance not found".to_string(),
                )
                .into());
            }
        }
        InstanceCommands::Create {
            name,
            version,
            description,
        } => {
            let mut instance_manager = launcher.instance_manager.lock().await;
            instance_manager
                .create_instance(name.clone(), version, description, &launcher.file_manager)
                .await?;
            info!("✓ Created instance '{name}'");
        }
        InstanceCommands::Delete { name } => {
            let mut instance_manager = launcher.instance_manager.lock().await;
            instance_manager.delete_instance(&name).await?;
            info!("✓ Deleted instance '{name}'");
        }
        InstanceCommands::Memory { name, memory } => {
            let mut instance_manager = launcher.instance_manager.lock().await;
            instance_manager.set_instance_memory(&name, memory).await?;
            info!("✓ Set memory for instance '{name}' to {memory}MB");
        }
    }
    Ok(())
}
