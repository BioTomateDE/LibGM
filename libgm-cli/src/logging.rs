use env_logger::{Builder, Env};
use log::{Level, Record};
use colored::{Color, Colorize as _};
use std::io::Write;

pub fn init() {
    let mut builder = Builder::new();

    builder.parse_env(get_env());

    builder.format(|f, record| {
        let color = color_by_level(record.level());
        let level = level_to_str(record.level()).color(color);
        let target = format_target(record);
        let message = record.args().to_string().color(color);

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

fn color_by_level(level: Level) -> Color {
    match level {
        Level::Trace => Color::Magenta,
        Level::Debug => Color::Blue,
        Level::Info => Color::Green,
        Level::Warn => Color::Yellow,
        Level::Error => Color::Red,
    }
}

fn level_to_str(level: Level) -> &'static str {
    match level {
        Level::Trace => "TRACE",
        Level::Debug => "DEBUG",
        Level::Info => "INFO",
        Level::Warn => "WARN",
        Level::Error => "ERROR",
    }
}
