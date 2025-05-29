use std::collections::HashMap;
use anyhow::{anyhow, Context, Result};
use url::Url;
use alloy::json_abi::JsonAbi;
use arrow::datatypes::ToByteSlice;
use object_store::parse_url;
use super::event::Event;
use super::function::Function;
use super::abi_item::AbiItem;
use super::abi_item_provider::AbiItemProvider;

pub struct FileAbiItemProvider {
    m: HashMap<Vec<u8>,AbiItem>
}

impl FileAbiItemProvider {
    pub async fn new(s: &str) -> Result<Self> {
        let u = Url::parse(s).context("cannot parse URL")?;
        let (objstr, path) = parse_url(&u)?;
        let content = objstr.get(&path).await?.bytes().await?;
        let abi: JsonAbi = serde_json::from_str(std::str::from_utf8(&content)?)?;
        let mut m: HashMap<Vec<u8>, AbiItem> = HashMap::new();

        for abi_item in abi.into_items() {
            match abi_item {
                alloy::json_abi::AbiItem::Event(e) => {
                    let s = e.selector();
                    let item = AbiItem::Event(Event::new(e.into_owned())?);
                     m.insert(s.to_byte_slice().to_vec(), item); 
                },
                alloy::json_abi::AbiItem::Function(f) => { 
                    let s = f.selector();
                    let item = AbiItem::Function(Function::new(f.into_owned())?);
                    m.insert(s.to_vec(), item); 
                },
                _ => ()
            }
        }
       
        Ok(Self { m })
    }
}

impl AbiItemProvider for FileAbiItemProvider {
    fn get_abi_item(&self, selector: &[u8]) -> Result<&AbiItem> {
        self.m.get(selector).ok_or(anyhow!("no ABI field found for selector"))
    }
}