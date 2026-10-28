# Mankato: sand terraces and one-way drops

The former wooden floor made low banks hard to distinguish from ordinary rock
walls. This revision uses light sand for raised terraces, darker soil for the
middle arena and wood at the entrance. Native earth borders replace the stray
wooden strips. The unused northwest floor is removed from the room footprint.

![Verified course, illustrated with room states](../renders/gyms/mankato-ledge-review/sequence/mankato-course.gif)

1. Beat Caldren. The wooden gate opens below the already visible left stair.
2. Climb left and hop down the low edge to Jorel's arena. It is a real native
   two-tile jump; the bank blocks climbing back. A lower drop permits retreat
   before the second battle.
3. Beat Jorel. The upper wooden gate opens below the visible stair to Torven.
   After the leader, two short drops return to the entrance.

There are no switches, quiz menus or required field moves. Both practice wins
remain necessary. Fixed rocks separate ascent, landing and return paths; they
do not change into stairs after a battle. The stair tiles are visible from the
start, behind wooden gates which open to ordinary floor.

| Feature | Coordinate / native behavior |
| --- | --- |
| Extent / arrival | 8 × 10 metatiles; walking tile `(6,17)` |
| Caldren / lower gate / stair | `(4,17)` / metatile `(1,7)` / metatile `(1,6)` |
| Required west drop | `(6,8)` → `(6,10)`; ordinary return 22 steps |
| Jorel / upper gate / stair | `(11,11)` / metatile `(6,3)` / metatile `(6,2)` |
| Torven | `(11,3)`, original Wooper 14/Sandshrew 15/Phanpy 16 and native moves |
| Court return drop | `(10,6)` → `(10,8)`; ordinary stair return 6 steps |
| Lower escape | `(10,14)` → `(10,16)`; ordinary return 28 steps |
| Carpets1/2 | `(6,19)` / `(7,19)` → existing city return 6 `(10,12)` |

The shorter west return is deliberate: removing one unnecessary boulder
reduces it from 24 to 22 steps. The ascent divider and required first drop remain
enforced. The native ledge checks verify actual hops, blocked reverse inputs,
safe landings and the necessary ordinary return passages.

## Executed Rust publication and proof

Ignored pack `content-packs/minnesota-mankato-terraces.crystalpack`: 541 maps,
content identity
`04992dc854f2219bf330948aecbd8ba7b96a0bbee211c5faa4dd2043dd27982b`.
It extends the Rochester gate revision, retaining that complete first gym.
Other story/city successors still need these revisions applied to their
lineage. Installation grants no progress; old in-room saves need review.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin revise_mankato_terraces -- \
  content-packs/minnesota-rochester-gates.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/gyms/mankato-terraces.json \
  content-packs/minnesota-mankato-terraces.crystalpack

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-mankato-terraces.crystalpack" \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  mankato_terraces_literal -- --ignored --nocapture --test-threads=1

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets \
  --test mankato_terrace_revision --test rochester_gate_revision \
  -- --ignored --nocapture --test-threads=1

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-mankato-terraces.crystalpack --audio --desktop
```

The literal control run passes on the current build. Initial fixture: Bayleef 19,
Pidgey 17, the prior Care Badge, entrance placement and deterministic test RNG.
Both partners use native level-up moves. No downstream victory, position,
badge, PP or RNG write occurs during play. The course earns both trainers and
the leader, spends native PP, verifies all three jumps and reverse denial,
retreats before Jorel, checks Continue, earns Prairie and one Mud-Slap TM and
uses both real city carpets. Walking uses fixed instructions without a path
finder steering the session.

Three earlier controller cases also pass: complete trainer/drop persistence,
loss refusing ascent and missing first badge refusing the leader. Rochester's
literal gate course passes on the cumulative pack. The room-rebuild code is
shared by the two revisions; Rochester's asset tests still reconstruct its
published identity exactly after that extraction.

Mankato's two asset cases preserve every other map, all trainer parties and
numbers, existing constants, metadata, actors, indexed warps, other art and
audio; uncached split-module reconstruction matches. Negatives reject a fake
ledge, ascent bypass, absent/painted stair, skipped second trainer, changed
party and repeated installation. All 377 PCM hash/frame/loop checks pass.

All current gym terrain pixels/collision quadrants match the selected source.
Native visual fixtures at `(6,8)` and `(6,10)`, noon, facing down, with Caldren's
win set were inspected. Independent source cameras match all 368640 pixels in
each capture. The animation is a room-state illustration of the verified
route, rather than a recording of the game. All originals, transformations,
rejected drafts and native captures remain archived.

The current index selects this source. World/interior reservations explicitly
point to it while retaining the historical town's unchanged door contract.
Plan validation checks every selected revision's extent, arrival, carpets and
registry binding. Six other gyms still await this design/solution review;
user visual acceptance and an earned full regional campaign remain open.

Scoped changed-file rustfmt passes. Full dependency Clippy for both affected
crates still stops at the pre-existing always-false bit mask in
`crates/crystal-core/src/multiplayer/mod.rs:1332`. Scoped all-target Clippy passes with existing warnings, using `--no-deps`
and the existing `never_loop` allowance; no warning is in the changed gym
modules. Workspace formatting still
first fails at the unrelated `crates/offset-allocator/src/lib.rs:408`.
