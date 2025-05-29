use anyhow::{bail, Result};
use super::abi_item::AbiItem;
use super::abi_item_provider::AbiItemProvider;

pub struct FullsigAbiItemProvider {
    item: AbiItem
}

impl FullsigAbiItemProvider {
    pub fn new(s: &str) -> Result<Self> {
        let item = AbiItem::parse(s)?;
        Ok(FullsigAbiItemProvider { item })
    }
}

impl AbiItemProvider for FullsigAbiItemProvider {
    fn get_abi_item(&self, selector: &[u8]) -> Result<&AbiItem> {
        match &self.item {
            AbiItem::Function(func) if func.selector() == selector => Ok(&self.item),
            AbiItem::Event(evt) if evt.selector() == selector => Ok(&self.item),
            _ => bail!("selector does not match") 
        }
    }
}
