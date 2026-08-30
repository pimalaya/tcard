//! # Reading and editing a card
//!
//! The one reader every verb uses, and the byte-preserving edits a fold-back
//! makes through it.
//!
//! [`TcardCards::parse`] reads a whole stream into vcard-rs's syntax tree,
//! which reproduces the wire bytes exactly. A card is therefore read once:
//! what the merge reconciles and what the projection walks are the same tree,
//! and no value passes through a second reader that might normalise it.
//!
//! [`TcardCards`] counts the cards a file holds, and the crate-private `Card`
//! trait sets the lines of one property of one of them, which is what applying
//! a document comes down to. An unchanged line keeps its own bytes, its
//! parameter casing and its group included.
//!
//! That trait extends vcard-rs's own syntax node rather than abstracting over
//! anything: a foreign type takes no inherent method, and the projection reads
//! better calling one than passing the node to a function.

use core::fmt;

use alloc::{
    borrow::{Cow, ToOwned},
    string::{String, ToString},
    vec,
    vec::Vec,
};

use vcard::{
    tree::{
        codec::mode::VcardEscaper, cst::VcardCst, leaf::VcardLeaf, line::VcardLine,
        param::node::VcardParamNode,
    },
    version::VcardVersion,
};

use crate::{
    error::{TcardError, TcardResult},
    template::patch::{Content, split},
};

/// A parsed vCard stream: the cards it holds, byte for byte.
///
/// A file often holds several, and every verb reads them all, so the stream is
/// the unit rather than the card: [`VcardCst::parse`] stops at the first card
/// and the rest would be lost.
#[derive(Clone, Default)]
pub struct TcardCards<'a>(pub Vec<VcardCst<'a>>);

impl<'a> TcardCards<'a> {
    /// Parse a whole vCard stream.
    ///
    /// A bare RFC 2425 record with no `BEGIN:VCARD` envelope is accepted as
    /// well as a full card.
    pub fn parse(input: &'a str) -> TcardResult<Self> {
        if input.trim().is_empty() {
            return Ok(Self::default());
        }

        if !input.trim_start().starts_with("BEGIN") {
            return VcardCst::parse(input)
                .map(|card| Self(vec![card]))
                .map_err(|err| TcardError::ParseVcard(err.to_string()));
        }

        VcardCst::parse_many(input)
            .collect::<core::result::Result<Vec<_>, _>>()
            .map(Self)
            .map_err(|err| TcardError::ParseVcard(err.to_string()))
    }

    /// The version the first card declares, `None` when it declares none.
    pub fn version(&self) -> Option<VcardVersion> {
        let card = self.0.first()?;
        card.version_line().map(|_| card.version())
    }

    /// Make the stream hold exactly `count` cards.
    ///
    /// Empty ones are appended, and a surplus is dropped from the back so the
    /// ones before it keep their bytes.
    pub fn set_count(&mut self, count: usize) {
        let eol = self.0.first().map(eol_of).unwrap_or_else(crlf);

        for _ in self.0.len()..count {
            self.0.push(empty(&eol));
        }

        self.0.truncate(count);
    }
}

impl fmt::Display for TcardCards<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for card in &self.0 {
            card.fmt(f)?;
        }

        Ok(())
    }
}

/// The byte-preserving property edits a fold-back makes to one card.
pub(crate) trait Card {
    /// The content lines of the properties of that name, in source order.
    ///
    /// Each comes without its group prefix and its line ending.
    fn lines(&self, name: &str) -> Vec<String>;

    /// Make those properties exactly `lines`.
    ///
    /// Unchanged keeps its bytes, surplus is dropped, missing is appended,
    /// empty removes all. A property is addressed by its bare name, so a
    /// reused line keeps its group rather than being doubled group-less.
    fn set_lines(&mut self, name: &str, lines: &[String]);
}

impl Card for VcardCst<'_> {
    fn lines(&self, name: &str) -> Vec<String> {
        props(self, name).map(ungrouped).collect()
    }

    fn set_lines(&mut self, name: &str, lines: &[String]) {
        let eol = eol_of(self);
        let escaper = VcardEscaper::for_version(self.version());
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
                *held = built(&line, &eol, escaper);
            }
        }

        for slot in held.iter().skip(lines.len()).rev() {
            self.props.remove(*slot);
        }

        for line in lines.iter().skip(held.len()) {
            self.props.push(built(line, &eol, escaper));
        }
    }
}

/// The properties of that name, in source order, a group prefix ignored.
fn props<'c, 'a>(card: &'c VcardCst<'a>, name: &str) -> impl Iterator<Item = &'c VcardLine<'a>> {
    card.props.iter().filter(move |line| named(line, name))
}

/// The logical content line a property occupies.
///
/// Its name, its parameters and its value, without the line ending or the
/// folds it was written with.
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

/// Whether a property carries that name.
///
/// vCard compares a name without regard to case and beneath any group prefix
/// (RFC 6350 section 3.3).
fn named(line: &VcardLine<'_>, name: &str) -> bool {
    bare(line.name.get()).eq_ignore_ascii_case(name)
}

/// The same line without its group prefix, the form a document reads back.
///
/// `item1.EMAIL:a` gives `EMAIL:a`.
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
/// parameter a fold-back kept from the source keeps the bytes it had. Both the
/// value and the parameters are stamped with `escaper`, the rules the card's
/// own version is read and written in.
///
/// The line carries no wire layout, so it goes out unfolded. It is one the
/// document wrote, and a layout is offsets into the bytes of the line it
/// replaces, which are not these.
fn built(text: &str, eol: &str, escaper: VcardEscaper) -> VcardLine<'static> {
    let mut params = split(Content(text).prefix(), ';');
    let name = params.remove(0);

    let mut line = VcardLine::text(name.to_owned(), Content(text).value().to_owned());

    line.params = params.iter().map(|text| param(text, escaper)).collect();
    line.value.escaper = escaper;
    line.eol = VcardLeaf(Cow::Owned(eol.to_owned()));
    line
}

/// Build one parameter node from its text, its values left as they were
/// written.
///
/// vcard-rs splits the values, a comma inside a quoted one not counting, and
/// the pieces are taken over owned since the text they come from is the line a
/// fold-back has just assembled.
fn param(text: &str, escaper: VcardEscaper) -> VcardParamNode<'static> {
    let node = VcardParamNode::parse(text);

    VcardParamNode {
        name: VcardLeaf(Cow::Owned(node.name.get().to_owned())),
        values: node
            .values
            .iter()
            .map(|value| VcardLeaf(Cow::Owned(value.get().to_owned())))
            .collect(),
        escaper,
    }
}

/// An empty card, its envelope written with the given line ending.
///
/// It declares no version, so its envelope takes the default escaping rules,
/// which are the ones a card declaring none is read at. The two lines carry
/// neither a parameter nor a value to escape anyway.
fn empty(eol: &str) -> VcardCst<'static> {
    let escaper = VcardEscaper::default();

    VcardCst {
        begin: Some(built("BEGIN:VCARD", eol, escaper)),
        props: Vec::new(),
        end: Some(built("END:VCARD", eol, escaper)),
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
