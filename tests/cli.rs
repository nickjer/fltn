use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn fltn() -> assert_cmd::Command {
    cargo_bin_cmd!("fltn")
}

fn temp_file_with_content(content: &str, suffix: &str) -> NamedTempFile {
    let temp_file = NamedTempFile::with_suffix(suffix).unwrap();
    let mut file = temp_file.reopen().unwrap();
    write!(file, "{}", content).unwrap();
    temp_file
}

// =============================================================================
// Basic functionality tests
// =============================================================================

#[test]
fn json_from_stdin() {
    fltn()
        .write_stdin(r#"{"name": "test"}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.name = \"test\";\n",
        );
}

#[test]
fn json_from_file() {
    let temp = temp_file_with_content(r#"{"key": "value"}"#, ".json");

    fltn()
        .arg(temp.path())
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.key = \"value\";\n",
        );
}

#[test]
fn json_array() {
    fltn()
        .write_stdin("[1, 2, 3]")
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = 1;\n\
             json[1] = 2;\n\
             json[2] = 3;\n",
        );
}

#[test]
fn json_nested() {
    fltn()
        .write_stdin(r#"{"outer": {"inner": 42}}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.outer = {};\n\
             json.outer.inner = 42;\n",
        );
}

#[test]
fn json_all_types() {
    fltn()
        .write_stdin(r#"{"s": "hello", "n": 42, "b": true, "null": null}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.s = \"hello\";\n\
             json.n = 42;\n\
             json.b = true;\n\
             json.null = null;\n",
        );
}

#[test]
fn json_null_value() {
    fltn()
        .write_stdin("null")
        .arg("--color=never")
        .assert()
        .success()
        .stdout("json = null;\n");
}

#[test]
fn json_bool_true() {
    fltn()
        .write_stdin("true")
        .arg("--color=never")
        .assert()
        .success()
        .stdout("json = true;\n");
}

#[test]
fn json_number() {
    fltn()
        .write_stdin("42")
        .arg("--color=never")
        .assert()
        .success()
        .stdout("json = 42;\n");
}

#[test]
fn json_string() {
    fltn()
        .write_stdin(r#""hello""#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout("json = \"hello\";\n");
}

// =============================================================================
// Format detection and explicit format tests
// =============================================================================

#[test]
fn csv_from_file() {
    let temp = temp_file_with_content("name,value\nfoo,bar\n", ".csv");

    fltn()
        .arg(temp.path())
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = {};\n\
             json[0].name = \"foo\";\n\
             json[0].value = \"bar\";\n",
        );
}

