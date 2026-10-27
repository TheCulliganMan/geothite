# Grand Marais, the native coast gate and Gunflint

The local successor is
`content-packs/minnesota-northstar-north-shore-story.crystalpack`: **544 maps,
251 species, 251 moves and 377 audio programs**. File SHA-256:
`644d6dba7ded59795ab890febcad5d5a63878186743521c74d5acc294101cd8e`.
Content identity:
`ab793e8df7a4f16f4b811aad5e2fc5e34c59fa394c33907f718e5a9fd4ea3c8f`.
The exact input is the [543-map Itasca story](itasca-story.md), identity
`4b9dbd5018544719027de20886cafd1a662699d69c9e39e5f3a8562cc8be8bc2`.
This preserves the frozen story lineage. Separate route-quality and expanded-city
compositions remain separate; no merge or replacement is implied. Packs,
builds, renders and saves remain ignored.

The authoring source is
[MnGrandMarais.connections.json](../towns/MnGrandMarais.connections.json).
It changes six installed map modules and adds one room. Ryan's house,
Baptism River waterfall, all previous story scripts and all other maps stay
exact. No badge, regional story result, HOME change or fee is added. Two
visit-selected clinic recovery records repair the missing regional destinations.

## Two real regional entrances

The coast trail enters through **IlexForestAzaleaGate**, the original east–west
Crystal gatehouse requested by the user. The entire native 5×4 metatile room
is unchanged. Native Johto roof, brick facade and side carpets make the two
exteriors. The coast trail curves into its west entrance at the northern end
of R11; the town entrance occupies Grand Marais's western quay. This is a
paired warp connection, with no invented outdoor seam across the harbor.

| Side | Active outside entrances | Safe outside returns | Passive room arrivals | Active room exits |
| --- | --- | --- | --- | --- |
| R11 west entrance | `(43,4)/(43,5)`, indices 3/4 | `(42,4)/(42,5)`, 5/6 | `(1,4)/(1,5)`, 5/6 | `(0,4)/(0,5)`, 1/2 |
| Town east entrance | `(4,18)/(4,19)`, indices 7/8 | `(8,18)/(8,19)`, 9/10 | `(8,4)/(8,5)`, 7/8 | `(9,4)/(9,5)`, 3/4 |

Town returns land **inside** its checkpoint. An earlier return at `(5,18/19)`
was trapped by the guards: warp arrival does not trigger the outside coordinate
event. The corrected destination does not depend on that event. Original
Ryan-house warp indices 1/2 and all six town door indices are preserved.

Gunflint is an ordinary two-lane walking connection: town west `(0,8)/(0,9)`
to `MnR12C01R01` east `(63,8)/(63,9)`, reciprocal offsets **0/0**.
Both outbound town checkpoints require all eight badges and the acknowledged
**Duluth** signal, `EVENT_MN_LIGHTHOUSE_SIGNAL`. Both allow unqualified inbound
return to town and free care. Neither the spare part nor the local lamp flag
is a road prerequisite. Native connection projection checks both directions
and both lanes before publication.

The town's formerly open water perimeter is now enclosed with ten bounded
variants assembled from exact native Johto quadrants. Only outward WATER
quadrants become native BUOY reefs; retained shoreline/land quadrants, glyphs
and palettes remain exact. No whirlpool is used as a permanent border. Three
native promenade blocks keep a dry approach to the western quay around the
gate's forest collar. Native walking/ledge/Surf reachability checks closed
checkpoints, the open town and both complete road perimeters.

Surf testing found the original demo town had no water encounter table.
The harbor now inherits **only** the adjoining `MnR11C00R00` water table:
rate 4, level-38 Poliwag, level-40 Quagsire and level-42 Mantine at each time
of day. Original grass encounters and every other encounter table remain exact.
The new tileset inherits Johto's existing Cut/Whirlpool replacements and
native water/flower animations.

## A local lamp repair, separate from the story signal

The existing scientist at `(5,3)` offers a spare Machine Part as an ordinary
optional conversation. The demo exit interception, Totodile/Berry gift,
five balls and three potions are retired. No rod was in that original bundle.
Declining leaves inventory, party and money unchanged; the scientist offers
again. A full key-item pocket retains the offer. Acknowledging receipt sets
only new event **2309**, `EVENT_MN_MARAIS_SPARE_PART_RECEIVED`.

The existing keeper takes the part, restores
`EVENT_MN_GRAND_MARAIS_SIGNAL` and gives one Super Potion. That is a distinct
local harbor lamp, not the mandatory northern relay. Repeat conversations
give no additional part or reward. Existing full-bag keeper refusal preserves
the part and permits retry. The scientist's older repair reminder/thanks
remain reachable. The old demo starter flag remains false.

The native clinic is **MnGrandMaraisPokecenter**, 5×4 metatiles; the old plan's
`Pokecenter1F` 10×8 room did not exist. The nurse stands at `(3,1)` across the
counter at `(3,2)`, approached from `(3,3)`. Original room carpets are
`(3,7)/(4,7)`. Exiting automatically steps to town `(15,10)`; another Down
reaches `(15,11)`. Existing indexed doors retain their original native automatic
exit behavior rather than being represented as modern passive returns.

The longer battle course exposed that Duluth's nurse healed the party but
neither regional clinic had a registered recovery record. A loss could send
the player to Winona's HOME. The successor now adds Duluth `(10,13)` and Grand
Marais `(3,5)` recovery records, preserving every original spawn and HOME.
Duluth registers its native `blackoutmod` immediately after actual healing;
Grand Marais wraps its original standard nurse script with its own registration.
The original healing, YES/NO and other conversations remain intact. Registration
is selected through nurse interaction, never granted as initial story progress.

