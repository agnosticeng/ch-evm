use std::sync::Arc;
use anyhow::Result;
use alloy_json_abi::AbiItem;

pub trait AbiItemProvider {
    fn get_abi_item(&self, sighash: &[u8]) -> Result<&AbiItem>;
}

pub struct AbiItemProviderFactory;

impl AbiItemProviderFactory {
    pub fn create(s: &str) -> Result<Arc<dyn AbiItemProvider + Sync + Send>> {
        FullsigAbiItemProvider::new(s).map(|x| Arc::new(x) as Arc<dyn AbiItemProvider + Sync + Send>)
    }
}

struct FullsigAbiItemProvider<'a> {
    item: AbiItem<'a>
}

impl FullsigAbiItemProvider<'_> {
    pub fn new(s: &str) -> Result<Self> {
        let item = AbiItem::parse(s)?;
        Ok(Self { item })
    }
}

impl AbiItemProvider for FullsigAbiItemProvider<'_> {
    fn get_abi_item(&self, _: &[u8]) -> Result<&AbiItem> {
        Ok(&self.item)
    }
}
