//! # Command-line interface
//!
//! The verbs the `tcard` binary offers: [`TemplateCommand`] prints the TOML
//! form of a card, [`EditCommand`] edits one through `$EDITOR`,
//! [`ApplyCommand`] folds an edited form back, and [`MergeCommand`] decides
//! what a three-way merge could not.
//!
//! [`Cli`] is the clap entry point parsed by main and [`Command`] the flat
//! grammar it dispatches to, one module per verb below it. [`args`] holds what
//! several verbs take, [`editor`] the round trip through `$EDITOR`, [`output`]
//! what the verbs print, [`error`] the codes of their failures and
//! [`json_schema`] the schemas of both.
//!
//! A source resolves deterministically: `-` reads stdin, an existing file is
//! read, otherwise the value is treated as literal vCard contents, and
//! omitting it starts from a blank template. The paths back to a vCard are
//! `edit` and `apply`, where the card the form came from is still in hand.

pub mod apply;
pub mod args;
pub mod edit;
pub mod editor;
pub mod error;
pub mod json_schema;
pub mod merge;
pub mod output;
pub mod template;

use alloc::format;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{JsonFlag, LogFlags},
        commands::{
            completion::CompletionCommand, json_schema::JsonSchemaCommand, manual::ManualCommand,
        },
    },
    footer, long_version,
    printer::Printer,
};

use crate::cli::{
    apply::ApplyCommand, edit::EditCommand, merge::MergeCommand, template::TemplateCommand,
};

/// The tCard command-line interface.
///
/// The version is not propagated to the verbs, which the rest of Pimalaya
/// does: `-V`/`--version` on `template` and `edit` is the vCard version they
/// write, and clap would refuse the two under one flag.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about, long_version = long_version!(), after_help = footer!())]
#[command(infer_subcommands = true)]
pub struct Cli {
    /// The verb to run.
    #[command(subcommand)]
    pub cmd: Command,
    /// Whether output and errors are rendered as JSON rather than as text.
    #[command(flatten)]
    pub json: JsonFlag,
    /// Where and how verbosely the logger writes.
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
    Apply(ApplyCommand),
    Merge(MergeCommand),
    #[command(alias = "completions")]
    Completion(CompletionCommand),
    #[command(alias = "manuals")]
    Manual(ManualCommand),
    #[command(alias = "json-schemas")]
    JsonSchema(JsonSchemaCommand),
}

impl Command {
    /// Runs the parsed verb, a library failure carrying its code.
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        let result = match self {
            Self::Template(cmd) => cmd.execute(printer),
            Self::Apply(cmd) => cmd.execute(printer),
            Self::Edit(cmd) => cmd.execute(printer),
            Self::Merge(cmd) => cmd.execute(printer),
            Self::Completion(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manual(cmd) => cmd.execute(printer, Cli::command()),
            Self::JsonSchema(cmd) => cmd.execute(printer, json_schema::registry()),
        };

        result.map_err(error::coded)
    }
}
