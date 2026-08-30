//! # Modelled vocabulary
//!
//! The vCard properties the form shows: how each maps to a TOML key, and how
//! it projects and reads back.

use core::slice;

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use toml_edit::TableLike;
use vcard::version::VcardVersion;

use crate::template::{
    datetime::{date_rhs, toml_date_value},
    line::Line,
    patch,
    util::{
        escape, items, join_components, read_components, read_type, tables, text, toml_array,
        toml_str, types,
    },
};

/// A named component of a structured value.
///
/// Its TOML key, an optional hint, and whether it is deprecated: hidden in
/// vCard 4.0, flagged `# deprecated` in older versions.
pub type Component = (&'static str, Option<&'static str>, bool);

/// Whether a property is required, possibly only in legacy versions.
#[derive(Clone, Copy)]
pub enum Req {
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
pub enum Kind {
    /// Single text value (`FN`, `NOTE`, ...).
    Scalar,
    /// Date or date-time (`BDAY`, `ANNIVERSARY`).
    ///
    /// A native TOML value when complete, a quoted RFC 6350 string for a
    /// partial (yearless) one.
    Date,
    /// Repeated or multi-valued text, joined on `sep` in the vCard.
    ///
    /// `NICKNAME`, `CATEGORIES` and `ORG` take this shape.
    List {
        /// The character joining the items on the vCard side.
        sep: char,
    },
    /// One structured value with named, ordered components (`N`, `GENDER`).
    Structured(&'static [Component]),
    /// Repeatable property with an optional `TYPE` and a single value.
    ///
    /// `EMAIL`, `TEL`, `URL` and `PHOTO` take this shape.
    Typed {
        /// The `TYPE` values the form lists as accepted.
        types: &'static [&'static str],
    },
    /// Repeatable property with an optional `TYPE` and components.
    ///
    /// Named and ordered, as `ADR` has them.
    TypedStructured {
        /// The `TYPE` values the form lists as accepted.
        types: &'static [&'static str],
        /// The named components, in the order the vCard writes them.
        components: &'static [Component],
    },
}

impl Kind {
    /// A bare key (vs a `[table]` / `[[array]]` section).
    pub fn is_simple(&self) -> bool {
        matches!(self, Kind::Scalar | Kind::Date | Kind::List { .. })
    }
}

/// A modeled vCard property and how it maps to TOML.
pub struct Field {
    /// TOML key.
    pub key: &'static str,
    /// Canonical vCard property name.
    pub name: &'static str,
    /// Whether the property is required.
    pub req: Req,
    /// Inline hint shown next to the value, where it is not self-evident.
    ///
    /// Rendered as ` # <hint>`.
    pub hint: Option<&'static str>,
    /// Mapping shape.
    pub kind: Kind,
}

/// `N` components, in RFC 6350 order.
const NAME_COMPONENTS: &[Component] = &[
    ("family", None, false),
    ("given", None, false),
    ("additional", None, false),
    ("prefixes", None, false),
    ("suffixes", None, false),
];

/// `ADR` components, in RFC 6350 order.
///
/// RFC 6350 deprecates `pobox` and `ext`: put the box and any suite or floor
/// in `street` instead.
const ADR_COMPONENTS: &[Component] = &[
    ("pobox", None, true),
    ("ext", None, true),
    ("street", None, false),
    ("locality", None, false),
    ("region", None, false),
    ("code", None, false),
    ("country", None, false),
];

/// `GENDER` components: sex code plus a free-text identity.
const GENDER_COMPONENTS: &[Component] = &[
    ("sex", Some("F, M, O, N, U"), false),
    ("identity", None, false),
];

/// The `TYPE` set of a property naming a place (`EMAIL`, `ADR`, `URL`).
const PLACE_TYPES: &[&str] = &["home", "work"];
/// The `TYPE` set of `TEL`, richer than the place one (RFC 6350 6.4.1).
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

/// The modeled vocabulary, everything outside it kept verbatim but unshown.
///
/// Required fields lead, the remaining bare keys follow as one block (`note`
/// last), then the sectioned properties: a TOML document root ends at the
/// first table or array-of-tables header.
pub const FIELDS: &[Field] = &[
    Field {
        key: "full-name",
        name: "FN",
        req: Req::Always,
        hint: None,
        kind: Kind::Scalar,
    },
    Field {
        key: "kind",
        name: "KIND",
        req: Req::No,
        hint: Some("individual, group, org"),
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
        key: "organization",
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
        key: "language",
        name: "LANG",
        req: Req::No,
        hint: Some("en, fr"),
        kind: Kind::List { sep: ',' },
    },
    Field {
        key: "birthday",
        name: "BDAY",
        req: Req::No,
        hint: Some("1996-04-15, or \"--0415\" without a year"),
        kind: Kind::Date,
    },
    Field {
        key: "anniversary",
        name: "ANNIVERSARY",
        req: Req::No,
        hint: Some("2009-08-08"),
        kind: Kind::Date,
    },
    Field {
        key: "geo",
        name: "GEO",
        req: Req::No,
        hint: Some("geo:37.78,-122.40"),
        kind: Kind::Scalar,
    },
    Field {
        key: "timezone",
        name: "TZ",
        req: Req::No,
        hint: Some("America/New_York"),
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
        hint: Some("email address"),
        kind: Kind::Typed { types: PLACE_TYPES },
    },
    Field {
        key: "phone",
        name: "TEL",
        req: Req::No,
        hint: Some("+1-555-0100"),
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
        hint: Some("file:// or https://"),
        kind: Kind::Typed { types: &[] },
    },
    Field {
        key: "url",
        name: "URL",
        req: Req::No,
        hint: Some("https://example.com"),
        kind: Kind::Typed { types: PLACE_TYPES },
    },
    Field {
        key: "messaging",
        name: "IMPP",
        req: Req::No,
        hint: Some("xmpp:jane@example.com"),
        kind: Kind::Typed { types: PLACE_TYPES },
    },
];

