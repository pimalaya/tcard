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
