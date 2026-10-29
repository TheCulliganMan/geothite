# Final NPC verification

Pack: `content-packs/minnesota-original-npcs-15-v01.crystalpack`.

Fifteen original NPCs have matched 56x56 battle art, six directly hand-authored 16x16 overworld poses each, independent four-color palettes and registered trainer encounters. The user rejected earlier art; every earlier set was redrawn again against actual Crystal sprites before renewed acceptance. Per-character comparisons use equal scale and a common display palette, followed by own-palette and actual game review. Original battle references/prompts are retained; generated concepts alone do not count.

Final combined-pack audit: 15 unique trainer/class/actor registrations, 90 exact native art resources, legal adjacent approaches, native active species/evolution families and original party levels. Every original runtime resource, trainer, audio payload, map terrain/attributes/events/actors/script body and unaffected gameplay field matches the original Minnesota parent. Native converter verification.json files are encoding-stage outputs; final runtime/manual acceptance lives in the per-NPC manifests and final audit. Exact source/evidence hashes are in `target/npc-sprites/final-audit/verification.json`; audit code and log remain with the ignored verification evidence.

All 15 complete encounters passed sequentially against this exact final pack using the production VisibleShellController. Each checked art loading/four-direction walking, talk/input ownership, a PP-spending turn, battle save/load, legal-input victory, defeat flag before capture and follow-up dialogue. Setup uses three legal level-60 Minnesota starters with maximum DVs; opponents retain native teams/scaling. The test driver uses legal Perish Song/Destiny Bond counterplay. Earlier failed Lark runs remain documented, not relabeled as passes.

Production client built with the pinned toolchain. A fresh `geothite mcp` session on the final pack opened Start/Pack, moved normally, talked to the home NPC, blocked directional movement during dialogue, saved and reloaded matching state. Stdout remained clean JSON-RPC. Transcripts: `target/npc-sprites/final-client/`. The smoke save is separate from user saves.

## Roster

| NPC | Map | Tile | Final-pack encounter |
|---|---|---|---|
| RANGER ROWAN | MnR01C00R00 | 8, 24 | passed (20.7s) |
| CANOE GUIDE OREN | MnBrainerd | 42, 21 | passed (20.01s) |
| MINER MARA | MnGrandRapids | 22, 24 | passed (22.25s) |
| BOTANIST ELDEN | MnMankato | 32, 24 | passed (25.21s) |
| ANGLER NETTA | MnBemidji | 28, 35 | passed (20.94s) |
| COURIER IVAR | MnInternationalFalls | 16, 18 | passed (25.95s) |
| CAPTAIN LEIF | MnStCloud | 14, 13 | passed (27.18s) |
| ORCHARD KEEPER POMONA | MnRochester | 23, 12 | passed (23.05s) |
| RICE HARVESTER SEDGE | MnLeechLake | 24, 19 | passed (26.74s) |
| MECHANIC TOBIN | MnDuluth | 54, 59 | passed (24.92s) |
| PHOTOGRAPHER LARK | MnEly | 23, 18 | passed (32.77s) |
| BAKER DOTTIE | MnStPaul | 27, 20 | passed (63.42s) |
| STORM WATCHER GALE | MnMoorhead | 16, 13 | passed (28.6s) |
| FOSSIL RESEARCHER FARA | MnWinona | 16, 13 | passed (24.0s) |
| NIGHT SKY OBSERVER ASTER | MnGrandMarais | 26, 27 | passed (24.26s) |

## Build and run

Executed build:

```sh
CARGO_BUILD_JOBS=1 cargo build --locked -p geothite --config profile.dev.package.crystal-assets.opt-level=0 --config profile.dev.package.crystal-bevy.opt-level=0
```

Verified production entry point:

```sh
target/debug/geothite mcp content-packs/minnesota-original-npcs-15-v01.crystalpack
```

Interactive terminal entry point:

```sh
target/debug/geothite play content-packs/minnesota-original-npcs-15-v01.crystalpack
```

Review gallery: `target/npc-sprites/final-audit/npc-roster-review.png`. Source records: `planning/minnesota/npcs/sprites/`. All packs, native encoded bundles, original Crystal references and verification output remain ignored; do not commit generated pack contents or native references.

## Check scope

Edited Rust leaf files pass formatting; focused Clippy passes with existing unrelated warnings. Workspace-wide formatting still differs in unrelated files, beginning in offset-allocator. Normal Clippy still fails in multiplayer/mod.rs:1332 bad_bit_mask; the scoped no-deps check uses -A clippy::never_loop for another existing blocker. See VALIDATION.md and clippy-counterplay-all-v1/focused-v1 logs. No production battle rules, opponents or source-wide lint suppression were changed.
