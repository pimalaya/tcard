//! Projection between a calcard [`VCard`] and an ergonomic TOML buffer.
//!
//! [`project`] turns a vCard into a fillable TOML form: known fields are
//! prefilled, the rest are listed empty (an empty value means the same as a
//! removed line, so nothing is commented out). A hint, when useful, sits inline
//! next to the value, and hints within a block are aligned to a common
//! column. [`apply`] takes the original vCard plus the edited buffer and
//! produces an updated vCard, rebuilding the modeled fields from TOML while
//! carrying every unmodeled property (custom `X-*`, vendor extensions, ...)
//! verbatim.
//!
//! `UID` is intentionally not modeled: like `VERSION` it is managed by the app
//! (seeded for new cards, preserved otherwise) and cannot be set through the
//! buffer.
//!
//! The buffer is an editing affordance, not an interchange format: `apply`
//! always needs the original vCard, because that is where unmodeled properties
//! live.
//!
//! NOTE: TOML attributes every bare key after a `[table]` / `[[array]]` header
//! to that table, so [`FIELDS`] lists all scalar/list keys first and every
//! sectioned property (`N`, `EMAIL`, `ADR`, ...) last.

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use calcard::{
    common::IanaString,
    vcard::{
        VCard, VCardEntry, VCardParameterName, VCardParameterValue, VCardProperty, VCardValue,
        VCardVersion,
    },
};
use log::trace;
use toml_edit::{DocumentMut, Item, TableLike};

use crate::{
    error::{Result, TcardError},
    vcard,
};

/// Project a vCard into a fillable TOML form.
///
/// An empty [`VCard`] yields a blank template: the required fields first, then
/// the rest of the bare keys, sections last.
pub fn project(vcard: &VCard, version: VCardVersion) -> String {
    trace!(
        "projecting {} entries to TOML (vCard {})",
        vcard.entries.len(),
        vcard::version_str(version),
    );

    let mut out = String::new();

    out.push_str("# vCard ");
    out.push_str(vcard::version_str(version));
    out.push_str(" as TOML, edited by tcard.\n");
    out.push_str("#\n");
    out.push_str("# Fill what you need; empty fields are ignored. Properties\n");
    out.push_str("# tcard does not model are kept verbatim, not shown here.\n");

    let collect = |field: &Field| -> Vec<&VCardEntry> {
        vcard
            .entries
            .iter()
            .filter(|entry| entry.name.as_str() == field.name)
            .collect()
    };

    // The bare keys form one block with a shared comment column.
    let bare: Vec<&Field> = FIELDS
        .iter()
        .take_while(|field| field.kind.is_simple())
        .collect();
    let bare_lines: Vec<Line> = bare
        .iter()
        .flat_map(|field| field.lines(&collect(field), version))
        .collect();
    out.push('\n');
    emit_lines(&mut out, &bare_lines, comment_column(bare_lines.iter()));

    // Each section is set off by a blank line and aligned within itself.
    for field in &FIELDS[bare.len()..] {
        out.push('\n');
        let lines = field.lines(&collect(field), version);
        emit_lines(&mut out, &lines, comment_column(lines.iter()));
    }

    out
}

/// Apply an edited TOML buffer onto the original vCard.
///
/// Modeled fields are rebuilt from the buffer; unmodeled properties of
/// `original` (including the app-managed `UID`) are preserved verbatim. The
/// result is serialized by calcard at the requested `version`, so output is
/// normalized (line folding, parameter casing) but lossless for unknown
/// properties.
pub fn apply(original: &VCard, edited_toml: &str, version: VCardVersion) -> Result<String> {
    trace!("applying {} bytes of edited TOML", edited_toml.len());

    let doc: DocumentMut = edited_toml.parse().map_err(TcardError::ParseToml)?;

    let mut assembled = String::from("BEGIN:VCARD\r\n");
    assembled.push_str("VERSION:");
    assembled.push_str(vcard::version_str(version));
    assembled.push_str("\r\n");

    for field in FIELDS {
        field.emit(&doc, &mut assembled);
    }

    assembled.push_str("END:VCARD\r\n");

    let mut rebuilt = vcard::parse(&assembled)?;
    rebuilt.entries.retain(|entry| is_data(&entry.name));
    let modeled = rebuilt.entries.len();

    let mut preserved = 0;
    for entry in &original.entries {
        if is_data(&entry.name) && !is_modeled(&entry.name) {
            rebuilt.entries.push(entry.clone());
            preserved += 1;
        }
    }

    trace!("rebuilt {modeled} modeled entries, preserved {preserved} unmodeled");

    let mut out = String::new();
    rebuilt
        .write_to(&mut out, version)
        .expect("writing a vCard to a String is infallible");

    Ok(out)
}

