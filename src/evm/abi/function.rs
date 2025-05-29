use anyhow::Result;
use alloy::primitives::Selector;
use alloy::dyn_abi::{DynSolValue,DynSolType,DynSolCall,DynSolReturns,Specifier};

#[derive(Debug, Clone)]

pub struct Function {
    d: Box<DynSolCall>,
    f: alloy::json_abi::Function,
    selector: Selector,
    sig: String,
    fullsig: String
}

impl Function {
    pub fn new(f: alloy::json_abi::Function) -> Result<Self> {
        let inputs = f.inputs
            .iter()
            .map(|p| p.resolve())
            .collect::<Result<Vec<DynSolType>, alloy::dyn_abi::Error>>()?;

        let outputs = f.outputs
            .iter()
            .map(|p| p.resolve())
            .collect::<Result<Vec<DynSolType>, alloy::dyn_abi::Error>>()?;

        Ok(Function { 
            selector: f.selector(),
            sig: f.signature(),
            fullsig: f.full_signature(),
            d: Box::new(DynSolCall::new(
                f.selector(),
                    inputs,
                    Some(f.name.clone()),
                    DynSolReturns::new(outputs)
                )
            ),
            f
        })
    }

    pub fn parse(s: &str) -> Result<Self> {
        Self::new(alloy::json_abi::Function::parse(s)?)
    }

    pub fn selector(&self) -> Selector {
        self.selector
    }

    pub fn name(&self) -> &str {
        &self.f.name
    }

    pub fn sig(&self) -> &str {
        &self.sig
    }

    pub fn fullsig(&self) -> &str {
        &self.fullsig
    }

    pub fn abi_decode_input(&self, data: &[u8]) -> Result<Vec<DynSolValue>, alloy::dyn_abi::Error> {
        self.d.abi_decode_input(data)
    }

    pub fn abi_decode_output(&self, data: &[u8]) -> Result<Vec<DynSolValue>, alloy::dyn_abi::Error> {
        self.d.abi_decode_output(data)
    }

    pub fn abi_encode_input(&self, values: &[DynSolValue]) -> Result<Vec<u8>, alloy::dyn_abi::Error> {
        self.d.abi_encode_input(values)
    }

    pub fn inputs(&self) -> &Vec<alloy::json_abi::Param> {
        &self.f.inputs
    }

    pub fn outputs(&self) -> &Vec<alloy::json_abi::Param> {
        &self.f.outputs
    }

    pub fn coerce_inputs(&self, inputs: &[serde_json::Value]) -> Result<Vec<DynSolValue>, alloy::dyn_abi::Error>
    {
        (0..self.f.inputs.len())
            .map(|i| self.f.inputs[i].resolve()?.coerce_json(&inputs[i]))
            .collect()
    }
}
