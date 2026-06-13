//! Projected lines and their tab-aligned inline comments.

use alloc::string::String;

/// Tab width assumed when aligning comments; their column is a multiple.
const TAB_WIDTH: usize = 8;

/// A projected line: a left side and an optional inline hint.
pub struct Line {
    pub lhs: String,
    pub hint: Option<String>,
}

/// The shared column at which a block's inline `#` comments align: the first
/// tab stop past the widest hinted left side, so every hinted line reaches it
/// with at least one tab (one too many is fine, one short would break the
/// column).
pub fn comment_column<'a>(lines: impl Iterator<Item = &'a Line>) -> usize {
    let widest = lines
        .filter(|line| line.hint.is_some())
        .map(|line| line.lhs.len())
        .max()
        .unwrap_or(0);

    (widest / TAB_WIDTH + 1) * TAB_WIDTH
}

/// Emit lines, padding a hinted line with tabs so its `#` lands on `column`.
pub fn emit_lines(out: &mut String, lines: &[Line], column: usize) {
    for line in lines {
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
