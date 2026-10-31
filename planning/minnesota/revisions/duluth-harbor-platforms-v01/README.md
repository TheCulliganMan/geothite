# Duluth open harbor and floating fishing decks

The user requested removal of the ledge seawall and isolated inner buoy, keeping
the outer buoys and adding two floating platforms with fishermen. This revision
starts from the selected North Shore shops child of the outdoor-composition
pack, retaining both new cafes and the newer Duluth garden, plateaus, stairs,
rooms, actors and regional connections.

`plan.json` identifies the immutable parent and sources. `after.layout.json`
freezes the already published native shoreline before applying this edit;
explicit source edges prevent automatic shore fitting from restoring the wall.
The seven raised water-edge glyphs become ordinary animated water. Their native
collision was already WATER. The isolated `3a` buoy at metatile `(47,33)` becomes
water. All four map perimeter rows/columns remain exact, including outer buoys.

The west platform occupies metatiles `(26,36)`–`(28,38)` and the east platform
`(34,39)`–`(36,41)`. Both use existing native railed dock art with an open western
boarding face. Fishermen stand at gameplay `(55,75)` and `(71,81)` and leave
approaches `(54,75)` and `(70,81)` clear. They use ordinary native paged dialogue.
Players can land and launch Surf through `(52,75)` and `(68,81)`. These are
optional water excursions; the main city route and story do not depend on them.
No Pokemon, trainers, wild encounters, rewards, audio or source art are added.

## Rust publication

The verified publisher build links the current Rust CLI against freshly
completed, pinned Cargo library artifacts. Its exact command and byte-identical
replay are retained in the ignored final verification record. For a new Cargo
source build after other memory-intensive compilers finish, use:

```sh
CARGO_TARGET_DIR=target/northstar-story CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 \
  cargo build --locked -p crystal-assets --bin revise_duluth_harbor
target/northstar-story/debug/revise_duluth_harbor \
  content-packs/minnesota-north-shore-shops-v01.crystalpack \
  66d03adbd5e245af480890ad985f0bfbd3093c835e0bb6d02b26e563629ecb27 \
  planning/minnesota/revisions/duluth-garden-terrace-v02/after.layout.json \
  planning/minnesota/revisions/duluth-harbor-platforms-v01/after.layout.json \
  content-packs/minnesota-current-duluth-harbor-v02.crystalpack
```

The builder rejects stale complete terrain, occupied platform water, unrelated
changes, modified art recipes and duplicate publication. It verifies the written
pack and retains the parent. Packs, saves, images, logs and verification output
stay ignored. No web deployment is implied.

## Verification

The external-pack asset regression checks other complete maps, native art,
audio, roster and trainer preservation, duplicate publication and unrelated
terrain rejection. The native controller regression starts from an explicit
west-deck/Surf-partner/badge/Repel fixture. All later boarding, fisherman dialogue,
dialogue input ownership, Surf, east-deck arrival, Continue, outer buoy denial,
return and Start-menu checks use `VisibleShellController::press`.

```sh
CRYSTAL_DULUTH_HARBOR_PARENT="$PWD/content-packs/minnesota-north-shore-shops-v01.crystalpack" \
  cargo test --locked -p crystal-assets --test duluth_harbor -- --ignored
CRYSTAL_DULUTH_CITY_CONTROLLER_PACK="$PWD/content-packs/minnesota-current-duluth-harbor-v02.crystalpack" \
CRYSTAL_DULUTH_HARBOR_SAVES="$PWD/planning/minnesota/renders/duluth-harbor-platforms-v01/controller" \
  cargo test --locked -p crystal-bevy --test duluth_harbor -- --ignored --nocapture
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_duluth_harbor.py \
  --parent content-packs/minnesota-north-shore-shops-v01.crystalpack \
  --pack content-packs/minnesota-current-duluth-harbor-v02.crystalpack \
  --output planning/minnesota/renders/duluth-harbor-platforms-v01/published-v01
target/northstar-story/debug/verify_crystalpack \
  content-packs/minnesota-current-duluth-harbor-v02.crystalpack --audio --desktop
```

## Verified local result

The v02 pack is SHA256
`2450f44c16fd9f783b8b1dae8e6fa292fbf6a1444a730527c7ebfc63af09b721`,
content identity
`10e6681625aa566d083fefe49c8f94d0b82a44a456971a2c1427cead98793843`.
At publication, `world.json` selected `duluth_harbor_platforms_revision`; resolve
its active revision for successors. All previous revisions remain available.
The earlier v01 harbor pack is superseded by this composition
with both North Shore cafes preserved.

Fresh final-pack asset preservation/rejection and production controller tests
pass. The production stdio round trip and separate Continue use 88 calls with
empty stderr. Six native GPU captures were inspected, including actual reached
deck saves and matched before/after shore fixtures. Both shore comparisons have
zero changed pixels outside the exact changed metatiles. All731 other complete
maps, native art, roster, encounters, trainers and377 audio programs stay exact.
All PCM hashes, frame counts and loop ranges pass. A freshly linked Rust
publisher reproduces the final pack byte for byte.

Both crates' all-targets Clippy checks pass with unrelated warnings; the one new
test borrowing warning was fixed and rechecked. Changed Rust formatting passes.
Whole-workspace formatting still reports unrelated differences. Retained results
are under `renders/duluth-harbor-platforms-v01`; packs and evidence stay ignored.
This is explicit local QA, not earned statewide progression or a web deployment.
