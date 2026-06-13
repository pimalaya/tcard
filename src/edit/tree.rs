//! The format-preserving DOM: a [`Card`] of nodes, each a [`Property`], a
//! nested [`Component`], or a verbatim line.

use core::fmt;

use alloc::{borrow::ToOwned, format, string::String, vec::Vec};

use crate::edit::{
    parse::{Parser, Stop},
    render::{eol_of, render},
};

/// A parsed vCard stream as a format-preserving tree.
pub struct Card {
    items: Nodes,
}

impl Card {
    /// Parse a vCard stream into a format-preserving tree. Infallible:
    /// anything unrecognized is kept verbatim so output can round-trip.
    pub fn parse(src: &str) -> Card {
        let mut parser = Parser::new(src);
        let mut items = Nodes::default();

        while !parser.done() {
            let (block, stop) = parser.block();
            items.append(block);

            // A stray END with no open component is kept as-is, not dropped.
            if let Stop::End(end_raw) = stop {
                items.push(Item::Raw(end_raw));
            }
        }

        Card { items }
    }

    /// The first component of the given type, searched depth-first and
    /// case-insensitively (`card.component_mut("VCARD")`).
    pub fn component_mut(&mut self, ty: &str) -> Option<&mut Component> {
        self.items.find_component_mut(ty)
    }

    /// Every top-level component of the given type, in document order (each
    /// `VCARD` of a multi-card file), for reading.
    pub fn components(&self, ty: &str) -> impl Iterator<Item = &Component> {
        self.items.components(ty)
    }

    /// Every top-level component of the given type, for in-place mutation.
    /// Use `.nth(i)` to address one occurrence.
    pub fn components_mut(&mut self, ty: &str) -> impl Iterator<Item = &mut Component> {
        self.items.components_mut(ty)
    }

    /// Ensure exactly `n` top-level components of the given type: append
    /// empty `BEGIN`/`END` components when there are fewer, or remove the
    /// trailing surplus. Fill new ones via [`Card::components_mut`].
    pub fn set_component_count(&mut self, ty: &str, n: usize) {
        let eol = self.items.eol();
        self.items.set_component_count(ty, n, &eol);
    }
}

impl fmt::Display for Card {
    /// Concatenate every node's bytes; only mutated or inserted properties
    /// were re-rendered, everything else is original.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.items.write(f)
    }
}

/// A `BEGIN`/`END` component (`VCARD`) and its contents.
pub struct Component {
    pub(crate) name: String,
    pub(crate) begin_raw: String,
    pub(crate) items: Nodes,
    pub(crate) end_raw: String,
}

impl Component {
    /// The first descendant component of the given type, depth-first and
    /// case-insensitive.
    pub fn component_mut(&mut self, ty: &str) -> Option<&mut Component> {
        self.items.find_component_mut(ty)
    }

    /// This component's direct child components of the given type.
    pub fn components(&self, ty: &str) -> impl Iterator<Item = &Component> {
        self.items.components(ty)
    }

    /// This component's direct child components of the given type, for
    /// in-place mutation. Use `.nth(i)` to address one occurrence.
    pub fn components_mut(&mut self, ty: &str) -> impl Iterator<Item = &mut Component> {
        self.items.components_mut(ty)
    }

    /// This component's own direct properties matching `name`.
    pub fn properties(&self, name: &str) -> impl Iterator<Item = &Property> {
        self.items.properties(name)
    }

    /// This component's own direct properties matching `name`, for in-place
    /// mutation. Use `.nth(i)` then [`Property::set`] to rewrite one.
    pub fn properties_mut(&mut self, name: &str) -> impl Iterator<Item = &mut Property> {
        self.items.properties_mut(name)
    }

    /// The logical content lines of this component's own direct properties
    /// matching `name`.
    pub fn get_all(&self, name: &str) -> Vec<&str> {
        self.properties(name).map(Property::logical).collect()
    }

