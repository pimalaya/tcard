//! # Editor round trip
//!
//! Opening `$EDITOR` on a TOML document and folding what comes back onto the
//! card it was projected from.
//!
//! A document that does not fold back re-opens seeded with the reader's own
//! buffer, so an edit is never lost, whether it is broken TOML or a collision
//! still to decide.

use alloc::{format, string::String};

use anyhow::{Context, Result};
use log::info;
use pimalaya_cli::{printer::Printer, prompt};

use crate::error::{TcardError, TcardResult};

/// The `$EDITOR` round trip over one TOML document.
pub struct Editor<'a> {
    /// The document the editor opens on.
    pub document: &'a str,
}

impl Editor<'_> {
    /// Edit the document, then fold it back with `apply`.
    ///
    /// A failed fold offers to re-open the editor on the buffer that failed,
    /// looping until it applies or the reader declines. JSON output is
    /// non-interactive and propagates the error instead.
    pub fn apply(
        &self,
        printer: &mut impl Printer,
        apply: impl Fn(&str) -> TcardResult<String>,
    ) -> Result<String> {
        let mut builder = edit::Builder::new();
        builder.suffix(".toml");

        info!("opening editor on the projected document");
        let mut edited =
            edit::edit_with_builder(self.document, &builder).context("Cannot spawn editor")?;

        loop {
            let err = match apply(&edited) {
                Ok(vcard) => return Ok(vcard),
                Err(err) => err,
            };

            let recoverable = matches!(err, TcardError::ParseToml(_) | TcardError::Undecided(_));

            if !recoverable || printer.is_json() || !prompt::bool(reprompt(&err), true)? {
                return Err(err.into());
            }

            edited = edit::edit_with_builder(&edited, &builder).context("Cannot spawn editor")?;
        }
    }
}

/// The question asked after a failed fold, and the offer to re-open.
///
/// It carries the parser's own detail when there is one.
fn reprompt(err: &TcardError) -> String {
    match err {
        TcardError::ParseToml(err) => {
            format!("Cannot parse TOML buffer:\n\n{err}\nRe-edit to fix it?")
        }
        err => format!("{err}\n\nRe-edit to fix it?"),
    }
}
