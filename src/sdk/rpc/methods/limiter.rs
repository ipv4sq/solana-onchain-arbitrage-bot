use crate::lazy_arc;
use crate::util::structs::rate_limiter::RateLimiter;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::Duration;

#[allow(non_upper_case_globals)]
pub static QueryRateLimiter: Lazy<Arc<RateLimiter>> = lazy_arc!({
    RateLimiter::new(
        60,
        Duration::from_secs(1),
        70,
        "AccountBalanceQueryRateLimiter".to_string(),
    )
});
