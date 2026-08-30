//! # Template command
//!
//! Printing the TOML form of a card, blank or prefilled. It always emits TOML
//! and never a vCard, the way back being the edit verb.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{clap::parsers::path_parser, printer::Printer};

use crate::{
    cli::args::{Output, SourceArg, VersionArg},
    template::TcardTemplate,
};

/// Print a TOML template, blank or prefilled from a vCard.
#[derive(Debug, Parser)]
pub struct TemplateCommand {
    /// The card the template is prefilled from.
    #[command(flatten)]
    pub source: SourceArg,
    /// Write to this file instead of stdout.
    #[arg(short, long, value_name = "PATH", value_parser = path_parser)]
    pub output: Option<PathBuf>,
    /// The vCard version the template is written for.
    #[command(flatten)]
    pub version: VersionArg,
}

impl TemplateCommand {
    /// Project the source card and write the TOML form out.
    pub fn execute(self, _printer: &mut impl Printer) -> Result<()> {
        let version = self.version.version.into();
        let source = self.source.load(version)?;
        let toml = TcardTemplate::parse(&source, version)?.project();

        Output(self.output.as_deref()).write(toml.as_bytes())
    }
}
