use env_logger::{Builder, Env};
use log::{Level, Record};
use colored::Colorize as _;
use std::io::Write;

pub fn init() {
    let mut builder = Builder::new();

    builder.parse_env(get_env());

    builder.format(|f, record| {
        let level = color_level(record.level());
        let target = format_target(record);
        let message = record.args();

        if let Some(target) = target {
            writeln!(f, "[{level} @ {target}] {message}")
        } else {
            writeln!(f, "[{level}] {message}")
        }
    });

    builder.init();
}

fn get_env() -> Env<'static> {
    let default_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    Env::default().default_filter_or(default_level)
}

fn format_target(record: &Record) -> Option<String> {
    let file = record.file()?;

    let file = if cfg!(target_os = "windows") {
        // Backslashes in paths look so ugly
        file.replace("\\", "/")
    } else {
        file.to_string()
    };

    let target = if let Some(line) = record.line() {
        format!("{file}:{line}")
    } else {
        file
    };

    Some(target.dimmed().to_string())
}

fn color_level(level: Level) -> String {
    match level {
        Level::Trace => "TRACE".magenta(),
        Level::Debug => "DEBUG".blue(),
        Level::Info => "INFO".green(),
        Level::Warn => "WARN".yellow(),
        Level::Error => "ERROR".red(),
    }
    .to_string()
}
