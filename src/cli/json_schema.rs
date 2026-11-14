//! # JSON Schema registry
//!
//! The schemas the `json-schema` command writes, one entry per verb printing
//! structured `--json` output, keyed by the command path joined with hyphens
//! and prefixed `tcard-`, as the man pages are named.
//!
//! Every verb prints a [`DocumentOutput`], and `tcard-error` is the failure
//! object any verb prints instead.
//!
//! [`DocumentOutput`]: crate::cli::output::DocumentOutput

use pimalaya_cli::{clap::commands::json_schema::JsonSchemaRegistry, error::ErrorReport};

use crate::cli::{error::ErrorCode, output::DocumentOutput};

/// Builds the command-to-schema registry consumed by `json-schema`.
pub fn registry() -> JsonSchemaRegistry {
    let mut registry = JsonSchemaRegistry::new("tcard", env!("CARGO_PKG_VERSION"));

    registry.insert::<DocumentOutput>("tcard-template");
    registry.insert::<DocumentOutput>("tcard-edit");
    registry.insert::<DocumentOutput>("tcard-apply");
    registry.insert::<DocumentOutput>("tcard-merge");
    registry.insert::<ErrorReport<ErrorCode>>("tcard-error");

    registry
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec::Vec};

    use clap::CommandFactory;

    use crate::cli::{Cli, json_schema::registry};

    #[test]
    fn the_registry_matches_the_command_tree() {
        assert_eq!(registry().check(&Cli::command(), &[]), Vec::<String>::new());
    }

    #[test]
    fn every_payload_is_an_object() {
        assert_eq!(registry().non_object_roots(), Vec::<String>::new());
    }
}