    /// Make this component's direct properties named `name` exactly equal
    /// `lines` (content lines without an end of line), with a minimal diff:
    /// unchanged lines keep their bytes, surplus is dropped, missing lines
    /// are inserted after the last property. `lines == []` removes them all.
    pub fn set_all(&mut self, name: &str, lines: &[String]) {
        let eol = eol_of(&self.begin_raw).to_owned();
        self.items.set_properties(name, lines, &eol);
    }

    /// Remove every direct property matching `name`.
    pub fn remove(&mut self, name: &str) {
        self.set_all(name, &[]);
    }

    fn write(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.begin_raw)?;
        self.items.write(f)?;
        f.write_str(&self.end_raw)
    }
}

/// A single content line (`NAME;PARAMS:VALUE`): unfolded for matching but
/// kept byte-for-byte (folding and end of line included) for output.
pub struct Property {
    pub(crate) name: String,
    pub(crate) logical: String,
    pub(crate) raw: String,
}

impl Property {
    /// The unfolded content line (`NAME;PARAMS:VALUE`), without end of line.
    pub fn logical(&self) -> &str {
        &self.logical
    }

    /// Replace this property's content line, re-rendering only when it
    /// differs so an unchanged value keeps its original bytes.
    pub fn set(&mut self, line: &str) {
        if self.logical != line {
            let eol = eol_of(&self.raw).to_owned();
            self.logical = line.to_owned();
            self.raw = render(line, &eol);
        }
    }
}

/// One node of the tree: a property, a nested component, or an unrecognized
/// line (blank line, junk) kept verbatim.
pub(crate) enum Item {
    Property(Property),
    Component(Component),
    Raw(String),
}

impl Item {
    fn write(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Property(property) => f.write_str(&property.raw),
            Item::Raw(raw) => f.write_str(raw),
            Item::Component(component) => component.write(f),
        }
    }
}

/// An ordered list of [`Item`]s, shared by the card root and every
/// component, with the navigation and minimal-diff mutation primitives.
#[derive(Default)]
pub(crate) struct Nodes(Vec<Item>);

impl Nodes {
    /// Append a node.
    pub(crate) fn push(&mut self, item: Item) {
        self.0.push(item);
    }

    /// Move every node of `other` onto the end of this list.
    fn append(&mut self, mut other: Nodes) {
        self.0.append(&mut other.0);
    }

    /// The first component matching `ty`, depth-first and case-insensitive.
    fn find_component_mut(&mut self, ty: &str) -> Option<&mut Component> {
        for item in self.0.iter_mut() {
            if let Item::Component(component) = item {
                if component.name.eq_ignore_ascii_case(ty) {
                    return Some(component);
                }
                if let Some(found) = component.items.find_component_mut(ty) {
                    return Some(found);
                }
            }
        }

        None
    }

    /// The direct components matching `ty`, in order.
    fn components(&self, ty: &str) -> impl Iterator<Item = &Component> {
        let upper = ty.to_uppercase();

        self.0.iter().filter_map(move |item| match item {
            Item::Component(component) if component.name == upper => Some(component),
            _ => None,
        })
    }

    /// The direct components matching `ty`, in order, mutable.
    fn components_mut(&mut self, ty: &str) -> impl Iterator<Item = &mut Component> {
        let upper = ty.to_uppercase();

        self.0.iter_mut().filter_map(move |item| match item {
            Item::Component(component) if component.name == upper => Some(component),
            _ => None,
        })
    }

    /// The direct properties matching `name`, in order.
    fn properties(&self, name: &str) -> impl Iterator<Item = &Property> {
        let upper = name.to_uppercase();

        self.0.iter().filter_map(move |item| match item {
            Item::Property(property) if property.name == upper => Some(property),
            _ => None,
        })
    }

    /// The direct properties matching `name`, in order, mutable.
    fn properties_mut(&mut self, name: &str) -> impl Iterator<Item = &mut Property> {
        let upper = name.to_uppercase();

        self.0.iter_mut().filter_map(move |item| match item {
            Item::Property(property) if property.name == upper => Some(property),
            _ => None,
        })
    }

