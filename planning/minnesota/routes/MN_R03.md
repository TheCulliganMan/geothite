# Minnesota River Valley: compact native review

## Grass and directional ledge pass, 2026-10-28

The current pass uses original Crystal Route30/42/45 as references. The bird
meadow now requires five grass steps northbound and offers a dry southbound
ledge return. Native terrain has 224 encounter tiles, up from 112. The review
pack and native 2D/3D captures pass exact terrain/nonterrain preservation and
directional checks. Production controls passed all four ledge lanes and the
populated journey with battles, native medicine and Save/Continue. See
[the current route runbook](../revisions/route-grass-v01/R03.md). This route is accepted in the named review pack; subsequent routes remain
separate work.

This hand-authored replacement is **24×32 metatiles**, two native maps and one small cave. It replaces the repeated 64×144 twelve-chunk layout. The river's stepped bends, two required timber crossings, lower habitat shelf and optional limestone terrace form a narrow north/south journey rather than another farmyard or lake fork.

[Whole route](../renders/river-valley/whole-route-13d463b2.png) · [ten player-scale views](../renders/river-valley/viewpoints.png) · [cave interior](../renders/river-valley/interiors.png) · [native upper bridge](../renders/river-valley/native-upper-bridge.png) · [native lower bridge](../renders/river-valley/native-lower-bridge.png) · [native terrace](../renders/river-valley/native-terrace.png) · [native cave mouth](../renders/river-valley/native-cave-mouth.png) · [native cave](../renders/river-valley/native-cave.png).

**Historical boundary pass:** bounded source boundary/canopy repair, 2026-10-26. The
historical native captures and populated journey below describe the preceding
publication. New source pixels/collision pass; changed blocks await native
republication and controller verification. Actor, room, party, port, encounter,
bridge, ramp and ledge coordinates are preserved.

## Bounded river audit

The complete route and actual native upper/lower bridges and terrace cameras
were inspected. Its compact two-crossing journey holds, so no broad redesign was
needed. All outdoor actors stand on actual FLOOR, their same-chunk approaches
remain clear, and both bridge/approach footprints have zero direct Floor ↔ Water
leaks. Both crossings remain necessary, the six-step habitat remains unavoidable
on ordinary travel, and the native `74`/`57` terrace still gives a three-tile hop
with an eleven-step ordinary ramp return.

The audit found sixteen open outer WATER cells: north `(30..37,0)` and south
`(28..35,63)`. Native blue-water rock recipes now close each mouth: metatile row0,
x15..18 uses `30/31/31/32`; row31, x14..17 uses `38/39/39/3a`. The complete
220-core-cell perimeter contains zero WATER, with sixteen explicit outer BUOY
cells. Conservative walking+Surf reaches 434 water cells and no outer water edge.
The separated southern 38-cell water pocket is also seeded and checked; its
boundary camera uses explicit Surf fixture placement, not an invented walking
launch. Total source WATER is 472 cells. The internal river seam at y47↔48, x20..31 stays open: native publication must
retain real reciprocal `MnR03C00R00` south ↔ `MnR03C00R01` north, offset zero.
The existing isolated native pack contains these connections; the default base
art pack has no route maps, and the checker reports that distinction.

Decorative `63` at metatile `(11,6)` contained incidental Cut core `(22,13)`
without gating a destination. Ordinary source canopy `3d` replaces it; an
explicit empty `cut_gates` contract and complete native-quadrant inventory now
prove zero Cut. No Cut prerequisite is introduced to this badge-two journey.

[Current whole](../renders/river-valley/whole-route-13d463b2.png),
[actual emptiest camera](../renders/river-valley/viewport-emptiest_ground-13d463b2.png),
[north/south Surf boundary cameras](../renders/river-valley/surf-boundaries-13d463b2.png)
and both player bridge views were inspected. The largest blank ground square
is three metatiles near the inhabited bird meadow; its actual `(13,9)` camera is
included rather than hiding the widest patch. Only the boundary mouths and one
canopy toe changed visually.

