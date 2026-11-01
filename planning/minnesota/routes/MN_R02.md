# Prairie Farm — unchanged terrain boundary audit

The existing [whole route](../renders/prairie-farm/whole-route-43198d1d.png),
[eight player views](../renders/prairie-farm/viewpoints-43198d1d.png), native bank,
farmyard and workshop were inspected again. The farm/nursery composition, real
westward ledge, required pasture, protected bridge and two room visits remain
useful. The [widest tool-yard camera](../renders/prairie-farm/widest-tool-yard-43198d1d.png)
still frames its real entrance, keeper/sign approach and small habitat shoulder.
**No layout, actor, party, room, flag, art or extent changes were warranted.**

## Farmland reference review — 2026-10-26

The published pack and current source were reviewed against authentic
[Route34 terrain](../renders/source/Route34.png),
[Route39 terrain](../renders/prairie-farm/reference-Route39.png) and an actual
[native Route39 farm/pasture camera](../renders/prairie-farm/reference-native-Route39.png).
The latter was captured from the same supplied pack at `(10,10)`, noon, through
the existing production Rust renderer with headless image capture. Its fences,
clear farm entrances, building approaches and purposeful pasture provided the
comparison; no additional crop or livestock mechanic is implied for R02.

The weakest R02 camera was recaptured at `(52,36)` as
[native widest tool yard](../renders/prairie-farm/native-widest-tool-yard.png),
also at noon. It includes the real workshop door above, the outside trainer and
sign to the west, the small habitat shoulder and the forest/post boundary.
The approach reaches actual door `(52,33)` and safe return `(52,34)`; the northern
yard opening connects to the road. The farmhouse likewise has a clear front
approach, useful pasture/item choice and real door `(11,11)` with return
`(11,12)`. Nursery rows connect to the useful westward drop, while the longer
southern lane supplies the ordinary return. Required pasture still affects
the Mankato trip. These are existing destinations and movement choices in the
whole map, rather than a reason to fill the courtyard with extra props.

No concrete entrance, field-access, road or yard defect justified a terrain
revision in this pass. Extent remains **30×22**, and the immutable whole render
remains `43198d1d`. Re-running the existing published comparison checked all
**660 exterior metatiles**, seven outdoor objects, five signs and both room
contracts; equality passed. All three valid boundary mutations were rejected,
with the sealed channel still having zero reachable water/mounting pairs.
Source render/check measured335ms after pack load. No source geometry, actors,
parties, flags, rooms, art, source hashes or pack files changed, so no new native
publication or controller rerun is claimed. This audit is not final user visual
approval or earned regional progression.

The drainage visibly reaches the north/south map edges, but every water tile is
sealed from the player by actual native fences and bridge walls. This differs
from an accessible lake that needs a new closure. No water neighbor or Surf seam
is claimed, and no rocks were added merely to reduce an outer-WATER count.

The dedicated review tool now checks this actual boundary contract instead of
applying another route's blanket outer-WATER assertion:

- Full walking+Surf flood from the real Rochester gate reaches **zero of152
  WATER cells**. There are zero cardinal land→water mounting pairs anywhere,
  even when considering unreachable land. Every WATER cell is inaccessible.
- The eight outer WATER quadrants are exactly x=30–33 at y=0/43. None is in the
  reachable Surf component. Both bridge edges are WALL across x=28–35: north
  y=22 and south y=27. Bank fences seal the sides.
- Three malformed copies independently remove a north bridge rail, south rail
  or bank fence. Each creates a mounting opening and is rejected. Any future
  intentional water access must update this proof and close the real Surf edge.
- Exact comparison against the supplied published Prairie Farm pack passes:
  all660 exterior metatiles, seven outdoor objects, five signs, parties/text,
  both interior arrays, room borders and all return warps match the unchanged
  dossier. The whole-map source pixel hash remains `43198d1d`.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_prairie_farm.py \
  --pack content-packs/minnesota-prairie-farm.crystalpack \
  --verify-published --negative-checks
