#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # tCard
//!
//! Editing a vCard as ergonomic TOML: a card is projected as a fillable form,
//! a person edits that form, and the edits are folded back onto the card's own
//! bytes.
//!
//! This header is the architecture; the behaviour behind it is specified
//! capability by capability in the repository's cairn/spec folder.
//!
//! ## Layers
//!
//! The crate does no I/O of its own and owns no protocol or storage logic, so
//! it has neither coroutines nor a client layer. Its core is a total function
//! over strings: vCard text in, TOML text out, and back.
//!
//! That core is always compiled and is `no_std` over `alloc`. [`vcard`] reads,
//! [`template`] projects and folds back, [`merge`] reconciles two divergent
//! cards, and [`error`] names every refusal.
//!
//! The `cli` feature adds the verbs and the binary above them, which is the
//! only place a file, an editor or `std` is reached. A library consumer
//! wanting the projection alone pays for none of it.
//!
//! ## The projection
//!
//! A body is read once. [`vcard::parse`] turns a whole stream into vcard-rs's
//! byte-faithful syntax tree, and every verb walks that one tree, so no value
//! passes through a second reader that might normalise it on the way in, where
//! no test comparing the document against that reader could see it.
//!
//! [`template::project`] walks the tree against the static field table and
//! writes the form. A single card flattens at the document root, two or more
//! become `[[card]]` blocks, and what the table does not model is not shown.
//!
//! [`template::apply`] folds an edited form back onto the original text,
//! patching a modelled line rather than rebuilding it: only the value the
//! document moved is written anew, and the rest of the line stays the card's
//! own bytes, its group and the parameters the form never showed included.
//!
//! That is why applying needs the original text and not just the document:
//! the TOML is an editing affordance, never an interchange format.
//!
//! ## The merge
//!
//! [`merge::project`] reconciles a local and a remote card against the base
//! they both came from, then renders the outcome through the same projection,
//! so a merge is read and edited in the form everything else is.
//!
//! What the merge settled by itself is said in a comment at the head of that
//! document. What it could not settle is written once per side, each line
//! naming its side, which makes the same TOML key appear twice.
//!
//! TOML forbids duplicate keys, so an undecided document does not parse.
//! [`merge::apply`] catches that refusal and names the field left undecided
//! rather than reporting a syntax error, and nothing is written until a person
//! has deleted the line they do not want.
//!
//! That refusal is the forcing mechanism: the document is TOML rather than a
//! report so that a collision cannot be scrolled past.
//!
//! ## Modules
//!
//! [`vcard`] is the reader and the byte-preserving edits a fold-back makes
//! through it, [`template`] the projection engine and its facade, [`merge`]
//! the three-way merge over that facade, and [`error`] the crate-wide error
//! enum.
//!
//! The projection's own layer sits under it, private to the crate: model holds
//! the static vocabulary, patch the content-line grammar a fold-back writes
//! through, datetime the dates, and line and util the comment alignment and
//! the TOML rendering.
//!
//! The `cli` feature adds the cli module: the three verbs, how each resolves
//! its source, and the editor round trip. The binary above it is wiring only,
//! and says so in its own header.

extern crate alloc;
#[cfg(feature = "cli")]
extern crate std;

#[cfg(feature = "cli")]
pub mod cli;
pub mod error;
pub mod merge;
pub mod template;
pub mod vcard;
