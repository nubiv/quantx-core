use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream,
    connect_async,
    tungstenite::{
        Utf8Bytes,
        client::IntoClientRequest,
        error::ProtocolError,
        protocol::{CloseFrame, frame::Frame},
    },
};

pub type BarterWebSocket = barter_integration::protocol::websocket::WebSocket;
pub type BarterWsSink = barter_integration::protocol::websocket::WsSink;
pub type BarterWsStream = barter_integration::protocol::websocket::WsStream;
pub type BarterWsMessage = barter_integration::protocol::websocket::WsMessage;
pub type BarterWsError = barter_integration::protocol::websocket::WsError;
