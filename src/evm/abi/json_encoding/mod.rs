use std::io::Write;
use anyhow::{anyhow, Context, Result};
use serde_json::ser::{Formatter,CompactFormatter};
use alloy::hex::ToHexExt;
use alloy::primitives::Bytes;
use alloy::dyn_abi::{DecodedEvent,DynSolValue};
use crate::evm::abi::{Event, Function};
use lazy_static::lazy_static;
use crate::json::format_escaped_str_contents;

lazy_static! {
    static ref positional_arg_names: Vec<String> = {
        (0..1024).map(|i| format!("arg{}", i)).collect() 
    };
}

pub fn encode_call<'a, I>(func: &Function, inputs: I, outputs: Option<I>) -> Result<Vec<u8>> 
where 
    I: IntoIterator<Item = &'a DynSolValue>, 
{
    let mut w = Vec::<u8>::with_capacity(1024);
    let mut f = CompactFormatter;

    f.begin_object(&mut w)?;
    f.write_object_key(&mut w, "value", true)?;
    f.begin_object_value(&mut w)?;
    f.begin_object(&mut w)?;

    f.write_key_value_str(&mut w, "signature", func.sig(), true)?;
    f.write_key_value_str(&mut w, "fullsig", func.fullsig(), false)?;

    f.write_object_key(&mut w, "inputs", false)?;
    f.begin_object_value(&mut w)?;
    f.write_values_as_object(&mut w, inputs.into_iter()
        .enumerate()
        .map(|(i, v)| (positional_arg_names[i].clone(), v))
    )?;

    f.end_object_value(&mut w)?;

    if outputs.is_some() {
        f.write_object_key(&mut w, "outputs", false)?;
        f.begin_object_value(&mut w)?;
        f.write_values_as_object(&mut w, outputs
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(i, v)| (positional_arg_names[i].clone(), v))
        )?;
        f.end_object_value(&mut w)?;
    }

    f.end_object(&mut w)?;
    f.end_object_value(&mut w)?;
    f.end_object(&mut w)?;
    Ok(w)
}

pub fn encode_event(evt: &Event, dec_evt: &DecodedEvent) -> Result<Vec<u8>>
{
    let mut w = Vec::<u8>::with_capacity(1024);
    let mut f = CompactFormatter;
    let mut indexed_iter = dec_evt.indexed.iter();
    let mut body_iter = dec_evt.body.iter();

    f.begin_object(&mut w)?;
    f.write_object_key(&mut w, "value", true)?;
    f.begin_object_value(&mut w)?;
    f.begin_object(&mut w)?;

    f.write_key_value_str(&mut w, "signature", evt.sig(), true)?;
    f.write_key_value_str(&mut w, "fullsig", evt.fullsig(), false)?;
    f.write_object_key(&mut w, "inputs", false)?;
    f.begin_object_value(&mut w)?;
    f.begin_object(&mut w)?;

    for (i, param) in evt.inputs().iter().enumerate() {
        let v = if param.indexed {
            indexed_iter.next().ok_or(anyhow!("not enough indexed values"))?
        } else {
            body_iter.next().ok_or(anyhow!("not enough unindexed values"))?
        };

        f.write_key_value(&mut w, &positional_arg_names[i], v, i==0)?;
    }

    f.end_object(&mut w)?;
    f.end_object_value(&mut w)?;
    f.end_object(&mut w)?;
    f.end_object(&mut w)?;
    Ok(w)
}

pub fn encode_values<'a, I>(values: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = (String, &'a DynSolValue)>
{
    let mut w = Vec::<u8>::with_capacity(1024);
    let mut f = CompactFormatter;
    f.begin_object(&mut w)?;
    f.write_object_key(&mut w, "value", true)?;
    f.begin_object_value(&mut w)?;
    f.write_values_as_object(&mut w, values)?;
    f.end_object_value(&mut w)?;
    f.end_object(&mut w)?;
    Ok(w)
}

pub trait CompactFormatterExt {
    fn write_single_fragment_string<W>(&mut self, w: &mut W, s: &str) -> Result<()> 
    where 
        W: ?Sized + Write;

    fn write_string<W>(&mut self, w: &mut W, s: &str) -> Result<()>
    where
        W: ?Sized + Write;

    fn write_object_key<W>(&mut self, w: &mut W, value: &str, first: bool) -> Result<()>
    where
        W: ?Sized + Write;

    fn write_value<W>(&mut self, w: &mut W, value: &DynSolValue) -> Result<()> 
    where 
        W: ?Sized + Write;

    fn write_values_as_array<'a, I, W>(&mut self, w: &mut W, params: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a DynSolValue>, 
        W: ?Sized + Write;

