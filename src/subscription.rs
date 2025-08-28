// pub trait SubscriptionLayer {
//     type Item;
//     type Transformer;

//     fn next(&self) -> Option<Self::Item>;
// }

// pub trait SubscriptionLayerAsync {
//     type Item;
//     type Transformer;

//     async fn next(&self) -> Option<Self::Item>;
// }

#[derive(Debug)]
pub struct SubscriptionLayer<Stream, Transformer, Tx> {
    stream: Stream,
    transformer: Transformer,
    txs_to_market: Tx,
}

impl<Stream, Transformer, Tx> SubscriptionLayer<Stream, Transformer, Tx> {
    pub fn subscribe() -> impl Future<Output = ()> {
        async {}
    }

    pub fn subscribe_batch() -> impl Future<Output = ()> {
        async {}
    }

    pub fn forward_to(self, tx_to_engien: Tx) -> impl Future<Output = ()> {
        async {}
    }
}
