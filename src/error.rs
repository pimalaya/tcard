//! # Errors
//!
//! The crate-wide error and result types.

use core::{error, fmt, result};

use alloc::string::String;

/// The global `Error` enum of the library.
#[derive(Debug)]
pub enum TcardError {
    /// The input is not a vCard the reader can parse.
    ParseVcard(String),
    /// One of a merge's three cards does not read, named by its side.
    ReadCard {
        /// The side the unreadable card was given as.
        side: &'static str,
        /// What vcard-rs made of it.
        message: String,
    },
    /// The edited TOML buffer is not valid TOML.
    ParseToml(toml_edit::TomlError),
    /// The edited document still holds a collision, written as one key twice.
    ///
    /// It cannot be applied until one of the two lines is gone.
    Undecided(String),
    /// The document names an embedded object by a digest the card does not
    /// hold.
    UnknownEmbedded(String),
    /// A file the document names cannot be read.
    ReadFile {
        /// The path as the document wrote it.
        path: String,
        /// Why it could not be read.
        message: String,
    },
}

impl fmt::Display for TcardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseVcard(message) => {
                write!(f, "Cannot parse vCard: {message}")
            }
            Self::ReadCard { side, message } => {
                write!(f, "Cannot read the {side} card: {message}")
            }
            Self::ParseToml(_) => {
                write!(f, "Cannot parse TOML buffer")
            }
            Self::Undecided(key) => {
                write!(f, "Field {key} left undecided, keep one of its lines")
            }
            Self::UnknownEmbedded(sha256) => {
                write!(f, "Embedded object {sha256} is not in the card")
            }
            Self::ReadFile { path, message } => {
                write!(f, "Cannot read file {path}: {message}")
            }
        }
    }
}

impl error::Error for TcardError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::ParseToml(err) => Some(err),
            _ => None,
        }
    }
}

/// The global `Result` alias of the library.
pub type TcardResult<T> = result::Result<T, TcardError>;
