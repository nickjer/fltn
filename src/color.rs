use clap::ValueEnum;

#[derive(Debug, Copy, Clone, ValueEnum)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use colored::Colorize;

    #[test]
    fn never_disables_colors() {
        colored::control::unset_override();
        Color::Never.set_color();
        let colored_str = "test".red();
        assert_eq!(colored_str.to_string(), "test");
    }

    #[test]
    fn always_enables_colors() {
        colored::control::unset_override();
        Color::Always.set_color();
        let colored_str = "test".red();
        assert!(colored_str.to_string().contains("\x1b["));
    }

    #[test]
    fn auto_does_not_override() {
        colored::control::unset_override();
        // Set to a known state first
        colored::control::set_override(true);
        Color::Auto.set_color();
        // Auto should not change the override, so it should still be true
        let colored_str = "test".red();
        assert!(colored_str.to_string().contains("\x1b["));
    }
}
