use std::fmt::Debug;

pub struct SyncChannel;
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

#[derive(Debug, Clone)]
pub struct UnboundedTx<K, T>
where
    K: ChannelKind,
    <K as ChannelKind>::Sender<T>: Debug + Clone,
{
    pub tx: <K as ChannelKind>::Sender<T>,
}

#[derive(Debug)]
pub struct UnboundedRx<K, T>
where
    K: ChannelKind,
    <K as ChannelKind>::Receiver<T>: Debug,
{
    pub rx: <K as ChannelKind>::Receiver<T>,
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

// pub fn mpsc_unbounded<T>() -> (UnboundedTx<T>, UnboundedRx<T>) {
//     let (tx, rx) = kanal::unbounded::<T>();

//     (UnboundedTx::new(tx), UnboundedRx::new(rx))
// }

// #[derive(Debug, Clone, Constructor)]
// pub struct UnboundedTx<T> {
//     pub tx: kanal::Sender<T>,
// }

// #[derive(Debug, Constructor)]
// pub struct UnboundedRx<T> {
//     pub rx: kanal::Receiver<T>,
// }

// #[derive(Debug)]
// pub struct Channel<T> {
//     pub tx: kanal::Sender<T>,
//     pub rx: kanal::Receiver<T>,
// }

// impl<T> Channel<T> {
//     pub fn new() -> Self {
//         let (tx, rx) = kanal::unbounded();
//         Self { tx, rx }
//     }
// }

// impl<T> Default for Channel<T> {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// pub trait Unrecoverable {
//     fn is_unrecoverable(&self) -> bool;
// }

// pub trait Tx
// where
//     Self: Debug + Clone + Send,
// {
//     type Item;
//     type Error: Unrecoverable + Debug;
//     fn send<Item: Into<Self::Item>>(&self, item: Item) -> Result<(), Self::Error>;
// }

// #[derive(Debug, Clone)]
// pub struct UnboundedTx<T> {
//     pub tx: kanal::Sender<T>,
// }

// impl<T> Tx for UnboundedTx<T>
// where
//     T: Debug + Clone + Send,
// {
//     type Item = T;
//     type Error = kanal::SendError;

//     fn send<Item: Into<Self::Item>>(&self, item: Item) -> Result<(), Self::Error> {
//         self.tx.send(item.into())
//     }
// }

// impl Unrecoverable for kanal::SendError {
//     fn is_unrecoverable(&self) -> bool {
//         true
//     }
// }

// #[derive(Debug, Constructor)]
// pub struct UnboundedRx<T> {
//     pub rx: kanal::Receiver<T>,
// }