## Executed evidence

Two asset cases pass written identity, native room blocks, cache-free map
reconstruction, exact declared terrain patches, native glyph copies, original
source files/audio/recovery and unrelated content preservation. Duplicate
publication, unsafe arrivals, repainted land, broken mouth terrain and a local
lamp prerequisite are rejected. The latest global planning check passes
**287 planned maps and 88 adjacency/gate contracts**, including separate
expanded-city source work. Those city candidates are not merged into this
frozen native shore pack. Planning validation does not execute gameplay.

Six controller cases pass on this identity (31.48 seconds): both gatehouse lanes/Continue and Gunflint seam;
missing badge/signal refusal and unqualified inbound return; optional part
decline/retry/receipt/Continue and once-only keeper repair; injured-partner free
care and both clinic carpets; and actual Surf launch, reef boundary, Surf-mode
Continue and native dismount; and played losses after healing/Continue at both
regional clinics. The latter prove native money loss, restored HP/PP, unchanged
bag/badges, correct recovery destination and another Continue.

The complete earned journey also passes (149.30 seconds). From the original
Brainerd six-badge/Mew45/Cut/placement setup, it earns Pine after the actual
loss/care/retry, the reading/report, foreman win, separate meter recovery,
depot handover, Harbor and all ten signal pages. It then walks the full R11,
enters the authentic gatehouse, uses Grand Marais care, crosses the complete
Gunflint road to Boundary Waters, saves/Continues there and walks home through
the other gatehouse lane to Duluth. The local lamp, spare part, demo starter
and survey-restoration flags remain false. No downstream progress or reserve
partner is inserted. Initial fixtures and full fresh-start balance remain
separate from this earned chapter.

The first battle attempts exposed weak test play and the missing clinic
registration. The course now selects Psychic/Fire against the visible mine
opponents, Thunderbolt against water Pokémon and Fire against Swinub,
Piloswine and Sneasel, with real clinic visits between gym trainers. Its bounded
gym retry uses native loss/care/Continue, never HP, PP, flag or RNG rewrites.

The existing full-bag keeper regression also passes on this exact pack:
refusal preserves the part and inventory, retry takes the part and gives one
Super Potion, and repetition gives no second reward. All **377 PCM hashes,
frame counts and loop ranges** pass. The production client and headless
location renderer build completes. Assets/Bevy all-target Clippy completes
with the four established allowances and existing warnings. Scoped rustfmt
passes; global formatting still first fails at the pre-existing
`crates/offset-allocator/src/lib.rs:408` difference.

Three actual stdio MCP replays pass the gatehouse/Gunflint round trip,
unqualified care/refusal, and optional part decline/accept/lamp repair/repeat.
Every replay opens the production Start menu, preserves money and badges,
and ends with byte-empty stderr. The replay now reads every offer page before
choosing and explicitly answers the nurse's native YES/NO prompt. Original
indexed room doors are observed first on their arrival tile, then stepped to
the public floor through ordinary input; no save or pack state is rewritten.

Whole town, native room, complete R11/R12 atlas and matching noon/down-facing
native cameras are retained under `renders/north-shore-story/`, including every
original, conversion, crop, rejected pass and difference. Source/native review
distinguishes animated water from static geometry, and native sprite priority
at the quay railing and gate counter. On the final renderer, the town gate,
Gunflint mouth, R11 gate and Surf harbor frames match source pixels exactly.
Earlier differing animation phases and their explicit native-glyph masks stay
archived; they were not replaced or discarded. The quay differs by 27 displayed pixels (three
source pixels) at the player's feet by the priority railing. The room differs
by 304 displayed pixels (19 source pixels) at the attendant's lower edge beside
the priority counter. Whole images and differences were inspected. No final
user visual approval is implied.

## Build and replay

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_north_shore_story -- \
  content-packs/minnesota-northstar-itasca-story.crystalpack \
  planning/minnesota/towns/MnGrandMarais.connections.json \
  content-packs/minnesota-northstar-north-shore-story.crystalpack
cargo test --locked --profile wasm-dev -p crystal-assets --test north_shore_story -- --ignored
export RUST_MIN_STACK=67108864
export CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-north-shore-story.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/north-shore-story-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib north_shore_story_controller -- --ignored
cargo test --locked --profile wasm-dev -p crystal-bevy --lib north_shore_journey_controller -- --ignored
cargo build --locked --profile wasm-dev -p geothite
cargo build --locked --profile wasm-dev -p geothite --bin geothite \
  -p crystal-bevy --example render_at_location -p crystal-assets --bin verify_crystalpack \
  --features crystal-bevy/location-tester
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-north-shore-story.crystalpack --audio --desktop
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_north_shore_story.py \
  --published content-packs/minnesota-northstar-north-shore-story.crystalpack --capture
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-north-shore-story.crystalpack
```

Actual stdio replays used these saved paths from the six-case controller run:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_north_shore_story_mcp.py \
  --mode gate --save .local/north-shore-story-review-saves/winona-shore-local-gate-east-85405-1793106371219813000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_north_shore_story_mcp.py \
  --mode part --save .local/north-shore-story-review-saves/winona-shore-local-part-declined-85405-1793106371808685000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_north_shore_story_mcp.py \
  --mode care --save .local/north-shore-story-review-saves/winona-shore-local-native-care-85405-1793106372553887000.crystalsave
```

The fullscreen command above is for the user; verification stays headless and
does not open a visible game. Full New Game progression and party balance,
Kettlurk runtime integration, Ely, the League, optional regional branches and
rail remain unfinished.