impl Field {
    /// Whether this property is required at `version`.
    fn required(&self, version: VcardVersion) -> bool {
        match self.req {
            Req::No => false,
            Req::Always => true,
            Req::Legacy => version != VcardVersion::V4_0,
        }
    }

    /// Render this field into projected lines, read from the card's own ones.
    ///
    /// A sectioned kind heads its block under `prefix`: `None` gives `[name]`
    /// and `[[email]]` at the top level, `card` gives `[card.name]` and
    /// `[[card.email]]`.
    pub fn lines(&self, held: &[String], version: VcardVersion, prefix: Option<&str>) -> Vec<Line> {
        let hint = if self.required(version) {
            Some("required".to_owned())
        } else {
            self.hint.map(str::to_owned)
        };
        let header = section_header(prefix, self.key);

        match &self.kind {
            Kind::Scalar => {
                let value = held.first().map(|line| text(line)).unwrap_or_default();
                vec![Line {
                    lhs: format!("{} = {}", self.key, toml_str(&value)),
                    hint,
                }]
            }

            Kind::Date => {
                let rhs = match held.first() {
                    Some(line) => date_rhs(&text(line)),
                    None => toml_str(""),
                };
                vec![Line {
                    lhs: format!("{} = {}", self.key, rhs),
                    hint,
                }]
            }

            Kind::List { sep } => {
                let values: Vec<String> = held.iter().flat_map(|line| items(line, *sep)).collect();
                vec![Line {
                    lhs: format!("{} = {}", self.key, toml_array(&values)),
                    hint,
                }]
            }

            Kind::Structured(components) => {
                let values = held
                    .first()
                    .map(|line| items(line, ';'))
                    .unwrap_or_default();
                let mut lines = vec![Line {
                    lhs: format!("[{header}]"),
                    hint,
                }];
                lines.extend(component_lines(components, &values, version));
                lines
            }

            Kind::Typed { types: accepted } => {
                let mut lines = Vec::new();

                if held.is_empty() {
                    lines.push(Line {
                        lhs: format!("[[{header}]]"),
                        hint: None,
                    });
                    type_line(&mut lines, "", accepted);
                    lines.push(Line {
                        lhs: "value = \"\"".into(),
                        hint,
                    });
                } else {
                    for line in held {
                        lines.push(Line {
                            lhs: format!("[[{header}]]"),
                            hint: None,
                        });
                        type_line(&mut lines, &types(line), accepted);
                        lines.push(Line {
                            lhs: format!("value = {}", toml_str(&text(line))),
                            hint: self.hint.map(str::to_owned),
                        });
                    }
                }

                lines
            }

            Kind::TypedStructured {
                types: accepted,
                components,
            } => {
                let mut lines = Vec::new();

                if held.is_empty() {
                    lines.push(Line {
                        lhs: format!("[[{header}]]"),
                        hint: None,
                    });
                    type_line(&mut lines, "", accepted);
                    lines.extend(component_lines(components, &[], version));
                } else {
                    for line in held {
                        lines.push(Line {
                            lhs: format!("[[{header}]]"),
                            hint: None,
                        });
                        type_line(&mut lines, &types(line), accepted);
                        lines.extend(component_lines(components, &items(line, ';'), version));
                    }
                }

                lines
            }
        }
    }