    /// The minimal-diff body behind [`Component::set_all`].
    fn set_properties(&mut self, name: &str, lines: &[String], eol: &str) {
        let upper = name.to_uppercase();

        let positions: Vec<usize> = self
            .0
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
            if let Item::Property(property) = &mut self.0[positions[slot]]
                && property.logical != lines[slot]
            {
                property.logical = lines[slot].clone();
                property.raw = render(&lines[slot], eol);
            }
        }

        if lines.len() < positions.len() {
            // Drop surplus from the back so earlier indices stay valid.
            for slot in (lines.len()..positions.len()).rev() {
                self.0.remove(positions[slot]);
            }
        } else if lines.len() > positions.len() {
            let at = self.insertion_point();
            let extras = lines[positions.len()..].iter().map(|line| {
                Item::Property(Property {
                    name: upper.clone(),
                    logical: line.clone(),
                    raw: render(line, eol),
                })
            });

            let tail = self.0.split_off(at);
            self.0.extend(extras);
            self.0.extend(tail);
        }
    }

    /// Make the direct components named `ty` number exactly `n`: append
    /// empty `BEGIN`/`END` components rendered with `eol`, or drop surplus.
    fn set_component_count(&mut self, ty: &str, n: usize, eol: &str) {
        let upper = ty.to_uppercase();

        let positions: Vec<usize> = self
            .0
            .iter()
            .enumerate()
            .filter_map(|(index, item)| match item {
                Item::Component(component) if component.name == upper => Some(index),
                _ => None,
            })
            .collect();

        if positions.len() < n {
            for _ in positions.len()..n {
                self.0.push(Item::Component(Component {
                    name: upper.clone(),
                    begin_raw: format!("BEGIN:{upper}{eol}"),
                    items: Nodes::default(),
                    end_raw: format!("END:{upper}{eol}"),
                }));
            }
        } else {
            // Drop surplus from the back so earlier indices stay valid.
            for &index in positions[n..].iter().rev() {
                self.0.remove(index);
            }
        }
    }

    /// The end of line of the first node that carries one, else CRLF.
    fn eol(&self) -> String {
        for item in &self.0 {
            let raw = match item {
                Item::Component(component) => &component.begin_raw,
                Item::Property(property) => &property.raw,
                Item::Raw(raw) => raw,
            };
            if raw.contains('\n') {
                return eol_of(raw).to_owned();
            }
        }

        "\r\n".to_owned()
    }

    /// Where a new property should land: after the last direct property,
    /// else before the first nested component, else at the end.
    fn insertion_point(&self) -> usize {
        if let Some(last) = self
            .0
            .iter()
            .rposition(|item| matches!(item, Item::Property(_)))
        {
            return last + 1;
        }

        self.0
            .iter()
            .position(|item| matches!(item, Item::Component(_)))
            .unwrap_or(self.0.len())
    }

    fn write(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.0 {
            item.write(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    };

    use super::Card;

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
        let mut card = Card::parse(src);
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
            assert_eq!(Card::parse(src).to_string(), src);
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

        // Folded into physical lines no wider than 75 octets, and the rest
        // of the card is left untouched.
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
        let mut card = Card::parse(SAMPLE);
        let component = card.component_mut("VCARD").unwrap();
        assert_eq!(component.get_all("FN"), vec!["FN:John Doe"]);
        assert_eq!(component.get_all("EMAIL").len(), 2);
    }

    #[test]
    fn properties_mut_edits_one_occurrence() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\n\
            EMAIL:a@x\r\nEMAIL:b@x\r\nEMAIL:c@x\r\nEMAIL:d@x\r\nEMAIL:e@x\r\n\
            END:VCARD\r\n";
        let mut card = Card::parse(src);

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
        let mut card = Card::parse(SAMPLE);
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
        let mut card = Card::parse(src);

        card.components_mut("VCARD")
            .nth(1)
            .unwrap()
            .set_all("FN", &["FN:b2".to_string()]);

        // The second card's FN changes; the first is untouched.
        assert_eq!(card.to_string(), src.replace("FN:b", "FN:b2"));
        assert_eq!(card.components("VCARD").count(), 2);
    }
}