Ten negative fixtures reject either river escape, brown land boulder substituted
into the water-rock boundary, decorative Cut, removed ramp, flattened native
ledge, either blocked bridge deck, and removed upper/lower rails:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_river_valley.py --self-test
```

These pass against both the base art input and existing native overlay art;
Python compile checking passes. A warm source render/check takes about 145 ms.
Current reachable counts are 778 clear / 890 including grass, 112 grass, four
trainers, two residents, three outdoor resources, five signs and one cave.
Shared owner must republish the eight water-rock blocks and ordinary canopy,
compare every native tile/event, verify both physical water edges and reciprocal
Surf seam travel, inventory Cut, and rerun the original geometry/populated
journey regressions before marking this repair natively verified. Those existing
fixtures remain valuable, but a historical pack does not verify revised blocks.

## Physical route and decisions

Coordinates below use global 16px gameplay tiles. The upper map is `MnR03C00R00`, 24×24 metatiles. The lower map is `MnR03C00R01`, 24×8 metatiles, with global gameplay origin `(0,48)`. For example, global `(18,58)` is lower-map local `(18,10)`.

| Place | Implemented behavior |
| --- | --- |
| Mankato arrival | South gate `(18,60)`, landing `(18,56)`. A short western spur gives Repel `(13,55)`; the road then turns east onto the lower bridge. |
| Lower timber bridge | Deck core x=28–35, y=54–55, between approaches x=26/37. Both deck rows work both ways. North timber rail has WALL at y=52 and FLOOR at y=53; south rail FLOOR y=56/WALL y=57. Animated water remains below the rail. Super Potion `(41,55)` rewards the small eastern landing. |
| Native map seam | The east-bank road crosses global y=47/48. Both maps are 24 metatiles wide and share offset 0. Native controller input crosses and returns; no actor stands on the seam. Cal is at `(37,49)`, one tile inside the lower map. |
| Habitat shelf | The band x=26–33, y=34–39 requires at least six grass steps. River closes the western side, the terrace closes the eastern side, and the optional ledge cannot bypass it. Arrivals and bridge landings are clear before the habitat. |
| Optional terrace | Native ramp `74` opens at `(41,40–41)` from the lower path `(41,42)`. Ether `(39,35)` and Ed's optional battle reward the ascent. South-facing ledge `57` at x=36–39, y=40–41 drops from approach `(37,39)` to floor `(37,42)`, skipping the wall face. Reverse Up is blocked; the eastern ramp provides the return. The required road stays bidirectional below the terrace. |
| Upper timber bridge | Deck x=18–25, y=28–29, between approaches x=16/27. North rail WALL y=26/FLOOR y=27; south rail FLOOR y=30/WALL y=31. Removing either complete bridge span disconnects the two town arrivals. |
| River Pocket Cave | CAVE `(8,37)` enters `MnR03Cave1F`, an 8×6 room with a dry loop around a native boulder cluster, a walker and Great Ball `(12,5)`. Its ladder `(7,9)` returns to a separate exterior FLOOR warp `(8,38)`. Walking across that floor does not enter the cave. The nearby grass neck is optional. |
| Bird meadow | Two short habitat patches, flowers, a birdwatcher and Jo's optional battle distinguish the upper bank from the lower fishing approach. The northern path joins the city approach. |
| Minneapolis arrival | North gate `(20,2)`, landing `(20,6)`. Both bridges remain available on the return. No Cut, Surf or required one-way hop is needed. |

Four optional trainers use native parties and persistent flags: Cal (Wooper 10/Poliwag 10), Jo (Hoppip 11/Pidgey 11), Ruth (Bellsprout 12/Mareep 12), and Ed (Slowpoke 12). Two outdoor residents, five signs and four rewards across exterior/cave have exact approaches. Reserved trainer flags 05–08 remain unused. The condition remains `BADGE_2`; neither resident completes Q02 or grants a badge. Real Mankato/Minneapolis ports, earned Q02 and regional recovery remain integration work.

## Source art and fast loop

The repaired exterior uses 40 metatile varieties. It reuses the bounded `mn_headwaters` timber-rail extension: all 128 original Johto metatiles remain unchanged, and three source recipes append at `80`–`82`. Existing animated water, source palettes and collision are retained. Native/source comparisons verify all 131 metatile images. There is no new painted art, source catalog or external exporter.

Native inspection exposed a black central void in the first cave room. It was replaced with native cave `18`/`1a` boulder quadrants, then recaptured and input-tested. The cave's `01` void belongs outside the playable room, not in a claimed rock obstacle. The limestone exterior was also broken into a stepped outline with two source outcrops instead of an empty rectangular slab.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_river_valley.py --serve
```

Open `http://127.0.0.1:8794/`. Edit [terrain strokes](MN_R03.layout.json) and [events/dialogue](MN_R03.json). Whole route, ten viewport crops, cave and anchors refresh without Cargo; a warm source check/render measured about 130–180ms. Invalid geometry remains marked invalid. Publication is separate. Regenerate review saves after a changed pack identity; do not patch their hashes.

Static checks prove both bridge spans are necessary, both route directions remain available, the terrace cannot bypass required habitat, all actor/door approaches resolve and the cave is connected. They establish intended geometry, not visual approval. Source/native comparison checks all 768 exterior metatiles, nine outdoor objects, five signs, all parties/dialogue and the room/return warp.

## Historical native build and verification (before boundary repair)

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_river_valley -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R03.layout.json \
  planning/minnesota/routes/MN_R03.json \
  content-packs/minnesota-river-valley.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_river_valley.py \
  --pack content-packs/minnesota-river-valley.crystalpack --verify-published
