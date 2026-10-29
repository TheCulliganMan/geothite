# CAPTAIN LEIF: accepted native and playable

Deliverable: `content-packs/minnesota-npcs-hand-08-v03.crystalpack`. Parent: `content-packs/minnesota-npcs-hand-07-v01.crystalpack`. Existing trainers and runtime assets passed preservation checks. Placement: MnStCloud [14, 13], facing south, approach [14, 14].

Six native 16x16 overworld poses and one 56x56 battle sprite share an independent four-color GBC palette. Source: `native-pixels-v2.json`. Placement: `placement-v1.json`. PNG/palette/tile/alpha agreement, nonblank poses, distinct walking pairs and independent tile decode passed.

The production controller test passed exactly once, 0 failures. It checked native art loaders and visible palette, introduction/input ownership, a PP-spending move, battle save/load, a complete legal-input win, defeat flag before render capture, and follow-up talk. Log: `target/npc-sprites/hockey/controller-hand-v01.log`. Actual game captures: `target/npc-sprites/hockey/gameplay-hand-v01`. These images were manually inspected before this record was written: Hand-redrawn six native grids reviewed at 1x and 8x and in actual St Cloud capture. Larger dark-haired face and navy jersey, distinct rear hair, real walking stride, anatomically consistent stick hand, connected padded glove, taped blade separated from skate. Full controller encounter passed. Battle rows and palette unchanged.

Evidence hashes are in `manifest.json`. This record accepts this NPC only; the goal still requires at least 15. Continue one at a time using this pack as the next parent. The generic test runs with explicit CRYSTAL_INTERIOR_PACK, CRYSTAL_ORIGINAL_NPC_SOURCE, POKEGEAR_PC_RENDER_DIR and RUST_MIN_STACK=33554432.
