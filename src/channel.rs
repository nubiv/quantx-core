use std::fmt::Debug;

use derive_more::Constructor;

#[derive(Debug)]
pub struct Channel<T> {
    pub tx: kanal::Sender<T>,
    pub rx: kanal::Receiver<T>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        let (tx, rx) = kanal::unbounded();
        Self { tx, rx }
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Unrecoverable {
    fn is_unrecoverable(&self) -> bool;
}

pub trait Tx
where
    Self: Debug + Clone + Send,
{
    type Item;
    type Error: Unrecoverable + Debug;
    fn send<Item: Into<Self::Item>>(&self, item: Item) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct UnboundedTx<T> {
    pub tx: kanal::Sender<T>,
}

impl<T> Tx for UnboundedTx<T>
where
    T: Debug + Clone + Send,
{
    type Item = T;
    type Error = kanal::SendError;

    fn send<Item: Into<Self::Item>>(&self, item: Item) -> Result<(), Self::Error> {
        self.tx.send(item.into())
    }
}

impl Unrecoverable for kanal::SendError {
    fn is_unrecoverable(&self) -> bool {
        true
    }
}

#[derive(Debug, Constructor)]
pub struct UnboundedRx<T> {
    pub rx: kanal::Receiver<T>,
}