/// A projected line: a left side and an optional inline hint.
struct Line {
    lhs: String,
    hint: Option<String>,
}

/// A named component of a structured value, with an optional hint.
type Component = (&'static str, Option<&'static str>);

/// Whether a property is required, possibly only in legacy versions.
#[derive(Clone, Copy)]
enum Req {
    /// Optional.
    No,
    /// Required in every version (`FN`).
    Always,
    /// Required before 4.0 only (`N`).
    Legacy,
}

/// Shape of a modeled property, driving both projection and emission.
///
/// `TYPE` never changes a property's shape (an `EMAIL` is one value whether
/// home or work), so typed properties keep a single section and list their
/// accepted types in a trailing comment.
enum Kind {
    /// Single text value (`FN`, `NOTE`, ...).
    Scalar,

    /// Repeated or multi-valued text, joined on `sep` in the vCard (`NICKNAME`,
    /// `CATEGORIES`, `ORG`).
    List { sep: char },

    /// One structured value with named, ordered components (`N`, `GENDER`).
    Structured(&'static [Component]),

    /// Repeatable property with an optional `TYPE` and a single value (`EMAIL`,
    /// `TEL`, `URL`, `PHOTO`).
    Typed { types: &'static [&'static str] },

    /// Repeatable property with an optional `TYPE` and named, ordered
    /// components (`ADR`).
    TypedStructured {
        types: &'static [&'static str],
        components: &'static [Component],
    },
}

impl Kind {
    /// A bare key (vs a `[table]` / `[[array]]` section).
    fn is_simple(&self) -> bool {
        matches!(self, Kind::Scalar | Kind::List { .. })
    }
}

/// A modeled vCard property and how it maps to TOML.
struct Field {
    /// TOML key.
    key: &'static str,
    /// Canonical vCard property name (matches calcard's `as_str`).
    name: &'static str,
    /// Whether the property is required.
    req: Req,
    /// Inline hint shown next to the value, only where it is not self-evident
    /// (rendered as ` # <hint>`).
    hint: Option<&'static str>,
    /// Mapping shape.
    kind: Kind,
}

/// `N` components, in RFC 6350 order.
const NAME_COMPONENTS: &[Component] = &[
    ("family", None),
    ("given", None),
    ("additional", None),
    ("prefixes", None),
    ("suffixes", None),
];

/// `ADR` components, in RFC 6350 order.
const ADR_COMPONENTS: &[Component] = &[
    ("pobox", None),
    ("ext", None),
    ("street", None),
    ("locality", None),
    ("region", None),
    ("code", None),
    ("country", None),
];

/// `GENDER` components: sex code plus a free-text identity.
const GENDER_COMPONENTS: &[Component] = &[("sex", Some("e.g. F, M, O, N, U")), ("identity", None)];

/// Common `TYPE` sets, shared between properties.
const PLACE_TYPES: &[&str] = &["home", "work"];
const TEL_TYPES: &[&str] = &[
    "home",
    "work",
    "cell",
    "fax",
    "voice",
    "video",
    "pager",
    "text",
    "textphone",
];

