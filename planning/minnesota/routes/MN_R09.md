# Native cliff stairs revision, 2026-10-29

The current selected Iron Range West revision repairs fragmented rock ribs and uses native cave12/30/36 ledge stairs for ordinary two-way terrace access. A separate mine-apron ledge gives a dry return after a grassy climb. See [current acceptance](../revisions/iron-cliffs-v01/README.md). All older results below are historical.

# Current installed quarry grass revision, 2026-10-28

[Selected current-parent revision](../revisions/route-grass-v01/current-r09-v01/README.md) adds8grass metatiles and4attached spoil boulders to the704-map current story/R03 cave child. The quarry circuit requires4grass steps uphill and has a dry ledge/ore-track return. Five native cases,13clean stdio calls and377audio programs pass. North Shore stays ice-free. Older isolated pack results below remain historical.

# Iron Range West — current native geography revision

## Route grass pass — 2026-10-28

Accepted in isolated `minnesota-route-grass-r09-v01.crystalpack`, identity prefix
`4c5fbe211a9f`. Eight new grass metatiles join useful quarry paths. The upper
quarry circuit requires four grass steps; its original ledge and lower ore track
keep a dry return. Four attached spoil boulders close actual cliff-foot bypasses.
Native geometry, five wild encounters, Cut, battle/PP, healing, Continue, stdio
and exact terrain/audio preservation passed. See
[R09 review](../revisions/route-grass-v01/R09.md) for scope and evidence.
The shared game is unchanged; older native reports below are historical.

The current **471-map** `content-packs/minnesota-geography-review.crystalpack`
(`f9b04e67f2bf…`) publishes this terrain revision. Root inspected the complete
map, weak source cameras and fresh native 2D captures. Fresh geometry, populated battle/PP/rest/Continue and native Cut
cases pass. All unrelated pack fields, original art/events/rooms/story and
audio remain exact. See the [current native proof and replay commands](../playable/geography-revisions.md).
The source-candidate notes and earlier standalone-pack results below are retained
as authoring history; final user visual approval and earned regional travel remain open.

---

# Rockier Iron Range source candidate — 2026-10-26

The user explicitly requested a much rockier Iron Range. The current authoring is a **SOURCE CANDIDATE**, with the old published/native/controller evidence below applying to the retained pre-revision terrain. Root owns visual selection, native terrain revision and fresh controller captures. No pack or Rust was changed by this source pass.

[Whole candidate0a507546](../renders/iron-range/whole-route-0a507546.png) · [same north-bench160×144 camera](../renders/iron-range/rocky-widest-north-bench-native160-9c53a571f6.png) · [mine apron160×144](../renders/iron-range/rocky-mine-apron-native160-e8da42d884.png) · [haul track160×144](../renders/iron-range/rocky-ore-haul-track-native160-2f814ee1f0.png) · [rail rock-cutting160×144](../renders/iron-range/rocky-rail-rock-cutting-native160-acbc4711f7.png). These are unmodified native-source pixels in player framing, not production-controller captures. Whole and enlarged versions are retained in the same render folder.

The 64×36 extent, four clipped chunks, world endpoints and every trainer, resident, pickup, sign, party, event flag, door, room row, encounter table, rail block and field-move anchor remain unchanged. The rockier composition uses existing Johto cliff68–73, stone71 and grouped erratic0a source art; `tileset`, `compiled_tileset`, `derived_blocks` and `source_pixels_modified` are unchanged. There are no new source-art recipes or runtime feature promises.

## What changed by sight

The first pass added broad pristine crowns above the route; I rejected those as empty tan platforms. The next pass narrowed them into stepped ribs and capped the broadest remnants with attached broken ore. The final pass brought rubble to the actual mine apron and haul-track camera, rather than relying on geology visible only in the whole-map overview. Native Route45 and the original Johto atlas were inspected as the rock/cliff reference; the prior published rail capture and the same existing weak-camera frame were also inspected.

