//! A format-preserving vCard editor, the `toml_edit` analog for vCard.
//!
//! calcard is a normalizing reader/writer: re-serializing churns line folding,
//! parameter casing (`TYPE=work` becomes `TYPE=WORK`) and property order even
//! where nothing changed. This module instead keeps every content line's
//! original bytes and re-renders only the lines a caller mutates, so editing
//! one property yields a minimal diff.
//!
//! vCard is line-oriented (`NAME;PARAMS:VALUE`) with a single wrinkle, line
//! folding, so the layer is small. It is deliberately calcard-independent
//! (no_std, alloc only) and could move to its own crate later, shared with the
//! iCalendar sibling.
//!
//! The core invariant is round-trip identity: `parse(s).to_string() == s` for
//! any input.

use core::fmt;

use alloc::{borrow::ToOwned, string::String, vec::Vec};

/// The longest octet length of a physical line before folding kicks in,
/// per RFC 6350 section 3.2; mirrors calcard's writer.
const MAX_LINE_OCTETS: usize = 75;

/// A parsed vCard stream as a format-preserving tree.
pub struct Card {
    items: Vec<Item>,
}

/// Parse a vCard stream into a format-preserving tree. Infallible:
/// anything unrecognized is kept verbatim so output can round-trip.
pub fn parse(src: &str) -> Card {
    let logicals = unfold(src);

    let mut items = Vec::new();
    let mut cursor = 0;

    while cursor < logicals.len() {
        let (mut block, stop) = parse_block(&logicals, &mut cursor);
        items.append(&mut block);

        // A stray END with no open component is kept as-is, not dropped.
        if let Stop::End(end_raw) = stop {
            items.push(Item::Raw(end_raw));
        }
    }

    Card { items }
}

impl Card {
    /// The first component of the given type, searched depth-first and
    /// case-insensitively (`card.component_mut("VCARD")`).
    pub fn component_mut(&mut self, ty: &str) -> Option<&mut Component> {
        find_component_mut(&mut self.items, ty)
    }

    /// Every top-level component of the given type, in document order
    /// (e.g. each `VCARD` of a multi-card file), for reading.
    pub fn components(&self, ty: &str) -> impl Iterator<Item = &Component> {
        components_of(&self.items, ty)
    }

    /// Every top-level component of the given type, in document order,
    /// for in-place mutation. Use `.nth(i)` to address one occurrence.
    pub fn components_mut(&mut self, ty: &str) -> impl Iterator<Item = &mut Component> {
        components_of_mut(&mut self.items, ty)
    }
}

impl fmt::Display for Card {
    /// Concatenate every node's raw bytes; only mutated or inserted
    /// properties were re-rendered, everything else is original.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            item.fmt(f)?;
        }

        Ok(())
    }
}

/// A `BEGIN`/`END` component (`VCARD`) and its contents.
pub struct Component {
    name: String,
    begin_raw: String,
    items: Vec<Item>,
    end_raw: String,
}

impl Component {
    /// The first descendant component of the given type, searched
    /// depth-first and case-insensitively.
    pub fn component_mut(&mut self, ty: &str) -> Option<&mut Component> {
        find_component_mut(&mut self.items, ty)
    }

    /// This component's direct child components of the given type, in
    /// document order, for reading.
    pub fn components(&self, ty: &str) -> impl Iterator<Item = &Component> {
        components_of(&self.items, ty)
    }

    /// This component's direct child components of the given type, in
    /// document order, for in-place mutation. Use `.nth(i)` to address
    /// one occurrence (e.g. the 2nd `VALARM` of a `VEVENT`).
    pub fn components_mut(&mut self, ty: &str) -> impl Iterator<Item = &mut Component> {
        components_of_mut(&mut self.items, ty)
    }

    /// This component's own direct properties matching `name`, in
    /// document order, for reading.
    pub fn properties(&self, name: &str) -> impl Iterator<Item = &Property> {
        properties_of(&self.items, name)
    }

