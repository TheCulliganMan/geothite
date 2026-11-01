# Original NPC roster

Fifteen original NPCs with matched battle art and hand-authored overworld sprites.

Final combined pack: `content-packs/minnesota-original-npcs-15-v01.crystalpack`.

Each NPC has one native 56x56 battle sprite, six 16x16 overworld poses and an independent four-color GBC palette. The engine mirrors the left poses for right-facing movement. Every set was compared directly with actual Crystal sprites at native size and equal enlargement, reviewed in its own palette and checked in actual game captures. Earlier rejected versions and failed encounter logs are retained as history; current manifests identify the delivered source versions.

The final pack adds exactly 15 trainers/actors and 90 native runtime art files. Original trainers, runtime resources, audio, map terrain/events/actors/scripts and unaffected gameplay data passed preservation checks against the original Minnesota parent. All parties and evolution families use active native Minnesota species. Each NPC was built and verified individually before advancing.

See [final verification](FINAL-VERIFICATION.md) for the final-pack encounter results, production-client menu/dialogue/movement/save/reload smoke, evidence hashes and check scope. Review gallery: `target/npc-sprites/final-audit/npc-roster-review.png`. Detailed real-sprite comparisons are under `target/npc-sprites/hand-review/`.

## Run locally

The built production client is `target/debug/geothite`. The verified MCP entry point is:

```sh
target/debug/geothite mcp content-packs/minnesota-original-npcs-15-v01.crystalpack
```

For an interactive terminal:

```sh
target/debug/geothite play content-packs/minnesota-original-npcs-15-v01.crystalpack
```

## Roster

| # | NPC | Map and tile | Authored party | Source and evidence |
|---|---|---|---|---|
| 1 | RANGER ROWAN | MnR01C00R00 (8, 24) | BIRCHIT 5, POUCHID 6 | [record](sprites/01-northwoods-ranger/ACCEPTANCE.md) |
| 2 | CANOE GUIDE OREN | MnBrainerd (42, 21) | BIRCHIT 28, RIVVIT 30 | [record](sprites/02-canoe-guide/ACCEPTANCE.md) |
| 3 | MINER MARA | MnGrandRapids (22, 24) | TACONIB 35, PELLETOR 37 | [record](sprites/03-iron-range-miner/ACCEPTANCE.md) |
| 4 | BOTANIST ELDEN | MnMankato (32, 24) | BLUESTIP 18, RICELING 20 | [record](sprites/04-prairie-botanist/ACCEPTANCE.md) |
| 5 | ANGLER NETTA | MnBemidji (28, 35) | JIGLET 36, PIKELING 38 | [record](sprites/05-lakeside-angler/ACCEPTANCE.md) |
| 6 | COURIER IVAR | MnInternationalFalls (16, 18) | WINKFIN 38, TIPULORD 40 | [record](sprites/06-winter-courier/ACCEPTANCE.md) |
| 7 | CAPTAIN LEIF | MnStCloud (14, 13) | PUCKLET 22, SKATUFF 24 | [record](sprites/07-hockey-captain/ACCEPTANCE.md) |
| 8 | ORCHARD KEEPER POMONA | MnRochester (23, 12) | CRISPIP 12, POUCHID 14 | [record](sprites/08-orchard-keeper/ACCEPTANCE.md) |
| 9 | RICE HARVESTER SEDGE | MnLeechLake (24, 19) | RICELING 26, REEDRAFT 28 | [record](sprites/09-wild-rice-harvester/ACCEPTANCE.md) |
| 10 | MECHANIC TOBIN | MnDuluth (54, 59) | TUGGET 30, LIFTRUSS 32 | [record](sprites/10-harbor-mechanic/ACCEPTANCE.md) |
| 11 | PHOTOGRAPHER LARK | MnEly (23, 18) | LOOBIT 40, YODELOON 42 | [record](sprites/11-trail-photographer/ACCEPTANCE.md) |
| 12 | BAKER DOTTIE | MnStPaul (27, 20) | CHISLET 28, BUTTERMUSE 30 | [record](sprites/12-state-fair-baker/ACCEPTANCE.md) |
| 13 | STORM WATCHER GALE | MnMoorhead (16, 13) | WINDHOPPER 24, VANECHICK 26 | [record](sprites/13-storm-watcher/ACCEPTANCE.md) |
| 14 | FOSSIL RESEARCHER FARA | MnWinona (16, 13) | TRILUME 28, SIPHLET 30 | [record](sprites/14-fossil-researcher/ACCEPTANCE.md) |
| 15 | NIGHT SKY OBSERVER ASTER | MnGrandMarais (26, 27) | MOTHOSAIC 32, SOLSTORT 34 | [record](sprites/15-night-sky-observer/ACCEPTANCE.md) |

## Authoring and verification

Battle reference sources and prompts remain in each sprite folder. Six overworld grids are authored directly in the native-pixels JSON; they are not scaled concept art. The final source is native-pixels-v3.json for the first twelve revised sets and native-pixels-v1.json for Gale, Fara and Aster. Native conversion/encoding, independent tile verification, pack registration, adjacent walkable approach, complete controller encounter and actual game review all precede per-character acceptance.

The final production controller sweep uses this exact combined pack for all fifteen encounters. It verifies four-direction art/walking, talk/input ownership, PP expenditure, battle save/load, a legal-input victory, defeat flag before render capture and follow-up talk. The fixture party is three legal level-60 Minnesota starters with maximum DVs; production teams/scaling are unchanged. A separate fresh production-client save verifies real Start/Pack menus, movement, dialogue ownership and save reload through clean JSON-RPC.

Packs, encoded native bundles, original Crystal references and generated verification artifacts remain ignored. Do not commit generated pack contents or native reference art. Original work and current source/evidence records remain reviewable. Check limitations and exact logs are documented in [VALIDATION.md](VALIDATION.md).

Native converter verification.json files describe the encoding stage; final manual/runtime status is in the per-NPC manifests and final-audit/verification.json.
