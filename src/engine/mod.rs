use chrono::{DateTime, Utc};

mod clock;
mod exec;
mod risk;
mod state;
mod strategy;

#[derive(Debug)]
pub struct Engine<Clock, State, Execution, Risk, Strategy> {
    meta: EngineMeta,
    clock: Clock,
    state: State,
    exec: Execution,
    risk: Risk,
    strategy: Strategy,
}

#[derive(Debug)]
pub struct EngineMeta {
    version: String,
    seq: usize,
    time_init: DateTime<Utc>,
}
