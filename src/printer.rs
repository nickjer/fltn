use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use std::io::Write;
use unicode_categories::UnicodeCategories;

#[derive(Debug)]
pub struct Printer {
    sort: bool,
}

impl Printer {
    pub fn new(sort: bool) -> Self {
        Self { sort }
    }

    pub fn print(&self, writer: &mut impl Write, value: &Value) -> Result<()> {
        let l_brace = self.brace_color("[");
        let r_brace = self.brace_color("]");

        let mut stack: Vec<(String, &Value)> = vec![(self.field_color("json").to_string(), value)];

        loop {
            let (prefix, value) = match stack.pop() {
                Some(args) => args,
                None => return Ok(()),
            };
            match value {
                Value::Null => writeln!(writer, "{prefix} = {};", self.null_color("null"))?,
                Value::Bool(value) => writeln!(writer, "{prefix} = {};", self.bool_color(value))?,
                Value::Number(value) => {
                    writeln!(writer, "{prefix} = {};", self.number_color(value))?
                }
                Value::String(value) => writeln!(
                    writer,
                    "{prefix} = {};",
                    self.string_color(serde_json::to_string(value).unwrap())
                )?,
                Value::Array(list) => {
                    writeln!(writer, "{prefix} = {l_brace}{r_brace};")?;
                    list.iter().enumerate().rev().for_each(|(index, value)| {
                        let new_prefix =
                            format!("{prefix}{l_brace}{}{r_brace}", self.number_color(index));
                        stack.push((new_prefix, value))
                    });
                }
                Value::Object(object) => {
                    writeln!(writer, "{prefix} = {};", self.brace_color("{}"))?;
                    let object_iter = move |sort: bool| -> Box<dyn DoubleEndedIterator<Item = _>> {
                        if sort {
                            let mut pairs: Vec<_> = object.into_iter().collect();
                            pairs.sort_by(|pair_1, pair_2| pair_1.0.cmp(pair_2.0));
                            Box::new(pairs.into_iter())
                        } else {
                            Box::new(object.into_iter())
                        }
                    };
                    object_iter(self.sort).rev().for_each(|(key, value)| {
                        let new_prefix = if self.valid_field_name(key) {
                            format!("{prefix}.{}", self.field_color(key))
                        } else {
                            format!(
                                "{prefix}{l_brace}{}{r_brace}",
                                self.string_color(serde_json::to_string(key).unwrap()),
                            )
                        };
                        stack.push((new_prefix, value));
                    })
                }
            };
        }
    }

    fn bool_color<T: ToString>(&self, value: T) -> colored::ColoredString {
        value.to_string().cyan()
    }

    fn brace_color<T: ToString>(&self, value: T) -> colored::ColoredString {
        value.to_string().magenta()
    }

    fn field_color<T: ToString>(&self, value: T) -> colored::ColoredString {
        value.to_string().blue().bold()
    }

    fn null_color<T: ToString>(&self, value: T) -> colored::ColoredString {
        value.to_string().cyan()
    }

    fn number_color<T: ToString>(&self, value: T) -> colored::ColoredString {
        value.to_string().red()
    }

    fn string_color<T: ToString>(&self, value: T) -> colored::ColoredString {
        value.to_string().yellow()
    }

    fn valid_field_name(&self, field_name: &str) -> bool {
        if field_name.is_empty() {
            return false;
        }

        field_name.chars().enumerate().all(|(idx, letter)| {
            if idx == 0 {
                self.valid_first_field_letter(letter)
            } else {
                self.valid_following_field_letter(letter)
            }
        })
    }

    fn valid_first_field_letter(&self, letter: char) -> bool {
        letter.is_letter_lowercase()
            || letter.is_letter_modifier()
            || letter.is_letter_other()
            || letter.is_letter_uppercase()
            || letter.is_number_letter()
            || letter == '$'
            || letter == '_'
    }

