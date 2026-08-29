//! Small value helpers shared across projection and apply: TOML rendering,
//! vCard text escaping, and reading calcard entry values.

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};

use calcard::{
    common::IanaString,
    vcard::{VCardEntry, VCardParameterName, VCardParameterValue, VCardValue},
};
use toml_edit::{Array, Item, TableLike, Value};

use crate::template::{datetime::vcard_date, patch};

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

/// The TOML tables addressed by an array-of-tables (`[[key]]`) or an inline
/// array of inline tables.
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

/// First value of an entry as text.
pub fn entry_text(entry: &VCardEntry) -> Option<&str> {
    entry.values.first().and_then(|value| value.as_text())
}

/// First value of a scalar entry as a string. Text-like values pass through;
/// a date or date-time value (`BDAY`, `ANNIVERSARY`) is rendered back to a
/// string, since calcard parses those to a typed value with no text accessor
/// (so a plain `as_text` would silently drop them).
pub fn scalar_text(entry: &VCardEntry) -> String {
    let Some(value) = entry.values.first() else {
        return String::new();
    };

    if let Some(text) = value.as_text() {
        return text.to_owned();
    }

    if let VCardValue::PartialDateTime(date) = value {
        return vcard_date(date);
    }

    value
        .clone()
        .into_text()
        .map(|text| text.into_owned())
        .unwrap_or_default()
}

/// All texts of an entry, flattening structured components.
pub fn entry_texts(entry: &VCardEntry) -> Vec<String> {
    entry.values.iter().flat_map(value_strings).collect()
}

/// Ordered components of a structured entry (`N`, `ADR`, `GENDER`).
pub fn entry_components(entry: &VCardEntry) -> Vec<String> {
    match entry.values.first() {
        Some(VCardValue::Component(parts)) => parts.clone(),
        _ => entry
            .values
            .iter()
            .filter_map(|value| value.as_text().map(str::to_owned))
            .collect(),
    }
}

/// All texts carried by a single value.
fn value_strings(value: &VCardValue) -> Vec<String> {
    match value {
        VCardValue::Component(parts) => parts.clone(),
        other => other.as_text().map(str::to_owned).into_iter().collect(),
    }
}

/// `TYPE` parameter values of an entry, lowercased.
pub fn type_strings(entry: &VCardEntry) -> Vec<String> {
    entry
        .parameters(&VCardParameterName::Type)
        .filter_map(param_text)
        .collect()
}

/// Text form of a parameter value, for `TYPE`.
fn param_text(value: &VCardParameterValue) -> Option<String> {
    match value {
        VCardParameterValue::Text(text) => Some(text.clone()),
        VCardParameterValue::Type(ty) => Some(ty.as_str().to_lowercase()),
        _ => None,
    }
}

/// Read named components from a TOML table, escaped and in order, each
/// positional slot preserved.
///
/// A component the document does not write is one the version hides
/// (`pobox`, `ext` in vCard 4.0), so it is taken from the line the value came
/// from: hiding a component from the form is not licence to drop it.
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
