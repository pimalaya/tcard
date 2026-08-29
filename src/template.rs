//! # Projection
//!
//! The two directions between a card and the ergonomic TOML form a reader
//! edits: projecting one out, and folding an edited one back.
//!
//! [`project`] turns a vCard file into a fillable form. vCard has a single
//! record type, so a single card (or a blank file) is flattened at the
//! document root (bare keys, top-level `[name]` / `[[email]]`, no wrapper);
//! two or more cards become a list of `[[card]]` blocks.
//!
//! [`apply`] takes the original vCard text plus the edited buffer, detecting
//! which shape it is, and produces an updated file. Only the lines the reader
//! changed are rewritten (through [`crate::vcard`]), so every unmodelled
//! property is kept byte for byte.
//!
//! `UID` is not modelled: like `VERSION` it is managed by the app, seeded for
//! new cards and preserved otherwise, and cannot be set through the buffer.
//!
//! NOTE: TOML attributes every bare key after a `[table]` / `[[array]]` header
//! to that table, so the scalar and list keys lead and the sectioned
//! properties follow.

pub(crate) mod datetime;
mod line;
pub(crate) mod model;
pub(crate) mod patch;
pub(crate) mod util;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use log::trace;
use toml_edit::{DocumentMut, TableLike};
use vcard::{tree::cst::VcardCst, version::VcardVersion};

use crate::{
    error::{Result, TcardError},
    template::{
        line::{Line, comment_column, emit_lines},
        model::{FIELDS, Field},
        util::tables,
    },
    vcard::{Card, Cards},
};

/// Project a vCard file into a fillable TOML form.
///
/// vCard has a single record type, so a single card (or a blank file) is
/// flattened at the document root (bare keys, top-level `[name]` /
/// `[[email]]`, no wrapper); two or more cards become a list of `[[card]]`
/// blocks. Known fields are prefilled, the rest listed empty.
pub fn project(cards: &Cards<'_>, version: VcardVersion) -> String {
    trace!(
        "projecting {} card(s) to TOML (vCard {})",
        cards.0.len(),
        &*version,
    );

    if cards.0.len() > 1 {
        project_blocks(cards, version)
    } else {
        project_flat(cards.0.first(), version)
    }
}

/// Render one card flat at the document root: bare keys at the top level,
/// with `[name]` / `[[email]]` sections, no wrapping header.
fn project_flat(card: Option<&VcardCst<'_>>, version: VcardVersion) -> String {
    let mut out = String::new();
    out.push_str("# vCard ");
    out.push_str(&version);
    out.push_str(" as TOML, edited by tcard.\n");
    out.push_str("#\n");
    out.push_str("# Fill what you need; empty fields are ignored. Properties\n");
    out.push_str("# tcard does not model are kept verbatim, not shown here.\n");
    out.push('\n');

    project_card(&mut out, card, version, None);
    out
}

/// Render every card as a `[[card]]` block, the multi-card form.
fn project_blocks(cards: &Cards<'_>, version: VcardVersion) -> String {
    let mut out = String::new();
    out.push_str("# vCard ");
    out.push_str(&version);
    out.push_str(" as TOML, edited by tcard.\n");
    out.push_str("#\n");
    out.push_str("# Each card is a [[card]] block; repeat a block for repeated\n");
    out.push_str("# cards, delete one you do not need. Empty fields and empty\n");
    out.push_str("# blocks are ignored. Properties tcard does not model are kept\n");
    out.push_str("# verbatim, not shown here.\n");

    for card in &cards.0 {
        project_card(&mut out, Some(card), version, Some("card"));
    }

    out
}

/// Render one card: flat (`prefix` is `None`, sections at the top level) or as
/// a `[[prefix]]` block with nested sections.
fn project_card(
    out: &mut String,
    card: Option<&VcardCst<'_>>,
    version: VcardVersion,
    prefix: Option<&str>,
) {
    if let Some(prefix) = prefix {
        out.push('\n');
        out.push_str("[[");
        out.push_str(prefix);
        out.push_str("]]\n");
    }

    // The bare keys form one block with a shared comment column.
    let bare: Vec<&Field> = FIELDS
        .iter()
        .take_while(|field| field.kind.is_simple())
        .collect();
    let bare_lines: Vec<Line> = bare
        .iter()
        .flat_map(|field| field.lines(&held(card, field), version, prefix))
        .collect();
    emit_lines(out, &bare_lines, comment_column(bare_lines.iter()));

    // Each section is set off by a blank line and aligned within itself.
    for field in &FIELDS[bare.len()..] {
        out.push('\n');
        let lines = field.lines(&held(card, field), version, prefix);
        emit_lines(out, &lines, comment_column(lines.iter()));
    }
}

