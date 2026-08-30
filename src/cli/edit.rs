//! # Edit command
//!
//! The full round trip: project a card as TOML, open `$EDITOR` on it, fold the
//! edits back, and emit the resulting vCard.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{clap::parsers::path_parser, printer::Printer};

use crate::{
    cli::{
        args::{Output, SourceArg, VersionArg},
        editor::Editor,
    },
    template::Template,
};

/// Edit a vCard as TOML in `$EDITOR`, blank or prefilled from a source.
///
/// The card is projected as a fillable TOML form, opened in an editor, then
/// folded back onto the source. Only the lines you changed are re-rendered, so
/// every other byte of the card survives, the properties this form does not
/// show included.
///
/// The editor is resolved from `$VISUAL`, then `$EDITOR`, then an OS default;
/// tCard reads no configuration file and offers no override. A buffer that
/// does not fold back re-opens seeded with what you wrote, so a broken edit is
/// never lost.
#[derive(Debug, Parser)]
pub struct EditCommand {
    /// The card the form is prefilled from.
    #[command(flatten)]
    pub source: SourceArg,
    /// Write the resulting vCard here instead of stdout (or the source file,
    /// when editing one in place).
    #[arg(short, long, value_name = "PATH", value_parser = path_parser)]
    pub output: Option<PathBuf>,
    /// The vCard version a new card is written at.
    #[command(flatten)]
    pub version: VersionArg,
}

impl EditCommand {
    /// Project the source, edit it, then write the resulting vCard out.
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        let version = self.version.version.into();
        let source = self.source.load(version)?;
        let template = Template::parse(&source, version)?;

        let scaffold = template.project();
        let editor = Editor {
            document: &scaffold,
        };

        let vcard = editor.apply(printer, |edited| template.apply(edited))?;

        let target = self.output.or_else(|| self.source.file_path());
        Output(target.as_deref()).write(vcard.as_bytes())
    }
}
