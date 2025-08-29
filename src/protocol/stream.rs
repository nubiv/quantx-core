use std::{
    collections::VecDeque,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use pin_project::pin_project;
use tracing::{error, info, warn};

use crate::protocol::{
    parser::{ProtocolParser, WebsocketParser},
    transformer::{Transformer, TransformerBatch, TransformerSingle},
};

pub trait RecoverableStream
where
    Self: Sized,
{
    fn with_reconnect_backoff<InnerSt, InitErr>(self, policy: ReconnectionBackoffPolicy, stream_key: u64) -> impl Stream<Item = InnerSt>
    where
        Self: Stream<Item = Result<InnerSt, InitErr>>,
        InnerSt: Stream,
        InitErr: std::fmt::Debug,
    {
        self.enumerate()
            .scan(ReconnectionState::from(policy), move |state, (attempt, result)| match result {
                Ok(stream) => {
                    info!(attempt, ?stream_key, "Successfully initialized Stream.");
                    state.reset_backoff();

                    futures::future::Either::Left(std::future::ready(Some(Ok(stream))))
                },
                Err(error) => {
                    warn!(attempt, ?stream_key, ?error, "Failed to re-initialize Stream.");
                    let sleep_fut = state.generate_sleep_future();
                    state.multiply_backoff();

                    // TODO:
                    // Would it still work without pin?
                    futures::future::Either::Right(Box::pin(async move {
                        sleep_fut.await;
                        Some(Err(error))
                    }))
                },
            })
            .filter_map(|result| std::future::ready(result.ok()))
    }

    fn with_termination_on_error<InnerSt, T, E, FnIsTerminal>(
        self,
        is_terminal: FnIsTerminal,
        stream_key: u64,
    ) -> impl Stream<Item = impl Stream<Item = Result<T, E>>>
    where
        Self: Stream<Item = InnerSt>,
        InnerSt: Stream<Item = Result<T, E>>,
        FnIsTerminal: Copy + Fn(&E) -> bool,
    {
        self.map(move |stream| {
            tokio_stream::StreamExt::map_while(stream, {
                move |result| match result {
                    Ok(item) => Some(Ok(item)),
                    Err(error) if is_terminal(&error) => {
                        error!(?stream_key, "MarketStream encountered terminal error that requires reconnecting.");

                        None
                    },
                    Err(error) => Some(Err(error)),
                }
            })
        })
    }

    fn with_reconnection_events<InnerSt, Origin>(self, origin: Origin) -> impl Stream<Item = StreamEvent<Origin, InnerSt::Item>>
    where
        Self: Stream<Item = InnerSt>,
        InnerSt: Stream,
        Origin: Clone + 'static,
    {
        self.map(move |stream| {
            stream
                .map(StreamEvent::Item)
                .chain(futures::stream::once(std::future::ready(StreamEvent::Reconnecting(origin.clone()))))
        })
        .flatten()
    }

    fn with_error_handler<FnOnErr, Origin, T, E>(self, op: FnOnErr) -> impl Stream<Item = StreamEvent<Origin, T>>
    where
        Self: Stream<Item = StreamEvent<Origin, Result<T, E>>>,
        FnOnErr: Fn(E) + 'static,
    {
        self.filter_map(move |event| {
            std::future::ready(match event {
                StreamEvent::Reconnecting(origin) => Some(StreamEvent::Reconnecting(origin)),
                StreamEvent::Item(Ok(item)) => Some(StreamEvent::Item(item)),
                StreamEvent::Item(Err(error)) => {
                    op(error);

                    None
                },
            })
        })
    }

    // TOOD:
    // Optional cron based reconnection, used for example to reconnect upon daily exchange maintenance windows.

    // fn forward_to<Transmitter>(self, tx: Transmitter) -> impl Future<Output = ()> + Send
    // where
    //     Self: Stream + Sized + Send,
    //     Self::Item: Into<Transmitter::Item>,
    //     Transmitter: Tx + Send + 'static,
    // {
    //     tokio_stream::StreamExt::map_while(self, move |event| tx.send(event.into()).ok()).collect()
    // }
}

pub async fn init_recoverable_stream<FnInit, InnerSt, InitErr, InitFut>(
    init_inner_stream: FnInit,
) -> Result<impl Stream<Item = Result<InnerSt, InitErr>>, InitErr>
where
    FnInit: Fn() -> InitFut,
    InitFut: Future<Output = Result<InnerSt, InitErr>>,
    InnerSt: Stream,
{
    let inner_stream = init_inner_stream().await?;
    let recoverables = futures::stream::repeat_with(init_inner_stream).then(std::convert::identity);

    Ok(futures::stream::once(std::future::ready(Ok(inner_stream))).chain(recoverables))
}

#[derive(Debug)]
pub enum StreamEvent<Origin, T> {
    Reconnecting(Origin),
    Item(T),
}

#[derive(Debug)]
pub struct ReconnectionBackoffPolicy {
    pub backoff_ms_initial: u64,
    pub backoff_multiplier: u8,
    pub backoff_ms_max: u64,
}

#[derive(Debug)]
struct ReconnectionState {
    policy: ReconnectionBackoffPolicy,
    backoff_ms_current: u64,
}

impl From<ReconnectionBackoffPolicy> for ReconnectionState {
    fn from(policy: ReconnectionBackoffPolicy) -> Self {
        Self {
            backoff_ms_current: policy.backoff_ms_initial,
            policy,
        }
    }
}

impl ReconnectionState {
    fn reset_backoff(&mut self) {
        self.backoff_ms_current = self.policy.backoff_ms_initial;
    }

    fn multiply_backoff(&mut self) {
        let next = self.backoff_ms_current * self.policy.backoff_multiplier as u64;
        let next_capped = std::cmp::min(next, self.policy.backoff_ms_max);

        self.backoff_ms_current = next_capped;
    }

    fn generate_sleep_future(&self) -> tokio::time::Sleep {
        let sleep_duration = std::time::Duration::from_millis(self.backoff_ms_current);

        tokio::time::sleep(sleep_duration)
    }
}

#[derive(Debug)]
#[pin_project]
pub struct ExchangeStream<Protocol, StreamRaw, StTransformer>
where
    Protocol: ProtocolParser,
    StreamRaw: Stream,
    StTransformer: Transformer,
{
    #[pin]
    pub stream: StreamRaw,
    pub transformer: StTransformer,
    pub buffer: VecDeque<Result<StTransformer::OutputItem, StTransformer::Error>>,
    pub protocol_marker: PhantomData<Protocol>,
}

// impl<Protocol, StreamRaw, StreamTransformer> Stream for ExchangeStream<Protocol, StreamRaw, StreamTransformer>
// where
//     Protocol: WebsocketParser,
//     StreamRaw: Stream<Item = Result<Protocol::Message, Protocol::Error>> + Unpin,
//     StreamTransformer: TransformerSingle,
//     StreamTransformer::Error: From<Box<dyn std::error::Error>>,
// {
//     type Item = Result<StreamTransformer::OutputItem, StreamTransformer::Error>;

//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         loop {
//             // Flush Self::Item buffer if it is not currently empty
//             if let Some(output) = self.buffer.pop_front() {
//                 return Poll::Ready(Some(output));
//             }

//             // Poll inner `Stream` for next the next input protocol message
//             let input = match self.as_mut().project().stream.poll_next(cx) {
//                 Poll::Ready(Some(input)) => input,
//                 Poll::Ready(None) => return Poll::Ready(None),
//                 Poll::Pending => return Poll::Pending,
//             };

//             // Parse input protocol message into `ExchangeMessage`
//             let exchange_message = match Protocol::parse::<StreamTransformer::Input>(input) {
//                 // `StreamParser` successfully deserialised `ExchangeMessage`
//                 Some(Ok(exchange_message)) => exchange_message,

//                 // If `StreamParser` returns an Err pass it downstream
//                 Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),

//                 // If `StreamParser` returns None it's a safe-to-skip message
//                 None => continue,
//             };

//             // Transform `ExchangeMessage` into `Transformer::OutputIter`
//             // ie/ IntoIterator<Item = Result<Output, SocketError>>
//             self.transformer
//                 .transform_one(exchange_message)
//                 .into_iter()
//                 .for_each(|output_result: StreamTransformer::OutputItem| self.buffer.push_back(output_result));
//         }
//     }
// }

impl<Protocol, StreamRaw, StreamTransformer> Stream for ExchangeStream<Protocol, StreamRaw, StreamTransformer>
where
    Protocol: WebsocketParser,
    StreamRaw: Stream<Item = Result<Protocol::Message, Protocol::Error>> + Unpin,
    StreamTransformer: TransformerBatch,
    StreamTransformer::Error: From<Box<dyn std::error::Error>>,
{
    type Item = Result<StreamTransformer::OutputItem, StreamTransformer::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Flush Self::Item buffer if it is not currently empty
            if let Some(output) = self.buffer.pop_front() {
                return Poll::Ready(Some(output));
            }

            // Poll inner `Stream` for next the next input protocol message
            let input = match self.as_mut().project().stream.poll_next(cx) {
                Poll::Ready(Some(input)) => input,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };

            // Parse input protocol message into `ExchangeMessage`
            let exchange_message = match Protocol::parse::<StreamTransformer::Input>(input) {
                // `StreamParser` successfully deserialised `ExchangeMessage`
                Some(Ok(exchange_message)) => exchange_message,

                // If `StreamParser` returns an Err pass it downstream
                Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),

                // If `StreamParser` returns None it's a safe-to-skip message
                None => continue,
            };

            // Transform `ExchangeMessage` into `Transformer::OutputIter`
            // ie/ IntoIterator<Item = Result<Output, SocketError>>
            self.transformer
                .transform_many(exchange_message)
                .into_iter()
                .for_each(|output_result: Result<StreamTransformer::OutputItem, StreamTransformer::Error>| self.buffer.push_back(output_result));

            // if let Some(output) = self.buffer.pop_front() {
            //     return Poll::Ready(Some(output));
            // }
        }
    }
}
