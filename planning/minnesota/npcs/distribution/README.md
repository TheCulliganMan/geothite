# Regular trainer mix: routes and State Fair

**125 route trainers and 30 State Fair trainers now use the 23 accepted regular trainer types.** All 20 numbered routes are included. Field and battle art match. Existing names, parties, levels, AI, dialogue, defeat flags, sight ranges, movement and rewards remain authored.

All eight Gym Leaders remain inside indoor Gym maps; all four Elite Four members remain inside indoor League rooms. None of their sprite references appear on routes or at the State Fair. Reed retains his private rival art and role. The original named NPC encounters are preserved. Five explicitly named child fair trainers retain their original native art rather than receiving adult professional sprites.

Bakers, dairy farmers, orchard keepers, rice harvesters and botanists match the fair food/exhibit roles; the wider fair crowd includes all 23 regular types. Routes use regional pools, including rangers, anglers, canoe guides, miners, lumberjacks, bird watchers, couriers and others.

## Play locally

From `/Users/ryanculligan/GitHub/geothite`:

```sh
./target/trainer-cast/tools/geothite play content-packs/minnesota-route-fair-trainer-mix-v02.crystalpack
```

The separate State Fairgrounds was built from the current native dossier/layout onto the existing 36-character compact-lineage pack before applying this art overlay. It connects to the existing `MnR05C00R02` road through the authored two-lane eastern entrance. The fair builder owns the bounded nine-metatile road connection; the trainer overlay changes no terrain. No active-world pointer or deployment was promoted.

[Route and State Fair gameplay gallery](../../../../target/trainer-distribution/gallery.png)

## Verification

- All 155 actor/battle assignments audited. Every other trainer field, raw script operand, actor property, map field and game-data field is preserved, including bosses. All runtime art and audio are byte-identical to the fair parent.
- 24 individual production-controller encounters: all 23 regular types tested across routes and the fair, plus an additional Gunflint encounter. Matched field/battle art, native dialogue, move PP, battle save/load, legal victory, repeat interaction and saved defeat state pass. Fair fixtures mark other fair battles as previously defeated to isolate the selected encounter. Three legal level-60 Minnesota starters and adjacent player positions are explicit fixtures; this is not earned traversal or intended-level balance proof.
- Production client MCP crosses both real fair/road lanes, walks to the existing road and returns through native inputs. Adjacent-position saves are explicit initial fixtures, not earned journeys to the fair.
- Fresh production-client processes exercise Start/Pack, movement, downstairs introductory dialogue, autosave and reload; stdout is valid JSON-RPC and stderr empty.
- A real-pack regression verifies preservation of the Gunflint control/text programs and rejection of stale actors, duplicates and boss/rival assignments.
- All eight final encounter review pages inspected: actual route/fair battle intros and field scenes. Art itself is reused byte-for-byte from the accepted hand-edited native sources.

Full all-target Clippy remains blocked by the pre-existing crystal-core bad-bit-mask error. Own Rust files pass formatting; focused all-target no-deps Clippy passed (with existing warnings; the unrelated never_loop lint was allowed). Workspace formatting reports unrelated existing differences.

A v01 candidate was rejected because a broad metadata refresh also regenerated unrelated route text/control programs. The publisher now refreshes only actor event rows and the class-bearing battle metadata; v02 passes the strict preservation audit. An older frozen fair service test used an obsolete entrance fixture and failed before service checks; the actual current two-lane connection is instead verified through the production client. This is not a claim to have rerun every fair service.

## Distribution

| Regular type | Route encounters | Fair encounters |
|---|---:|---:|
| Northwoods Ranger | 8 | 2 |
| Canoe Guide | 6 | 1 |
| Iron Range Miner | 3 | 1 |
| Prairie Botanist | 5 | 2 |
| Lakeside Angler | 8 | 1 |
| Winter Courier | 9 | 1 |
| Hockey Captain | 1 | 1 |
| Orchard Keeper | 5 | 1 |
| Rice Harvester | 8 | 1 |
| Harbor Mechanic | 1 | 1 |
| Trail Photographer | 7 | 1 |
| State Fair Baker | 1 | 3 |
| Storm Watcher | 9 | 1 |
| Fossil Researcher | 1 | 2 |
| Night Sky Observer | 8 | 1 |
| Cycle Tourist | 5 | 1 |
| Beekeeper | 6 | 2 |
| Lumberjack | 7 | 1 |
| Bird Watcher | 5 | 1 |
| Dairy Farmer | 7 | 2 |
| Mushroom Forager | 4 | 1 |
| Volunteer | 3 | 1 |
| Ski Patroller | 8 | 1 |

Source overlay: [route-fair-v1.json](route-fair-v1.json).

Review and logs remain ignored under `target/trainer-distribution/`. The earlier 36-character gallery and native source manifests remain under `planning/minnesota/npcs/cast/`; no new image generation or sprite recoloring was needed.

Final pack SHA-256: `e3076ab62edfb492d5c30b40a4c3ea60db660185aa3ed97a81c408d9ce9056ce`.
