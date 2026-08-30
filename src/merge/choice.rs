//! # Choices
//!
//! A collision the reader has to decide, as the projected key it contests and
//! the value each side proposes for it.
//!
//! A collision the merge already settled, and one on a part the projection
//! does not show, make no choice: they have no key to contest and are said in
//! the document's header instead.

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::String,
    vec,
    vec::Vec,
};

use vcard::{
    param::VcardParam,
    tree::{
        codec::{VcardCodec, mode::VcardEscaper},
        cst::VcardCst,
        merge::{VcardMergeAction, VcardMergeConflict, VcardMergeReport, VcardPropPath},
    },
    value::VcardValue,
};

use crate::{
    merge::{field_of, path},
    template::{
        datetime::date_rhs,
        model::{Component, Field, Kind},
        toml::{toml_array, toml_str},
    },
};

/// One collision the reader has to decide.
///
/// A projected key, the ancestor value commented above it, and the value each
/// side proposes for it.
pub struct Choice {
    /// The field the contested key belongs to, which says where to look for it.
    pub field: &'static Field,
    /// The instance the report indexes, which says which block to look in.
    pub instance: usize,
    /// The contested key, a bare key or one inside the instance's table.
    pub key: &'static str,
    /// The ancestor value, as the right-hand side the projection would write.
    pub base: String,
    /// The local side's value.
    pub local: String,
    /// The remote side's value.
    pub remote: String,
}

impl Choice {
    /// The collision as a decidable choice on one projected key.
    ///
    /// `None` when the merge already decided it (a removal against an update)
    /// or when the projection holds no key to contest.
    pub fn new(
        base: &VcardCst<'_>,
        report: &VcardMergeReport<'_>,
        conflict: &VcardMergeConflict<'_>,
        escaper: VcardEscaper,
    ) -> Option<Self> {
        let at = path(&conflict.left);
        let field = field_of(&at.name)?;

        if let Some(choice) = Self::list(base, report, conflict, field) {
            return Some(choice);
        }

        let (key, mut base, local) = addressed(field, &conflict.left, escaper)?;
        let (other, _, remote) = addressed(field, &conflict.right, escaper)?;

        if key != other || local == remote {
            return None;
        }

        if matches!(conflict.left, VcardMergeAction::PropAdded { .. })
            && let Some(value) = vacated(report, at)
        {
            base = value_rhs(field, value, escaper);
        }

        Some(Self {
            field,
            instance: at.index,
            key,
            base,
            local,
            remote,
        })
    }

    /// The lines replacing the one this choice contests.
    ///
    /// The ancestor commented above, then one live line per side, each naming
    /// its side: two live lines of one key are what makes the document refuse
    /// to parse until one is gone. The ancestor is commented because keeping
    /// it is never a resolution, both sides having moved away from it.
    pub fn render(&self) -> Vec<String> {
        let mut lines = vec!["# conflict, keep one line".to_owned()];

        if self.base != empty_rhs(self.field) {
            lines.push(format!("# {} = {} # base", self.key, self.base));
        }

        lines.push(format!("{} = {} # local", self.key, self.local));
        lines.push(format!("{} = {} # remote", self.key, self.remote));

        lines
    }

    /// A list collision reported one component at a time, as one whole choice.
    ///
    /// `None` for any other field or pair of actions. The document writes such
    /// a field (`ORG`) as a single key, so the reader can decide it: reporting
    /// each component apart and finding no key for it would demote a collision
    /// in plain sight to a note about a part they cannot see.
    fn list(
        base: &VcardCst<'_>,
        report: &VcardMergeReport<'_>,
        conflict: &VcardMergeConflict<'_>,
        field: &'static Field,
    ) -> Option<Self> {
        if !matches!(field.kind, Kind::List { .. })
            || !matches!(
                (&conflict.left, &conflict.right),
                (
                    VcardMergeAction::ValueComponentChanged { .. },
                    VcardMergeAction::ValueComponentChanged { .. },
                ),
            )
        {
            return None;
        }

        let at = path(&conflict.left);
        let line = base
            .props
            .iter()
            .filter(|line| line.name.get().eq_ignore_ascii_case(&at.name))
            .nth(at.index)?;

        let ancestor: Vec<String> = (0..line.value.component_count())
            .map(|component| line.value.decode_component_list(component).join(","))
            .collect();

        let moved = |actions: &[VcardMergeAction<'_>]| {
            let mut components = ancestor.clone();

            for action in actions {
                if let VcardMergeAction::ValueComponentChanged {
                    at: on,
                    component,
                    new,
                    ..
                } = action
                    && on == at
                {
                    if *component >= components.len() {
                        components.resize(component + 1, String::new());
                    }
                    components[*component] = new.join(",");
                }
            }

            components
        };

        let (local, remote) = (moved(&report.left), moved(&report.right));

        if local == remote {
            return None;
        }

        Some(Self {
            field,
            instance: at.index,
            key: field.key,
            base: toml_array(&ancestor),
            local: toml_array(&local),
            remote: toml_array(&remote),
        })
    }
}

/// The right-hand side of a field carrying nothing.
///
/// It is also how the projection writes an absent one.
pub fn empty_rhs(field: &Field) -> String {
    match field.kind {
        Kind::List { .. } => toml_array::<&str>(&[]),
        _ => toml_str(""),
    }
}

