//! # Output
//!
//! What a verb prints: the document it produced, whole, or the receipt of the
//! file it wrote the document to.
//!
//! In text, a document prints as itself, so stdout stays a vCard or a TOML
//! form a pipe can take on, and a receipt prints nothing: tCard runs as the
//! composer of a caller whose own stdout may be the card it pipes onwards.

use alloc::{
    format,
    string::{String, ToString},
};
use core::fmt;

use std::{fs, path::Path};

use anyhow::{Context, Result};
use log::info;
use schemars::JsonSchema;
use serde::Serialize;

/// A document a verb produced, printed whole or written to a file.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum DocumentOutput {
    /// The document, printed whole.
    Printed {
        /// The TOML form for `template`, the vCard for the other verbs.
        contents: String,
    },
    /// The receipt of the file the document was written to.
    Saved {
        /// The file written.
        path: String,
        /// How many bytes were written.
        size: usize,
    },
}

impl DocumentOutput {
    /// Writes the document to `target` when there is one, or keeps it whole.
    pub fn write(target: Option<&Path>, contents: String) -> Result<Self> {
        let Some(path) = target else {
            return Ok(Self::Printed { contents });
        };

        info!("writing {} bytes to {path:?}", contents.len());
        fs::write(path, &contents).with_context(|| format!("Cannot write to {path:?}"))?;

        Ok(Self::Saved {
            path: path.display().to_string(),
            size: contents.len(),
        })
    }
}

impl fmt::Display for DocumentOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Printed { contents } => f.write_str(contents),
            Self::Saved { .. } => Ok(()),
        }
    }
}
