//! Folding content lines back to vCard bytes, and end-of-line detection.

use alloc::string::String;

/// Longest octet length of a physical line before folding (RFC 6350 3.2).
const MAX_LINE_OCTETS: usize = 75;

/// Fold `content` at [`MAX_LINE_OCTETS`] octets with `{eol} ` continuations
/// and terminate with `eol`, mirroring calcard's writer.
pub fn render(content: &str, eol: &str) -> String {
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

/// The trailing end of line of `raw`, defaulting to CRLF when absent.
pub fn eol_of(raw: &str) -> &str {
    if raw.ends_with("\r\n") {
        "\r\n"
    } else if raw.ends_with('\n') {
        "\n"
    } else {
        "\r\n"
    }
}
