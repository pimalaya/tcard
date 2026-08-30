//! # Merge
//!
//! Three-way merge of a vCard, projected as a TOML document to decide.
//!
//! [`project`] merges a local and a remote card against the base they both
//! diverged from, then renders the outcome through [`crate::template`], so a
//! merge is read and edited in the same ergonomic form as everything else. What
//! the merge settled on its own is already in that document. What it could not
//! settle is written twice, once per side, as duplicate TOML keys: TOML forbids
//! them, so an undecided document does not parse and [`apply`] refuses it,
//! naming the field left undecided rather than reporting a syntax error.
//! Resolving is deleting the line that is not wanted, or replacing both with a
//! value of one's own.
//!
//! Only a genuine choice is written that way. A removal meeting an update is
//! already decided (the update wins, whichever side it came from), and a
//! collision the projection does not surface has no key to contest, so both are
//! said in a comment at the top of the document instead. So is an instance the
//! merge paired by position rather than by `PID`, since the pairing behind the
//! choice may be the wrong one and only the reader can see that.
//!
//! A list both sides edited is said there too. Its items merge as a set, which
//! is right for a value RFC 6350 gives no order to, so there is nothing to
//! choose; but the merged value is one neither side wrote, and a reader who is
//! not told cannot review it.

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use log::debug;
use vcard::{
    param::VcardParam,
    tree::{
        codec::{VcardCodec, mode::VcardEscaper},
        cst::VcardCst,
        merge::{
            VcardMerge, VcardMergeAction, VcardMergeConflict, VcardMergeReport, VcardPropPath,
        },
    },
    value::VcardValue,
};

use crate::{
    error::{Result, TcardError},
    template::{
        self,
        datetime::date_rhs,
        model::{Component, FIELDS, Field, Kind},
        util::{toml_array, toml_str},
    },
    vcard::Cards,
};

/// The column a header comment wraps at, its `# ` prefix included.
const WRAP: usize = 66;

/// A merged card ready to be decided: the vCard an edited document folds back
/// onto, and that document.
pub struct Merged {
    /// The merged card, the source [`apply`] patches: every field the merge
    /// settled is already written into it.
    pub vcard: String,
    /// The TOML projection of the merged card, each undecided collision
    /// written as duplicate keys.
    pub toml: String,
}

/// Merge a local and a remote card against their common base, and project the
/// result as a TOML document to decide.
///
/// Only the first card of each input is merged: a merge projects one card, and
/// the callers of a merge hand over one card per body.
pub fn project(base: &str, local: &str, remote: &str) -> Result<Merged> {
    let base = read(base, "base")?;
    let local = read(local, "local")?;
    let remote = read(remote, "remote")?;

    let report = VcardMerge {
        base: &base,
        left: &local,
        right: &remote,
    }
    .merge();
    debug!("merged with {} collision(s)", report.conflicts.len());

    let version = report.merged.version();
    let escaper = VcardEscaper::for_version(version);
    let cards = Cards(vec![report.merged.clone()]);

    let toml = decorate(template::project(&cards, version), &base, &report, escaper);

    Ok(Merged {
        vcard: report.merged.to_string(),
        toml,
    })
}

/// Fold an edited merge document back onto the merged card.
///
/// A collision left as written holds the same key twice, which TOML refuses:
/// that parse error is reported as the field left undecided, so the reader is
/// told what to resolve instead of being shown a syntax error.
pub fn apply(vcard: &str, edited: &str) -> Result<String> {
    template::apply(vcard, edited).map_err(|err| undecided(err, edited))
}

/// Read one side of a merge as a syntax tree, named by the side it is.
fn read<'a>(text: &'a str, side: &'static str) -> Result<VcardCst<'a>> {
    VcardCst::parse(text).map_err(|err| TcardError::ReadCard {
        side,
        message: err.to_string(),
    })
}

/// Rewrite a duplicate-key parse error into the key it leaves undecided. The
/// key is read back from the edited buffer at the span the parser refused,
/// since the error itself only says that a key repeats.
fn undecided(err: TcardError, edited: &str) -> TcardError {
    let TcardError::ParseToml(parse) = &err else {
        return err;
    };

    if !parse.message().contains("duplicate key") {
        return err;
    }

    match parse.span().and_then(|span| edited.get(span)) {
        Some(key) => TcardError::Undecided(key.to_owned()),
        None => err,
    }
}

