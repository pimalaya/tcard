//! Golden fixture tests over real-world and crafted vCards.
//!
//! Each `tests/data/<name>.<mode>.toml` is the expected projection of
//! `tests/data/<name>.vcf` for `<mode>` (`all` projects the whole file: a
//! single card flat at the root, two or more as `[[card]]` blocks). To add a
//! case (e.g. from a bug report), drop the `.vcf` in and generate the `.toml`
//! with `tcard template`.
//!
//! Projection is deterministic, so equality is asserted for every fixture.
//! Round-trip is checked only for fixtures whose source is already in
//! calcard's canonical form (no `.lossy` marker file): real exports often
//! reorder structured components or drop unmodeled parameters on read, which
//! apply then canonicalises, so byte-exact round-trip is not expected there.

use std::{fs, path::Path};

use calcard::vcard::VCardVersion;

#[test]
fn fixtures_project_and_round_trip() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data");

    let mut paths: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    paths.sort();

    assert!(!paths.is_empty(), "no fixtures in {}", dir.display());

    for path in paths {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let (name, mode) = stem
            .rsplit_once('.')
            .expect("fixture must be named <name>.<mode>.toml");

        let vcf = fs::read_to_string(dir.join(format!("{name}.vcf"))).unwrap();
        let expected = fs::read_to_string(&path).unwrap();

        let cards = tcard::vcard::parse_all(&vcf).unwrap();
        let version = cards
            .first()
            .and_then(|card| card.version())
            .unwrap_or(VCardVersion::V4_0);

        let projected = match mode {
            "all" => tcard::template::project(&cards, version),
            other => panic!("unknown fixture mode {other:?}: {}", path.display()),
        };
        assert_eq!(
            projected,
            expected,
            "projection mismatch: {}",
            path.display()
        );

        // Untouched, the projection folds back onto the source byte-for-byte,
        // unless the source is flagged `.lossy` (calcard canonicalises it).
        if !dir.join(format!("{name}.lossy")).exists() {
            let round_trip = tcard::template::apply(&vcf, &expected).unwrap();
            assert_eq!(round_trip, vcf, "round-trip mismatch: {}", path.display());
        }
    }
}
