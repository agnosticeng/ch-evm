use std::io::Write;
use serde_json::ser::{Formatter,CompactFormatter};
use alloy::hex::ToHexExt;
use alloy_dyn_abi::{DecodedEvent};
use alloy_primitives::{Bytes};
use alloy_json_abi::{Event};
use alloy_dyn_abi::DynSolValue;
use anyhow::{Context, Result};
use crate::json::format_escaped_str_contents;
pub trait DecodedEventExt {
    fn format_as_json<W>(&self, evt: &Event, w: &mut W) -> Result<()>
    where
        W: ?Sized + Write;
}

impl DecodedEventExt for DecodedEvent {
    fn format_as_json<W>(&self, evt: &Event, w: &mut W) -> Result<()>
    where
        W: ?Sized + Write
    {
        let mut f = CompactFormatter;
        let mut indexed_iter = self.indexed.iter();
        let mut body_iter = self.body.iter();

        f.begin_object(w)?;

        f.begin_object_key(w, true)?;
        format_string(w, &mut f, "signature")?;
        f.end_object_key(w)?;
        f.begin_object_value(w)?;
        format_string(w, &mut f, &evt.signature())?;
        f.end_object_value(w).context("failed to format Object key-value")?;

        f.begin_object_key(w, false)?;
        format_string(w, &mut f, "fullsig")?;
        f.end_object_key(w)?;
        f.begin_object_value(w)?;
        format_string(w, &mut f, &evt.full_signature())?;
        f.end_object_value(w).context("failed to format Object key-value")?;

        f.begin_object_key(w, false)?;
        format_string(w, &mut f, "inputs")?;
        f.end_object_key(w)?;
        f.begin_object_value(w)?;
        f.begin_object(w)?;

        for (i, param) in evt.inputs.iter().enumerate() {
            f.begin_object_key(w, i==0)?;
            format_string(w, &mut f, &format!("arg{}", i))?;
            f.end_object_key(w)?;
            f.begin_object_value(w)?;

            if param.indexed {
                format_dyn_sol_value(w, &mut f, indexed_iter.next().context("not enough indexed values")?)?;
            } else {
                format_dyn_sol_value(w, &mut f, body_iter.next().context("not enough unindexed values")?)?;
            }

            f.end_object_value(w).context("failed to format Object key-value")?;
        }

        f.end_object(w)?;
        f.end_object_value(w)?;
        f.end_object(w).context("failed to format Object")
    }
}

fn format_dyn_sol_value<W>(w: &mut W, f: &mut CompactFormatter,  v: &DynSolValue) -> Result<()> 
    where 
        W: ?Sized + Write,
{
    match v {
        DynSolValue::Bool(b) => 
            f.write_bool(w, *b).context("failed to format Bool"),
        DynSolValue::String(s) => {
            f.begin_string(w)?;
            format_escaped_str_contents(w, f, s)?;
            f.end_string(w).context("failed to format String")
        }
        DynSolValue::Bytes(data) => 
            format_string(w, f, &Bytes::copy_from_slice(data).to_string()),
        DynSolValue::FixedBytes(data, _) => 
            format_string(w, f, &data.to_string()),
        DynSolValue::Address(addr) => 
            format_string(w, f, &addr.encode_hex_with_prefix()),
        DynSolValue::Function(func) => 
            format_string(w, f, &func.to_string()),
        DynSolValue::Uint(i, _, ) if i.bit_len() <= 32 => 
            f.write_u32(w, i.to::<u32>()).context("failed to format Uint"),
        DynSolValue::Uint(i, _) => 
            format_string(w, f, &i.to_string()),
        DynSolValue::Int(i, size, ) if *size <= 32 => 
            f.write_i32(w, i.as_i32()).context("failed to format Int"),
        DynSolValue::Int(i, _) => 
            format_string(w, f, &i.to_string()),
        DynSolValue::Array(values) => {
            f.begin_array(w)?;

            for (i, v) in values.iter().enumerate() {
                f.begin_array_value(w, i == 0)?;
                format_dyn_sol_value(w, f, v)?;
                f.end_array_value(w)?;
            }

            f.end_array(w).context("failed to format Array")
        }
        DynSolValue::FixedArray(values) => {
            f.begin_array(w)?;

            for (i, v) in values.iter().enumerate() {
                f.begin_array_value(w, i == 0)?;
                format_dyn_sol_value(w, f, v)?;
                f.end_array_value(w)?;
            }

            f.end_array(w).context("failed to format Array")
        }
        DynSolValue::Tuple(values) => {
            f.begin_object(w)?;

            for (i, v) in values.iter().enumerate() {
                format_object_key_value(w, f, &format!("arg{}", i), v, i==0)?;
            }

            f.end_object(w).context("failed to format Object")
        }
    }
}


fn format_string<W>(w: &mut W, f: &mut CompactFormatter, s: &str) -> Result<()> 
    where 
        W: ?Sized + Write,
{
    f.begin_string(w)?;
    f.write_string_fragment(w, s)?;
    f.end_string(w).context("failed to format String")
}

fn format_object_key_value<W>(w: &mut W, f: &mut CompactFormatter, key: &str, value: &DynSolValue, first: bool) -> Result<()> 
    where 
        W: ?Sized + Write,
{
    f.begin_object_key(w, first)?;
    format_string(w, f, key)?;
    f.end_object_key(w)?;
    f.begin_object_value(w)?;
    format_dyn_sol_value(w, f, value)?;
    f.end_object_value(w).context("failed to format Object key-value")
}