/// Decorate a projected document with what the merge could not settle on its
/// own: an undecided collision replaces the line it contests, everything else
/// is said in the header comment.
///
/// A key the document writes once is contested once, however many of its
/// parts the report reported, so a further collision on it is neither a
/// second choice nor a note.
fn decorate(
    toml: String,
    base: &VcardCst<'_>,
    report: &VcardMergeReport<'_>,
    escaper: VcardEscaper,
) -> String {
    let mut lines: Vec<String> = toml.lines().map(str::to_owned).collect();
    let mut choices: Vec<(usize, Choice)> = Vec::new();
    let mut contested: Vec<(&str, usize, &str)> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    for conflict in &report.conflicts {
        let at = path(&conflict.left);
        let choice = choice(base, report, conflict, escaper);
        let key = |choice: &Choice| (at.name.as_ref(), at.index, choice.key);
        let again = choice
            .as_ref()
            .is_some_and(|choice| contested.contains(&key(choice)));

        let contest = choice.filter(|_| !again).and_then(|choice| {
            let taken: Vec<usize> = choices.iter().map(|(at, _)| *at).collect();
            locate(&lines, &choice, &taken).map(|at| (at, choice))
        });

        match contest {
            Some((line, choice)) => {
                contested.push(key(&choice));
                choices.push((line, choice));
            }
            None if again => {}
            None => push_note(&mut notes, note(conflict)),
        }

        if let Some(note) = pairing_note(base, conflict) {
            push_note(&mut notes, note);
        }
    }

    note_unions(&mut notes, report);

    // The contested lines are rewritten from the bottom up, so an earlier
    // index stays the one that was located.
    choices.sort_by_key(|(at, _)| *at);
    for (at, choice) in choices.iter().rev() {
        lines.splice(at..=at, render(choice));
    }

    if !notes.is_empty() {
        let at = header_end(&lines);
        lines.splice(at..at, header(&notes));
    }

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// One collision the reader has to decide: a projected key, the ancestor value
/// commented above it, and the value each side proposes for it.
struct Choice {
    /// The field the contested key belongs to, which says where to look for it.
    field: &'static Field,
    /// The instance the report indexes, which says which block to look in.
    instance: usize,
    /// The contested key, a bare key or one inside the instance's table.
    key: &'static str,
    /// The ancestor value, as the right-hand side the projection would write.
    base: String,
    /// The local side's value.
    local: String,
    /// The remote side's value.
    remote: String,
}

/// The collision as a decidable choice on one projected key, or `None` when
/// the merge already decided it (a removal against an update) or when the
/// projection holds no key to contest.
fn choice(
    base: &VcardCst<'_>,
    report: &VcardMergeReport<'_>,
    conflict: &VcardMergeConflict<'_>,
    escaper: VcardEscaper,
) -> Option<Choice> {
    let at = path(&conflict.left);
    let field = field_of(&at.name)?;

    if let Some(choice) = list_choice(base, report, conflict, field) {
        return Some(choice);
    }

    let (key, mut base, local) = addressed(field, &conflict.left, escaper)?;
    let (other, _, remote) = addressed(field, &conflict.right, escaper)?;

    if key != other || local == remote {
        return None;
    }

    // NOTE: two arrivals contest the instance both sides took away, so the
    // ancestor is that instance and not the nothing an addition comes from.
    if matches!(conflict.left, VcardMergeAction::PropAdded { .. })
        && let Some(value) = vacated(report, at)
    {
        base = value_rhs(field, value, escaper);
    }

    Some(Choice {
        field,
        instance: at.index,
        key,
        base,
        local,
        remote,
    })
}

/// The ancestor a contested addition stands over: the instance both sides
/// took away and each then wrote anew.
///
/// A property whose identity is its own value cannot be seen to change, so
/// vcard-rs reports such an edit as a departure and an arrival. Two arrivals
/// collide only over a departure both sides agreed on, and that departed
/// instance is the ancestor the document comments above the choice; reading
/// the arrival alone would say the field came from nothing.
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

    // NOTE: nothing ties one arrival to one departure, so they are paired in
    // order, which is what `pairing_note` warns the reader about.
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

/// The collision on a list field the merge reported one component at a time
/// (`ORG`), as one choice on the whole array; `None` for any other field or
/// any other pair of actions.
///
/// The document writes such a field as a single key, so the reader can decide
/// it. Reporting each component apart and then finding no key for it would
/// demote a collision in plain sight to a note saying a part they cannot see
/// was contested.
fn list_choice(
    base: &VcardCst<'_>,
    report: &VcardMergeReport<'_>,
    conflict: &VcardMergeConflict<'_>,
    field: &'static Field,
) -> Option<Choice> {
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
        .map(|component| line.value.decode_at(component).join(","))
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

    Some(Choice {
        field,
        instance: at.index,
        key: field.key,
        base: toml_array(&ancestor),
        local: toml_array(&local),
        remote: toml_array(&remote),
    })
}

/// The key an action addresses in the projection, with the ancestor value and
/// the value the action proposes, both as TOML right-hand sides.
///
/// A structured value decomposes here: the merge reports which `;`-component
/// moved, and each component is one projected key, so a collision inside an
/// address contests that one key rather than the whole table. An action with
/// no projected key at all (a removal, an unmodeled parameter, a list item
/// both sides can keep) has nothing to contest.
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

/// The instance an action targets, which every action carries.
fn path<'p, 'a>(action: &'p VcardMergeAction<'a>) -> &'p VcardPropPath<'a> {
    match action {
        VcardMergeAction::PropAdded { at, .. }
        | VcardMergeAction::PropRemoved { at, .. }
        | VcardMergeAction::ValueChanged { at, .. }
        | VcardMergeAction::ValueComponentChanged { at, .. }
        | VcardMergeAction::ValueItemAdded { at, .. }
        | VcardMergeAction::ValueItemRemoved { at, .. }
        | VcardMergeAction::ParamAdded { at, .. }
        | VcardMergeAction::ParamRemoved { at, .. }
        | VcardMergeAction::ParamChanged { at, .. }
        | VcardMergeAction::ParamItemAdded { at, .. }
        | VcardMergeAction::ParamItemRemoved { at, .. } => at,
    }
}

/// The modeled field a property name projects to, its group prefix stripped.
/// An unmodeled property (a custom `X-*`, a vendor extension) has none: it is
/// kept verbatim but never shown, so a collision on it cannot be contested.
fn field_of(name: &str) -> Option<&'static Field> {
    let name = name.rsplit('.').next().unwrap_or(name);

    FIELDS
        .iter()
        .find(|field| field.name.eq_ignore_ascii_case(name))
}

