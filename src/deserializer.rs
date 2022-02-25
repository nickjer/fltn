use crate::format::Format;
use crate::input::Input;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug)]
pub struct Deserializer {
    input: Input,
    format: Format,
}

impl Deserializer {
    pub fn new(input: Input, format: Format) -> Self {
        Self { input, format }
    }

    pub fn deserialize(&self) -> Result<Value> {
        match self.format {
            Format::Csv => self.deserialize_csv(),
            Format::Json => self.deserialize_json(),
            Format::Toml => self.deserialize_toml(),
            Format::Yaml => self.deserialize_yaml(),
        }
        .context("Failed to deserialize")
    }

    fn deserialize_csv(&self) -> Result<Value> {
        let contents = self.input.contents();
        let reader = csv::Reader::from_reader(contents.as_bytes());
        let rows_result: Result<Vec<_>> = reader
            .into_deserialize()
            .map(|result| {
                result
                    .map(serde_json::map::Map::from)
                    .map(Value::Object)
                    .context("In CSV deserializer")
            })
            .collect();
        rows_result.map(Value::Array)
    }

    fn deserialize_json(&self) -> Result<Value> {
        let contents = self.input.contents();
        let mut deserializer = serde_json::Deserializer::from_str(contents);
        deserializer.disable_recursion_limit();
        let deserializer = serde_stacker::Deserializer::new(&mut deserializer);
        Value::deserialize(deserializer).context("In JSON deserializer")
    }

    fn deserialize_toml(&self) -> Result<Value> {
        let contents = self.input.contents();
        toml::from_str(contents).context("In TOML deserializer")
    }

    fn deserialize_yaml(&self) -> Result<Value> {
        let contents = self.input.contents();
        serde_yaml::from_str(contents).context("In YAML deserializer")
    }
}
