//! # Media values
//!
//! A property whose value is a picture, a sound or a key, told apart as bytes
//! the card carries or a reference to bytes elsewhere.
//!
//! Inline bytes are base64 nobody reads, so the form shows their media type
//! and their SHA-256 instead, and folds an unchanged digest back as the line
//! it came from. A `file` the document names is inlined at the card's
//! version, which is the only way bytes enter a card through the form.

use alloc::{borrow::ToOwned, format, string::String, vec::Vec};
use core::fmt::Write;

use base64::{
    Engine,
    alphabet::STANDARD,
    engine::{
        DecodePaddingMode,
        general_purpose::{GeneralPurpose, GeneralPurposeConfig},
    },
};
use sha2::{Digest, Sha256};
use vcard::version::VcardVersion;

use crate::template::patch::{Content, unescape};

/// Base64 as vCard writes it: padded on the way out, either way on the way in.
const BASE64: GeneralPurpose = GeneralPurpose::new(
    &STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

/// The parameters describing how a value is encoded rather than what it is.
///
/// A line changing between inline and referenced drops them, `TYPE` included
/// before vCard 4.0, where it names the format.
const ENCODING_PARAMS: &[&str] = &["ENCODING", "VALUE", "MEDIATYPE", "TYPE"];

/// The formats a media property holds, as vCard 2.1 and 3.0 name them.
///
/// Those versions write a bare `TYPE=JPEG` rather than a media type, so the
/// property says which top-level type it belongs to, and which names do not
/// spell their subtype.
pub struct MediaFormat {
    /// The top-level media type a bare `TYPE` falls under.
    pub top: &'static str,
    /// Legacy `TYPE` names and the media types they stand for.
    pub aliases: &'static [(&'static str, &'static str)],
    /// Whether the value may be text rather than a URI (`KEY`).
    pub text: bool,
}

impl MediaFormat {
    /// The media type a legacy `TYPE` names.
    fn mediatype(&self, ty: &str) -> String {
        match self
            .aliases
            .iter()
            .find(|(alias, _)| alias.eq_ignore_ascii_case(ty))
        {
            Some((_, mediatype)) => (*mediatype).to_owned(),
            None if ty.contains('/') => ty.to_ascii_lowercase(),
            None => format!("{}/{}", self.top, ty.to_ascii_lowercase()),
        }
    }

    /// The legacy `TYPE` a media type is written as, `None` for one outside
    /// this property's formats.
    fn legacy(&self, mediatype: &str) -> Option<String> {
        if let Some((alias, _)) = self.aliases.iter().find(|(_, of)| *of == mediatype) {
            return Some((*alias).to_owned());
        }

        let subtype = mediatype.strip_prefix(self.top)?.strip_prefix('/')?;
        Some(subtype.to_ascii_uppercase())
    }

    /// The media type a legacy line declares, its encoding aside.
    fn declared(&self, line: &Content<'_>) -> Option<String> {
        if let Some(mediatype) = line.param("MEDIATYPE") {
            return Some(mediatype.to_ascii_lowercase());
        }

        line.types()
            .into_iter()
            .find(|ty| !ty.eq_ignore_ascii_case("BASE64"))
            .map(|ty| self.mediatype(ty))
    }
}

/// What a media line holds.
pub enum Media<'a> {
    /// Bytes the card carries, named by their digest.
    Embedded {
        /// The media type the line declares, when it declares one.
        mediatype: Option<String>,
        /// The SHA-256 of the decoded bytes, in lowercase hex.
        sha256: String,
    },
    /// A reference, as the card wrote it.
    Uri(&'a str),
}

impl<'a> Media<'a> {
    /// Read a content line as inline bytes or as a reference.
    ///
    /// Inline bytes that do not decode are shown as the reference they then
    /// are, which folds back to the same line.
    pub fn of(line: &'a str, format: &MediaFormat) -> Self {
        let content = Content(line);

        match inline(&content, format) {
            Some((mediatype, bytes)) => Media::Embedded {
                mediatype,
                sha256: sha256(&bytes),
            },
            None => Media::Uri(content.value()),
        }
    }
}

/// A line inlining `bytes`, at `version`.
///
/// The line it replaces keeps every parameter but the ones saying how its old
/// value was encoded.
pub fn inlined(
    name: &str,
    original: Option<&String>,
    bytes: &[u8],
    version: VcardVersion,
    format: &MediaFormat,
) -> String {
    let mediatype = sniff(bytes);
    let data = BASE64.encode(bytes);
    let mut prefix = stripped(name, original, version);

    match version {
        VcardVersion::V4_0 => return format!("{prefix}:data:{mediatype};base64,{data}"),
        VcardVersion::V3_0 => prefix.push_str(";ENCODING=b"),
        VcardVersion::V2_1 => prefix.push_str(";ENCODING=BASE64"),
    }

    if let Some(legacy) = format.legacy(mediatype) {
        prefix.push_str(";TYPE=");
        prefix.push_str(&legacy);
    }

    format!("{prefix}:{data}")
}

/// A line referencing `uri`, at `version`.
///
/// A line already holding a reference is patched; one that held bytes loses
/// the parameters saying how they were encoded.
pub fn referenced(
    name: &str,
    original: Option<&String>,
    uri: &str,
    version: VcardVersion,
    format: &MediaFormat,
) -> String {
    if let Some(line) = original.filter(|line| matches!(Media::of(line, format), Media::Uri(_))) {
        return format!("{}:{uri}", Content(line).prefix());
    }

    let mut prefix = stripped(name, original, version);

    match version {
        VcardVersion::V4_0 => (),
        VcardVersion::V3_0 => prefix.push_str(";VALUE=uri"),
        VcardVersion::V2_1 => prefix.push_str(";VALUE=URL"),
    }

    format!("{prefix}:{uri}")
}

/// The lowercase hex SHA-256 of some bytes, what `sha256sum` prints.
pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}

