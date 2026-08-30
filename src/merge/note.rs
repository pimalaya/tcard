//! # Notes
//!
//! What the merge settled by itself, said in a comment at the head of the
//! document.
//!
//! A collision with no key to contest has to be said somewhere, and so has a
//! merged value neither side wrote: a reader who is not told cannot review
//! what they are about to keep.

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use vcard::tree::{
    cst::VcardCst,
    merge::{VcardMergeAction, VcardMergeConflict, VcardMergeReport, VcardPropPath},
};

use crate::merge::{field_of, path};

/// The column a header comment wraps at, its `# ` prefix included.
const WRAP: usize = 66;

/// What the merge settled on its own, in the order it settled it.
#[derive(Default)]
pub struct Notes(Vec<String>);

impl Notes {
    /// Say what became of a collision the document puts no choice on.
    ///
    /// A removal the merge already decided in favour of the update, or a field
    /// the projection does not show, where the local value was kept.
    pub fn settled(&mut self, conflict: &VcardMergeConflict<'_>) {
        let label = label(&conflict.left);

        let note = match (&conflict.left, &conflict.right) {
            (VcardMergeAction::PropRemoved { .. }, _) => {
                format!("{label}: removed by local, updated by remote; the update was kept")
            }
            (_, VcardMergeAction::PropRemoved { .. }) => {
                format!("{label}: removed by remote, updated by local; the update was kept")
            }
            _ => format!(
                "{label}: both sides changed a part not shown here; the local value was kept"
            ),
        };

        self.push(note);
    }

    /// Say when two instances were paired by position.
    ///
    /// The base card holds several properties of that name and this one
    /// carries no `PID`, so the pairing rests on order alone and may well have
    /// brought together what the reader thinks of as two different numbers.
    pub fn pairing(&mut self, base: &VcardCst<'_>, conflict: &VcardMergeConflict<'_>) {
        let at = path(&conflict.left);
        let instances: Vec<_> = base
            .props
            .iter()
            .filter(|line| line.name.get().eq_ignore_ascii_case(&at.name))
            .collect();

        if instances.len() < 2 {
            return;
        }

        let Some(instance) = instances.get(at.index) else {
            return;
        };

        let pid = instance
            .params
            .iter()
            .any(|param| param.name.get().eq_ignore_ascii_case("PID"));

        if pid {
            return;
        }

        let label = label(&conflict.left);
        self.push(format!(
            "{label}: paired by position, not by PID; the pairing may be wrong"
        ));
    }

    /// Say every list both sides edited.
    ///
    /// The merge keeps the items of both and reports no conflict, which is
    /// right for a value RFC 6350 gives no order to: two sides each adding a
    /// nickname should keep both, and asking a reader to choose would be
    /// wrong. Saying nothing is what would be.
    pub fn unions(&mut self, report: &VcardMergeReport<'_>) {
        for action in &report.left {
            let Some((at, param)) = edited_items(action) else {
                continue;
            };

            let both = report
                .right
                .iter()
                .filter_map(edited_items)
                .any(|(other, held)| other == at && held == param);

            if !both {
                continue;
            }

            let label = label(action);
            self.push(match param {
                Some(param) => {
                    format!("{label}: both sides changed its {param}; the values of both were kept")
                }
                None => {
                    format!("{label}: both sides changed its list; the items of both were kept")
                }
            });
        }
    }

    /// Whether the merge settled anything worth saying.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The notes as a block continuing the document's header comment.
    pub fn render(&self) -> Vec<String> {
        let mut lines = vec!["#".to_owned(), "# Merge notes:".to_owned(), "#".to_owned()];
        lines.extend(self.0.iter().flat_map(|note| comment(&format!("- {note}"))));
        lines
    }

    /// Append a note, unless the same one was said already.
    fn push(&mut self, note: String) {
        if !self.0.contains(&note) {
            self.0.push(note);
        }
    }
}

/// The list an action edited, and the instance it belongs to.
///
/// The parameter's name comes with it when it is a list parameter rather than
/// the value itself.
fn edited_items<'p, 'a>(
    action: &'p VcardMergeAction<'a>,
) -> Option<(&'p VcardPropPath<'a>, Option<&'p str>)> {
    match action {
        VcardMergeAction::ValueItemAdded { at, .. }
        | VcardMergeAction::ValueItemRemoved { at, .. } => Some((at, None)),
        VcardMergeAction::ParamItemAdded { at, param, .. }
        | VcardMergeAction::ParamItemRemoved { at, param, .. } => Some((at, Some(param))),
        _ => None,
    }
}

/// How a note names a property: its TOML key, else its vCard name.
fn label(action: &VcardMergeAction<'_>) -> String {
    let name = &path(action).name;

    match field_of(name) {
        Some(field) => field.key.to_owned(),
        None => name.to_string(),
    }
}

/// One comment paragraph wrapped at [`WRAP`].
///
/// A continuation line of a bullet is indented under its text.
fn comment(text: &str) -> Vec<String> {
    let indent = if text.starts_with("- ") { "  " } else { "" };
    let mut lines = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + word.len() + 1 > WRAP {
            lines.push(format!("# {line}"));
            line = indent.to_owned();
        }

        if !line.is_empty() && !line.ends_with(' ') {
            line.push(' ');
        }

        line.push_str(word);
    }

    if !line.trim().is_empty() {
        lines.push(format!("# {line}"));
    }

    lines
}
