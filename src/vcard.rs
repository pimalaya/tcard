//! # Reading and editing a card
//!
//! The one reader every verb uses, and the byte-preserving edits a fold-back
//! makes through it.
//!
//! [`parse`] reads a whole stream into vcard-rs's syntax tree, which
//! reproduces the wire bytes exactly. A card is therefore read once: what the
//! merge reconciles and what the projection walks are the same tree, and no
//! value passes through a second reader that might normalise it.
//!
//! [`Card`] is the edits applying a document makes, setting a property's
//! lines, and [`Cards`] the one a whole document makes, counting the cards a
//! file holds. An unchanged line keeps its own bytes, its parameter casing and
//! its group included, and only a line the document moved is written anew.

use core::fmt;

use alloc::{
    borrow::{Cow, ToOwned},
    string::{String, ToString},
    vec,
    vec::Vec,
};

use vcard::{
    tree::{cst::VcardCst, leaf::VcardLeaf, line::VcardLine, param::node::VcardParamNode},
    version::VcardVersion,
};

use crate::{
    error::{Result, TcardError},
    template::patch::{prefix, split, value},
};

/// A parsed vCard stream: the cards it holds, byte for byte.
///
/// A file often holds several, and every verb reads them all, so the stream is
/// the unit rather than the card: [`VcardCst::parse`] stops at the first card
/// and the rest would be lost.
#[derive(Default)]
pub struct Cards<'a>(pub Vec<VcardCst<'a>>);

impl<'a> Cards<'a> {
    /// The version the first card declares, `None` when it declares none.
    pub fn version(&self) -> Option<VcardVersion> {
        let card = self.0.first()?;
        card.version_line().map(|_| card.version())
    }

    /// Make the stream hold exactly `count` cards: append empty ones, or drop
    /// the surplus from the back so the ones before it keep their bytes.
    pub fn set_count(&mut self, count: usize) {
        let eol = self.0.first().map(eol_of).unwrap_or_else(crlf);

        for _ in self.0.len()..count {
            self.0.push(empty(&eol));
        }

        self.0.truncate(count);
    }
}

impl fmt::Display for Cards<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for card in &self.0 {
            card.fmt(f)?;
        }

        Ok(())
    }
}

/// Parse a whole vCard stream. A bare RFC 2425 record with no `BEGIN:VCARD`
/// envelope is accepted as well as a full card.
pub fn parse(input: &str) -> Result<Cards<'_>> {
    if input.trim().is_empty() {
        return Ok(Cards::default());
    }

    if !input.trim_start().starts_with("BEGIN") {
        return VcardCst::parse(input)
            .map(|card| Cards(vec![card]))
            .map_err(|err| TcardError::ParseVcard(err.to_string()));
    }

    VcardCst::parse_many(input)
        .collect::<core::result::Result<Vec<_>, _>>()
        .map(Cards)
        .map_err(|err| TcardError::ParseVcard(err.to_string()))
}

/// The byte-preserving property edits a fold-back makes to one card.
pub trait Card {
    /// The logical content lines of the properties of that name, in source
    /// order, each without its group prefix and its line ending.
    fn lines(&self, name: &str) -> Vec<String>;

    /// Make those properties exactly `lines`: an unchanged one keeps its own
    /// bytes, a surplus one is dropped, a missing one is appended. An empty
    /// slice removes them all.
    ///
    /// A reused line keeps the group it carried, a property being addressed by
    /// its bare name, so a grouped one is rewritten in place rather than
    /// doubled by a group-less copy.
    fn set_lines(&mut self, name: &str, lines: &[String]);
}

impl Card for VcardCst<'_> {
    fn lines(&self, name: &str) -> Vec<String> {
        props(self, name).map(ungrouped).collect()
    }

    fn set_lines(&mut self, name: &str, lines: &[String]) {
        let eol = eol_of(self);
        let held: Vec<usize> = self
            .props
            .iter()
            .enumerate()
            .filter(|(_, line)| named(line, name))
            .map(|(at, _)| at)
            .collect();

        for (slot, line) in held.iter().zip(lines) {
            let held = &mut self.props[*slot];
            let line = match group_of(held.name.get()) {
                Some(group) => group.to_owned() + line,
                None => line.clone(),
            };

            if logical(held) != line {
                *held = built(&line, &eol);
            }
        }

        for slot in held.iter().skip(lines.len()).rev() {
            self.props.remove(*slot);
        }

        for line in lines.iter().skip(held.len()) {
            self.props.push(built(line, &eol));
        }
    }
}

/// The properties of that name, in source order, a group prefix ignored.
fn props<'c, 'a>(card: &'c VcardCst<'a>, name: &str) -> impl Iterator<Item = &'c VcardLine<'a>> {
    card.props.iter().filter(move |line| named(line, name))
}

/// The logical content line a property occupies: its name, its parameters and
/// its value, without the line ending or the folds it was written with.
fn logical(line: &VcardLine<'_>) -> String {
    let mut out = String::from(line.name.get());

    for param in &line.params {
        out.push(';');
        out.push_str(&param.to_string());
    }

    out.push(':');
    out.push_str(&line.value.to_string());
    out
}

/// Whether a property carries that name, which vCard compares without regard
/// to case and beneath any group prefix (RFC 6350 section 3.3).
fn named(line: &VcardLine<'_>, name: &str) -> bool {
    bare(line.name.get()).eq_ignore_ascii_case(name)
}

/// The same line without its group prefix (`item1.EMAIL:a` gives `EMAIL:a`),
/// which is the form a document reads and writes back.
fn ungrouped(line: &VcardLine<'_>) -> String {
    let logical = logical(line);
    let group = group_of(line.name.get()).map_or(0, str::len);

    logical[group..].to_owned()
}

/// The group a property name carries, its trailing dot included.
fn group_of(name: &str) -> Option<&str> {
    name.rfind('.').map(|dot| &name[..=dot])
}

/// A property name beneath its group prefix.
fn bare(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// Build an owned content line from its text, ended with `eol`.
///
/// The name, every parameter and the value are carried across verbatim, so a
/// parameter a fold-back kept from the source keeps the bytes it had.
fn built(text: &str, eol: &str) -> VcardLine<'static> {
    let mut params = split(prefix(text), ';');
    let name = params.remove(0);

    let mut line = VcardLine::text(name.to_owned(), value(text).to_owned());

    line.params = params.into_iter().map(param).collect();
    line.eol = VcardLeaf(Cow::Owned(eol.to_owned()));
    line
}

/// Build one parameter node from its text, its value left as it was written.
fn param(text: &str) -> VcardParamNode<'static> {
    let (name, values) = match text.split_once('=') {
        Some((name, values)) => (name, vec![VcardLeaf(Cow::Owned(values.to_owned()))]),
        None => (text, Vec::new()),
    };

    VcardParamNode {
        name: VcardLeaf(Cow::Owned(name.to_owned())),
        values,
    }
}

/// An empty card, its envelope written with the given line ending.
fn empty(eol: &str) -> VcardCst<'static> {
    VcardCst {
        begin: Some(built("BEGIN:VCARD", eol)),
        props: Vec::new(),
        end: Some(built("END:VCARD", eol)),
        trailing: Cow::Borrowed(""),
    }
}

/// The line ending a card was written with, CRLF where it has none.
fn eol_of(card: &VcardCst<'_>) -> String {
    card.begin
        .as_ref()
        .map(|begin| begin.eol.get().to_owned())
        .filter(|eol| !eol.is_empty())
        .unwrap_or_else(crlf)
}

/// The line ending assumed where a card carries none.
fn crlf() -> String {
    "\r\n".to_owned()
}
