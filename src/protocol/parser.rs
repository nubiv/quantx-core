pub trait ProtocolParser {
    fn parse() -> ();
}

pub trait HttpParser
where
    Self: ProtocolParser,
{
}

pub trait WebsocketParser
where
    Self: ProtocolParser,
{
}