CRYSTAL_RIVER_VALLEY_TEST_PACK="$PWD/content-packs/minnesota-river-valley.crystalpack" \
CRYSTAL_RIVER_VALLEY_FIXTURE_SAVE="$PWD/.local/river-valley-review.crystalsave" \
CRYSTAL_RIVER_VALLEY_JOURNEY_SAVE="$PWD/.local/river-valley-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --lib --profile wasm-dev --locked \
  -p crystal-bevy --features location-tester river_valley_ -- --ignored
CRYSTAL_RIVER_VALLEY_TEST_PACK="$PWD/content-packs/minnesota-river-valley.crystalpack" \
CRYSTAL_RIVER_VALLEY_JOURNEY_SAVE="$PWD/.local/river-valley-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p geothite --test river_valley -- --ignored
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-assets --test river_valley --test prairie_farm --test driftless \
  --test central_lakes --test headwaters --test league_approach \
  --test gunflint --test north_shore
target/runtime-fps/wasm-dev/geothite play \
  content-packs/minnesota-river-valley.crystalpack \
  --load .local/river-valley-journey.crystalsave --compact
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-river-valley.crystalpack --audio --desktop
```

The isolated pack has **391 maps**. An in-memory composition with Prairie Farm has 394 and preserves every prior map, runtime file, HOME/recovery destination and audio program. Eight route asset regressions pass. The River Valley regression additionally rejects a changed badge condition, removed habitat, a floor replacement for the ledge, a landing inside its wall face and removal of either bridge. All 377 audio programs retain their PCM hashes, frame counts and loop ranges. Packs, renders and fixture saves remain ignored.

Two native-controller fixtures pass against the final cave revision. The empty-party geometry fixture checks both maps, seam travel both ways, both rows of both bridges in both directions, rail barriers, ramp and ledge/reverse, every sign/resident/pickup, the cave's two sides, dialogue input ownership and a safe cave return. It establishes geometry separately from battle balance.

The populated-party journey uses source-created Croconaw 18/Flaaffy 16, two badges and a deterministic divider as explicit setup. It escapes one real wild encounter, crosses both bridges and the required habitat in both directions, visits the optional terrace and collects Ether. All four trainers/seven opposing species are defeated through visible battle menus with actual PP spent. Party HP/status/PP, bag, Ether and all four defeat flags survive Continue at global `(18,58)`. This is an isolated local journey, not earned badges or a connected regional playthrough.

Visual approval, losing/recovery through Minnesota hubs and the earned Q02 journey remain open. The full twenty-route objective is unfinished; do not use this route's arrangement as a template for other regions.


The journey also collects the lower bridge's Super Potion and uses it through
production Start → Pack → USE → Croconaw. HP increases from 5/54 to 54/54,
the item is consumed, its pickup flag survives Continue and ordinary walking
resumes. Ether remains in the bag. This uncovered a shared Pack targeting bug:
ordinary HP medicine had no party-target branch despite an existing compiled
healing plan. `RuntimeItemCatalogSnapshot::party_hp_restore` now exposes that
plan to the controller; PP-item fields are excluded so numeric PP amounts are
not treated as HP recovery. Pack targeting uses that capability, without item
name guesses or changes to the supplied pack's item catalog.

A focused runtime regression passes for a custom HP-effect alias, maximum HP
recovery, PP-target exclusion and status-only medicine. The wider
`cargo test --lib --profile wasm-dev --locked -p crystal-runtime runtime_battle_item_`
run has **11 passing tests and one failure**:
`runtime_battle_item_heals_active_party_pokemon_from_exact_pack_effect` stops
at `runtime_tests/special_routines.rs:971`, where its fixture expects an active
combat before reaching the item-use call. That setup failure remains outside
this map and catalog change. Do not report the whole runtime suite as green.

The canonical Mom/New Bark gate, battle-command/PP, MCP movement/Start and
clean-stdio regressions pass with the controller change. Stdio MCP resumes the
completed, healed route save at lower-map local `(18,10)`, moves to `(18,11)`,
opens/closes Start and emits empty stderr. The production TUI loads the same
save, shows Croconaw 19 at 54/54 and Flaaffy 16 at 48/48, and opens Start/Pack, returns to movement and exits cleanly.

New route leaf modules pass scoped rustfmt. Focused all-target Clippy passes
for assets, runtime, Bevy and geothite with `--no-deps` and the existing
`never_loop` allowance; it reports existing warnings outside the new route and
medicine code. Workspace formatting still fails beginning at
`crates/offset-allocator/src/lib.rs:408`. The touched runtime root, economy and
runtime test files also have existing formatting differences outside the new
hunks; those unrelated lines are preserved.
