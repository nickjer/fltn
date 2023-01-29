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
