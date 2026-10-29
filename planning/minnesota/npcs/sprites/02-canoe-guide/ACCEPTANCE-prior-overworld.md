# Canoe guide accepted: NPC 2 of 15

CANOE GUIDE OREN stands near Brainerd's lake launch at (42,21), facing south; approach (42,22). The final candidate is `content-packs/minnesota-original-npcs-02-v02.crystalpack`, built on the accepted ranger pack. Existing trainers and runtime assets, including Rowan, passed preservation checks. Oren uses registered native Birchit and Rivvit.

Six native 16x16 overworld poses and one 56x56 battle sprite share an independent four-color GBC palette. Reviewed source and native renders; refined paddle grip/blade and stepping feet. The first playable battle capture exposed merged eyes/hat shadow, so it was rejected for visual acceptance. Guarded face-pixel refinement separates those details in the final sprite.

The final production controller run passed 1 test, 0 failures, 55.99s: art loaders/palette, talk and dialogue input ownership, PP-spending move, battle save/load, a real legal-input win, defeat flag checked before render capture, and follow-up talk. Log: `target/npc-sprites/canoe-guide/controller-v02.log`. Reviewed the actual trainer intro and post-win world PNGs in `target/npc-sprites/canoe-guide/gameplay-v02/`; face, life vest, shorts, paddle and native scale are clear and consistent. Source/evidence hashes are in `manifest.json`.

Repeat the generic compiled test with RUST_MIN_STACK=33554432, CRYSTAL_INTERIOR_PACK pointing to the final pack, CRYSTAL_ORIGINAL_NPC_SOURCE pointing to this placement-v1.json, and POKEGEAR_PC_RENDER_DIR set to a fresh ignored folder. No further Cargo rebuild is needed while relevant code remains unchanged.

Full goal remains open: continue one at a time with the Iron Range miner, using this accepted two-NPC pack as parent.