- The west creek now has a bent granite spine and stepped spring crown, while its original timber bridge remains the sole required crossing.
- The northern excavation is framed by separate ore seams and a broken headwall. The north walking bench keeps its geologist and rim challenge; its two attached ore-cap groups replace the former small pine finger without filling the lane.
- The mine apron has one connected rubble toe and a two-walking-tile exposed-stone path. Stone continues along the quarry haul track, with attached sampling rocks and the toe of the southern ore seam visible in the player frame.
- Three low southern ribs and the rail-side rock cutting replace portions of the former continuous pine mass. The rail keeper's small turnout still has an approach and a view of the original static crossing. These are scenery headwalls, not unbound new walking destinations, fake doors or implied train service.

All new geography is an explicit orthogonal stroke or a named, hand-placed connected erratic group. The complete coordinates are in `MN_R09.layout.json`; no random placement or copied base-map block array is involved. Main changed metatile extents are:

| Terrain feature | Bounding region / exact walking treatment |
| --- | --- |
| West creek spine and spring crown | x7–17,y5–18; bent native cliff ribs stay off the bridge, west route and Cut camp. |
| Northern ore seams/headwall | x20–41,y4–10; stepped outlines and grouped0a caps; existing rim stays intact. |
| North rim ore cap | Half-open `[24,8,26,9)` plus attached points `(24,7)`, `(28,8)`, `(29,8)`, `(29,7)`. The geologist remains core `(55,19)`. |
| Mine-toe finger and apron rubble | Rim points `(21,17)`, `(22,17)`, `(23,17)`; apron points `(19,22)`, `(20,22)`, `(21,22)`, `(21,23)`. No mine trigger or passive return moves. |
| Exposed-stone walking track | Half-open meta `[18,21,26,22)`, `[25,22,26,24)`, `[26,23,44,24)`, existing native71/FLOOR. The final x44 join remains ordinary lawn for the existing medic spur. |
| Southern ore seams | x20–42,y24–28; stepped narrow ribs; haul toe `(30,24)`, `(31,24)`, `(32,24)`, `(31,25)` frames the walking camera. |
| Rail cutting and siding crown | x43–54,y5–12; the actual track at y15/x43–55 and walking rail `(50,15)` are unchanged. |

## Source compatibility and remaining verification

Pre-edit authoring is retained at `.local/r09-rocky-before.layout.json` and `.local/r09-rocky-before.json`. A read-only comparison confirms that all four installed R09 chunks in `content-packs/minnesota-quality-review.crystalpack` exactly match that previous layout. This identifies the legitimate `BEFORE_LAYOUT` input for Root's terrain-only revision helper. The candidate changes251 metatiles, with no new art, map, event or encounter identity. That number describes the handoff, not visual quality.

