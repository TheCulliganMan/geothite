# Voyageurs winding cleft

This complete 14×12-metatile cave replaces the cramped two-mouth room on the Voyageurs Border Loop. The west approach climbs toward the original Hyper Potion and passage scout. The dry gallery crosses to a separate corner stair and eastern approach. Two westward drop lanes skip the gallery climb on the return trip. A closed seep and an optional wayfinder battle give the eastern branch a reason to explore.

The outside trail takes 18 ordinary steps between the two returns; the cave's dry route takes 44. This is an optional exploration passage, not a faster regional shortcut. The internal drops are useful: their ordinary walking returns take 29 and 31 steps. Both lanes have solid reverse/side courses and real native jump behavior.

LEIF offers a Yes/No challenge with WHIRLNUT 43 and LUTEBRISK 44. The Normal/Flying scout and pure Ice lake specialist provide different attack and defensive roles. The inherited Minnesota trainer scaling, evolution and move machinery remains enabled. The original scout, Hyper Potion, item flag and all original programs survive exactly. There is no new SQ03 authority, admission charge or field-move requirement.

## Physical connections and sources

| Mouth | Interior ladder | Clear north step | Outside active entrance | Passive return |
|---|---|---|---|---|
| West, index 1 | `(5,21)` | `(5,20)` | `MnR20C00R00 (60,33)` | `(60,34)` |
| East, index 2 | `(23,21)` | `(23,20)` | `MnR20C01R00 (12,31)` | `(12,32)` |

Exterior warp records and indices remain exact. The east passive FLOOR record still carries its original target-warp 1 field; it is not an active entrance and must not be normalized. Both active entrances and passive returns were exercised through production input.

The native west stair 30 occupies metatile `(2,4)` and the east corner stair 36 occupies `(8,4)`. Their traversed walking columns are 4 and 16, from y10 through y9/y8 to y7. The westward jump approaches are `(14,12)` and `(14,13)`, with landings `(11,12)` and `(11,13)`. The supply remains at `(5,5)`, its approach `(4,5)`. The retained scout remains at `(12,7)`; LEIF stands at `(21,17)` with a clear approach `(21,18)`. The seep bank is `(17,15)`, facing south into `(17,16)`. All 187 free FLOOR quadrants are connected with the three real bodies present, and all 32 WATER quadrants remain internal behind native rock boundaries.

- `before.json` freezes the former 9×8 terrain source.
- `after.json` freezes the reviewed publication input and its original candidate annotations.
- `contract.json` binds the complete installed native module and both original warp pairings.
- `trainers.json` defines the optional challenge, exact regional party, dialogue and defeat flag.
- `current.json`, `route-dossier.json`, `sources.json` and the shared interior reservation identify the selected version.
- `plan.json` records the exact paths, fixture limits and retained evidence.

## Fast review and reproduction

From this checkout, the existing Pillow/CBOR renderer environment supports:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-r20-winding-cleft-v02.crystalpack \
  --source planning/minnesota/caves/r20-winding-cleft-v01/current.json \
  --output planning/minnesota/renders/r20-cleft-live --watch
```

The loop loads native art once and saves each source version, full original image and 2× view in a new immutable folder. Warm renders measured 0.013–0.028 seconds. All earlier passes, including the repeated pool caps and invalid water-camera position, remain under `planning/minnesota/renders/r20-cleft-v437/`. Source previews do not publish a pack.

The Rust publisher is `rebuild_r20_winding_cleft`. The verified original build and publication commands, exact parent SHA256 and source hashes are retained in `build-results-v439.json` and `publication-ready-v453.json` under that render folder. Its original parent is `content-packs/minnesota-r10-exploration-v03.crystalpack`; resolve `planning/minnesota/world.json` before further work. The publisher takes `PACK SHA256 BEFORE AFTER CONTRACT TRAINERS FRESH_OUTPUT`. It rejects stale full modules, changed original fields, duplicated/unsafe mouths, blocked terrain and foreign regional teams. Output must be fresh and ignored.

## Native evidence and limits

The published full-room pixels match the source. All nine headless production camera comparisons pass with zero mismatched pixels, and the actual images were inspected. The final room has continuous rock shoulders, a narrower stepped gallery, one correct corner stair and an enclosed pool without the first pass's repeated rock caps. Source/compiler counts alone were not used as visual acceptance.

Two current-source production-controller cases pass: both mouths/stairs/drop lanes and walking returns, original scout ownership, original reward/once-only persistence, actual 200 HP healing and Continue; optional decline, selected-move PP use, victory, postbattle conversation and Continue. The stdio-MCP course adds 227 recorded RPC calls through the two real exterior entrances, pickup, scout, drops, refusal, cross-cave trip and fresh-process Continue. Its stderr is empty. A registered native Old Rod cast from the actual bank hooked SCUTLET 28; Run restored bank movement and Start. That separate fishing course records 55 RPC calls.

The independent complete-pack audit preserves all 731 other maps, all original cave programs/actors/reward, every exterior record, native art/audio and unrelated catalogs. All dimension/event/script mirrors are checked. The touched cave's wild/fishing references, all regional trainer entries and active evolutions pass the original Minnesota roster audit. Both affected crates pass all-target Clippy with existing unrelated warnings. Owned Rust formatting passes; full workspace formatting retains unrelated differences beginning at `offset-allocator/src/lib.rs:408`. Audio verification checks all 377 programs' PCM hashes, frame counts and loop ranges.

Initial positions, partners, seven badges/depot flags and supplies are declared local QA setup. The real Route 20 access contract requires eight badges and lighthouse progress. These tests establish local mechanics and persistence, not earned regional access or endgame balance. Unrelated inherited old LAKE/SHORE fishing references on other maps remain outside this delta. The broader map/cave improvement goal remains active.

The selected v02 child retains the later Route3 stair-direction correction. `whole-root-audit-v454.json` proves its complete cave, shared gameplay tables, art and audio are identical to the v01 pack used for the controller, MCP and native camera cases. The only other gameplay delta is two text-only Route3 labels and their module mirrors.