```

Source audit hashes:

- Layout: `e643c194444c9239e08674740bb694607753b95507e4c10f5a080e061ad971a6`
- Dossier: `06ba9e5290b5d7cc3cbd0591dacf6f0596e6a5b6031b53816abc25b0d1c23015`

Warm render/check measured136ms. Live8772 retains the full map, rooms and camera
sheet. This audit does not newly execute controller Surf input, earned regional
progression or final user visual approval. Existing native fixtures below remain
historical evidence for the same unchanged geometry; no new pack export is
required by an audit-only tool/runbook change.

---

# Existing compact native implementation

This hand-authored replacement is **30×22 metatiles**, one exterior with a farmhouse and tool shed. It replaces the repeated 96×72 nine-chunk blockout. The farmyards, nursery rows and straight fenced drainage channel have a different structure from Driftless's limestone bench or Central Lakes's meadow/lake fork.

[Whole route](../renders/prairie-farm/whole-route-43198d1d.png) · [eight player-scale views](../renders/prairie-farm/viewpoints.png) · [both interiors](../renders/prairie-farm/interiors.png) · [native west bank](../renders/prairie-farm/native-bank.png) · [native bridge](../renders/prairie-farm/native-bridge.png) · [native farmhouse](../renders/prairie-farm/native-farmyard.png) · [native workshop](../renders/prairie-farm/native-workshop.png).

## Decisions and physical contract

Coordinates below are 16px gameplay tiles; terrain strokes and dimensions use 32px metatiles.

| Location | Implemented behavior |
| --- | --- |
| Rochester arrival | East gate `(58,14)`, landing `(54,14)`. Walk west toward the nursery bank or take the longer southern lane. |
| Nursery bank | Native Kanto `58` occupies metatile x=23, y=7–8. It has WALL at core x=46 and HOP_LEFT at x=47, y=14–17. From `(48,15)`, Left enters the hop and lands `(45,15)`; Right from the landing is blocked. Stone closures at `(47,13)` and `(47,18)` prevent slipping around either end. |
| Nursery alternative | The westward drop leads through a short grass strip and worked-soil rows to the bridge approach `(40,24)`. On the eastbound return, the ledge cannot be climbed; take the longer southern lane through core y=26 and back north at x=52. No HM is needed. |
| Drainage bridge | The channel reaches both map boundaries. Its only dry crossing is the bridge at metatiles x=14–17, y=11–13. Both core deck rows y=24–25 work in both directions; the inner rail strips at y=23/26 are floor, with WALL at y=22/27. Native posts fence the banks above/below the crossing. Removing the whole bridge span disconnects the arrivals. |
| Lind Farm | Door `(11,11)` enters `MnR02Farmhouse`, a 5×5 room with farmer, field calendar and Berry. Both carpet exits return to a separate FLOOR warp `(11,12)`; walking across that floor does not enter the house. Poké Ball `(19,11)` is outside in the small pasture. |
| Tool shed | Door `(52,33)` enters `MnR02ToolShed`, a 5×4 workshop using the native traditional-house tileset: shelves, cabinets, brown floor and hanging journal. Keeper and Parlyz Heal are inside. Its adjacent carpet exits `(5,7)`/`(6,7)` return to safe exterior FLOOR `(52,34)`. |
| Southern seed spur | A short grass neck leads to Potion `(25,35)` on the west bank. It has no water entry or hidden HM requirement. |
| Mankato approach | Gate `(2,30)`, landing `(6,30)`. The pasture band core x=8–13, y=28–33 requires at least six grass steps in either direction, including if the nursery shortcut is used. |

Three optional trainers have local dialogue and native persistent defeat flags: Tess has Hoppip 7/Sunkern 7, Drew has Mareep 8, and Nell has Pidgey 7/Sentret 8. Two outdoor residents, five signs, two outdoor pickups and two indoor pickups fill specific short approaches. Reserved trainer flags 04–06 remain unused. The first-badge condition remains `BADGE_1`; neither resident completes Q02 or awards a badge. Real Rochester/Mankato ports and earned Q02 progression remain integration work.

## Art and review loop

The exterior uses **31 unchanged Kanto metatile varieties**. Nursery clear rows use brown native `2c` soil, encounter rows `0b`, gravel `31`, natural trail `01`, flowers `74`, round trees `0f`/`4c` and actual native posts. Kanto `00` has CUT_TREE collision despite its blank pixels and must not be used as walkable lawn. The blue farmhouse uses the source `38 39` roof over `3c 3d`; `02 03` beneath that roof doubles the roof line and was removed after native inspection. The tool shed uses the original `20 54 21` roof over `37 3a 7e`, with a dedicated workshop interior. No custom pixels, copied map geometry or duplicate source catalog is added.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_prairie_farm.py --serve
```

Open `http://127.0.0.1:8772/`. Edit [terrain strokes](MN_R02.layout.json) and [events/dialogue](MN_R02.json). The whole map, eight viewport crops, both rooms and anchor sheet refresh together without Cargo. A warm one-shot check/render measured 127ms. Invalid geometry is marked invalid. Publication is separate; after a new pack identity, regenerate saves through native fixtures instead of editing hashes.

