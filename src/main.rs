mod auth;

use env_logger::{Builder, Env};
use log::{info, error, LevelFilter};
use std::io::Write;
use time::{format_description::FormatItem, macros::format_description};
use dotenv::dotenv; // Import dotenv

const LOG_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv().ok(); 

    // Initialize the logger with custom format
    setup_logger()?;

    info!("Rustified Minecraft Launcher");
    info!("---------------------------");
    
    // Initialize the authentication process
    info!("Starting authentication process...");
    match auth::authenticate().await {
        Ok(result) => {
            info!("Authentication successful!");
            info!("Welcome, {}!", result.profile.name);
            result
        },
        Err(e) => {
            error!("Authentication failed: {}", e);
            // Log backtrace if available
            let backtrace = e.backtrace();
            if backtrace.status() != std::backtrace::BacktraceStatus::Disabled {
                error!("Error backtrace: {}", backtrace);
            }
            return Err(e);
        }
    };
    
    // Here you would continue with the launcher functionality
    
    Ok(())
}

/// Setup a custom logger with timestamps and colored output
fn setup_logger() -> anyhow::Result<()> {
    let env = Env::default().default_filter_or("info");
    
    Builder::from_env(env)
        .format(|buf, record| {
            let now = time::OffsetDateTime::now_utc();
            let timestamp = now.format(LOG_FORMAT).unwrap_or_else(|_| String::from("timestamp-error"));
            
            let level_style = buf.default_level_style(record.level());
            
            writeln!(
                buf,
                "{} [{}] - {}",
                timestamp,
                level_style.value(record.level()),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
    
    Ok(())
}
