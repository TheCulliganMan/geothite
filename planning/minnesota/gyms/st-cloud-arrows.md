# St. Cloud: read the arrows before riding the wind

The [regional rebuild](current-regional.md) now includes this same course in
the 554-map story and 660-map city branches. Its integrated ordinary-party
replay passes with four initially carried Hyper Potions and native Pack care
after Wyrel/Hallen; the medicine-free trial lost. Actual HP effects and item
consumption are checked. The puzzle, opponents, levels and fixed wind inputs
remain unchanged. The earlier publication evidence below describes its original
541-map branch.

The former eastbound and westbound lanes had identical wooden slats. Their
opposing behavior was explained only in dialogue. Every walking tile now has a
clear directional chevron. The original native right-arrow font glyph is painted
onto the original wood pixels and reflected for westbound lanes. The wood's
native palette remains intact. No new image asset or font copy is committed.

![Illustrated route through the four wind lanes](../renders/gyms/st-cloud-arrows/sequence/st-cloud-wind.gif)

1. Beat Wyrel to open the lower eastbound lane.
2. Follow its east arrows to Hallen on the opposite bank.
3. Beat Hallen to open the upper westbound lane.
4. Follow those west arrows and climb the blue walkway to Saven.
5. Return on the top eastbound lane and the bottom westbound lane.

The wind carries the player automatically to ordinary stone. Counter-steering
does not override it. Entering a lane from the wrong end returns the player to
the same bank. The other trainer/leader cannot be reached by using a return
lane early. Safe landings and both trainers remain necessary. There are no book
props, quiz menus, field moves or additional switches.

| Feature |16px walking coordinates, except gates |
| --- | --- |
| Extent / entrance |9×11 metatiles / `(4,19)` |
| Wyrel / first gate | `(4,15)` / metatile `(3,6)` |
| Outbound east | `(7,13)` → `(12,13)` |
| Hallen / second gate | `(14,10)` / metatile `(6,4)` |
| Outbound west | `(12,9)` → `(7,9)` |
| Saven | `(3,3)` |
| Upper return east | `(7,5)` → `(12,5)` |
| Lower return west | `(12,17)` → `(7,17)` |
| Carpets | `(4,21)`, `(5,21)` → city return `(8,28)` |

The course dimensions, terrain block indices, every collision quadrant, actors,
gates/requirements, parties/moves/levels, trainer numbers, prior badge4, Wind
Badge/Swift TM and indexed city bindings are preserved. Only arrow graphics and
shorter directional hints change. Native source font/slat bytes are read from
the supplied local pack and lowered by Rust into its private gym tileset.

## Executed Rust publication and proof

Ignored `content-packs/minnesota-st-cloud-arrows.crystalpack`:541 maps, identity
`1aa55a37a6d67cb907ff438422e90ec6d6fb0a9f658a85b3101c8eb6a9a73189`.
It extends the cumulative St. Paul loop parent and retains all first four
revisions. Later city/story successors still need integration. The exterior
coordinates above belong to this exact parent, not newer city recompositions.
Installation grants no progress; existing saves need review before migration.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin revise_st_cloud_arrows -- \
  content-packs/minnesota-st-paul-loop.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/gyms/st-cloud-arrows.json \
  content-packs/minnesota-st-cloud-arrows.crystalpack

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-st-cloud-arrows.crystalpack" \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  st_cloud_arrows_literal -- --ignored --nocapture --test-threads=1

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets \
  --test st_cloud_arrow_revision --test st_paul_loop_revision \
  --test minneapolis_circuit_revision --test mankato_terrace_revision \
  --test rochester_gate_revision -- --ignored --nocapture --test-threads=1

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-st-cloud-arrows.crystalpack --audio --desktop
```

The fixed control course passes in10.88s with initial Ampharos31/Graveler30,
native level-up moves, four prior badges, entrance placement and deterministic
RNG. No later party, position, badge, victory, PP or RNG writes are supplied.
The player earns both practice wins and Saven, spends native PP, exercises all
four lanes, verifies wrong-end entries and counter-steering, Continue on both
banks, one badge/TM, repeat protection and both actual city carpets. There is
no pathfinder steering this run.

An earlier attempt lost because the battle helper could choose Magnitude
against Flying opponents after a party change. It now selects available Rock
Throw in that case. The same partners, levels and opponents then pass. This
changes the test's legal menu selections, not the game or the supplied battle
results. ThunderPunch/ThunderShock fallback choices support native Electric
partners; existing strong-fixture moves retain priority.

The existing first-trainer-loss and missing-fourth-badge cases also pass on the
written pack, run through the freshly compiled test executable. Loss uses an
initial Magikarp5/1HP fixture; the permission negative uses the Mew100 harness
and an initial missing badge4. These local cases do not establish an earned
regional party, campaign or full capture/level balance.

All ten asset cases across the five revisions pass. The new cases reconstruct
the written identity and uncached module; preserve other maps, all collisions,
actors, trainer rosters/numbers, constants/metadata, warps, unrelated runtime
files/art and audio; and reject absent markings, reversed wind, changed gate,
changed geometry, changed party and repeated installation. Prior four published
identities reconstruct exactly with the optional marking field absent.

All377 audio PCM hash/frame/loop checks pass. Published/source terrain pixels
and collision quadrants match. Inspected native captures at `(7,14)` with
Wyrel's win seeded and `(14,9)` with both wins seeded, noon/facing down, match
all368640 source pixels each. These use ordinary floor spawn points adjacent
to the opened crossings. The animation illustrates source room states, not a
recording of gameplay. Every native capture, original, conversion, atlas and
rejected arrow experiment is retained.

Three gyms still need this design/solution pass. User visual acceptance, latest
regional integration and a complete earned campaign remain open.

The current index and world/interior reservations select this revision. All
eight selected terrains/collision projections match the cumulative pack; plan
validation passes20 routes,19 hubs,57 chunks,88 contracts and288 planned maps.
Scoped changed-file rustfmt passes. Workspace formatting still first differs at
`crates/offset-allocator/src/lib.rs:408`. Full dependency Clippy stops at the
pre-existing always-false bit mask in `crates/crystal-core/src/multiplayer/mod.rs:1332`.
Scoped all-target Clippy passes for both affected crates with existing warnings,
using `--no-deps -- -A clippy::never_loop`; no warning points to the changed gym
modules. Executed commands:

```sh
cargo fmt --all -- --check
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  clippy --locked --profile wasm-dev -p crystal-assets -p crystal-bevy --all-targets
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  clippy --locked --profile wasm-dev -p crystal-assets -p crystal-bevy \
  --all-targets --no-deps -- -A clippy::never_loop
```
