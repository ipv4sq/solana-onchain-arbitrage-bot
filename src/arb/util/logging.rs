use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;
use tracing::{info, Event, Subscriber};
use tracing_subscriber::{
    fmt::{
        format::{self, FormatEvent, FormatFields},
        time::FormatTime,
        FmtContext,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

struct CustomTimer;

impl FormatTime for CustomTimer {
    fn format_time(&self, w: &mut format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Utc::now();
        write!(w, "{}", now.format("%Y-%m-%dT%H:%M:%S%.3f"))
    }
}

struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();

        CustomTimer.format_time(&mut writer)?;

        let level = meta.level();
        write!(writer, "  {}", level)?;

        if let Some(target) = meta.target().strip_prefix("solana_onchain_arbitrage_bot::") {
            let formatted_target = target.replace("::", ":");
            write!(writer, " {}", formatted_target)?;
        } else {
            let formatted_target = meta.target().replace("::", ":");
            write!(writer, " {}", formatted_target)?;
        }

        if let Some(line) = meta.line() {
            write!(writer, ":{}", line)?;
        }

        write!(writer, ": ")?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

pub fn init() -> Result<String> {
    std::env::set_var("RUST_BACKTRACE", "full");

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
        .event_format(CustomFormatter);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .event_format(CustomFormatter);

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
