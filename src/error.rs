//! The crate-wide error and result types.

use core::result;

use alloc::string::String;

use thiserror::Error;

/// The global `Error` enum of the library.
#[derive(Debug, Error)]
pub enum TcardError {
    /// calcard parsed the input as iCalendar instead of a vCard.
    #[error("Contents parsed as iCalendar, not a vCard")]
    NotAVcard,
    /// calcard could not parse the input as a vCard.
    #[error("Cannot parse vCard: {0}")]
    ParseVcard(String),
    /// The edited TOML buffer is not valid TOML.
    #[error("Cannot parse TOML buffer")]
    ParseToml(#[source] toml_edit::TomlError),
}

/// The global `Result` alias of the library.
pub type Result<T> = result::Result<T, TcardError>;