/// The media type and the bytes a line carries inline, if it does.
///
/// vCard 4.0 writes them as an RFC 2397 `data:` URI, older versions as base64
/// under `ENCODING`, the format in `TYPE`. A key may also be text, which is
/// its own bytes.
fn inline(line: &Content<'_>, format: &MediaFormat) -> Option<(Option<String>, Vec<u8>)> {
    let value = line.value();

    if value
        .get(..5)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:"))
    {
        let (header, data) = value[5..].split_once(',')?;
        let base64 = header
            .rsplit(';')
            .next()
            .is_some_and(|last| last.eq_ignore_ascii_case("base64"));
        let mediatype = header
            .split(';')
            .next()
            .filter(|mediatype| mediatype.contains('/'))
            .map(str::to_ascii_lowercase);

        let bytes = match base64 {
            true => decode(data)?,
            false => percent_decoded(data),
        };

        return Some((mediatype, bytes));
    }

    let encoded = line
        .param("ENCODING")
        .is_some_and(|encoding| matches!(encoding.to_ascii_lowercase().as_str(), "b" | "base64"))
        || line
            .types()
            .iter()
            .any(|ty| ty.eq_ignore_ascii_case("BASE64"));

    if encoded {
        return Some((format.declared(line), decode(value)?));
    }

    // NOTE: RFC 6350 section 6.8.1 lets a key be text, and older exports
    // write an armoured block with no `VALUE` at all, so what carries no URI
    // scheme is text too.
    let text = line
        .param("VALUE")
        .is_some_and(|value| value.eq_ignore_ascii_case("text"))
        || !has_scheme(value);

    match format.text && text {
        true => Some((format.declared(line), unescape(value).into_bytes())),
        false => None,
    }
}

/// Whether a value opens with an RFC 3986 scheme and its colon.
fn has_scheme(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };

    let mut chars = scheme.chars();

    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

/// Base64 decoded, whatever whitespace a vCard 2.1 export left inside it.
fn decode(data: &str) -> Option<Vec<u8>> {
    let data: String = data
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect();
    BASE64.decode(data).ok()
}

/// A `data:` URI's payload written without base64, its `%XX` escapes undone.
fn percent_decoded(data: &str) -> Vec<u8> {
    let bytes = data.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;

    while at < bytes.len() {
        let hex = bytes
            .get(at + 1..at + 3)
            .and_then(|hex| core::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());

        match (bytes[at], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                at += 3;
            }
            (byte, _) => {
                out.push(byte);
                at += 1;
            }
        }
    }

    out
}

/// The media type some bytes open with, `application/octet-stream` when
/// unknown.
fn sniff(bytes: &[u8]) -> &'static str {
    match bytes {
        [0xFF, 0xD8, 0xFF, ..] => "image/jpeg",
        [0x89, b'P', b'N', b'G', ..] => "image/png",
        [b'G', b'I', b'F', b'8', ..] => "image/gif",
        [
            b'R',
            b'I',
            b'F',
            b'F',
            _,
            _,
            _,
            _,
            b'W',
            b'E',
            b'B',
            b'P',
            ..,
        ] => "image/webp",
        [
            b'R',
            b'I',
            b'F',
            b'F',
            _,
            _,
            _,
            _,
            b'W',
            b'A',
            b'V',
            b'E',
            ..,
        ] => "audio/wav",
        [b'O', b'g', b'g', b'S', ..] => "audio/ogg",
        [b'f', b'L', b'a', b'C', ..] => "audio/flac",
        [b'I', b'D', b'3', ..] | [0xFF, 0xF2 | 0xF3 | 0xFB, ..] => "audio/mpeg",
        // NOTE: an armoured block, or a binary public key packet in the old
        // (0x98-0x9B) or the new (0xC6) framing of RFC 9580 section 4.2.
        [
            b'-',
            b'-',
            b'-',
            b'-',
            b'-',
            b'B',
            b'E',
            b'G',
            b'I',
            b'N',
            b' ',
            b'P',
            b'G',
            b'P',
            ..,
        ]
        | [0x98..=0x9B | 0xC6, ..] => "application/pgp-keys",
        [0x30, 0x82, ..] => "application/pkix-cert",
        _ => "application/octet-stream",
    }
}

