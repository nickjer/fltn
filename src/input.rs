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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    // contents() tests
    #[test]
    fn contents_from_stdin() {
        let input = Input::Stdin("test content".to_string());
        assert_eq!(input.contents(), "test content");
    }

    #[test]
    fn contents_from_file() {
        let input = Input::File(PathBuf::from("/test/path.json"), "file content".to_string());
        assert_eq!(input.contents(), "file content");
    }

    #[test]
    fn contents_empty_string() {
        let input = Input::Stdin(String::new());
        assert_eq!(input.contents(), "");
    }

    #[test]
    fn contents_with_unicode() {
        let input = Input::Stdin("日本語 テスト 🎉".to_string());
        assert_eq!(input.contents(), "日本語 テスト 🎉");
    }

    #[test]
    fn contents_with_newlines() {
        let input = Input::Stdin("line1\nline2\nline3".to_string());
        assert_eq!(input.contents(), "line1\nline2\nline3");
    }

    // guess_format() tests
    #[test]
    fn guess_format_json_file() {
        let input = Input::File(PathBuf::from("data.json"), String::new());
        assert!(matches!(input.guess_format(), Some(Format::Json)));
    }

    #[test]
    fn guess_format_csv_file() {
        let input = Input::File(PathBuf::from("data.csv"), String::new());
        assert!(matches!(input.guess_format(), Some(Format::Csv)));
    }

    #[test]
    fn guess_format_toml_file() {
        let input = Input::File(PathBuf::from("config.toml"), String::new());
        assert!(matches!(input.guess_format(), Some(Format::Toml)));
    }

    #[test]
    fn guess_format_yaml_file() {
        let input = Input::File(PathBuf::from("config.yaml"), String::new());
        assert!(matches!(input.guess_format(), Some(Format::Yaml)));
    }

    #[test]
    fn guess_format_yml_file() {
        let input = Input::File(PathBuf::from("config.yml"), String::new());
        assert!(matches!(input.guess_format(), Some(Format::Yaml)));
    }

    #[test]
    fn guess_format_unknown_extension() {
        let input = Input::File(PathBuf::from("data.xyz"), String::new());
        assert!(input.guess_format().is_none());
    }

    #[test]
    fn guess_format_stdin_returns_none() {
        let input = Input::Stdin("{}".to_string());
        assert!(input.guess_format().is_none());
    }

    #[test]
    fn guess_format_no_extension() {
        let input = Input::File(PathBuf::from("README"), String::new());
        assert!(input.guess_format().is_none());
    }

    // TryFrom tests with tempfile
    #[test]
    fn try_from_file_reads_content() {
        use std::io::Write;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{{\"key\": \"value\"}}").unwrap();

        let path = temp_file.path().to_path_buf();
        let input = Input::try_from(Some(path.clone())).unwrap();

        assert!(matches!(input, Input::File(_, _)));
        assert_eq!(input.contents(), "{\"key\": \"value\"}");
    }

    #[test]
    fn try_from_file_preserves_path() {
        use std::io::Write;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "content").unwrap();

        let path = temp_file.path().to_path_buf();
        let input = Input::try_from(Some(path.clone())).unwrap();

        if let Input::File(input_path, _) = input {
            assert_eq!(input_path, path);
        } else {
            panic!("Expected Input::File variant");
        }
    }

    #[test]
    fn try_from_nonexistent_file_returns_error() {
        let path = PathBuf::from("/nonexistent/path/to/file.json");
        let result = Input::try_from(Some(path));

        assert!(result.is_err());
    }

    #[test]
    fn try_from_file_reads_unicode() {
        use std::io::Write;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "日本語テスト").unwrap();

        let path = temp_file.path().to_path_buf();
        let input = Input::try_from(Some(path)).unwrap();

        assert_eq!(input.contents(), "日本語テスト");
    }

    #[test]
    fn try_from_file_reads_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        let input = Input::try_from(Some(path)).unwrap();

        assert_eq!(input.contents(), "");
    }

    #[test]
    fn try_from_file_reads_large_content() {
        use std::io::Write;

        let mut temp_file = NamedTempFile::new().unwrap();
        let large_content = "x".repeat(1_000_000);
        write!(temp_file, "{}", large_content).unwrap();

        let path = temp_file.path().to_path_buf();
        let input = Input::try_from(Some(path)).unwrap();

        assert_eq!(input.contents().len(), 1_000_000);
    }

    #[test]
    fn try_from_file_with_json_extension_can_guess_format() {
        use std::io::Write;

        let temp_file = NamedTempFile::with_suffix(".json").unwrap();
        let mut file = temp_file.reopen().unwrap();
        write!(file, "{{}}").unwrap();

        let path = temp_file.path().to_path_buf();
        let input = Input::try_from(Some(path)).unwrap();

        assert!(matches!(input.guess_format(), Some(Format::Json)));
    }
}
