use futures::Stream;
use serde::de::DeserializeOwned;

pub(crate) trait ProtocolParser {}

pub trait WebsocketParser
where
    Self: ProtocolParser,
{
    type Stream: Stream;
    type Message;
    type Error;

    fn parse<Output>(input: Result<Self::Message, Self::Error>) -> Option<Result<Output, Box<dyn std::error::Error>>>
    where
        Output: DeserializeOwned;
}

pub trait HttpParser
where
    Self: ProtocolParser,
{
}
