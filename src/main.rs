mod deserializer;
mod format;
mod input;
mod printer;

use crate::deserializer::Deserializer;
use crate::format::Format;
use crate::input::Input;
use crate::printer::Printer;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The serialized data file to be parsed
    #[clap(parse(from_os_str))]
    file: Option<std::path::PathBuf>,

    /// Format of serialized data
    #[clap(short, long, arg_enum)]
    format: Option<Format>,

    /// Sort output
    #[clap(short, long)]
    sort: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let input = Input::try_from(cli.file)?;

    let format = cli
        .format
        .or_else(|| input.guess_format())
        .unwrap_or(Format::Json);

    let value = Deserializer::new(input, format).deserialize()?;
    let printer = Printer::new(cli.sort);
    printer.print(value);
    Ok(())
}
