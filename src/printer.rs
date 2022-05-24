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
