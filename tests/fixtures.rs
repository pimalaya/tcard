//! # Golden fixtures
//!
//! Projection and round trip over the real-world and crafted vCards in
//! tests/data.
//!
//! Each name.mode.toml is the expected projection of name.vcf, the `all` mode
//! projecting the whole file: one card flat at the root, two or more as
//! `[[card]]` blocks. CONTRIBUTING.md carries the steps for adding a case.
//!
//! Projection is deterministic, so equality is asserted for every fixture.
//! Round trip is asserted only where the source is already in the reader's
//! canonical form, which a name.lossy marker denies.
//!
//! A real export often reorders structured components or drops an unmodeled
//! parameter on read, which apply then canonicalises, so a byte-exact round
//! trip is not expected there.

use std::{fs, path::Path};

use tcard::template::Template;
use vcard::version::VcardVersion;

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

        let template = Template::parse(&vcf, VcardVersion::V4_0).unwrap();

        let projected = match mode {
            "all" => template.project(),
            other => panic!("unknown fixture mode {other:?}: {}", path.display()),
        };
        assert_eq!(
            projected,
            expected,
            "projection mismatch: {}",
            path.display()
        );

        if !dir.join(format!("{name}.lossy")).exists() {
            let round_trip = template.apply(&expected).unwrap();
            assert_eq!(round_trip, vcf, "round-trip mismatch: {}", path.display());
        }
    }
}
