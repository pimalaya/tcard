//! # Content lines
//!
//! The grammar a projected value is read through and patched with: a line
//! taken apart into its name, its parameters and its value, and the RFC 6350
//! section 3.4 escapes that value carries.
//!
//! The projection shows a modelled property's value and its `TYPE`, and
//! nothing else. Rebuilding the line out of the document alone would therefore
//! drop every other parameter (`PREF`, `PID`, `LANGUAGE`) and every component
//! the version hides, so the line is patched instead.

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// One content line, without its group prefix and its line ending.
pub struct Content<'a>(pub &'a str);

impl<'a> Content<'a> {
    /// The name and parameters of the line, its value excluded.
    pub fn prefix(&self) -> &'a str {
        &self.0[..colon(self.0)]
    }

    /// The value of the line, its name and parameters excluded.
    pub fn value(&self) -> &'a str {
        self.0.get(colon(self.0) + 1..).unwrap_or_default()
    }

    /// The value as one unescaped string, its commas kept literal.
    ///
    /// A single-valued property is one value however it is punctuated, so a
    /// comma inside a URI stays in the value rather than truncating it.
    pub fn text(&self) -> String {
        unescape(self.value())
    }

    /// The value as the items `sep` separates, each unescaped on its own.
    pub fn texts(&self, sep: char) -> Vec<String> {
        items(self.value(), sep).into_iter().map(unescape).collect()
    }

    /// The value as the items `sep` separates, as the card wrote them.
    pub fn items(&self, sep: char) -> Vec<&'a str> {
        items(self.value(), sep)
    }

    /// The `TYPE` values the line carries, in source order.
    ///
    /// A bare vCard 2.1 type parameter (`;WORK`) counts. This is the grammar
    /// [`Content::rewritten`] writes a type back with, so what the projection
    /// shows and what a fold-back compares against agree.
    pub fn types(&self) -> Vec<&'a str> {
        split(self.prefix(), ';')
            .iter()
            .skip(1)
            .filter_map(|param| type_of(param))
            .flatten()
            .collect()
    }

    /// This line's prefix, with its `TYPE` replaced by the document's.
    ///
    /// A `None` `types` leaves the parameters as they were, the projection
    /// showing no type, and so does one spelling the same types the line
    /// already carries, which is what lets an untouched line keep its bytes.
    pub fn rewritten(&self, types: Option<&str>) -> String {
        let prefix = self.prefix();
        let params = split(prefix, ';');

        let Some(types) = types.filter(|types| !carries(&params, types)) else {
            return prefix.to_owned();
        };

        let mut out = params
            .first()
            .map(|name| (*name).to_string())
            .unwrap_or_default();

        for param in params
            .iter()
            .skip(1)
            .filter(|param| type_of(param).is_none())
        {
            out.push(';');
            out.push_str(param);
        }

        if !types.is_empty() {
            out.push_str(";TYPE=");
            out.push_str(types);
        }

        out
    }
}

/// The prefix a line the card does not hold yet carries.
///
/// Its bare property name, and the types the document writes for it.
pub fn named(name: &str, types: Option<&str>) -> String {
    match types.filter(|types| !types.is_empty()) {
        Some(types) => format!("{name};TYPE={types}"),
        None => name.to_owned(),
    }
}

/// Escape a vCard text value per RFC 6350 section 3.4.
pub fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            ',' => out.push_str("\\,"),
            ';' => out.push_str("\\;"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }

    out
}

/// Undo that escaping, the inverse of [`escape`].
///
/// What a card wrote as `\,` is a comma, and either spelling of an escaped
/// newline is one.
pub fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(next) => out.push(next),
            None => out.push('\\'),
        }
    }

    out
}

/// Split on every `sep` outside a quoted parameter value.
pub fn split(text: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut quoted = false;
    let mut start = 0;

    for (at, ch) in text.char_indices() {
        match ch {
            '"' => quoted = !quoted,
            _ if ch == sep && !quoted => {
                out.push(&text[start..at]);
                start = at + ch.len_utf8();
            }
            _ => {}
        }
    }

    out.push(&text[start..]);
    out
}

/// Split a value into the items `sep` separates.
///
/// An escaped separator (RFC 6350 section 3.4) belongs to the item it sits in.
pub fn items(value: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut escaped = false;
    let mut start = 0;

    for (at, ch) in value.char_indices() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == sep {
            out.push(&value[start..at]);
            start = at + ch.len_utf8();
        }
    }

    out.push(&value[start..]);
    out
}

