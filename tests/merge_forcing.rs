//! # Merge forcing laws
//!
//! Property-based laws of the merge document's duplicate-key forcing.
//!
//! A collision the projection can address is written as the same TOML key
//! once per side, which TOML refuses, so an undecided document cannot be
//! applied at all.
//!
//! These laws pin that down from both ends: an undecided document is refused
//! and names a field that genuinely collided, keeping either side yields that
//! side, and a value of the reader's own is taken as written.
//!
//! A structured collision never escapes the single table its instance
//! projects to, where a repeated array-of-tables header would be legal TOML
//! and would quietly make a second instance instead of an error.
//!
//! What the merge settled on its own is said in the header instead, and the
//! laws below cover that too: a removal against an update, a part the
//! projection does not show, a positional pairing, and a list both sides
//! edited, whose items are all kept.

use proptest::prelude::*;
use tcard::{
    error::TcardError,
    merge::{Merge, Merged},
};

/// One property a merge can put to a reader: how to spell it in a card, the
/// TOML key its collision contests, and the section that key sits in.
#[derive(Debug)]
struct Contested {
    /// The vCard content line, `{}` standing in for the contested value.
    line: &'static str,
    /// The TOML key the collision writes once per side.
    key: &'static str,
    /// The section header the key must stay inside, for a property the
    /// projection writes as a table or an array of tables.
    header: Option<&'static str>,
}

/// The properties whose collision the projection can address, one per shape
/// of the vocabulary: a bare scalar, a component of a structured value, and
/// the value of a typed instance.
const CONTESTED: &[Contested] = &[
    Contested {
        line: "FN:{}",
        key: "full-name",
        header: None,
    },
    Contested {
        line: "NOTE:{}",
        key: "note",
        header: None,
    },
    Contested {
        line: "TITLE:{}",
        key: "title",
        header: None,
    },
    Contested {
        line: "N:{};Jane;;;",
        key: "family",
        header: Some("[name]"),
    },
    Contested {
        line: "GENDER:O;{}",
        key: "identity",
        header: Some("[gender]"),
    },
    Contested {
        line: "EMAIL;TYPE=home:{}",
        key: "value",
        header: Some("[[email]]"),
    },
    Contested {
        line: "TEL;TYPE=work:{}",
        key: "value",
        header: Some("[[phone]]"),
    },
    Contested {
        line: "ADR;TYPE=home:;;{};Springfield;IL;62701;USA",
        key: "street",
        header: Some("[[address]]"),
    },
    Contested {
        line: "URL:{}",
        key: "value",
        header: Some("[[url]]"),
    },
];

/// A card carrying one contested property at the given value.
fn card(spec: &Contested, value: &str) -> String {
    let line = spec.line.replace("{}", value);
    let anchor = if line.starts_with("FN:") {
        ""
    } else {
        "FN:Anchor\r\n"
    };

    format!("BEGIN:VCARD\r\nVERSION:4.0\r\n{anchor}{line}\r\nEND:VCARD\r\n")
}

/// A value distinct enough to be told apart in a card and in a document, and
/// plain enough to need no escaping in either.
fn value() -> impl Strategy<Value = String> {
    "[a-z]{4,8}".prop_map(String::from)
}

prop_compose! {
    /// One collision: a property, and three values for it that differ from
    /// one another, so both sides genuinely moved away from the ancestor.
    fn collision()(
        which in 0..CONTESTED.len(),
        base in value(),
        local in value(),
        remote in value(),
    ) -> (&'static Contested, String, String, String) {
        (&CONTESTED[which], format!("b{base}"), format!("l{local}"), format!("r{remote}"))
    }
}

/// The document lines that keep one side of a collision and drop the other.
fn keeping(toml: &str, dropped: &str) -> String {
    toml.lines()
        .filter(|line| !line.ends_with(dropped))
        .map(|line| format!("{line}\n"))
        .collect()
}

/// The document's header comment as one unwrapped line, so a note can be
/// looked for without minding where it happens to have been folded.
fn notes(toml: &str) -> String {
    toml.lines()
        .take_while(|line| line.starts_with('#'))
        .map(|line| line.trim_start_matches('#').trim())
        .collect::<Vec<_>>()
        .join(" ")
}

