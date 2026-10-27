# Current garden/pine revision

This route now matches the471-map `minnesota-canopy-review.crystalpack`. See
[the current sight, preservation and native-input proof](../playable/garden-and-pine-revisions.md).
Earlier source/native milestones below are retained as history.

# Twin Cities River Crossing: native park review

This **28×18-metatile** civic park is one native exterior, `MnR04C00R00`.
It replaces the repeated six-map reserve with a protected river crossing,
rose beds, benches, a battle-or-grass fork and a picnic reward. This route's
short extent is an individual choice; other maps can be larger when their
features and pacing justify the space.

[Whole map](../renders/twin-cities/whole-route-7e4b9a05.png) ·
[eight player-scale frames](../renders/twin-cities/viewpoints.png) ·
[native bridge](../renders/twin-cities/native-bridge.png) ·
[native rose walk](../renders/twin-cities/native-rose-walk.png) ·
[native short lane](../renders/twin-cities/native-cole-lane.png) ·
[native bank lookout](../renders/twin-cities/native-bank.png) ·
[native eastern garden](../renders/twin-cities/native-east-gardens.png) ·
[native picnic spur](../renders/twin-cities/native-picnic.png).

## Physical contract

Coordinates below are global 16px gameplay tiles; this map has no chunk seam.

| Place | Implemented behavior |
| --- | --- |
| Minneapolis arrival | West gate `(2,18)`, landing `(6,18)`. The main diamond-paved lane leads east. Northern and southern garden paths branch before Cole. |
| Short lane | A two-tile gold fence has FLOOR at `(16–17,18)` and WALL at y=19; elms close its northern and southern sides. Cole `(19,18)` faces west with sight range **1**. Entering `(18,18)` triggers his native challenge. He stays adjacent instead of walking into the one-cell pinch. After victory, `(18,19)→(20,19)` passes beneath his body, and the lane works both ways. |
| Rose walk alternative | The northern loop uses x=12/13, crosses habitat x=16–19, y=8–13, and descends by x=23 to avoid the litter bin at `(22,16)`. Avoiding Cole requires at least **four grass steps in either direction**. The short lane is dry after victory. There is no free option that avoids both battle and habitat. |
| Western bench | Super Potion `(7,13)` is below the rose-bed seat. Its approach `(8,13)` stays off the bench and flower border. |
| Quiet nook | Two seats, the gardener `(7,25)`, Ether `(9,27)`, a small tree and enclosing gold fence reward the short southern spur. The fence's lower wall at y=31 stops movement. |
| River bridge | Paved deck core x=24–33, y=18–19; both rows work in both directions. North rail has WALL at y=16/FLOOR y=17; south rail FLOOR y=20/WALL y=21. The inner rail strips are paved rather than grass. Removing the complete span disconnects the arrivals. |
| Retaining banks | Continuous source-derived cyan stone edges, native WALL collision and selected vertical fences keep the player off the river. The bridge keeper `(23,23)` has a narrow western lookout, reached through `(22,23)`. Water remains animated with Park's `WATER_21` behavior. |
| Saint Paul garden | Two framed rose beds, a bench, litter bin and a south fence form the eastern forecourt. East gate `(52,18)`, landing `(48,18)`. The picnic entrance is a deliberate gap at x=40–41 through the south fence. |
| Picnic spur | Mica `(41,25)` can be passed along x=40 or challenged by A. Great Ball `(49,27)` lies across the habitat at x=44–47, y=24–29. Returning from his battle uses the clear row y=26 around his body; the occupied tile is never a through path. |

Two trainers use persistent native flags: Cole has Ditto 17/Yanma 17; Mica
has Abra 16/Snubbull 17. Two residents, three pickups and five signs have exact
approaches. Flags 03–08 are unused trainer reservations; pickup 03 is unused.
No facade promises a nonexistent interior. This flat park has no ledge; its
movement choice is trainer challenge versus habitat.

## Story and connections

The design preserves `BADGE_3` and Q03, **Two cities one watershed**. Q03's
depot courier and fourth-badge gym belong in Saint Paul; the northern checkpoint
belongs to Route 5 and checks both Twin Cities badges. Park residents do not
grant a badge or complete that quest. Real Minneapolis/Saint Paul gate events,
earned Q03 progression and regional recovery remain integration work. The
isolated overlay retains the original source HOME and recovery destinations.

## Source art and fast loop