/// The modeled vocabulary. Everything outside this list is preserved verbatim
/// by [`apply`] but not surfaced in the scaffold.
///
/// Required fields lead, the remaining bare keys follow as one block (`note`
/// last), and the sectioned properties come last: a TOML document root ends at
/// the first table or array-of-tables header.
const FIELDS: &[Field] = &[
    Field {
        key: "fn",
        name: "FN",
        req: Req::Always,
        hint: None,
        kind: Kind::Scalar,
    },
    Field {
        key: "kind",
        name: "KIND",
        req: Req::No,
        hint: Some("e.g. individual, group, org"),
        kind: Kind::Scalar,
    },
    Field {
        key: "nickname",
        name: "NICKNAME",
        req: Req::No,
        hint: None,
        kind: Kind::List { sep: ',' },
    },
    Field {
        key: "org",
        name: "ORG",
        req: Req::No,
        hint: None,
        kind: Kind::List { sep: ';' },
    },
    Field {
        key: "title",
        name: "TITLE",
        req: Req::No,
        hint: None,
        kind: Kind::Scalar,
    },
    Field {
        key: "role",
        name: "ROLE",
        req: Req::No,
        hint: None,
        kind: Kind::Scalar,
    },
    Field {
        key: "categories",
        name: "CATEGORIES",
        req: Req::No,
        hint: None,
        kind: Kind::List { sep: ',' },
    },
    Field {
        key: "lang",
        name: "LANG",
        req: Req::No,
        hint: None,
        kind: Kind::List { sep: ',' },
    },
    Field {
        key: "bday",
        name: "BDAY",
        req: Req::No,
        hint: Some("e.g. 1990-05-23"),
        kind: Kind::Scalar,
    },
    Field {
        key: "anniversary",
        name: "ANNIVERSARY",
        req: Req::No,
        hint: Some("e.g. 2014-09-21"),
        kind: Kind::Scalar,
    },
    Field {
        key: "geo",
        name: "GEO",
        req: Req::No,
        hint: Some("e.g. geo:37.78,-122.40"),
        kind: Kind::Scalar,
    },
    Field {
        key: "tz",
        name: "TZ",
        req: Req::No,
        hint: Some("e.g. America/New_York"),
        kind: Kind::Scalar,
    },
    Field {
        key: "note",
        name: "NOTE",
        req: Req::No,
        hint: None,
        kind: Kind::Scalar,
    },
    Field {
        key: "name",
        name: "N",
        req: Req::Legacy,
        hint: None,
        kind: Kind::Structured(NAME_COMPONENTS),
    },
    Field {
        key: "gender",
        name: "GENDER",
        req: Req::No,
        hint: None,
        kind: Kind::Structured(GENDER_COMPONENTS),
    },
    Field {
        key: "email",
        name: "EMAIL",
        req: Req::No,
        hint: None,
        kind: Kind::Typed { types: PLACE_TYPES },
    },
    Field {
        key: "tel",
        name: "TEL",
        req: Req::No,
        hint: None,
        kind: Kind::Typed { types: TEL_TYPES },
    },
    Field {
        key: "address",
        name: "ADR",
        req: Req::No,
        hint: None,
        kind: Kind::TypedStructured {
            types: PLACE_TYPES,
            components: ADR_COMPONENTS,
        },
    },
    Field {
        key: "photo",
        name: "PHOTO",
        req: Req::No,
        hint: Some("e.g. file:// or http://"),
        kind: Kind::Typed { types: &[] },
    },
    Field {
        key: "url",
        name: "URL",
        req: Req::No,
        hint: None,
        kind: Kind::Typed { types: PLACE_TYPES },
    },
    Field {
        key: "impp",
        name: "IMPP",
        req: Req::No,
        hint: Some("e.g. xmpp:jane@example.com"),
        kind: Kind::Typed { types: PLACE_TYPES },
    },
];

impl Field {
    /// Whether this property is required at `version`.
    fn required(&self, version: VCardVersion) -> bool {
        match self.req {
            Req::No => false,
            Req::Always => true,
            Req::Legacy => version != VCardVersion::V4_0,
        }
    }

