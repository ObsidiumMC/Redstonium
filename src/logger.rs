use env_logger::{Builder, Env};
use log::LevelFilter;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// Setup a custom logger with timestamps and colored output
pub fn setup_logger() {
    let env = Env::default().default_filter_or("info");

    Builder::from_env(env)
        .format(|buf, record| {
            let level_style = buf.default_level_style(record.level());

            if record.level() == log::Level::Debug {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                writeln!(
                    buf,
                    "{} [{}] {}",
                    now,
                    level_style.value(record.level()),
                    record.args()
                )
            } else {
                writeln!(
                    buf,
                    "[{}] {}",
                    level_style.value(record.level()),
                    record.args()
                )
            }
        })
        .filter(None, LevelFilter::Info)
        .init();
}
