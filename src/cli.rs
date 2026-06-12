//! The `tcard` binary: two verbs over the [`crate::template`] projection.
//!
//! - `template [SOURCE]`: print the TOML scaffold, blank or prefilled from a
//!   vCard. Always emits TOML.
//! - `edit [SOURCE]`: project, open `$EDITOR`, apply the edits back onto the
//!   source, and emit the resulting vCard. Always emits a vCard.
//!
//! `SOURCE` resolves deterministically: `-` reads stdin, an existing file is
//! read, otherwise the value is treated as literal vCard contents, and omitting
//! it starts from a blank template. The TOML is an editing affordance; the only
//! path back to a vCard is `edit`, where the original is still in hand.

use std::{
    fs,
    io::{Read, Write, stdin, stdout},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use calcard::vcard::{VCard, VCardVersion};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use pimalaya_cli::{
    clap::{
        args::{JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
        parsers::path_parser,
    },
    long_version,
    printer::Printer,
};
use uuid::Uuid;

use crate::{template, vcard};

/// Root CLI parser.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(infer_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,

    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

/// Top-level subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(visible_alias = "tpl")]
    Template(TemplateCommand),
    Edit(EditCommand),

    Completions(CompletionCommand),
    Manuals(ManualCommand),
}

impl Command {
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        match self {
            Self::Template(cmd) => cmd.execute(printer),
            Self::Edit(cmd) => cmd.execute(printer),
            Self::Completions(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manuals(cmd) => cmd.execute(printer, Cli::command()),
        }
    }
}

/// Print a TOML template, blank or prefilled from a vCard.
#[derive(Debug, Parser)]
pub struct TemplateCommand {
    #[command(flatten)]
    pub source: SourceArg,

    /// Write to this file instead of stdout.
    #[arg(short, long, value_name = "PATH", value_parser = path_parser)]
    pub output: Option<PathBuf>,

    #[command(flatten)]
    pub version: VersionArg,
}

impl TemplateCommand {
    pub fn execute(self, _printer: &mut impl Printer) -> Result<()> {
        let (card, version) = load(&self.source, &self.version)?;
        let toml = template::project(&card, version);

        write_out(self.output.as_deref(), toml.as_bytes())
    }
}

/// Edit a vCard as TOML in `$EDITOR`, blank or prefilled from a source.
#[derive(Debug, Parser)]
pub struct EditCommand {
    #[command(flatten)]
    pub source: SourceArg,
    /// Write the resulting vCard here instead of stdout (or the source file,
    /// when editing one in place).
    #[arg(short, long, value_name = "PATH", value_parser = path_parser)]
    pub output: Option<PathBuf>,
    #[command(flatten)]
    pub version: VersionArg,
}

impl EditCommand {
    pub fn execute(self, _printer: &mut impl Printer) -> Result<()> {
        let (card, version) = load(&self.source, &self.version)?;
        let scaffold = template::project(&card, version);

        let edited = edit::edit_with_builder(&scaffold, edit::Builder::new().suffix(".toml"))
            .context("Cannot spawn editor")?;

        let vcard = template::apply(&card, &edited, version)?;

        let target = self.output.or_else(|| self.source.file_path());
        write_out(target.as_deref(), vcard.as_bytes())
    }
}

/// Positional vCard source shared by both verbs.
#[derive(Debug, Parser)]
pub struct SourceArg {
    /// A path to a vCard file, raw vCard contents, or `-` for stdin.  Omit to
    /// start from a blank template.
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,
}

impl SourceArg {
    /// Resolve the source into vCard text, or `None` for a blank template.
    pub fn resolve(&self) -> Result<Option<String>> {
        let Some(source) = &self.source else {
            return Ok(None);
        };

        if source == "-" {
            let mut buffer = String::new();
            stdin()
                .read_to_string(&mut buffer)
                .context("Cannot read vCard from stdin")?;
            return Ok(Some(buffer));
        }

        if let Some(path) = self.file_path() {
            let contents =
                fs::read_to_string(&path).with_context(|| format!("Cannot read vCard {path:?}"))?;
            return Ok(Some(contents));
        }

        if source.trim_start().starts_with("BEGIN:VCARD") {
            return Ok(Some(source.clone()));
        }

        bail!("Source {source:?} is neither a readable file nor vCard contents")
    }

    /// The source as an existing file path, when it resolves to one; used for
    /// the in-place write default of `edit`.
    fn file_path(&self) -> Option<PathBuf> {
        let source = self.source.as_ref()?;

        if source == "-" {
            return None;
        }

        let path = path_parser(source).ok()?;
        path.is_file().then_some(path)
    }
}

/// Target vCard version, used for blank templates and serialization.
#[derive(Debug, Parser)]
pub struct VersionArg {
    /// Target vCard version. For an existing source the card's own version
    /// wins.
    #[arg(short = 'V', short_alias = 'v', long = "version")]
    #[arg(default_value = "4.0")]
    pub version: CardVersion,
}

/// vCard versions tcard can target, validated by clap.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CardVersion {
    #[value(name = "2.1")]
    V2_1,
    #[value(name = "3.0")]
    V3_0,
    #[value(name = "4.0")]
    V4_0,
}

impl From<CardVersion> for VCardVersion {
    fn from(version: CardVersion) -> Self {
        match version {
            CardVersion::V2_1 => VCardVersion::V2_1,
            CardVersion::V3_0 => VCardVersion::V3_0,
            CardVersion::V4_0 => VCardVersion::V4_0,
        }
    }
}

/// Load the source vCard and resolve the version: the card's own
/// version when present, else the requested one.
fn load(source: &SourceArg, version: &VersionArg) -> Result<(VCard, VCardVersion)> {
    let requested: VCardVersion = version.version.into();

    match source.resolve()? {
        Some(text) => {
            let card = vcard::parse(&text)?;
            let version = card.version().unwrap_or(requested);
            Ok((card, version))
        }
        None => {
            // A new card is seeded with a fresh UID so the contact has
            // a stable identifier from the start.
            let card = vcard::parse(&format!(
                "BEGIN:VCARD\r\nVERSION:{requested}\r\nUID:urn:uuid:{}\r\nEND:VCARD\r\n",
                Uuid::new_v4()
            ))?;
            Ok((card, requested))
        }
    }
}

/// Write bytes to a file, or to stdout when no path is given.
fn write_out(path: Option<&Path>, bytes: &[u8]) -> Result<()> {
    match path {
        Some(path) => fs::write(path, bytes).with_context(|| format!("Cannot write to {path:?}")),
        None => stdout().write_all(bytes).context("Cannot write to stdout"),
    }
}