/// Apply an edited TOML buffer onto the original vCard text.
///
/// The buffer's shape is detected: a flat card (bare keys, no `[[card]]`
/// header) folds onto the single card; otherwise each `[[card]]` block
/// reconciles a card. Only the lines that actually changed are rewritten, so
/// unmodelled properties (the app-managed `UID` and `VERSION` among them),
/// ordering and parameter casing are all kept verbatim. A filled block updates
/// or adds a card, an empty or absent block removes it.
pub fn apply(original_src: &str, edited_toml: &str) -> Result<String> {
    trace!("applying {} bytes of edited TOML", edited_toml.len());

    let doc: DocumentMut = edited_toml.parse().map_err(TcardError::ParseToml)?;

    let mut cards = crate::vcard::parse(original_src)?;

    if doc.contains_key("card") {
        // Block form: one [[card]] table per card.
        let blocks: Vec<&dyn TableLike> = doc
            .get("card")
            .map(tables)
            .unwrap_or_default()
            .into_iter()
            .filter(|table| card_has_content(*table))
            .collect();

        cards.set_count(blocks.len());
        for (card, table) in cards.0.iter_mut().zip(blocks) {
            apply_card(card, table);
        }
    } else {
        // Flat form: the document top level is one card's table.
        cards.set_count(usize::from(card_has_content(doc.as_table())));
        if let Some(card) = cards.0.first_mut() {
            apply_card(card, doc.as_table());
        }
    }

    Ok(cards.to_string())
}

/// Rewrite one card's fields from its TOML table, with a minimal diff.
///
/// Each field is folded onto the lines the card already holds for it, so a
/// parameter or a component the document does not write survives.
fn apply_card(card: &mut VcardCst<'_>, table: &dyn TableLike) {
    for field in FIELDS {
        let held = card.lines(field.name);
        card.set_lines(field.name, &field.content_lines(table, &held));
    }
}

/// The content lines a card holds for a field (empty when the card is absent,
/// for the example block).
fn held(card: Option<&VcardCst<'_>>, field: &Field) -> Vec<String> {
    card.map(|card| card.lines(field.name)).unwrap_or_default()
}

