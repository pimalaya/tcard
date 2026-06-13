//! The `tcard` binary CLI.
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

use alloc::{format, string::String, vec::Vec};
use std::{
    fs,
    io::{Read, Write, stdin, stdout},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use calcard::vcard::{VCard, VCardVersion};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use log::debug;
use pimalaya_cli::{
    clap::{
        args::{JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
        parsers::path_parser,
    },
    long_version,
    printer::Printer,
    prompt,
};
use uuid::Uuid;

use crate::{error::TcardError, template, vcard};

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
        let (cards, version, _) = load(&self.source, &self.version)?;
        let toml = template::project(&cards, version);
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
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        let (cards, version, src) = load(&self.source, &self.version)?;
        let scaffold = template::project(&cards, version);

        let mut builder = edit::Builder::new();
        builder.suffix(".toml");

        debug!("opening editor on the projected scaffold");
        let mut edited =
            edit::edit_with_builder(&scaffold, &builder).context("Cannot spawn editor")?;

        // A broken edit is recoverable: re-open the editor seeded with the
        // user's own buffer so the edits are never lost. JSON output is
        // non-interactive, so the error just propagates there.
        let vcard = loop {
            match template::apply(&src, &edited) {
                Ok(vcard) => break vcard,
                Err(TcardError::ParseToml(err)) if !printer.is_json() => {
                    let message = format!("Cannot parse TOML buffer:\n\n{err}\nRe-edit to fix it?");
                    if !prompt::bool(message, true)? {
                        return Err(TcardError::ParseToml(err).into());
                    }
                    edited = edit::edit_with_builder(&edited, &builder)
                        .context("Cannot spawn editor")?;
                }
                Err(err) => return Err(err.into()),
            }
        };

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
            debug!("reading vCard from stdin");
            let mut buffer = String::new();
            stdin()
                .read_to_string(&mut buffer)
                .context("Cannot read vCard from stdin")?;
            return Ok(Some(buffer));
        }

        if let Some(path) = self.file_path() {
            debug!("reading vCard from {path:?}");
            let contents =
                fs::read_to_string(&path).with_context(|| format!("Cannot read vCard {path:?}"))?;
            return Ok(Some(contents));
        }

        if source.trim_start().starts_with("BEGIN:VCARD") {
            debug!("treating source as literal vCard contents");
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

/// Load every source vCard with the raw text and resolved version: the
/// first card's own version when present, else the requested one. The text
/// is returned so [`template::apply`] can preserve every untouched byte.
fn load(source: &SourceArg, version: &VersionArg) -> Result<(Vec<VCard>, VCardVersion, String)> {
    let requested: VCardVersion = version.version.into();

    match source.resolve()? {
        Some(text) => {
            let cards = vcard::parse_all(&text)?;
            let version = cards
                .first()
                .and_then(|card| card.version())
                .unwrap_or(requested);
            Ok((cards, version, text))
        }
        None => {
            // A new card is seeded with a fresh UID so the contact has
            // a stable identifier from the start.
            debug!("seeding a new card with a fresh UID");
            let text = format!(
                "BEGIN:VCARD\r\nVERSION:{requested}\r\nUID:urn:uuid:{}\r\nEND:VCARD\r\n",
                Uuid::new_v4()
            );
            let cards = vcard::parse_all(&text)?;
            Ok((cards, requested, text))
        }
    }
}

/// Write bytes to a file, or to stdout when no path is given.
fn write_out(path: Option<&Path>, bytes: &[u8]) -> Result<()> {
    match path {
        Some(path) => {
            debug!("writing {} bytes to {path:?}", bytes.len());
            fs::write(path, bytes).with_context(|| format!("Cannot write to {path:?}"))
        }
        None => {
            debug!("writing {} bytes to stdout", bytes.len());
            stdout().write_all(bytes).context("Cannot write to stdout")
        }
    }
}