/// The key a whole-value change addresses: the field's own key for a bare
/// field, the value key of an instance for a typed one. A structured value has
/// none, its keys being its components.
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

/// A whole value as the right-hand side the projection writes for this field:
/// an array for a list field, a native date for a date field, a quoted string
/// everywhere else.
fn value_rhs(field: &Field, value: &VcardValue<'_>, escaper: VcardEscaper) -> String {
    let node = value.encode(escaper);

    match field.kind {
        Kind::List { .. } => toml_array(&node.decode_at(0)),
        Kind::Date => date_rhs(&node.decode_joined_at(0)),
        // NOTE: the whole value, not its first `;`-component. A well-formed
        // text value escapes its semicolons and so has one component either
        // way; one that does not is shown as it is rather than cut short.
        _ => toml_str(&node.decode_joined()),
    }
}

/// The values of one component or one list parameter as a single quoted
/// string, the form the projection writes them in.
fn joined_rhs(values: &[Cow<'_, str>]) -> String {
    toml_str(&values.join(","))
}

/// The right-hand side of a field carrying nothing, which is also how the
/// projection writes an absent one.
fn empty_rhs(field: &Field) -> String {
    match field.kind {
        Kind::List { .. } => toml_array::<&str>(&[]),
        _ => toml_str(""),
    }
}

/// The line a choice contests in the projected document: a bare key at the
/// document root, else the key inside the block of the instance the report
/// indexes. Lines already contested by another choice are skipped, and a key
/// the version hides (a deprecated component) is not there to be found.
///
/// That index is the instance's position in the *base* card, and the document
/// projects the merged one, so the two agree only while the pairing behind
/// them was positional. The block it names is therefore taken only when it
/// holds the value the merge kept, which is the local one wherever a choice
/// is rendered at all; failing that the search falls back to the first block
/// holding that value, then to the first block holding the key.
fn locate(lines: &[String], choice: &Choice, taken: &[usize]) -> Option<usize> {
    let headers = match choice.field.kind {
        // Bare keys lead the document, before the first section header.
        Kind::Scalar | Kind::Date | Kind::List { .. } => {
            return find_key(lines, 0, choice.key, taken);
        }
        Kind::Structured(_) => headers(lines, &format!("[{}]", choice.field.key)),
        Kind::Typed { .. } | Kind::TypedStructured { .. } => {
            headers(lines, &format!("[[{}]]", choice.field.key))
        }
    };

    let indexed = headers
        .get(choice.instance)
        .and_then(|header| find_key(lines, header + 1, choice.key, taken))
        .filter(|at| rhs(&lines[*at], choice.key) == Some(choice.local.as_str()));

    if indexed.is_some() {
        return indexed;
    }

    let mut first = None;

    for header in headers {
        let Some(at) = find_key(lines, header + 1, choice.key, taken) else {
            continue;
        };

        if rhs(&lines[at], choice.key) == Some(choice.local.as_str()) {
            return Some(at);
        }

        first.get_or_insert(at);
    }

    first
}

/// The index of the line writing `key`, searched from `from` up to the end of
/// its block (the next section header).
fn find_key(lines: &[String], from: usize, key: &str, taken: &[usize]) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(from)
        .take_while(|(_, line)| !line.starts_with('['))
        .find(|(at, line)| !taken.contains(at) && rhs(line, key).is_some())
        .map(|(at, _)| at)
}

