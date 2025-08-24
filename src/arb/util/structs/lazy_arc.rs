#[macro_export]
macro_rules! lazy_arc {
    ($init:expr) => {
        once_cell::sync::Lazy::new(|| std::sync::Arc::new($init))
    };
}
