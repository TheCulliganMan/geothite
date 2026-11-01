# Duluth: ice stops stay silver, moving gates are gold

The 7 × 10 native ice room keeps two short courses with rough patches for turns.
Four trainer-operated gates now use native gold panels; permanent silver stops
remain silver. The original ice gym's graphics and collision define the room.
The gold gates visibly disappear after victories, so their purpose differs
from the fixed obstacles. No new controls, quiz, books or field moves are added.

![Illustrated ice course](../renders/gyms/duluth-gold-gates/sequence/duluth-gold-gates.gif)

1. From the west launch `(2,16)`, slide UP to `(2,8)`, RIGHT to the rough patch
   `(4,8)`, then UP onto the dry landing. Reach Livra `(3,4)` and win.
2. Her win opens the west return and east launch. Return down the west lane;
   at `(11,16)`, slide UP to `(11,10)`, LEFT to the patch `(9,10)`, then UP to
   `(9,7)` and Tordel `(9,6)`.
3. Tordel's win opens the gold cross gate. Walk LEFT across to Anver `(5,6)`.
   He refuses an early challenge. His win opens the last gold gate to Norek.
4. Win Harbor, then return down the west lane to the entrance.

The east course also works backward before fighting Tordel: DOWN, RIGHT, DOWN.
Rough patches stop motion; ordinary silver blocks stop the player at the
adjacent ice tile. Opposing input cannot steer a live slide. Trainer wins change
real collision: west return becomes floor, east launch becomes ice, then the
cross passage and upper court become floor. All three wins and badge 7 remain
required for Norek. The original parties, levels, moves, actors, trainer numbers,
gate events, reward and dialogue outside the return hints stay intact.

| Feature |16px walking coordinates, except gates |
| --- | --- |
| Arrival / carpets | `(6,17)` / `(6,19)`, `(7,19)` |
| West rough patch / eastern patch | `(4,8)` / `(9,10)` |
| Livra / Tordel / Anver | `(3,4)` / `(9,6)` / `(5,6)` |
| Gold west-return / east-launch gates | Metatiles `(2,7)` / `(5,7)` |
| Gold cross / court gates | Metatiles `(3,3)` / `(3,2)` |
| Norek / actual city return | `(8,3)` / `MnDuluth (16,12)`, warp 4 |

## Executed publication and verification

Ignored `content-packs/minnesota-duluth-gold-gates.crystalpack`: 541 maps, identity
`65984d942a6fcf9812ce3d56035a25a6b88c8d100b9ff5233bacf81f6d27154f`.
It extends the Bemidji garden-return parent and preserves all seven earlier
revisions. Later regional city/story lineages still need integration; the
current registry's expanded Duluth facade is separate from this pinned pack's
actual `(16,12)` return. Installation grants no progress. Existing saves require
migration review before switching packs.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin revise_duluth_gates -- \
  content-packs/minnesota-bemidji-return.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/gyms/duluth-gold-gates.json \
  content-packs/minnesota-duluth-gold-gates.crystalpack

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-duluth-gold-gates.crystalpack" \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  duluth_gates_literal -- --ignored --nocapture --test-threads=1

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets \
  --test duluth_gate_revision --test bemidji_return_revision \
  --test brainerd_winch_revision --test st_cloud_arrow_revision \
  --test st_paul_loop_revision --test minneapolis_circuit_revision \
  --test mankato_terrace_revision --test rochester_gate_revision \
  -- --ignored --nocapture --test-threads=1

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-duluth-gold-gates.crystalpack --audio --desktop

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-duluth-gold-gates.crystalpack --verify-published \
  --output planning/minnesota/renders/gyms/duluth-gold-gates/all-current
```

All four existing `minnesota_duluth_controller_` cases pass on this final pack:
live slides/empty-frame movement/stops/Continue, both complete courses and
three required trainers/rewards/carpets, loss and missing seventh badge. Their
strong-party fixture proves behavior rather than ordinary-team balance.
The final clean fixed ordinary-team rerun passes in 87.64 seconds. Initial fixtures are four
level 44 partners—Ampharos, Typhlosion, Feraligatr and Electabuzz—with native
non-HP DVs of 10, zero effort values, seven prior badges, entrance placement,
six Hyper Potions and the shared deterministic divider. Feraligatr starts with
previously taught legal Surf; the other moves are native level-up defaults.
These fixtures do not prove earning the party, move, items, badges or travel.

Every subsequent input uses the production controller. Walking uses fixed
cardinal instructions; slide tests check live intermediate ice, opposing input,
empty-frame movement and exact resting endpoints. The run tries the closed east
launch, completes the west course, earns Livra, tests Anver's early refusal,
completes and retreats from the east course before Tordel, then earns Tordel,
Anver and Norek. Continue on the rough patch and after trainer/leader wins,
one badge/TM, no repeat battle/reward and both real city carpets pass.

Matchup switches use the production PKMN menu: Electabuzz against Water types,
Feraligatr against Swinub/Piloswine, Typhlosion against Sneasel, with Ampharos
in reserve. Native Pack healing after Livra and Anver checks the actual HP
effect, unchanged PP and item consumption for each use. No party, HP, PP,
position, badge, victory or RNG is rewritten once play begins. Smaller
three-partner drafts lost to Norek, including a Grass-partner draft; the final
run proves this specified four-partner course, not universal balance or a
medicine-free challenge. The original opponents remain unchanged.

All sixteen asset cases across eight revisions pass. The new preservation case
reconstructs the written identity and uncached native map; every other map,
collision quadrant, actor, trainer party/number, constant, metadata, field-move
rule, door index, unrelated tileset/runtime file and audio remains exact.
Negative cases reject indistinguishable silver gates, changed fixed stops or
rough patch, skipped trainers, altered party and duplicate installation.

All 377 audio PCM hashes, frame counts and loop ranges pass. Five inspected
native captures match all 368,640 source pixels each: locked/cross-open/court-open
views at `(5,7)`, and locked/open east-launch views at `(11,16)`. Captures face
down at noon; gate-state flags are explicit visual fixtures. The GIF illustrates
seven source states rather than recording play. Whole-room and native views,
originals, conversions and rejected passes remain archived. All eight current
source terrains and collision projections match publication. The source index
and world/interior contracts now select all eight revisions; plan validation
passes 20 routes/19 hubs/57 chunks/88 contracts/288 maps.

Workspace formatting first differs at pre-existing
`crates/offset-allocator/src/lib.rs:408`. Full dependency Clippy stops at the
pre-existing always-false mask in `crates/crystal-core/src/multiplayer/mod.rs:1332`.
Scoped changed-file rustfmt and all-target Clippy for both affected crates pass
with existing warnings using `--no-deps -- -A clippy::never_loop`; no warning
points to the new gate publisher, asset checks or literal controller course.
User visual acceptance, current regional integration and a complete earned
campaign remain open.
