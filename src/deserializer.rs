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
        let reader = csv::ReaderBuilder::new().from_reader(contents.as_bytes());
        let rows_result: Result<Vec<_>> = reader
            .into_deserialize()
            .map(|result| {
                result
                    .map(|record: indexmap::IndexMap<String, String>| {
                        let mut json_map = serde_json::Map::new();
                        record.into_iter().for_each(|(key, val)| {
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

    // CSV tests
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

    #[test]
    fn deserialize_csv_empty() {
        let csv = String::from("name,value\n");
        let json = Deserializer::new(Input::Stdin(csv), Format::Csv)
            .deserialize()
            .unwrap();

        assert_eq!(json, json!([]));
    }

    #[test]
    fn deserialize_csv_preserves_header_order() {
        let csv = String::from("z,a,m\n1,2,3\n");
        let json = Deserializer::new(Input::Stdin(csv), Format::Csv)
            .deserialize()
            .unwrap();

        let keys: Vec<&String> = json[0].as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    }

    #[test]
    fn deserialize_csv_with_commas_in_values() {
        let csv = String::from("name,address\nJohn,\"123 Main St, Apt 4\"\n");
        let json = Deserializer::new(Input::Stdin(csv), Format::Csv)
            .deserialize()
            .unwrap();

        assert_eq!(json[0]["address"], "123 Main St, Apt 4");
    }

    // JSON tests
    #[test]
    fn deserialize_json_object() {
        let input = String::from(r#"{"name": "test", "value": 42}"#);
        let json = Deserializer::new(Input::Stdin(input), Format::Json)
            .deserialize()
            .unwrap();

        assert_eq!(json, json!({"name": "test", "value": 42}));
    }

    #[test]
    fn deserialize_json_array() {
        let input = String::from(r#"[1, 2, 3]"#);
        let json = Deserializer::new(Input::Stdin(input), Format::Json)
            .deserialize()
            .unwrap();

        assert_eq!(json, json!([1, 2, 3]));
    }

    #[test]
    fn deserialize_json_nested() {
        let input = String::from(r#"{"outer": {"inner": {"deep": true}}}"#);
        let json = Deserializer::new(Input::Stdin(input), Format::Json)
            .deserialize()
            .unwrap();

        assert_eq!(json["outer"]["inner"]["deep"], true);
    }

    #[test]
    fn deserialize_json_with_all_types() {
        let input = String::from(
            r#"{"string": "hello", "number": 42, "float": 1.5, "bool": true, "null": null, "array": [1,2]}"#,
        );
        let json = Deserializer::new(Input::Stdin(input), Format::Json)
            .deserialize()
            .unwrap();

        assert_eq!(json["string"], "hello");
        assert_eq!(json["number"], 42);
        assert_eq!(json["float"], 1.5);
        assert_eq!(json["bool"], true);
        assert!(json["null"].is_null());
        assert_eq!(json["array"], json!([1, 2]));
    }

    #[test]
    fn deserialize_json_invalid() {
        let input = String::from(r#"{"invalid": }"#);
        let result = Deserializer::new(Input::Stdin(input), Format::Json).deserialize();

        assert!(result.is_err());
    }

    // TOML tests
    #[test]
    fn deserialize_toml_basic() {
        let input = String::from("name = \"test\"\nvalue = 42\n");
        let json = Deserializer::new(Input::Stdin(input), Format::Toml)
            .deserialize()
            .unwrap();

        assert_eq!(json, json!({"name": "test", "value": 42}));
    }

    #[test]
    fn deserialize_toml_nested_table() {
        let input = String::from("[server]\nhost = \"localhost\"\nport = 8080\n");
        let json = Deserializer::new(Input::Stdin(input), Format::Toml)
            .deserialize()
            .unwrap();

        assert_eq!(json["server"]["host"], "localhost");
        assert_eq!(json["server"]["port"], 8080);
    }

    #[test]
    fn deserialize_toml_array() {
        let input = String::from("values = [1, 2, 3]\n");
        let json = Deserializer::new(Input::Stdin(input), Format::Toml)
            .deserialize()
            .unwrap();

        assert_eq!(json["values"], json!([1, 2, 3]));
    }

    #[test]
    fn deserialize_toml_invalid() {
        let input = String::from("invalid = [unclosed");
        let result = Deserializer::new(Input::Stdin(input), Format::Toml).deserialize();

        assert!(result.is_err());
    }

    // YAML tests
    #[test]
    fn deserialize_yaml_basic() {
        let input = String::from("name: test\nvalue: 42\n");
        let json = Deserializer::new(Input::Stdin(input), Format::Yaml)
            .deserialize()
            .unwrap();

        assert_eq!(json, json!({"name": "test", "value": 42}));
    }

    #[test]
    fn deserialize_yaml_nested() {
        let input = String::from("server:\n  host: localhost\n  port: 8080\n");
        let json = Deserializer::new(Input::Stdin(input), Format::Yaml)
            .deserialize()
            .unwrap();

        assert_eq!(json["server"]["host"], "localhost");
        assert_eq!(json["server"]["port"], 8080);
    }

    #[test]
    fn deserialize_yaml_array() {
        let input = String::from("values:\n  - 1\n  - 2\n  - 3\n");
        let json = Deserializer::new(Input::Stdin(input), Format::Yaml)
            .deserialize()
            .unwrap();

        assert_eq!(json["values"], json!([1, 2, 3]));
    }

    #[test]
    fn deserialize_yaml_inline_array() {
        let input = String::from("values: [1, 2, 3]\n");
        let json = Deserializer::new(Input::Stdin(input), Format::Yaml)
            .deserialize()
            .unwrap();

        assert_eq!(json["values"], json!([1, 2, 3]));
    }

    #[test]
    fn deserialize_yaml_invalid() {
        let input = String::from("invalid:\n  - missing\n value");
        let result = Deserializer::new(Input::Stdin(input), Format::Yaml).deserialize();

        assert!(result.is_err());
    }
}
