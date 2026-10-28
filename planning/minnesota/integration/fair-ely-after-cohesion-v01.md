# Fairgrounds and Ely after the connected-state milestone

This is a read-only feasibility and additive-rebase plan. No terrain, publisher,
canonical source, catalog or pack was changed for this audit, and no Cargo or
exporter ran. The current98-outdoor image is actual native content. Fairgrounds
and Ely are the next integration gaps, not installed additions to that image.

The input is `content-packs/minnesota-state-cohesion-v01.crystalpack`, file SHA256
`c4816242295a236fdfe2a6a96fb248cd3eab11de1b526be0decb7df36e5e95eb`, content identity
`11a5f3b39ffba6252fb2246af17433ed70b6b4d722d0b3a67394a1526845b03a`,681 maps,
98 outdoors,251 species/251 moves. Its completed physical milestone receipt is
`renders/outdoor-connections/native-state-cohesion-v01/root-review-v01/acceptance.json`.
The full-state quality goal remains active.

## Implement Ely first

The separate verified Ely pack is
`content-packs/minnesota-northstar-ely-league.crystalpack`, file SHA256
`ab049e907b8712f5ee5f174b474c38fcc52460f272e60336aff17445d62cb6c4`, identity
`e96cd37fd94ca71f39d72a4a7924fe613ddb1cecf64f12f5a5985095a5fea7d4`.
It contains554 maps and adds10 maps to the544-map North Shore parent. Do not
replace the681-map branch with it or import its unrelated Marais gate/group80.

`build_ely_league_modpack` currently rejects this input solely through its exact
old `ab793e8d…` parent identity guard. A new explicitly guarded c481 acceptance
path can reuse the existing source builder. All ten original native template
modules used by Ely are whole-module equal between c481 and the old Ely pack:
Route23, GoldenrodPokecenter1F, CherrygroveMart, IlexForestAzaleaGate, KogasRoom,
BrunosRoom, WillsRoom, KarensRoom, LancesRoom and HallOfFame.

The existing R13 south mouth is already literal Floor at34/35,31. Append only:

| Map | Direction | Target | Offset | Native lanes |
| --- | --- | --- | --- | --- |
| MnR13C00R02 | south | MnEly | +5 |34/35,31→24/25,0|
| MnEly | north | MnR13C00R02 |−5|24/25,0→34/35,31|

The entire old Ely-pack R13 module, after removing only this south connection
and restoring its NORTH connection_flags, equals current c481 R13 exactly.
Current R13 whole-module JSON SHA256 is
`db1df63b6e16737f9538657427e7a88300962a85b9ed604df1f3a669128ac8dc`.
Publish this old map's connection attributes directly; do not rewrite old
scripts/NPC/event mirrors through a general replace helper.

Ely's proposed world rectangle is `[71,-339,91,-321)` metatiles,20×18, with zero
overlaps against all98 existing native footprints. It adds one outdoor and
nine existing native room plans: clinic, mart, Ilex registration gatehouse,
four League rooms, champion and record room. All eight facade warp records,
free public services, all-eight-badge plus restored-survey registration, actual
sequential five battles, withdrawal/blackout reset, native Hall of Fame and
post-credits recovery semantics must be retained. No upstream checkpoint is
removed. The expected first milestone is691 maps/99 outdoors.

Required sources, read at audit time:

| File | SHA256 |
| --- | --- |
| towns/MnEly.json |aca04118e1b4206de2a2c504e9862e8cee0a8d61f801971f6c57ee3b4165e912|
| towns/MnEly.layout.json |42916dee1f0daa6b899c6fe79891ab973a185437a71bf9ac4c85dd101dc1e784|
| story.json |05805d255d00df3c2e09217883a48da974d5b212fbf9f1e62b6b1a7baae08ead|

The current Rust CLI is `pack_ely_league BASE TOWN LAYOUT STORY OUTPUT`.
After the bounded c481 identity/preimage path exists, the concrete invocation is:

