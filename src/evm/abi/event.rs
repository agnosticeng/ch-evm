use anyhow::Result;
use alloy::primitives::B256;
use alloy::dyn_abi::{DynSolEvent,Specifier,DecodedEvent};

#[derive(Debug, Clone)]
pub struct Event {
    d: DynSolEvent,
    e: alloy::json_abi::Event,
    selector: B256,
    sig: String,
    fullsig: String
}

impl Event {
    pub fn new(e: alloy::json_abi::Event) -> Result<Self> {
        Ok(Event { 
            selector: e.selector(),
            sig: e.signature(),
            fullsig: e.full_signature(),
            d: e.resolve()?,
            e
        })
    }

    pub fn parse(s: &str) -> Result<Self> {
        Self::new(alloy::json_abi::Event::parse(s)?)
    }

    pub fn selector(&self) -> B256 {
        self.selector
    }

    pub fn name(&self) -> &str {
        &self.e.name
    }

    pub fn sig(&self) -> &str {
        &self.sig
    }

    pub fn fullsig(&self) -> &str {
        &self.fullsig
    }

    pub fn inputs(&self) -> &Vec<alloy::json_abi::EventParam> {
        &self.e.inputs
    }

    pub fn decode_log_parts<I>(&self, topics: I, data: &[u8]) -> Result<DecodedEvent, alloy::dyn_abi::Error>
    where
        I: IntoIterator<Item = B256>
    {
        self.d.decode_log_parts(topics, data)
    }
}