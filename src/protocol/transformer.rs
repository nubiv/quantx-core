use serde::Deserialize;

pub trait Transformer {
    type Input: for<'de> Deserialize<'de>;
    type OutputItem;
    type Error;
}

pub trait TransformerSingle
where
    Self: Transformer,
{
    fn transform_one(&mut self, input: Self::Input) -> Result<Self::OutputItem, Self::Error>;
}

pub trait TransformerBatch
where
    Self: Transformer,
{
    type OutputIter: IntoIterator<Item = Result<Self::OutputItem, Self::Error>>;

    fn transform_many<'a>(&'a mut self, input: Self::Input) -> Self::OutputIter;
}