proptest! {
    /// A document holding an addressable collision does not parse, and the
    /// refusal names the field left undecided rather than reporting a
    /// syntax error. The two live lines and the commented ancestor are all
    /// there, so the reader can see what is being asked.
    #[test]
    fn an_undecided_document_is_refused_by_the_field_it_leaves_undecided(
        (spec, base, local, remote) in collision(),
    ) {
        let merged = merge(
            &card(spec, &base),
            &card(spec, &local),
            &card(spec, &remote),
        );

        let ancestor = format!("# {} = \"{}\" # base\n", spec.key, base);
        let left = format!("{} = \"{}\" # local\n", spec.key, local);
        let right = format!("{} = \"{}\" # remote\n", spec.key, remote);

        prop_assert!(merged.toml.contains("# conflict, keep one line\n"));
        prop_assert!(merged.toml.contains(&ancestor), "no ancestor: {}", merged.toml);
        prop_assert!(merged.toml.contains(&left), "no local line: {}", merged.toml);
        prop_assert!(merged.toml.contains(&right), "no remote line: {}", merged.toml);

        match merged.apply(&merged.toml) {
            Err(TcardError::Undecided(key)) => prop_assert_eq!(key, spec.key),
            other => prop_assert!(false, "not refused as undecided: {:?}", other.map(|_| ())),
        }
    }

    /// Deleting the lines of one side decides the collision for the other,
    /// in both directions. A renderer that always found the same line would
    /// pass a one-sided test, so both are asked for.
    #[test]
    fn keeping_one_side_yields_that_side((spec, base, local, remote) in collision()) {
        let merged = merge(
            &card(spec, &base),
            &card(spec, &local),
            &card(spec, &remote),
        );

        let kept_local = merged.apply(&keeping(&merged.toml, "# remote")).unwrap();
        prop_assert!(kept_local.contains(&local), "{}", kept_local);
        prop_assert!(!kept_local.contains(&remote), "{}", kept_local);

        let kept_remote = merged.apply(&keeping(&merged.toml, "# local")).unwrap();
        prop_assert!(kept_remote.contains(&remote), "{}", kept_remote);
        prop_assert!(!kept_remote.contains(&local), "{}", kept_remote);
    }

    /// Replacing every line of a collision with a value of the reader's own
    /// yields that value, neither side's.
    #[test]
    fn replacing_the_lines_yields_ones_own_value((spec, base, local, remote) in collision()) {
        let merged = merge(
            &card(spec, &base),
            &card(spec, &local),
            &card(spec, &remote),
        );

        let mine: String = merged
            .toml
            .lines()
            .filter(|line| !line.ends_with("# remote"))
            .map(|line| match line.ends_with("# local") {
                true => format!("{} = \"decided\"\n", spec.key),
                false => format!("{line}\n"),
            })
            .collect();

        let out = merged.apply(&mine).unwrap();

        prop_assert!(out.contains("decided"), "{}", out);
        prop_assert!(!out.contains(&local), "{}", out);
        prop_assert!(!out.contains(&remote), "{}", out);
    }

    /// The commented ancestor is a comment and nothing more: deleting it
    /// decides nothing and changes nothing.
    #[test]
    fn the_commented_ancestor_decides_nothing((spec, base, local, remote) in collision()) {
        let merged = merge(
            &card(spec, &base),
            &card(spec, &local),
            &card(spec, &remote),
        );

        let with = keeping(&merged.toml, "# remote");
        let without = keeping(&with, "# base");

        prop_assert_ne!(&with, &without);
        prop_assert_eq!(
            merged.apply(&with).unwrap(),
            merged.apply(&without).unwrap(),
        );
    }

    /// A collision inside a structured or typed value stays inside the one
    /// table its instance projects to. Repeating the array-of-tables header
    /// instead is legal TOML: it would make a second address rather than a
    /// parse error, and the forcing would vanish exactly where the value is
    /// most complex.
    #[test]
    fn a_structured_collision_never_repeats_its_table_header(
        (spec, base, local, remote) in collision(),
    ) {
        let Some(header) = spec.header else {
            return Ok(());
        };

        let merged = merge(
            &card(spec, &base),
            &card(spec, &local),
            &card(spec, &remote),
        );

        prop_assert_eq!(
            merged.toml.lines().filter(|line| *line == header).count(),
            1,
            "{} written more than once",
            header,
        );

        let mut written: Vec<(&str, usize)> = Vec::new();

        for line in block(&merged.toml, header) {
            let Some((key, _)) = line.split_once('=') else {
                continue;
            };

            let key = key.trim();
            match written.iter_mut().find(|(held, _)| *held == key) {
                Some((_, count)) => *count += 1,
                None => written.push((key, 1)),
            }
        }

        for (key, count) in written {
            prop_assert_eq!(count, usize::from(key == spec.key) + 1, "{}", key);
        }
    }
}

