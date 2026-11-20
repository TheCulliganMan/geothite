# Mothlight Lodge

An optional lodge on Cedar Approach, north of Grand Marais. The ground floor is a long dining hall with a continuous painted moth-wing vault, a broad river-stone fireplace, tall windows, navy tablecloths, bentwood chairs, star lamps and a midnight-blue runner. Upstairs, three red-quilted cedar beds sit under the painted eaves. The exterior is an original eight-block-wide cedar building with tall red-trimmed windows and a central entrance tower.

The visual reference is [Naniboujou's dining room](https://www.naniboujou.com/about/) and its [cedar exterior](https://www.naniboujou.com/wp-content/uploads/nani-9-2020-1024x683.jpg). The [architect's interior photograph](https://mgerwingarch.com/m-gerwing/2018/8/16/naniboujou-lodge-minnesota) informed the saturated orange, gold, navy and green, large connected mural surfaces, stone hearth and blue linens. Moth wings and star patterns are original Northstar artwork. This is a native north-up 2D interior, not a perspective renderer.

Eight dining groups each span four metatiles: a round blue table and four inward-facing chairs. Seats and tabletops block walking; the corners and broad aisles remain traversable.

The entry forecourt uses staggered flagstones, cedar benches and spruce planters. Broad woven gold diamonds frame the wall murals. All art uses four shade indices per 8×8 tile, native RGB555 colors and at most eight palettes per tileset; the indoor pigments share a warm cream highlight and dark ink.

## Play

Marnie is looking for the cook's missing raspberry cake. Crumbs at a dining table lead toward the cookbooks; examining the shelf reveals that the cook hid it from himself. Marnie gives one Rare Candy. A full Pack leaves the reward available, and the receipt flag prevents duplicates. The hearth attendant heals the team for free. Other guests comment on the ceiling and their questionable sleeping habits. Neither activity grants a badge or bypasses the shore gate.

## Authored contract

Coordinates below are 16px walking tiles, not 32px metatiles.

- Cedar Approach retains all old terrain outside the explicit pine reservation, all existing objects, scripts, encounters, shore-gate warps and regional connections. Its new warp 12 is at `(30,9)`; the safe approach is `(30,10)`. A sign is at `(34,10)`.
- `MnMothlightLodge` is 13×15 metatiles. Warp 1 `(13,29)` returns to Cedar warp 12. Warp 2 `(20,8)` leads to the loft. Arrival review square `(13,28)` is floor.
- `MnMothlightLoft` is 8×6 metatiles. Warp 1 `(12,8)` returns to hall warp 2. `(13,8)` is a safe floor square beside the stair.
- Marnie `(8,16)`; hearth attendant `(10,9)`; diner `(17,16)`; cook `(2,17)`; sleeper upstairs `(5,6)`. All are stationary ordinary people.
- Table clue `(3,13)`, cookbook `(2,25)`, hidden cake `(5,25)`, hearth `(12,7)`, mural `(6,7)`.
- `EVENT_MN_MOTHLIGHT_CAKE_FOUND` records examining the cake; `EVENT_MN_MOTHLIGHT_CAKE_REWARD` records successful delivery of one Rare Candy.

`layout.json`, `dialogue.json`, `art.json` and `hall.pal` are editable authoring sources. The indexed art stamps each contain 32×32 pixels, four shade indices, a palette per 8×8 cell, and explicit collision quadrants. The Rust compiler deduplicates glyphs into private 256-glyph native atlases. Old source atlases are never modified. No extracted source catalog or compiled pack is tracked.

## Build and review

The builder requires the exact Crossings content identity and writes only a fresh ignored output:

```sh
CARGO_INCREMENTAL=0 cargo run --locked -p crystal-assets --bin pack_mothlight_lodge -- \
  content-packs/minnesota-crossings-v01.crystalpack \
  content-packs/minnesota-mothlight-v10.crystalpack
```

v10 is selected locally after the checks below. It has not been deployed and existing saves were not migrated. Generated packs, reference photographs, source previews and native renders remain under ignored `content-packs/` and `target/mothlight-review/`.


The disposable controller review uses the production stdio client. Build the
`mothlight_fixture` example, then run both journeys with fresh save paths:

```sh
cargo build --locked --release -p geothite --example mothlight_fixture
target/release/examples/mothlight_fixture content-packs/minnesota-mothlight-v10.crystalpack target/mothlight-review/normal.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/landmarks/mothlight-lodge/play_check.py content-packs/minnesota-mothlight-v10.crystalpack target/mothlight-review/normal.crystalsave
target/release/examples/mothlight_fixture content-packs/minnesota-mothlight-v10.crystalpack target/mothlight-review/full.crystalsave --full-bag
.local/minnesota-review-env/bin/python planning/minnesota/landmarks/mothlight-lodge/play_check.py content-packs/minnesota-mothlight-v10.crystalpack target/mothlight-review/full.crystalsave --full-bag
```

The Python review needs `cbor2`; it reads collision to plan routes and sends
normal inputs through the real controller. The fixture starts a fresh disposable
party at Cedar Approach with 1 HP. It is not an earned campaign playthrough or a
migration of an existing save. `MOTHLIGHT_FIXTURE_BIN` can select a locally built
copy of the same Rust fixture source.


## Verified v10 result

- File SHA-256: `07de9efc54913119e25b461aba703e93e394b2bab755496dec65dda4faec949f`.
- Content identity: `ad63c0f1b5d452e9d20823053c740e2931a3891aa0e0850ed7cd63954f88c10a`.
- 745 maps; every pre-existing map except the explicit Cedar edits preserved by the builder. All 602 audio PCM hashes, frame counts and loop ranges match.
- Production controller: Cedar entry, all clue interactions, 1 HP to full healing, blocking chairs, one Rare Candy, repeat without duplicate, loft entry/exit, Save/Continue upstairs, return to Grand Marais and Start passed. A separate full-Pack fixture kept the reward unclaimed and released input after repeated refusals. Existing trainer sightlines are routed around, not disabled.
- Native full-map renders: `target/mothlight-review/native-v10/`. Actual Bevy review: `entry-v10.png`, `hearth-v09.png`, `loft-v09.png`, and `table-quads-v07.png` under the same review directory. Later variants preserve their predecessors; v10 changes only the outdoor bench/planter pigments from v09.
- `cargo fmt --all -- --check` and explicit new-module rustfmt checks passed. `cargo clippy --locked -p crystal-assets -p geothite --all-targets --features crystal-assets/pack-authoring` passed with existing warnings and no Mothlight warnings. The earlier default-feature invocation was blocked by an unrelated in-progress authoring binary import. A one-line tuple-field correction in the concurrent Gym lesson module unblocked the shared build.
- The fixture source was compiled against the existing release dependencies for this review; its standard Cargo build command is above. No browser/public deployment or earned campaign coverage is claimed.
- Removed the unused standalone renderer binary, recovering 183 MiB. Shared active build output and all review images, packs and saves were retained; the volume still had over 370 GiB free.

## Battle café

The cook at `(2,17)` now offers an optional battle with level-36 Mothosaic and
Muffiend. A win earns one Berry Juice per game day. Declining or losing leaves
the challenge available. The daily win receipt prevents repeat battles that day;
a separate pending-prize receipt holds the juice if the Items pocket is full.
Midnight clears only the daily win receipt. An older unclaimed prize remains
collectible, then the new day's battle becomes available. Marnie's cake quest,
the hearth attendant, loft, furniture and original art remain intact.

The café dialogue is authored under `mothlight_battle_cafe` in
`planning/minnesota/story.json`. The Rust overlay requires the reviewed parent;
it changes only the cook's program, its dialogue, the trail sign, and the new
trainer and receipts. The shared daily-reset handler expires the café win.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets \
  --features crystal-core/test-fixtures --bin pack_mothlight_battle_cafe
target/debug/pack_mothlight_battle_cafe \
  content-packs/minnesota-mothlight-northern-trails-v01.crystalpack \
  content-packs/minnesota-mothlight-battle-cafe-v01.crystalpack
CARGO_INCREMENTAL=0 cargo build --locked -p geothite --bin geothite \
  --example mothlight_fixture --example terminal_art_eval
```

Use fresh ignored save names. The café review uses real MCP/controller inputs
for entry, conversations, battle turns, Pack actions, Save/Continue and exit.
`--next-day` is an explicitly simulated daily boundary in the disposable fixture;
it calls the production reset handler rather than waiting overnight.

```sh
export MOTHLIGHT_CLIENT_BIN=target/debug/geothite
export MOTHLIGHT_FIXTURE_BIN=target/debug/examples/mothlight_fixture
pack=content-packs/minnesota-mothlight-battle-cafe-v01.crystalpack
save=target/mothlight-cafe-review/normal.crystalsave
$MOTHLIGHT_FIXTURE_BIN "$pack" "$save"
.local/minnesota-review-env/bin/python \
  planning/minnesota/landmarks/mothlight-lodge/cafe_check.py "$pack" "$save"
```

Repeat with fresh saves and `--full-bag` or `--lose` on both commands. Add
`--midnight-full` to the Python full-Pack journey to carry the held juice across
a simulated daily reset. The original `play_check.py` still covers the cake,
healing, loft and return journey. These are disposable fixtures, not an earned
campaign playthrough or a migration of a player's save.

### Verified café result — November 19, 2026

`minnesota-mothlight-battle-cafe-v01.crystalpack` is selected locally in
`world.json`. Its parent is the selected Northern Trails Mothlight revision,
not the older lodge-only v10. File SHA-256:
`5fd62866c08058192f6f939ea7a3ce641aad19b048107a2c32ab468619613edc`;
content identity:
`701440cf37077155a8c690d54fddc48d91aadb3bf6f7ffcee52525dc034f04a8`.

Verified against this exact pack:

- Real production MCP: voluntary refusal; both opponents defeated; move PP,
  EXP and money changes; one Berry Juice; same-day refusal without duplicates;
  Save/Continue; a second win after a simulated daily reset; loft and town return.
- Full Pack: victory retained the unpaid prize, including across Continue and
  a simulated midnight. Normal Pack/TOSS inputs freed a slot, the cook delivered
  exactly one juice, and an older held prize did not consume the new day's battle.
- Loss: a damaged fixture lost through the normal battle engine, recovered at
  Grand Marais's clinic, walked back and received the same-day challenge again.
  The journey crosses the retired road guards' tiles with actual inputs;
  their historical source rows are not evidence of active collision.
- Original lodge regression: healing from 1 HP, crumbs/book/cake discovery,
  one Rare Candy, repeat without duplication, blocking furniture, upstairs
  Save/Continue, return to Grand Marais and the production Start menu. When
  Repel expired on the longer walk, the test used the real RUN command.
- Six daily-reset unit tests passed, including café receipt serialization and
  preservation of the cake receipt. `cargo fmt --all -- --check` and the full
  affected-package Clippy run passed with existing warnings and no café warnings.
- Native 2D screenshots inspected the cook, opening dialogue and restored
  final Yes/No page. All native artwork and all 602 audio programs, PCM hash
  metadata, frame counts and loop ranges exactly match the parent. Only the
  lodge and its approach map differ; all other maps remain equal.

Evidence stays ignored under `target/mothlight-cafe-review/` (`normal`, `full`,
`midnight-full`, `loss6`, and `quest2` logs/transcripts; `offer-walk.png` and
`saved-question.png`). This is scoped fixture verification, not an earned
campaign or a public deployment. Existing player saves were not rewritten.

Launch a new local session with the rebuilt production client:

```sh
target/debug/geothite play content-packs/minnesota-mothlight-battle-cafe-v01.crystalpack \
  --save target/mothlight-cafe-review/player.crystalsave
```

For a disposable lodge review, copy the completed `normal.crystalsave` to a fresh
ignored save and use `--load` and `--save` with that copy. Do not substitute the
identity of an older save.
