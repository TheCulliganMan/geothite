# Current Crystal-reference grass climbs, 2026-10-28

R05 now has33additional native grass cells along useful cliff climbs and
approaches:76 to208walking tiles of habitat. Upper/lower climbs require7/6grass
steps, with dry ledge returns in both bands. Current R01/R20 cave and League
work is preserved. All new review is2D only; North Shore stays ice-free.

Eleven native route cases,11production stdio calls, exact source/native
preservation and all377audio checks pass. See [the selected receipt and explicit
setup limits](../revisions/route-grass-v01/compact-rebase-v01/R05.md).
R06 onward and the full statewide grass objective remain open.

## Historical source and review notes

The retained notes below describe earlier source/pack identities and party
roster templates. The current receipt above supersedes their acceptance bindings.

# Crystal-reference grass and dry ledge returns, 2026-10-28

R05 now follows Crystal Route45's cliff-lane pacing: encounter grass on the
western climbs, dry southbound ledge returns on the eastern lanes. Habitat
increases from 76 to 208 walking tiles. The ordinary upper climb requires seven
grass steps and the lower six, with zero in those bands on the ledge returns.
The original two-ended cave remains a separate shortcut and healing stop.

The named review pack `minnesota-route-grass-r05-v01.crystalpack` passed exact
preservation, native cameras, walking/ledge/cave geometry, six wild encounters,
five trainer wins, native medicine/healing, Save/Continue, Surf fixtures and all
377 audio checks. See [the current runbook and explicit fixture
limits](../revisions/route-grass-v01/R05.md). No default pack is replaced.

Earlier quality-review milestones below are historical.

# River Bluffs North: ridge route review

The current hand-authored **32×56-metatile** route spans three native maps and
one **8×10** rock cut. It replaces the repeated ten-map reserve with two
limestone climbs, two one-way returns, a cave passage between ridges, a
river-facing lookout and a protected fishing pier.

[Whole route](../renders/river-bluffs/whole-route-a988740b.png) ·
[player-scale source views](../renders/river-bluffs/viewpoints.png) ·
[one-way paths](../renders/river-bluffs/ledges.png) ·
[native upper drop](../renders/river-bluffs/native-upper-drop.png) ·
[native lower switchback](../renders/river-bluffs/native-middle-switchback.png) ·
[native lower cave mouth](../renders/river-bluffs/native-lower-mouth.png) ·
[native cave](../renders/river-bluffs/native-rock-cut.png) ·
[native lookout](../renders/river-bluffs/native-lookout.png) ·
[native pier](../renders/river-bluffs/native-pier.png).

## Current upper-ridge revision

The current **471-map** `minnesota-quality-review.crystalpack` contains a
hand-authored northern cleft in the upper ridge. Its former core `(23,21)`
camera was almost entirely bare tan plateau. The inset between metatile x9
and11, ending at y11, brings actual native cliff faces and connected pines
beside the ascent. The reward knoll still joins the ascent below the cleft.
No actor, ramp, ledge, mouth, water barrier, event or party moves.

[Before/after cameras](../renders/river-bluffs/cleft-review/before-after-a988740b.png)
and [actual native ascent](../renders/river-bluffs/cleft-review/native-ascent.png)
were inspected by sight. Exactly13 metatiles change; the complete pack's other
fields remain exact, apart from R08's separately documented22-block peninsula
and the derived content identity. All three current R05 controller cases pass:
three-map traversal/both drops/four ramps/both cave mouths, the populated
five-trainer journey with medicine/keeper/resources/Continue, and actual Surf
barriers/both reciprocal water seams. All377 audio checks pass.

The current [terrain-revision runbook](../playable/terrain-quality-revisions.md)
records the exact native pack, saved source snapshots, preservation proof and
commands. Older standalone commands below remain useful build recipes; their
old captures cover unchanged locations. Full regional progression and final
visual approval remain open.

## Traversal contract

Coordinates below use global 16px gameplay tiles. The first two exterior maps
are 32×24 metatiles; the last is 32×8. Seams lie at core y=48 and y=96.

