//! # Apply command
//!
//! Folding an edited TOML document back onto the card it was projected from,
//! with no editor in the middle. It is the way back the edit verb takes
//! interactively, available to a script.

use alloc::{format, string::String};

use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Error, Result, bail};
use clap::Parser;
use log::info;
use pimalaya_cli::{clap::parsers::path_parser, printer::Printer};

use crate::{
    cli::args::{Output, SourceArg, VersionArg},
    template::TcardTemplate,
};

/// Fold an edited TOML document back onto a vCard.
///
/// This is `template` in reverse and `edit` without the editor: the document
/// is the form as it was edited, the source is the card it was projected from,
/// and only the lines the form changed are re-rendered. A document that does
/// not parse, or that leaves a merge collision undecided, is an error here
/// rather than a question.
///
/// Editing a file writes it back in place, as `edit` does, so `--output`
/// is what sends the result elsewhere.
#[derive(Debug, Parser)]
pub struct ApplyCommand {
    /// The edited TOML document: a path, or `-` for stdin.
    #[arg(value_name = "TEMPLATE")]
    pub template: String,
    /// The card the document was projected from.
    #[command(flatten)]
    pub source: SourceArg,
    /// Write the resulting vCard here instead of stdout (or the source file,
    /// when folding onto one in place).
    #[arg(short, long, value_name = "PATH", value_parser = path_parser)]
    pub output: Option<PathBuf>,
    /// The vCard version a card folded onto nothing is written at.
    #[command(flatten)]
    pub version: VersionArg,
}

impl ApplyCommand {
    /// Read the document, fold it onto the source, then write the vCard out.
    pub fn execute(self, _printer: &mut impl Printer) -> Result<()> {
        if self.template == "-" && self.source.source.as_deref() == Some("-") {
            bail!("Only one of TEMPLATE and SOURCE can be read from stdin");
        }

        let edited = self.read()?;
        let version = self.version.version.into();
        let source = self.source.load(version)?;
        let vcard = TcardTemplate::parse(&source, version)?.apply(&edited)?;

        let target = self.output.or_else(|| self.source.file_path());

        Output(target.as_deref()).write(vcard.as_bytes())
    }

    /// The edited document, from stdin or from the path it names.
    fn read(&self) -> Result<String> {
        if self.template == "-" {
            info!("reading TOML document from stdin");
            let mut buffer = String::new();
            stdin()
                .read_to_string(&mut buffer)
                .context("Cannot read TOML document from stdin")?;
            return Ok(buffer);
        }

        let path = path_parser(&self.template).map_err(Error::msg)?;
        info!("reading TOML document from {path:?}");

        fs::read_to_string(&path).with_context(|| format!("Cannot read TOML document {path:?}"))
    }
}
