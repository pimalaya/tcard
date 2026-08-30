//! # Merge
//!
//! Three-way merge of a vCard, projected as a TOML document to decide.
//!
//! [`Merge`] reconciles a local and a remote card against the base they both
//! diverged from and renders the outcome through [`crate::template`], so a
//! merge is read and edited in the form everything else is. What it settled on
//! its own is already in that document.
//!
//! What it could not settle is written twice, once per side, as duplicate TOML
//! keys. TOML forbids them, so an undecided document does not parse and
//! [`Merged::apply`] names the field left undecided rather than a syntax
//! error. Resolving is deleting the unwanted line.
//!
//! Below it, choice turns a collision into the key it contests, document
//! writes that key into the projection, and note says at the head of the
//! document what carries no key at all.

pub(crate) mod choice;
pub(crate) mod document;
pub(crate) mod note;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec,
};

use log::debug;
use vcard::{
    tree::{
        codec::mode::VcardEscaper,
        cst::VcardCst,
        merge::{VcardMerge, VcardMergeAction, VcardPropPath},
    },
    version::VcardVersion,
};

use crate::{
    error::{Result, TcardError},
    merge::document::Document,
    template::{
        Template,
        model::{FIELDS, Field},
    },
    vcard::Cards,
};

/// Two divergent cards and the base they both came from.
pub struct Merge<'a> {
    /// The common ancestor, as vCard text.
    pub base: &'a str,
    /// The local side of the divergence, as vCard text.
    pub local: &'a str,
    /// The remote side of the divergence, as vCard text.
    pub remote: &'a str,
}

impl Merge<'_> {
    /// Merge the three cards into a document to decide.
    ///
    /// Only the first card of each side is merged: a merge projects one card,
    /// and its callers hand over one card per body.
    pub fn project(self) -> Result<Merged> {
        let base = read(self.base, "base")?;
        let local = read(self.local, "local")?;
        let remote = read(self.remote, "remote")?;

        let report = VcardMerge {
            base: &base,
            left: &local,
            right: &remote,
        }
        .merge();
        debug!("merged with {} collision(s)", report.conflicts.len());

        let version = report.merged.version();
        let template = Template {
            cards: Cards(vec![report.merged.clone()]),
            version,
        };

        let mut document = Document::new(&template.project());
        document.decorate(&base, &report, VcardEscaper::for_version(version));

        Ok(Merged {
            vcard: report.merged.to_string(),
            toml: document.into_string(),
        })
    }
}

/// A merged card, and the document deciding what the merge could not.
pub struct Merged {
    /// The merged card, the source [`Merged::apply`] patches.
    ///
    /// Every field the merge settled is already written into it.
    pub vcard: String,
    /// The projection of that card, each undecided collision written twice.
    pub toml: String,
}

impl Merged {
    /// Fold an edited merge document back onto the merged card.
    ///
    /// A collision left as written holds the same key twice, which TOML
    /// refuses: that parse error is reported as the field left undecided, so
    /// the reader is told what to resolve instead of being shown a syntax
    /// error.
    pub fn apply(&self, edited: &str) -> Result<String> {
        let template = Template::parse(&self.vcard, VcardVersion::V4_0)?;

        template.apply(edited).map_err(|err| undecided(err, edited))
    }
}

/// The instance an action targets, which every action carries.
pub(crate) fn path<'p, 'a>(action: &'p VcardMergeAction<'a>) -> &'p VcardPropPath<'a> {
    match action {
        VcardMergeAction::PropAdded { at, .. }
        | VcardMergeAction::PropRemoved { at, .. }
        | VcardMergeAction::ValueChanged { at, .. }
        | VcardMergeAction::ValueComponentChanged { at, .. }
        | VcardMergeAction::ValueItemAdded { at, .. }
        | VcardMergeAction::ValueItemRemoved { at, .. }
        | VcardMergeAction::ParamAdded { at, .. }
        | VcardMergeAction::ParamRemoved { at, .. }
        | VcardMergeAction::ParamChanged { at, .. }
        | VcardMergeAction::ParamItemAdded { at, .. }
        | VcardMergeAction::ParamItemRemoved { at, .. } => at,
    }
}

/// The modelled field a property name projects to, its group prefix stripped.
///
/// An unmodelled property (a custom `X-*`, a vendor extension) has none: it is
/// kept verbatim but never shown, so a collision on it cannot be contested.
pub(crate) fn field_of(name: &str) -> Option<&'static Field> {
    let name = name.rsplit('.').next().unwrap_or(name);

    FIELDS
        .iter()
        .find(|field| field.name.eq_ignore_ascii_case(name))
}

/// Read one side of a merge as a syntax tree, named by the side it is.
fn read<'a>(text: &'a str, side: &'static str) -> Result<VcardCst<'a>> {
    VcardCst::parse(text).map_err(|err| TcardError::ReadCard {
        side,
        message: err.to_string(),
    })
}

/// Rewrite a duplicate-key parse error into the key it leaves undecided.
///
/// The key is read back from the edited buffer at the span the parser refused,
/// since the error itself only says that a key repeats.
fn undecided(err: TcardError, edited: &str) -> TcardError {
    let TcardError::ParseToml(parse) = &err else {
        return err;
    };

    if !parse.message().contains("duplicate key") {
        return err;
    }

    match parse.span().and_then(|span| edited.get(span)) {
        Some(key) => TcardError::Undecided(key.to_owned()),
        None => err,
    }
}

