use derive_more::Constructor;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct RoutingKey<K, V> {
    key: K,
    value: V,
}

#[derive(Debug)]
pub struct RoutingTable<Front, Exchange, Asset, Instrument> {
    fronts: Vec<RoutingKey<FrontIndex, Front>>,
    exchanges: Vec<RoutingKey<ExchangeIndex, Exchange>>,
    assets: Vec<RoutingKey<AssetIndex, Asset>>,
    instruments: Vec<RoutingKey<InstrumentIndex, Instrument>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct FrontIndex(usize);
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct ExchangeIndex(usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct AssetIndex(usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct InstrumentIndex(usize);
