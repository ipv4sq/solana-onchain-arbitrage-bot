use std::time::Duration;

pub struct Interval;

impl Interval {
    pub const MILLISECOND: Duration = Duration::from_millis(1);
    pub const SECOND: Duration = Duration::from_secs(1);
    pub const MINUTE: Duration = Duration::from_secs(60);
    pub const HOUR: Duration = Duration::from_secs(3600);
    pub const DAY: Duration = Duration::from_secs(24 * 3600);
    pub const WEEK: Duration = Duration::from_secs(7 * 24 * 3600);
}