    fn valid_following_field_letter(&self, letter: char) -> bool {
        self.valid_first_field_letter(letter)
            || letter.is_mark_nonspacing()
            || letter.is_mark_spacing_combining()
            || letter.is_number_decimal_digit()
            || letter.is_punctuation_connector()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn print_to_string(value: &Value, sort: bool) -> String {
        // Disable colors for predictable test output
        colored::control::set_override(false);
        let printer = Printer::new(sort);
        let mut buffer = Vec::new();
        printer.print(&mut buffer, value).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    // Basic value tests
    #[test]
    fn print_null() {
        let output = print_to_string(&json!(null), false);
        assert_eq!(output, "json = null;\n");
    }

    #[test]
    fn print_bool_true() {
        let output = print_to_string(&json!(true), false);
        assert_eq!(output, "json = true;\n");
    }

    #[test]
    fn print_bool_false() {
        let output = print_to_string(&json!(false), false);
        assert_eq!(output, "json = false;\n");
    }

    #[test]
    fn print_integer() {
        let output = print_to_string(&json!(42), false);
        assert_eq!(output, "json = 42;\n");
    }

    #[test]
    fn print_negative_integer() {
        let output = print_to_string(&json!(-42), false);
        assert_eq!(output, "json = -42;\n");
    }

    #[test]
    fn print_float() {
        let output = print_to_string(&json!(1.5), false);
        assert_eq!(output, "json = 1.5;\n");
    }

    #[test]
    fn print_string() {
        let output = print_to_string(&json!("hello"), false);
        assert_eq!(output, "json = \"hello\";\n");
    }

    #[test]
    fn print_string_with_quotes() {
        let output = print_to_string(&json!("say \"hello\""), false);
        assert_eq!(output, "json = \"say \\\"hello\\\"\";\n");
    }

    #[test]
    fn print_empty_string() {
        let output = print_to_string(&json!(""), false);
        assert_eq!(output, "json = \"\";\n");
    }

    // Array tests
    #[test]
    fn print_empty_array() {
        let output = print_to_string(&json!([]), false);
        assert_eq!(output, "json = [];\n");
    }

    #[test]
    fn print_array_of_numbers() {
        let output = print_to_string(&json!([1, 2, 3]), false);
        let expected = "json = [];\njson[0] = 1;\njson[1] = 2;\njson[2] = 3;\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn print_nested_array() {
        let output = print_to_string(&json!([[1, 2]]), false);
        let expected = "json = [];\njson[0] = [];\njson[0][0] = 1;\njson[0][1] = 2;\n";
        assert_eq!(output, expected);
    }

    // Object tests
    #[test]
    fn print_empty_object() {
        let output = print_to_string(&json!({}), false);
        assert_eq!(output, "json = {};\n");
    }

    #[test]
    fn print_simple_object() {
        let output = print_to_string(&json!({"name": "test"}), false);
        let expected = "json = {};\njson.name = \"test\";\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn print_nested_object() {
        let output = print_to_string(&json!({"outer": {"inner": 42}}), false);
        let expected = "json = {};\njson.outer = {};\njson.outer.inner = 42;\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn print_object_with_array() {
        let output = print_to_string(&json!({"items": [1, 2]}), false);
        let expected = "json = {};\njson.items = [];\njson.items[0] = 1;\njson.items[1] = 2;\n";
        assert_eq!(output, expected);
    }

    // Field name tests (dot notation vs bracket notation)
    #[test]
    fn print_valid_field_name() {
        let output = print_to_string(&json!({"validName": 1}), false);
        assert!(output.contains("json.validName"));
    }

    #[test]
    fn print_field_with_hyphen_uses_bracket() {
        let output = print_to_string(&json!({"field-name": 1}), false);
        assert!(output.contains("json[\"field-name\"]"));
    }

    #[test]
    fn print_field_with_space_uses_bracket() {
        let output = print_to_string(&json!({"field name": 1}), false);
        assert!(output.contains("json[\"field name\"]"));
    }

    #[test]
    fn print_field_starting_with_number_uses_bracket() {
        let output = print_to_string(&json!({"123abc": 1}), false);
        assert!(output.contains("json[\"123abc\"]"));
    }

    #[test]
    fn print_empty_field_name_uses_bracket() {
        let output = print_to_string(&json!({"": 1}), false);
        assert!(output.contains("json[\"\"]"));
    }

    #[test]
    fn print_field_with_underscore() {
        let output = print_to_string(&json!({"_private": 1}), false);
        assert!(output.contains("json._private"));
    }

    #[test]
    fn print_field_with_dollar() {
        let output = print_to_string(&json!({"$special": 1}), false);
        assert!(output.contains("json.$special"));
    }

    // Sort tests
    #[test]
    fn print_object_unsorted() {
        // With preserve_order feature, keys should maintain insertion order
        let output = print_to_string(&json!({"z": 1, "a": 2, "m": 3}), false);
        let lines: Vec<&str> = output.lines().collect();
        // First line is "json = {};" so check the key order in subsequent lines
        assert!(lines[1].contains(".z"));
        assert!(lines[2].contains(".a"));
        assert!(lines[3].contains(".m"));
    }

    #[test]
    fn print_object_sorted() {
        let output = print_to_string(&json!({"z": 1, "a": 2, "m": 3}), true);
        let lines: Vec<&str> = output.lines().collect();
        // When sorted, should be alphabetical
        assert!(lines[1].contains(".a"));
        assert!(lines[2].contains(".m"));
        assert!(lines[3].contains(".z"));
    }

    // Valid field name helper tests
    #[test]
    fn valid_field_name_simple() {
        let printer = Printer::new(false);
        assert!(printer.valid_field_name("name"));
        assert!(printer.valid_field_name("Name"));
        assert!(printer.valid_field_name("_name"));
        assert!(printer.valid_field_name("$name"));
        assert!(printer.valid_field_name("name123"));
    }

    #[test]
    fn invalid_field_name_cases() {
        let printer = Printer::new(false);
        assert!(!printer.valid_field_name(""));
        assert!(!printer.valid_field_name("123"));
        assert!(!printer.valid_field_name("field-name"));
        assert!(!printer.valid_field_name("field name"));
        assert!(!printer.valid_field_name("field.name"));
    }

    #[test]
    fn valid_field_name_unicode() {
        let printer = Printer::new(false);
        assert!(printer.valid_field_name("名前")); // Japanese
        assert!(printer.valid_field_name("имя")); // Russian
    }
}