The full source checker and all nine existing adversarial mutations pass:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_iron_range.py --negative-checks
```

The original pre-Cut endpoint walk remains160 ordinary movement steps. The optional spoil ledge's ordinary return changes127→129 steps while retaining its exact ramp, hop, landing, reverse-wall and side-barrier coordinates. It remains a useful one-way return. The original source Cut targets stay sealed before the actual63→3f override, including the walking-plus-Surf test; the recovery spur stays free; required grass and the battle-versus-habitat rail choice still have their original roles. No story or field-move prerequisite was relaxed.

The source loop measured roughly1.5–1.7 seconds per full archived render/check on this pass. All full-resolution originals, native crops, converted images, enlarged previews and rejected passes remain saved through `render_archive`/`render_stages`. Rejected whole passes `a6a68412`, `790202c7` and `f6572d61` are retained. The known current limits remain: source collision is not native input proof, Q06 chief/cache/depot completion is not implemented by this terrain pass, the static rail is not Glacier Line runtime, and no final user visual approval is claimed.

---

# Current native publication and controller-fixture status

Root published `content-packs/minnesota-iron-range.crystalpack` by running
`pack_iron_range` over `minnesota-league-approach.crystalpack`. The actual overlay
contains **410 maps**. Exact comparison passes five authored maps, 2,304 exterior
metatiles and 15 outdoor objects; source art and source negative checks pass.
Root inspected the actual [native rail-crossing capture](../renders/iron-range/native-rail-crossing.png).
Current integration audio evidence is the shared **443-map** composition: all
**377 audio programs** match their PCM hashes, frame counts and loop ranges.
No duplicate route-specific audio run is inferred from this shared result.

All three current native asset regressions **PASS**, including named and
schema-guarded terrain mutations and the gate-specific numeric Surf bypass.
The production-controller leaf compiles. Root's actual run passed
`iron_range_populated_fixture_spends_surf_pp_wins_rests_and_continues`: selected
Surf PP, the real Rafe victory/flag, Continue, actual medic HP/PP restoration and
another Continue were verified. That local fixture supplies party, badges,
position and divider initially; it proves no earned Q06 or regional access.
The geometry and shared Cut cases failed in that first run. Bounded fixes for
native room-arrival suppression and the shared Cut planner's grass target are
ready; their passing rerun remains pending. No failed test is counted as PASS.
Root owns warm registration and execution.

Register it in `crystal-bevy/src/bevy_shell/tests.rs` as:

```rust
#[path = "tests/iron_range.rs"]
mod iron_range;
```

Use `CRYSTAL_IRON_RANGE_TEST_PACK` for the actual published pack. The two ignored
fixture tests are:

- `iron_range_controller_tours_clipped_seams_rails_drop_rooms_and_open_rewards`
- `iron_range_populated_fixture_spends_surf_pp_wins_rests_and_continues`

The empty-party geometry fixture drives only `VisibleShellController::press`
through `RouteGrid` and native indoor walking. It covers three actually walkable
internal seams in both directions: x63/64,y24; x14,y47/48; and x88,y47/48.
The lower east-west adjacency has **no legal walking pair** at y48–71 and is
asserted closed rather than reported as a fourth walkable seam. Both bridge
walking rows, actual83 rail walls/84 crossing columns, native ramp/drop/reverse,
seven open signs, all three ordinary residents, three open pickups, Scout,
mine entry and separate passive return are covered. The gated camp notice and
Revive are excluded before Cut. No trainer battle is claimed from an empty party.

The separate populated fixture begins beside Rafe at `(98,38)` with explicit
source-compatible Feraligatr48/SURF and Ampharos46/THUNDERPUNCH, a20HP initial
injury to observe real healing, seven badges and a divider replay
set once. The published overlay does not register `EVENT_MN_HEADWATERS_REPORT`;
the fixture does not invent that quest flag or claim to enter through its
unimplemented regional gate. Every subsequent game action uses production input; there are no
post-setup party, position, flag, map, clock or divider rewrites. It must observe
Geodude/Skarmory/Machop, spend selected Surf PP, win, Continue that spent PP and
victory flag, reach the native field medic, restore actual HP/PP via `HealParty`,
and Continue the healed party/bag/flag. This proves no earned regional journey.
Optional fixture/journey saves use `CRYSTAL_IRON_RANGE_FIXTURE_SAVE` and
`CRYSTAL_IRON_RANGE_JOURNEY_SAVE` respectively.

Root's existing shared `cut_gated_routes` case can use the following exact
native contract: prefix `MnR09`, two columns/two rows, tileset `mn_iron_range`,
approach `(12,56)`, tree `(12,57)`, inside `(12,58)` on native FLOOR, source63→3f,
reward `(11,63)` REVIVE, flag `EVENT_MN_R09_ITEM_04`. The source camp notice at
`(14,62)` also belongs after Cut. Native denial/removal/traversal/reset and
persistence remain with that shared controller test.

---

# Frozen source design and implementation contract

The authored **64×36-metatile** route replaces the 96×72 district lattice with
four clipped exterior maps, a spruce creek approach, one continuous low quarry
rim around a flooded excavation, a mine cache, an abandoned rail siding and a
field-medic turnout. The source-art revision has been visually inspected and
reviewed by the integration owner. It is not final visual approval or native
gameplay evidence.

[Whole route](../renders/iron-range/whole-route-0aa98d41.png) ·
[player views](../renders/iron-range/viewpoints-0aa98d41.png) ·
[widest north bench](../renders/iron-range/viewport-widest_north_bench.png) ·
[rail inspection turnout](../renders/iron-range/viewport-rail_inspection_turnout.png) ·
[mine interior](../renders/iron-range/room-MnR09Cave1F.png) ·
[Cut before](../renders/iron-range/cut-before-0aa98d41.png) ·
[Cut after](../renders/iron-range/cut-after-0aa98d41.png).

## Geometry and event contract

Coordinates in this table are global 16px gameplay tiles unless marked meta.
The upper chunks are 32×24 metatiles; the lower chunks are 32×12. Native seams
are core x=63/64 and y=47/48. Event bodies and A approaches stay within their
actual source chunk.

| Feature | Exact implementation contract |
| --- | --- |
| Bemidji approach | West gate meta `(1,25)`, inward landing `(3,25)`, on `MnR09C00R01`. An intentional west-edge opening joins the road; the gate is off the native chunk corner. |
| Grand Rapids approach | East gate meta `(62,12)`, inward landing `(60,12)`, on `MnR09C01R00`. The final pine lane reaches the east edge. |
| Entry story conditions | Preserve `BADGE_7` and `EVENT_MN_HEADWATERS_REPORT`, quest `Q06`. The raw headwaters reading does not substitute for the completed report. Native gate scripts remain integration work. |
| Spruce neck | Required grass core `[14,46,18,52)` costs at least four grass steps. The western lookout and sample-camp spur do not bypass it. The complete endpoint path remains available before Cut, without Surf: 160 ordinary movement steps excluding battles. |
| Timber creek bridge | Meta x=12–14, north rail y=21 block `82`, south rail y=22 block `81`; inner core rows y=43/44 walk, outer rows y=42/45 block. Actual WATER remains below at meta y=23. Removing these crossing cells disconnects the two route ends. |
| Ore shelf | Hyper Potion `(39,29)` lies in a reclaimed ore-grass pocket. Mina `(41,37)` challenges the lower rim. The ordinary ascent is native ramp `74` at meta `(24,20)`. The north bench is a narrow stepped ribbon rather than the rejected broad tan platform. |
| North inspection bend | Geologist `(55,19)` gives a concrete Q06 mine-cache direction. Orin `(61,23)` faces the lake-side bend; the native camera shows the water. Their width serves actual interactions. |
| Spoil return | Native `57` at meta `(40,20)`: approach `(81,39)`, hop `(81,40)`, landing `(81,42)`, reverse WALL `(81,41)`. Native rock shoulders at meta `(39,20)` and `(41,20)` close side bypasses. The ordinary return takes 127 steps through ramp core `[48,40,50,42)`; a hop saves 124 steps. Surf is an additional field action, not part of this ordinary walking proof. |
| Quarry shoreline | Source `41` fences protect the lower excavation-facing trail at meta x=27–31,y=20. The wider lake bank supports Nell, habitat and a readable view. The creek and excavation are enclosed: all outer map edges contain zero WATER tiles. Internal water seams must remain reciprocal native connections. |
| Mine entrance | Native `73` at meta `(20,20)` provides CAVE `(40,41)`. Entry warp 1 targets `MnR09Cave1F` entrance 1; passive exterior return record 2 uses FLOOR `(40,42)`. Never return onto the active CAVE trigger. |
| Mine room | `MnR09Cave1F`, **12×11**, template `UnionCave1F`, exact hexadecimal rows in dossier. Native LADDER exit `(13,19)`, safe arrival `(13,18)`, same-mouth pairing `(40,41)`. Scout `(19,7)`, native foreman `(9,5)`/approach `(9,6)`, native cache `(7,5)`/approach `(7,4)`. Occupied bodies are included in source reachability. |
| Rail siding | Exact native Johto Modern `7e`→`83` is all WALL; `7f`→`84` is all FLOOR. The alignment occupies meta x=43–55,y=15, with one walking crossing at `(50,15)` and a blocking native rock at `(56,15)`. Fences and forest close the surrounding corridor. Removing the crossing disconnects endpoints. These are static rails; no train or rail controller is promised. |
| Rail inspection turnout | Keeper `(95,27)` and sign `(96,26)` stand in a short bay beside the actual rail. The former long empty northern lane now curves through the pine habitat toward the exit. |
| Challenge or habitat | Rafe `(98,37)` faces east. Actual terrain stops sight after `(99,37)`, `(100,37)`, `(101,37)`. The grass alternate core `[104,34,108,44)` requires at least four grass steps; no free dry bypass exists around that sight line. |
| Field medic | Resident `(91,53)`, approached from `(90,53)`, declares the existing native `heal_party` service. A short pine-sheltered flower turnout makes the lower eastern chunk useful after quarry battles. Native restoration and dialogue ownership remain to be verified. |
| Optional Cut supply | Native tree `63` at meta `(6,28)`, CUT_TREE `(12,57)`, approach `(12,56)`. Use actual source override `63→3f`; require party CUT and native Hive Badge permission (`ENGINE_HIVEBADGE`, badge index 1). Revive approach `(12,63)` and sample-notice approach `(14,63)` are unreachable before Cut even with Surf, and reachable afterward. It opens 87 walking tiles. No shortcut is claimed; `shortcut:null`. |

The first rim review was rejected for broad empty brown ground. The revision
narrows the exposed walking band, steps the cliff silhouette, uses native pine
cut-ins, adds the ore inspection interaction and shortens the rail alignment
from 21 to 13 metatiles. The widest bench and rail turnout were inspected at
actual 160×144 player framing, enlarged with nearest-neighbor pixels.

## Actors, rewards and Q06

Eight exterior trainers retain their original IDs, defeat flags and exact
parties. `MN_R09_T01`–`T08` are Arden, Walt, Mina, Tess, Nell, Rafe, Orin and
June. Their 24 opposing Pokémon and all text/sight coordinates are explicit in
the dossier. Trainer 09/10 reservations retain their original IDs, flags and
parties under `reserved_trainers`; neither is placed by this revision.

The four original pickup identities persist: `ITEM_01` Hyper Potion `(39,29)`,
`ITEM_02` Ultra Ball `(119,19)`, `ITEM_03` Max Repel `(7,39)` and `ITEM_04`
Revive `(11,63)`. All are walking rewards; the last requires Cut. Residents are
`MN_R09_ORE_GEOLOGIST`, `MN_R09_RAIL_KEEPER`, and
`MN_R09_QUARRY_GUIDE` (the field medic). Eight signs have exact source WALL
quadrants and reachable approaches.

The mine's `MN_R09_DEPOT_FOREMAN` now has a native, talk-triggered Q06 binding in
`crates/crystal-assets/src/iron_range_story.rs`. It uses `SPRITE_ROCKET`, class
`GRUNTM`, and canonical trainer **`MN_LODE_FOREMAN`: Graveler35, Machoke35,
Magneton36**. The conflicting older route-draft party is superseded. Badge7 and
the completed headwaters report are required; there is no automatic sight
battle. Actual victory sets only `EVENT_MN_DEPOT_FOREMAN_DEFEATED`.

The separate `MN_R09_SURVEY_CACHE` requires that win. Its ordinary yes/no menu
lets the player leave the meter for later. YES still requires acknowledging the
recovery text before setting `EVENT_MN_MINE_INSTRUMENT_RECOVERED`. The meter
and original test log are a persisted quest payload, not an invented bag item.
Repeating either interaction cannot replay the battle, reward or recovery.
The foreman admits seeing the warning report and orders another copper-nickel
test anyway; losing the battle does not redeem him.

MS08, MS08_CACHE and MS09 are distinct. The native Grand Rapids handover requires
both defeat and recovery before it can set `EVENT_MN_DEPOT_CLEARED` and open R10.
The mine never grants depot clearance. Normal `pack_iron_range` publication now
includes the binding; `pack_iron_range_story` can add it to an existing mine
without replacing route terrain, services, items, encounters, art or audio.
`grand_rapids_modpack.rs` now binds the technician and ordinary town services;
`grand_rapids_connections.rs` binds the guarded reciprocal R09/town/R10 seams.
Actual handover, east-road release, inbound care and Continue pass through
production input in the current 502-map story/city candidate. Upstream badges,
report and initial evidence state are explicit fixtures; the earned upstream
report, onward Duluth link and full regional journey remain open.

## Fast source loop and integration

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_iron_range.py --negative-checks
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_iron_range.py --serve --port 8791
```

