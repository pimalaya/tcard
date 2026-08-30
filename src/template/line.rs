//! # Lines
//!
//! The projected lines of one block, and the column their inline comments
//! share.

use alloc::{string::String, vec::Vec};

/// Tab width assumed when aligning comments; their column is a multiple.
const TAB_WIDTH: usize = 8;

/// A projected line: a left side and an optional inline hint.
pub struct Line {
    /// The line itself, up to where its hint would start.
    pub lhs: String,
    /// The inline `#` hint, aligned on the block's shared column.
    pub hint: Option<String>,
}

/// The lines of one block, whose hints align on one column.
#[derive(Default)]
pub struct Lines(Vec<Line>);

impl Lines {
    /// Add one line, hinted or not.
    pub fn push(&mut self, lhs: String, hint: Option<String>) {
        self.0.push(Line { lhs, hint });
    }

    /// Add every line of another block.
    pub fn extend(&mut self, lines: Lines) {
        self.0.extend(lines.0);
    }

    /// Write the block out, each hint padded with tabs to the shared column.
    pub fn emit(&self, out: &mut String) {
        let column = self.column();

        for line in &self.0 {
            out.push_str(&line.lhs);

            if let Some(hint) = &line.hint {
                let mut at = line.lhs.len();

                while at < column {
                    out.push('\t');
                    at = (at / TAB_WIDTH + 1) * TAB_WIDTH;
                }

                out.push_str("# ");
                out.push_str(hint);
            }

            out.push('\n');
        }
    }

    /// The column at which the block's inline `#` comments align.
    ///
    /// It is the first tab stop past the widest hinted left side, so every
    /// hinted line reaches it with at least one tab: one too many is fine, one
    /// short would break the column.
    fn column(&self) -> usize {
        let widest = self
            .0
            .iter()
            .filter(|line| line.hint.is_some())
            .map(|line| line.lhs.len())
            .max()
            .unwrap_or(0);

        (widest / TAB_WIDTH + 1) * TAB_WIDTH
    }
}

impl FromIterator<Line> for Lines {
    fn from_iter<I: IntoIterator<Item = Line>>(lines: I) -> Self {
        Self(lines.into_iter().collect())
    }
}

impl IntoIterator for Lines {
    type Item = Line;
    type IntoIter = <Vec<Line> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
