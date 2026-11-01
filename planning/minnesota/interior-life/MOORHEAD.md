# Moorhead: levee, care and garden neighbors

The completed Rust publication is
`content-packs/minnesota-moorhead-v01.crystalpack`: 708 maps, 475 species,
270 moves, 377 original audio programs and 208 installed Minnesota non-gym
interiors. It adds Moorhead and two rooms over the verified
[Detroit Lakes parent](DETROIT-LAKES.md). The all-interiors objective remains
active; [coverage](COVERAGE.md) retains the missing rooms and full transit work.

The runtime content hash is
`dfcd85f9632e00e3270f5035f91ec80ccc7d7a2204247141d7f5aec219eb9db3`.
The whole-file SHA-256 is
`fa3eacce7f3c9c612ca5a6f811e44dbf68b3b6e249b8925d188893e79f1d4e14`.
These are distinct identities. Use a fresh save for this pack.

## Places and people

| Place | Purpose and production behavior |
| --- | --- |
| Pokémon Center | Nurse, levee volunteer and a student who takes hotdish home from the seed swap. Free HP/PP/status care, native PC and registered poison/blackout return at `(3,6)`. |
| Mart | Clerk and gardener buying a shared first-aid box. Native priced Great Ball, Super Potion, Antidote, Ice Heal, Escape Rope and Repel stock. |
| Levee and pickup court | Five neighbors discuss the river gauge, spare mittens, meadowlarks, campus starts, late frost and helping during floods. A real readable river placard supplies the notebook errand's first step. |

The town contains five residents, the clinic three and the shop two. Counter
interactions use the actual native counter tiles. Ordinary approaches, the PC,
both return carpets and all three outdoor signs remain reachable.

Read the river placard, then speak to the clinic volunteer. His father's
notebook records neighbors' names and help, rather than a scorecard. He gives
one Full Heal. `EVENT_MN_MOORHEAD_GAUGE_READ` records the real sign read;
`EVENT_MN_MOORHEAD_DRY_KIT_GIVEN` is set only after successful inventory mutation.
A full pocket retains the gift; native Pack/Toss makes space for a retry.
Both flags survive Save/Continue, and repeat conversation cannot award again.
The errand grants no chapter, badge or railway progression.

## Native geometry

The [town dossier](../towns/MnMoorhead.json) and
[explicit layout](../towns/MnMoorhead.layout.json) author a compact 16×12 town
using original `johto_modern` art. The uninstalled older reservation is retained
under `revisions/moorhead-source-v1/`.

The clinic uses the complete native `18 19 / 1a 1b` frontage, with its real door
at `(17,7)` and passive return at `(17,8)`. The Mart uses
`18 19 / 1a 33`, door `(25,11)`, return `(25,12)`. Exterior active/passive indices
are 1/2 and 3/4. Both rooms have two return carpets and passive arrival index 3.
The compact clinic has no unpaired communication-floor stair.

Town `(31,12/13)` connects east to R16 `(0,24/25)` with town offset -6 and
reciprocal route offset +6. Original R16 terrain, residents, trainers and its
existing eastern connection remain exact. The only old-map change is the new
western connection and corresponding flags.

The visible Red River runs behind a continuous native bank and levee fence.
Its water continues beyond the town, and this street has no Surf launch.
Both ordinary walking and a legal Surf party are denied across all twenty
public fence tiles. The community garden and passing path stay accessible.

## Executed publication and checks

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
export RUST_MIN_STACK=67108864
cargo build --locked --profile wasm-dev -p crystal-assets --bin pack_moorhead \
  --config 'profile.wasm-dev.opt-level=1'
target/interior-life/wasm-dev/pack_moorhead \
  content-packs/minnesota-detroit-lakes-v01.crystalpack \
  planning/minnesota/towns/MnMoorhead.json \
  planning/minnesota/towns/MnMoorhead.layout.json \
  content-packs/minnesota-moorhead-v01.crystalpack
```

The final path already exists: the publisher refuses to overwrite completed
outputs. Choose a new name when reproducing publication. Strict write/read
verification passed before acceptance.

```sh
export CRYSTAL_MOORHEAD_BASE="$PWD/content-packs/minnesota-detroit-lakes-v01.crystalpack"
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-moorhead-v01.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test moorhead --config 'profile.wasm-dev.opt-level=1' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-bevy --lib moorhead_controller \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored --test-threads=1
```

The asset case passes (5.76 seconds), including deterministic identity, exact
old-map/program/catalog/art/audio preservation and rejected unsafe sources.
Four production controller cases pass (24.99 seconds): every resident and room
return, both R16 lanes, the complete Surf barrier, free care/PC/recovery, priced
stock, marker-before-gift sequencing, full-pocket retry, Save/Continue and
repeat-gift denial. Explicit party, position and inventory fixtures are setup;
every tested interaction uses `VisibleShellController::press`.

All 377 audio programs pass PCM hash, frame-count and loop-range verification.
The source review passes eight adversarial geometry/story cases; its final warm
review took 691.2 ms after loading. All thirteen final native views pass complete
640×576 pixel comparisons: seven room cameras and six street/levee cameras.
All were personally inspected. Untouched GPU originals, descriptors, source
references and differences remain ignored under
`renders/interior-life/moorhead-v01-native/` and `renders/moorhead/native-v02/`.

The first garden camera accidentally occupied the gardener's tile. Its
684-pixel rejection remains in `renders/moorhead/native-v01/`; the corrected
empty approach `(20,18)` passes. Both source and capture tools now reject
occupied viewpoints. A Johto/JohtoModern Mart-block mix-up was corrected during
source review, with the rejected renders retained.

Changed Rust leaves pass `rustfmt --check`. All-target Clippy for assets/Bevy
passes with only the existing `bad_bit_mask` and `never_loop` denials allowed;
it reports pre-existing warnings and none in the Moorhead leaves. Whole-workspace
`cargo fmt --all -- --check` still fails on existing formatting beginning in
`offset-allocator`. These are not clean-workspace claims.

## Bedroom camera follow-up

The older Winona bedroom comparison assumed 3× source pixels. The review helper
now independently derives native room-fit scale and origin from map dimensions
and the renderer's reserved field area. The 4× bedroom capture on this 708-map
pack passes all 368,640 pixels, with the cousin, stairs, furniture and complete
frame personally inspected. Its evidence is retained in
`renders/interior-life/bedroom-camera-v01-native/`.

Seven retained Moorhead room frames also pass with the corrected helper at 3×
in `reference-recomparison-v02.json`. This recomparison uses the original GPU
frames and their recorded executable identity. The shared executable had been
rebuilt by another task, so the regular resume correctly rejected its changed
hash; no changed executable was attributed to those original frames. Earlier
rejected bedroom references remain retained. A separate city-context correction
now includes the native compositor's second-hop Mississippi scenery: both fresh
St. Paul angler-house frames pass all pixels and were personally inspected.
See the [city runbook](../towns/st-paul-repair/README.md).