    fn write_values_as_object<'a, I, W>(&mut self, w: &mut W, params: I) -> Result<()>
    where
        I: IntoIterator<Item = (String, &'a DynSolValue)>, 
        W: ?Sized + Write;

    fn write_key_value<W>(&mut self, w: &mut W, key: &str, value: &DynSolValue, first: bool) -> Result<()>
    where
        W: ?Sized + Write;

    fn write_key_value_str<W>(&mut self, w: &mut W, key: &str, value: &str, first: bool) -> Result<()>
    where
        W: ?Sized + Write;
}

impl CompactFormatterExt for CompactFormatter {
    #[inline(always)]
    fn write_single_fragment_string<W>(&mut self, w: &mut W, s: &str) -> Result<()>
    where
        W: ?Sized + Write
    {
        self.begin_string(w)?;
        self.write_string_fragment(w, s)?;
        self.end_string(w).context("failed to write string")
    }

    #[inline(always)]
    fn write_string<W>(&mut self, w: &mut W, s: &str) -> Result<()> 
    where 
        W: ?Sized + Write
    {
        self.begin_string(w)?;
        format_escaped_str_contents(w, self, s)?;
        self.end_string(w).context("failed to write single fragment string")
    }

    #[inline(always)]
    fn write_object_key<W>(&mut self, w: &mut W, value: &str, first: bool) -> Result<()>
    where
        W: ?Sized + Write
    {
        self.begin_object_key(w, first)?;
        self.write_single_fragment_string(w, value)?;
        self.end_object_key(w).context("failed to write object key")
    }

    fn write_value<W>(&mut self, w: &mut W, value: &DynSolValue) -> Result<()> 
    where 
        W: ?Sized + Write
    {
        match value {
            DynSolValue::Bool(b) => 
                self.write_bool(w, *b).context("failed to write Bool"),
            DynSolValue::String(s) => {
                self.begin_string(w)?;
                self.write_string(w, s)?;
                self.end_string(w).context("failed to write String")
            }
            DynSolValue::Bytes(data) => 
                self.write_single_fragment_string(w, &Bytes::copy_from_slice(data).to_string()),
            DynSolValue::FixedBytes(data, _) => 
                self.write_single_fragment_string(w, &data.to_string()),
            DynSolValue::Address(addr) => 
                self.write_single_fragment_string(w, &addr.encode_hex_with_prefix()),
            DynSolValue::Function(func) => 
                self.write_single_fragment_string(w,&func.to_string()),
            DynSolValue::Uint(i, _, ) if i.bit_len() <= 32 => 
                self.write_u32(w, i.to::<u32>()).context("failed to write Uint"),
            DynSolValue::Uint(i, _) => 
                self.write_single_fragment_string(w, &i.to_string()),
            DynSolValue::Int(i, size, ) if *size <= 32 => 
                self.write_i32(w, i.as_i32()).context("failed to write Int"),
            DynSolValue::Int(i, _) => 
                self.write_single_fragment_string(w, &i.to_string()),
            DynSolValue::Array(values) => 
                self.write_values_as_array(w, values),
            DynSolValue::FixedArray(values) =>
                self.write_values_as_array(w, values),
            DynSolValue::Tuple(values) => 
                self.write_values_as_object(w, values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (positional_arg_names[i].clone(), v))
                ),
            DynSolValue::CustomStruct { prop_names, tuple, .. } => 
                self.write_values_as_object(w, tuple
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (prop_names[i].clone(), v))
                )
        }
    }

    #[inline(always)]
    fn write_values_as_array<'a, I, W>(&mut self, w: &mut W, values: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a DynSolValue>, 
        W: ?Sized + Write
    {
        self.begin_array(w)?;

        for (i, v) in values.into_iter().enumerate() {
            self.begin_array_value(w, i == 0)?;
            self.write_value(w,v)?;
            self.end_array_value(w)?;
        }

        self.end_array(w).context("failed to write array")
    }

    #[inline(always)]
    fn write_values_as_object<'a, I, W>(&mut self, w: &mut W, values: I) -> Result<()>
    where
        I: IntoIterator<Item = (String, &'a DynSolValue)>, 
        W: ?Sized + Write
    {
        self.begin_object(w)?;

        for (i, (key, value)) in values.into_iter().enumerate() {
            self.write_key_value(w, &key, value, i==0)?;
        }

        self.end_object(w).context("failed to write object")
    }

    #[inline(always)]
    fn write_key_value<W>(&mut self, w: &mut W, key: &str, value: &DynSolValue, first: bool) -> Result<()>
    where
        W: ?Sized + Write
    {
        self.write_object_key(w, key, first)?;
        self.begin_object_value(w)?;
        self.write_value(w, value)?;
        self.end_object_value(w).context("failed to write key-value")
    }

    #[inline(always)]
    fn write_key_value_str<W>(&mut self, w: &mut W, key: &str, value: &str, first: bool) -> Result<()>
    where
        W: ?Sized + Write
    {
        self.write_object_key(w, key, first)?;
        self.begin_object_value(w)?;
        self.write_string(w, value)?;
        self.end_object_value(w).context("failed to write key-value")
    }
}