    /// This field's content lines, built from a TOML table, without line ends.
    ///
    /// Empty when absent or blank, so [`crate::vcard::Card::set_lines`] drops
    /// it. `originals` are the card's own lines in projection order, patched
    /// not rebuilt so what the document does not write survives. An empty item
    /// leaves a `,` list, saying nothing, and holds its slot in a `;` list.
    pub fn content_lines(&self, source: &dyn TableLike, originals: &[String]) -> Vec<String> {
        let Some(item) = source.get(self.key) else {
            return Vec::new();
        };

        let mut lines = Vec::new();

        match &self.kind {
            Kind::Scalar => {
                if let Some(value) = item.as_str().filter(|value| !value.is_empty()) {
                    lines.push(self.line(originals.first(), None, &escape(value)));
                }
            }

            Kind::Date => {
                if let Some(dtm) = item.as_datetime() {
                    lines.push(self.line(originals.first(), None, &toml_date_value(dtm)));
                } else if let Some(value) = item.as_str().filter(|value| !value.is_empty()) {
                    lines.push(self.line(originals.first(), None, &escape(value)));
                }
            }

            Kind::List { sep } => {
                if let Some(array) = item.as_array() {
                    let ordered = *sep == ';';
                    let parts: Vec<String> = array
                        .iter()
                        .filter_map(|value| value.as_str())
                        .filter(|value| ordered || !value.is_empty())
                        .map(escape)
                        .collect();

                    let parts = match parts.iter().all(String::is_empty) {
                        true => Vec::new(),
                        false => parts,
                    };

                    for (original, items) in spread(&parts, originals, *sep) {
                        let value = items.join(&sep.to_string());
                        lines.push(self.line(original, None, &value));
                    }
                }
            }

            Kind::Structured(components) => {
                if let Some(table) = item.as_table_like() {
                    let original = originals.first();
                    let parts = read_components(table, components, original);

                    if parts.iter().any(|part| !part.is_empty()) {
                        lines.push(self.line(original, None, &join_components(&parts)));
                    }
                }
            }

            Kind::Typed { types } => {
                for (instance, table) in tables(item).into_iter().enumerate() {
                    let Some(value) = table
                        .get("value")
                        .and_then(|item| item.as_str())
                        .filter(|value| !value.is_empty())
                    else {
                        continue;
                    };

                    let original = originals.get(instance);
                    lines.push(self.line(original, shown(types, table), &escape(value)));
                }
            }

            Kind::TypedStructured { types, components } => {
                for (instance, table) in tables(item).into_iter().enumerate() {
                    let original = originals.get(instance);
                    let parts = read_components(table, components, original);

                    if !parts.iter().any(|part| !part.is_empty()) {
                        continue;
                    }

                    let value = join_components(&parts);
                    lines.push(self.line(original, shown(types, table), &value));
                }
            }
        }

        lines
    }

    /// One content line for this field.
    ///
    /// The value behind the prefix its own line carried, or behind the bare
    /// property name when the line is new.
    fn line(&self, original: Option<&String>, types: Option<&str>, value: &str) -> String {
        let original = original.map(String::as_str);
        format!("{}:{value}", patch::rewritten(original, self.name, types))
    }
}

/// Spread a field's items over the lines they came from.
///
/// Each original keeps the items it held and a surplus opens its own line, so
/// two properties of one name (`LANG;PREF=1:fr`, `LANG;PREF=2:en`) never
/// collapse. A `;` instead joins one property's components (`ORG`), one line.
fn spread<'i, 'o>(
    items: &'i [String],
    originals: &'o [String],
    sep: char,
) -> Vec<(Option<&'o String>, &'i [String])> {
    if items.is_empty() {
        return Vec::new();
    }

    if sep == ';' {
        return vec![(originals.first(), items)];
    }

    let mut out = Vec::new();
    let mut rest = items;

    for original in originals {
        if rest.is_empty() {
            break;
        }

        let held = patch::items(patch::value(original), sep)
            .len()
            .clamp(1, rest.len());
        let (head, tail) = rest.split_at(held);
        out.push((Some(original), head));
        rest = tail;
    }

    out.extend(rest.iter().map(|item| (None, slice::from_ref(item))));
    out
}

/// The TOML header for a section `key` under an optional parent `prefix`.
///
/// `"key"` at the top level, else `"prefix.key"`.
fn section_header(prefix: Option<&str>, key: &str) -> String {
    match prefix {
        Some(prefix) => format!("{prefix}.{key}"),
        None => key.to_owned(),
    }
}

/// The types a TOML table writes.
///
/// `None` for a property whose projection lists no type at all (`PHOTO`),
/// whose own ones are therefore not the document's to clear.
fn shown<'t>(types: &[&str], table: &'t dyn TableLike) -> Option<&'t str> {
    (!types.is_empty()).then(|| read_type(table))
}

/// Push a `type =` line with its accepted-types hint.
///
/// Nothing is pushed for a property with no common type set.
fn type_line(lines: &mut Vec<Line>, value: &str, types: &[&str]) {
    if types.is_empty() {
        return;
    }

    lines.push(Line {
        lhs: format!("type = {}", toml_str(value)),
        hint: Some(types.join(", ")),
    });
}

/// Render named components, filled or empty, in order.
///
/// A deprecated component is hidden in vCard 4.0 and flagged in older
/// versions; either way its positional slot survives apply, read back by key.
fn component_lines(
    components: &[Component],
    values: &[String],
    version: VcardVersion,
) -> Vec<Line> {
    components
        .iter()
        .enumerate()
        .filter(|(_, component)| !component.2 || version != VcardVersion::V4_0)
        .map(|(index, (name, hint, deprecated))| {
            let value = values.get(index).map(String::as_str).unwrap_or_default();
            let hint = if *deprecated {
                Some("deprecated".to_owned())
            } else {
                hint.map(str::to_owned)
            };
            Line {
                lhs: format!("{name} = {}", toml_str(value)),
                hint,
            }
        })
        .collect()
}
