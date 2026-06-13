//! Thin adapter over [`calcard`]: parse raw vCard text into a [`VCard`].
//!
//! calcard owns the model and the writer; tcard never hand-builds
//! entries. Parse here, project to TOML in [`crate::template`], then let
//! calcard serialize the result back.

use alloc::format;

use calcard::{
    Entry,
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