    /// This component's own direct properties matching `name`, in
    /// document order, for in-place mutation. Use `.nth(i)` then
    /// [`Property::set`] to rewrite one occurrence (e.g. the 3rd `EMAIL`)
    /// without restating the whole group.
    pub fn properties_mut(&mut self, name: &str) -> impl Iterator<Item = &mut Property> {
        properties_of_mut(&mut self.items, name)
    }

    /// The logical content lines of this component's own direct
    /// properties matching `name` (no enclosing component is searched).
    pub fn get_all(&self, name: &str) -> Vec<&str> {
        self.properties(name).map(Property::logical).collect()
    }

    /// Make this component's direct properties named `name` exactly equal
    /// `lines` (full content lines without an end of line, e.g.
    /// `"FN:Jane Roe"`), with a minimal diff.
    ///
    /// Existing properties are reused in order: where the desired line
    /// already matches, the original bytes are left untouched; otherwise
    /// the line is re-rendered. Surplus properties are removed and missing
    /// ones inserted after the last direct property, before any nested
    /// component. `lines == []` removes every matching property.
    pub fn set_all(&mut self, name: &str, lines: &[String]) {
        let upper = name.to_uppercase();
        let eol = eol_of(&self.begin_raw).to_owned();

        let positions: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| match item {
                Item::Property(property) if property.name == upper => Some(index),
                _ => None,
            })
            .collect();

        // Reuse existing slots positionally, re-rendering only on change.
        let reuse = positions.len().min(lines.len());
        for slot in 0..reuse {
            if let Item::Property(property) = &mut self.items[positions[slot]]
                && property.logical != lines[slot]
            {
                property.logical = lines[slot].clone();
                property.raw = render(&lines[slot], &eol);
            }
        }

        if lines.len() < positions.len() {
            // Drop surplus from the back so earlier indices stay valid.
            for slot in (lines.len()..positions.len()).rev() {
                self.items.remove(positions[slot]);
            }
        } else if lines.len() > positions.len() {
            let at = self.insertion_point();
            let extras = lines[positions.len()..].iter().map(|line| {
                Item::Property(Property {
                    name: upper.clone(),
                    logical: line.clone(),
                    raw: render(line, &eol),
                })
            });

            let tail = self.items.split_off(at);
            self.items.extend(extras);
            self.items.extend(tail);
        }
    }

    /// Remove every direct property matching `name`.
    pub fn remove(&mut self, name: &str) {
        self.set_all(name, &[]);
    }

    /// Where a new property should land: after the last direct property,
    /// else before the first nested component, else at the end (which
    /// sits just before `END`, kept separately in `end_raw`).
    fn insertion_point(&self) -> usize {
        if let Some(last) = self
            .items
            .iter()
            .rposition(|item| matches!(item, Item::Property(_)))
        {
            return last + 1;
        }

        self.items
            .iter()
            .position(|item| matches!(item, Item::Component(_)))
            .unwrap_or(self.items.len())
    }
}

/// One node of a [`Card`]: a property, a nested component, or an
/// unrecognized line (blank line, junk) kept verbatim.
enum Item {
    Property(Property),
    Component(Component),
    Raw(String),
}

impl Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Property(property) => f.write_str(&property.raw),
            Item::Raw(raw) => f.write_str(raw),
            Item::Component(component) => {
                f.write_str(&component.begin_raw)?;
                for item in &component.items {
                    item.fmt(f)?;
                }
                f.write_str(&component.end_raw)
            }
        }
    }
}

/// A single content line (`NAME;PARAMS:VALUE`): unfolded for matching but
/// kept byte-for-byte (folding and end of line included) for output.
pub struct Property {
    name: String,
    logical: String,
    raw: String,
}

impl Property {
    /// The unfolded content line (`NAME;PARAMS:VALUE`), without an end of
    /// line.
    pub fn logical(&self) -> &str {
        &self.logical
    }

    /// Replace this property's content line with `line`, re-rendering
    /// (folding and end of line) only when it differs, so an unchanged
    /// value keeps its original bytes. The line's own end of line is
    /// preserved.
    pub fn set(&mut self, line: &str) {
        if self.logical != line {
            let eol = eol_of(&self.raw).to_owned();
            self.logical = line.to_owned();
            self.raw = render(line, &eol);
        }
    }
}

