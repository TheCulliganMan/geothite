# MECHANIC TOBIN: accepted native and playable

Deliverable: `content-packs/minnesota-original-npcs-10-v01.crystalpack`. Parent: `content-packs/minnesota-original-npcs-09-v01.crystalpack`. Existing trainers and runtime assets passed preservation checks. Placement: MnDuluth [54, 59], facing south, approach [54, 60].

Six native 16x16 overworld poses and one 56x56 battle sprite share an independent four-color GBC palette. Source: `native-pixels-v1.json`. Placement: `placement-v1.json`. PNG/palette/tile/alpha agreement, nonblank poses, distinct walking pairs and independent tile decode passed.

The production controller test passed exactly once, 0 failures. It checked native art loaders and visible palette, introduction/input ownership, a PP-spending move, battle save/load, a complete legal-input win, defeat flag before render capture, and follow-up talk. Log: `target/npc-sprites/mechanic/controller-v02.log`. Actual game captures: `target/npc-sprites/mechanic/gameplay-v02`. These images were manually inspected before this record was written: Reviewed native-size/enlarged six-pose strips, native battle grid, actual intro and post-win Duluth capture. Rebuilt independent goggle lenses and separated eyes/beard; stocky slate overalls, rolled shirt, tool belt, open wrench jaw and broad boots match across views. Rejected the first blackout run; the second passed using unchanged teams and legal defensive switching through the production PKMN menu.

Evidence hashes are in `manifest.json`. This record accepts this NPC only; the goal still requires at least 15. Continue one at a time using this pack as the next parent. The generic test runs with explicit CRYSTAL_INTERIOR_PACK, CRYSTAL_ORIGINAL_NPC_SOURCE, POKEGEAR_PC_RENDER_DIR and RUST_MIN_STACK=33554432.
