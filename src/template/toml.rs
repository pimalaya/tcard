//! # TOML side
//!
//! Rendering a value as the TOML the document writes, and reading one back out
//! of an edited table.
//!
//! A component the document does not write is read from the line it came from,
//! through the same [`crate::template::patch`] grammar a fold-back patches,
//! rather than from a second reading of the card.

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};

use toml_edit::{Array, Item, TableLike, Value};

use crate::template::patch::{Content, escape};

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
        .map(|line| Content(line).items(';'))
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

/// The `type` a TOML table writes, empty when it writes none.
pub fn read_type(table: &dyn TableLike) -> &str {
    table
        .get("type")
        .and_then(|item| item.as_str())
        .unwrap_or_default()
}
