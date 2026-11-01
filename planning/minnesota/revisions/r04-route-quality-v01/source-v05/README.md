# Civic park pond, bridge and eastern shoulders — source v05

Source composition is approved for bounded native integration. Native publication,
GPU review and production-controller traversal are **pending**. The wider route
quality objective remains open.

The exact input is `content-packs/minnesota-river-long-v02.crystalpack`, file SHA
`3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243`, identity
`3ead916d12c705f2c0559c98b983f5af4760c9a8f557d8e31f62b36ea197e29a`.
Its complete R04 module remains
`ca7e08ba7ec44571661298e28ab8d21560f5287eccfee5d5797983f68d47d6ca`.
`native-preimage.json` records the actual module, alias art and encoded mirror
preimages. No old pond source-v04, canonical route, world or interior was edited.

## Exact bounded change

`native.contract.json` SHA
`7c55b8813a53655daaa8dba88507e7a2f4d7a8e4b4d3c8ee2f90f7c1973a7cfb`
binds 57 distinct metatile changes: 45 close the unconnected pond ends, six
add blocking gold bridge rails, and six ordinary elm shoulders tighten the
underused eastern apron. Four metatiles `44..47` append exact old native glyph
rows, with four `WALL` quadrants each. No glyph/palette/old collision changes.
The existing bridge sign keeps its root, position and second line; only its
first text line becomes `PARK POND BRIDGE`, across global raw, cached raw and
typed body mirrors.

The park remains a contained civic pond crossed by its original Foot-only
bridge, not a Mississippi Surf-through route. Bridge walking rows 18/19 and
bank passing room remain. Both park ports, trainer bodies/sight, encounters,
items, flags, scripts and all unrelated pack content remain exact. Elm `00`
contains no Cut quadrant. The existing picnic bench `(24,12)`, cache `(49,27)`
and four-grass approach remain. No extra picnic prop is added.

The full vertical walking runs are x40:12→10 (useful picnic junction), x42:7→5
(including the walkable upper fence half), and x50:6→2. This is a composition
choice; no universal road width is claimed. See the preserved audit for the
rejected duplicate-bench proposal and the remaining flower/habitat shape limits.

## Review and source proofs

- `compiled-candidate.blocks.json` is the exact 28×18 native candidate array;
  the authored layout produces it directly.
- Full native-pixel source comparison and 12 player views:
  `planning/minnesota/renders/r04-route-quality-v01/combined-source-v05-v02/`.
  I personally inspected the whole comparison and every player view. The west
  choice and picnic cameras have zero RGB pixel differences.
- Four-bank source art proof:
  `planning/minnesota/renders/r04-route-quality-v01/combined-source-v05-art-proof-v01/`.
  All 278,528 old-block pixels and 16,384 appended donor-row pixels compare
  exactly. This is source-pixel evidence, not GPU equality.
- `controller-input-plan.json` recomputes legal paths against actor bodies and
  both existing trainer sight squares. Main W↔E courses are 60 steps/four grass;
  exact port-to-port courses 73/four; cache approach six/four. It includes bridge
  rail denial, sign ownership, both town seam returns and Save/Continue cases.
  These are planned inputs, not execution. Bind the actual candidate identity
  after publication; never relabel an existing saved game.

All render epochs, failed camera attempts and rejected proposals are retained.

## Rust handoff and verification

Only the unregistered leaf files `park_pond_route.rs`,
`park_pond_route_tests.rs`, and `bin/build_park_pond_route.rs` were added.
The API is `build_park_pond_route_pack(&CompiledGamePack, &str)`.
The parent agent owns registration, Cargo, fresh publication and readback.
Scoped Rust formatting passed; compilation and the three external-parent tests
have not been run by this worker.

After registration, the intended commands from the repository root are:

```sh
cargo build --locked -p crystal-assets --bin build_park_pond_route
CRYSTAL_PARK_POND_TEST_PACK="$PWD/content-packs/minnesota-river-long-v02.crystalpack" \
  cargo test --locked -p crystal-assets --lib park_pond_ -- --ignored
./target/debug/build_park_pond_route \
  content-packs/minnesota-river-long-v02.crystalpack \
  planning/minnesota/revisions/r04-route-quality-v01/source-v05/native.contract.json \
  content-packs/minnesota-park-pond-route-v05.crystalpack
```

These build/test/publication commands remain **pending**, and the output path
must be fresh. The publisher checks the actual input file hash, derived parent
identity, complete module/art/sign preimages, all 57 cells and mirrors, exact
old-glyph row recipes, full normalized pack residual, candidate identity and
cached/uncached module equality. The CLI writes and verifies an exact readback.
Tests cover geometry/paid grass, stale inputs and 15 out-of-scope corruptions
including cap pixels, rail collision, Cut, actors, programs, audio and mirrors.
The owner must also run required formatting/Clippy, native headless captures,
production input courses and written-pack preservation checks before acceptance.