| Feature | Authored behavior |
| --- | --- |
| Minneapolis trailhead | South gate meta `(11,54)`, inward landing `(11,52)`. Warden `(19,103)` explains the climb. The shared habitat core `[20,98,26,102]` requires four grass steps; all optional caves and returns lie beyond it. |
| Southern shelf | Lena `(11,85)`, flowers and Super Potion `(9,89)` make the short western clearing useful before the climbs. It is not a healing service. |
| Lower climb | Native ramps at core x=13/y=76–77 and x=29/y=52–53 join the west switchback and northern saddle. Owen `(31,65)` stands beside the sunny grass shelf, with room to pass. |
| Lower drop | Approach `(37,75)`, hop at `(37,76)`, landing `(37,78)`. Reverse y=77 is WALL; both sides close. The ordinary return takes 91 walking steps through the western ramp passage `[12,76,14,78]`, versus three tiles of hop displacement. |
| Two-ended rock cut | Lower mouth `(16,77)` enters room ladder 1 at `(5,17)`; upper mouth `(30,27)` enters ladder 2 at `(11,5)`. Each room exit targets its own exterior's passive return record 2 at `(16,78)` or `(30,28)`. The cave genuinely crosses between ridges. The guide restores the team through native `HealParty`; Repel `(5,5)` rewards the upper pocket. |
| Upper climb | Native ramps at core x=15/y=40–41 and x=25/y=14–15 join the west ascent and St Cloud path. Iris `(19,33)`, Wes `(33,29)` and Theo `(29,9)` are optional challenges. The lookout spur also crosses its native ramp at `(37,26–27)`. |
| Upper drop | Approach `(37,39)`, hop at `(37,40)`, landing `(37,42)`. Reverse y=41 and side edges are WALL. Its ordinary uphill return takes 71 steps through `[14,40,16,42]`. |
| Mississippi lookout | The old tree-enclosed bay did not show the river in the native camera. A narrow spur now reaches a fenced turnout at meta `[24,8,28,12]`, with sign `(50,18)` and resident `(53,21)`. Native north, south and east fences protect the water-facing edges while their inner strips remain walkable. Great Ball `(37,19)` remains on the western approach. |
| Landing garden | Ether `(41,81)` rewards the lower ledge's eastern landing before rejoining the southern paths. |
| Fishing pier | Native wooden rails frame a six-metatile spur at row 44, with both core deck rows y=88–89 usable. Upper y=87 and lower y=90 are walkable inner strips; y=86/91 block water access. The eastern bollard wall at `(61,89)` closes the end. Fisher `(57,89)` leaves the second walking row clear. |
| St Cloud approach | North gate meta `(12,1)`, landing `(12,3)`; the north sign `(26,4)` names both cities. Actual hub travel is still unimplemented. |

The ledge distances describe ordinary walking geometry. Trainer sight and
occupied bodies are excluded from safe fixture paths; the cave is a separate
warp shortcut, not an alternate ordinary ascent. Five trainers use seven
source-created opposing Pokémon: Lena Geodude 21/Machop 22, Owen Pidgeotto 22,
Iris Skiploom 22/Flaaffy 22, Theo Vulpix 23, Wes Girafarig 23. Reserved trainer
flags 06–08 are unused. Four pickup flags include the cave Repel.

## Story and implementation boundary

The dossier preserves both `BADGE_3` and `BADGE_4` and Q03. The warden's text
does not enforce those conditions. Real Minneapolis/St Cloud gates, the earned
Q03 journey, regional recovery and visual acceptance remain open. The local
pack retains the original source HOME and recovery destinations.

The Rust publisher adds four maps to the supplied 388-map source pack. Its
`mn_headwaters` art retains all 128 original Johto metatiles and appends the
three existing source-pier recipes; original graphics, palette and water
animation remain unchanged. The current layout uses **32 metatile varieties**. The lower bluff has an explicitly drawn green meadow shelf and flower pocket, contrasting with the exposed limestone above.

