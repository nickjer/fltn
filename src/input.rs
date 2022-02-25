use crate::format::Format;

use anyhow::{Context, Error, Result};
use std::io::Read;

#[derive(Debug)]
pub enum Input {
    Stdin(String),
    File(std::path::PathBuf, String),
}

impl Input {
    pub fn guess_format(&self) -> Option<Format> {
        match self {
            Input::File(path, _file) => Format::guess_from_path(path),
            _ => None,
        }
    }

    pub fn contents(&self) -> &str {
        match self {
            Input::Stdin(contents) => contents,
            Input::File(_path, contents) => contents,
        }
    }
}

impl std::convert::TryFrom<Option<std::path::PathBuf>> for Input {
    type Error = Error;

    fn try_from(path: Option<std::path::PathBuf>) -> Result<Self> {
        let mut buffer = String::new();
        match path {
            None => {
                std::io::stdin()
                    .read_to_string(&mut buffer)
                    .context("Failed to read from stdin")?;
                Ok(Input::Stdin(buffer))
            }
            Some(path) => {
                buffer = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {path:?}"))?;
                Ok(Input::File(path, buffer))
            }
        }
    }
}
