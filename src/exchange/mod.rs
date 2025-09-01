use crate::protocol::websocket::BarterWsMessage;

mod simulation;

pub trait Exchange {}

pub trait ExchangeExt {}

pub trait BarterConnector: barter_data::exchange::Connector {}
pub trait BarterExchangeServer: barter_data::exchange::ExchangeServer {}

#[derive(Debug)]
pub struct PingInterval {
    pub interval: tokio::time::Interval,
    pub ping: fn() -> BarterWsMessage,
}