/// A logical (unfolded) line: its joined content and the exact original
/// bytes of every physical line that made it up.
struct Logical {
    content: String,
    raw: String,
}

/// Why [`parse_block`] returned: it consumed a closing `END` (whose raw
/// bytes it hands back) or reached the end of input.
enum Stop {
    End(String),
    Eof,
}

/// Parse items until the matching `END` or the end of input. `BEGIN`
/// recurses, so an `END` closes the innermost open component.
fn parse_block(logicals: &[Logical], cursor: &mut usize) -> (Vec<Item>, Stop) {
    let mut items = Vec::new();

    while *cursor < logicals.len() {
        let logical = &logicals[*cursor];

        if end_name(&logical.content).is_some() {
            let end_raw = logical.raw.clone();
            *cursor += 1;
            return (items, Stop::End(end_raw));
        }

        if let Some(name) = begin_name(&logical.content) {
            let begin_raw = logical.raw.clone();
            *cursor += 1;

            let (inner, stop) = parse_block(logicals, cursor);
            let end_raw = match stop {
                Stop::End(raw) => raw,
                Stop::Eof => String::new(),
            };

            items.push(Item::Component(Component {
                name,
                begin_raw,
                items: inner,
                end_raw,
            }));
            continue;
        }

        let item = match property_name(&logical.content) {
            Some(name) => Item::Property(Property {
                name,
                logical: logical.content.clone(),
                raw: logical.raw.clone(),
            }),
            None => Item::Raw(logical.raw.clone()),
        };
        items.push(item);
        *cursor += 1;
    }

    (items, Stop::Eof)
}

/// The first component of the given type within `items`, depth-first
/// (pre-order) and case-insensitive.
fn find_component_mut<'a>(items: &'a mut [Item], ty: &str) -> Option<&'a mut Component> {
    for item in items.iter_mut() {
        if let Item::Component(component) = item {
            if component.name.eq_ignore_ascii_case(ty) {
                return Some(component);
            }
            if let Some(found) = find_component_mut(&mut component.items, ty) {
                return Some(found);
            }
        }
    }

    None
}

/// The direct components of `items` matching `ty`, in order.
fn components_of<'a>(items: &'a [Item], ty: &str) -> impl Iterator<Item = &'a Component> {
    let upper = ty.to_uppercase();

    items.iter().filter_map(move |item| match item {
        Item::Component(component) if component.name == upper => Some(component),
        _ => None,
    })
}

/// The direct components of `items` matching `ty`, in order, mutable.
fn components_of_mut<'a>(
    items: &'a mut [Item],
    ty: &str,
) -> impl Iterator<Item = &'a mut Component> {
    let upper = ty.to_uppercase();

    items.iter_mut().filter_map(move |item| match item {
        Item::Component(component) if component.name == upper => Some(component),
        _ => None,
    })
}

/// The direct properties of `items` matching `name`, in order.
fn properties_of<'a>(items: &'a [Item], name: &str) -> impl Iterator<Item = &'a Property> {
    let upper = name.to_uppercase();

    items.iter().filter_map(move |item| match item {
        Item::Property(property) if property.name == upper => Some(property),
        _ => None,
    })
}

/// The direct properties of `items` matching `name`, in order, mutable.
fn properties_of_mut<'a>(
    items: &'a mut [Item],
    name: &str,
) -> impl Iterator<Item = &'a mut Property> {
    let upper = name.to_uppercase();

    items.iter_mut().filter_map(move |item| match item {
        Item::Property(property) if property.name == upper => Some(property),
        _ => None,
    })
}

/// Split `src` into logical lines, joining folded continuations (a
/// physical line starting with a space or tab) onto the previous line
/// while recording the exact original bytes.
fn unfold(src: &str) -> Vec<Logical> {
    let mut logicals: Vec<Logical> = Vec::new();

    for (content, raw) in physical_lines(src) {
        let is_continuation = content.starts_with(' ') || content.starts_with('\t');

        if is_continuation && let Some(last) = logicals.last_mut() {
            // Unfolding drops the CRLF and exactly one leading space.
            last.content.push_str(&content[1..]);
            last.raw.push_str(raw);
            continue;
        }

        logicals.push(Logical {
            content: content.to_owned(),
            raw: raw.to_owned(),
        });
    }

    logicals
}