#[cfg(test)]
mod tests {
    use alloc::{format, string::String};

    use crate::{
        error::TcardError,
        merge::{Merge, Merged},
    };

    fn card(props: &str) -> String {
        format!("BEGIN:VCARD\r\nVERSION:4.0\r\n{props}END:VCARD\r\n")
    }

    fn merge(base: &str, local: &str, remote: &str) -> Merged {
        Merge {
            base,
            local,
            remote,
        }
        .project()
        .unwrap()
    }

    /// The document's header comment as one unwrapped line.
    ///
    /// A note can then be looked for without minding where it was folded.
    fn notes(toml: &str) -> String {
        let mut header = String::new();

        for line in toml.lines().take_while(|line| line.starts_with('#')) {
            if !header.is_empty() {
                header.push(' ');
            }

            header.push_str(line.trim_start_matches('#').trim());
        }

        header
    }

    #[test]
    fn undecided_document_does_not_apply() {
        let base = card("FN:Jane Doe\r\n");
        let local = card("FN:Jane Doe-Smith\r\n");
        let remote = card("FN:Jane A. Doe\r\n");

        let merged = merge(&base, &local, &remote);

        assert!(merged.toml.contains("# conflict, keep one line\n"));
        assert!(merged.toml.contains("# full-name = \"Jane Doe\" # base\n"));
        assert!(
            merged
                .toml
                .contains("full-name = \"Jane Doe-Smith\" # local\n")
        );
        assert!(
            merged
                .toml
                .contains("full-name = \"Jane A. Doe\" # remote\n")
        );

        let err = merged.apply(&merged.toml).unwrap_err();
        assert!(
            matches!(&err, TcardError::Undecided(key) if key == "full-name"),
            "{err:?}",
        );
    }

    #[test]
    fn keeping_one_line_applies_that_value() {
        let base = card("FN:Jane Doe\r\n");
        let local = card("FN:Jane Doe-Smith\r\n");
        let remote = card("FN:Jane A. Doe\r\n");

        let merged = merge(&base, &local, &remote);
        let decided = merged
            .toml
            .replace("full-name = \"Jane A. Doe\" # remote\n", "");

        let out = merged.apply(&decided).unwrap();

        assert!(out.contains("FN:Jane Doe-Smith\r\n"));
        assert!(!out.contains("Jane A. Doe"));
    }

    /// Repeating the address header would be valid TOML and would silently
    /// make a second address instead of a refusal.
    #[test]
    fn structured_collision_stays_inside_its_table() {
        let base = card("FN:Jane\r\nADR;TYPE=home:;;1 Main St;Springfield;IL;62701;USA\r\n");
        let local = card("FN:Jane\r\nADR;TYPE=home:;;2 Oak St;Springfield;IL;62701;USA\r\n");
        let remote = card("FN:Jane\r\nADR;TYPE=home:;;3 Elm St;Springfield;IL;62701;USA\r\n");

        let merged = merge(&base, &local, &remote);

        assert_eq!(
            merged
                .toml
                .lines()
                .filter(|line| *line == "[[address]]")
                .count(),
            1,
        );
        assert!(merged.toml.contains("# street = \"1 Main St\" # base\n"));
        assert!(merged.toml.contains("street = \"2 Oak St\" # local\n"));
        assert!(merged.toml.contains("street = \"3 Elm St\" # remote\n"));
        assert_eq!(
            merged
                .toml
                .lines()
                .filter(|line| line.starts_with("locality = "))
                .count(),
            1,
        );

        let err = merged.apply(&merged.toml).unwrap_err();
        assert!(
            matches!(&err, TcardError::Undecided(key) if key == "street"),
            "{err:?}",
        );
    }

    #[test]
    fn removal_against_update_is_a_comment() {
        let base = card("FN:Jane\r\nNOTE:hi\r\n");
        let local = card("FN:Jane\r\n");
        let remote = card("FN:Jane\r\nNOTE:hello\r\n");

        let merged = merge(&base, &local, &remote);

        assert!(
            notes(&merged.toml)
                .contains("- note: removed by local, updated by remote; the update was kept")
        );
        assert!(!merged.toml.contains("# conflict"));
        assert_eq!(
            merged
                .toml
                .lines()
                .filter(|line| line.starts_with("note = "))
                .count(),
            1,
        );

        let out = merged.apply(&merged.toml).unwrap();
        assert!(out.contains("NOTE:hello\r\n"));
    }

    #[test]
    fn an_unreadable_side_is_named() {
        let base = card("FN:Jane\r\n");

        let Err(err) = (Merge {
            base: &base,
            local: &base,
            remote: "not a vCard at all",
        })
        .project() else {
            panic!("the unreadable side was read");
        };

        assert!(
            matches!(&err, TcardError::ReadCard { side, .. } if *side == "remote"),
            "{err:?}",
        );
    }

    #[test]
    fn positional_pairing_is_noted() {
        let base = card("FN:Jane\r\nTEL:+1\r\nTEL:+2\r\n");
        let local = card("FN:Jane\r\nTEL:+1\r\nTEL:+3\r\n");
        let remote = card("FN:Jane\r\nTEL:+1\r\nTEL:+4\r\n");

        let merged = merge(&base, &local, &remote);

        assert!(
            notes(&merged.toml)
                .contains("- phone: paired by position, not by PID; the pairing may be wrong")
        );
        assert!(merged.toml.contains("value = \"+3\" # local\n"));
        assert!(merged.toml.contains("value = \"+4\" # remote\n"));
    }
}