```sh
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_ely_league -- \
  content-packs/minnesota-state-cohesion-v01.crystalpack \
  planning/minnesota/towns/MnEly.json \
  planning/minnesota/towns/MnEly.layout.json planning/minnesota/story.json \
  content-packs/minnesota-state-cohesion-ely-v01.crystalpack
```

That command is intentionally not claimed working on681 before its old-parent
guard is extended. Reuse `ely_league.rs`, `ely_league_rooms.rs` and
`ely_league_scripts.rs`; rerun the five existing production cases and focused
preservation/Hall-of-Fame/blackout tests against the new output. Old map/room,
river, bridge, script, item, HOME and recovery records must remain exact except
the declared R13 connection and new Ely records.

## Fresh namespaces, never old branch numeric tables

Current c481 has no metadata or global map-group claims in80 or81. Group82 is
Mississippi and83 is cohesion. Reserve80 for the fair's four maps and81 for
Ely's ten, while checking the actual fresh input again. The old Ely branch also
contains `MnGrandMaraisShoreGate` in80; that unrelated map is not one of the ten
Ely modules and must not be copied into the fair reservation.

Current maximum event ID is2381. All29 new fair event names and both League event
names are absent. Reusing old League2310/2311 would collide with current
`EVENT_MN_MP_CLUB_CEDAR_BEATEN` and `EVENT_MN_MP_CLUB_COACH_INEZ_BEATEN`.
The old fair's2373..2381 also collide with the five river and four cohesion
trainer events. Allocate symbolically from the fresh current maximum:
Ely-first gives League2382/2383, then fair2384..2412. Do not overwrite, renumber
or alias any old event. `wMnLeagueRun` is absent from the current global symbol
namespace; retain that exact native persistent run-memory key and script rules.

Current highest recovery ID is277. Ely-first dynamically allocates outdoor278
and clinic279; the later fair outdoor recovery becomes280. Fair-first would
instead use278, then Ely279/280. The old runbook's279/280 are historical values,
not permission to overwrite current destinations. Existing builder logic finds
the actual clinic ID for `GEOTHITE_POST_CREDITS_SPAWN`; preserve that dynamic
binding. Temporary room extension spawns are removed exactly as in the existing
publisher. HOME0/all old records stay exact, and reserved28..255 remain invalid.

All22 fair trainer string identities and five League identities are absent.
Fair uses the existing named `trainer` protocol and needs no copied numeric
global trainer table. The League allocates within native classes; current next
IDs are COOLTRAINERF29, HIKER24, SAGE13, COOLTRAINERM34, CHAMPION2. Recheck after
any intervening trainer overlay and enforce the native identifier range.
All32 `MART_MnFair…` and `MART_MN_ELY` must be fresh; no old mart is replaced.

No seven-bit landmark slots are free. The actual old publications already use
existing fallback landmarks consistently in attributes AND map_to_landmark:
all four fair maps→`LANDMARK_MN_TWIN_CITIES`114, all ten Ely maps→
`LANDMARK_MN_LEAGUE_APPROACH`100. Keep these explicit bindings on the new pack;
do not add IDs128+ or copy a wholesale landmark table.

## The actual latest fair and its catalog dependency

The source authority is foods-v2:
`content-packs/minnesota-state-fair-foods-v2.crystalpack`, file SHA256
`bf688621307c6e4805296fa0412b67777327899c507512432200569d0b5f7ee8`, identity
`2c45b49dbac2b7f291a6614e7b455e4c679b8ea8748e388b7d9aa8d6c41b3b61`.
Its664 maps come from the separate660-map R04 lineage;474 species/270 moves
are not the681-map parent catalog. Preserve the85 actual outdoor NPCs,
32 merchants/39 staffed positions,22 trainers, four patrols,29 grass species,
three interiors,16 purchasable foods/beer and all stamp/prize/full-bag/consumption
semantics. Existing St Paul indoor fair rooms remain separate real content.

