use std::future::Future;

/// Blocks on an async operation from within a sync context.
///
/// This function allows calling async code from synchronous contexts
/// when you're already inside a tokio runtime.
///
/// # Panics
/// Panics if not called from within a tokio runtime.
///
/// # Example
/// ```
/// let result = block_on(async {
///     some_async_function().await
/// });
/// ```
pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T>,
{
    tokio::runtime::Handle::current().block_on(future)
}

/// Macro for calling async code from sync contexts.
///
/// This macro provides a convenient syntax for blocking on async operations
/// when you're already inside a tokio runtime.
///
/// # Panics
/// Panics if not called from within a tokio runtime.
///
/// # Example
/// ```
/// let result = block_on_async!(some_async_function());
/// let result_with_await = block_on_async!({
///     let data = fetch_data().await;
///     process_data(data).await
/// });
/// ```
#[macro_export]
macro_rules! block_on_async {
    ($expr:expr) => {
        $crate::arb::util::tokio_util::block_on(async { $expr })
    };
    ({ $($body:tt)* }) => {
        $crate::arb::util::tokio_util::block_on(async { $($body)* })
    };
}

pub use block_on_async;
