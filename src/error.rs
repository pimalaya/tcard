//! # Errors
//!
//! The crate-wide error and result types.

use core::result;

use alloc::string::String;

use thiserror::Error;

/// The global `Error` enum of the library.
#[derive(Debug, Error)]
pub enum TcardError {
    /// The input is not a vCard the reader can parse.
    #[error("Cannot parse vCard: {0}")]
    ParseVcard(String),
    /// One of a merge's three cards does not read, named by the side it was
    /// given as.
    #[error("Cannot read the {side} card: {message}")]
    ReadCard {
        /// The side the unreadable card was given as.
        side: &'static str,
        /// What vcard-rs made of it.
        message: String,
    },
    /// The edited TOML buffer is not valid TOML.
    #[error("Cannot parse TOML buffer")]
    ParseToml(#[source] toml_edit::TomlError),
    /// The edited document still holds a collision, written as the same key
    /// twice, so it cannot be applied until one of the lines is gone.
    #[error("Field {0} left undecided, keep one of its lines")]
    Undecided(String),
}

/// The global `Result` alias of the library.
pub type Result<T> = result::Result<T, TcardError>;
