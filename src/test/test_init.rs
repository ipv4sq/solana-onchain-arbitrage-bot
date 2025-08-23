use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment once for all tests
pub fn init_test_env() {
    INIT.call_once(|| {
        // Initialize tracing
        let _ = tracing_subscriber::fmt()
            .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
            .with_test_writer() // Use test writer to work better with cargo test
            .try_init(); // Use try_init in case it's already initialized

        // Load .env file for database connection
        dotenv::dotenv().ok();
    });
}

/// Initialize test environment with custom log level
pub fn init_test_env_with_level(level: &str) {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(level)
            .with_test_writer()
            .try_init();

        dotenv::dotenv().ok();
    });
}

// Automatically initialize for all tests when running cargo test
#[cfg(test)]
#[ctor::ctor]
fn init() {
    init_test_env();
}