/// Whether a `[[card]]` table carries any modeled value, i.e. is a real card
/// rather than the empty example placeholder.
fn card_has_content(table: &dyn TableLike) -> bool {
    FIELDS
        .iter()
        .any(|field| !field.content_lines(table, &[]).is_empty())
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use vcard::version::VcardVersion;

    use crate::vcard::{Cards, parse};

    const SAMPLE: &str = "BEGIN:VCARD\r\n\
        VERSION:4.0\r\n\
        FN:John Doe\r\n\
        N:Doe;John;;;\r\n\
        EMAIL;TYPE=work:john@work.example\r\n\
        EMAIL;TYPE=home:john@home.example\r\n\
        ADR;TYPE=home:;;123 Main St;Springfield;IL;62701;USA\r\n\
        X-CUSTOM;TYPE=weird:keep me verbatim\r\n\
        END:VCARD\r\n";

    // Every modeled field here round-trips byte-for-byte through the
    // projection (no structured trailing-empty normalization like `N`), so
    // it can pin down the exact minimal-diff guarantee.
    const CLEAN: &str = "BEGIN:VCARD\r\n\
        VERSION:4.0\r\n\
        FN:John Doe\r\n\
        EMAIL;TYPE=work:john@work.example\r\n\
        X-CUSTOM:keep me verbatim\r\n\
        END:VCARD\r\n";

    #[test]
    fn project_prefills_known_fields() {
        let card = parse(SAMPLE).unwrap();
        let toml = super::project(&card, VcardVersion::V4_0);

        // A single card flattens at the root, no [[card]] wrapper.
        assert!(!toml.contains("[[card]]"));
        assert!(toml.contains("full-name = \"John Doe\""));
        assert!(toml.contains("[name]"));
        assert!(toml.contains("family = \"Doe\""));
        assert!(toml.contains("[[email]]"));
        assert!(toml.contains("value = \"john@work.example\""));
        assert!(toml.contains("street = \"123 Main St\""));
        // Unmodeled properties never appear in the scaffold.
        assert!(!toml.contains("X-CUSTOM"));
    }

    #[test]
    fn blank_project_layout() {
        let toml = super::project(&Cards::default(), VcardVersion::V4_0);

        // A blank file flattens at the root; bare keys lead, sections follow.
        assert!(!toml.contains("[[card]]"));
        assert!(toml.find("full-name =").unwrap() < toml.find("kind =").unwrap());
        assert!(toml.find("role =").unwrap() < toml.find("categories =").unwrap());
        assert!(toml.find("categories =").unwrap() < toml.find("language =").unwrap());
        assert!(toml.find("language =").unwrap() < toml.find("note =").unwrap());
        assert!(toml.find("[name]").unwrap() < toml.find("[gender]").unwrap());
        assert!(toml.find("[[photo]]").unwrap() < toml.find("[[url]]").unwrap());

        // Empty, uncommented fields; note as a plain empty string.
        assert!(toml.contains("full-name = \"\""));
        assert!(toml.contains("note = \"\""));
        assert!(!toml.contains("#full-name"));

        // full-name is flagged required; hints carry no "e.g." prefix, just
        // the example value or the list of accepted values.
        assert!(toml.contains("# required"));
        assert!(toml.contains("# F, M, O, N, U"));
        assert!(toml.contains("# geo:37.78,-122.40"));
        assert!(toml.contains("# home, work, cell"));
        assert!(toml.contains("# file:// or https://"));
        assert!(toml.contains("# email address"));
        assert!(!toml.contains("e.g."));
    }

    #[test]
    fn uid_is_hidden_and_app_managed() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nUID:urn:uuid:keep\r\nEND:VCARD\r\n";
        let card = parse(src).unwrap();

        // Hidden from the form.
        let toml = super::project(&card, VcardVersion::V4_0);
        assert!(!toml.contains("uid"));

        // Preserved on round-trip, and not overridable from the buffer.
        let edited = "[[card]]\nfull-name = \"A\"\nuid = \"hacked\"\n";
        let out = super::apply(src, edited).unwrap();
        assert!(out.contains("UID:urn:uuid:keep"));
        assert!(!out.contains("hacked"));
    }

    #[test]
    fn n_required_only_before_v4() {
        let v4 = super::project(&Cards::default(), VcardVersion::V4_0);
        let v3 = super::project(&Cards::default(), VcardVersion::V3_0);

        let name_required = |toml: &str| {
            toml.lines()
                .any(|line| line.starts_with("[name]") && line.contains("required"))
        };

        assert!(!name_required(&v4));
        assert!(name_required(&v3));
    }

    #[test]
    fn hints_are_tab_aligned() {
        let toml = super::project(&Cards::default(), VcardVersion::V4_0);

        // Every inline hint is separated from its value by a tab, so the
        // comment lands at a tab stop instead of a far, space-padded column.
        // No hinted key line carries a run of padding spaces.
        let hinted: Vec<&str> = toml
            .lines()
            .filter(|line| line.contains('=') && line.contains('#'))
            .collect();
        assert!(!hinted.is_empty());

        for line in hinted {
            assert!(line.contains("\t#"), "not tab-aligned: {line:?}");
            let before = &line[..line.find('#').unwrap()];
            assert!(!before.contains("  "), "space padded: {line:?}");
        }
    }

    #[test]
    fn date_fields_are_not_dropped() {
        // A date projects as a native TOML value and folds back into the
        // basic form the card wrote, rather than vanishing between the two.
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\n\
            BDAY:19960415\r\nANNIVERSARY:20090808\r\nEND:VCARD\r\n";
        let card = parse(src).unwrap();
        let toml = super::project(&card, VcardVersion::V4_0);

        assert!(toml.contains("birthday = 1996-04-15"));
        assert!(toml.contains("anniversary = 2009-08-08"));
        assert_eq!(super::apply(src, &toml).unwrap(), src);
    }

    #[test]
    fn adr_pobox_and_ext_deprecated_by_version() {
        let v4 = super::project(&Cards::default(), VcardVersion::V4_0);
        let v3 = super::project(&Cards::default(), VcardVersion::V3_0);

        // RFC 6350 deprecates pobox/ext: hidden in 4.0, flagged before it.
        for key in ["pobox =", "ext ="] {
            assert!(v4.lines().all(|line| !line.starts_with(key)));
            assert!(
                v3.lines()
                    .any(|line| line.starts_with(key) && line.contains("# deprecated"))
            );
        }
        // street stays in both.
        assert!(v4.contains("street ="));
        assert!(v3.contains("street ="));
    }

    #[test]
    fn photo_has_no_type_line() {
        let toml = super::project(&Cards::default(), VcardVersion::V4_0);
        let photo = toml.split("[[photo]]").nth(1).unwrap();

        assert!(!photo.lines().take(2).any(|line| line.starts_with("type =")));
    }

    #[test]
    fn gender_roundtrips_with_identity() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nGENDER:O;intersex\r\nEND:VCARD\r\n";
        let card = parse(src).unwrap();
        let toml = super::project(&card, VcardVersion::V4_0);

        assert!(toml.contains("sex = \"O\""));
        assert!(toml.contains("identity = \"intersex\""));

        let out = super::apply(src, &toml).unwrap();
        assert!(out.contains("GENDER:O;intersex"));
    }

    #[test]
    fn apply_projection_is_a_no_op() {
        // Projecting then applying an untouched buffer must reproduce the
        // source byte-for-byte: the minimal-diff guarantee at its limit.
        let card = parse(CLEAN).unwrap();
        let toml = super::project(&card, VcardVersion::V4_0);

        assert_eq!(super::apply(CLEAN, &toml).unwrap(), CLEAN);
    }

    #[test]
    fn apply_changes_only_the_edited_line() {
        let card = parse(CLEAN).unwrap();
        let toml = super::project(&card, VcardVersion::V4_0).replace("John Doe", "Jane Roe");

        let out = super::apply(CLEAN, &toml).unwrap();

        assert_eq!(out, CLEAN.replace("FN:John Doe", "FN:Jane Roe"));
    }

    #[test]
    fn apply_roundtrip_preserves_unknown_properties() {
        let card = parse(SAMPLE).unwrap();
        let toml = super::project(&card, VcardVersion::V4_0);

        let out = super::apply(SAMPLE, &toml).unwrap();

        assert!(out.contains("FN:John Doe"));
        assert!(out.contains("john@work.example"));
        assert!(out.contains("john@home.example"));
        // The unmodeled property survives the round-trip verbatim.
        assert!(out.contains("X-CUSTOM"));
        assert!(out.contains("keep me verbatim"));
    }

    #[test]
    fn project_then_apply_preserves_bare_fields_after_sections() {
        // These scalar/list fields are emitted before the sections so TOML
        // does not nest them inside a table; a round-trip through the
        // projected scaffold must keep every one of them.
        let filled = "BEGIN:VCARD\r\n\
            VERSION:4.0\r\n\
            FN:Ada Lovelace\r\n\
            NICKNAME:Ada\r\n\
            NOTE:Pioneer\r\n\
            CATEGORIES:science\r\n\
            UID:urn:uuid:1234\r\n\
            EMAIL;TYPE=work:ada@analytical.example\r\n\
            END:VCARD\r\n";
        let card = parse(filled).unwrap();
        let toml = super::project(&card, VcardVersion::V4_0);

        let out = super::apply(filled, &toml).unwrap();

        assert!(out.contains("NICKNAME:Ada"));
        assert!(out.contains("NOTE:Pioneer"));
        assert!(out.contains("CATEGORIES:science"));
        assert!(out.contains("UID:urn:uuid:1234"));
        assert!(out.contains("ada@analytical.example"));
    }

    #[test]
    fn apply_empty_buffer_removes_cards() {
        // The blank scaffold is one empty flat card; applying it keeps no
        // card (an empty card is ignored, like a blank field).
        let blank = super::project(&Cards::default(), VcardVersion::V4_0);

        let out = super::apply(SAMPLE, &blank).unwrap();

        assert!(!out.contains("BEGIN:VCARD"));
        assert!(out.is_empty());
    }

    #[test]
    fn apply_edits_modeled_field() {
        let edited = "[[card]]\nfull-name = \"Jane Roe\"\n";

        let out = super::apply(SAMPLE, edited).unwrap();

        assert!(out.contains("FN:Jane Roe"));
        assert!(!out.contains("John Doe"));
        // The card is kept, so its unmodeled property stays.
        assert!(out.contains("X-CUSTOM"));
    }

    #[test]
    fn projects_and_edits_multiple_cards() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:first\r\nEND:VCARD\r\n\
            BEGIN:VCARD\r\nVERSION:4.0\r\nFN:second\r\nEND:VCARD\r\n";
        let cards = parse(src).unwrap();
        let toml = super::project(&cards, VcardVersion::V4_0);

        // Two cards project as two blocks (ignore the comment mention).
        assert_eq!(toml.lines().filter(|line| *line == "[[card]]").count(), 2);

        // Editing the second leaves the first byte-for-byte.
        let edited = toml.replace("second", "2nd");
        let out = super::apply(src, &edited).unwrap();
        assert_eq!(out, src.replace("FN:second", "FN:2nd"));
    }

    #[test]
    fn apply_adds_a_card() {
        let edited = "[[card]]\nfull-name = \"New Person\"\n";

        let out = super::apply("", edited).unwrap();

        assert!(out.contains("BEGIN:VCARD\r\n"));
        assert!(out.contains("FN:New Person\r\n"));
        assert!(out.contains("END:VCARD\r\n"));
    }
}
