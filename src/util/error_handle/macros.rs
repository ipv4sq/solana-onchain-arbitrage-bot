#[macro_export]
macro_rules! lined_err {
    ($msg:expr) => {{
        let location = format!("{}:{}:{}", file!(), line!(), column!());
        anyhow::anyhow!("{} (at {})", $msg, location)
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        let location = format!("{}:{}:{}", file!(), line!(), column!());
        anyhow::anyhow!("{} (at {})", format!($fmt, $($arg)*), location)
    }};
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    #[test]
    fn test_lined_err_simple_message() {
        let err = lined_err!("test error");
        let err_str = err.to_string();

        assert!(err_str.contains("test error"));
        assert!(err_str.contains("(at "));
        assert!(err_str.contains("src/util/error_handle/macros.rs:"));
    }

    #[test]
    fn test_lined_err_format_message() {
        let value = 42;
        let err = lined_err!("error with value: {}", value);
        let err_str = err.to_string();

        assert!(err_str.contains("error with value: 42"));
        assert!(err_str.contains("(at "));
        assert!(err_str.contains("src/util/error_handle/macros.rs:"));
    }
}
