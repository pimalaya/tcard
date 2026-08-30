//! # Shared arguments
//!
//! The source a verb reads its card from, the vCard version it writes at, and
//! the file it writes to.

use alloc::{format, string::String};

use std::{
    fs,
    io::{Read, Write, stdin, stdout},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use log::{debug, info};
use pimalaya_cli::clap::parsers::path_parser;
use uuid::Uuid;
use vcard::version::VcardVersion;

/// Positional vCard source shared by the template and edit verbs.
#[derive(Debug, Parser)]
pub struct SourceArg {
    /// A path to a vCard file, raw vCard contents, or `-` for stdin. Omit to
    /// start from a blank template.
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,
}

impl SourceArg {
    /// The vCard text a verb reads, seeding a new card where there is none.
    ///
    /// A new card is written at `version` and given a fresh `urn:uuid`, so a
    /// contact has a stable identifier from the start.
    pub fn load(&self, version: VcardVersion) -> Result<String> {
        if let Some(text) = self.resolve()? {
            return Ok(text);
        }

        info!("seeding a new card with a fresh UID");

        Ok(format!(
            "BEGIN:VCARD\r\nVERSION:{}\r\nUID:urn:uuid:{}\r\nEND:VCARD\r\n",
            &*version,
            Uuid::new_v4()
        ))
    }

    /// The source as an existing file path, when it resolves to one.
    ///
    /// This is the in-place write default of `edit`.
    pub fn file_path(&self) -> Option<PathBuf> {
        let source = self.source.as_ref()?;

        if source == "-" {
            return None;
        }

        let path = path_parser(source).ok()?;
        path.is_file().then_some(path)
    }

    /// Resolve the source into vCard text, or `None` for a blank template.
    fn resolve(&self) -> Result<Option<String>> {
        let Some(source) = &self.source else {
            return Ok(None);
        };

        if source == "-" {
            info!("reading vCard from stdin");
            let mut buffer = String::new();
            stdin()
                .read_to_string(&mut buffer)
                .context("Cannot read vCard from stdin")?;
            return Ok(Some(buffer));
        }

        if let Some(path) = self.file_path() {
            info!("reading vCard from {path:?}");
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

/// vCard versions tCard can target, validated by clap.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CardVersion {
    /// vCard 2.1, the pre-standard version older exporters still write.
    #[value(name = "2.1")]
    V2_1,
    /// vCard 3.0, RFC 2426.
    #[value(name = "3.0")]
    V3_0,
    /// vCard 4.0, RFC 6350.
    #[value(name = "4.0")]
    V4_0,
}

impl From<CardVersion> for VcardVersion {
    fn from(version: CardVersion) -> Self {
        match version {
            CardVersion::V2_1 => VcardVersion::V2_1,
            CardVersion::V3_0 => VcardVersion::V3_0,
            CardVersion::V4_0 => VcardVersion::V4_0,
        }
    }
}

/// Where a verb writes its result, stdout when it has no path.
pub struct Output<'a>(pub Option<&'a Path>);

impl Output<'_> {
    /// Write the bytes out, creating or truncating a file target.
    pub fn write(&self, bytes: &[u8]) -> Result<()> {
        match self.0 {
            Some(path) => {
                info!("writing {} bytes to {path:?}", bytes.len());
                fs::write(path, bytes).with_context(|| format!("Cannot write to {path:?}"))
            }
            None => {
                info!("writing {} bytes to stdout", bytes.len());
                stdout().write_all(bytes).context("Cannot write to stdout")
            }
        }
    }
}