/// The indices of every line opening a block with the given section header.
fn headers(lines: &[String], header: &str) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| lhs(line) == header)
        .map(|(at, _)| at)
        .collect()
}

/// The right-hand side a line writes for `key`, or `None` when it writes
/// another key.
fn rhs<'l>(line: &'l str, key: &str) -> Option<&'l str> {
    let rest = lhs(line).strip_prefix(key)?.trim_start();
    Some(rest.strip_prefix('=')?.trim_start())
}

/// A line without its aligned inline hint, which is set off by a tab (a TOML
/// value never carries a raw one, so the split is unambiguous).
fn lhs(line: &str) -> &str {
    line.split('\t').next().unwrap_or(line).trim_end()
}

/// The lines replacing the contested one: the ancestor commented above, then
/// one live line per side, each naming its side. Two live lines of one key are
/// what makes the document refuse to parse until one of them is gone.
///
/// The ancestor is commented because keeping it is never the resolution of a
/// collision, both sides having moved away from it; and it is left out
/// entirely when it says nothing, the field having been empty.
fn render(choice: &Choice) -> Vec<String> {
    let mut lines = vec!["# conflict, keep one line".to_owned()];

    if choice.base != empty_rhs(choice.field) {
        lines.push(format!("# {} = {} # base", choice.key, choice.base));
    }

    lines.push(format!("{} = {} # local", choice.key, choice.local));
    lines.push(format!("{} = {} # remote", choice.key, choice.remote));

    lines
}

/// What the header says about a collision the document puts no choice on: a
/// removal the merge already decided in favour of the update, or a field the
/// projection does not show, where the local value was kept.
fn note(conflict: &VcardMergeConflict<'_>) -> String {
    let label = label(&conflict.left);

    match (&conflict.left, &conflict.right) {
        (VcardMergeAction::PropRemoved { .. }, _) => {
            format!("{label}: removed by local, updated by remote; the update was kept")
        }
        (_, VcardMergeAction::PropRemoved { .. }) => {
            format!("{label}: removed by remote, updated by local; the update was kept")
        }
        _ => format!("{label}: both sides changed a part not shown here; the local value was kept"),
    }
}

/// Say in the header every list both sides edited, since the merge keeps the
/// items of both and reports no conflict for them.
///
/// Merging items as a set is right for a value RFC 6350 gives no order to: two
/// sides each adding a nickname should keep both, and asking a reader to choose
/// between them would be wrong. Saying nothing is what is wrong, since the
/// merged value is then one neither side wrote and nobody was told.
fn note_unions(notes: &mut Vec<String>, report: &VcardMergeReport<'_>) {
    for action in &report.left {
        let Some((at, param)) = edited_items(action) else {
            continue;
        };

        let both = report
            .right
            .iter()
            .filter_map(edited_items)
            .any(|(other, held)| other == at && held == param);

        if both {
            push_note(notes, union_note(&label(action), param));
        }
    }
}

