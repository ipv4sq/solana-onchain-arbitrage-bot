use crate::lazy_arc;
use crate::util::structs::rate_limiter::RateLimiter;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::Duration;

#[allow(non_upper_case_globals)]
pub(crate) static QueryRateLimiter: Lazy<Arc<RateLimiter>> = lazy_arc!({
    RateLimiter::new(
        50,
        Duration::from_secs(1),
        70,
        "AccountBalanceQueryRateLimiter".to_string(),
    )
});

#[allow(non_upper_case_globals)]
pub(crate) static SimulationRateLimiter: Lazy<Arc<RateLimiter>> = lazy_arc!({
    RateLimiter::new(
        20,
        Duration::from_secs(1),
        30,
        "AccountBalanceQueryRateLimiter".to_string(),
    )
});
