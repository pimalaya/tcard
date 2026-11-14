---
cairn: log
change: the-family-standards
landed: 2026-11-14
---

# tCard took the family standards

Cargo.toml moved to vcard-rs 0.6 and pimalaya-cli 0.4 (normal and build dependencies, `[patch.crates-io]` to ../cli in place of the 55b6693 git pin), `rust-version` 1.89, every dependency with `default-features = false` and named features (clap without `env` and `wrap_help`, with `error-context`, `help`, `suggestions` and `usage`; toml_edit `parse` and `display`; uuid `std` and `v4`), and schemars, serde and simd-cpu under `cli`. vcard-rs 0.6 needed no code change; pimalaya-cli 0.4 moved the completion and manual commands under `clap::commands::completion` and `clap::commands::manual`.

src/template/media.rs encodes and decodes on base64-codec, refusing a decoding that is not clean, so `PHOTO;ENCODING=b:not base64!` still reads as a reference; src/main.rs installs `SimdCpu::install_detected()` first.

src/cli/output.rs holds `DocumentOutput`, an untagged `Printed {contents}` or `Saved {path, size}`, which replaces the `Output` writer of src/cli/args.rs; the four verbs print it. src/cli/error.rs holds `ErrorCode` and `coded`, applied by `Command::execute`, on Comodoro's model; src/cli/editor.rs raises `editor-missing` and `editor-failed` as `CodedError`s at the root of their chains. `apply - -` fails through `error::usage_error`, coded `usage-invalid`, exit 1. src/main.rs parses with `pimalaya_cli::clap::parse` and evaluates with `ErrorReport::<ErrorCode>::eval_coded`. src/cli/json_schema.rs registers the four verbs and `tcard-error`, the registry check holding no command silent; package.nix installs the schemas under share/schemas.

vcard-rs 0.6 folds an edited 3.0 or 4.0 line at 75 octets, where tCard wrote it unfolded: checked with the binary on 4.0, 3.0 (refolded from the first changed byte, a fold landing inside a `\,` escape, which unfolding restores) and 2.1 (one line). The template spec and the lib header's known limitations say so.

Tests: tests/output.rs (nine, the binary under `--json` and in text) and `an_edited_long_line_is_folded` in tests/projection.rs failed on the old code first, run from a `git archive HEAD` copy. deny.toml took Comodoro's `[graph] targets` and lost the unused CC0-1.0.

Checks: `cargo fmt`; `cargo clippy --all-targets -- -D warnings` with default features, `--all-features` and `--no-default-features`; `cargo test --all-features`; `cargo deny check` (advisories, bans, licences and sources ok; warnings: syn 2 beside syn 3 through pimalaya-cli's inquire and crossterm, and `allow-org.github` unmatched while the patch is a path).

Capabilities: output created (five requirements); api (Base64 runs on the family codec ADDED); template (A line the document leaves alone keeps its layout MODIFIED).
