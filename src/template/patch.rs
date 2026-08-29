//! # Patching a content line
//!
//! Taking apart the line a projected value came from, so folding the
//! document back rewrites only what the document writes.
//!
//! The projection shows a modelled property's value and its `TYPE`, and
//! nothing else. Rebuilding the line out of the document alone would
//! therefore drop every other parameter (`PREF`, `PID`, `LANGUAGE`) and
//! every component the version hides, so the line is patched instead.

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// The name and parameters of a content line, its value excluded.
pub fn prefix(line: &str) -> &str {
    &line[..colon(line)]
}

/// The value of a content line, its name and parameters excluded.
pub fn value(line: &str) -> &str {
    line.get(colon(line) + 1..).unwrap_or_default()
}

/// The prefix a folded-back line carries: the bare property name when the
/// line is new, else the original's name and parameters with its `TYPE`
/// replaced by the one the document writes.
///
/// `types` is `None` for a property whose projection shows no type, which
/// leaves the original's parameters exactly as they were.
pub fn rewritten(original: Option<&str>, name: &str, types: Option<&str>) -> String {
    let Some(original) = original else {
        return match types.filter(|types| !types.is_empty()) {
            Some(types) => format!("{name};TYPE={types}"),
            None => name.to_owned(),
        };
    };

    let prefix = prefix(original);
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

/// Split a value into the items `sep` separates, an escaped separator (RFC
/// 6350 section 3.4) belonging to the item it sits in.
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

/// Where the colon ending a line's name and parameters sits, one inside a
/// quoted parameter value (`GEO="geo:1,2"`) not counting.
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

/// Split on every `sep` outside a quoted parameter value.
fn split(text: &str, sep: char) -> Vec<&str> {
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

/// The values a parameter carries when it is a `TYPE`, a bare vCard 2.1 type
/// parameter (`;WORK`) included; `None` for any other parameter.
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

/// Whether a line's parameters already spell exactly the types the document
/// writes, which is what lets an unchanged line keep its bytes. vCard
/// compares parameter values case-insensitively, so the spellings may differ.
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

    #[test]
    fn a_quoted_parameter_holds_its_own_colon() {
        let line = "ADR;GEO=\"geo:1,2\";LABEL=\"x\":;;S;L;;;";

        assert_eq!(super::prefix(line), "ADR;GEO=\"geo:1,2\";LABEL=\"x\"");
        assert_eq!(super::value(line), ";;S;L;;;");
    }

    #[test]
    fn an_unchanged_type_keeps_the_original_bytes() {
        // A quoted list, one parameter per type, and the bare vCard 2.1 form
        // all spell the same types, whatever their case.
        for original in [
            "TEL;VALUE=uri;TYPE=\"work,voice\";PREF=1:x",
            "TEL;VALUE=uri;type=WORK;type=Voice;PREF=1:x",
            "TEL;VALUE=uri;WORK;VOICE;PREF=1:x",
        ] {
            assert_eq!(
                super::rewritten(Some(original), "TEL", Some("work,voice")),
                super::prefix(original),
            );
        }
    }

    #[test]
    fn a_changed_type_keeps_every_other_parameter() {
        let original = "EMAIL;PREF=1;TYPE=work;PID=1.1:a@x";

        assert_eq!(
            super::rewritten(Some(original), "EMAIL", Some("home")),
            "EMAIL;PREF=1;PID=1.1;TYPE=home",
        );
        assert_eq!(
            super::rewritten(Some(original), "EMAIL", Some("")),
            "EMAIL;PREF=1;PID=1.1",
        );
        assert_eq!(
            super::rewritten(Some(original), "EMAIL", None),
            "EMAIL;PREF=1;TYPE=work;PID=1.1",
        );
    }

    #[test]
    fn a_new_line_is_named_and_typed_alone() {
        assert_eq!(
            super::rewritten(None, "EMAIL", Some("work")),
            "EMAIL;TYPE=work"
        );
        assert_eq!(super::rewritten(None, "EMAIL", Some("")), "EMAIL");
        assert_eq!(super::rewritten(None, "NOTE", None), "NOTE");
    }

    #[test]
    fn an_escaped_separator_belongs_to_its_item() {
        assert_eq!(super::items("a,b", ','), vec!["a", "b"]);
        assert_eq!(super::items("a\\,b", ','), vec!["a\\,b"]);
        assert_eq!(super::items(";;S;L", ';'), vec!["", "", "S", "L"]);
        assert_eq!(super::items("", ',').len(), 1);
        assert_eq!(super::items("a\\\\,b", ','), vec!["a\\\\", "b"]);
    }

    #[test]
    fn a_line_without_a_value_is_all_prefix() {
        assert_eq!(super::prefix("NOTE"), "NOTE");
        assert_eq!(super::value("NOTE"), "".to_string());
    }
}
