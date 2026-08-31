//! # Projection
//!
//! The two directions between a card and the ergonomic TOML form a reader
//! edits: projecting one out, and folding an edited one back.
//!
//! [`TcardTemplate`] carries the cards and the version they are shown at, so the
//! tree it projects is the tree it patches. vCard has a single record type, so
//! one card (or a blank file) flattens at the document root and two or more
//! become `[[card]]` blocks.
//!
//! Only the lines the reader changed are rewritten, so an unmodelled property
//! survives byte for byte. `UID` and `VERSION` are app-managed, seeded for a
//! new card and preserved for every other one, and cannot be set through the
//! form.
//!
//! TOML attributes every bare key after a `[table]` / `[[array]]` header to
//! that table, so the scalar and list keys lead and the sectioned properties
//! follow.

pub(crate) mod datetime;
mod line;
pub(crate) mod model;
pub(crate) mod patch;
pub(crate) mod toml;

use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use log::debug;
use toml_edit::{DocumentMut, TableLike};
use vcard::{tree::cst::VcardCst, version::VcardVersion};

use crate::{
    error::{TcardError, TcardResult},
    template::{
        line::Lines,
        model::{FIELDS, Field},
        toml::tables,
    },
    vcard::{Card, TcardCards},
};

/// A vCard stream and the TOML form it is edited through.
pub struct TcardTemplate<'a> {
    /// The cards the form shows.
    pub cards: TcardCards<'a>,
    /// The vCard version the form is written for.
    pub version: VcardVersion,
}

impl<'a> TcardTemplate<'a> {
    /// Read a vCard stream as the form it will be edited through.
    ///
    /// The form is written at the version the first card declares, falling
    /// back to `version` for a stream declaring none.
    pub fn parse(source: &'a str, version: VcardVersion) -> TcardResult<Self> {
        let cards = TcardCards::parse(source)?;
        let version = cards.version().unwrap_or(version);

        Ok(Self { cards, version })
    }

    /// Project the cards into a fillable TOML form.
    ///
    /// Known fields are prefilled and the rest listed empty.
    pub fn project(&self) -> String {
        debug!(
            "projecting {} card(s) to TOML (vCard {})",
            self.cards.0.len(),
            &*self.version,
        );

        match self.cards.0.len() > 1 {
            true => self.project_blocks(),
            false => self.project_flat(),
        }
    }

    /// Fold an edited form back onto the cards it was projected from.
    ///
    /// A filled `[[card]]` block updates or adds a card and an empty or absent
    /// one removes it.
    pub fn apply(&self, edited: &str) -> TcardResult<String> {
        debug!("applying {} bytes of edited TOML", edited.len());

        let doc: DocumentMut = edited.parse().map_err(TcardError::ParseToml)?;
        let mut cards = self.cards.clone();

        if doc.contains_key("card") {
            let blocks: Vec<&dyn TableLike> = doc
                .get("card")
                .map(tables)
                .unwrap_or_default()
                .into_iter()
                .filter(|table| filled(*table))
                .collect();

            cards.set_count(blocks.len());

            for (card, table) in cards.0.iter_mut().zip(blocks) {
                apply_card(card, table);
            }
        } else {
            cards.set_count(usize::from(filled(doc.as_table())));

            if let Some(card) = cards.0.first_mut() {
                apply_card(card, doc.as_table());
            }
        }

        Ok(cards.to_string())
    }

    /// Render the single card flat at the document root.
    ///
    /// Bare keys at the top level, with `[name]` / `[[email]]` sections and no
    /// wrapping header.
    fn project_flat(&self) -> String {
        let mut out = self.preamble();
        out.push_str("# Fill what you need; empty fields are ignored. Properties\n");
        out.push_str("# tCard does not model are kept verbatim, not shown here.\n");
        out.push('\n');

        emit(&mut out, &self.project_card(self.cards.0.first(), None));
        out
    }