    /// Render this field into projected lines.
    fn lines(&self, entries: &[&VCardEntry], version: VCardVersion) -> Vec<Line> {
        let hint = if self.required(version) {
            Some("required".to_owned())
        } else {
            self.hint.map(str::to_owned)
        };

        match &self.kind {
            Kind::Scalar => {
                let value = entries
                    .first()
                    .and_then(|entry| entry_text(entry))
                    .unwrap_or_default();
                vec![Line {
                    lhs: format!("{} = {}", self.key, toml_str(value)),
                    hint,
                }]
            }

            Kind::List { .. } => {
                let items: Vec<String> = entries
                    .iter()
                    .flat_map(|entry| entry_texts(entry))
                    .collect();
                vec![Line {
                    lhs: format!("{} = {}", self.key, toml_array(&items)),
                    hint,
                }]
            }

            Kind::Structured(components) => {
                let values = entries
                    .first()
                    .map(|entry| entry_components(entry))
                    .unwrap_or_default();
                let mut lines = vec![Line {
                    lhs: format!("[{}]", self.key),
                    hint,
                }];
                lines.extend(component_lines(components, &values));
                lines
            }

            Kind::Typed { types } => {
                let mut lines = Vec::new();

                if entries.is_empty() {
                    lines.push(Line {
                        lhs: format!("[[{}]]", self.key),
                        hint: None,
                    });
                    type_line(&mut lines, "", types);
                    lines.push(Line {
                        lhs: "value = \"\"".into(),
                        hint,
                    });
                } else {
                    for entry in entries {
                        lines.push(Line {
                            lhs: format!("[[{}]]", self.key),
                            hint: None,
                        });
                        type_line(&mut lines, &type_strings(entry).join(","), types);
                        let value = entry_text(entry).unwrap_or_default();
                        lines.push(Line {
                            lhs: format!("value = {}", toml_str(value)),
                            hint: self.hint.map(str::to_owned),
                        });
                    }
                }

                lines
            }

            Kind::TypedStructured { types, components } => {
                let mut lines = Vec::new();

                if entries.is_empty() {
                    lines.push(Line {
                        lhs: format!("[[{}]]", self.key),
                        hint: None,
                    });
                    type_line(&mut lines, "", types);
                    lines.extend(component_lines(components, &[]));
                } else {
                    for entry in entries {
                        lines.push(Line {
                            lhs: format!("[[{}]]", self.key),
                            hint: None,
                        });
                        type_line(&mut lines, &type_strings(entry).join(","), types);
                        lines.extend(component_lines(components, &entry_components(entry)));
                    }
                }

                lines
            }
        }
    }

    /// Emit this field's vCard content line(s) from the edited `doc` into
    /// `out`, skipping empty values.
    fn emit(&self, doc: &DocumentMut, out: &mut String) {
        let Some(item) = doc.get(self.key) else {
            return;
        };

        match &self.kind {
            Kind::Scalar => {
                if let Some(value) = item.as_str().filter(|value| !value.is_empty()) {
                    push_line(out, &format!("{}:{}", self.name, escape(value)));
                }
            }

            Kind::List { sep } => {
                let Some(array) = item.as_array() else {
                    return;
                };

                let parts: Vec<String> = array
                    .iter()
                    .filter_map(|value| value.as_str())
                    .filter(|value| !value.is_empty())
                    .map(escape)
                    .collect();

                if !parts.is_empty() {
                    push_line(
                        out,
                        &format!("{}:{}", self.name, parts.join(&sep.to_string())),
                    );
                }
            }

            Kind::Structured(components) => {
                let Some(table) = item.as_table_like() else {
                    return;
                };

                let parts = read_components(table, components);

                if parts.iter().any(|part| !part.is_empty()) {
                    push_line(out, &format!("{}:{}", self.name, join_components(&parts)));
                }
            }

            Kind::Typed { .. } => {
                for table in tables(item) {
                    let Some(value) = table
                        .get("value")
                        .and_then(|item| item.as_str())
                        .filter(|value| !value.is_empty())
                    else {
                        continue;
                    };

                    let mut line = self.name.to_string();
                    push_type(&mut line, table);
                    line.push(':');
                    line.push_str(&escape(value));
                    push_line(out, &line);
                }
            }

            Kind::TypedStructured { components, .. } => {
                for table in tables(item) {
                    let parts = read_components(table, components);

                    if !parts.iter().any(|part| !part.is_empty()) {
                        continue;
                    }

                    let mut line = self.name.to_string();
                    push_type(&mut line, table);
                    line.push(':');
                    line.push_str(&join_components(&parts));
                    push_line(out, &line);
                }
            }
        }
    }
}

/// The column at which a block's inline `#` comments are aligned: the widest
/// left side among the lines that carry a hint.
fn comment_column<'a>(lines: impl Iterator<Item = &'a Line>) -> usize {
    lines
        .filter(|line| line.hint.is_some())
        .map(|line| line.lhs.len())
        .max()
        .unwrap_or(0)
}

