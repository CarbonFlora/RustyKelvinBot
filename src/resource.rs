use chrono::{DateTime, Utc};

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Resources {
    pub active_timers: Vec<DateTime<Utc>>,
}