/// The list an action edited: the instance it belongs to, and the parameter's
/// name when it is a list parameter rather than the value itself.
fn edited_items<'p, 'a>(
    action: &'p VcardMergeAction<'a>,
) -> Option<(&'p VcardPropPath<'a>, Option<&'p str>)> {
    match action {
        VcardMergeAction::ValueItemAdded { at, .. }
        | VcardMergeAction::ValueItemRemoved { at, .. } => Some((at, None)),
        VcardMergeAction::ParamItemAdded { at, param, .. }
        | VcardMergeAction::ParamItemRemoved { at, param, .. } => Some((at, Some(param))),
        _ => None,
    }
}

/// What the header says about a list both sides edited.
fn union_note(label: &str, param: Option<&str>) -> String {
    match param {
        Some(param) => {
            format!("{label}: both sides changed its {param}; the values of both were kept")
        }
        None => format!("{label}: both sides changed its list; the items of both were kept"),
    }
}

/// What the header says when the two sides' instances were paired by position:
/// the base card holds several properties of that name and this one carries no
/// `PID`, so the pairing rests on order alone and may well have brought
/// together what the reader thinks of as two different phone numbers.
fn pairing_note(base: &VcardCst<'_>, conflict: &VcardMergeConflict<'_>) -> Option<String> {
    let at = path(&conflict.left);
    let instances: Vec<_> = base
        .props
        .iter()
        .filter(|line| line.name.get().eq_ignore_ascii_case(&at.name))
        .collect();

    if instances.len() < 2 {
        return None;
    }

    let pid = instances
        .get(at.index)?
        .params
        .iter()
        .any(|param| param.name.get().eq_ignore_ascii_case("PID"));

    if pid {
        return None;
    }

    let label = label(&conflict.left);
    Some(format!(
        "{label}: paired by position, not by PID; the pairing may be wrong"
    ))
}

/// How a property is named in a note: by its TOML key when the document shows
/// it, by its vCard name otherwise.
fn label(action: &VcardMergeAction<'_>) -> String {
    let name = &path(action).name;

    match field_of(name) {
        Some(field) => field.key.to_owned(),
        None => name.to_string(),
    }
}

/// Append a note, unless the same one was said already.
fn push_note(notes: &mut Vec<String>, note: String) {
    if !notes.contains(&note) {
        notes.push(note);
    }
}

/// The notes as a block continuing the document's header comment.
fn header(notes: &[String]) -> Vec<String> {
    let mut lines = vec!["#".to_owned(), "# Merge notes:".to_owned(), "#".to_owned()];
    lines.extend(notes.iter().flat_map(|note| comment(&format!("- {note}"))));
    lines
}

/// One comment paragraph wrapped at [`WRAP`], a continuation line of a bullet
/// indented under its text.
fn comment(text: &str) -> Vec<String> {
    let indent = if text.starts_with("- ") { "  " } else { "" };
    let mut lines = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + word.len() + 1 > WRAP {
            lines.push(format!("# {line}"));
            line = indent.to_owned();
        }

        if !line.is_empty() && !line.ends_with(' ') {
            line.push(' ');
        }

        line.push_str(word);
    }

    if !line.trim().is_empty() {
        lines.push(format!("# {line}"));
    }

    lines
}

/// Where the document's header comment ends, which is where the notes go.
fn header_end(lines: &[String]) -> usize {
    lines
        .iter()
        .position(|line| !line.starts_with('#'))
        .unwrap_or(lines.len())
}

#[cfg(test)]
mod tests {
    use alloc::{format, string::String};

    use crate::error::TcardError;

    fn card(props: &str) -> String {
        format!("BEGIN:VCARD\r\nVERSION:4.0\r\n{props}END:VCARD\r\n")
    }

    /// The document's header comment as one unwrapped line, so a note can be
    /// looked for without minding where it happens to have been folded.
    fn notes(toml: &str) -> String {
        let mut header = String::new();

        for line in toml.lines().take_while(|line| line.starts_with('#')) {
            if !header.is_empty() {
                header.push(' ');
            }

            header.push_str(line.trim_start_matches('#').trim());
        }

        header
    }

    #[test]
    fn undecided_document_does_not_apply() {
        let base = card("FN:Jane Doe\r\n");
        let local = card("FN:Jane Doe-Smith\r\n");
        let remote = card("FN:Jane A. Doe\r\n");

        let merged = super::project(&base, &local, &remote).unwrap();

        assert!(merged.toml.contains("# conflict, keep one line\n"));
        assert!(merged.toml.contains("# full-name = \"Jane Doe\" # base\n"));
        assert!(
            merged
                .toml
                .contains("full-name = \"Jane Doe-Smith\" # local\n")
        );
        assert!(
            merged
                .toml
                .contains("full-name = \"Jane A. Doe\" # remote\n")
        );

        // The same key twice is a TOML parse error, reported as the field the
        // reader still has to decide.
        let err = super::apply(&merged.vcard, &merged.toml).unwrap_err();
        assert!(
            matches!(&err, TcardError::Undecided(key) if key == "full-name"),
            "{err:?}",
        );
    }

