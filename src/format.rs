use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Format {
    Csv,
    Json,
    Toml,
    Yaml,
}

impl Format {
    pub fn guess_from_path(path: &std::path::Path) -> Option<Self> {
        let mime = mime_guess::from_path(path).first_raw()?;
        match mime {
            "application/json" => Some(Self::Json),
            "text/csv" => Some(Self::Csv),
            "text/x-toml" => Some(Self::Toml),
            "text/x-yaml" => Some(Self::Yaml),
            _ => None,
        }
    }

    pub fn deserialize(&self, contents: &str) -> Result<Value> {
        match self {
            Format::Csv => deserialize_csv(contents),
            Format::Json => deserialize_json(contents),
            Format::Toml => deserialize_toml(contents),
            Format::Yaml => deserialize_yaml(contents),
        }
        .context("Failed to deserialize")
    }
}

fn trim_quotes(input: String) -> String {
    input
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .map(|s| s.to_owned())
        .unwrap_or(input)
}

fn deserialize_csv(contents: &str) -> Result<Value> {
    let reader = csv::ReaderBuilder::new().from_reader(contents.as_bytes());
    let rows: Result<Vec<Value>> = reader
        .into_deserialize()
        .map(|result| {
            result
                .map(|record: indexmap::IndexMap<String, String>| {
                    record
                        .into_iter()
                        .map(|(key, val)| (trim_quotes(key), Value::String(trim_quotes(val))))
                        .collect::<serde_json::Map<_, _>>()
                        .into()
                })
                .context("In CSV deserializer")
        })
        .collect();
    rows.map(Value::Array)
}

fn deserialize_json(contents: &str) -> Result<Value> {
    let mut deserializer = serde_json::Deserializer::from_str(contents);
    deserializer.disable_recursion_limit();
    let deserializer = serde_stacker::Deserializer::new(&mut deserializer);
    Value::deserialize(deserializer).context("In JSON deserializer")
}

fn deserialize_toml(contents: &str) -> Result<Value> {
    toml::from_str(contents).context("In TOML deserializer")
}