The tool consumes the supplied ignored `content-packs/text-tui.crystalpack`,
reuses source art/render helpers, and writes only ignored review images. Warm
full render/checks measured approximately 0.4–0.8 seconds; no pack export or
Cargo build is needed for an edit. It never copies game source catalogs or
publishes a pack.

Positive checks cover actors/parties/text, all four used chunks, the actual
room/return, closed Surf boundary, compulsory grass, the sole bridge and rail
crossing, battle-versus-grass cost, real Cut targets and the directional ledge.
Nine in-memory negative mutations fail: Cut side entry, outer Surf hole,
removed grass, extra rail gap, sealed crossing, painted ledge, free dry trainer
bypass, wrong mine exit and a diagonal polygon closing edge. Both proposed rail
imports are byte-for-byte equal to their native source blocks.

Root owns native publication and shared registration. Required work includes
`mn_iron_range` inheriting the existing Headwaters `80`–`82` and Johto field
move behavior, importing native rail `83`/`84` through
`iron_range_art::build_iron_range_art_pack`, accepting the exact layout/dossier
in typed Rust, replacing the old nine-chunk world reservations/ports, updating
`interiors.json`, and implementing Q06 story bindings. Keep original source
maps/audio/HOME/recovery data intact.

A source preview proves neither native playability nor final visual acceptance.
Native evidence still needs matching 2D captures, all real map seams and room
returns, denied/successful Cut and persistence, rail collision and grass
encounters, trainer victory/PP/flags, pickups, `HealParty`, foreman loss/win/cache
transactions, depot delivery, Continue and clean stdio input through
`VisibleShellController`.

