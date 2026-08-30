//! # Decorated document
//!
//! The projected TOML document, with what the merge could not settle written
//! into it.
//!
//! An undecided collision replaces the line it contests, so the same key
//! appears twice and the document stops parsing. Everything else the merge did
//! on its own is said in the header comment.

use alloc::{borrow::ToOwned, format, string::String, vec::Vec};

use vcard::tree::{codec::mode::VcardEscaper, cst::VcardCst, merge::VcardMergeReport};

use crate::{
    merge::{choice::Choice, note::Notes, path},
    template::model::Kind,
};

/// The lines of a projected document, contested keys included.
pub struct Document(Vec<String>);

impl Document {
    /// Take a projection apart into the lines a merge decorates.
    pub fn new(toml: &str) -> Self {
        Self(toml.lines().map(str::to_owned).collect())
    }

    /// Write into the document what the merge could not settle.
    ///
    /// A key the document writes once is contested once, however many parts
    /// the report reported, so a further collision on it is neither a second
    /// choice nor a note.
    pub fn decorate(
        &mut self,
        base: &VcardCst<'_>,
        report: &VcardMergeReport<'_>,
        escaper: VcardEscaper,
    ) {
        let mut choices: Vec<(usize, Choice)> = Vec::new();
        let mut contested: Vec<(&str, usize, &str)> = Vec::new();
        let mut notes = Notes::default();

        for conflict in &report.conflicts {
            let at = path(&conflict.left);
            let choice = Choice::new(base, report, conflict, escaper);
            let key = |choice: &Choice| (at.name.as_ref(), at.index, choice.key);
            let again = choice
                .as_ref()
                .is_some_and(|choice| contested.contains(&key(choice)));

            let contest = choice.filter(|_| !again).and_then(|choice| {
                let taken: Vec<usize> = choices.iter().map(|(at, _)| *at).collect();
                self.locate(&choice, &taken).map(|at| (at, choice))
            });

            match contest {
                Some((line, choice)) => {
                    contested.push(key(&choice));
                    choices.push((line, choice));
                }
                None if again => {}
                None => notes.settled(conflict),
            }

            notes.pairing(base, conflict);
        }

        notes.unions(report);

        // NOTE: the contested lines are rewritten from the bottom up, so an
        // earlier index stays the one that was located.
        choices.sort_by_key(|(at, _)| *at);

        for (at, choice) in choices.iter().rev() {
            self.0.splice(at..=at, choice.render());
        }

        if !notes.is_empty() {
            let at = self.header_end();
            self.0.splice(at..at, notes.render());
        }
    }

    /// The document as the text a reader edits.
    pub fn into_string(self) -> String {
        let mut out = self.0.join("\n");
        out.push('\n');
        out
    }

    /// The line a choice contests.
    ///
    /// A bare key at the document root, else the key inside the block of the
    /// instance the report indexes. A line another choice already contests is
    /// skipped, and a key the version hides (a deprecated component) is not
    /// there to be found.
    ///
    /// That index is the instance's position in the *base* card while the
    /// document projects the merged one, so the two agree only where the
    /// pairing was positional. The named block is therefore taken only when it
    /// holds the value the merge kept, the local one wherever a choice is
    /// rendered at all, falling back to the first block holding that value,
    /// then the first holding the key.
    fn locate(&self, choice: &Choice, taken: &[usize]) -> Option<usize> {
        let headers = match choice.field.kind {
            Kind::Scalar | Kind::Date | Kind::List { .. } => {
                return self.find_key(0, choice.key, taken);
            }
            Kind::Structured(_) => self.headers(&format!("[{}]", choice.field.key)),
            Kind::Typed { .. } | Kind::TypedStructured { .. } => {
                self.headers(&format!("[[{}]]", choice.field.key))
            }
        };

        let indexed = headers
            .get(choice.instance)
            .and_then(|header| self.find_key(header + 1, choice.key, taken))
            .filter(|at| rhs(&self.0[*at], choice.key) == Some(choice.local.as_str()));

        if indexed.is_some() {
            return indexed;
        }

        let mut first = None;

        for header in headers {
            let Some(at) = self.find_key(header + 1, choice.key, taken) else {
                continue;
            };

            if rhs(&self.0[at], choice.key) == Some(choice.local.as_str()) {
                return Some(at);
            }

            first.get_or_insert(at);
        }

        first
    }

    /// The index of the line writing `key`, searched from `from`.
    ///
    /// The search stops at the end of the block, the next section header.
    fn find_key(&self, from: usize, key: &str, taken: &[usize]) -> Option<usize> {
        self.0
            .iter()
            .enumerate()
            .skip(from)
            .take_while(|(_, line)| !line.starts_with('['))
            .find(|(at, line)| !taken.contains(at) && rhs(line, key).is_some())
            .map(|(at, _)| at)
    }

    /// The indices of every line opening a block with the given section header.
    fn headers(&self, header: &str) -> Vec<usize> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, line)| lhs(line) == header)
            .map(|(at, _)| at)
            .collect()
    }

    /// Where the document's header comment ends, which is where the notes go.
    fn header_end(&self) -> usize {
        self.0
            .iter()
            .position(|line| !line.starts_with('#'))
            .unwrap_or(self.0.len())
    }
}

/// The right-hand side a line writes for `key`, `None` for another key.
fn rhs<'l>(line: &'l str, key: &str) -> Option<&'l str> {
    let rest = lhs(line).strip_prefix(key)?.trim_start();
    Some(rest.strip_prefix('=')?.trim_start())
}

/// A line without its aligned inline hint, which a tab sets off.
///
/// A TOML value never carries a raw tab, so the split is unambiguous.
fn lhs(line: &str) -> &str {
    line.split('\t').next().unwrap_or(line).trim_end()
}
