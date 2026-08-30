//! # Command-line interface
//!
//! The verbs the `tcard` binary offers: [`TemplateCommand`] prints the TOML
//! form of a card, [`EditCommand`] edits one through `$EDITOR`, and
//! [`MergeCommand`] decides what a three-way merge could not.
//!
//! [`Cli`] is the clap entry point parsed by main and [`Command`] the flat
//! grammar it dispatches to, one module per verb below it. [`args`] holds what
//! several verbs take, [`editor`] the round trip through `$EDITOR`.
//!
//! A source resolves deterministically: `-` reads stdin, an existing file is
//! read, otherwise the value is treated as literal vCard contents, and
//! omitting it starts from a blank template. The only path back to a vCard is
//! `edit`, where the card the form came from is still in hand.

pub mod args;
pub mod edit;
pub mod editor;
pub mod merge;
pub mod template;

use alloc::format;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
    },
    long_version,
    printer::Printer,
};

use crate::cli::{edit::EditCommand, merge::MergeCommand, template::TemplateCommand};

/// The tCard command-line interface.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(infer_subcommands = true)]
pub struct Cli {
    /// The verb to run.
    #[command(subcommand)]
    pub cmd: Command,
    /// Whether command output is written as JSON.
    #[command(flatten)]
    pub json: JsonFlag,
    /// How much the run logs, and where the log goes.
    #[command(flatten)]
    pub log: LogFlags,
}

/// The verbs tCard exposes.
///
/// Each variant is documented by the command type it carries, clap taking the
/// help of a subcommand from there: a doc comment here would override it and
/// drop that command's `--help` body.
#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(visible_alias = "tpl")]
    Template(TemplateCommand),
    Edit(EditCommand),
    Merge(MergeCommand),
    #[command(alias = "completions")]
    Completion(CompletionCommand),
    #[command(alias = "manuals")]
    Manual(ManualCommand),
}

impl Command {
    /// Run the verb, reporting through the shared printer.
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        match self {
            Self::Template(cmd) => cmd.execute(printer),
            Self::Edit(cmd) => cmd.execute(printer),
            Self::Merge(cmd) => cmd.execute(printer),
            Self::Completion(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manual(cmd) => cmd.execute(printer, Cli::command()),
        }
    }
}
