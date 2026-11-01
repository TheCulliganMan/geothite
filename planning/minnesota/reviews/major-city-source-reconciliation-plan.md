# Tight cities: canonical reconciliation plan

Read-only plan: no shared world, interiors, canonical towns, transit, atlas or pack was changed. Completed native reference is final-v02, **660 maps**, SHA `052de08898bef175959d7beeab75409b949612d71266e59c86561221a8c137e6`. All selected candidate rows match it. Root owns final controller/visual acceptance and application.

**Preserve all seven CURRENT original Duluth room objects.** The frozen HarborWorkshop source predates its current Q07/MS10 annotations: it lacks `native_dialogue_source`, has a stale source-only status and calls the receiver service pending. Whole native room preservation is correct; copying the rich candidate wholesale would regress authoring metadata. The machine plan records exact conflicting fields/hashes.

The [machine plan](major-city-source-reconciliation-plan.json) contains every selected source path/hash, canonical destination, current top-level/room key format, six connection deltas, all original34 exterior warp coordinates,17 existing interior binding updates and117 new room reservations/pair bindings. Old final-v01 inventory remains separately retained.

Current full-file preimages:

- `world.json`: `5bb908fa0ec9360c75b0142d954dc374f7d7fecdfd177973a7f7e5ed47353fd0`
- `interiors.json`: `cbe9d61846f1d567c0ff93a2c4727fc5135414b60d91895e1aebb073e04d3a81`

These files changed concurrently after the first inventory. The six owned connection objects, all listed existing-room binding preimages and town field preimages were rechecked against fresh current files; complete owned record hashes are in `fresh_owned_preimage_guards`. Preserve fresh unowned metadata. The old epoch is historical, not an apply guard.

## Exact selected sources and schemas

| City | Frozen source | Original + new rooms | Original + new street facades |
|---|---|---:|---:|
|Minneapolis|`renders/minneapolis-expansion/tight64-v02/`|5 +39|5 +35|
|St Paul|`renders/st-paul/tight-20261027/v10/`|5 +37|5 +35|
|Duluth|`renders/duluth/expansion-64x48/v09-furnished-public-rooms/`|7 +41|7 +31|

Canonical destinations are `towns/<City>.json`, `towns/<City>.layout.json`, `towns/<City>.expansion.contract.json`. Copy approved layout/typed contract bytes. Merge the rich dossier while restoring every current original authored room object by ID. All17 original native room modules compare whole-module exact to the543-map pre-expansion parent; hashes are recorded. Never re-lower old gym/story programs from source approximations.

Minneapolis/Duluth rich `interiors` include originals and new rooms, and rich `doors` include originals and new facades. St Paul retains four inline `interiors`,37 `new_rooms`,35 `new_buildings`, and five original `doors`; its fifth original room is the external UnionDepot. Keep this rich schema and the canonical expansion reference; do not destructively flatten source ownership. Each typed contract `rooms`/`doors` array owns ONLY NEW content.

All old active/passive exterior indices and room target indices remain exact. Minneapolis/Duluth old ordinary exterior coordinates move by `[32,24]` core; St Paul uses explicit indexed relocations. Approved rich old doors already match final native `tile`, `return_tile` AND `arrival_tile`. The last two are exterior passive arrivals, not room arrival coordinates. Replacing rich doors with new-only contract doors would drop all17 original entrances/passives.

## World and interior record formats

`world.json` keeps its existing top-level object/`towns` array. For the three selected town IDs, set dimensions `[64,48]`, copy approved rich `ports`, add actual `compiled_tileset` and canonical `expansion_contract_file`. Keep original nested room reservation objects intact and append117new reservations: `id`, `kind`, `dimensions_metatiles`, `authoring_file`, `expansion_contract_file`, `source_room_id`. Existing original room dimensions already match native. Cities stay single whole modules; do not create city chunks or change IDs.

`interiors.json` remains `{schema_version,status,contracts}`. Select old records by `interior`; update only listed exterior binding/source-index fields. Existing interior arrivals/return carpet coordinates stay local and unchanged. Add missing Minneapolis source/index bindings from native. Append117new records with actual `parent`, `town`, canonical source references, dimensions, arrival/return indices and explicit `paired_bindings`. Upper floors have actual internal parents, not invented exterior doors. Total across the three cities: **134 rooms,118 street facades**.