/// The prefix a media line keeps: its own without its encoding, or the bare
/// property name.
fn stripped(name: &str, original: Option<&String>, version: VcardVersion) -> String {
    // NOTE: vCard 4.0's `TYPE` on a media property is a place, not a format,
    // so it stays.
    let params = match version {
        VcardVersion::V4_0 => &ENCODING_PARAMS[..3],
        _ => ENCODING_PARAMS,
    };

    match original {
        Some(line) => Content(line).without(params),
        None => name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use vcard::version::VcardVersion;

    use crate::template::media::{Media, MediaFormat, inlined, referenced, sha256};

    const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0];

    const IMAGE: MediaFormat = MediaFormat {
        top: "image",
        aliases: &[],
        text: false,
    };

    const KEY: MediaFormat = MediaFormat {
        top: "application",
        aliases: &[("PGP", "application/pgp-keys")],
        text: true,
    };

    fn embedded(line: &str, format: &MediaFormat) -> (Option<String>, String) {
        match Media::of(line, format) {
            Media::Embedded { mediatype, sha256 } => (mediatype, sha256),
            Media::Uri(uri) => panic!("read as uri {uri}"),
        }
    }

    #[test]
    fn every_version_inlines_the_same_bytes() {
        let digest = sha256(JPEG);

        for line in [
            "PHOTO:data:image/jpeg;base64,/9j/4A==",
            "PHOTO;ENCODING=b;TYPE=JPEG:/9j/4A==",
            "PHOTO;JPEG;ENCODING=BASE64:/9j/4A",
            "PHOTO;TYPE=image/jpeg;BASE64:/9j/ 4A==",
        ] {
            assert_eq!(
                embedded(line, &IMAGE),
                (Some("image/jpeg".into()), digest.clone()),
                "{line}"
            );
        }
    }

    #[test]
    fn a_digest_is_what_sha256sum_prints() {
        assert_eq!(
            sha256(b"foo"),
            "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"
        );
        assert_eq!(embedded("PHOTO:data:,f%6Fo", &IMAGE).1, sha256(b"foo"));
    }

    #[test]
    fn a_reference_is_kept_as_written() {
        for line in [
            "PHOTO:https://example.com/a,b.jpg",
            "PHOTO;VALUE=uri:file:///me.jpg",
            "PHOTO;ENCODING=b:not base64!",
            "PHOTO:not a uri",
        ] {
            assert!(matches!(Media::of(line, &IMAGE), Media::Uri(_)), "{line}");
        }

        assert!(matches!(
            Media::of("KEY:https://example.com/key.asc", &KEY),
            Media::Uri(_)
        ));
    }

    #[test]
    fn a_text_key_is_its_own_bytes() {
        let armoured = (
            Some("application/pgp-keys".into()),
            sha256(b"-----BEGIN PGP\n"),
        );

        assert_eq!(embedded("KEY;TYPE=PGP:-----BEGIN PGP\\n", &KEY), armoured);
        assert_eq!(
            embedded("KEY;VALUE=text:ssh-ed25519 AAAA", &KEY).1,
            sha256(b"ssh-ed25519 AAAA")
        );
    }

    #[test]
    fn a_file_is_inlined_at_the_version() {
        let old = String::from("PHOTO;PREF=1;VALUE=uri:https://x");

        assert_eq!(
            inlined("PHOTO", Some(&old), JPEG, VcardVersion::V4_0, &IMAGE),
            "PHOTO;PREF=1:data:image/jpeg;base64,/9j/4A=="
        );
        assert_eq!(
            inlined("PHOTO", Some(&old), JPEG, VcardVersion::V3_0, &IMAGE),
            "PHOTO;PREF=1;ENCODING=b;TYPE=JPEG:/9j/4A=="
        );
        assert_eq!(
            inlined("PHOTO", None, b"?", VcardVersion::V2_1, &IMAGE),
            "PHOTO;ENCODING=BASE64:Pw=="
        );
        assert_eq!(
            inlined("KEY", None, &[0xC6], VcardVersion::V3_0, &KEY),
            "KEY;ENCODING=b;TYPE=PGP:xg=="
        );
    }

    #[test]
    fn a_reference_replacing_bytes_drops_their_encoding() {
        let old = String::from("PHOTO;ENCODING=b;TYPE=JPEG:/9j/4A==");

        assert_eq!(
            referenced("PHOTO", Some(&old), "https://x", VcardVersion::V3_0, &IMAGE),
            "PHOTO;VALUE=uri:https://x"
        );

        let old = String::from("PHOTO;VALUE=uri:https://x");

        assert_eq!(
            referenced("PHOTO", Some(&old), "https://y", VcardVersion::V3_0, &IMAGE),
            "PHOTO;VALUE=uri:https://y"
        );
    }
}
