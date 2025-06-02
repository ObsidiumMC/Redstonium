use env_logger::{Builder, Env};
use log::LevelFilter;
use std::io::Write;
use time::{format_description::FormatItem, macros::format_description};

const LOG_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");

/// Setup a custom logger with timestamps and colored output
pub fn setup_logger() {
    let env = Env::default().default_filter_or("info");

    Builder::from_env(env)
        .format(|buf, record| {
            let now = time::OffsetDateTime::now_utc();
            let timestamp = now
                .format(LOG_FORMAT)
                .unwrap_or_else(|_| String::from("timestamp-error"));

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
}
