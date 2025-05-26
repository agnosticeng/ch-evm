use std::io::{stdin,stdout,Read,Write};
use std::fs::File;
use anyhow::{Result,bail,anyhow};
use arrow::array::{ArrayRef,RecordBatch};

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