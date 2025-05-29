use std::sync::Arc;
use anyhow::{Result,bail};
pub use super::abi_item::AbiItem;
pub use super::event::Event;
pub use super::function::Function;
use super::file_abi_item_provider::FileAbiItemProvider;
use super::fullsig_abi_item_provider::FullsigAbiItemProvider;

pub trait AbiItemProvider {
    fn get_abi_item(&self, selector: &[u8]) -> Result<&AbiItem>;

    fn get_event(&self, selector: &[u8]) -> Result<&Event> {
        match self.get_abi_item(selector)? {
            AbiItem::Event(evt) => Ok(evt),
            _ => bail!("abi item is not an event")
        }
    }

    fn get_function(&self, selector: &[u8]) -> Result<&Function> {
        match self.get_abi_item(selector)? {
            AbiItem::Function(func) => Ok(func),
            _ => bail!("abi item is not an event")
        }
    }
}

pub struct AbiItemProviderFactory;

impl AbiItemProviderFactory {
    pub async fn create(s: &str) -> Result<Arc<dyn AbiItemProvider + Sync + Send>> {
        match s {
            _ if s.starts_with("http://") => FileAbiItemProvider::new(s)
                .await
                .map(Arc::new)
                .map(|x| x as Arc<dyn AbiItemProvider + Sync + Send> ),
            _ if s.starts_with("https://") => FileAbiItemProvider::new(s)
                .await
                .map(Arc::new)
                .map(|x| x as Arc<dyn AbiItemProvider + Sync + Send> ),
            _ if s.starts_with("file://") => FileAbiItemProvider::new(s)
                .await
                .map(Arc::new)
                .map(|x| x as Arc<dyn AbiItemProvider + Sync + Send> ),
            _ => FullsigAbiItemProvider::new(s)
                .map(Arc::new)
                .map(|x| x as Arc<dyn AbiItemProvider + Sync + Send>)
        }
    }
}