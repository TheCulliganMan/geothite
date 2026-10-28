# St. Paul: cross each room to the far pad

The previous four-room graph made the player return through the foyer between
Orven and the leader. This revision makes a continuous loop:

**Entrance → Belva → Orven → leader landing → entrance.**

There are two pads in each chamber. In the ordinary rooms they occupy opposing
corners; on the leader landing they sit side by side. Crossing to the other pad
moves onward, while stepping off and back onto the arrival pad retraces the last
link. Every pair is reciprocal. The guide and defeated trainers explain this
rule. There are no books, quiz menus or field-move requirements.

![Illustrated course, with the next destination marked](../renders/gyms/st-paul-diagonal/sequence/st-paul-loop.gif)

1. Take the left entrance pad to Belva and win her battle.
2. Cross to her far pad to reach Orven and win his battle.
3. Cross to Orven's far pad, which now goes straight to the leader landing.
4. Both victories open the visible metal grille. Battle Brannic beyond it.
5. The landing's other pad returns directly to the entrance.

The loop also permits reverse inspection order. Trying the leader landing early
still encounters a solid grille and a safe way back. Each individual trainer
win leaves it closed; both are necessary. The main forward course uses three
pad trips rather than four before the leader, then one for the return.

| Pair / original warp records | Endpoints in16px walking tiles |
| --- | --- |
| WEST_ROOM4/5 | Entrance `(1,11)` ↔ Belva `(1,3)` |
| CROSS_ROOM6/7 | Belva `(5,7)` ↔ Orven `(9,11)` |
| EAST_RETURN8/9 | Orven `(11,17)` ↔ leader landing `(9,7)` |
| MAIN_LIFT10/11 | Entrance `(5,15)` ↔ leader landing `(11,7)` |
| Belva / Orven / Brannic | `(3,3)` / `(11,12)` / `(11,2)` |
| Grille | Metatile `(5,2)`, native Elite Four room14 metal plates |
| Passive entrance / carpets | `(2,17)` / `(2,19)` and `(3,19)` |
| City return | Original return4 `(54,16)` in this parent lineage |

The7×10 extent, all actor coordinates, original parties/levels/moves, trainer
numbers, prior-badge3 permission, Capitol Badge/Steel Wing TM and indexed city
doors stay intact. The two trainers retain Magnemite21 and Pineco22/Magnemite22;
Brannic retains Magnemite23/Pineco24/Skarmory25. No new character names are used.

## Executed publication

Ignored `content-packs/minnesota-st-paul-loop.crystalpack`:541 maps, identity
`3983e786b3509fc0e2beddf5c4c03ac6a9809eabe93c57a037bbe90a1bd0a9eb`.
The exact parent is the cumulative Minneapolis-circuit pack, retaining the
Rochester lane course and Mankato terraces. Later city/story successors need
separate integration. Their larger exterior coordinates do not describe this
older parent. Installation grants no progress; old in-room saves need migration
review because internal pad coordinates change.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin revise_st_paul_loop -- \
  content-packs/minnesota-minneapolis-circuit.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/gyms/st-paul-loop.json \
  content-packs/minnesota-st-paul-loop.crystalpack

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-st-paul-loop.crystalpack" \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  st_paul_loop_ -- --ignored --nocapture --test-threads=1

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets \
  --test st_paul_loop_revision --test minneapolis_circuit_revision \
  --test mankato_terrace_revision --test rochester_gate_revision \
  -- --ignored --nocapture --test-threads=1

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-st-paul-loop.crystalpack --audio --desktop
```

All three controller cases and all eight revision asset cases pass.

The literal course's initial fixtures are Quilava27/Geodude25 using native
level-up moves, three prior badges, entrance placement and deterministic RNG.
Every step and pad ride uses fixed directions through production input, with
no pathfinder steering or later position/party/badge/victory/PP/RNG writes.
It exercises all eight pad endpoints, checks arrival/Continue without bouncing,
earns both trainers and Brannic, checks early grille collision, one badge/TM,
repeat reward protection and both genuine city carpets.

The separate reverse-order/missing-badge test uses the existing Mew100 harness
fixture, earns Orven before Belva and verifies that the opened grille does not
replace badge3 permission. The loss fixture uses Magikarp5 with1HP and verifies
ordinary recovery without opening the court. These are local tests; they do not
claim an earned regional campaign or complete capture/level balance.

Asset cases reconstruct the written identity and uncached native module,
preserve every other map, trainer roster/number, actor, metadata, constant,
indexed city binding, unrelated art/runtime file and compiled audio. Only the
declared internal pad positions change among warp records. Negatives reject a
broken pair, floor impersonating a pad, an unbound extra pad, a skipped trainer,
changed party and repeated installation. Shared rebuilding now accepts the
existing reciprocal pad count; earlier three-warp revisions remain supported.

All377 PCM hash/frame/loop checks pass. Native cameras at `(5,6)` in Belva's
room and `(11,6)` at the open court, noon/facing down, were inspected. The open
court capture explicitly seeds both trainer-win flags. Both reconstructed source
cameras match all368640 pixels. The animation illustrates source room states
and marks the next destination; it is not a gameplay recording. All originals,
conversions, rejected pad placements, atlases and native captures are retained.

Four other gyms still need this design/solution pass. User visual acceptance,
regional integration and a complete earned campaign remain open.

All eight selected source terrains/collision quadrants match the published pack.
Plan validation passes20 routes,19 hubs,57 chunks,88 contracts and288 planned
maps. Its revised-room check accepts St. Paul's existing canonical reference
kind and verifies the matching source ID/native constant, without altering the
frozen city contract or its historical room source. The current index, world
reservation and interior registry select this revised terrain.

Scoped changed-file rustfmt passes. Workspace formatting still first differs in
`crates/offset-allocator/src/lib.rs:408`. Full dependency Clippy stops at the
pre-existing always-false mask in `crates/crystal-core/src/multiplayer/mod.rs:1332`.
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
