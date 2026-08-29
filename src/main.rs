//! # tCard
//!
//! Binary entry point of the tCard CLI. It parses the command-line interface
//! declared in [`tcard::cli`], wires the logger and the printer from the shared
//! pimalaya-cli toolkit, then hands control to the parsed command.
//!
//! Everything below this file lives in the library, whose own header carries
//! the crate architecture: see [`tcard`].

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::StdoutPrinter};

use tcard::cli::Cli;

fn main() {
    let cli = Cli::parse();
    let mut printer = StdoutPrinter::new(&cli.json);

    let result = execute(cli, &mut printer);
    ErrorReport::eval(&mut printer, result);
}

fn execute(cli: Cli, printer: &mut StdoutPrinter) -> Result<()> {
    Logger::try_init(&cli.log)?;
    cli.cmd.execute(printer)
}