    /// Render every card as a `[[card]]` block, the multi-card form.
    fn project_blocks(&self) -> String {
        let mut out = self.preamble();
        out.push_str("# Each card is a [[card]] block; repeat a block for repeated\n");
        out.push_str("# cards, delete one you do not need. Empty fields and empty\n");
        out.push_str("# blocks are ignored. Properties tCard does not model are kept\n");
        out.push_str("# verbatim, not shown here.\n");

        // NOTE: one column per card rather than one for the file, which is
        // what tCal does per component: a long value on one card would
        // otherwise push every other card's comments out with it.
        for card in &self.cards.0 {
            out.push('\n');
            emit(&mut out, &self.project_card(Some(card), Some("card")));
        }

        out
    }

    /// The first lines of the document, which name the version it is written
    /// at.
    fn preamble(&self) -> String {
        let mut out = String::from("# vCard ");
        out.push_str(&self.version);
        out.push_str(" as TOML, edited by tCard.\n");
        out.push_str("#\n");
        out
    }

    /// Render one card, flat or as a `[[prefix]]` block.
    ///
    /// A `None` prefix puts the sections at the top level, a named one nests
    /// them under the block.
    fn project_card(&self, card: Option<&VcardCst<'_>>, prefix: Option<&str>) -> Vec<Lines> {
        let bare: Vec<&Field> = FIELDS
            .iter()
            .take_while(|field| field.kind.is_simple())
            .collect();

        // NOTE: the `[[card]]` header leads the bare keys rather than
        // standing as a block of its own, the blank line belonging before
        // the header rather than after it.
        let mut lead = Lines::default();

        if let Some(prefix) = prefix {
            lead.push(format!("[[{prefix}]]"), None);
        }

        lead.extend(
            bare.iter()
                .flat_map(|field| field.lines(&held(card, field), self.version, prefix))
                .collect(),
        );

        let mut blocks = vec![lead];

        for field in &FIELDS[bare.len()..] {
            blocks.push(field.lines(&held(card, field), self.version, prefix));
        }

        blocks
    }
}

/// Write one card's blocks out, a blank line between them.
///
/// The column is measured across the card, so its comments align down the
/// page rather than stepping in and out at each section.
fn emit(out: &mut String, blocks: &[Lines]) {
    let column = line::column(blocks.iter().flat_map(Lines::iter));

    for (index, block) in blocks.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }

        block.emit(out, column);
    }
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

/// The content lines a card holds for a field.
///
/// Empty when the card is absent, which is the example block.
fn held(card: Option<&VcardCst<'_>>, field: &Field) -> Vec<String> {
    card.map(|card| card.lines(field.name)).unwrap_or_default()
}

