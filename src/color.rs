use clap::ArgEnum;

#[derive(Debug, Copy, Clone, ArgEnum)]
pub enum Color {
    Never,
    Auto,
    Always,
}

impl Color {
    pub fn set_color(&self) {
        match self {
            Color::Never => colored::control::set_override(false),
            Color::Auto => (),
            Color::Always => colored::control::set_override(true),
        };
    }
}
