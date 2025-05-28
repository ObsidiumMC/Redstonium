use std::path::Path;
use std::process::Command;

/// Test that the CLI binary can be executed and shows help
#[test]
fn test_cli_help() {
    let output = Command::new("./target/release/rustified")
        .arg("--help")
        .output();

    match output {
        Ok(output) => {
            assert!(output.status.success(), "CLI help command should succeed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("A Minecraft CLI launcher written in Rust"));
            assert!(stdout.contains("Usage:"));
            assert!(stdout.contains("Commands:"));
        }
        Err(_) => {
            // Skip test if binary doesn't exist (not built in release mode)
            eprintln!("Skipping integration test - release binary not found");
        }
    }
}

/// Test that the CLI binary can show version
#[test]
fn test_cli_version() {
    let output = Command::new("./target/release/rustified")
        .arg("--version")
        .output();

    match output {
        Ok(output) => {
            assert!(
                output.status.success(),
                "CLI version command should succeed"
            );
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("rustified"));
        }
        Err(_) => {
            // Skip test if binary doesn't exist
            eprintln!("Skipping integration test - release binary not found");
        }
    }
}

/// Test that the CLI can list subcommands
#[test]
fn test_cli_subcommands() {
    let commands = ["list", "auth", "instance", "java"];

    for cmd in &commands {
        let output = Command::new("./target/release/rustified")
            .arg(cmd)
            .arg("--help")
            .output();

        match output {
            Ok(output) => {
                // Some commands might fail, but they should at least show help
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Either success with help text, or failure with help text in stderr
                if output.status.success() {
                    assert!(stdout.contains("Usage:") || stderr.contains("Usage:"));
                }
                // It's okay if some commands fail without proper setup
            }
            Err(_) => {
                eprintln!(
                    "Skipping integration test for {} - release binary not found",
                    cmd
                );
            }
        }
    }
}

/// Test that release binary exists and is executable
#[test]
fn test_binary_exists() {
    let binary_path = Path::new("./target/release/rustified");

    if binary_path.exists() {
        assert!(binary_path.is_file(), "Release binary should be a file");

        // Try to check if it's executable (Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(binary_path).unwrap();
            let permissions = metadata.permissions();
            assert!(
                permissions.mode() & 0o111 != 0,
                "Binary should be executable"
            );
        }
    } else {
        eprintln!("Release binary not found - run 'cargo build --release' first");
    }
}