## Native builder leaf handoff

The frozen source/layout remains unchanged. Native lowering now has a focused
[iron_range_modpack.rs](../../../crates/crystal-assets/src/iron_range_modpack.rs)
entry point, `build_iron_range_modpack`, and
[iron_range.rs](../../../crates/crystal-assets/tests/iron_range.rs) asset regressions.
The leaf delegates to the existing shared authoring compiler; it contains no
second terrain generator, content catalog or frontend dispatcher. Shared
registration, compilation, publication and native input tests belong to root.
The author ran pinned rustfmt on both leaf files, without a competing compile
or pack export. Root reports that the initial asset suite passed in the warm build; the
generic negative mutations are now named and guarded against schema-only
rejection, so that negative test must be rerun. A further
focused Surf-bypass regression now requires a gate-specific rejection rather
than accepting any unrelated build failure; its execution remains with root.
Native publication/controller evidence is still pending.

Exact shared registration contracts:

| Shared table | Entry |
| --- | --- |
| Dossier parser | `MN_R09`: `[64,36]`, two columns, two rows, eight trainers, three residents, four pickups, eight signs, one door, one room, prefix `MnR09` |
| Required physical contracts | Exact flags `BADGE_7` + `EVENT_MN_HEADWATERS_REPORT`; one required habitat, one battle/grass choice, one ledge, one optional Cut area; no `shortcut` |
| Layout parser | `MN_R09` → `[64,36]`, `mn_iron_range`, `johto_8_neighbors`; schema2 and `reject_cliff_cuts:true`; highest native block `84` |
| RouteSpec | group54, `MINNESOTA_IRON_RANGE`, manifest `minnesota-iron-range`, `LANDMARK_MN_IRON_RANGE`, label `MN_IRON_RANGE`, name `IRON RANGE WEST` |
| Source art branch | Existing `build_iron_range_art_pack`; unchanged Johto/headwaters `80–82`, then exact Johto Modern `7e→83` WALL and `7f→84` FLOOR |
| Gear/music metadata | Root registered position `[120,32]`, native music source `Route45` |
| Native map IDs | Four clipped exteriors in row-major order IDs1–4; `MnR09Cave1F` ID5, all group54 |

