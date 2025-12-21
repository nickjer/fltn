mod color;
mod format;
mod input;
mod printer;

use crate::color::Color;
use crate::format::Format;
use crate::input::Input;
use crate::printer::Printer;

use anyhow::{Error, Result};
use clap::Parser;
use jsonpath_rust::JsonPath;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The serialized data file to be parsed
    file: Option<std::path::PathBuf>,

    /// JSONPath expression used for querying data
    #[clap(short, long)]
    path: Option<String>,

    /// Format of serialized data
    #[clap(short, long, value_enum)]
    format: Option<Format>,

    /// When to use colors
    #[clap(short, long, value_enum, value_name = "WHEN", default_value_t = Color::Auto)]
    color: Color,

    /// Sort output
    #[clap(short, long)]
    sort: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.color.set_color();

    let input = Input::try_from(cli.file)?;

    let format = cli
        .format
        .or_else(|| input.guess_format())
        .unwrap_or(Format::Json);

    let value = format.deserialize(input.contents())?;
    let filtered_value = match cli.path {
        Some(path) => value
            .query(&path)
            .map_err(Error::msg)?
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
            .into(),
        None => value,
    };
    let printer = Printer::new(cli.sort);

    let mut stdout = std::io::stdout().lock();
    printer
        .print(&mut stdout, &filtered_value)
        .or_else(|error| {
            let is_broken_pipe = error
                .root_cause()
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::BrokenPipe);
            if is_broken_pipe { Ok(()) } else { Err(error) }
        })?;

    Ok(())
}
