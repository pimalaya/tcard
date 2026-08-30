//! # Value helpers
//!
//! The small conversions projection and apply share: rendering TOML scalars,
//! and reading a content line's value and parameters.
//!
//! A line is read through the same [`crate::template::patch`] grammar that
//! patches it, so what the projection shows and what a fold-back writes agree
//! by construction rather than by round trip.

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};

use toml_edit::{Array, Item, TableLike, Value};

use crate::template::patch;

/// Render a string as a quoted, escaped TOML scalar.
pub fn toml_str(value: &str) -> String {
    Value::from(value).to_string().trim().to_string()
}

/// Render strings as a TOML array.
pub fn toml_array<S: AsRef<str>>(items: &[S]) -> String {
    let mut array = Array::new();

    for item in items {
        array.push(item.as_ref());
    }

    array.to_string().trim().to_string()
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

/// The TOML tables `[[key]]` or an inline array of inline tables addresses.
pub fn tables(item: &Item) -> Vec<&dyn TableLike> {
    if let Some(array) = item.as_array_of_tables() {
        array.iter().map(|table| table as &dyn TableLike).collect()
    } else if let Some(array) = item.as_array() {
        array
            .iter()
            .filter_map(|value| value.as_inline_table())
            .map(|table| table as &dyn TableLike)
            .collect()
    } else {
        Vec::new()
    }
}

/// A line's value as one unescaped string, its commas kept literal.
///
/// A single-valued property is one value however it is punctuated, so a comma
/// inside a URI stays in the value rather than truncating it.
pub fn text(line: &str) -> String {
    unescape(patch::value(line))
}

/// A line's value as the items `sep` separates, each unescaped on its own.
pub fn items(line: &str, sep: char) -> Vec<String> {
    patch::items(patch::value(line), sep)
        .into_iter()
        .map(unescape)
        .collect()
}

/// A line's `TYPE` values, joined the way the projection writes them.
pub fn types(line: &str) -> String {
    patch::types(line).join(",")
}

/// Read named components from a TOML table, escaped and in order.
///
/// Each positional slot is preserved. A component the document does not write
/// is one the version hides (`pobox`, `ext` in vCard 4.0), so it is taken from
/// the line the value came from: hiding one is not licence to drop it.
pub fn read_components(
    table: &dyn TableLike,
    components: &[(&str, Option<&str>, bool)],
    original: Option<&String>,
) -> Vec<String> {
    let held = original
        .map(|line| patch::items(patch::value(line), ';'))
        .unwrap_or_default();

    components
        .iter()
        .enumerate()
        .map(
            |(index, (name, _, _))| match table.get(name).and_then(|item| item.as_str()) {
                Some(value) => escape(value),
                None => held
                    .get(index)
                    .map(|part| (*part).to_owned())
                    .unwrap_or_default(),
            },
        )
        .collect()
}

/// Join structured components with `;`, dropping trailing empties.
pub fn join_components(parts: &[String]) -> String {
    let last = parts
        .iter()
        .rposition(|part| !part.is_empty())
        .map_or(0, |index| index + 1);
    parts[..last].join(";")
}

/// The `type` a TOML table writes, empty when it writes none.
pub fn read_type(table: &dyn TableLike) -> &str {
    table
        .get("type")
        .and_then(|item| item.as_str())
        .unwrap_or_default()
}
