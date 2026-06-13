//! Parsing a vCard stream into a format-preserving [`Nodes`] tree.

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::edit::tree::{Component, Item, Nodes, Property};

/// A cursor over the unfolded logical lines of a stream, yielding the
/// format-preserving tree one component block at a time.
pub struct Parser {
    logicals: Vec<Logical>,
    cursor: usize,
}

/// Why [`Parser::block`] returned: it consumed a closing `END` (whose raw
/// bytes it hands back) or reached the end of input.
pub enum Stop {
    End(String),
    Eof,
}

/// A logical (unfolded) line: its joined content and the exact original
/// bytes of every physical line that made it up.
struct Logical {
    content: String,
    raw: String,
}

impl Parser {
    /// Unfold `src` into logical lines, ready to parse from the start.
    pub fn new(src: &str) -> Parser {
        Parser {
            logicals: unfold(src),
            cursor: 0,
        }
    }

    /// Whether every logical line has been consumed.
    pub fn done(&self) -> bool {
        self.cursor >= self.logicals.len()
    }

    /// Parse nodes until the matching `END` or the end of input; a `BEGIN`
    /// recurses, so an `END` closes the innermost open component.
    pub fn block(&mut self) -> (Nodes, Stop) {
        let mut items = Nodes::default();

        while self.cursor < self.logicals.len() {
            let logical = &self.logicals[self.cursor];

            if end_name(&logical.content).is_some() {
                let end_raw = logical.raw.clone();
                self.cursor += 1;
                return (items, Stop::End(end_raw));
            }

            if let Some(name) = begin_name(&logical.content) {
                let begin_raw = logical.raw.clone();
                self.cursor += 1;

                let (inner, stop) = self.block();
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
            self.cursor += 1;
        }

        (items, Stop::Eof)
    }
}

/// Split `src` into logical lines, joining folded continuations (a physical
/// line starting with a space or tab) while recording the original bytes.
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

/// Split `src` into physical lines: for each, its content (no end of line)
/// and its raw bytes (the end of line, when present, included).
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

/// The property name of a content line (chars up to the first `;` or `:`),
/// uppercased; `None` for blank or nameless lines.
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

/// The type a `BEGIN`/`END` marker names (`marker` is `"BEGIN"`/`"END"`).
fn component_name(content: &str, marker: &str) -> Option<String> {
    if property_name(content)? != marker {
        return None;
    }

    let value = content.split_once(':')?.1.trim();
    Some(value.to_uppercase())
}