/// Emit lines, padding hinted ones so their `#` lands on `column`.
fn emit_lines(out: &mut String, lines: &[Line], column: usize) {
    for line in lines {
        out.push_str(&line.lhs);

        if let Some(hint) = &line.hint {
            for _ in line.lhs.len()..column {
                out.push(' ');
            }
            out.push_str("  # ");
            out.push_str(hint);
        }

        out.push('\n');
    }
}

/// Push a `type =` line with its accepted-types hint, when the property has a
/// common type set.
fn type_line(lines: &mut Vec<Line>, value: &str, types: &[&str]) {
    if types.is_empty() {
        return;
    }

    lines.push(Line {
        lhs: format!("type = {}", toml_str(value)),
        hint: Some(format!("e.g. {}", types.join(", "))),
    });
}

/// Render named components, filled or empty, in order.
fn component_lines(components: &[Component], values: &[String]) -> Vec<Line> {
    components
        .iter()
        .enumerate()
        .map(|(index, (name, hint))| {
            let value = values.get(index).map(String::as_str).unwrap_or_default();
            Line {
                lhs: format!("{name} = {}", toml_str(value)),
                hint: hint.map(str::to_owned),
            }
        })
        .collect()
}

/// Read named components from a TOML table, escaped and in order; missing
/// components become empty strings.
fn read_components(table: &dyn TableLike, components: &[Component]) -> Vec<String> {
    components
        .iter()
        .map(|(name, _)| {
            table
                .get(name)
                .and_then(|item| item.as_str())
                .map(escape)
                .unwrap_or_default()
        })
        .collect()
}

/// Join structured components with `;`, dropping trailing empties.
fn join_components(parts: &[String]) -> String {
    let last = parts
        .iter()
        .rposition(|part| !part.is_empty())
        .map_or(0, |index| index + 1);
    parts[..last].join(";")
}

/// Append `;TYPE=<value>` to `line` when the table carries a non-empty `type`.
fn push_type(line: &mut String, table: &dyn TableLike) {
    if let Some(ty) = table
        .get("type")
        .and_then(|item| item.as_str())
        .filter(|ty| !ty.is_empty())
    {
        line.push_str(";TYPE=");
        line.push_str(ty);
    }
}

/// Push a vCard content line with CRLF, as the spec mandates.
fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push_str("\r\n");
}

/// Collect the TOML tables addressed by an array-of-tables (`[[key]]`) or an
/// inline array of inline tables.
fn tables(item: &Item) -> Vec<&dyn TableLike> {
    if let Some(array) = item.as_array_of_tables() {
        array.iter().map(|table| table as &dyn TableLike).collect()
    } else if let Some(array) = item.as_array() {
        array
            .iter()
            .filter_map(|value| value.as_inline_table())
            .map(|table| table as &dyn TableLike)
            .collect()
    } else {
        Vec::new()
    }
}

/// First value of an entry as text.
fn entry_text(entry: &VCardEntry) -> Option<&str> {
    entry.values.first().and_then(|value| value.as_text())
}

/// All texts of an entry, flattening structured components.
fn entry_texts(entry: &VCardEntry) -> Vec<String> {
    entry.values.iter().flat_map(value_strings).collect()
}

/// Ordered components of a structured entry (`N`, `ADR`, `GENDER`).
fn entry_components(entry: &VCardEntry) -> Vec<String> {
    match entry.values.first() {
        Some(VCardValue::Component(parts)) => parts.clone(),
        _ => entry
            .values
            .iter()
            .filter_map(|value| value.as_text().map(str::to_owned))
            .collect(),
    }
}

/// All texts carried by a single value.
fn value_strings(value: &VCardValue) -> Vec<String> {
    match value {
        VCardValue::Component(parts) => parts.clone(),
        other => other.as_text().map(str::to_owned).into_iter().collect(),
    }
}

/// `TYPE` parameter values of an entry, lowercased.
fn type_strings(entry: &VCardEntry) -> Vec<String> {
    entry
        .parameters(&VCardParameterName::Type)
        .filter_map(param_text)
        .collect()
}

/// Text form of a parameter value, for `TYPE`.
fn param_text(value: &VCardParameterValue) -> Option<String> {
    match value {
        VCardParameterValue::Text(text) => Some(text.clone()),
        VCardParameterValue::Type(ty) => Some(ty.as_str().to_lowercase()),
        _ => None,
    }
}

