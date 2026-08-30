//! # Projection laws
//!
//! Property-based laws of the TOML projection, which is only trustworthy if
//! folding it back is a no-op.
//!
//! An untouched document must leave the card exactly as it was, projecting
//! the folded card again must give the very same document, and a property the
//! vocabulary does not model must come out the other side byte for byte.
//!
//! The generator below writes cards the way a fold-back writes one, so a
//! failure is the projection's and not the generator's.

use proptest::prelude::*;
use tcard::template::Template;
use vcard::version::VcardVersion;

/// Read a card the way the CLI does, at the version it declares.
fn template(src: &str) -> Template<'_> {
    Template::parse(src, VcardVersion::V4_0).unwrap()
}

/// Fold an untouched projection of a card back onto its own source.
fn round_trip(src: &str) -> String {
    let template = template(src);

    template.apply(&template.project()).unwrap()
}

/// Project a card at the version it declares.
fn project(src: &str) -> String {
    template(src).project()
}

/// Wrap content lines into a vCard 4.0 file.
fn card(lines: &[String]) -> String {
    let mut out = String::from("BEGIN:VCARD\r\nVERSION:4.0\r\n");

    for line in lines {
        out.push_str(line);
        out.push_str("\r\n");
    }

    out.push_str("END:VCARD\r\n");
    out
}

/// Escape a text value the way RFC 6350 section 3.4 asks, which is how a
/// fold-back writes one.
fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace(';', "\\;")
}

/// Join structured components and drop the trailing empty ones, which is the
/// canonical spelling a fold-back emits.
fn structured(components: &[String]) -> String {
    let mut components: Vec<&String> = components.iter().collect();

    while components.last().is_some_and(|last| last.is_empty()) {
        components.pop();
    }

    components
        .iter()
        .map(|component| escape(component))
        .collect::<Vec<_>>()
        .join(";")
}

/// A non-empty text value, drawn from an alphabet that includes every
/// character RFC 6350 escapes, so escaping is exercised rather than avoided.
fn value() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            proptest::char::range('a', 'z'),
            proptest::char::range('A', 'Z'),
            proptest::char::range('0', '9'),
            Just(' '),
            Just('-'),
            Just('.'),
            Just('\''),
            Just('é'),
            Just(','),
            Just(';'),
            Just('\\'),
            Just(':'),
        ],
        1..10usize,
    )
    .prop_map(|chars| chars.into_iter().collect::<String>().trim().to_owned())
    .prop_filter("a value is not empty", |text| !text.is_empty())
}

/// The same, or nothing, for a component that may be left out.
fn component() -> impl Strategy<Value = String> {
    prop_oneof![Just(String::new()), value()]
}

/// A gender identity, which RFC 6350 section 6.2.7 makes free text.
fn identity() -> impl Strategy<Value = String> {
    prop_oneof![Just(String::new()), value()]
}

/// A `TYPE` parameter drawn from the sets the projection lists.
fn type_param() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(";TYPE=home".to_owned()),
        Just(";TYPE=work".to_owned()),
        Just(";TYPE=home,work".to_owned()),
    ]
}

/// A property name the vocabulary does not model, which the projection never
/// shows and apply must keep untouched.
fn extension() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("X-CUSTOM".to_owned()),
        Just("X-VENDOR-THING".to_owned()),
        Just("SOUND".to_owned()),
        Just("KEY".to_owned()),
        Just("ORG-DIRECTORY".to_owned()),
    ]
}

/// One unmodelled property line, parameters and all.
fn extension_line() -> impl Strategy<Value = String> {
    (extension(), prop::option::of(value()), value()).prop_map(|(name, param, text)| match param {
        Some(param) => format!("{name};X-P={};VALUE=text:{}", escape(&param), escape(&text)),
        None => format!("{name}:{}", escape(&text)),
    })
}