c481 lacks31 required regional species:29 grass species plus POUCHID and SODCALF
used by trainers. Full evolution closure is35 species across these15 families:
`MN_F001, MN_F005, MN_F010, MN_F016, MN_F020, MN_F032, MN_F033, MN_F038,
MN_F042, MN_F051, MN_F083, MN_F090, MN_F098, MN_F101, MN_F105`.
The four additional closure members are SLIPPERINA, CYPRIELLE, RUMBLEHIDE and
THUNDRAZE. Native PIDGEY/RATTATA/SENTRET trainer partners already exist.

Use `MinnesotaPokemonCatalog::load/build`, not copied Pokemon/learnset/asset
catalogs from bf688. IDs are authoritative `251+MN_number`, so these selected
new species do not collide with current1..251. The complete active source
catalog has101 families/223 species. All101 registered native `line.review.png`
receipts currently exist and match their accepted source manifest hashes;
that is an inventory check, not a replacement for Rust's full source/2bpp/palette
validation. Catalog::load validates all active family art even when selecting
one family, so the full external native directory is still required.

| Source file under pokemon/ | SHA256 |
| --- | --- |
| roster.json |27d41154b2e0e731bdb8fc4d402a8784a9cd3faee2b52bb592896028dc43b1e3|
| roster-curation.json |ed636627f70768b940049b52d6172f0fdb9284ab934d1038c0675b8c4294febc|
| species-profiles.json |6949b25e1e8e67ae0cdd7df2a85c615170458e1718fd9a6be9590dafe0fd5333|
| sprite-generation.json |2c70e143dbda21756379546b6720857f4ac50fa91b78e7323d5525850e5d53cb|
| palettes.json |534616622ff8500d8ee36699b5fa6a753a2dd8023a8d5da371ad46a7e41f2087|

Native reviewed inputs are at
`target/minnesota-pokemon/native/MN_F…/`; sprite-generation.json supplies the
canonical manifest and source-art references, including all retained reviews.

The documented Rust pipeline starts with `pack_fairy_type types`, builds the
regional catalog, installs13 Northwoods moves, six Fairy moves and their native
animations. A minimal35-species variant can sequentially call the existing
`--family` option for the15 families, or a focused Rust wrapper can load once
and perform those same fifteen additive builds in memory. Regional learners
absent from a partial catalog are explicitly skipped by both move builders.
Concrete Root-owned commands, after the Ely rebase above exists and with new
ignored output paths, are:

```sh
export CARGO_TARGET_DIR=target/runtime-fps
set -e
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_fairy_type --bin pack_minnesota_pokemon \
  --bin pack_northwoods_moves --bin pack_northwoods_animations

target/runtime-fps/wasm-dev/pack_fairy_type types \
  content-packs/minnesota-state-cohesion-ely-v01.crystalpack \
  content-packs/minnesota-fair-ely-deps-types-v01.crystalpack
cohesion_catalog_stage=content-packs/minnesota-fair-ely-deps-types-v01.crystalpack
for cohesion_family in MN_F001 MN_F005 MN_F010 MN_F016 MN_F020 MN_F032 MN_F033 \
  MN_F038 MN_F042 MN_F051 MN_F083 MN_F090 MN_F098 MN_F101 MN_F105; do
  cohesion_next_stage="content-packs/minnesota-fair-ely-deps-${cohesion_family}-v01.crystalpack"
  target/runtime-fps/wasm-dev/pack_minnesota_pokemon "$cohesion_catalog_stage" \
    planning/minnesota/pokemon target/minnesota-pokemon/native \
    planning/minnesota/pokemon/species-profiles.json "$cohesion_next_stage" \
    --family "$cohesion_family"
  cohesion_catalog_stage="$cohesion_next_stage"
done
target/runtime-fps/wasm-dev/pack_northwoods_moves "$cohesion_catalog_stage" \
  content-packs/minnesota-fair-ely-deps-moves-v01.crystalpack
target/runtime-fps/wasm-dev/pack_fairy_type moves \
  content-packs/minnesota-fair-ely-deps-moves-v01.crystalpack \
  content-packs/minnesota-fair-ely-deps-fairy-v01.crystalpack \
  planning/minnesota/pokemon/fairy-moves.json
target/runtime-fps/wasm-dev/pack_northwoods_animations \
  content-packs/minnesota-fair-ely-deps-fairy-v01.crystalpack \
  content-packs/minnesota-fair-ely-deps-final-v01.crystalpack
```

