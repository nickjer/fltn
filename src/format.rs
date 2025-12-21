use clap::ValueEnum;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
}
