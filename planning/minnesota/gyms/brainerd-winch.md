# Brainerd: one visible dock control, two pier positions

The5×8 pool keeps its native Port water, brick piers, edging and carpet. A
single blue dock control replaces the right entrance statue. Moving the walk
now means pressing that fixture with A, rather than asking Lorla to select a
position through a menu. Her repeat interaction now gives a clue.

![Illustrated pier course](../renders/gyms/brainerd-winch/sequence/brainerd-winch.gif)

1. Beat Lorla on the first pier.
2. Press the blue control. The center walk extends to Bravel's pier.
3. Cross and beat Bravel, then return to the control.
4. Press again. The center walk retracts, the left walk extends, and the upper
   connection opens to Sivren. Return by the left walk after the leader.

Pressing before Lorla refuses. Pressing twice before Bravel retracts the center
walk but cannot open the left route; another press restores the center. The
control remains reachable from safe foyer floor in all six meaningful states.
Both wins remain required for Sivren. There are no quiz or position-choice menus,
books, new field-move requirements or added attendants.

| Feature |16px walking coordinates, except gates |
| --- | --- |
| Extent / passive entrance |5×8 metatiles / `(4,14)` |
| Lorla / blue control | `(2,10)` / `(6,13)`, approached from `(5,13)` |
| Center walk | Gate metatile `(2,4)`; walkable column5 at y8–9 |
| Bravel | `(2,7)` |
| Left walk | Gate metatile `(1,4)`; walkable column3 at y8–9 |
| Upper connection | Gate metatile `(3,2)` |
| Sivren | `(5,3)`, approached from `(5,4)` |
| Carpets | `(4,15)`, `(5,15)` → actual city return `(12,36)` |

Original actors/parties/levels/moves, trainer numbers, gate conditions, pier
modes/events, badge5 permission, Lake Badge/Rain Dance TM, dimensions and indexed
city doors stay intact. The only static collision change is the new full solid
control at metatile `(3,6)`, replacing the narrower statue. All other quadrants
remain exact. The authored Port LADDER tiles have no warp and behave as the
original reference's walkable piers. Walking the course needs no field move.

## Executed Rust publication and proof

Ignored `content-packs/minnesota-brainerd-winch.crystalpack`:541 maps, identity
`9e67016986aef09f2cae8fafd3d5c8e9970a3ca5653f6ab9466c4752f3e1e053`.
It extends the St. Cloud arrow parent and preserves all five earlier revisions.
Later city/story successors need separate integration. Coordinates above describe
this exact parent. Installation grants no progress; existing saves need review
before migration because private symbol indices change.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin revise_brainerd_winch -- \
  content-packs/minnesota-st-cloud-arrows.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/gyms/brainerd-winch.json \
  content-packs/minnesota-brainerd-winch.crystalpack

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-brainerd-winch.crystalpack" \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  brainerd_winch_ -- --ignored --nocapture --test-threads=1

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets \
  --test brainerd_winch_revision --test st_cloud_arrow_revision \
  --test st_paul_loop_revision --test minneapolis_circuit_revision \
  --test mankato_terrace_revision --test rochester_gate_revision \
  -- --ignored --nocapture --test-threads=1

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-brainerd-winch.crystalpack --audio --desktop
```

Both new controller cases pass. The literal course uses initial Sunflora36,
Ampharos35 and Venusaur36 with native level-up moves, five prior badges, entrance
placement and deterministic RNG. It earns Lorla, Bravel and Sivren, checks early
presses, no choice menu, input ownership, both directions of the toggle, actual
floor-to-water retraction, Continue on the center pier and at the leader, one
badge/TM, repeat protection and both actual city carpets. All walking uses fixed
instructions; no pathfinder steers this course.

Matchup-based party changes use the production battle's PKMN menu. They select
Ampharos against Poliwag/Goldeen/Poliwhirl, Sunflora against Wooper/Chinchou, and
Venusaur against Quagsire/Lanturn. The shared battle helper retains its previous
behavior when no partner policy is supplied. No battle engine, opponent, level,
HP, PP, victory, badge, position or RNG is rewritten during play. No healing
items are supplied or used in the final case. Smaller two-partner drafts lost;
this is evidence for the specified team rather than universal balance.

The new missing-badge5 case earns both trainers and pier positions with the
existing Mew100 harness, then verifies Sivren refuses. Existing loss and isolated
poolside-denial cases also pass on the written pack. Loss uses the
initial Magikarp5/1HP fixture. Poolside placement models a later visit and proves
the leader's trainer requirement; it is not proof of actual Surf travel. These
local fixtures do not establish a complete earned regional campaign.

All twelve asset cases across the six revisions pass. The new cases reconstruct
the written identity and uncached module, preserve all other maps, original
trainer rosters/numbers, actors, constants/metadata, warps, unrelated runtime
files/art and audio. Every collision outside the control remains exact. Negatives
reject a menu-based control, restored NPC operator, missing hardware, skipped
trainer, changed party and repeated installation. The publisher checks all six
pier states, reachable controls/exits and the required trainer/leader access.

All377 PCM hash/frame/loop checks pass. All eight selected source terrains and
collision projections match publication. Three inspected native views at
`(5,13)`, noon/facing down, show the initial, center and left states. Center
explicitly seeds Lorla/CENTER; left seeds both wins/LEFT. Each independently
reconstructed source camera matches all368640 pixels.

The source sprite preview previously discarded all white pixels, including an
enclosed Lorla highlight. It now follows the native renderer's border-connected
background mask and preserves enclosed paper in every facing frame. No native
capture or game sprite was edited. The original9-pixel discrepancy and all
originals, conversions, rejected drafts and native captures remain archived.
The animation illustrates room states rather than recording gameplay.

Current source selection and world/interior reservations point to the revision.
The registry now explicitly records the existing passive arrival3. Plan validation
passes20 routes,19 hubs,57 chunks,88 contracts and288 planned maps. Bemidji and
Duluth still need this pass. User visual acceptance, latest regional integration
and a complete earned campaign remain open.

The earlier St. Cloud ordinary-party literal course also passes on this cumulative
pack after the optional partner-selection helper was added, exercising its
unchanged default path. Scoped changed-file rustfmt passes. Workspace formatting
still first differs at `crates/offset-allocator/src/lib.rs:408`. Full dependency
Clippy stops at the pre-existing always-false mask in
`crates/crystal-core/src/multiplayer/mod.rs:1332`. Scoped all-target Clippy passes
for both affected crates with existing warnings using
`--no-deps -- -A clippy::never_loop`; no warning points to the changed gym modules.

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