These use existing CLI signatures; this audit executed none of them. Use `set -e` for the loop, and stop the
sequence on any failure; freeze/recheck inputs and require fresh output names
before Root executes it. A focused Rust wrapper can avoid15 repeated catalog
loads. The already-documented aggregate build in `playable/state-fairgrounds.md`
and `docs/northwoods-fairy.md` is also available if Root selects the complete223
species rather than the minimal35-species closure. Neither path copies a branch
catalog or deletes current maps.

The Fairy stage is an explicit mechanics change: it retypes13 returning native
species and CHARM/SWEET_KISS/MOONLIGHT, changes the named Fairy/Steel chart rows,
and installs modern move categories. It is NOT an unchanged-old-catalog claim.
Record and test those declared deltas separately from map preservation. Do not
silently strip Fairy from the selected species to bypass dependencies. Source
moves, native animation donors and retained native assets should produce the
intended fair parties; compare the compiled22 parties and all food facts to the
accepted foods-v2 definitions rather than assume branch equality.

## Why the old northern fair bind cannot be replayed

Current St Paul is40×34, native kanto, at world origin[68,-6]. Its original
AgricultureHall active facade is core18,9/warp15. The old64×48 fair binder paints
meta x8,y0..7 and hardcodes north/south offset0. Replaying that patch would damage
an occupied compact neighborhood. Do not change that hall, its passive return,
any existing facade, quest, service, checkpoint or interior.

A geometrically aligned compact north bind would use city28/29,0→fair16/17,71,
offsets+6/−6. Native city meta14,0 is13/WALL; meta14,1 is31/Floor. Its two
existing officers28/29,2 are pure dialogue placeholders saying the north annex
is shut; they impose no badge/event check. However the direct48×36 Fair rectangle
`[74,-42,122,-6)` overlaps three preserved river maps. This route is rejected,
not an invitation to hide overlap or move established rivers.

## Eastern approach: feasible, not authored

I personally viewed current native St Paul, both LowerWoods middle chunks and
the accepted fair whole map. A48×36 Fair at origin[172,8], rectangle
`[172,8,220,44)`, has zero overlaps against all98 current map rectangles.
Other tested y origins−16/0/16/24 atx172 also have no overlaps;8 gives a simple
middle-chunk west approach and preserves its existing woodland context.

Use the existing StPaulEast↔LowerWoodsC00R01 pair, offset+14/−14. A safe new
walking opening can extend existing city Floor75,40/41 across its outer pine
shoulder to79,40/41. Exact candidate city preimages are meta38,20=0f and
39,20=13, both Wall; no house/door/actor occupies these proposed cells. The
new aligned river-side arrival is C00R01 core0,12/13. This changes no existing
Surf mouth, gate program or city connection offset.

The initial course hypothesis is:

1. Wind from that western forest edge toward C00R01 core6/7. A north–south foot
   bridge crosses actual water aroundy22..29; its underpass axis is east–west.
   Land south of the river, then use the forest shoulder toward the existing
   C00→C01 seam aroundy32/33. Both chunks already share reciprocal offset0.
2. Approach C01R01's real grass/trainer meadow atcore34..41,y32..39. Preserve
   its resident, original encounter grass and rewards. A short woodland course
   should use that bank, not a new empty rectangular plaza or puzzle quota.
3. An east–west foot bridge crosses the actual main stem aroundcore44..55,
   y32/33; its underpass axis is north–south. Reach the existing eastern forest
   edge at63,32/33 and add the actual reciprocal FairWest pair, offset0.
