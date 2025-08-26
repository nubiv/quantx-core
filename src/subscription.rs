pub trait SubscriptionLayer {
    type Item;
    type Transformer;

    fn next(&self) -> Option<Self::Item>;
}

pub trait SubscriptionLayerAsync {
    type Item;
    type Transformer;

    async fn next(&self) -> Option<Self::Item>;
}

#[derive(Debug)]
pub struct Subscription {}
