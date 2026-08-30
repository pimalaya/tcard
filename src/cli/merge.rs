//! # Merge command
//!
//! Merging two divergent cards against their base, then deciding the rest in
//! `$EDITOR`. It takes three paths rather than a source, a merge needing three
//! cards at once.

use alloc::{format, string::String};

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use pimalaya_cli::{clap::parsers::path_parser, printer::Printer};

use crate::{
    cli::{args::Output, editor::Editor},
    merge::Merge,
};

/// Merge two divergent vCards against their common base, then decide the rest
/// in `$EDITOR`.
///
/// The merge settles every field only one side touched. What both sides
/// changed is written into the document twice, once per side, which TOML
/// refuses to parse: keep one of the two lines, or replace them with a value
/// of your own. The output is written only once the document parses.
#[derive(Debug, Parser)]
pub struct MergeCommand {
    /// The common ancestor both sides diverged from.
    #[arg(value_name = "BASE", value_parser = path_parser)]
    pub base: PathBuf,
    /// The local side of the divergence.
    #[arg(value_name = "LOCAL", value_parser = path_parser)]
    pub local: PathBuf,
    /// The remote side of the divergence.
    #[arg(value_name = "REMOTE", value_parser = path_parser)]
    pub remote: PathBuf,
    /// Write the merged vCard here, once the document is decided.
    #[arg(short, long, value_name = "PATH", value_parser = path_parser)]
    pub output: PathBuf,
}

impl MergeCommand {
    /// Merge the three cards, decide the rest, then write the vCard out.
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        let base = read(&self.base)?;
        let local = read(&self.local)?;
        let remote = read(&self.remote)?;

        let merged = Merge {
            base: &base,
            local: &local,
            remote: &remote,
        }
        .project()?;

        let editor = Editor {
            document: &merged.toml,
        };
        let vcard = editor.apply(printer, |edited| merged.apply(edited))?;

        Output(Some(&self.output)).write(vcard.as_bytes())
    }
}

/// Read one of the three cards a merge takes.
fn read(path: &PathBuf) -> Result<String> {
    info!("reading vCard from {path:?}");
    fs::read_to_string(path).with_context(|| format!("Cannot read vCard {path:?}"))
}