    #[test]
    fn keeping_one_line_applies_that_value() {
        let base = card("FN:Jane Doe\r\n");
        let local = card("FN:Jane Doe-Smith\r\n");
        let remote = card("FN:Jane A. Doe\r\n");

        let merged = super::project(&base, &local, &remote).unwrap();
        let decided = merged
            .toml
            .replace("full-name = \"Jane A. Doe\" # remote\n", "");

        let out = super::apply(&merged.vcard, &decided).unwrap();

        assert!(out.contains("FN:Jane Doe-Smith\r\n"));
        assert!(!out.contains("Jane A. Doe"));
    }

    #[test]
    fn structured_collision_stays_inside_its_table() {
        let base = card("FN:Jane\r\nADR;TYPE=home:;;1 Main St;Springfield;IL;62701;USA\r\n");
        let local = card("FN:Jane\r\nADR;TYPE=home:;;2 Oak St;Springfield;IL;62701;USA\r\n");
        let remote = card("FN:Jane\r\nADR;TYPE=home:;;3 Elm St;Springfield;IL;62701;USA\r\n");

        let merged = super::project(&base, &local, &remote).unwrap();

        // One address, one contested key: repeating the [[address]] header
        // would be valid TOML and would silently make a second address.
        assert_eq!(
            merged
                .toml
                .lines()
                .filter(|line| *line == "[[address]]")
                .count(),
            1,
        );
        assert!(merged.toml.contains("# street = \"1 Main St\" # base\n"));
        assert!(merged.toml.contains("street = \"2 Oak St\" # local\n"));
        assert!(merged.toml.contains("street = \"3 Elm St\" # remote\n"));

        // The address' other keys are written once, so only the street is
        // undecided.
        assert_eq!(
            merged
                .toml
                .lines()
                .filter(|line| line.starts_with("locality = "))
                .count(),
            1,
        );

        let err = super::apply(&merged.vcard, &merged.toml).unwrap_err();
        assert!(
            matches!(&err, TcardError::Undecided(key) if key == "street"),
            "{err:?}",
        );
    }

    #[test]
    fn removal_against_update_is_a_comment() {
        let base = card("FN:Jane\r\nNOTE:hi\r\n");
        let local = card("FN:Jane\r\n");
        let remote = card("FN:Jane\r\nNOTE:hello\r\n");

        let merged = super::project(&base, &local, &remote).unwrap();

        assert!(
            notes(&merged.toml)
                .contains("- note: removed by local, updated by remote; the update was kept")
        );
        assert!(!merged.toml.contains("# conflict"));
        assert_eq!(
            merged
                .toml
                .lines()
                .filter(|line| line.starts_with("note = "))
                .count(),
            1,
        );

        // Nothing to decide, so the document applies as it stands and keeps
        // the update.
        let out = super::apply(&merged.vcard, &merged.toml).unwrap();
        assert!(out.contains("NOTE:hello\r\n"));
    }

    #[test]
    fn an_unreadable_side_is_named() {
        let base = card("FN:Jane\r\n");

        let Err(err) = super::project(&base, &base, "not a vCard at all") else {
            panic!("the unreadable side was read");
        };

        assert!(
            matches!(&err, TcardError::ReadCard { side, .. } if *side == "remote"),
            "{err:?}",
        );
    }

    #[test]
    fn positional_pairing_is_noted() {
        let base = card("FN:Jane\r\nTEL:+1\r\nTEL:+2\r\n");
        let local = card("FN:Jane\r\nTEL:+1\r\nTEL:+3\r\n");
        let remote = card("FN:Jane\r\nTEL:+1\r\nTEL:+4\r\n");

        let merged = super::project(&base, &local, &remote).unwrap();

        assert!(
            notes(&merged.toml)
                .contains("- phone: paired by position, not by PID; the pairing may be wrong")
        );
        assert!(merged.toml.contains("value = \"+3\" # local\n"));
        assert!(merged.toml.contains("value = \"+4\" # remote\n"));
    }
}
