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
//! A body is read once. [`vcard::TcardCards::parse`] turns a whole stream into
//! vcard-rs's byte-faithful syntax tree, and every verb walks that one tree,
//! so no value passes through a second reader that might normalise it on the
//! way in, where no test comparing the two could see it.
//!
//! [`template::TcardTemplate::project`] walks that tree against the static field
//! table and writes the form. A single card flattens at the document root, two
//! or more become `[[card]]` blocks, and what the table does not model is not
//! shown.
//!
//! [`template::TcardTemplate::apply`] folds an edited form back onto that same
//! tree, patching a modelled line rather than rebuilding it: only the value the
//! document moved is written anew, and the rest of the line stays the card's
//! own bytes, its group and the parameters the form never showed included.
//!
//! That is why a fold-back is a method on the template the form came from,
//! rather than a function over a document: the TOML is an editing affordance,
//! never an interchange format.
//!
//! The boundary is that tCard owns the content-line grammar it patches while
//! vcard-rs owns the card's structure and its bytes. The projection reads
//! through that same grammar, so what the form shows and what a fold-back
//! writes agree by construction.
//!
//! ## The modelled vocabulary
//!
//! A static table in model names every property the form shows. Each entry
//! decouples the friendly TOML key from the vCard property behind it, so
//! `address` can read well without `ADR` moving, and carries the requirement,
//! the inline hint and the kind that drives both directions.
//!
//! A kind is one of five. A scalar is one value, a list joins several on a
//! separator, a structured value expands into named components instead of
//! bare semicolons, a typed property repeats as sections carrying an optional
//! `TYPE`, and a typed structured one is both.
//!
//! A structured component can be deprecated, which RFC 6350 does to `ADR`'s
//! post office box and extended address: hidden from a vCard 4.0 scaffold,
//! flagged in older versions, and on apply written back from the card's own
//! line into its slot, since hiding a component is no licence to drop it.
//!
//! `UID` and `VERSION` are deliberately absent. They are app-managed, seeded
//! for a new card and preserved for every other one.
//!
//! ## The document's layout
//!
//! TOML attributes every bare key after a table or array header to that
//! table, so the scalar and list fields are written first as one block and
//! the sectioned properties follow it. That order is the format's rule rather
//! than a preference.
//!
//! Inline hints within a block share one column, reached with tabs a stop
//! past the widest hinted line, so filling a value shifts the comments as
//! little as it can.
//!
//! ## The merge
//!
//! [`merge::TcardMerge`] reconciles a local and a remote card against the base
//! they both came from, then renders the outcome through the same projection,
//! so a merge is read and edited in the form everything else is.
//!
//! What the merge settled by itself is said in a comment at the head of that
//! document. What it could not settle is written once per side, each line
//! naming its side, which makes the same TOML key appear twice.
//!
//! TOML forbids duplicate keys, so an undecided document does not parse.
//! [`merge::TcardMerged::apply`] catches that refusal and names the field left undecided
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
//! the static vocabulary, patch the content-line grammar a fold-back reads and
//! writes through, toml the TOML side of the same, datetime the dates and line
//! the blocks whose hints share a column.
//!
//! The merge splits the same way: choice turns a collision into the key it
//! contests, document writes that key into the projection, and note says what
//! carries no key at all.
//!
//! The `cli` feature adds the cli module, one module per verb over the shared
//! arguments and the editor round trip. The binary above it is wiring only,
//! and says so in its own header.
//!
//! ## The golden fixture database
//!
//! The tests/data directory is a regression database of real and crafted
//! cards, checked by tests/fixtures.rs. Each `<name>.<mode>.toml` is the
//! expected projection of `<name>.vcf`, round-tripped byte-exact unless a
//! `<name>.lossy` marker says the source is not in a fold-back's own form.
//!
//! The imported cards come from the ez-vcard corpus (Gmail, Evolution, MS
//! Outlook 2.1) and the calcard parser's own, alongside the RFC 6350 example
//! and an Apple-style export. Every one of those is lossy; clean, folded and
//! two_cards are crafted to round trip byte-exact, folded pinning the layout a
//! card keeps: its folds and the blank line between two of its properties.
//!
//! A real-world export is the most valuable case, so adding one is the
//! fastest way to turn a bug report into a regression test. CONTRIBUTING.md
//! carries the steps.
//!
//! ## Known limitations
//!
//! These are deliberate or pending, and they are what the lossy markers
//! record. Structured components are joined with their trailing empties
//! dropped, so `N:Doe;John;;;` is re-emitted `N:Doe;John`.
//!
//! A line the document moved goes back out unfolded. vcard-rs records where a
//! line was folded as offsets into that line's own bytes, so an edit changing
//! its length invalidates them and the layout is dropped rather than applied
//! in the wrong places. A line the document left alone keeps its folds, its
//! blank lines and its quoted-printable soft breaks.
//!
//! A birthday or anniversary is read as the card wrote it and written back in
//! RFC 6350 basic form, so an extended date comes back basic. A text value is
//! written back escaped per RFC 6350 section 3.4, so an unescaped comma comes
//! back escaped and a needless escape comes back without it.
//!
//! A line whose `TYPE` the document changed is rewritten with the document's
//! spelling and its type parameters gathered into one. A line whose types are
//! unchanged keeps its own bytes, however they were spelled.

extern crate alloc;
#[cfg(feature = "cli")]
extern crate std;

#[cfg(feature = "cli")]
pub mod cli;
pub mod error;
pub mod merge;
pub mod template;
pub mod vcard;
