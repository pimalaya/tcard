//! A format-preserving vCard editor, the `toml_edit` analog for vCard.
//!
//! calcard is a normalizing reader/writer: re-serializing churns line folding,
//! parameter casing (`TYPE=work` becomes `TYPE=WORK`) and property order even
//! where nothing changed. This editor instead keeps every content line's
//! original bytes and re-renders only the lines a caller mutates ([`tree`]), so
//! editing one property yields a minimal diff. vCard is line-oriented with a
//! single wrinkle, line folding (`parse`, `render`).  It is calcard-independent
//! (no_std, alloc only) and could move to its own crate later, shared with the
//! iCalendar sibling. The core invariant is `Card::parse(s).to_string() == s`
//! for any input.

mod parse;
mod render;
pub mod tree;