prop_compose! {
    /// A vCard 4.0 file built from the modelled vocabulary, written the way a
    /// fold-back writes one, plus a few properties outside that vocabulary.
    ///
    /// `ADR` is generated with the `pobox` and `ext` components vCard 4.0
    /// deprecates and the projection therefore hides, since hiding one is
    /// not licence to drop it.
    fn vcard()(
        full_name in value(),
        name in prop::option::of(prop::collection::vec(component(), 5)),
        title in prop::option::of(value()),
        note in prop::option::of(value()),
        organization in prop::collection::vec(value(), 0..3),
        categories in prop::collection::vec(value(), 0..3),
        gender in prop::option::of((prop::sample::select(vec!["F", "M", "O", "N", "U"]), identity())),
        emails in prop::collection::vec((type_param(), value()), 0..3),
        phones in prop::collection::vec((type_param(), value()), 0..3),
        addresses in prop::collection::vec((type_param(), prop::collection::vec(component(), 7)), 0..3),
        urls in prop::collection::vec(value(), 0..2),
        extensions in prop::collection::vec(extension_line(), 0..3),
    ) -> String {
        let mut lines = vec![format!("FN:{}", escape(&full_name))];

        if let Some(name) = name.filter(|name| !name.iter().all(String::is_empty)) {
            lines.push(format!("N:{}", structured(&name)));
        }
        if let Some(title) = title {
            lines.push(format!("TITLE:{}", escape(&title)));
        }
        if !organization.is_empty() {
            lines.push(format!("ORG:{}", structured(&organization)));
        }
        if !categories.is_empty() {
            let joined: Vec<String> = categories.iter().map(|item| escape(item)).collect();
            lines.push(format!("CATEGORIES:{}", joined.join(",")));
        }
        if let Some(note) = note {
            lines.push(format!("NOTE:{}", escape(&note)));
        }
        if let Some((sex, identity)) = gender {
            lines.push(format!("GENDER:{}", structured(&[sex.to_owned(), identity])));
        }
        for (param, address) in &emails {
            lines.push(format!("EMAIL{param}:{}", escape(address)));
        }
        for (param, number) in &phones {
            lines.push(format!("TEL{param}:{}", escape(number)));
        }
        for (param, components) in &addresses {
            if components.iter().all(String::is_empty) {
                continue;
            }

            lines.push(format!("ADR{param}:{}", structured(components)));
        }
        for url in &urls {
            lines.push(format!("URL:{}", escape(url)));
        }

        lines.extend(extensions);
        card(&lines)
    }
}

proptest! {
    /// The foundation: an untouched projection folds back onto the card it
    /// came from, byte for byte. Every other law rests on this one, and a
    /// card written in that canonical form has nothing to renormalise, so the
    /// comparison can be exact.
    #[test]
    fn folding_an_untouched_projection_changes_nothing(src in vcard()) {
        prop_assert_eq!(round_trip(&src), src.clone());
    }

    /// Projecting, folding and projecting again gives the very same
    /// document: the projection settles at once rather than converging over
    /// repeated edits, so a reader never sees a card move under them.
    #[test]
    fn projecting_a_folded_projection_gives_an_identical_document(src in vcard()) {
        let once = round_trip(&src);
        prop_assert_eq!(project(&once), project(&src));
        prop_assert_eq!(round_trip(&once), once.clone());
    }

    /// A property the vocabulary does not model is never shown and never
    /// touched: it comes out of the round trip byte for byte, once, and the
    /// projection never names it.
    #[test]
    fn an_unmodelled_property_survives_verbatim(src in vcard()) {
        let toml = project(&src);
        let out = round_trip(&src);

        for line in src.lines().filter(|line| is_unmodelled(line)) {
            let name = line.split([':', ';']).next().unwrap();
            prop_assert!(!toml.contains(name), "{} is shown in the projection", name);
            prop_assert_eq!(
                out.matches(line).count(),
                src.matches(line).count(),
                "{} did not survive as it was",
                line,
            );
        }
    }
}

/// Whether a content line writes a property the vocabulary does not model.
fn is_unmodelled(line: &str) -> bool {
    let name = line.split([':', ';']).next().unwrap_or(line);

    matches!(
        name,
        "X-CUSTOM" | "X-VENDOR-THING" | "SOUND" | "KEY" | "ORG-DIRECTORY"
    )
}

/// A property carried by a group survives the round trip once, rather than
/// being written out a second time without its group.
///
/// See findings/tcard-grouped-property-duplication.md.
#[test]
fn a_grouped_property_is_not_duplicated() {
    let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\n\
        item1.EMAIL:a@example.com\r\nitem1.X-ABLabel:_$!<Work>!$_\r\nEND:VCARD\r\n";

    let out = round_trip(src);

    assert_eq!(out.matches("EMAIL").count(), 1, "{out}");
    assert_eq!(out, src);
}