#[test]
fn csv_with_explicit_format() {
    fltn()
        .write_stdin("name,value\nfoo,bar\n")
        .args(["--format", "csv", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = {};\n\
             json[0].name = \"foo\";\n\
             json[0].value = \"bar\";\n",
        );
}

#[test]
fn csv_multiple_rows() {
    fltn()
        .write_stdin("id,name\n1,alice\n2,bob\n")
        .args(["--format", "csv", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = {};\n\
             json[0].id = \"1\";\n\
             json[0].name = \"alice\";\n\
             json[1] = {};\n\
             json[1].id = \"2\";\n\
             json[1].name = \"bob\";\n",
        );
}

#[test]
fn toml_from_file() {
    let temp = temp_file_with_content("name = \"test\"\nvalue = 42\n", ".toml");

    fltn()
        .arg(temp.path())
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.name = \"test\";\n\
             json.value = 42;\n",
        );
}

#[test]
fn toml_with_explicit_format() {
    fltn()
        .write_stdin("[server]\nhost = \"localhost\"\nport = 8080\n")
        .args(["--format", "toml", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.server = {};\n\
             json.server.host = \"localhost\";\n\
             json.server.port = 8080;\n",
        );
}

#[test]
fn yaml_from_file() {
    let temp = temp_file_with_content("name: test\nvalue: 42\n", ".yaml");

    fltn()
        .arg(temp.path())
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.name = \"test\";\n\
             json.value = 42;\n",
        );
}

#[test]
fn yaml_with_explicit_format() {
    fltn()
        .write_stdin("items:\n  - one\n  - two\n")
        .args(["--format", "yaml", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.items = [];\n\
             json.items[0] = \"one\";\n\
             json.items[1] = \"two\";\n",
        );
}

#[test]
fn yml_extension_detected_as_yaml() {
    let temp = temp_file_with_content("key: value\n", ".yml");

    fltn()
        .arg(temp.path())
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.key = \"value\";\n",
        );
}

// =============================================================================
// JSONPath filtering tests
// =============================================================================

#[test]
fn jsonpath_filter_single_value() {
    fltn()
        .write_stdin(r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#)
        .args(["--path", "$.users[0].name", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = \"alice\";\n",
        );
}

#[test]
fn jsonpath_filter_all_array_elements() {
    fltn()
        .write_stdin(r#"{"items": [1, 2, 3]}"#)
        .args(["--path", "$.items[*]", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = 1;\n\
             json[1] = 2;\n\
             json[2] = 3;\n",
        );
}

#[test]
fn jsonpath_filter_nested_object() {
    fltn()
        .write_stdin(r#"{"a": {"b": {"c": 42}}}"#)
        .args(["--path", "$.a.b", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = {};\n\
             json[0].c = 42;\n",
        );
}

// =============================================================================
// Sort option tests
// =============================================================================

#[test]
fn sort_option_orders_keys_alphabetically() {
    fltn()
        .write_stdin(r#"{"z": 1, "a": 2, "m": 3}"#)
        .args(["--sort", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.a = 2;\n\
             json.m = 3;\n\
             json.z = 1;\n",
        );
}

#[test]
fn without_sort_preserves_order() {
    fltn()
        .write_stdin(r#"{"z": 1, "a": 2, "m": 3}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.z = 1;\n\
             json.a = 2;\n\
             json.m = 3;\n",
        );
}

#[test]
fn sort_nested_objects() {
    fltn()
        .write_stdin(r#"{"b": {"y": 1, "x": 2}, "a": 3}"#)
        .args(["--sort", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.a = 3;\n\
             json.b = {};\n\
             json.b.x = 2;\n\
             json.b.y = 1;\n",
        );
}

// =============================================================================
// Color option tests
// =============================================================================

#[test]
fn color_never_produces_no_ansi() {
    let output = fltn()
        .write_stdin(r#"{"key": "value"}"#)
        .arg("--color=never")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("\x1b["));
}

#[test]
fn color_always_produces_ansi() {
    let output = fltn()
        .write_stdin(r#"{"key": "value"}"#)
        .arg("--color=always")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\x1b["));
}

// =============================================================================
// Field name notation tests
// =============================================================================

#[test]
fn valid_field_uses_dot_notation() {
    fltn()
        .write_stdin(r#"{"validName": 1}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.validName = 1;\n",
        );
}

#[test]
fn field_with_hyphen_uses_bracket_notation() {
    fltn()
        .write_stdin(r#"{"field-name": 1}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json[\"field-name\"] = 1;\n",
        );
}

#[test]
fn field_with_space_uses_bracket_notation() {
    fltn()
        .write_stdin(r#"{"field name": 1}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json[\"field name\"] = 1;\n",
        );
}

#[test]
fn field_starting_with_number_uses_bracket_notation() {
    fltn()
        .write_stdin(r#"{"123abc": 1}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json[\"123abc\"] = 1;\n",
        );
}

#[test]
fn empty_field_name_uses_bracket_notation() {
    fltn()
        .write_stdin(r#"{"": 1}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json[\"\"] = 1;\n",
        );
}

#[test]
fn underscore_field_uses_dot_notation() {
    fltn()
        .write_stdin(r#"{"_private": 1}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json._private = 1;\n",
        );
}

#[test]
fn dollar_field_uses_dot_notation() {
    fltn()
        .write_stdin(r#"{"$special": 1}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.$special = 1;\n",
        );
}

// =============================================================================
// Error handling tests
// =============================================================================

#[test]
fn invalid_json_returns_error() {
    fltn()
        .write_stdin(r#"{"invalid": }"#)
        .arg("--color=never")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to deserialize"));
}

#[test]
fn nonexistent_file_returns_error() {
    fltn()
        .arg("/nonexistent/path/to/file.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read"));
}

#[test]
fn invalid_format_option_returns_error() {
    fltn()
        .write_stdin("{}")
        .args(["--format", "invalid"])
        .assert()
        .failure();
}

#[test]
fn invalid_jsonpath_returns_error() {
    fltn()
        .write_stdin(r#"{"key": "value"}"#)
        .args(["--path", "$[invalid", "--color=never"])
        .assert()
        .failure();
}

// =============================================================================
// Help and version tests
// =============================================================================

#[test]
fn help_flag_shows_usage() {
    fltn()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--path"))
        .stdout(predicate::str::contains("--color"))
        .stdout(predicate::str::contains("--sort"));
}

#[test]
fn version_flag_shows_version() {
    fltn()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("fltn"));
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn empty_json_object() {
    fltn()
        .write_stdin("{}")
        .arg("--color=never")
        .assert()
        .success()
        .stdout("json = {};\n");
}

#[test]
fn empty_json_array() {
    fltn()
        .write_stdin("[]")
        .arg("--color=never")
        .assert()
        .success()
        .stdout("json = [];\n");
}

#[test]
fn deeply_nested_structure() {
    fltn()
        .write_stdin(r#"{"a": {"b": {"c": {"d": {"e": 1}}}}}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.a = {};\n\
             json.a.b = {};\n\
             json.a.b.c = {};\n\
             json.a.b.c.d = {};\n\
             json.a.b.c.d.e = 1;\n",
        );
}

#[test]
fn unicode_content() {
    fltn()
        .write_stdin(r#"{"greeting": "こんにちは"}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.greeting = \"こんにちは\";\n",
        );
}

#[test]
fn string_with_escaped_quotes() {
    fltn()
        .write_stdin(r#"{"text": "say \"hello\""}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.text = \"say \\\"hello\\\"\";\n",
        );
}

#[test]
fn negative_number() {
    fltn()
        .write_stdin(r#"{"value": -42}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.value = -42;\n",
        );
}

#[test]
fn float_number() {
    fltn()
        .write_stdin(r#"{"value": 1.5}"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = {};\n\
             json.value = 1.5;\n",
        );
}

#[test]
fn csv_empty_with_headers_only() {
    fltn()
        .write_stdin("name,value\n")
        .args(["--format", "csv", "--color=never"])
        .assert()
        .success()
        .stdout("json = [];\n");
}

#[test]
fn csv_preserves_header_order() {
    fltn()
        .write_stdin("z,a,m\n1,2,3\n")
        .args(["--format", "csv", "--color=never"])
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = {};\n\
             json[0].z = \"1\";\n\
             json[0].a = \"2\";\n\
             json[0].m = \"3\";\n",
        );
}

#[test]
fn mixed_array_types() {
    fltn()
        .write_stdin(r#"[1, "two", true, null]"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = 1;\n\
             json[1] = \"two\";\n\
             json[2] = true;\n\
             json[3] = null;\n",
        );
}

#[test]
fn array_of_objects() {
    fltn()
        .write_stdin(r#"[{"a": 1}, {"b": 2}]"#)
        .arg("--color=never")
        .assert()
        .success()
        .stdout(
            "json = [];\n\
             json[0] = {};\n\
             json[0].a = 1;\n\
             json[1] = {};\n\
             json[1].b = 2;\n",
        );
}
