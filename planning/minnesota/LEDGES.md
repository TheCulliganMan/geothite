# One-way paths in the compact routes

Ledges shorten a deliberate loop or change which direction a branch can be
used. Use the source `HOP_DOWN`/`HOP_LEFT` face, its reverse WALL quadrant,
closed sides and a clear landing. The player must be able to return by a
connected walking route. Do not make a required journey depend on climbing a
ledge, or place an actor, pickup, warp or water at its landing.

The current four compact rural routes have exact authored contracts:

| Route | Hop: approach → landing, core tiles | Purpose and walking return | Shortest return |
| --- | --- | --- | ---: |
| [Driftless](routes/MN_R01.md) | `(29,15)` → `(29,18)`, south | Westbound bluff alternative drops onto the lower road. Uphill uses the eastern ramp, core rectangle `[40,18,42,20)`. | 43 tiles |
| [Prairie Farm](routes/MN_R02.md) | `(48,15)` → `(45,15)`, west | Nursery shortcut works westbound. Eastbound walkers use the southern farm lane, `[48,23,55,28)`. | 41 tiles |
| [River Valley](routes/MN_R03.md) | `(37,39)` → `(37,42)`, south | Drop from the optional reward terrace. Its eastern ramp, `[40,40,42,42)`, provides the climb back. | 11 tiles |
| [Central Lakes](routes/MN_R15.md) | `(22,17)` → `(22,20)`, south | Northern meadow bypasses Ron eastbound; the lower mouth cannot be used to enter the meadow. Returning through the western entrance, `[14,18,16,20)`, passes Ron's challenge until his defeat. | 31 tiles |

Distances count ordinary core-tile walking steps through FLOOR/TALL_GRASS,
with actor bodies blocked. They are geometry measurements, not elapsed time
or button presses. Grass and trainer sight can still affect the journey. Each
drop spans three core tiles from approach to landing. The rectangle bounds
are half-open; the upper coordinates are excluded.

## Review and implementation

Open the [first route's movement overlay](renders/driftless/ledges.png),
[farm overlay](renders/prairie-farm/ledges.png),
[terrace overlay](renders/river-valley/ledges.png) and
[meadow overlay](renders/central-lakes/ledges.png). Amber shows the permitted
hop, red marks the reverse wall, and cyan shows the shortest walking return.
These annotations sit above unchanged native source pixels and are not game
terrain. [The first route's native player view](renders/driftless/native-ledge.png)
shows the ledge at production scale.

Each `.json` dossier's `one_way_ledges` declares collision coordinates,
`return_passage_core`, and `minimum_return_steps`. Keep those synchronized
with its `.layout.json` strokes. The Rust publisher rejects an occupied
landing, a missing native hop/wall, a disconnected or trivial return, and an
uphill route that bypasses the declared passage. It excludes warp triggers
and other hops from the walking-return graph. The four publishers have
negative regressions for an unrelated return passage and an impossible
return-distance claim, in addition to the existing ledge/landing checks.

The source review loop performs the same return-path checks and generates a
**One-way paths** layer on every edit. For example:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_driftless.py --serve
```

Open `http://127.0.0.1:8771/` and select **One-way paths**. The four warm render
passes measured 108–181 ms including source collision checks and PNG output.
Cargo is only needed when publishing or changing Rust, not while moving an
authored stroke. Invalid ledge geometry remains marked invalid.

Native controller fixtures exercise the downhill hop, rejected reverse input
and the complete uphill walking return. Driftless, Prairie Farm and River
Valley use explicit empty-party geometry setup. Central Lakes' populated-party
journey first skips Ron eastbound, meets his challenge westbound, then walks
the full 31-tile return after his persistent defeat flag is set. It rechecks
that the ledge still rejects reverse input after victory. Separate journey
fixtures cover other battles where already implemented; these checks do not establish the earned regional story
or final visual acceptance. No terrain expansion is required for this ledge
pass, and it does not complete the twenty-route objective.

## Verified commands

Four asset-publication regressions pass, including the new return-distance and
passage rejection cases. Four native geometry fixtures and the expanded
Central Lakes journey pass. The latter escapes three wild encounters, wins
both trainer battles with PP spent and preserves party/bag/flags through
Continue. All four native player-scale ledge captures were inspected.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-assets --test driftless --test prairie_farm \
  --test river_valley --test central_lakes
CRYSTAL_DRIFTLESS_TEST_PACK="$PWD/content-packs/minnesota-driftless.crystalpack" \
CRYSTAL_PRAIRIE_FARM_TEST_PACK="$PWD/content-packs/minnesota-prairie-farm.crystalpack" \
CRYSTAL_RIVER_VALLEY_TEST_PACK="$PWD/content-packs/minnesota-river-valley.crystalpack" \
CRYSTAL_CENTRAL_LAKES_TEST_PACK="$PWD/content-packs/minnesota-central-lakes.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --lib --profile wasm-dev --locked \
  -p crystal-bevy --features location-tester controller_walks_ -- \
  --ignored --skip gunflint --skip headwaters --skip league_approach
CRYSTAL_CENTRAL_LAKES_TEST_PACK="$PWD/content-packs/minnesota-central-lakes.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --lib --profile wasm-dev --locked \
  -p crystal-bevy --features location-tester \
  central_lakes_journey_uses_grass_ledge_bypass_real_trainers_and_saved_flags \
  -- --ignored
```

Changed Rust files pass scoped rustfmt. Workspace `cargo fmt --all -- --check`
still reports pre-existing differences starting at
`crates/offset-allocator/src/lib.rs:408`; those unrelated changes are preserved.
Focused all-target Clippy passes for `crystal-assets` and `crystal-bevy` with
`--no-deps`, `location-tester` and the existing `never_loop` allowance. It
reports existing warnings outside the new ledge changes.

## Explicit local return loops

A lake can have both a short shore return and a much longer walk around its
opposite bank. `one_way_ledges[].return_region_core` optionally declares the
half-open core bounds of that local loop. Both hop endpoints and the complete
`return_passage_core` must lie inside it. A real ordinary path must exist inside
the region and become disconnected when its declared passage is closed. The
whole-map shortest ordinary return must still meet `minimum_return_steps`;
a cheap escape outside the region remains invalid. Omitting the field retains
the original whole-map passage requirement. Hops and warp triggers are excluded
from both return graphs. This describes validation semantics; each route still
needs its separate controller and visual evidence.

Native `HEADBUTT_TREE` and `HEADBUTT_TREE_1D` quadrants are permanent wall
barriers and can guard a ledge side. Headbutt starts an encounter without
clearing that collision. `CUT_TREE` can open and remains invalid as a permanent
side guard. Choose the art that belongs to the terrain; do not add artificial
rock fins merely to satisfy a narrower token list in a planning checker.
