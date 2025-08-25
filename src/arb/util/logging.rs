use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;
use tracing::info;
use tracing_subscriber::{fmt::time::FormatTime, layer::SubscriberExt, util::SubscriberInitExt};

struct CustomTimer;

impl FormatTime for CustomTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Utc::now();
        write!(w, "{}", now.format("%Y-%m-%dT%H:%M:%S%.3f"))
    }
}

pub fn init() -> Result<String> {
    std::env::set_var("RUST_BACKTRACE", "1");

    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        fs::create_dir(logs_dir)?;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let log_file_path = format!("logs/bot_{}.log", timestamp);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_file_path)?;

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::sync::Arc::new(file))
        .with_ansi(false)
        .with_line_number(false)
        .with_file(false)
        .with_target(false)
        .with_timer(CustomTimer);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(true)
        .with_line_number(true)
        .with_timer(CustomTimer)
        .compact();

    let filter = tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".to_string()),
    );

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(filter)
        .init();

    info!("Starting Solana MEV Bot Listener");
    info!("Logs are being written to: {}", log_file_path);

    Ok(log_file_path)
}