/// Whether a card table carries any modelled value.
///
/// A table that carries none is the empty example placeholder, not a card.
fn filled(table: &dyn TableLike) -> bool {
    FIELDS
        .iter()
        .any(|field| !field.content_lines(table, &[]).is_empty())
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec::Vec};

    use vcard::version::VcardVersion;

    use crate::template::TcardTemplate;

    const SAMPLE: &str = "BEGIN:VCARD\r\n\
        VERSION:4.0\r\n\
        FN:John Doe\r\n\
        N:Doe;John;;;\r\n\
        EMAIL;TYPE=work:john@work.example\r\n\
        EMAIL;TYPE=home:john@home.example\r\n\
        ADR;TYPE=home:;;123 Main St;Springfield;IL;62701;USA\r\n\
        X-CUSTOM;TYPE=weird:keep me verbatim\r\n\
        END:VCARD\r\n";

    /// A card whose every modelled field round trips byte for byte, with no
    /// structured value to normalise, which pins the minimal diff exactly.
    const CLEAN: &str = "BEGIN:VCARD\r\n\
        VERSION:4.0\r\n\
        FN:John Doe\r\n\
        EMAIL;TYPE=work:john@work.example\r\n\
        X-CUSTOM:keep me verbatim\r\n\
        END:VCARD\r\n";

    /// Project a card at the version it declares.
    fn project(source: &str) -> String {
        TcardTemplate::parse(source, VcardVersion::V4_0)
            .unwrap()
            .project()
    }

    /// Project a blank scaffold at the given version.
    fn blank(version: VcardVersion) -> String {
        TcardTemplate::parse("", version).unwrap().project()
    }

    /// Fold an edited document back onto a card.
    fn apply(source: &str, edited: &str) -> String {
        TcardTemplate::parse(source, VcardVersion::V4_0)
            .unwrap()
            .apply(edited)
            .unwrap()
    }

    #[test]
    fn project_prefills_known_fields() {
        let toml = project(SAMPLE);

        assert!(!toml.contains("[[card]]"));
        assert!(toml.contains("full-name = \"John Doe\""));
        assert!(toml.contains("[name]"));
        assert!(toml.contains("family = [\"Doe\"]"));
        assert!(toml.contains("[[email]]"));
        assert!(toml.contains("value = \"john@work.example\""));
        assert!(toml.contains("street = [\"123 Main St\"]"));
        assert!(!toml.contains("X-CUSTOM"));
    }

    #[test]
    fn blank_project_layout() {
        let toml = blank(VcardVersion::V4_0);

        assert!(!toml.contains("[[card]]"));
        assert!(toml.find("full-name =").unwrap() < toml.find("kind =").unwrap());
        assert!(toml.find("role =").unwrap() < toml.find("categories =").unwrap());
        assert!(toml.find("categories =").unwrap() < toml.find("language =").unwrap());
        assert!(toml.find("language =").unwrap() < toml.find("note =").unwrap());
        assert!(toml.find("[name]").unwrap() < toml.find("[gender]").unwrap());
        assert!(toml.find("[[photo]]").unwrap() < toml.find("[[url]]").unwrap());

        assert!(toml.contains("full-name = \"\""));
        assert!(toml.contains("note = \"\""));
        assert!(!toml.contains("#full-name"));

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

        assert!(!project(src).contains("uid"));

        let out = apply(src, "[[card]]\nfull-name = \"A\"\nuid = \"hacked\"\n");
        assert!(out.contains("UID:urn:uuid:keep"));
        assert!(!out.contains("hacked"));
    }

    #[test]
    fn n_required_only_before_v4() {
        let name_required = |toml: &str| {
            toml.lines()
                .any(|line| line.starts_with("[name]") && line.contains("required"))
        };

        assert!(!name_required(&blank(VcardVersion::V4_0)));
        assert!(name_required(&blank(VcardVersion::V3_0)));
    }

    #[test]
    fn hints_are_tab_aligned() {
        let toml = blank(VcardVersion::V4_0);
        let hinted: Vec<&str> = toml
            .lines()
            .filter(|line| line.contains('=') && line.contains('#'))
            .collect();

        assert!(!hinted.is_empty());

        for line in &hinted {
            assert!(line.contains("\t#"), "not tab-aligned: {line:?}");
            let before = &line[..line.find('#').unwrap()];
            assert!(!before.contains("  "), "space padded: {line:?}");
        }

        // One column for the whole card, not one per section: expanding the
        // tabs puts every `#` at the same offset, across the bare keys,
        // `[name]`, `[[email]]` and the rest alike.
        let columns: Vec<usize> = hinted
            .iter()
            .map(|line| {
                let before = &line[..line.find('#').unwrap()];

                before.chars().fold(0, |at, ch| match ch {
                    '\t' => (at / 8 + 1) * 8,
                    _ => at + 1,
                })
            })
            .collect();

        assert!(
            columns.windows(2).all(|pair| pair[0] == pair[1]),
            "comments do not share one column: {columns:?}",
        );
    }

    #[test]
    fn date_fields_are_not_dropped() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\n\
            BDAY:19960415\r\nANNIVERSARY:20090808\r\nEND:VCARD\r\n";
        let toml = project(src);

        assert!(toml.contains("birthday = 1996-04-15"));
        assert!(toml.contains("anniversary = 2009-08-08"));
        assert_eq!(apply(src, &toml), src);
    }

    #[test]
    fn adr_pobox_and_ext_deprecated_by_version() {
        let v4 = blank(VcardVersion::V4_0);
        let v3 = blank(VcardVersion::V3_0);

        for key in ["pobox =", "ext ="] {
            assert!(v4.lines().all(|line| !line.starts_with(key)));
            assert!(
                v3.lines()
                    .any(|line| line.starts_with(key) && line.contains("# deprecated"))
            );
        }

        assert!(v4.contains("street ="));
        assert!(v3.contains("street ="));
    }

    #[test]
    fn photo_has_no_type_line() {
        let toml = blank(VcardVersion::V4_0);
        let photo = toml.split("[[photo]]").nth(1).unwrap();

        assert!(!photo.lines().take(2).any(|line| line.starts_with("type =")));
    }

    #[test]
    fn gender_roundtrips_with_identity() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nGENDER:O;intersex\r\nEND:VCARD\r\n";
        let toml = project(src);

        assert!(toml.contains("sex = \"O\""));
        assert!(toml.contains("identity = \"intersex\""));
        assert!(apply(src, &toml).contains("GENDER:O;intersex"));
    }

    /// RFC 6350 6.2.2: an `N` component holds several comma-separated
    /// values, which the form shows as an array.
    const MULTI: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nN:Stevenson;John;Philip,Paul;;Jr.,M.D.\r\nEND:VCARD\r\n";

    #[test]
    fn a_component_holding_several_values_projects_as_an_array() {
        let toml = project(MULTI);

        assert!(toml.contains("additional = [\"Philip\", \"Paul\"]"));
        assert!(toml.contains("suffixes = [\"Jr.\", \"M.D.\"]"));
        assert!(toml.contains("prefixes = []"));
    }

    #[test]
    fn editing_one_component_leaves_the_separators_of_the_others() {
        // The bug this guards: `N` is one line, so changing any component
        // re-renders all of them, and a component typed as a string had its
        // separators escaped into the value on the way past.
        let toml = project(MULTI).replace("given = [\"John\"]", "given = [\"Jon\"]");

        assert!(apply(MULTI, &toml).contains("N:Stevenson;Jon;Philip,Paul;;Jr.,M.D."));
    }

    #[test]
    fn a_component_written_as_a_string_is_read_as_one_value() {
        let toml = project(MULTI).replace("given = [\"John\"]", "given = \"Jon\"");

        assert!(apply(MULTI, &toml).contains("N:Stevenson;Jon;Philip,Paul;;Jr.,M.D."));
    }

    #[test]
    fn a_comma_typed_into_a_component_is_escaped() {
        let toml = project(MULTI).replace("family = [\"Stevenson\"]", "family = [\"Smith, Jr\"]");

        assert!(apply(MULTI, &toml).contains("N:Smith\\, Jr;John;"));
    }

    /// Two properties of one repeatable name, whose items the form shows as
    /// one array.
    const REPEATED: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\n\
        NICKNAME;PREF=1:Jim,Jimmy\r\nNICKNAME;PREF=2:Big Tuna\r\nEND:VCARD\r\n";

    #[test]
    fn removing_an_item_leaves_the_other_lines_alone() {
        // The bug this guards: the items were counted off the front of the
        // array, so dropping one slid every item behind it onto the line
        // before, taking that line's parameters. "Big Tuna" became the
        // preferred nickname and its own line disappeared.
        let toml = project(REPEATED).replace("\"Jim\", \"Jimmy\", ", "\"Jim\", ");
        let out = apply(REPEATED, &toml);

        assert!(out.contains("NICKNAME;PREF=1:Jim\r\n"), "{out}");
        assert!(out.contains("NICKNAME;PREF=2:Big Tuna\r\n"), "{out}");
    }

    #[test]
    fn renaming_an_item_rewrites_its_own_line() {
        let toml = project(REPEATED).replace("\"Big Tuna\"", "\"Tuna\"");
        let out = apply(REPEATED, &toml);

        assert!(out.contains("NICKNAME;PREF=2:Tuna\r\n"), "{out}");
        assert_eq!(out.matches("NICKNAME").count(), 2, "{out}");
    }

    #[test]
    fn items_added_to_a_single_line_join_it() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nNICKNAME;PREF=1:Jim\r\nEND:VCARD\r\n";
        let toml = project(src).replace("[\"Jim\"]", "[\"Jim\", \"Jimmy\"]");
        let out = apply(src, &toml);

        // One line has nothing to disambiguate, so an added item can only
        // belong to it, parameters and all.
        assert!(out.contains("NICKNAME;PREF=1:Jim,Jimmy\r\n"), "{out}");
        assert_eq!(out.matches("NICKNAME").count(), 1, "{out}");
    }

    #[test]
    fn items_no_line_held_share_one_new_line() {
        let toml = project(REPEATED).replace("\"Big Tuna\"]", "\"Big Tuna\", \"a\", \"b\"]");
        let out = apply(REPEATED, &toml);

        // Which line's parameters they should carry is the question two
        // lines make unanswerable, so they carry none, together.
        assert!(out.contains("NICKNAME:a,b\r\n"), "{out}");
        assert_eq!(out.matches("NICKNAME").count(), 3, "{out}");
    }

    #[test]
    fn structured_components_stay_on_one_line() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nORG:Acme;Eng\r\nEND:VCARD\r\n";
        let toml = project(src).replace("[\"Acme\", \"Eng\"]", "[\"Acme\", \"Eng\", \"Team\"]");
        let out = apply(src, &toml);

        // A `;` joins one property's own components, so a third one is a
        // third component rather than a second `ORG`.
        assert!(out.contains("ORG:Acme;Eng;Team\r\n"), "{out}");
        assert_eq!(out.matches("ORG").count(), 1, "{out}");
    }

    #[test]
    fn apply_projection_is_a_no_op() {
        assert_eq!(apply(CLEAN, &project(CLEAN)), CLEAN);
    }

    #[test]
    fn apply_changes_only_the_edited_line() {
        let toml = project(CLEAN).replace("John Doe", "Jane Roe");

        assert_eq!(
            apply(CLEAN, &toml),
            CLEAN.replace("FN:John Doe", "FN:Jane Roe")
        );
    }

    #[test]
    fn apply_roundtrip_preserves_unknown_properties() {
        let out = apply(SAMPLE, &project(SAMPLE));

        assert!(out.contains("FN:John Doe"));
        assert!(out.contains("john@work.example"));
        assert!(out.contains("john@home.example"));
        assert!(out.contains("X-CUSTOM"));
        assert!(out.contains("keep me verbatim"));
    }

    #[test]
    fn project_then_apply_preserves_bare_fields_after_sections() {
        let filled = "BEGIN:VCARD\r\n\
            VERSION:4.0\r\n\
            FN:Ada Lovelace\r\n\
            NICKNAME:Ada\r\n\
            NOTE:Pioneer\r\n\
            CATEGORIES:science\r\n\
            UID:urn:uuid:1234\r\n\
            EMAIL;TYPE=work:ada@analytical.example\r\n\
            END:VCARD\r\n";
        let out = apply(filled, &project(filled));

        assert!(out.contains("NICKNAME:Ada"));
        assert!(out.contains("NOTE:Pioneer"));
        assert!(out.contains("CATEGORIES:science"));
        assert!(out.contains("UID:urn:uuid:1234"));
        assert!(out.contains("ada@analytical.example"));
    }

    #[test]
    fn apply_empty_buffer_removes_cards() {
        let out = apply(SAMPLE, &blank(VcardVersion::V4_0));

        assert!(!out.contains("BEGIN:VCARD"));
        assert!(out.is_empty());
    }

    #[test]
    fn apply_edits_modeled_field() {
        let out = apply(SAMPLE, "[[card]]\nfull-name = \"Jane Roe\"\n");

        assert!(out.contains("FN:Jane Roe"));
        assert!(!out.contains("John Doe"));
        assert!(out.contains("X-CUSTOM"));
    }

    #[test]
    fn projects_and_edits_multiple_cards() {
        let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:first\r\nEND:VCARD\r\n\
            BEGIN:VCARD\r\nVERSION:4.0\r\nFN:second\r\nEND:VCARD\r\n";
        let toml = project(src);

        assert_eq!(toml.lines().filter(|line| *line == "[[card]]").count(), 2);

        let edited = toml.replace("second", "2nd");
        assert_eq!(apply(src, &edited), src.replace("FN:second", "FN:2nd"));
    }

    #[test]
    fn apply_adds_a_card() {
        let out = apply("", "[[card]]\nfull-name = \"New Person\"\n");

        assert!(out.contains("BEGIN:VCARD\r\n"));
        assert!(out.contains("FN:New Person\r\n"));
        assert!(out.contains("END:VCARD\r\n"));
    }
}
