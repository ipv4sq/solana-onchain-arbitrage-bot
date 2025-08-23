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
