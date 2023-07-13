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
        fn trim_quotes(input: String) -> String {
            let len = input.len();
            if len > 1 && input.starts_with('"') && input.ends_with('"') {
                input[1..len - 1].to_owned()
            } else {
                input
            }
        }

        let contents = self.input.contents();
        let reader = csv::ReaderBuilder::new()
            .from_reader(contents.as_bytes());
        let rows_result: Result<Vec<_>> = reader
            .into_deserialize()
            .map(|result| {
                result
                    .map(|mut record: std::collections::HashMap<String, String>| {
                        let mut json_map = serde_json::Map::new();
                        record.drain().for_each(|(key, val)| {
                            let key = trim_quotes(key);
                            let val = serde_json::Value::String(trim_quotes(val));
                            json_map.insert(key, val);
                        });
                        serde_json::Value::Object(json_map)
                    })
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::format::Format;
    use crate::input::Input;

    #[test]
    fn deserialize_csv_numbers_as_strings() {
        let csv = String::from(
            "zip,state\n\
            \"02345\",OH\n\
            13003,AL\n",
        );
        let json = Deserializer::new(Input::Stdin(csv), Format::Csv)
            .deserialize()
            .unwrap();

        let expected_json = json!([
            { "zip": "02345", "state": "OH" },
            { "zip": "13003", "state": "AL" }
        ]);

        assert_eq!(json, expected_json);
    }
}