/// Two properties of one repeatable name keep their identity and their
/// parameters through the round trip, rather than collapsing into one value
/// that a second pass would escape into one nonsense value.
///
/// See findings/tcard-repeated-property-collapse.md.
#[test]
fn repeated_properties_of_one_name_do_not_collapse() {
    let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\n\
        LANG;PREF=1:fr\r\nLANG;PREF=2:en\r\nEND:VCARD\r\n";

    let once = round_trip(src);

    assert_eq!(
        once.lines().filter(|line| line.starts_with("LANG")).count(),
        2
    );
    assert_eq!(round_trip(&once), once);
}

/// An address keeps its post office box through the round trip, deprecated
/// or not: hiding a component from the form is not licence to drop it.
///
/// See findings/tcard-deprecated-address-components-dropped.md.
#[test]
fn a_deprecated_address_component_is_kept() {
    let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\n\
        ADR:PO Box 12;;1 Main St;Springfield;IL;62701;USA\r\nEND:VCARD\r\n";

    let out = round_trip(src);

    assert!(out.contains("PO Box 12"), "{out}");
}

/// A modelled property keeps the parameters the projection does not show,
/// the way an unmodelled property keeps everything.
///
/// See findings/tcard-modelled-property-parameter-loss.md.
#[test]
fn a_modelled_property_keeps_its_unshown_parameters() {
    for src in [
        "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nEMAIL;PREF=1;TYPE=work:a@example.com\r\nEND:VCARD\r\n",
        "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nNOTE;LANGUAGE=en:hi\r\nEND:VCARD\r\n",
        "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nTEL;VALUE=uri;TYPE=work:tel:+1\r\nEND:VCARD\r\n",
    ] {
        assert_eq!(round_trip(src), src);
    }
}

/// Every golden fixture survives repeated round trips: one pass settles the
/// card, and a second changes nothing.
///
/// See findings/tcard-grouped-property-duplication.md and
/// findings/tcard-repeated-property-collapse.md.
#[test]
fn every_fixture_settles_after_one_round_trip() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data");

    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();

        if path.extension().is_none_or(|ext| ext != "vcf") {
            continue;
        }

        let src = std::fs::read_to_string(&path).unwrap();
        let once = round_trip(&src);

        assert_eq!(round_trip(&once), once, "drifts: {}", path.display());
        assert_eq!(project(&once), project(&src), "loses: {}", path.display());
    }
}

/// An escape inside a multi-valued item comes back as it was, rather than
/// eating the space behind it.
#[test]
fn an_escape_in_a_list_item_keeps_the_space_behind_it() {
    for escaped in ["\\,", "\\;", "\\\\"] {
        let src = format!(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nCATEGORIES:a{escaped}  b\r\nEND:VCARD\r\n",
        );

        assert_eq!(round_trip(&src), src);
    }
}

/// A gender identity is free text and survives the round trip as written,
/// including a one-letter one.
#[test]
fn a_one_letter_gender_identity_keeps_its_case() {
    let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\nGENDER:F;m\r\nEND:VCARD\r\n";

    assert_eq!(round_trip(src), src);
}

/// A folded line the document leaves alone keeps its folds, and one the
/// document moves goes back out unfolded.
///
/// The wire layout is a list of offsets into the line's own bytes, so an edit
/// that changes the line's length invalidates it, and RFC 6350 section 3.2
/// recommends folding rather than requiring it.
#[test]
fn a_fold_survives_a_line_the_document_leaves_alone() {
    let folded = "NOTE:a note long enough that the exporter which wrote it folded the\r\n  line";
    let src = format!("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane\r\n{folded}\r\nEND:VCARD\r\n");

    assert_eq!(round_trip(&src), src);

    let edited = project(&src).replace("a note", "the note");
    let out = template(&src).apply(&edited).unwrap();

    assert!(
        out.contains(
            "NOTE:the note long enough that the exporter which wrote it folded the line\r\n"
        ),
        "{out}",
    );
}