/// Split `src` into physical lines, yielding for each its content (no end
/// of line) and its raw bytes (the end of line, when present, included).
fn physical_lines(src: &str) -> Vec<(&str, &str)> {
    let mut lines = Vec::new();
    let bytes = src.as_bytes();
    let mut start = 0;

    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }

        let raw = &src[start..=index];
        let content_end = if index > start && bytes[index - 1] == b'\r' {
            index - 1
        } else {
            index
        };
        lines.push((&src[start..content_end], raw));
        start = index + 1;
    }

    if start < bytes.len() {
        lines.push((&src[start..], &src[start..]));
    }

    lines
}

/// Render a content line as vCard bytes: fold at [`MAX_LINE_OCTETS`]
/// octets with a `{eol} ` continuation and terminate with `eol`,
/// mirroring calcard's writer.
fn render(content: &str, eol: &str) -> String {
    let mut out = String::with_capacity(content.len() + eol.len());
    let mut line_len = 0;

    for ch in content.chars() {
        let ch_len = ch.len_utf8();
        if line_len + ch_len > MAX_LINE_OCTETS {
            out.push_str(eol);
            out.push(' ');
            // The continuation space already fills one octet.
            line_len = 1;
        }
        out.push(ch);
        line_len += ch_len;
    }

    out.push_str(eol);
    out
}

/// The end of line of `raw` (its trailing terminator), defaulting to
/// CRLF when there is none.
fn eol_of(raw: &str) -> &str {
    if raw.ends_with("\r\n") {
        "\r\n"
    } else if raw.ends_with('\n') {
        "\n"
    } else {
        "\r\n"
    }
}

/// The property name of a content line: the characters up to the first
/// `;` or `:`, uppercased for matching. `None` for blank or nameless
/// lines.
fn property_name(content: &str) -> Option<String> {
    let end = content.find([';', ':'])?;
    let name = &content[..end];

    if name.is_empty() {
        return None;
    }

    Some(name.to_uppercase())
}

/// The component type of a `BEGIN:<type>` line, uppercased.
fn begin_name(content: &str) -> Option<String> {
    component_name(content, "BEGIN")
}

/// The component type of an `END:<type>` line, uppercased.
fn end_name(content: &str) -> Option<String> {
    component_name(content, "END")
}

/// The type that a `BEGIN`/`END` marker line names, when `content` is
/// such a marker (`marker` is `"BEGIN"` or `"END"`).
fn component_name(content: &str, marker: &str) -> Option<String> {
    if property_name(content)? != marker {
        return None;
    }

    let value = content.split_once(':')?.1.trim();
    Some(value.to_uppercase())
}

#[cfg(test)]
mod tests {
    use alloc::{
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    };

    use super::{Card, parse};

    const SAMPLE: &str = "BEGIN:VCARD\r\n\
        VERSION:4.0\r\n\
        FN:John Doe\r\n\
        N:Doe;John;;;\r\n\
        EMAIL;TYPE=work:john@work.example\r\n\
        EMAIL;TYPE=home:john@home.example\r\n\
        X-CUSTOM;TYPE=weird:keep me verbatim\r\n\
        END:VCARD\r\n";

    fn applied(src: &str, name: &str, lines: &[&str]) -> String {
        let owned: Vec<String> = lines.iter().map(|line| line.to_string()).collect();
        let mut card = parse(src);
        card.component_mut("VCARD").unwrap().set_all(name, &owned);
        card.to_string()
    }

    #[test]
    fn round_trips_verbatim() {
        // CRLF, LF, a folded line, two cards and blank lines.
        let folded =
            "BEGIN:VCARD\r\nNOTE:a very long note that has\r\n  been folded\r\nEND:VCARD\r\n";
        let lf = "BEGIN:VCARD\nFN:x\nEND:VCARD\n";
        let two = "BEGIN:VCARD\r\nFN:a\r\nEND:VCARD\r\nBEGIN:VCARD\r\nFN:b\r\nEND:VCARD\r\n";
        let blanks = "BEGIN:VCARD\r\n\r\nFN:x\r\n\r\nEND:VCARD\r\n";

        for src in [SAMPLE, folded, lf, two, blanks] {
            assert_eq!(parse(src).to_string(), src);
        }
    }

