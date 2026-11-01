# R09 story mine — frozen source review

Final candidate: `MnR09Cave1F.candidate.json`, SHA256 `243f04da49a01f59e6fe8fb004049fd942c3acac1e1e6a5ad6c865f89b650d43`. This is a **source-only** terrain proposal. Active `routes/MN_R09.json`, native packs, Rust publication and story programs have not been changed by this review.

Frozen evidence: `../renders/caves/r09-candidate/243f04da49a01f59/`. Start with `whole-before-after.png`, `decision-cameras.png`, `mouth_offset_stair-4x.png` and `actual_weakest_floor-4x.png`. Every original, rejected pass, conversion and crop is retained. `../renders/caves/r09-candidate/baseline-c16-1793069600708277000/` holds the full original route epoch, exact original room and native nonterrain field hashes. The supplied external pack retains its programs; no raw program dump is introduced.

The original12×11 room had three broad rectangular floor bands, straight necks and isolated boulder clusters. The candidate keeps12×11, but bends the entrance around an attached rock shoulder, separates the foreman bench from the scout work bay with a fitted stone rib, and gives the foreman bench a short downward return beside an ordinary stair. The working drift passes around that rib. These deliberately cut benches suit a mine; no spring, healer, extra room, new item or regional mechanic is added. The final one-cell change from827c60a1… replaces metatile6,8 cave2f with native0a, shortening the bulky entrance boulder face without opening a stair bypass. That earlier complete pass remains immutable.

All original fields except `rows` remain exactly equal to `MnR09Cave1F.previous.json`, SHA256 `8553987f7da3cfaa3e8bdb3a680686c9a32f9c869b8615bf01e87782b0f0e088`. Only nonruntime `review_contract` metadata is added. Exact retained anchors are:

| Contract | Core coordinate |
|---|---|
| Sole mouth LADDER / clear arrival |13,19 /13,18|
| Foreman / talk approach |9,5 /9,6|
| Separate survey cache / approach |7,5 /7,4|
| Original scientist scout |19,7|

The foreman remains `MN_LODE_FOREMAN`, GRUNTM: GRAVELER35, MACHOKE35, MAGNETON36. Native full party moves, DVs, AI and battle bindings are preserved by not lowering/replacing the programs. His native prerequisite checks remain `ENGINE_GLACIERBADGE` and `EVENT_MN_HEADWATERS_REPORT`; victory sets only the existing `EVENT_MN_DEPOT_FOREMAN_DEFEATED`. The separate cache requires that defeat, offers its original yes/no with retryable decline, and records `EVENT_MN_MINE_INSTRUMENT_RECOVERED`. It gives no ordinary bag item and does not perform Grand Rapids' later handover or set `EVENT_MN_DEPOT_CLEARED`.

Native cave30 stairs are at metatiles4,3;8,5;5,7. Its quadrants are FLOOR/FLOOR/FLOOR/WALL; the scout stair is entered round its corner at core17,10 because16,9 is WALL. The single cave39 drop has two real HOP_DOWN lanes10,8 and11,8, landing10,10 and11,10 across blocking lower quadrants10,9 and11,9. Side caps9,9 and12,9 remain WALL. Ordinary returns to the two upper approach cells take9 and11 steps respectively, with all three actor bodies blocked. Both ledge lanes and the full ordinary floor graph must be exercised later through production input; these static checks do not establish runtime behavior.

Fast rerender and source checks:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/caves/MnR09Cave1F.review.py --self-test
```

The review reads actual cave art/collision from `content-packs/minnesota-northstar-map-refinements.crystalpack`, SHA256 `c16af00c82167c809620b956981ee80641ddc9fd1eeaf1e2c91f59ab16a9cc7d`. It verifies all132 original native blocks before rendering, preserves forced NITE and uses authentic NPC sprites. The complete loop took1.789 seconds. All ordinary Floor remains connected with actors blocked; all three interactions and the sole ladder remain reachable; outer edge closed; no new unpaired entrance, water or pit. The widest empty all-Floor square is3core tiles; its actual camera is11,4 on the foreman workbench. Twelve corruptions are rejected, including moved actors/exit, altered party/cache flag, blocked approaches, broken lip, opened side/border and extra ladder/actor. See `source-review.json`.

The renderer's temporary display roster includes the three story actors only to draw their authentic sprites. Authoritative JSON keeps story trainers/objects in their original categories; no native event or item lowering runs and no generic story guard is weakened. Source player views are clamped160×144 native-art composites, not production captures. The whole, seven decision views and weakest frame were personally inspected; the remaining upper rectangular bench provides passing space for two critical interactions.

Native acceptance remains pending: narrow story-preserving terrain API, exact cached/global event and program certification, native paired-mouth traversal, all stair lanes, actual hop/reverse/ordinary return, prerequisite denial and original battle loss/victory/repeat, cache decline/acknowledgment/repeat, Save/Continue and matching headless captures. Brainerd independently confirmed a85's cached mine module equals c16 and reported an existing cached/global MapEvents divergence for separate investigation. Neither preview success nor existing healing/story claims certify that drift repaired.
