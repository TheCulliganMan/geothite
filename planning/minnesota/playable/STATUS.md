# Grand Marais verification

Verified locally on 2026-10-25. Visual review remains open. This town is preserved while route work proceeds. North Shore and Gunflint now have native overlays and focused controller fixtures, with connected regional progression still pending.

## Current exterior

The center's fenced pocket and cut tree have been removed. The promenade is continuous. A second hand pass removed redundant street strips, gave the field station a wider facade, restored the lighthouse crown and lantern room, and tightened the shoreline and pine edge. The west lookout rail now follows the shore, with no disconnected walkable pocket. The geometry review requires every walkable exterior tile to join the same reachable area, excluding occupied residents. The authoritative layout is [grand-marais.json](grand-marais.json): 20×19 metatiles, or 40×38 gameplay tiles, with six working entrances and seven exterior residents.

The [complete published-pack render](../renders/grand-marais/whole-town-25a561a8.png) contains the entire exterior at 2× native scale. Its 380 metatiles, door anchors, sign positions and resident positions were checked against the layout. [The native waterfront capture](../renders/grand-marais/native-waterfront.png) was also inspected. The [six-interior review](../renders/grand-marais/interior-review.png) now shows four distinct authored rooms: a laboratory, school workroom, cottage and stone lighthouse chamber. The Center and Mart retain their native service layouts. The [native west lookout capture](../renders/grand-marais/native-west-lookout.png) was checked after moving its guardrail to the shoreline. All render files are ignored local review output.

The fast preview uses the same JSON and the supplied pack's source art. All authored town dialogue now lives in that JSON, including the eleven interior text bodies and four post-repair resident reactions. A JSON-only republish measured 0.74s without invoking Cargo. The published-pack renderer checks all authored dialogue against the current source, including signs; signs guide the shore walk east and then south toward the lighthouse. Warm exterior and four-room edits measured 46ms in the latest pass; earlier exterior-only edits measured 40–93ms. Start it with the command in the [runbook](README.md); the browser review is at <http://127.0.0.1:8765/>.

## Executed checks

All Cargo commands used `CARGO_TARGET_DIR=target/runtime-fps` and the pinned toolchain. Tests used the `wasm-dev` profile and `--locked`.

| Check | Result |
| --- | --- |
| `crystal-assets --test grand_marais` | Passed: original content preserved, seven maps added, original spawn-table size retained, duplicate application, an exit placed on a wall, and missing starter dialogue rejected. |
| `crystal-bevy --test grand_marais` with the town pack | Passed: fresh-game starter scene, dialogue ownership, all three physical signs, the west lookout, every entrance and return, lighthouse delivery and one-time rewards, four resident reactions, return dialogue, real encounter and move PP mutation, battle completion, healing, opening/closing the visible PC, purchase with money deduction, save/load and the saved harbormaster reaction. |
| `crystal-bevy --lib grand_marais_keeper_retries_a_full_bag_without_losing_the_part_or_reward -- --ignored` | Passed: deliberate capacity fixture, native refusal dialogue, intact bag/part, successful retry and actual repeat conversation without a second reward. |
| `geothite --test grand_marais` with the town pack | Passed: production MCP movement, Start menu and observation; stderr empty. |
| `crystal-bevy --lib renderer_neutral_controller_` with the canonical TUI pack | 11 passed, one ignored. Includes Mom/New Bark gate, production battles and retained reward presentation. |
| Canonical `geothite mcp_drives_real_movement_and_production_start_menu` | Passed. |
| Canonical `geothite --test stdio_clean` | Passed. |
| `verify_crystalpack ... --audio --desktop` on the town pack | Passed: all 377 PCM hashes, frame counts and loop ranges; 395 maps. |
| Release `geothite` build | Passed. |
| Release MCP walkthrough | Completed starter, cottages, keeper, real wild battle, nurse and shop using production inputs; stderr empty. |
| Interactive release TUI | Rechecked after the room redesign: launched in the laboratory; obtained starter, moved into town, opened Start/Pack, saved with F5 and exited cleanly. A temporary save was checked for nonzero length and removed. |

The walkthrough exposed a production controller defect: battle EXP presentation advanced in the graphical loop but could retain input forever in the renderer-neutral controller. Both now use the same reward clock. A focused regression checks EXP completion, visible level-stat values, input ownership, acknowledgment and return to walking.

The three town regressions were rerun after moving all authored dialogue into the JSON; all three passed. The separate full-bag fixture also passed: refusal preserved the complete bag and signal part, freeing a slot permitted delivery, and the actual repeat conversation awarded no second potion. The updated pack again passed desktop and all 377 audio checks. A fresh headless production capture at town tile `(19, 12)`, facing down at noon, was inspected against the source-art layout. Preview crops now follow the authored door and resident anchors rather than coordinates from the older layout.

## Formatting and lint limits

New Rust files and the added regression were formatted. Workspace `cargo fmt --all -- --check` still fails on existing unrelated formatting, beginning in `crates/offset-allocator`.

Normal all-target Clippy is blocked by an existing `clippy::bad_bit_mask` in `crystal-core/src/multiplayer/mod.rs:1332`. A dependency-free check encounters existing `clippy::never_loop` errors in `crystal-assets/src/mutation_protocol.rs`. The following targeted check completed with existing warnings and no warnings in the new town modules, tests or reward-clock module:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo clippy --profile wasm-dev --locked --no-deps \
  -p crystal-assets -p crystal-bevy --all-targets -- -A clippy::never_loop
```

The local pack remains ignored at `content-packs/minnesota-grand-marais.crystalpack`. These results establish the town's implemented gameplay; they do not establish visual acceptance or completion of the Minnesota region.
