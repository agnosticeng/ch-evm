use std::io::{stdin,stdout,Read,Write};
use std::fs::File;
use anyhow::{Result,bail};

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