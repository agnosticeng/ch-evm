use anyhow::{Result,bail};
use super::event::Event;
use super::function::Function;

#[derive(Debug, Clone)]

pub enum AbiItem {
    Event(Event),
    Function(Function)
}

impl AbiItem {
    pub fn parse(s: &str) -> Result<AbiItem> {
        let item = alloy::json_abi::AbiItem::parse(s)?;
        
        match item {
            alloy::json_abi::AbiItem::Function(f) => Ok(AbiItem::Function(Function::new(f.into_owned())?)),
            alloy::json_abi::AbiItem::Event(e) => Ok(AbiItem::Event(Event::new(e.into_owned())?)),
            _ => bail!("ABI item is neither function nor event")
        }
    }
}