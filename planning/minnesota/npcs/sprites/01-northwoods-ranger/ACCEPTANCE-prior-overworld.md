# Ranger accepted: NPC 1 of 15

Current deliverable: `content-packs/minnesota-original-ranger-v04.crystalpack`, built on `minnesota-trainers-v02.crystalpack`. Rowan stands at MnR01C00R00 (8,24), facing south; approach (8,25). Existing trainers and runtime assets are preserved.

The production controller test passed exactly once (1 passed, 0 failed, 17.56s). It verified native art loaders and palette, introduction and dialogue ownership, PP spending, battle save/load, a complete legal-input victory, the actual defeat flag before any rendering capture, and post-win conversation. See `target/npc-sprites/ranger-controller-v05.log`.

Reviewed `target/npc-sprites/ranger/gameplay-v05/`: trainer intro, first battle turn and overworld follow-up. Hat, braid, binoculars and green/tan outfit are readable; overworld scale fits the player and native terrain. Native banks have six 16x16 overworld poses and a 56x56 battle view, with four GBC colors and PNG/tile roundtrip validation. Source is `native-pixels-v1.json`, placement `placement-v2.json`. Evidence hashes are in `manifest.json`.

Earlier missing-palette failure was a test mount setup issue, corrected using production runtime materialization. The naive raw-power battle driver failed on the scaled parent; legal type-aware move selection with the same party and unchanged opponent now passes. No victory assertion, trainer party or opponent stats were weakened.

Repeat with the exact compiled executable `target/debug/deps/crystal_bevy-0e4f79a33ebb6d13`, filter `original_npc_controller_talk_battle_turn_and_save --ignored --nocapture`, and environment RUST_MIN_STACK=33554432, CRYSTAL_INTERIOR_PACK pointing to v04, CRYSTAL_ORIGINAL_NPC_SOURCE pointing to placement-v2.json, and POKEGEAR_PC_RENDER_DIR pointing to a fresh ignored review folder. Recompile only when relevant code changes require it.

The full goal is not complete. Proceed with the canoe guide next, preserving this accepted ranger in its parent pack.
