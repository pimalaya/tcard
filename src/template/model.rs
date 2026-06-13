//! The modeled vCard vocabulary: how each property maps to a TOML key and
//! how it projects and reads back.

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use calcard::vcard::{VCardEntry, VCardVersion};
use toml_edit::TableLike;

use crate::template::{
    line::Line,
    util::{
        entry_components, entry_text, entry_texts, escape, join_components, push_type,
        read_components, tables, toml_array, toml_str, type_strings,
    },
};

/// A named component of a structured value: TOML key, optional hint, and
/// whether it is deprecated (hidden in vCard 4.0, flagged `# deprecated` in
/// older versions).
type Component = (&'static str, Option<&'static str>, bool);

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

    /// Repeated or multi-valued text, joined on `sep` in the vCard
    /// (`NICKNAME`, `CATEGORIES`, `ORG`).
    List { sep: char },

    /// One structured value with named, ordered components (`N`, `GENDER`).
    Structured(&'static [Component]),

    /// Repeatable property with an optional `TYPE` and a single value
    /// (`EMAIL`, `TEL`, `URL`, `PHOTO`).
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
    pub fn is_simple(&self) -> bool {
        matches!(self, Kind::Scalar | Kind::List { .. })
    }
}

/// A modeled vCard property and how it maps to TOML.
pub struct Field {
    /// TOML key.
    pub key: &'static str,
    /// Canonical vCard property name (matches calcard's `as_str`).
    pub name: &'static str,
    /// Whether the property is required.
    pub req: Req,
    /// Inline hint shown next to the value, only where it is not self-evident
    /// (rendered as ` # <hint>`).
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

/// `ADR` components, in RFC 6350 order. `pobox` and `ext` are deprecated by
/// RFC 6350: put the box and any suite/floor in `street` instead.
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
/// by apply but not surfaced in the scaffold.
///
/// Required fields lead, the remaining bare keys follow as one block (`note`
/// last), and the sectioned properties come last: a TOML document root ends at
/// the first table or array-of-tables header.
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
        hint: Some("1990-05-23"),
        kind: Kind::Scalar,
    },
    Field {
        key: "anniversary",
        name: "ANNIVERSARY",
        req: Req::No,
        hint: Some("2014-09-21"),
        kind: Kind::Scalar,
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
    fn required(&self, version: VCardVersion) -> bool {
        match self.req {
            Req::No => false,
            Req::Always => true,
            Req::Legacy => version != VCardVersion::V4_0,
        }
    }

    /// Render this field into projected lines. Sectioned kinds head their
    /// blocks under `prefix` (e.g. `vcard`): flat (`None`) gives `[name]`
    /// and `[[email]]`, a card block gives `[card.name]` / `[[card.email]]`.
    pub fn lines(
        &self,
        entries: &[&VCardEntry],
        version: VCardVersion,
        prefix: Option<&str>,
    ) -> Vec<Line> {
        let hint = if self.required(version) {
            Some("required".to_owned())
        } else {
            self.hint.map(str::to_owned)
        };
        let header = section_header(prefix, self.key);

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
                    lhs: format!("[{header}]"),
                    hint,
                }];
                lines.extend(component_lines(components, &values, version));
                lines
            }

            Kind::Typed { types } => {
                let mut lines = Vec::new();

                if entries.is_empty() {
                    lines.push(Line {
                        lhs: format!("[[{header}]]"),
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
                            lhs: format!("[[{header}]]"),
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
                        lhs: format!("[[{header}]]"),
                        hint: None,
                    });
                    type_line(&mut lines, "", types);
                    lines.extend(component_lines(components, &[], version));
                } else {
                    for entry in entries {
                        lines.push(Line {
                            lhs: format!("[[{header}]]"),
                            hint: None,
                        });
                        type_line(&mut lines, &type_strings(entry).join(","), types);
                        lines.extend(component_lines(
                            components,
                            &entry_components(entry),
                            version,
                        ));
                    }
                }

                lines
            }
        }
    }

    /// This field's vCard content line(s) built from a TOML table (a single
    /// `[[card]]` table), without an end of line, skipping empty values.
    /// Empty when the field is absent or blank, so
    /// [`crate::edit::tree::Component::set_all`] removes it.
    pub fn content_lines(&self, source: &dyn TableLike) -> Vec<String> {
        let Some(item) = source.get(self.key) else {
            return Vec::new();
        };

        let mut lines = Vec::new();

        match &self.kind {
            Kind::Scalar => {
                if let Some(value) = item.as_str().filter(|value| !value.is_empty()) {
                    lines.push(format!("{}:{}", self.name, escape(value)));
                }
            }

            Kind::List { sep } => {
                if let Some(array) = item.as_array() {
                    let parts: Vec<String> = array
                        .iter()
                        .filter_map(|value| value.as_str())
                        .filter(|value| !value.is_empty())
                        .map(escape)
                        .collect();

                    if !parts.is_empty() {
                        lines.push(format!("{}:{}", self.name, parts.join(&sep.to_string())));
                    }
                }
            }

            Kind::Structured(components) => {
                if let Some(table) = item.as_table_like() {
                    let parts = read_components(table, components);

                    if parts.iter().any(|part| !part.is_empty()) {
                        lines.push(format!("{}:{}", self.name, join_components(&parts)));
                    }
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
                    lines.push(line);
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
                    lines.push(line);
                }
            }
        }

        lines
    }
}

/// The TOML header for a section `key` under an optional parent `prefix`:
/// `"key"` at the top level (flat), else `"prefix.key"`.
fn section_header(prefix: Option<&str>, key: &str) -> String {
    match prefix {
        Some(prefix) => format!("{prefix}.{key}"),
        None => key.to_owned(),
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
        hint: Some(types.join(", ")),
    });
}

/// Render named components, filled or empty, in order. A deprecated component
/// is hidden in vCard 4.0 and flagged `# deprecated` in older versions; either
/// way its positional slot is preserved on apply, read back by key.
fn component_lines(
    components: &[Component],
    values: &[String],
    version: VCardVersion,
) -> Vec<Line> {
    components
        .iter()
        .enumerate()
        .filter(|(_, component)| !component.2 || version != VCardVersion::V4_0)
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
