#[macro_export]
macro_rules! f {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}

#[macro_export]
macro_rules! return_ok_if_some {
    ($opt:expr) => {
        if let Some(val) = $opt {
            return Ok(val);
        }
    };
}

#[macro_export]
macro_rules! ok_map_else {
    ($result:expr, |$var:ident| $extract:expr, $default:expr) => {
        match $result {
            Ok($var) => $extract,
            Err(_) => $default,
        }
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
macro_rules! log_error_with_backtrace {
    ($level:ident, $msg:expr, $err:expr) => {
        tracing::$level!("{}: {}", $msg, $err);
        if $err.backtrace().to_string() != "disabled backtrace" {
            tracing::$level!("Backtrace:\n{}", $err.backtrace());
        }
    };
}
