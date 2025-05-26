use alloy::hex::ToHexExt;
use alloy_dyn_abi::EventExt;
use alloy_primitives::Bytes;
use alloy_json_abi::Event;
use alloy_primitives::B256;
use alloy_dyn_abi::DynSolValue;
use serde_json::{Value, Map, Number};
use anyhow::{Context, Ok, Result};

mod abi_item_provider;

pub use crate::evm::abi_item_provider::AbiItemProviderFactory;

pub trait EventJSONExt {
    fn decode_log_parts_to_json_value<I>(&self, topics: I, data: &[u8]) -> Result<Value>
    where
        I: IntoIterator<Item = B256>;
}

impl EventJSONExt for Event {
    fn decode_log_parts_to_json_value<I>(&self, topics: I, data: &[u8]) -> Result<Value>
    where
        I: IntoIterator<Item = B256>,
    {
        let decoded = self.decode_log_parts(topics, data)?;
        let mut res = Map::new();
        let mut indexed_iter = decoded.indexed.iter();
        let mut body_iter = decoded.body.iter();

        res.insert("signature".to_string(), Value::String(self.signature()));
        res.insert("fullsig".to_string(), Value::String(self.full_signature()));

        for (i, param) in self.inputs.iter().enumerate() {
            if param.indexed {
                res.insert(format!("arg{}", i), dyn_sol_value_to_json_value(indexed_iter.next().context("not enough indexed values")?)?);
            } else {
                res.insert(format!("arg{}", i), dyn_sol_value_to_json_value(body_iter.next().context("not enough body values")?)?);
            }
        }

        Ok(Value::Object(res))
    }
}

fn dyn_sol_value_to_json_value(v: &DynSolValue) -> Result<Value> {
    match v {
        DynSolValue::Bool(b) =>
            Ok(Value::Bool(*b)),
        DynSolValue::String(s) =>
            Ok(Value::String(s.clone())),
        DynSolValue::Bytes(data) =>
            Ok(Value::String(Bytes::copy_from_slice(data).to_string())),
        DynSolValue::FixedBytes(w, _) =>
            Ok(Value::String(w.to_string())),
        DynSolValue::Address(addr) => 
            Ok(Value::String(addr.encode_hex_with_prefix())),
        DynSolValue::Function(func) => 
            Ok(Value::String(func.to_string())),
        DynSolValue::Uint(i, _, ) if i.bit_len() <= 32 => {
            Ok(Value::Number(Number::from(i.to::<u32>())))
        },
        DynSolValue::Uint(i, _) => 
            Ok(Value::String(i.to_string())),
        DynSolValue::Int(i, size, ) if *size <= 32 =>
            Ok(Value::Number(Number::from(i.as_i32()))),
        DynSolValue::Int(i, _) => 
            Ok(Value::String(i.to_string())),
        DynSolValue::Array(values) =>
            Ok(Value::Array(values
                .iter()
                .map(dyn_sol_value_to_json_value)
                .collect::<Result<Vec<Value>, _>>()?
            )),
        DynSolValue::FixedArray(values) =>
            Ok(Value::Array(values
                .iter()
                .map(dyn_sol_value_to_json_value)
                .collect::<Result<Vec<Value>, _>>()?
            )),
        DynSolValue::Tuple(values) => {
            let params = values
                .iter()
                .map(dyn_sol_value_to_json_value)
                .collect::<Result<Vec<Value>, _>>()?;

            Ok(Value::Object(
                params
                .iter()
                .enumerate()
                .map(|(i, value)| (format!("arg{}", i), value.clone()))
                .collect::<Map<String, Value>>()
            ))
        }
    }
}