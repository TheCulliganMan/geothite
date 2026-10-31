# Iron Range crosscut: rock walls and an optional supply loop

The selected `iron_crosscut_rock_cache_revision` improves the existing16×13 crosscut. Eighty black WALL metatiles become the original cave `2e` rock fill, with a matching solid border and the original NITE palette. All832 collision quadrants, existing paths, two opponents, story programs and two real stairways remain unchanged. Four warp records represent the two active/passive stair pairs.

The upper-left side loop now offers one optional Ether at walking tile `(5,5)`, approached from `(4,5)`. The native item ball uses `MN_IRON_APPROACH_ETHER` and independent once-only flag `EVENT_MN_IRON_APPROACH_ETHER`. A full item pocket leaves the ball and flag untouched; native Toss and retry collect one Ether, and reached Continue preserves its removal. Walking across the cave does not collect it or grant chapter progress.

`before.json`, `after.json` and `contract.json` are actionable sources. The contract binds the complete native preimage, stair pairs and existing trainer anchors. `plan.json` records the exact reward, arrivals, preservation rules and evidence. Both existing parties use active original Minnesota species. The publication retains every prior program, party, native sprite and audio asset; all731 other complete maps remain exact, including current Route14 and Minneapolis cycling.

The final local pack is `content-packs/minnesota-iron-crosscut-rock-cache-v02.crystalpack`, SHA256 `bc38b8920bdc6d5e3c0e18ddac283aa4087e0020d694861f277d10314a3875d1`. It extends `route_grass_r14_forest_bank_revision`. The earlier v01 candidate extends the older cycling revision and remains historical review evidence.

## Executed publication and fast review

Rust owns publication and strict readback. This executed command requires a fresh ignored output on rerun:

```sh
target/runtime-fps/debug/revise_iron_drill_approach \
  content-packs/minnesota-current-r14-forest-bank-v01.crystalpack \
  e9f4108d3b0a35f04145e4928446c03845982e1f13d7ff8f34e383bd701757d5 \
  planning/minnesota/caves/current-iron-approach-v01/before.json \
  planning/minnesota/caves/current-iron-approach-v01/after.json \
  planning/minnesota/caves/current-iron-approach-v01/contract.json \
  content-packs/minnesota-iron-crosscut-rock-cache-v02.crystalpack
```

The publisher source is `crates/crystal-assets/src/iron_approach_revision.rs`. It rejects stale modules, altered floor geometry, changed stair or actor bindings and any reward other than the declared cache. The negative guard case passed; the independent full-root audit reconstructs the expected delta separately.

For source review without rebuilding or changing a pack, run `planning/minnesota/tools/review_iron_approach.py` with `--pack content-packs/minnesota-current-r14-forest-bank-v01.crystalpack --directory planning/minnesota/caves/current-iron-approach-v01 --output planning/minnesota/renders/iron-approach-source-review`. Add `--watch` for fast edits. Use the existing Python environment with Pillow/cbor2. Each source review writes immutable complete-room originals and enlarged copies; the measured warm loop was0.098seconds.

## Native evidence and limits

All evidence is retained under `planning/minnesota/renders/iron-approach-v356/`, including rejected harness attempts:

- `final-whole-room-original-v390.png` and its2× copy: final pack's complete native room, matching the reviewed source pixels.
- `final-native-gpu-v386/`: five inspected actual native2D camera captures, all640×576, with zero mismatches against the same source palette, actors, position and state.
- `controller-v383.log` and `controller-result-v383.json`: both final-child production-controller cases pass, covering full-pocket rejection/input ownership, native Toss/retry, once-only award, actual reached saves and Continue; both stairways in both directions and each safe arrival also pass.
- `final-production-stairs-v391/receipt.json`:125 production stdio-MCP calls, four real stair transitions, room walking, reached save/fresh-process Continue and Start; stderr is empty and original inputs remain unchanged.
- `whole-root-preservation-v392.json`: only the declared wall/cache/mirror/new-flag delta, all731 other maps exact, original parties legal, art/audio/runtime files unchanged.
- `code-check-results-v393.json`: completed shared-checkout all-target Clippy for both affected crates with applicable features; owned leaf rustfmt passes. Existing warnings and unrelated workspace-format differences remain.

Reusable controller tests are in `crates/crystal-bevy/src/bevy_shell/tests/iron_approach_cache.rs`. Set `CRYSTAL_INTERIOR_PACK` to the absolute final pack and `CRYSTAL_IRON_APPROACH_REVIEW_DIR` to a fresh ignored directory, then run `cargo test --locked -p crystal-bevy --lib --features location-tester iron_approach_ -- --ignored --nocapture --test-threads=1` with `RUST_MIN_STACK=67108864`.

The initial positions, original Minnesota parties, full pocket and prior foreman/local trainer wins are explicit QA setup before the first input. Later transaction, walking, warps, save and Continue use the production controller. They prove this bounded change; they do not establish earned regional progression or completion of the broader map/cave-quality goal. The first stair helper chased old-map coordinates after an automatic warp; the corrected adjacent-floor/explicit-travel test passes. The earlier Toss harness expected a cursor where the native UI uses quantity and confirmation text; its continued actual save passed the corrected production-input course. Every earlier render, save and rejected record remains preserved.
