mod color;
mod deserializer;
mod format;
mod input;
mod printer;

use crate::color::Color;
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

    /// When to use colors
    #[clap(short, long, arg_enum, value_name = "WHEN", default_value_t = Color::Auto)]
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

    let value = Deserializer::new(input, format).deserialize()?;
    let printer = Printer::new(cli.sort);

    let mut stdout = std::io::stdout().lock();
    printer.print(&mut stdout, &value).or_else(|error| {
        match error.root_cause().downcast_ref::<std::io::Error>() {
            Some(io_error) => match io_error.kind() {
                std::io::ErrorKind::BrokenPipe => Ok(()),
                _ => Err(error),
            },
            None => Err(error),
        }
    })?;

    Ok(())
}
