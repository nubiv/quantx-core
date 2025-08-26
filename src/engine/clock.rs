use chrono::{DateTime, Utc};

pub trait EngineClock {
    fn time(&self) -> DateTime<Utc>;
}

#[derive(Debug)]
pub struct SimulatedClock {
    inner: std::sync::Arc<parking_lot::RwLock<SimulatedClockInner>>,
}

#[derive(Debug)]
struct SimulatedClockInner {
    time_exchange_last_event: DateTime<Utc>,
    time_live_last_event: DateTime<Utc>,
}

#[derive(Debug)]
pub struct LiveClock;
