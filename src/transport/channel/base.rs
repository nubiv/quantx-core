use std::{fmt::Debug, marker::PhantomData};

#[derive(Debug, Clone)]
pub struct SyncChannel<K>(PhantomData<K>)
where
    K: Debug + Clone;
#[derive(Debug, Clone)]
pub struct AsyncChannel<K>(PhantomData<K>)
where
    K: Debug + Clone;

pub trait SyncChannelKind
where
    Self: Debug + Clone,
{
    type Sender<T>: Debug + Clone;
    type Receiver<T>: Debug;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>);
}

pub trait AsyncChannelKind
where
    Self: Debug + Clone,
{
    type Sender<T>: Debug + Clone;
    type Receiver<T>: Debug;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>);
}

pub trait ChannelBaseKind {
    type Sender<T>;
    type Receiver<T>;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>);
}

impl<K> ChannelBaseKind for SyncChannel<K>
where
    K: SyncChannelKind,
{
    type Sender<T> = K::Sender<T>;
    type Receiver<T> = K::Receiver<T>;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        K::unbounded::<T>()
    }
}

impl<K> ChannelBaseKind for AsyncChannel<K>
where
    K: AsyncChannelKind,
{
    type Sender<T> = K::Sender<T>;
    type Receiver<T> = K::Receiver<T>;

    fn unbounded<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        K::unbounded::<T>()
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
    K: ChannelBaseKind,
    K::Sender<T>: Debug + Clone,
{
    pub tx: K::Sender<T>,
}

impl<K, T> SyncTx<T> for UnboundedTx<SyncChannel<K>, T>
where
    K: SyncChannelKind,
    K::Sender<T>: Debug + Clone + SendSyncLike<T>,
{
    type SendError = <K::Sender<T> as SendSyncLike<T>>::SendError;

    fn send(&self, item: T) -> Result<(), Self::SendError> {
        SendSyncLike::send_sync(&self.tx, item)
    }
}

impl<K, T> AsyncTx<T> for UnboundedTx<AsyncChannel<K>, T>
where
    K: AsyncChannelKind,
    K::Sender<T>: Debug + Clone + SendAsyncLike<T>,
{
    type SendError = <K::Sender<T> as SendAsyncLike<T>>::SendError;
    type SendFuture<'a>
        = <K::Sender<T> as SendAsyncLike<T>>::SendFuture<'a>
    where
        Self: 'a,
        T: 'a;

    fn send(&self, item: T) -> Self::SendFuture<'_> {
        SendAsyncLike::send_async(&self.tx, item)
    }
}

pub trait SyncRx<T> {
    type ReceiveError: Debug;

    fn recv(&mut self) -> Result<T, Self::ReceiveError>;
}

pub trait AsyncRx<T> {
    type ReceiveError: Debug;
    type RecvFuture<'a>: Future<Output = Result<T, Self::ReceiveError>> + 'a
    where
        Self: 'a,
        T: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_>;
}

#[derive(Debug)]
pub struct UnboundedRx<K, T>
where
    K: ChannelBaseKind,
    K::Receiver<T>: Debug,
{
    pub rx: K::Receiver<T>,
}

impl<K, T> SyncRx<T> for UnboundedRx<SyncChannel<K>, T>
where
    K: SyncChannelKind,
    K::Receiver<T>: Debug + RecvSyncLike<T>,
{
    type ReceiveError = <K::Receiver<T> as RecvSyncLike<T>>::ReceiveError;

    fn recv(&mut self) -> Result<T, Self::ReceiveError> {
        RecvSyncLike::recv_sync(&mut self.rx)
    }
}

impl<K, T> AsyncRx<T> for UnboundedRx<AsyncChannel<K>, T>
where
    K: AsyncChannelKind,
    K::Receiver<T>: Debug + RecvAsyncLike<T>,
{
    type ReceiveError = <K::Receiver<T> as RecvAsyncLike<T>>::ReceiveError;
    type RecvFuture<'a>
        = <K::Receiver<T> as RecvAsyncLike<T>>::RecvFuture<'a>
    where
        Self: 'a,
        T: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        RecvAsyncLike::recv_async(&mut self.rx)
    }
}

pub trait SendSyncLike<T> {
    type SendError: Debug;

    fn send_sync(&self, item: T) -> Result<(), Self::SendError>;
}

pub trait RecvSyncLike<T> {
    type ReceiveError: Debug;

    fn recv_sync(&mut self) -> Result<T, Self::ReceiveError>;
}

pub trait SendAsyncLike<T> {
    type SendError: Debug;
    type SendFuture<'a>: Future<Output = Result<(), Self::SendError>> + 'a
    where
        Self: 'a,
        T: 'a;

    fn send_async(&self, item: T) -> Self::SendFuture<'_>;
}

pub trait RecvAsyncLike<T> {
    type ReceiveError: Debug;
    type RecvFuture<'a>: Future<Output = Result<T, Self::ReceiveError>> + 'a
    where
        Self: 'a,
        T: 'a;

    fn recv_async(&mut self) -> Self::RecvFuture<'_>;
}

pub fn mpsc_unbounded<K, T>() -> (UnboundedTx<K, T>, UnboundedRx<K, T>)
where
    K: ChannelBaseKind,
    K::Sender<T>: Debug + Clone,
    K::Receiver<T>: Debug,
{
    let (tx, rx) = K::unbounded::<T>();

    (UnboundedTx { tx }, UnboundedRx { rx })
}
