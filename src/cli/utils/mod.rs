use std::str;
use std::io::{stdin,stdout,Read,Write};
use std::fs::File;
use std::sync::Arc;
use anyhow::{anyhow, bail, Result};
use arrow::array::{ArrayRef,RecordBatch};
use quick_cache::sync::Cache;
use crate::evm::abi::{AbiItemProvider,AbiItemProviderFactory, Function};

pub fn open_file_or_stdin(path: &str) -> Result<Box<dyn Read>> {
    if path.is_empty() {
        Ok(Box::new(stdin()))
    } else {
        match File::open(path) {
            Ok(f) => Ok(Box::new(f)),
            Err(e) => bail!(e)
        }
    }
}

pub fn create_file_or_stdout(path: &str) -> Result<Box<dyn Write>> {
    if path.is_empty() {
        Ok(Box::new(stdout()))
    } else {
        match File::create(path) {
            Ok(f) => Ok(Box::new(f)),
            Err(e) => bail!(e)
        }
    }
}


pub trait RecordBatchExt {
    fn get_column<T: 'static>(&self, col_name: &str) -> Result<&T>;
}

impl RecordBatchExt for RecordBatch {
    fn get_column<T: 'static>(&self, col_name: &str) -> Result<&T> {
        let col = self.column_by_name(col_name).ok_or(anyhow!(format!("cannot find column {}", &col_name)))?;
        let arr = col.as_any().downcast_ref().ok_or(anyhow!(format!("cannot downcast column {}", &col_name)));
        arr
    }
}

pub trait ArrayRefExt {
    fn as_array<T: 'static>(&self) -> Result<&T>;
}

impl ArrayRefExt for ArrayRef {
    fn as_array<T: 'static>(&self) -> Result<&T> {
        self.as_any().downcast_ref().ok_or(anyhow!(format!("cannot downcast array")))
    }
}

pub async fn get_cached_abi_item_provider(
    cache: Arc<Cache<String, Arc<dyn AbiItemProvider + Send + Sync>>>,
    key: &[u8]
) -> Result<Arc<dyn AbiItemProvider + Send + Sync>> {
    let key = str::from_utf8(key)?;

    cache.get_or_insert_async(
        key,
        async { AbiItemProviderFactory::create(key).await }
    ).await
}

pub fn get_cached_func_sync(
    cache: Arc<Cache<String, Function>>,
    key: &[u8]
) -> Result<Function> {
    let key = str::from_utf8(key)?;

    cache.get_or_insert_with(
        key,
        || { Function::parse(key) }
    )
}

