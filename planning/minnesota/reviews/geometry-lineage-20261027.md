# Geometry and story lineage audit — 2026-10-27

The 538-map story branch and completed 510-map geometry branch are compatible only through a bounded merge. The independent written-pack merge proof now **passes172 checks**: exactly nine modules and148 terrain metatiles change; all529 other modules, all eight newer gyms,28 new town/room maps, story programs, caps, connections, catalogs, recovery and audio remain exact. No source or pack was written by this audit worker, and no Cargo/export/controller run is claimed here.

| Epoch | Actual pack | File SHA256 |
|---|---|---|
| Completed geometry | `minnesota-northstar-rock-stairs-review.crystalpack` | `6071912eeb2fde6982f9e857bed9157d8d86da0f7ae56dba38665c809ab100e1` |
| Newer story | `minnesota-northstar-lake-country.crystalpack` | `da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50` |
| Root's bounded merge | `minnesota-northstar-map-quality.crystalpack` | `9aa950a35453f06cfee2e4a9b674c6d902af7c311f9ca64a083665ac6c60ddb2` |

All files are completed ignored packs under `content-packs/`. The [three-way inventory JSON](geometry-lineage-20261027-6071-da27.json), SHA256 `6f51f4d59caebfd46c4dcf95b231791301614d007db7d05073c25b11eb33a28d`, records every route/cave extent, actors, exact changed cells, cached/split payload checks, art changes, connections, and64 active authoring/contract input hashes. Those pack and source hashes remained stable throughout that audit. The [independent merge proof](geometry-merge-20261027-da27-9aa9.json), SHA256 `c6f921f08a053b651cc46040f9f2329e897ea1c18271be5d3f6c19df2a33bbe0`, records every declared changed native cell and all172 actual checks.

## Missing improvements recovered without replacing story

| Area missing fromda27 | Exact owned modules | Restored cells |
|---|---|---:|
| R03 cave bank refinement | `MnR03Cave1F`8×6 |18|
| R05 rock-cut cave refinement | `MnR05RockCut`8×10 |45|
| R10 tighter gallery | `MnR10Cave1F`18×14 |16|
| R17 tor cave refinement | `MnR17Cave1F`7×8 |21|
| R06 northern bends | `MnR06C00R00`32×24 |28|
| R16 farm bends/hay bank | `MnR16C00R00`32×24; `MnR16C01R00`16×24 |18|
| R10 outdoor stairs | `MnR10C00R00`32×20; `MnR10C01R00`16×20 |2|

R10's east module changes only its tileset attribute; its terrain is already identical. The merge therefore changes nine module records but only eight block payload values. Both R10 exterior cached/split attributes use `mn_r10_rockstairs`. Exactly five alias runtime files and two manifests are added, byte-identical to the completed6071 art; all shared Johto and all other existing runtime files remain exact. Runtime ID derives exactly those two manifest suffixes, and the content identity is recalculated. Ordinary REPEL changes200→100; every other item field and item remains exact.

All nine current caves match6071 geometry after the merge: R01, R03, R05, R09, R10, R12, R13, R17 and R20. R06/R16/R10 exterior geometry also matches6071 except the six retained R06 forest caps below. The cave actors17 and route exterior actors260 have identical native records between the original two branches. All57 route exterior chunks and all nine caves have coherent block payloads, cached/split attributes, NPC mirrors and per-map script mirrors in both branches.

## Story geometry that must win

The R03 south-bank caps are source-declared in [river-city-links.json](../playable/river-city-links.json). Preserve `MnR03C00R01` local metatiles `(7,7),(8,7),(10,7),(11,7),(13,7),(18,7)` as05 forest; global y31. The older6071 values are02/02/02/02/46/44. The entire R03 exterior remains exactda27 in9aa9.

The six R06 caps are source-declared in [lake-country-story.json](../playable/lake-country-story.json): north chunk `(20,0),(22,0)` and south chunk `(6,19),(7,19),(9,19),(10,19)`, all02→05. The merge retains all six while restoring the northern bends. R06's new Brainerd/StCloud connections, offsets, flags and programs remain exactda27. Brainerd's caps `(16,0),(0,12),(31,12)` and its complete two-page ranger/report/Surf-gift state also remain exact.

