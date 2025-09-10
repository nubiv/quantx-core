use crate::channel::base::{AsyncChannel, AsyncChannelKind, RecvAsyncLike, RecvSyncLike, SendAsyncLike, SendSyncLike, SyncChannel, SyncChannelKind};

pub type KanalSyncChannel = SyncChannel<KanalSync>;
pub type KanalAsyncChannel = AsyncChannel<KanalAsync>;

#[derive(Debug, Clone)]
pub struct KanalSync;
impl SyncChannelKind for KanalSync {
    type Sender<T> = kanal::Sender<T>;
    type Receiver<T> = kanal::Receiver<T>;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        kanal::unbounded::<T>()
    }
}

#[derive(Debug, Clone)]
pub struct KanalAsync;
impl AsyncChannelKind for KanalAsync {
    type Sender<T> = kanal::AsyncSender<T>;
    type Receiver<T> = kanal::AsyncReceiver<T>;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        kanal::unbounded_async::<T>()
    }
}

impl<T> SendSyncLike<T> for kanal::Sender<T> {
    type SendError = kanal::SendError;

    fn send_sync(&self, item: T) -> Result<(), Self::SendError> {
        self.send(item)
    }
}

impl<T> RecvSyncLike<T> for kanal::Receiver<T> {
    type ReceiveError = kanal::ReceiveError;

    fn recv_sync(&mut self) -> Result<T, Self::ReceiveError> {
        self.recv()
    }
}

impl<T> SendAsyncLike<T> for kanal::AsyncSender<T> {
    type SendError = kanal::SendError;
    type SendFuture<'a>
        = kanal::SendFuture<'a, T>
    where
        T: 'a;

    fn send_async(&self, item: T) -> Self::SendFuture<'_> {
        self.send(item)
    }
}

impl<T> RecvAsyncLike<T> for kanal::AsyncReceiver<T> {
    type ReceiveError = kanal::ReceiveError;
    type RecvFuture<'a>
        = kanal::ReceiveFuture<'a, T>
    where
        T: 'a;

    fn recv_async(&mut self) -> Self::RecvFuture<'_> {
        self.recv()
    }
}
