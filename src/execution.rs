pub trait ExecutionLayer {
    type Item;
    type Transformer;

    fn next(&self) -> Option<Self::Item>;
}

pub trait ExecutionLayerAsync {
    type Item;
    type Transformer;

    async fn next(&self) -> Option<Self::Item>;
}

#[derive(Debug)]
pub struct Execution {}