St Paul's separate `transit/MnStPaulUnionDepot.json` needs only exterior binding documentation `[44,37]→[92,69]` and safe return `[44,38]→[92,70]`, under a fresh file guard. Preserve its entire original room, four carpets/passive5, courier/manifest programs and boarding restrictions. Train service remains unimplemented. Its public fair district is inside the expanded city; the northern annex stays closed by bodies28,2/29,2. Do not invent an external fair map/connection.

## Six actual walking connections

Offsets are signed32px metatiles; lanes are16px core tiles. The JSON includes original world records and exact proposed `direction_a/b`, `offset_a/b`, `a_edge_core/b_edge_core` values.

| City → neighbor | City lanes | Neighbor lanes | City / reciprocal offset |
|---|---|---|---|
|Minneapolis south →`MnR03C00R00`|64,95 /65,95|20,0 /21,0|+22 /−22|
|Minneapolis east →`MnR04C00R00`|127,48 /127,49|0,18 /0,19|+15 /−15|
|Minneapolis north →`MnR05C00R02`|64,0 /65,0|22,15 /23,15|+21 /−21|
|StPaul west →`MnR04C00R00`|0,48 /0,49|55,18 /55,19|+15 /−15|
|Duluth west →`MnR10C01R00`|0,48 /0,49|31,24 /31,25|+12 /−12|
|Duluth north →`MnR11C00R03`|64,0 /65,0|46,47 /47,47|+9 /−9|

The four Minneapolis/StPaul world records still describe planned `gatehouse_warp` geometry. Reconcile to native connections without inventing behavior: Minneapolis east has real `MnMPEastCheck`/`ENGINE_PLAINBADGE`; north has `MnMPNorthCheck`/`ENGINE_PLAINBADGE`+`ENGINE_FOGBADGE`. StPaul west and Minneapolis south have no city-side coordinate guard plane. Preserve upstream route checks separately. Duluth retains original Range/Shore roots/requirements/inbound-free semantics, four officers and eight indexed coordinate planes.

Legacy Duluth `connections.json` has old0/−7 offsets and old city patch coordinates2,11/2,12. Preserve its bootstrap/history; the current expanded binding must prefer the typed expansion contract. The expanded layout already contains the native mouth correction. Do not apply old city patch indices to64×48 terrain or mutate neighbor terrain/programs.

## Preview and historical atlas

A plain rich `residents` render drops real objects. Minneapolis omits four old wardens and eight discoveries, Duluth four old officers and six discoveries, and St Paul keeps14new ordinary/trainer/item actors outside its original resident list. Normalize the preserved native old object prefix plus new typed `residents`/`trainers`/`items`, deduplicated by identity. Expected native outdoor objects are28/23/27. Preserve event flags and programs; do not turn guards into dialogue-only residents.

Atlas can remain explicitly historical. Current StoryAtlas accepts only exact538/541 hashes/counts and unchanged city families, so660 is rejected. Its internal-room rule requires both ends `STAIRCASE`, while new valid stairs enter passive `FLOOR` through active `LADDER`/`STAIRCASE`. The legacy branch requires a street facade for every `interiors` entry and cannot represent upper floors/split StPaul rooms. A future certified final-epoch adapter should read exact contracts/native pairs and retain old strict branches.

City bounds/gallery heights already derive dimensions dynamically;32×24chunk strides are route-only. New Minneapolis atlas bounds `[142,757,206,805]` overlap R04 `[203,775,231,793]` by3metatiles. A separately reviewed diagram-only center shift x174→170 would clear it without changing gameplay. The660pack also contains14installed world-town modules including Lake Itasca, missing from the old roster; its gatehouse has a different prefix and actual paired warps. None of this requires blocking canonical city source reconciliation or claiming the old atlas is current.

Apply guarded source merges, update only owned records, and independently read back outside-owned bytes/full original room objects. No Cargo/export/native write/atlas mutation was performed for this plan.
