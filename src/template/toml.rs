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

use crate::template::{
    model::Component,
    patch::{Content, escape, items, unescape},
};

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
///
/// A component the document left as the card already meant it is taken from
/// the card too. A structured value is one line, so changing any component
/// re-renders every component, and a comma the card used as a separator would
/// otherwise be escaped into the value on the way past.
pub fn read_components(
    table: &dyn TableLike,
    components: &[Component],
    original: Option<&String>,
) -> Vec<String> {
    let held = original
        .map(|line| Content(line).items(';'))
        .unwrap_or_default();

    components
        .iter()
        .enumerate()
        .map(|(index, component)| {
            let held = held.get(index).copied().unwrap_or_default();

            let Some(written) = read_component(table, component) else {
                return held.to_owned();
            };

            match written == component_of(held, component) {
                true => held.to_owned(),
                false => written,
            }
        })
        .collect()
}

/// One component as the document writes it, escaped for the vCard.
///
/// A list joins its values on commas, each escaped on its own, and accepts a
/// bare string as the single value it is: reading stays liberal, and only what
/// a fold-back writes is canonical.
fn read_component(table: &dyn TableLike, component: &Component) -> Option<String> {
    let item = table.get(component.key)?;

    if !component.list {
        return item.as_str().map(escape);
    }

    if let Some(value) = item.as_str() {
        return Some(escape(value));
    }

    let values = item.as_array()?;
    let escaped: Vec<String> = values
        .iter()
        .filter_map(Value::as_str)
        .map(escape)
        .collect();

    Some(escaped.join(","))
}

/// The card's own component, canonicalized the way a fold-back writes one.
///
/// Comparing against this rather than against the raw bytes is what lets a
/// needless escape be dropped while a real separator is kept.
fn component_of(held: &str, component: &Component) -> String {
    match component.list {
        true => items(held, ',')
            .into_iter()
            .map(|value| escape(&unescape(value)))
            .collect::<Vec<_>>()
            .join(","),
        false => escape(&unescape(held)),
    }
}

/// The `type` a TOML table writes, empty when it writes none.
pub fn read_type(table: &dyn TableLike) -> &str {
    table
        .get("type")
        .and_then(|item| item.as_str())
        .unwrap_or_default()
}
