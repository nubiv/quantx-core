pub trait EngineState {}

pub type BarterEngineState<GlobalData, InstrumentData> = barter::engine::state::EngineState<GlobalData, InstrumentData>;
