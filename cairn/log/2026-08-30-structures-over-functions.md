---
cairn: log
change: structures-over-functions
date: 2026-08-30
---

# An entry point names what it is given

The free functions are gone. `Cards::parse` reads a stream, `Template { cards, version }` projects a form and folds one back, and `Merge { base, local, remote }.project()` yields a `Merged` whose `apply` takes the edited document. Nothing public is a function over positional arguments any more, which is the shape vcard-rs took one version below with `VcardMerge`.

## Both halves of a round trip

`Template` holds the cards, so `apply` no longer takes the source text a second time: it clones the tree the form was projected from and patches that. The one-reader-per-body requirement used to rest on every caller passing the same card twice, and now it cannot be broken from outside. `Template::parse` also absorbs the version resolution the CLI and both test harnesses each did by hand, the card's own version winning over the requested one.

## Two files became eight

cli.rs kept `Cli` and `Command` and gave up the rest: args holds the source, the version and the output sink, editor the `$EDITOR` round trip and its re-edit prompt, and template, edit and merge one verb each. merge.rs kept `Merge`, `Merged` and the two helpers its children share, and gave choice the collision-to-key reading, document the projected lines it writes into, and note what carries no key at all. `Notes` and `Document` are now types with methods rather than functions taking a `&mut Vec<String>` along.

## The projection's own layer

The same reading went one level down, where the free functions were densest. patch is now `Content`, one content line with the whole grammar on it: `prefix`, `value`, `items` and `types` read it as the card wrote it, `text` and `texts` read it unescaped, and `rewritten` patches its prefix. The RFC 6350 escapes moved there too, since they are that grammar rather than a TOML detail, and `rewritten`'s `Option<&str>` name argument split off into `named`, which was the only thing the new-line branch ever used.

util is gone, and what it held is where it belongs: the TOML side kept its own module under the honest name, the line readers became methods on `Content`, and joining structured components moved next to the only field that joins them. line.rs gained `Lines`, a block that knows its own comment column, so emitting one is `lines.emit(out)` rather than a width computed by one function and passed to another.

That is 12 free functions fewer, and the two remaining collections of them, the date conversions and the TOML rendering, are conversions with no receiver to hang off.

## Smaller things

thiserror is gone: four variants, four match arms, and `source` returning the TOML parse error. The core builds with one dependency fewer.

The product is written tCard in prose, `tcard` staying the crate, the binary and the module. The generated document says so in its own header, which is where a reader meets the name first, so the golden fixtures moved with it.

The inline comments that narrated what the code below them did are gone, in the sources and in the tests. Four remain, each carrying something the code cannot say: why the contested lines are spliced from the bottom up, and what a value read whole rather than by component avoids. The rest moved into the doc comment of the function or the test that needed it.

## Verification

The whole suite is green unchanged in what it asserts, 56 tests over the unit, fixture, forcing and projection layers, plus clippy, rustdoc and both feature builds. The fixtures moved only in the header line naming the product.

Capabilities moved: `api` (new), `documentation`.
