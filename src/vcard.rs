//! Thin adapter over [`calcard`]: parse raw vCard text into a [`VCard`].
//!
//! calcard owns the model and the writer; tcard never hand-builds
//! entries. Parse here, project to TOML in [`crate::template`], then let
//! calcard serialize the result back.

use alloc::{format, vec::Vec};

use calcard::{
    Entry, Parser,
    vcard::{VCard, VCardVersion},
};
use log::trace;

use crate::error::{Result, TcardError};

/// The canonical version string (`"4.0"`, ...), for projecting and emitting
/// without allocating.
pub fn version_str(version: VCardVersion) -> &'static str {
    match version {
        VCardVersion::V2_0 => "2.0",
        VCardVersion::V2_1 => "2.1",
        VCardVersion::V3_0 => "3.0",
        VCardVersion::V4_0 => "4.0",
    }
}

/// Parse raw vCard text into a calcard [`VCard`].
///
/// A vCard that parses with trailing issues is still returned (via calcard's
/// `Err(Entry::VCard(_))` recovery path); only a genuine failure or an
/// iCalendar payload is rejected.
pub fn parse(input: &str) -> Result<VCard> {
    match VCard::parse(input) {
        Ok(vcard) | Err(Entry::VCard(vcard)) => {
            trace!(
                "parsed {} entries from {} bytes of vCard",
                vcard.entries.len(),
                input.len(),
            );
            Ok(vcard)
        }
        Err(Entry::ICalendar(_)) => Err(TcardError::NotAVcard),
        Err(Entry::InvalidLine(line)) => Err(TcardError::ParseVcard(line)),
        Err(other) => Err(TcardError::ParseVcard(format!("{other:?}"))),
    }
}

/// Parse a whole vCard stream into every [`VCard`] it contains, in
/// document order (a file may hold several cards). An iCalendar payload
/// or a genuinely invalid line is rejected.
pub fn parse_all(input: &str) -> Result<Vec<VCard>> {
    let mut parser = Parser::new(input);
    let mut cards = Vec::new();

    loop {
        match parser.entry() {
            Entry::VCard(card) => cards.push(card),
            Entry::Eof => break,
            Entry::ICalendar(_) => return Err(TcardError::NotAVcard),
            Entry::InvalidLine(line) => return Err(TcardError::ParseVcard(line)),
            other => return Err(TcardError::ParseVcard(format!("{other:?}"))),
        }
    }

    trace!("parsed {} card(s) from {} bytes", cards.len(), input.len());
    Ok(cards)
}