/// Merge three cards into the document a reader decides.
fn merge(base: &str, local: &str, remote: &str) -> Merged {
    Merge {
        base,
        local,
        remote,
    }
    .project()
    .unwrap()
}

/// The lines a section header opens, up to the next header.
fn block<'t>(toml: &'t str, header: &str) -> Vec<&'t str> {
    toml.lines()
        .skip_while(|line| *line != header)
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.starts_with('#'))
        .collect()
}

/// A collision the merge already settled is a header comment, not duplicate
/// keys: a removal against an update has nothing to choose, the update wins
/// whichever side it came from, and the document applies as it stands.
#[test]
fn a_removal_against_an_update_is_a_comment() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nNOTE:hi\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nNOTE:hello\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);

    assert!(notes(&merged.toml).contains("- note: removed by local, updated by remote"));
    assert!(!merged.toml.contains("# conflict"));
    assert_eq!(
        merged
            .toml
            .lines()
            .filter(|line| line.starts_with("note = "))
            .count(),
        1,
    );

    let out = merged.apply(&merged.toml).unwrap();
    assert!(out.contains("NOTE:hello\r\n"));
}

/// A collision the projection cannot address keeps the local value, says so
/// in the header, and leaves the document appliable: there is no key to
/// write twice, so there is nothing to force.
#[test]
fn an_unprojectable_collision_keeps_the_local_value_and_says_so() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nX-FOO:one\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nX-FOO:two\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nX-FOO:three\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);

    assert!(merged.vcard.contains("X-FOO:two"));
    assert!(!merged.vcard.contains("X-FOO:three"));
    assert!(notes(&merged.toml).contains("the local value was kept"));
    assert!(!merged.toml.contains("# conflict"));

    let out = merged.apply(&merged.toml).unwrap();
    assert!(out.contains("X-FOO:two"));
}

/// A note longer than the column the document is written to is folded over
/// two comment lines, the second indented under the first line's text, so the
/// header keeps the width everything below it keeps.
#[test]
fn a_long_note_wraps_under_itself() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nX-FOO:one\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nX-FOO:two\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nX-FOO:three\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);
    let header: Vec<&str> = merged
        .toml
        .lines()
        .take_while(|line| line.starts_with('#'))
        .collect();

    assert!(header.iter().all(|line| line.len() <= 68), "{header:#?}");
    assert!(
        header.iter().any(|line| line.starts_with("#   ")),
        "{header:#?}",
    );
}

/// Pairing two instances by position rather than by `PID` is said in the
/// header, because the choice below it may be pairing what the reader thinks
/// of as two different numbers.
#[test]
fn a_positional_pairing_is_said_in_the_header() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL:+1\r\nTEL:+2\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL:+1\r\nTEL:+3\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL:+1\r\nTEL:+4\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);

    assert!(notes(&merged.toml).contains("- phone: paired by position, not by PID"));
    assert!(merged.toml.contains("value = \"+3\" # local\n"));
    assert!(merged.toml.contains("value = \"+4\" # remote\n"));
}

