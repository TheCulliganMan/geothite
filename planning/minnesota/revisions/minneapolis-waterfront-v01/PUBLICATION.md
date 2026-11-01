# Executed waterfront publication and checks

Run from the repository root with the completed ignored parent present:
`content-packs/minnesota-ely-pine-knolls-v02.crystalpack`, SHA256
`1f5604a6c743904e5b0c8fbfe36b32ef056f50dab309a42795aa639457ac0bf3`,
identity `1d3d63099937c03ad2c390ef2910ddfe23de232321cfdcb226aa26e250df6347`.
It already contains169 joined city metatiles. Do not substitute another branch
or publish the older town blockout alone.

## Source and Rust commands

The following source stages were executed with the local Pillow/cbor2 runtime.
Source-v06 is immutable; choose fresh names for further work.

```sh
export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/revisions/minneapolis-waterfront-v01/author.py \
  --epoch source-v06
python3 planning/minnesota/revisions/minneapolis-waterfront-v01/freeze.py \
  --source planning/minnesota/revisions/minneapolis-waterfront-v01/source-v06 \
  --output planning/minnesota/revisions/minneapolis-waterfront-v01/source-v06/native.contract.json

CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=33554432 \
cargo build --profile wasm-dev --locked -p crystal-assets \
  --bin pack_minneapolis_waterfront \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0

RUST_MIN_STACK=33554432 target/runtime-fps/wasm-dev/pack_minneapolis_waterfront \
  content-packs/minnesota-ely-pine-knolls-v02.crystalpack \
  planning/minnesota/revisions/minneapolis-waterfront-v01/source-v06/native.contract.json \
  c02018e79cf6fd3b09c538afae8934be5b2094abdcaf5fc0c1162b1eceae8695 \
  content-packs/minnesota-waterfront-frontages-v01.crystalpack

python3 planning/minnesota/revisions/minneapolis-waterfront-v01/audit.py \
  --pack content-packs/minnesota-waterfront-frontages-v01.crystalpack \
  --source planning/minnesota/revisions/minneapolis-waterfront-v01/source-v06 \
  --epoch native-v02

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-waterfront-frontages-v01.crystalpack --audio --desktop
```

The CLI rejects existing, tracked or nonignored outputs, an incorrect parent
file, an unreviewed contract hash, changed native donor art and changes outside
the16 reviewed cells. Rust verifies the input, publishes the pack and strictly
reads it back. Python performs read-only source review and comparison.

After rebuilding the CLI with the tracked-output guard, the same command with
fresh output `minnesota-waterfront-frontages-replay-v01.crystalpack` passed.
Its complete file bytes and SHA256 equal the first publication.

The output identity is
`3e28ac6c91696876e4572dac67494fcd3fe8997f31d93ee5463d2b89ac41da52`;
file SHA256 is
`3862c34b044a456721b5f7ee5464b42dd123f07e3bd12d16e7e503bec972ffa2`.
There are691 maps,475 installed species,224 active Minnesota species,270 moves
and377 audio programs. All audio PCM hashes, frame counts and loop ranges pass.

The [native receipt](../../renders/minneapolis-waterfront-v01/native-v02/receipt.json)
proves unchanged city collision, doors/events/scripts/rooms/recovery/species and
690 other maps. The only changed runtime files are the five existing private
city graphics, metatile and palette-map files. All224 original palette entries
and692,224 old four-bank metatile pixels remain exact. No source art catalog,
runtime key, base palette or donor tileset is added or replaced.

## Focused regressions and native images

The external-pack integration tests were executed successfully:

```sh
CRYSTAL_WATERFRONT_PARENT_PACK="$PWD/content-packs/minnesota-ely-pine-knolls-v02.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=33554432 \
cargo test --profile wasm-dev --locked -p crystal-assets --features test-fixtures \
  --test minneapolis_waterfront \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  -- --ignored --test-threads=1
```

Both cases pass, including mutations to collision, occupied glyph allocation
and the reviewed hash. The freshly rebuilt Bevy test executable
`target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652` passed four
waterfall-allocation unit cases, the actual new-pack animation test and the
actual original64×48 city compatibility test. The new-pack test used
`CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK` set to the absolute new pack path.
It checks all waterfall scroll phases and production ticks, unchanged other
animations, and every new static glyph's exact frame0/frame168 pixels.

The fresh `render_at_location` executable was built with `location-tester`.
All four final native captures and four parent captures succeeded, using:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=33554432 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-waterfront-frontages-v01.crystalpack \
  --map MnMinneapolis --x 9 --y 30 --view 2d --hour 12 \
  --screenshot planning/minnesota/renders/minneapolis-waterfront-v01/native-fixtures-v02/boathouse.png
```

The other cameras are(19,30),(23,42),(7,46). Parent captures use the same
arguments and `native-before-v01/`. All were inspected. There is no claim of
final3D acceptance or earned regional progression. Existing gameplay/collision
is preserved; a dump or screenshot is not treated as a new playthrough.

## Whole state and review limits

`audit_outdoor_connections.py --pack NEW_PACK --output
planning/minnesota/renders/minneapolis-waterfront-v01/state-continuity-v01
--scale8` was executed with the actual new pack. Its97 outdoor maps have one
direct connection group, zero inconsistent placement cycles and no footprint
overlaps. The atlas contains actual map terrain at one scale. Named hubs still
absent from this lineage include Detroit Lakes, Moorhead, International Falls
and Mille Lacs. A connection graph does not prove Surf or story prerequisites.

Eight raw water-edge cells in R02 were separately qualified against its actual
compiled module: all152 water cells are sealed by fences and bridge rails;
there are zero land-to-water mounting pairs and zero reachable outer Surf
cells. Three barrier-opening mutations are detected. The12-cell source-branch
difference is recorded, not substituted into this pack. See the
[channel proof](../../renders/minneapolis-waterfront-v01/native-v02/prairie-channel-qualification.json).

All five changed Rust leaf files pass rustfmt. Required whole-workspace
`cargo fmt --all -- --check` fails on existing shared-tree formatting, beginning
in `crates/offset-allocator/src/lib.rs:408`. The required affected-crate
all-target Clippy run was attempted with `test-fixtures` and `location-tester`.
It fails on existing core `bad_bit_mask` at multiplayer/mod.rs:1332 and voxel
`approx_constant` errors in scenery_finish.rs:264, foliage/pine.rs:184 and
mesh/shore_ecology.rs:54. These unrelated files were not changed for this task.

The assets-only `--no-deps` Clippy check also encounters11 existing
`clippy::never_loop` errors in the compiler. Repeating that focused library,
publisher and integration-test check with `-A clippy::never_loop` passes; none
of its diagnostics point to the new waterfront files. This scoped result does
not replace or turn the required all-target failure into a pass.
The same `--no-deps --lib -A clippy::never_loop` check for `crystal-bevy` with
`location-tester` also passes, with no diagnostic in the changed animation file.

The shared planning checker was also run without generating a gallery. It
stops in `check_city_recomposition_plan.py:212`: the residents of
`MnMinneapolisDepartmentStore5F` differ from an earlier44-room snapshot.
No interior was changed by this overlay;
the full native residual-equality audit passes independently. The new world,
town and Markdown source/evidence links resolve. The shared source mismatch
remains visible rather than being fixed by weakening its assertion.

Logs remain under `.local/minneapolis-waterfront-*`, including builds, focused
tests, native/legacy animation, audio, source/native review, paired GPU captures,
state atlas, formatting and Clippy. All renders, rejected versions and ignored
packs remain saved. They are not deliverables to commit.
