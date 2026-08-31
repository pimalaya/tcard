//! # Editor round trip
//!
//! Opening the reader's editor on a TOML document and folding what comes back
//! onto the card it was projected from.
//!
//! tCard writes the document to a temporary file, spawns the editor on its
//! path with every stream inherited, and reads the file back once the command
//! exits. Nothing is captured: an editor handed a pipe instead of the terminal
//! hangs, or draws where nothing reads.
//!
//! A document that does not fold back re-opens seeded with the reader's own
//! buffer, so an edit is never lost, whether it is broken TOML or a collision
//! still to decide.

use alloc::{format, string::String, vec::Vec};

use std::{
    env::{self, temp_dir},
    fs,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
};

use anyhow::{Context, Error, Result, anyhow, bail};
use log::{debug, info};
use pimalaya_cli::{printer::Printer, prompt};
use uuid::Uuid;

use crate::error::{TcardError, TcardResult};

/// The editor round trip over one TOML document.
pub struct Editor<'a> {
    /// The document the editor opens on.
    pub document: &'a str,
    /// Command to open it with, winning over `$VISUAL` and `$EDITOR`.
    pub command: Option<&'a str>,
}

impl Editor<'_> {
    /// Edit the document, then fold it back with `apply`.
    ///
    /// A failed fold offers to re-open the editor on the buffer that failed,
    /// looping until it applies or the reader declines. JSON output is
    /// non-interactive and propagates the error instead.
    ///
    /// The temporary file goes when the fold succeeds, and stays, named in the
    /// error, when it does not: what someone typed outlives the run.
    pub fn apply(
        &self,
        printer: &mut impl Printer,
        apply: impl Fn(&str) -> TcardResult<String>,
    ) -> Result<String> {
        let command = self.command()?;
        let path = temp_dir().join(format!("tcard-{}.toml", Uuid::new_v4()));

        info!(
            "seeding the editor with {} bytes in {path:?}",
            self.document.len()
        );
        fs::write(&path, self.document).with_context(|| format!("Cannot write {path:?}"))?;

        loop {
            let status = match spawn(&command, &path) {
                Ok(status) => status,
                // NOTE: the editor never ran, so the file holds nothing but
                // what tCard put there a moment ago.
                Err(err) => {
                    remove(&path);
                    return Err(err);
                }
            };

            if !status.success() {
                return Err(keep(&path, anyhow!("Editor exited with {status}")));
            }

            let edited =
                fs::read_to_string(&path).with_context(|| format!("Cannot read {path:?}"))?;

            let err = match apply(&edited) {
                Ok(vcard) => {
                    remove(&path);
                    return Ok(vcard);
                }
                Err(err) => err,
            };

            let recoverable = matches!(err, TcardError::ParseToml(_) | TcardError::Undecided(_));

            if !recoverable || printer.is_json() || !prompt::bool(reprompt(&err), true)? {
                return Err(keep(&path, err.into()));
            }
        }
    }

    /// The editor command line: the flag, then `$VISUAL`, then `$EDITOR`.
    ///
    /// Nothing follows those three. A fallback list ends in generic file
    /// openers, which hand the document to the desktop and return before it is
    /// closed, and a round trip that read back an untouched document is
    /// indistinguishable from an edit someone thought better of.
    fn command(&self) -> Result<String> {
        if let Some(command) = self.command {
            return Ok(command.into());
        }

        for name in ["VISUAL", "EDITOR"] {
            match env::var(name) {
                Ok(command) if !command.trim().is_empty() => return Ok(command),
                _ => debug!("{name} names no editor"),
            }
        }

        bail!("No editor found; set $VISUAL or $EDITOR, or pass --editor <COMMAND>")
    }
}

/// Runs `command` on `path`, waiting for it to exit.
///
/// The command line is split on whitespace, so `code --wait` carries its
/// argument, and the path goes last.
fn spawn(command: &str, path: &PathBuf) -> Result<ExitStatus> {
    let mut argv = command.split_whitespace();

    let Some(program) = argv.next() else {
        bail!("Editor command is empty");
    };

    let args: Vec<&str> = argv.collect();
    info!("spawning {program} on {path:?}");

    Command::new(program)
        .args(args)
        .arg(path)
        // NOTE: inherited is the default, and stating it is the point: an
        // editor given a pipe instead of the terminal hangs or draws where
        // nothing reads.
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Cannot spawn editor {program}"))
}

/// Keeps the buffer and names it in the error, the path being the recovery.
fn keep(path: &PathBuf, err: Error) -> Error {
    err.context(format!("Cannot fold back {path:?}"))
}

/// Drops the buffer, a failure to do so being nothing to report.
fn remove(path: &PathBuf) {
    if let Err(err) = fs::remove_file(path) {
        debug!("cannot remove {path:?}: {err}");
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
