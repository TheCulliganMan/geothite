# PHOTOGRAPHER LARK: accepted native and playable

Deliverable: `content-packs/minnesota-original-npcs-11-v01.crystalpack`. Parent: `content-packs/minnesota-original-npcs-10-v01.crystalpack`. Existing trainers and runtime assets passed preservation checks. Placement: MnEly [23, 18], facing south, approach [23, 19].

Six native 16x16 overworld poses and one 56x56 battle sprite share an independent four-color GBC palette. Source: `native-pixels-v1.json`. Placement: `placement-v1.json`. PNG/palette/tile/alpha agreement, nonblank poses, distinct walking pairs and independent tile decode passed.

The production controller test passed exactly once, 0 failures. It checked native art loaders and visible palette, introduction/input ownership, a PP-spending move, battle save/load, a complete legal-input win, defeat flag before render capture, and follow-up talk. Log: `target/npc-sprites/photo/controller-v01.log`. Actual game captures: `target/npc-sprites/photo/gameplay-v01`. These images were manually inspected before this record was written: Reviewed native-size and enlarged six-pose strips and battle art, then actual intro and post-win Ely captures. Rebuilt separate facial eyes and a clear cream camera lens in its black body. Short light hair, rust vest, shorts, long socks, hiking boots, camera and square daypack match across views; rear view shows pack and no front facial details.

Evidence hashes are in `manifest.json`. This record accepts this NPC only; the goal still requires at least 15. Continue one at a time using this pack as the next parent. The generic test runs with explicit CRYSTAL_INTERIOR_PACK, CRYSTAL_ORIGINAL_NPC_SOURCE, POKEGEAR_PC_RENDER_DIR and RUST_MIN_STACK=33554432.