/// True unless the property is a structural marker calcard emits on its own
/// (`BEGIN`, `END`, `VERSION`).
fn is_data(name: &VCardProperty) -> bool {
    !matches!(
        name,
        VCardProperty::Begin | VCardProperty::End | VCardProperty::Version
    )
}

/// True when the property is part of the modeled vocabulary.
fn is_modeled(name: &VCardProperty) -> bool {
    FIELDS.iter().any(|field| field.name == name.as_str())
}

/// Render a string as a quoted, escaped TOML scalar.
fn toml_str(value: &str) -> String {
    toml_edit::Value::from(value).to_string().trim().to_string()
}

/// Render strings as a TOML array.
fn toml_array<S: AsRef<str>>(items: &[S]) -> String {
    let mut array = toml_edit::Array::new();

    for item in items {
        array.push(item.as_ref());
    }

    array.to_string().trim().to_string()
}

/// Escape a vCard text value per RFC 6350 section 3.4.
fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            ',' => out.push_str("\\,"),
            ';' => out.push_str("\\;"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use calcard::vcard::VCardVersion;

    use crate::vcard;

    const SAMPLE: &str = "BEGIN:VCARD\r\n\
        VERSION:4.0\r\n\
        FN:John Doe\r\n\
        N:Doe;John;;;\r\n\
        EMAIL;TYPE=work:john@work.example\r\n\
        EMAIL;TYPE=home:john@home.example\r\n\
        ADR;TYPE=home:;;123 Main St;Springfield;IL;62701;USA\r\n\
        X-CUSTOM;TYPE=weird:keep me verbatim\r\n\
        END:VCARD\r\n";

    #[test]
    fn project_prefills_known_fields() {
        let card = vcard::parse(SAMPLE).unwrap();
        let toml = super::project(&card, VCardVersion::V4_0);

        assert!(toml.contains("fn = \"John Doe\""));
        assert!(toml.contains("family = \"Doe\""));
        assert!(toml.contains("value = \"john@work.example\""));
        assert!(toml.contains("street = \"123 Main St\""));
        // Unmodeled properties never appear in the scaffold.
        assert!(!toml.contains("X-CUSTOM"));
    }

    #[test]
    fn blank_project_layout() {
        let toml = super::project(&Default::default(), VCardVersion::V4_0);

        // fn leads; categories and lang sit below role; note is the
        // last bare key; gender follows name; photo precedes url.
        assert!(toml.find("fn =").unwrap() < toml.find("kind =").unwrap());
        assert!(toml.find("role =").unwrap() < toml.find("categories =").unwrap());
        assert!(toml.find("categories =").unwrap() < toml.find("lang =").unwrap());
        assert!(toml.find("lang =").unwrap() < toml.find("note =").unwrap());
        assert!(toml.find("[name]").unwrap() < toml.find("[gender]").unwrap());
        assert!(toml.find("[[photo]]").unwrap() < toml.find("[[url]]").unwrap());

        // Empty, uncommented fields; note as a plain empty string.
        assert!(toml.contains("fn = \"\""));
        assert!(toml.contains("note = \"\""));
        assert!(!toml.contains("#fn"));

        // FN is flagged required; hints use the e.g. form.
        assert!(toml.contains("# required"));
        assert!(toml.contains("# e.g. F, M, O, N, U"));
        assert!(toml.contains("# e.g. geo:37.78,-122.40"));
        assert!(toml.contains("# e.g. home, work, cell"));
        assert!(toml.contains("# e.g. file:// or http://"));
    }

    #[test]
    fn uid_is_hidden_and_app_managed() {
        let card = vcard::parse(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nUID:urn:uuid:keep\r\nEND:VCARD\r\n",
        )
        .unwrap();

        // Hidden from the form.
        let toml = super::project(&card, VCardVersion::V4_0);
        assert!(!toml.contains("uid"));

        // Preserved on round-trip, and not overridable from the buffer.
        let out = super::apply(&card, "uid = \"hacked\"\n", VCardVersion::V4_0).unwrap();
        assert!(out.contains("UID:urn:uuid:keep"));
        assert!(!out.contains("hacked"));
    }

    #[test]
    fn n_required_only_before_v4() {
        let v4 = super::project(&Default::default(), VCardVersion::V4_0);
        let v3 = super::project(&Default::default(), VCardVersion::V3_0);

        let name_required = |toml: &str| {
            toml.lines()
                .any(|line| line.starts_with("[name]") && line.contains("required"))
        };

        assert!(!name_required(&v4));
        assert!(name_required(&v3));
    }

    #[test]
    fn blank_bare_hints_share_a_column() {
        let toml = super::project(&Default::default(), VCardVersion::V4_0);

        let column = |needle: &str| -> usize {
            let line = toml.lines().find(|line| line.contains(needle)).unwrap();
            line.find('#').unwrap()
        };

        // fn, bday, anniversary, geo, tz all align in the bare block.
        assert_eq!(column("bday ="), column("fn ="));
        assert_eq!(column("bday ="), column("anniversary ="));
        assert_eq!(column("bday ="), column("geo ="));
        assert_eq!(column("bday ="), column("tz ="));
    }

    #[test]
    fn photo_has_no_type_line() {
        let toml = super::project(&Default::default(), VCardVersion::V4_0);
        let photo = toml.split("[[photo]]").nth(1).unwrap();

        assert!(!photo.lines().take(2).any(|line| line.starts_with("type =")));
    }

    #[test]
    fn gender_roundtrips_with_identity() {
        let card = vcard::parse(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nGENDER:O;intersex\r\nEND:VCARD\r\n",
        )
        .unwrap();
        let toml = super::project(&card, VCardVersion::V4_0);

        assert!(toml.contains("sex = \"O\""));
        assert!(toml.contains("identity = \"intersex\""));

        let out = super::apply(&card, &toml, VCardVersion::V4_0).unwrap();
        assert!(out.contains("GENDER:O;intersex"));
    }

    #[test]
    fn apply_roundtrip_preserves_unknown_properties() {
        let card = vcard::parse(SAMPLE).unwrap();
        let toml = super::project(&card, VCardVersion::V4_0);

        let out = super::apply(&card, &toml, VCardVersion::V4_0).unwrap();

        assert!(out.contains("FN:John Doe"));
        assert!(out.contains("john@work.example"));
        assert!(out.contains("john@home.example"));
        // The unmodeled property survives the round-trip verbatim.
        assert!(out.contains("X-CUSTOM"));
        assert!(out.contains("keep me verbatim"));
    }

    #[test]
    fn project_then_apply_preserves_bare_fields_after_sections() {
        // These scalar/list fields are emitted before the sections so
        // TOML does not nest them inside a table; a round-trip through
        // the projected scaffold must keep every one of them.
        let filled = "BEGIN:VCARD\r\n\
            VERSION:4.0\r\n\
            FN:Ada Lovelace\r\n\
            NICKNAME:Ada\r\n\
            NOTE:Pioneer\r\n\
            CATEGORIES:science\r\n\
            UID:urn:uuid:1234\r\n\
            EMAIL;TYPE=work:ada@analytical.example\r\n\
            END:VCARD\r\n";
        let card = vcard::parse(filled).unwrap();
        let toml = super::project(&card, VCardVersion::V4_0);

        let out = super::apply(&card, &toml, VCardVersion::V4_0).unwrap();

        assert!(out.contains("NICKNAME:Ada"));
        assert!(out.contains("NOTE:Pioneer"));
        assert!(out.contains("CATEGORIES:science"));
        assert!(out.contains("UID:urn:uuid:1234"));
        assert!(out.contains("ada@analytical.example"));
    }

    #[test]
    fn apply_ignores_empty_fields() {
        let card = vcard::parse(SAMPLE).unwrap();
        // A whole blank form must drop every modeled field (all empty)
        // yet keep the unknown property.
        let blank = super::project(&Default::default(), VCardVersion::V4_0);

        let out = super::apply(&card, &blank, VCardVersion::V4_0).unwrap();

        assert!(!out.contains("FN:"));
        assert!(!out.contains("EMAIL"));
        assert!(out.contains("X-CUSTOM"));
    }

    #[test]
    fn apply_edits_modeled_field() {
        let card = vcard::parse(SAMPLE).unwrap();
        let edited = "fn = \"Jane Roe\"\n";

        let out = super::apply(&card, edited, VCardVersion::V4_0).unwrap();

        assert!(out.contains("FN:Jane Roe"));
        assert!(!out.contains("John Doe"));
        assert!(out.contains("X-CUSTOM"));
    }
}