/// Where the colon ending a line's name and parameters sits.
///
/// One inside a quoted parameter value (`GEO="geo:1,2"`) does not count.
fn colon(line: &str) -> usize {
    let mut quoted = false;

    for (at, ch) in line.char_indices() {
        match ch {
            '"' => quoted = !quoted,
            ':' if !quoted => return at,
            _ => {}
        }
    }

    line.len()
}

/// The values a parameter carries when it is a `TYPE`.
///
/// A bare vCard 2.1 type parameter (`;WORK`) counts, and any other parameter
/// gives `None`.
fn type_of(param: &str) -> Option<Vec<&str>> {
    let Some((name, values)) = param.split_once('=') else {
        return Some(vec![param]);
    };

    name.eq_ignore_ascii_case("TYPE").then(|| {
        split(values, ',')
            .into_iter()
            .flat_map(|value| unquote(value).split(','))
            .collect()
    })
}

/// Whether a line's parameters already spell the types the document writes.
///
/// vCard compares parameter values case-insensitively, so the spellings may
/// differ.
fn carries(params: &[&str], types: &str) -> bool {
    let held: Vec<&str> = params
        .iter()
        .skip(1)
        .filter_map(|param| type_of(param))
        .flatten()
        .collect();
    let written: Vec<&str> = match types.is_empty() {
        true => Vec::new(),
        false => types.split(',').collect(),
    };

    held.len() == written.len()
        && held
            .iter()
            .zip(&written)
            .all(|(held, written)| held.eq_ignore_ascii_case(written))
}

/// Strip the quotes a parameter value may be written in.
fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec};

    use crate::template::patch::{Content, named};

    #[test]
    fn a_quoted_parameter_holds_its_own_colon() {
        let line = Content("ADR;GEO=\"geo:1,2\";LABEL=\"x\":;;S;L;;;");

        assert_eq!(line.prefix(), "ADR;GEO=\"geo:1,2\";LABEL=\"x\"");
        assert_eq!(line.value(), ";;S;L;;;");
    }

    /// A quoted list, one parameter per type and the bare vCard 2.1 form all
    /// spell the same types, whatever their case.
    #[test]
    fn an_unchanged_type_keeps_the_original_bytes() {
        for original in [
            "TEL;VALUE=uri;TYPE=\"work,voice\";PREF=1:x",
            "TEL;VALUE=uri;type=WORK;type=Voice;PREF=1:x",
            "TEL;VALUE=uri;WORK;VOICE;PREF=1:x",
        ] {
            let line = Content(original);

            assert_eq!(line.rewritten(Some("work,voice")), line.prefix());
        }
    }

    #[test]
    fn a_changed_type_keeps_every_other_parameter() {
        let line = Content("EMAIL;PREF=1;TYPE=work;PID=1.1:a@x");

        assert_eq!(
            line.rewritten(Some("home")),
            "EMAIL;PREF=1;PID=1.1;TYPE=home"
        );
        assert_eq!(line.rewritten(Some("")), "EMAIL;PREF=1;PID=1.1");
        assert_eq!(line.rewritten(None), "EMAIL;PREF=1;TYPE=work;PID=1.1");
    }

    #[test]
    fn a_new_line_is_named_and_typed_alone() {
        assert_eq!(named("EMAIL", Some("work")), "EMAIL;TYPE=work");
        assert_eq!(named("EMAIL", Some("")), "EMAIL");
        assert_eq!(named("NOTE", None), "NOTE");
    }

    #[test]
    fn an_escaped_separator_belongs_to_its_item() {
        assert_eq!(Content("X:a,b").items(','), vec!["a", "b"]);
        assert_eq!(Content("X:a\\,b").items(','), vec!["a\\,b"]);
        assert_eq!(Content("X:;;S;L").items(';'), vec!["", "", "S", "L"]);
        assert_eq!(Content("X:").items(',').len(), 1);
        assert_eq!(Content("X:a\\\\,b").items(','), vec!["a\\\\", "b"]);
    }

    #[test]
    fn a_line_without_a_value_is_all_prefix() {
        assert_eq!(Content("NOTE").prefix(), "NOTE");
        assert_eq!(Content("NOTE").value(), "".to_string());
    }
}
