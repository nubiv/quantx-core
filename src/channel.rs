use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct SyncChannel;
#[derive(Debug, Clone)]
pub struct AsyncChannel;

pub trait ChannelKind {
    type Sender<T>;
    type Receiver<T>;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>);
}

impl ChannelKind for SyncChannel {
    type Sender<T> = kanal::Sender<T>;
    type Receiver<T> = kanal::Receiver<T>;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        kanal::unbounded::<T>()
    }
}

impl ChannelKind for AsyncChannel {
    type Sender<T> = kanal::AsyncSender<T>;
    type Receiver<T> = kanal::AsyncReceiver<T>;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        kanal::unbounded_async::<T>()
    }
}

pub trait SyncTx<T> {
    type SendError: Debug;

    fn send(&self, item: T) -> Result<(), Self::SendError>;
}

pub trait AsyncTx<T> {
    type SendError: Debug;
    type SendFuture<'a>: Future<Output = Result<(), Self::SendError>> + 'a
    where
        Self: 'a,
        T: 'a;

    fn send(&self, item: T) -> Self::SendFuture<'_>;
}

#[derive(Debug, Clone)]
pub struct UnboundedTx<K, T>
where
    K: ChannelKind,
    <K as ChannelKind>::Sender<T>: Debug + Clone,
{
    pub tx: <K as ChannelKind>::Sender<T>,
}

impl<T> SyncTx<T> for UnboundedTx<SyncChannel, T>
where
    <SyncChannel as ChannelKind>::Sender<T>: Debug + Clone,
{
    type SendError = kanal::SendError;

    fn send(&self, item: T) -> Result<(), Self::SendError> {
        self.tx.send(item)
    }
}

impl<T> AsyncTx<T> for UnboundedTx<AsyncChannel, T>
where
    <AsyncChannel as ChannelKind>::Sender<T>: Debug + Clone,
{
    type SendError = kanal::SendError;
    type SendFuture<'a>
        = kanal::SendFuture<'a, T>
    where
        T: 'a;

    fn send(&self, item: T) -> Self::SendFuture<'_> {
        self.tx.send(item)
    }
}

pub trait SyncRx<T> {
    type ReceiveError: Debug;

    fn recv(&mut self) -> Result<T, Self::ReceiveError>;
}

pub trait AsyncRx<T> {
    type ReceiveError: Debug;
    type ReceiveFuture<'a>: Future<Output = Result<T, Self::ReceiveError>> + 'a
    where
        Self: 'a,
        T: 'a;

    fn recv(&mut self) -> Self::ReceiveFuture<'_>;
}

#[derive(Debug)]
pub struct UnboundedRx<K, T>
where
    K: ChannelKind,
    <K as ChannelKind>::Receiver<T>: Debug,
{
    pub rx: <K as ChannelKind>::Receiver<T>,
}

impl<T> SyncRx<T> for UnboundedRx<SyncChannel, T>
where
    <SyncChannel as ChannelKind>::Receiver<T>: Debug,
{
    type ReceiveError = kanal::ReceiveError;

    fn recv(&mut self) -> Result<T, Self::ReceiveError> {
        self.rx.recv()
    }
}

impl<T> AsyncRx<T> for UnboundedRx<AsyncChannel, T>
where
    <AsyncChannel as ChannelKind>::Receiver<T>: Debug,
{
    type ReceiveError = kanal::ReceiveError;
    type ReceiveFuture<'a>
        = kanal::ReceiveFuture<'a, T>
    where
        T: 'a;

    fn recv(&mut self) -> Self::ReceiveFuture<'_> {
        self.rx.recv()
    }
}

pub fn mpsc_unbounded<K, T>() -> (UnboundedTx<K, T>, UnboundedRx<K, T>)
where
    K: ChannelKind,
    <K as ChannelKind>::Sender<T>: Debug + Clone,
    <K as ChannelKind>::Receiver<T>: Debug,
{
    let (tx, rx) = K::unbounded::<T>();

    (UnboundedTx { tx }, UnboundedRx { rx })
}