fn deserialize_yaml(contents: &str) -> Result<Value> {
    serde_yaml::from_str(contents).context("In YAML deserializer")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;

    // Format guessing tests
    #[test]
    fn guess_json_from_extension() {
        assert!(matches!(
            Format::guess_from_path(Path::new("data.json")),
            Some(Format::Json)
        ));
    }

    #[test]
    fn guess_csv_from_extension() {
        assert!(matches!(
            Format::guess_from_path(Path::new("data.csv")),
            Some(Format::Csv)
        ));
    }

    #[test]
    fn guess_toml_from_extension() {
        assert!(matches!(
            Format::guess_from_path(Path::new("config.toml")),
            Some(Format::Toml)
        ));
    }

    #[test]
    fn guess_yaml_from_extension() {
        assert!(matches!(
            Format::guess_from_path(Path::new("config.yaml")),
            Some(Format::Yaml)
        ));
    }

    #[test]
    fn guess_yml_from_extension() {
        assert!(matches!(
            Format::guess_from_path(Path::new("config.yml")),
            Some(Format::Yaml)
        ));
    }

    #[test]
    fn guess_returns_none_for_unknown_extension() {
        assert!(Format::guess_from_path(Path::new("data.xyz")).is_none());
    }

    #[test]
    fn guess_returns_none_for_no_extension() {
        assert!(Format::guess_from_path(Path::new("README")).is_none());
    }

    #[test]
    fn guess_works_with_full_path() {
        assert!(matches!(
            Format::guess_from_path(Path::new("/home/user/data/file.json")),
            Some(Format::Json)
        ));
    }

    #[test]
    fn guess_is_case_insensitive_for_json() {
        assert!(matches!(
            Format::guess_from_path(Path::new("data.JSON")),
            Some(Format::Json)
        ));
    }

    #[test]
    fn guess_handles_multiple_dots() {
        assert!(matches!(
            Format::guess_from_path(Path::new("data.backup.json")),
            Some(Format::Json)
        ));
    }

    // CSV deserialization tests
    #[test]
    fn deserialize_csv_numbers_as_strings() {
        let csv = "zip,state\n\"02345\",OH\n13003,AL\n";
        let json = Format::Csv.deserialize(csv).unwrap();
        assert_eq!(
            json,
            json!([
                { "zip": "02345", "state": "OH" },
                { "zip": "13003", "state": "AL" }
            ])
        );
    }

    #[test]
    fn deserialize_csv_empty() {
        let json = Format::Csv.deserialize("name,value\n").unwrap();
        assert_eq!(json, json!([]));
    }

    #[test]
    fn deserialize_csv_preserves_header_order() {
        let json = Format::Csv.deserialize("z,a,m\n1,2,3\n").unwrap();
        let keys: Vec<&String> = json[0].as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    }

    #[test]
    fn deserialize_csv_with_commas_in_values() {
        let json = Format::Csv
            .deserialize("name,address\nJohn,\"123 Main St, Apt 4\"\n")
            .unwrap();
        assert_eq!(json[0]["address"], "123 Main St, Apt 4");
    }

    // JSON deserialization tests
    #[test]
    fn deserialize_json_object() {
        let json = Format::Json
            .deserialize(r#"{"name": "test", "value": 42}"#)
            .unwrap();
        assert_eq!(json, json!({"name": "test", "value": 42}));
    }

    #[test]
    fn deserialize_json_array() {
        let json = Format::Json.deserialize("[1, 2, 3]").unwrap();
        assert_eq!(json, json!([1, 2, 3]));
    }

    #[test]
    fn deserialize_json_nested() {
        let json = Format::Json
            .deserialize(r#"{"outer": {"inner": {"deep": true}}}"#)
            .unwrap();
        assert_eq!(json["outer"]["inner"]["deep"], true);
    }

    #[test]
    fn deserialize_json_with_all_types() {
        let json = Format::Json
            .deserialize(
                r#"{"string": "hello", "number": 42, "float": 1.5, "bool": true, "null": null, "array": [1,2]}"#,
            )
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
        assert!(Format::Json.deserialize(r#"{"invalid": }"#).is_err());
    }

    // TOML deserialization tests
    #[test]
    fn deserialize_toml_basic() {
        let json = Format::Toml
            .deserialize("name = \"test\"\nvalue = 42\n")
            .unwrap();
        assert_eq!(json, json!({"name": "test", "value": 42}));
    }

    #[test]
    fn deserialize_toml_nested_table() {
        let json = Format::Toml
            .deserialize("[server]\nhost = \"localhost\"\nport = 8080\n")
            .unwrap();
        assert_eq!(json["server"]["host"], "localhost");
        assert_eq!(json["server"]["port"], 8080);
    }

    #[test]
    fn deserialize_toml_array() {
        let json = Format::Toml.deserialize("values = [1, 2, 3]\n").unwrap();
        assert_eq!(json["values"], json!([1, 2, 3]));
    }

    #[test]
    fn deserialize_toml_invalid() {
        assert!(Format::Toml.deserialize("invalid = [unclosed").is_err());
    }

    // YAML deserialization tests
    #[test]
    fn deserialize_yaml_basic() {
        let json = Format::Yaml.deserialize("name: test\nvalue: 42\n").unwrap();
        assert_eq!(json, json!({"name": "test", "value": 42}));
    }

    #[test]
    fn deserialize_yaml_nested() {
        let json = Format::Yaml
            .deserialize("server:\n  host: localhost\n  port: 8080\n")
            .unwrap();
        assert_eq!(json["server"]["host"], "localhost");
        assert_eq!(json["server"]["port"], 8080);
    }

    #[test]
    fn deserialize_yaml_array() {
        let json = Format::Yaml
            .deserialize("values:\n  - 1\n  - 2\n  - 3\n")
            .unwrap();
        assert_eq!(json["values"], json!([1, 2, 3]));
    }

    #[test]
    fn deserialize_yaml_inline_array() {
        let json = Format::Yaml.deserialize("values: [1, 2, 3]\n").unwrap();
        assert_eq!(json["values"], json!([1, 2, 3]));
    }

    #[test]
    fn deserialize_yaml_invalid() {
        assert!(Format::Yaml
            .deserialize("invalid:\n  - missing\n value")
            .is_err());
    }
}