The exterior uses **27 metatile varieties**. `mn_river_park` retains all 64
original Park metatiles and their unchanged graphics/palette data, appending
four bounded recipes. `40` combines the upper gold rail with native paving;
`41`/`42` repeat a straight middle row from the original fountain corners;
`43` combines paving with the lower gold rail. All rows come from existing
8px tiles. Collision, animation behavior and row recipes are compiled in Rust.

Native review caught isolated cyan caps in the original banks, then a zigzag
from alternating corner rows. The final straight rows remove both defects.
The bridge's original grass strips were replaced with source paving. The
western approach was narrowed around the short lane while retaining the
keeper lookout. Six final native views and the whole map were inspected.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_twin_cities.py --serve
```

Open `http://127.0.0.1:8774/`. Edit [terrain strokes](MN_R04.layout.json) and
[events/dialogue](MN_R04.json); a warm source render/check measured 86–102ms.
No Cargo invocation or pack export is needed for that iteration. Publication
is separate. A changed pack identity requires newly exported review saves.

## Verification and local play

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_twin_cities -- \
  content-packs/text-tui.crystalpack planning/minnesota/routes/MN_R04.layout.json \
  planning/minnesota/routes/MN_R04.json content-packs/minnesota-twin-cities.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_twin_cities.py \
  --pack content-packs/minnesota-twin-cities.crystalpack --verify-published
CRYSTAL_TWIN_CITIES_TEST_PACK="$PWD/content-packs/minnesota-twin-cities.crystalpack" \
CRYSTAL_TWIN_CITIES_FIXTURE_SAVE="$PWD/.local/twin-cities-review.crystalsave" \
CRYSTAL_TWIN_CITIES_JOURNEY_SAVE="$PWD/.local/twin-cities-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --lib --profile wasm-dev --locked \
  -p crystal-bevy --features location-tester twin_cities_ -- --ignored
CRYSTAL_TWIN_CITIES_TEST_PACK="$PWD/content-packs/minnesota-twin-cities.crystalpack" \
CRYSTAL_TWIN_CITIES_JOURNEY_SAVE="$PWD/.local/twin-cities-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p geothite --test twin_cities -- --ignored
target/runtime-fps/wasm-dev/geothite play \
  content-packs/minnesota-twin-cities.crystalpack \
  --load .local/twin-cities-journey.crystalsave --compact
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-twin-cities.crystalpack --audio --desktop
```

The **389-map** pack matches all 504 exterior metatiles, seven objects, five
signs, parties and dialogue. All 68 aliased metatile images match their source
recipes. Nine route asset regressions pass. The park regression composes with
River Valley and preserves every previous/source map, runtime file, tileset,
audio program, HOME and recovery point. It rejects changed badge conditions,
an uncovered lane, removed habitat, a dry bypass and a removed bridge. All
377 audio PCM hashes, frame counts and loop ranges pass verification.

The native empty-party geometry fixture passes both grass bypass directions,
both bridge rows both ways, rails/bank fences, all residents/signs/rewards,
dialogue input ownership and Start. That is explicit geometry setup.

The populated journey uses source-created Croconaw 21/Flaaffy 20, three badge
bits and a deterministic divider as explicit setup. It escapes one native wild
encounter, takes the grass alternative both ways while Cole remains unfought,
defeats both trainers/four opposing species through visible battle menus with
PP spent, and traverses the short lane both ways after Cole's victory. It
collects all three rewards and uses Ether through production Start → Pack →
USE → party/move selection. The chosen move gains the exact source-declared
PP amount capped at its maximum, HP stays unchanged, Ether is consumed and walking resumes. Party,
bag and both victory/all pickup flags survive Continue at `(4,18)`.

These local checks do not establish the earned connected regional story or
final visual acceptance. The full twenty-route objective remains unfinished.

Stdio MCP continues the exported journey at `(4,18)`, moves to `(4,19)`,
opens/closes production Start and emits empty stderr. The production TUI loads
the same save with Croconaw 22 at 21/61 HP and Flaaffy 20 at 58/58, opens
Start/Pack and returns to movement. Super Potion remains in the Items pocket;
the Ether used in the journey is gone.

The canonical Mom/New Bark gate, battle-command/PP, MCP movement/Start and
clean-stdio regressions pass against the original external pack. Changed Rust
leaf files pass scoped rustfmt. Focused all-target Clippy passes for assets,
Bevy and geothite with `--no-deps`, `location-tester` and the existing
`never_loop` allowance; existing warnings remain outside the new route code.
Workspace formatting still reports pre-existing differences beginning at
`crates/offset-allocator/src/lib.rs:408`, which this map change leaves alone.
