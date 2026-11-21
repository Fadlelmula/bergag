use crate::error::{AppError, AppResult};
use crate::security::redaction::redact;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::FmtSubscriber;

pub struct LoggingGuard {
    // Kept alive for the lifetime of the app so log buffers are flushed.
    #[allow(dead_code)]
    pub worker: WorkerGuard,
}

pub fn initialize(log_dir: &Path) -> AppResult<LoggingGuard> {
    fs::create_dir_all(log_dir)
        .map_err(|error| AppError::Internal(format!("Could not create log directory: {error}")))?;

    let appender = rolling::daily(log_dir, "bergag.log");
    let (writer, worker) = tracing_appender::non_blocking(appender);
    let subscriber = FmtSubscriber::builder()
        .with_writer(writer)
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .with_target(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|error| AppError::Internal(format!("Could not initialize logging: {error}")))?;

    install_panic_hook();
    tracing::info!(log_dir = %log_dir.display(), "Bergag logging initialized");
    Ok(LoggingGuard { worker })
}

fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("unknown panic");
        tracing::error!(message = %message, "Bergag panic");
        previous(info);
    }));
}

pub fn latest_log_file(log_dir: &Path) -> AppResult<Option<PathBuf>> {
    let entries = match fs::read_dir(log_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(None),
    };

    let mut candidates: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let is_log = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("bergag.log"));
        if !is_log || !path.is_file() {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        candidates.push((modified, path));
    }

    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    Ok(candidates.into_iter().next().map(|(_, path)| path))
}

pub fn recent_logs(log_dir: &Path, limit: usize) -> AppResult<Vec<String>> {
    let Some(path) = latest_log_file(log_dir)? else {
        return Ok(Vec::new());
    };
    let file = File::open(&path)
        .map_err(|error| AppError::Internal(format!("Could not read log file: {error}")))?;
    let mut reader = BufReader::new(file);
    let mut lines: Vec<String> = Vec::new();
    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        let read = reader
            .read_until(b'\n', &mut buffer)
            .map_err(|error| AppError::Internal(format!("Could not read log file: {error}")))?;
        if read == 0 {
            break;
        }
        let line = String::from_utf8_lossy(&buffer).trim_end().to_string();
        lines.push(truncate(&redact(&line), 500));
        if lines.len() > limit * 4 {
            lines.remove(0);
        }
    }

    let start = lines.len().saturating_sub(limit);
    Ok(lines[start..].to_vec())
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut result: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        result.push('…');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_keeps_unicode_bounds() {
        assert_eq!(truncate("ééé", 2), "éé…");
        assert_eq!(truncate("ok", 10), "ok");
    }
}
