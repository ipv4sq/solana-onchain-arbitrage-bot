use anyhow::{anyhow, Result, Error};

pub trait OptionExt<T> {
    fn or_err<S: Into<String>>(self, msg: S) -> Result<T>;

    fn or_err_with<F, S>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>;
    
    fn or_else_err<E>(self, err: E) -> Result<T>
    where
        E: Into<Error>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_err<S: Into<String>>(self, msg: S) -> Result<T> {
        self.ok_or_else(|| anyhow!(msg.into()))
    }

    fn or_err_with<F, S>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.ok_or_else(|| anyhow!(f().into()))
    }
    
    fn or_else_err<E>(self, err: E) -> Result<T>
    where
        E: Into<Error>,
    {
        self.ok_or_else(|| err.into())
    }
}
