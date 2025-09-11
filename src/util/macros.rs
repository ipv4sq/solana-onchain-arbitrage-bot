#[macro_export]
macro_rules! f {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}

#[macro_export]
macro_rules! unit_ok {
    () => {
        Ok(())
    };
}

#[macro_export]
macro_rules! spawn_with_error_handling {
    ($name:expr, $future:expr) => {
        tokio::spawn(async move {
            if let Err(e) = $future.await {
                tracing::error!("{} error: {}", $name, e);
                if e.backtrace().to_string() != "disabled backtrace" {
                    tracing::error!("Backtrace:\n{}", e.backtrace());
                }
            }
        })
    };
}

#[macro_export]
macro_rules! return_error {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        return Err(anyhow::anyhow!(msg));
    }};
}