All538 connection vectors and connection flags are unchanged. There are19 reciprocal town/route pairs with exactly two native FLOOR lanes each, compared with seven in6071. The JSON records their actual coordinates and offsets; this does not certify controller traversal or the chapter's badge/text/input conditions by itself.

All eight active canonical gym row sets matchda27, and none match the older6071 row sets. Their newer dimensions are Rochester7×10, Mankato8×10, Minneapolis5×9, StPaul7×10, StCloud9×11, Brainerd5×8, Bemidji6×10 and Duluth7×10. Preserve their complete newer modules, trainer/puzzle/reward programs and all40 changed native gym runtime art files. The merge proves all eight whole gym modules and original files remain exactda27. No old gym or party is restored from6071.

## Six newly published towns and their sources

The538 branch adds six towns and22 rooms,28 maps total. Five older review-gym IDs become bound through these towns, so the six town dossiers reference27 rooms altogether. WinonaHome2F is reached through the authored internal stair; StPaul's depot has its own source. These are inventory counts, not a claim that every room is paired through an exterior facade.

| Town | Active source pair under `towns/` | Native extent | Referenced rooms / newly added rooms | Actual connection authority |
|---|---|---|---|---|
| Winona | `MnWinona.json`, `MnWinona.layout.json` |16×14|5 /5|[rochester-links.json](../playable/rochester-links.json): R01|
| Rochester | `MnRochester.json`, `MnRochester.layout.json` |20×16|5 /4|rochester-links: R01; [prairie-story.json](../playable/prairie-story.json): R02|
| Mankato | `MnMankato.json`, `MnMankato.layout.json` |24×18|3 /2|prairie-story: R02; river-city-links: R03|
| Minneapolis | `MnMinneapolis.json`, `MnMinneapolis.layout.json` |32×24|5 /4|river-city-links: R03; [courier-story.json](../playable/courier-story.json): R04; lake-country-story: R05|
| StPaul | `MnStPaul.json`, `MnStPaul.layout.json` |32×24|5 /4|R04 binding in `st_paul_connections.rs`; [transit/MnStPaulUnionDepot.json](../transit/MnStPaulUnionDepot.json) adds the fifth room outside its four-room dossier list|
| StCloud | `MnStCloud.json`, `MnStCloud.layout.json` |24×18|4 /3|`st_cloud_connections.rs`: R05 south offset0/R06 north+4; lake-country-story owns the Wind checkpoint|

Their source/layout hashes, all source/native mouth-cell deltas, room IDs, actual native connections, two FLOOR lanes and actor records are in the JSON. Winona/Rochester/Mankato/Minneapolis raw layouts intentionally retain forest-mouth preimages at respectively1/2/2/3 cells; native chapter overlays open them. Do not overwrite these mouths with unadjusted source rasters. StPaul/StCloud raw source blocks match their published exteriors. No original town actor anchor conflicts were found for these six dossiers.

## Active source conflicts to keep separate

Eight live trainer display-name edits have not reached either native branch. Actors' IDs, positions, sprites, parties and defeated flags still agree; only names differ:

| Actor | Active authoring | Both native packs |
|---|---|---|
| `MN_R07_T01` |SIVREN|MARA|
| `MN_R08_T01` |NOREK|SOREN|
| `MN_R08_T04` |AVRIN|ADA|
| `MN_R08_T06` |SIVREN|MARA|
| `MN_R11_T03` |SIVREN|MARA|
| `MN_R13_T01` |AVRIN|ADA|
| `MN_R14_T01` |SIVREN|MARA|
| `MN_R17_T05` |NOREK|SOREN|

The geometry merge preserves the native records. Renaming is a separate authored/native correction; do not silently rewrite authoring, bypass a source/native guard, or replace a party to make names agree. R10's five city-seam layout deltas remain known installed overlays, not missing geometry. All other active exterior geometry matches6071, apart from those five R10 seam cells; all active cave rows match6071.

This audit does not establish new GPU acceptance or controller/Continue/save compatibility on9aa9. Root owns those checks and the larger twenty-route quality goal. The next R19/R02/R03 passage review will be a separate bounded proposal before authoring.
