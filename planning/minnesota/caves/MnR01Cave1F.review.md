# R01 limestone hollows — source-only review

Final candidate: `MnR01Cave1F.candidate.json`, SHA256 `214bd2388b04598e4bca79834a7ac61684870ceffa301b9dac4580ebd9458243`. Exact original: `MnR01Cave1F.previous.json`, SHA256 `f19975c19bc4e17f2340143091f349ad941096b56f00efe2dd538b7f34408f5a`. No active route/interiors source, native module, exporter or Rust guard is changed. Native publication and production input acceptance remain pending.

Evidence is frozen under `../renders/caves/r01-candidate/214bd2388b04598e/`. Review `whole-before-after.png`, `whole-2x.png`, `decision-cameras.png`, `antidote_discovery-4x.png` and `actual_weakest_floor-4x.png`. All original/crop/conversion/enlargement stages are retained. The full original route, room and native field hashes are retained at `baseline-1630-1793070873062798000/`. Rejected first pass42a3ad2fd0a01a6e remains complete.

The original8×7 room is a broad common rectangle with two identical northern caps. The candidate stays8×7. Its entrance offers two short ordinary climbs: a longer west hollow containing the original guide, and a shorter east Antidote alcove. The west stair faces a different way from the east stair and both climbs have real blocking sides. The original guide still correctly says that two small hollows meet by the exit and invites either path. No one-way ledge is justified in this small introductory room; no HM gate, extra actor, battle, new item, water or additional ladder is added.

After whole/player review, Root identified that the first pass's right shelf was still a broad rectangle resembling the later refuge benches. The final bounded revision changes just three metatiles from that first pass:4,1 `04→01`,5,1 `05→04`,4,2 `08→04`. This moves the native ceiling inward and tapers the discovery alcove into a small hook without moving the Antidote. The whole and actual weakest/player frames were inspected again. The guide hollow remains deliberately simple and provides passing space around its original actor; maximum empty all-Floor square is3core tiles, at the weakest camera4,7.

All original fields except `rows` are exact; only nonruntime `review_contract` is added. The native installed module's full56-block original matches the original source. Native actor IDs/positions/sprites, actual Antidote×1 program/event and paired exterior CAVE/passive FLOOR are checked against the supplied pack.

| Retained interaction | Exact core tile |
|---|---|
| `MN_R01_HOLLOW_GUIDE`, original Pokéfan and all five lines |5,5|
| `MN_R01_ITEM_03`, ANTIDOTE×1, `EVENT_MN_R01_ITEM_03` |11,5|
| Interior sole LADDER / clear arrival |9,11 /9,10|
| `MnR01C00R00` active CAVE / passive FLOOR |18,17 /18,18|

Interior warp1 still targets the exterior's warp2; both exterior records target interior warp1. The original exterior anchor remains9,8 metatiles. The candidate does not rewrite any pairing.

| Ordinary climb | Native metatile/block | Exact core lane | Actual wall side |
|---|---|---|---|
| Guide hollow |2,4 /cave12|5,7→5,8→5,9→5,10|4,9|
| Antidote alcove |5,3 /cave30|10,5→10,6→10,7→10,8|11,7|

All points are unoccupied FLOOR, with both original bodies present. Closing each stair lane disconnects its intended upper destination; it is an actual climb rather than a stair prop beside a dry bypass. The source validator checks the entire ordinary Floor graph, all actor approaches, closed outer collision, sole declared ladder, native art/palette and legal camera positions. Fifteen negative mutations are rejected, including changed actor/item/flag/exit, both blocked stairs, both side bypasses, false/missing ladder, open border, unexpected water and occupied camera.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/caves/MnR01Cave1F.review.py --self-test
```

The complete verified final loop took1.769 seconds, including pack load and rendering. It uses the exact1630 glacial pack and native cave source art, forced NITE, correct sprite directions and the actual tile-top-left sprite anchor without the historical minus4px offset. Source player views are clamped160×144 previews, clearly separate from production screenshots. The supplied pack SHA256 is `1630ad063aa4a1cf252087bdf743ca48a2a8cb6c9c46ad95cb7aceb1c74c3b05`.

Next native acceptance must preserve the whole module/programs except these56 terrain blocks, every other map and all global mirrors, then prove paired entry/exit, both stair lanes and side rejection, original guide text/input ownership, actual Antidote reward/full bag/retry/repeat/Continue and matching native headless captures. Position/party fixtures cannot establish earned early-route travel. Root owns integration and source promotion.

## Authored production-controller acceptance — execution pending

`crates/crystal-bevy/src/bevy_shell/tests/r01_hollows.rs` contains two ignored cases targeting `CRYSTAL_R01_HOLLOWS_CONTROLLER_PACK`. The default source is the exact immutable214 candidate; `CRYSTAL_R01_HOLLOWS_SOURCE` can select an equivalent captured source. Root owns registration, build and actual execution. Scoped rustfmt/check passed; this worker ran no Cargo command and does not claim these cases passed.

- `r01_hollows_controller_both_stairs_bidirectional_and_true_passive_return`: both source-declared ordinary stairs in both directions with actual wall-side rejection; original entry9,11 and real exterior passive18,18; idle/revisit without rewarp; Continue and paired reentry.
- `r01_hollows_controller_original_guide_antidote_once_cures_poison_and_continues`: exact three fully printed original guide pages and rejected movement while dialogue owns input; original Antidote×1 and saved collection event; fully printed pickup notice; production Start/Pack/USE/party target cures initially poisoned legal Totodile20, consumes the Antidote and changes only the current status (preserving HP/moves/PP at use); reentry cannot collect again, poison remains cured and Continue retains party/resources/semantic flags.

Initial setup is explicit local position, optional legal party/POISON status and deterministic RNG. Every subsequent gameplay action uses controller input; no flags, bag, party or position are edited after setup. Existing R10 page-printer/Continue/settling/semantic-flag helpers and shared authored movement are reused. The small Antidote target sequence is local because the existing exported medicine helper supports PP items only.

## Native presentation and first controller run

Root published the exact214 terrain into the510-map limestone review pack e549c178… . Read-only verification and six headless production views now PASS, with147 preservation checks and zero measured player/terrain mismatches; see `../renders/caves/r01-native/README.md`. These are native images, distinct from the earlier source previews. Parent owns broader acceptance and source promotion.

The first actual two-case controller run passed stairs/mouth/true passive return. Guide's three pages, Antidote pickup and actual poison cure also passed before a repeat-check fixture failed: a single Right input from the north-facing approach turned in place, leaving10,5. The original leaf and failure log remain retained. The revised test checks the actual turn and then bounded production directional steps, retaining the strict11,5 walk-through/no-second-reward assertion and Continue checks. No controller/runtime code changed. The revised execution is pending Root's build. Local R01 full-bag retry remains untested; shared R10 coverage is not represented as a passing local test.
