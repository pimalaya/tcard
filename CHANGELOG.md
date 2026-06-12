# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added the `project` / `apply` projection library between a calcard `VCard` and an ergonomic TOML buffer.

  `project` emits a fillable TOML form listing the modeled vCard vocabulary; fields are uncommented and empty (an empty value is ignored, like a removed line), prefilled when present, and carry an inline `# e.g. ...` hint only where the value is not self-evident (`geo`, `tz`, `impp`, `photo`, `gender`, dates); required properties are flagged `# required` version-aware (`FN` always, `N` before 4.0). Typed properties (`email`, `tel`, `address`, `url`) keep a single section with their accepted `TYPE` values listed in a trailing comment. `UID` is not modeled: like `VERSION` it is app-managed (seeded for new cards, preserved otherwise) and cannot be set through the buffer. `apply` rebuilds modeled fields from the edited buffer and carries every unmodeled property (custom `X-*`, vendor extensions, Apple `item1.*` groups) over verbatim, since the TOML is an editing affordance rather than an interchange format.

- Added the `tcard` CLI with two verbs.

  `template [SOURCE]` prints the TOML scaffold (blank or prefilled). `edit [SOURCE]` runs the full "project → `$EDITOR` → apply" round-trip and emits the resulting vCard, writing a file source back in place. `SOURCE` resolves deterministically: `-` reads stdin, an existing file is read, otherwise the value is treated as literal vCard contents, and omitting it starts from a blank template. A `-V`/`--version` flag on each verb selects the target vCard version (the root `--version` stays the app version), and new (sourceless) cards are seeded with a fresh `urn:uuid` v4 `UID`.