The asset regressions compose R09 onto the current Central Lakes overlay and
compare every base map, source runtime file, tileset, trainer, encounter table,
map metadata, Pokémon/item/mart catalog, audio payload/manifest and HOME/recovery
spawn. They verify clipped dimensions, reciprocal internal connections, exact
serialized modules, original parties/flags, actual rail/grass/ledge/Cut collision
and the field medic's compiled native `HealParty` command. That command check
is not evidence that party HP was restored through production input.

The mine explicitly needs **two exterior warp records**: active CAVE entry1 at
`(40,41)` and passive FLOOR return2 at `(40,42)`. Both target interior entrance1;
the native LADDER `(13,19)` returns to exterior record2. The test rejects missing
return2, return onto the active cave tile, wrong ladder quadrant and wrong-mouth
pairing. It also rejects changed story prerequisites, paved mandatory habitat,
walking/Surf Cut bypasses, a flattened ledge, sealed rail crossing and a false
ledge-return saving. The earlier terrain-only baseline contained the mine shell
and Scout; current initial publication also includes the separate Q06 binding
described above.

Earlier terrain-only source hashes, retained as authoring history:

- Layout: `91a9c6497a5b0739c9b8f37cf92e051f01a1235d029b8aba31e874a70f39ffdd`
- Dossier: `ec7179e835b46441c8d5f6b1bc1b39ea1693f01b7fc991d879d5e016c321a57c`

Root's next validation is the registered `crystal-assets --test iron_range`
regression, followed by exact published-source comparison, preservation/audio
verification, headless native captures and production-controller traversal,
Cut denial/removal/reset, battle/PP, field-medic restoration and mine return.
No source preview or asset test substitutes for those input/runtime checks.


The focused Surf-negative mutation paints native WATER at meta x=5, y=28–29.
Source collision analysis independently confirms: Cut approach `(12,56)` stays
walkable, target `(12,63)` stays unreachable on foot, and walking+Surf reaches
that target behind the intact tree. The native regression requires an error
naming `MN_R09_SAMPLE_CAMP_CUT` and `Surf`, so an unrelated failure cannot be
reported as gate enforcement. General native reachable-water support belongs
to root's shared compiler work. Workspace formatting was checked and still
fails on pre-existing formatting beginning in `crates/offset-allocator/src/lib.rs:408`;
both new Rust leafs pass pinned scoped rustfmt.

Root's subsequent warm native run reports all three R09 asset regressions PASS,
including the corrected named/schema-guarded mutations and the specific Surf
landing bypass rejection. This supersedes the earlier pending-negative note.
The geometry/shared Cut rerun remains pending; the separate populated battle/rest/Continue fixture subsequently passed as recorded at the top.