Static checks cover mandatory pasture grass, all actor approaches, rooms, source collision and road polylines. Directed traversal proves the nursery shortcut works westward, its reverse requires the southern lane, and the bridge is the only dry channel crossing. Native publication comparison independently checks all 660 exterior metatiles, seven outdoor objects, five signs, teams/dialogue and both rooms/return targets. The preview verifier uses actual module width, including this 30-metatile-wide final chunk.

## Build and native checks

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_prairie_farm -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R02.layout.json \
  planning/minnesota/routes/MN_R02.json \
  content-packs/minnesota-prairie-farm.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_prairie_farm.py \
  --pack content-packs/minnesota-prairie-farm.crystalpack --verify-published
CRYSTAL_PRAIRIE_FARM_TEST_PACK="$PWD/content-packs/minnesota-prairie-farm.crystalpack" \
CRYSTAL_PRAIRIE_FARM_FIXTURE_SAVE="$PWD/.local/prairie-farm-review.crystalsave" \
CRYSTAL_PRAIRIE_FARM_JOURNEY_SAVE="$PWD/.local/prairie-farm-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --lib --profile wasm-dev --locked \
  -p crystal-bevy --features location-tester prairie_farm_ -- --ignored
CRYSTAL_PRAIRIE_FARM_TEST_PACK="$PWD/content-packs/minnesota-prairie-farm.crystalpack" \
CRYSTAL_PRAIRIE_FARM_JOURNEY_SAVE="$PWD/.local/prairie-farm-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p geothite --test prairie_farm -- --ignored
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-assets --test prairie_farm --test driftless --test central_lakes \
  --test headwaters --test league_approach --test gunflint --test north_shore
target/runtime-fps/wasm-dev/geothite play \
  content-packs/minnesota-prairie-farm.crystalpack \
  --load .local/prairie-farm-journey.crystalsave --compact
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-prairie-farm.crystalpack --audio --desktop
```

The isolated pack has **391 maps**, retaining original maps, assets, audio, HOME and recovery destinations. An in-memory composition with Driftless has 394 maps and preserves every prior map/file/audio program. Seven route asset regressions pass; the Prairie Farm regression also rejects a changed badge condition, a dry replacement for required grass, an ordinary-floor ledge and a landing inside its wall face. All 377 audio programs match their PCM hashes, frame counts and loop ranges.

The empty-party geometry fixture passes the real westward hop and rejected reverse, both bridge deck rows both ways, bank/rail barriers, all signs/residents/pickups, both room entries, all four indoor carpet exits and safe exterior returns. It tests dialogue input ownership. The separate populated-party journey uses a source-created level-12 Cyndaquil and level-9 Mareep, one badge and deterministic divider input as explicit fixture setup. Earned first-badge progression is not implied.

Native bank, bridge, farmhouse and workshop captures were inspected against the source-pixel map. A double-roof error was corrected and recaptured. Final visual approval, losing/recovery through Minnesota hubs, regional ports and an earned Q02 playthrough remain open. The full twenty-route objective remains open; this route's compact arrangement is not a template to duplicate across the region.


Two production-controller fixtures pass against the final pack. The populated
journey takes the native westward bank shortcut, rejects reverse climbing,
escapes two real wild encounters, crosses the bridge, walks through required
pasture grass in both directions and takes the long lane on return. Tess, Drew
and Nell are defeated through visible battle menus; all five opposing species
are observed and each battle consumes actual move PP. Cyndaquil grows from 12
to 13 and finishes at 14/33 HP, with Mareep 9 still at 28/28. Party HP/status/PP,
bag and all three defeat flags survive Continue. This is explicit fixture
setup, not an earned badge or regional journey.

Stdio MCP continues that final journey at `(54,14)`, moves to `(54,15)`, opens
and closes the production Start menu, and emits byte-for-byte empty stderr.
The production terminal client loads the same completed save and opens
Start/Pack before returning to movement and exiting cleanly.

Changed Rust files pass scoped rustfmt. Focused all-target Clippy passes with
`--no-deps` and the existing `never_loop` allowance, without warnings in the
changed route modules. Workspace `cargo fmt --all -- --check` still fails on
pre-existing formatting beginning at `crates/offset-allocator/src/lib.rs:408`;
unrelated existing Clippy warnings remain outside the map change.


The canonical external-pack regressions also pass unchanged: Mom/New Bark
input ownership and gate scene, production battle commands with executed turn,
MCP movement/Start and clean stdio. These guard the original game's controller
behavior; they do not prove Minnesota's missing regional story.