A resident may explicitly declare the native `heal_party` service; ordinary residents retain their previous dialogue. The shared interior schema now supports explicit `exit_doors` for a cave with
different exterior mouths. Existing rooms keep their previous behavior when
the list is absent. Ambiguous/duplicate ladders, swapped pairings and incorrect
entrance indices are rejected. Resident facing and healing commands are lowered
through the native object/runtime parsers, with a regression requiring both
compiled commands. The composition regression preserves every
Twin Cities/source map, runtime file, tileset, spawn and audio program.

## Fast review and native verification

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_river_bluffs.py --serve
```

Open `http://127.0.0.1:8775/`. Edit [terrain](MN_R05.layout.json) or
[events](MN_R05.json). A warm source render/check measured 229.5ms and the live
page refreshed in 385ms. Source iteration needs no pack export or Cargo build.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_river_bluffs -- \
  content-packs/text-tui.crystalpack planning/minnesota/routes/MN_R05.layout.json \
  planning/minnesota/routes/MN_R05.json content-packs/minnesota-river-bluffs.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_river_bluffs.py \
  --pack content-packs/minnesota-river-bluffs.crystalpack --verify-published
CRYSTAL_RIVER_BLUFFS_TEST_PACK="$PWD/content-packs/minnesota-river-bluffs.crystalpack" \
CRYSTAL_RIVER_BLUFFS_FIXTURE_SAVE="$PWD/.local/river-bluffs-review.crystalsave" \
CRYSTAL_RIVER_BLUFFS_JOURNEY_SAVE="$PWD/.local/river-bluffs-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --lib --profile wasm-dev --locked \
  -p crystal-bevy river_bluffs_ -- --ignored --nocapture
CRYSTAL_RIVER_BLUFFS_TEST_PACK="$PWD/content-packs/minnesota-river-bluffs.crystalpack" \
CRYSTAL_RIVER_BLUFFS_JOURNEY_SAVE="$PWD/.local/river-bluffs-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p geothite --test river_bluffs -- --ignored
target/runtime-fps/wasm-dev/geothite play content-packs/minnesota-river-bluffs.crystalpack \
  --load .local/river-bluffs-journey.crystalsave --compact
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-river-bluffs.crystalpack --audio --desktop
```

Verified so far: all 1,792 exterior metatiles, eleven outdoor objects, six signs,
trainer parties/text and paired cave/return events match the current native
pack; the cave's 80 metatiles match too. Eleven route/town asset regressions pass after the shared event compiler change. Native geometry checks cover both
seams, ramps, two drops and reverse denial, ordinary uphill returns, both cave
directions, all residents/signs/pickups, pier barriers, input ownership and
Start. The revised lookout fences pass geometry. The populated journey passes with an explicit source-created Croconaw 26, Flaaffy 25 and Golbat 25 plus four badge bits: three wild escapes, five trainer wins/seven opponents with actual PP spent, Super Potion collection and use through the production Pack, both cave directions, both ledges and uphill returns, and Continue preserving party/bag/flags. Two actual keeper rests restore HP/PP/status; the second revives fainted partners. This fixture does not establish earned regional progression. All 377 audio PCM hashes, frame counts and loop ranges pass.

Eight native views were inspected. The latest lookout capture replaces its
previous tree-obstructed view. This is an individual revision, not final
acceptance of the route or the twenty-route regional objective.

## Surf boundary correction

Original Johto blue-water rock blocks `31/32/36/39/3a` visibly close the north, east and south river perimeter. All 348 outer core tiles were audited; 126 are blocking BUOY quadrants and none are WATER. Conservative walking-plus-Surf reaches 822 water tiles and zero unconnected outside cells. Matching WATER remains across internal y=47/48 (nine pairs) and y=95/96 (seven pairs); the native pack must retain reciprocal offset-zero north/south connections. Source geometry does not substitute for a native Surf input test. [Actual native pier/boundary capture](../renders/river-bluffs/native-pier-boundary.png).

The production Surf regression now passes too: native A/Yes mounting from the pier, actual BUOY rejection at the eastern water rocks, and both reciprocal water seams crossed in each direction. The refreshed full journey save also passes stdio MCP movement/Start with empty stderr; the actual production TUI loads it, opens Start/Pack and exits cleanly.