/// Two contested instances of one repeatable property each get their own
/// contest in their own table, even when both sides spell them the same on
/// one side, which is what the value-based line search has to tell apart.
#[test]
fn two_contested_instances_holding_equal_values_are_told_apart() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL:+1\r\nTEL:+2\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL:+9\r\nTEL:+9\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL:+3\r\nTEL:+4\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);

    assert_eq!(
        merged
            .toml
            .lines()
            .filter(|line| *line == "[[phone]]")
            .count(),
        2
    );
    assert!(merged.toml.contains("# value = \"+1\" # base\n"));
    assert!(merged.toml.contains("# value = \"+2\" # base\n"));
    assert!(merged.toml.contains("value = \"+3\" # remote\n"));
    assert!(merged.toml.contains("value = \"+4\" # remote\n"));

    let decided = keeping(&merged.toml, "# local");
    let out = merged.apply(&decided).unwrap();
    assert!(out.contains("TEL:+3\r\nTEL:+4\r\n"), "{out}");
}

/// Both sides adding to a multi-valued property keeps every item, which is
/// right for a value RFC 6350 gives no order to, and the header says so, so
/// the union is something the reader reviews rather than something that
/// happened to them.
#[test]
fn a_list_union_is_said_in_the_header() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nNICKNAME:a,b\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nNICKNAME:c,d\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nNICKNAME:e,f\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);

    assert!(
        merged.vcard.contains("NICKNAME:c,d,e,f"),
        "{}",
        merged.vcard
    );
    assert!(
        notes(&merged.toml)
            .contains("- nickname: both sides changed its list; the items of both were kept"),
        "{}",
        merged.toml,
    );

    assert!(
        merged
            .toml
            .contains("nickname = [\"c\", \"d\", \"e\", \"f\"]")
    );
    let out = merged.apply(&merged.toml).unwrap();
    assert!(out.contains("NICKNAME:c,d,e,f"), "{out}");
}

/// The items of a `TYPE` are a list like any other, so both sides' types are
/// kept and the header says so.
#[test]
fn a_type_union_is_said_in_the_header() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL;TYPE=home:+1\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL;TYPE=work:+1\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL;TYPE=cell:+1\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);

    assert!(merged.vcard.contains("TYPE=work,cell"), "{}", merged.vcard);
    assert!(
        notes(&merged.toml)
            .contains("- phone: both sides changed its TYPE; the values of both were kept"),
        "{}",
        merged.toml,
    );
    assert!(
        merged.toml.contains("type = \"work,cell\""),
        "{}",
        merged.toml
    );
}

/// A collision the projection writes a key for is offered as a choice on
/// that key, not demoted to a comment saying the local value was kept.
///
/// See findings/tcard-projectable-collision-demoted-to-a-note.md.
#[test]
fn a_collision_on_a_projected_key_is_offered_as_a_choice() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nORG:A;B\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nORG:C;D\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nORG:E;F\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);

    assert!(
        merged
            .toml
            .contains("organization = [\"C\", \"D\"] # local"),
        "{}",
        merged.toml
    );
}

/// The date fields contest their key in the syntax the projection writes it
/// in, so a reader editing a contested line and an untouched one is editing
/// the same thing.
///
/// See findings/tcard-date-collision-rendered-as-a-string.md.
#[test]
fn a_date_collision_is_written_the_way_the_projection_writes_dates() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nBDAY:19960415\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nBDAY:19970415\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nBDAY:19980415\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);

    assert!(
        merged.toml.contains("birthday = 1997-04-15 # local"),
        "{}",
        merged.toml
    );
}

/// A contest is rendered in the block of the instance it belongs to, even
/// where a sibling instance happens to carry the same value on the local
/// side, which is what the value-based line search has to survive.
///
/// See findings/tcard-contest-rendered-in-the-wrong-block.md.
#[test]
fn a_contest_is_rendered_in_its_own_block() {
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL:+1\r\nTEL:+2\r\nEND:VCARD\r\n";
    let local = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL:+5\r\nTEL:+5\r\nEND:VCARD\r\n";
    let remote = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL:+1\r\nTEL:+9\r\nEND:VCARD\r\n";

    let merged = merge(base, local, remote);

    let out = merged.apply(&keeping(&merged.toml, "# local")).unwrap();

    assert!(out.contains("TEL:+5\r\nTEL:+9\r\n"), "{out}");
}