/// The key an action addresses in the projection, with two values for it.
///
/// The ancestor and the proposal, both as TOML right-hand sides. A structured
/// value decomposes here, each `;`-component being one projected key, so a
/// collision inside an address contests that key rather than the whole table.
/// An action with no projected key at all has nothing to contest.
fn addressed(
    field: &'static Field,
    action: &VcardMergeAction<'_>,
    escaper: VcardEscaper,
) -> Option<(&'static str, String, String)> {
    match action {
        VcardMergeAction::ValueChanged { old, new, .. } => {
            let key = value_key(field)?;
            let old = value_rhs(field, old, escaper);
            let new = value_rhs(field, new, escaper);
            Some((key, old, new))
        }
        VcardMergeAction::ValueComponentChanged {
            component,
            old,
            new,
            ..
        } => {
            let (key, _, _) = components(field)?.get(*component)?;
            Some((key, joined_rhs(old), joined_rhs(new)))
        }
        VcardMergeAction::ParamAdded { param, .. } => {
            let new = type_values(param)?;
            Some(("type", toml_str(""), joined_rhs(new)))
        }
        VcardMergeAction::ParamRemoved { param, .. } => {
            let old = type_values(param)?;
            Some(("type", joined_rhs(old), toml_str("")))
        }
        VcardMergeAction::ParamChanged { old, new, .. } => {
            let old = type_values(old)?;
            let new = type_values(new)?;
            Some(("type", joined_rhs(old), joined_rhs(new)))
        }
        VcardMergeAction::PropAdded { prop, .. } => {
            let key = value_key(field)?;
            let new = value_rhs(field, &prop.value, escaper);
            Some((key, empty_rhs(field), new))
        }
        VcardMergeAction::PropRemoved { .. }
        | VcardMergeAction::ValueItemAdded { .. }
        | VcardMergeAction::ValueItemRemoved { .. }
        | VcardMergeAction::ParamItemAdded { .. }
        | VcardMergeAction::ParamItemRemoved { .. } => None,
    }
}

/// The ancestor a contested addition stands over.
///
/// It is the instance both sides took away and each then wrote anew. A
/// property whose identity is its own value cannot be seen to change, so
/// vcard-rs reports such an edit as a departure and an arrival, and two
/// arrivals collide only over a departure both sides agreed on. Reading the
/// arrival alone would say the field came from nothing. Nothing ties one
/// arrival to one departure, so they are paired in order, which is what the
/// pairing note warns the reader about.
fn vacated<'r, 'a>(
    report: &'r VcardMergeReport<'a>,
    at: &VcardPropPath<'_>,
) -> Option<&'r VcardValue<'a>> {
    let departures = |actions: &'r [VcardMergeAction<'a>]| {
        actions
            .iter()
            .filter_map(|action| match action {
                VcardMergeAction::PropRemoved { at: on, prop } if on.name == at.name => {
                    Some((on, &prop.value))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    let rank = report
        .left
        .iter()
        .filter_map(|action| match action {
            VcardMergeAction::PropAdded { at: on, .. } if on.name == at.name => Some(on),
            _ => None,
        })
        .position(|on| on == at)?;

    let (ours, value) = *departures(&report.left).get(rank)?;

    departures(&report.right)
        .iter()
        .any(|(theirs, _)| *theirs == ours)
        .then_some(value)
}

/// The key a whole-value change addresses.
///
/// The field's own key for a bare field, the value key of an instance for a
/// typed one. A structured value has none, its keys being its components.
fn value_key(field: &Field) -> Option<&'static str> {
    match field.kind {
        Kind::Scalar | Kind::Date | Kind::List { .. } => Some(field.key),
        Kind::Typed { .. } => Some("value"),
        Kind::Structured(_) | Kind::TypedStructured { .. } => None,
    }
}

/// The named components of a structured field, in order.
fn components(field: &Field) -> Option<&'static [Component]> {
    match field.kind {
        Kind::Structured(components) | Kind::TypedStructured { components, .. } => Some(components),
        _ => None,
    }
}

/// The values of a `TYPE` parameter, the only parameter the projection shows.
fn type_values<'p, 'a>(param: &'p VcardParam<'a>) -> Option<&'p [Cow<'a, str>]> {
    match param {
        VcardParam::Type(values) => Some(values),
        _ => None,
    }
}

/// A whole value as the right-hand side the projection writes for the field.
///
/// An array for a list field, a native date for a date field, a quoted string
/// everywhere else. That string is the whole value rather than its first
/// `;`-component: a well-formed text value escapes its semicolons, and one
/// that does not is shown as it is rather than cut short.
fn value_rhs(field: &Field, value: &VcardValue<'_>, escaper: VcardEscaper) -> String {
    let node = value.encode(escaper);

    match field.kind {
        Kind::List { .. } => toml_array(&node.decode_component_list(0)),
        Kind::Date => date_rhs(&node.decode_component(0)),
        _ => toml_str(&node.decode()),
    }
}

/// One component or list parameter, quoted the way the projection writes it.
fn joined_rhs(values: &[Cow<'_, str>]) -> String {
    toml_str(&values.join(","))
}
