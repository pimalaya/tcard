//! # Error codes
//!
//! A stable, machine-readable name for the failures a caller is expected to
//! act on, printed as `code` beside the message under `--json`.
//!
//! The message stays for humans and may be reworded at any time; the code is
//! the contract. A failure with no code prints `code: null`. The mechanism
//! (`CodedError`, `code_of`, the report) lives in `pimalaya_cli::error`; only
//! the vocabulary is tCard's.
//!
//! The library is free of pimalaya-cli, so it fails with a [`TcardError`],
//! and [`coded`] gives the chain the matching code. The editor round trip
//! raises its own codes where it fails.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use anyhow::Error;
use pimalaya_cli::error::CodedError;
use schemars::JsonSchema;
use serde::Serialize;

use crate::error::TcardError;

/// The stable codes, each serialized in kebab-case.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// A card does not read as a vCard.
    CardUnreadable,
    /// The TOML document does not parse.
    TemplateUnreadable,
    /// The document still holds a merge collision, one key written twice.
    CollisionUndecided,
    /// The document names an embedded digest the card does not hold.
    EmbeddedUnknown,
    /// A file the document names cannot be read.
    FileUnreadable,
    /// Neither `--editor`, `$VISUAL` nor `$EDITOR` names an editor.
    EditorMissing,
    /// The editor could not be spawned or exited with a failure.
    EditorFailed,
}

impl From<&TcardError> for ErrorCode {
    fn from(err: &TcardError) -> Self {
        match err {
            TcardError::ParseVcard(_) | TcardError::ReadCard { .. } => Self::CardUnreadable,
            TcardError::ParseToml(_) => Self::TemplateUnreadable,
            TcardError::Undecided(_) => Self::CollisionUndecided,
            TcardError::UnknownEmbedded(_) => Self::EmbeddedUnknown,
            TcardError::ReadFile { .. } => Self::FileUnreadable,
        }
    }
}

/// Gives a failure the code of the library error it carries, messages kept.
pub fn coded(err: Error) -> Error {
    let Some(cause) = err
        .chain()
        .find_map(|cause| cause.downcast_ref::<TcardError>())
    else {
        return err;
    };

    let code = ErrorCode::from(cause);
    let mut messages: Vec<String> = err.chain().map(ToString::to_string).collect();
    let root = messages.pop().unwrap_or_default();
    let mut coded = Error::new(CodedError::new(code, root));

    for message in messages.into_iter().rev() {
        coded = coded.context(message);
    }

    coded
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use anyhow::{Context, anyhow};
    use pimalaya_cli::error::code_of;

    use crate::{
        cli::error::{ErrorCode, coded},
        error::TcardError,
    };

    #[test]
    fn a_library_cause_is_coded_with_its_messages_kept() {
        let err = Err::<(), _>(TcardError::Undecided("email".into()))
            .context("Cannot fold back \"/tmp/tcard.toml\"")
            .unwrap_err();
        let before = format!("{err:#}");

        let err = coded(err);

        assert_eq!(
            code_of::<ErrorCode>(&err),
            Some(ErrorCode::CollisionUndecided)
        );
        assert_eq!(format!("{err:#}"), before);
    }

    #[test]
    fn a_failure_with_no_library_cause_stays_uncoded() {
        let err = coded(anyhow!("Something else"));
        assert_eq!(code_of::<ErrorCode>(&err), None);
    }
}
