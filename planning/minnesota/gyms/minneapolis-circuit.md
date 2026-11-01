# Minneapolis: trainers between the electrical locks

The old room required both battles before either switch and used gray barriers.
The new5×9 room alternates battle and puzzle progress. Red current fills each
closed passage. Only the closed barriers block movement; ordinary walls stay
fixed. Both barriers use the native Facility0b grid with explicit private WALL
collision in all four quadrants. No field move, book or quiz menu is required.

![Illustrated room states](../renders/gyms/minneapolis-circuit/sequence/minneapolis-course.gif)

1. Beat Lenvi, who identifies the left can in the top row.
2. Press that can to release the first lock. Walk through to Jossa.
3. Beat Jossa, who identifies the neighboring can in the same row.
4. Return and press the next can right to release the second lock and reach Avrin.

Before Lenvi, switches refuse. Before Jossa, second-stage controls refuse while
retaining the first switch. A wrong second choice after Jossa restores both
barriers but preserves battle wins. Wrong choices before the first switch do
nothing. Once solved, inspecting any can keeps the locks open. All six cans
remain reachable after a reset. Both exits remain reachable in every stage.

| Feature | Walking coordinates, except where specified |
| --- | --- |
| Extent / entrance | 5×9 metatiles / `(4,15)` |
| Lenvi | `(1,13)`, original Mareep17 |
| First switch | `(3,11)`, upper-left can |
| First barrier | Metatile `(2,4)` |
| Jossa | `(4,6)`, original Magnemite18/Voltorb18 |
| Second switch | `(5,11)`, upper-middle can |
| Second barrier | Metatile `(2,2)` |
| Other cans | `(7,11)`, `(3,13)`, `(5,13)`, `(7,13)` |
| Avrin | `(4,3)`, original Mareep19/Magnemite20/Flaaffy21 |
| Carpets | `(4,17)` and `(5,17)` → city return4 `(42,24)` |

## Executed publication and proof

Ignored `content-packs/minnesota-minneapolis-circuit.crystalpack`:541 maps,
content identity
`d3ffc94b129fee3e3e8d604a07f1c44b9401d64908ea244b4610c9ebeb2c6d31`.
Its parent is the cumulative Rochester-gates/Mankato-terraces pack. It preserves
all other maps, full trainer rosters/numbers, metadata, constants, indexed city
warps, unrelated art and source audio. Only Jossa's object position changes.
Later city/story lineages still need integration; their updated city coordinates
are not asserted to match this older parent. Installation grants no progress;
old in-room saves require migration review.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin revise_minneapolis_circuit -- \
  content-packs/minnesota-mankato-terraces.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/gyms/minneapolis-circuit.json \
  content-packs/minnesota-minneapolis-circuit.crystalpack

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-minneapolis-circuit.crystalpack" \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  minneapolis_circuit_ -- --ignored --nocapture --test-threads=1

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets \
  --test minneapolis_circuit_revision --test mankato_terrace_revision \
  --test rochester_gate_revision -- --ignored --nocapture --test-threads=1

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-minneapolis-circuit.crystalpack --audio --desktop
```

The literal ordinary-party test passed in13.10s. Initial fixtures supply native
Geodude23/Bayleef23 with their level-up moves, two prior badges, entrance
placement and deterministic RNG. No later position, party, battle result, badge,
PP or RNG mutation is supplied. Production inputs earn Lenvi, Jossa and Avrin,
spend PP, verify wrong/disabled controls and actual reset collision, Continue
at several stages, one badge/TM, repeat reward protection and both city exits.
The fixed directions do not use a pathfinder to steer the run.

Both additional tests pass: they isolate a second-trainer loss with the first win and
switch explicitly supplied at setup, and a missing second badge while earning
the entire circuit. Those cases do not claim earned regional travel. Asset
checks reconstruct the published identity and uncached module, preserve all
unrelated content and reject deadlock, a skipped trainer, an invisible barrier,
a missing solid barrier, changed party and duplicate installation. Historical
switch generation remains unchanged for previous pinned publishers.

All377 PCM hash/frame/loop checks pass. Native captures at `(4,10)` with both
locks closed and `(5,7)` with Lenvi/first-switch flags seeded, both noon and
facing down, were inspected. Each independently reconstructed source camera
matches all368640 pixels. Source/published terrain and four-quadrant collisions
also match. The animation illustrates source room states; it is not a gameplay
recording. Every original, conversion, rejected draft and capture is retained.

The current source index and world/interior metadata select this revision.
Five gym designs, user visual acceptance and a complete earned regional campaign
remain open. Workspace formatting has unrelated existing failures beginning at
`crates/offset-allocator/src/lib.rs:408`; changed gym leaves are formatted.

All six asset cases across this revision and its two predecessors pass. All
eight selected gym terrains/collision maps match the cumulative published pack.
Plan validation passes20 routes,19 hubs,57 chunks,88 contracts and288 planned
maps; symbolic story reachability is not a gameplay claim.

The isolated loss fixture uses Magikarp5 with1HP and explicitly supplies the
first trainer win/switch before input. Normal exit/re-entry runs the map callback
before its Continue check. The missing-badge negative uses the existing level100
Mew harness party; the ordinary-party claim belongs to the main literal course.

Scoped all-target Clippy passes for both affected crates with existing warnings,
using `--no-deps -- -A clippy::never_loop`; no warning points to these changed gym
modules. Full dependency Clippy stops at the existing always-false mask test in
`crates/crystal-core/src/multiplayer/mod.rs:1332`. The executed commands were:

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  clippy --locked --profile wasm-dev -p crystal-assets -p crystal-bevy --all-targets
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  clippy --locked --profile wasm-dev -p crystal-assets -p crystal-bevy \
  --all-targets --no-deps -- -A clippy::never_loop
```