    #[test]
    fn set_all_same_value_is_byte_identical() {
        assert_eq!(applied(SAMPLE, "FN", &["FN:John Doe"]), SAMPLE);
    }

    #[test]
    fn set_all_changes_only_one_line() {
        let out = applied(SAMPLE, "FN", &["FN:Jane Roe"]);
        assert_eq!(out, SAMPLE.replace("FN:John Doe", "FN:Jane Roe"));
    }

    #[test]
    fn set_all_long_value_folds() {
        let long = format!("NOTE:{}", "x".repeat(100));
        let out = applied(SAMPLE, "NOTE", &[&long]);

        // Folded into physical lines no wider than 75 octets, and the
        // rest of the card is left untouched.
        assert!(out.contains("\r\n "));
        for line in out.split("\r\n") {
            assert!(line.len() <= 75, "line too wide: {line:?}");
        }
        assert!(out.contains("FN:John Doe"));
    }

    #[test]
    fn set_all_empty_removes() {
        let out = applied(SAMPLE, "FN", &[]);
        assert!(!out.contains("FN:John Doe"));
        assert_eq!(out, SAMPLE.replace("FN:John Doe\r\n", ""));
    }

    #[test]
    fn set_all_resizes_a_group() {
        assert_eq!(SAMPLE.matches("EMAIL").count(), 2);

        let one = applied(SAMPLE, "EMAIL", &["EMAIL:a@x"]);
        assert_eq!(one.matches("EMAIL").count(), 1);

        let three = applied(&one, "EMAIL", &["EMAIL:a@x", "EMAIL:b@x", "EMAIL:c@x"]);
        assert_eq!(three.matches("EMAIL").count(), 3);
    }

    #[test]
    fn mutation_leaves_unmodeled_untouched() {
        let out = applied(SAMPLE, "FN", &["FN:edited"]);
        assert!(out.contains("FN:edited"));
        assert!(out.contains("X-CUSTOM;TYPE=weird:keep me verbatim"));
    }

    #[test]
    fn get_all_reads_direct_properties() {
        let mut card: Card = parse(SAMPLE);
        let component = card.component_mut("VCARD").unwrap();
        assert_eq!(component.get_all("FN"), vec!["FN:John Doe"]);
        assert_eq!(component.get_all("EMAIL").len(), 2);
    }

    #[test]
    fn properties_mut_edits_one_occurrence() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\n\
            EMAIL:a@x\r\nEMAIL:b@x\r\nEMAIL:c@x\r\nEMAIL:d@x\r\nEMAIL:e@x\r\n\
            END:VCARD\r\n";
        let mut card = parse(src);

        card.component_mut("VCARD")
            .unwrap()
            .properties_mut("EMAIL")
            .nth(2)
            .unwrap()
            .set("EMAIL:c2@x");

        // Only the 3rd EMAIL changes; the other four stay byte-for-byte.
        assert_eq!(card.to_string(), src.replace("EMAIL:c@x", "EMAIL:c2@x"));
    }

    #[test]
    fn property_set_same_value_keeps_bytes() {
        let mut card = parse(SAMPLE);
        card.component_mut("VCARD")
            .unwrap()
            .properties_mut("FN")
            .next()
            .unwrap()
            .set("FN:John Doe");
        assert_eq!(card.to_string(), SAMPLE);
    }

    #[test]
    fn components_mut_reaches_each_card() {
        let src = "BEGIN:VCARD\r\nFN:a\r\nEND:VCARD\r\nBEGIN:VCARD\r\nFN:b\r\nEND:VCARD\r\n";
        let mut card = parse(src);

        card.components_mut("VCARD")
            .nth(1)
            .unwrap()
            .set_all("FN", &["FN:b2".to_string()]);

        // The second card's FN changes; the first is untouched.
        assert_eq!(card.to_string(), src.replace("FN:b", "FN:b2"));
        assert_eq!(card.components("VCARD").count(), 2);
    }
}
