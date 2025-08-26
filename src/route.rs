use derive_more::Constructor;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct RoutingKey<K, V> {
    key: K,
    value: V,
}

#[derive(Debug)]
pub struct RoutingTable<F, E, A, I> {
    fronts: Vec<RoutingKey<FrontIndex, F>>,
    exchanges: Vec<RoutingKey<ExchangeIndex, E>>,
    assets: Vec<RoutingKey<AssetIndex, A>>,
    instruments: Vec<RoutingKey<InstrumentIndex, I>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct FrontIndex(usize);
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct ExchangeIndex(usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct AssetIndex(usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct InstrumentIndex(usize);