4. FairWest core0,32/33 joins its existing Floor4..9,32/33 by opening only the
   two forest metas0,16 and1,16 (both native05). There is no actor/warp on this
   short approach; trainer7,34 and nearby merchant positions remain unchanged.
   Close the former unconnected southern opening16/17,71 visibly (native
   meta8,35 currently01/Floor) while retaining its safe existing recovery/start
   footing16,68. No original room return or activity needs to move.

These are proposed courses, not finished bridge footprints or a controller
proof. The retained1,269-step Surf itinerary moves LEFT through C00's first
crossing zone, then turns DOWN atx2; keep that turn outside the proposed bridge
aperture. It moves UP atC01 x51,y35..30 through the second crossing zone. Thus
the proposed water axes agree with the actual itinerary, but exact rail/shore
quads and both approach mouths still require native collision/axis validation
and replay of the complete existing journey. Preserve lower/upper player planes,
NPC interactions and current native overlay/OAM behavior.

A roughly two-chunk wooded trip with two real crossings can give useful river
views and the existing meadow encounter. Do not add a68-metatile isolated gap
route or force the fair into the north overlap. Keep northern St Paul annex
closure intact; bounded wayfinding text can later direct visitors east. No fake
warp bridge, path through a house or invented city hub is proposed.

If these two additions need no extra connector map, the eventual composition
would be695 maps/100 outdoors. That is a forecast, not the current pack count.

## Publication and acceptance contract to author next

Current native whole-module JSON preimages (sorted compact UTF-8 JSON):

| Map | SHA256 |
| --- | --- |
| MnStPaul |77d99c9b83c7db4d7dc786f12858a5b68bc13f0a457f056216db130ff0dbcd17|
| MnMississippiLowerWoodsC00R01 |b0e01763b72aa22dd1622127f73b4bb947e419bc4707eafb7e27d1997372cc16|
| MnMississippiLowerWoodsC01R01 |96408d9ab2ecfc85e75ceb99fb5b62b5958f4e0c6182827097ea2f3034d28b47|
| Fair accepted bf688 module |c5103efa4e7b9436606369c0c5ea55e5f6c58836d585fa0490dde9395e49b7b0|

Fair source files are `zones/state-fairgrounds.json` SHA256
`b401f0e9b0857bdd4f5dd74271b4b4d28e7697306d256ffb53fe9df26659c4e0`
and `zones/state-fairgrounds.layout.json` SHA256
`4f214f6ee8585e720fe1a63c55665219ec9bcf7b43e362b276545fe209ad925c`.
New art recipes are the existing11 native block imports. Original Johto PNG/
palette/background-bank bytes and all original fair room template modules are
exact between c481 and bf688. Preserve native source pixels and all85 actors.

Current `pack_state_fair BASE LAYOUT DOSSIER OUTPUT` remains guarded to old64×48
St Paul/x8 patch/offset0 and cannot publish this east approach as written. Extend
its bounded connection recipe or add a focused additive publisher, rather than
loosening the old binder to accept arbitrary extents/edits. Keep the old64 mode
available. Recompile only the proposed ingress/approach layouts after review;
copy no unrelated bf688 map/catalog/global table. All current681 old maps stay
exact except the declared R13 connection, future two city pine cells, two real
woodland courses/bridge overlays and necessary connection attributes. Every
remaining old raw program, actor, item/flag, recovery record, room, audio byte
and source art file stays exact apart from separately declared catalog rules.

Rerun: strict written preservation/negative guards and uncached/native equality;
all five existing Ely cases and its actual ceremony/loss recovery; all current
cohesion and original river cases; old St Paul services/quests/interiors; all
fair merchant/food/beer/crowd/patrol/22-trainer/habitat/stamp/prize/full-bag cases;
both walking directions and all new bridge mouths/planes/Continue; fresh native
whole/camera pixel comparisons plus the complete one-scale outdoor embedding.
Do not claim the existing foods-v2 tests prove the uninstalled new approach.
