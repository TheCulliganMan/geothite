# Iron Range miner: accepted

See ACCEPTANCE.md and manifest.json. The planning notes below are superseded;
the accepted pack is minnesota-original-npcs-03-v01.crystalpack.

NPCs 1 and 2 are accepted. Use `content-packs/minnesota-original-npcs-02-v02.crystalpack` as the parent so Rowan and Oren remain present.

New single-sprite battle source: `battle-source-v2.png`, built-in imagegen, matching the original sturdy woman with short dark bob, hardhat/headlamp, overalls, hammer and rock specimen. Final intended palette is white/black, warm cream RGB555 21343 and rust red 6450. The converter now accepts optional LIGHT_RGB555 and MID_RGB555 arguments, so red overalls quantize to a separate ink instead of merging with skin.

Converted draft in ignored `target/npc-sprites/miner/battle-candidate-v1`. `battle-native-draft-v1.json` holds a deliberate native face/lamp refinement, with guarded edits in `battle-refinement-v1.json`. It has NOT been rendered or accepted yet. Source and prompt retained.

Next: author all six native 16x16 overworld views, keeping the rounded helmet, visible front/side lamp and dark bob. Back view must not show the front lamp. Use stocky red overalls and heavy boots, with stable foot registration and distinct walking phases. Combine the battle draft and six poses into the normal native-pixels-v1.json schema, encode with the validated --native compiler, inspect all outputs. Pick a reachable open Iron Range/Grand Rapids mining-town tile using actual parent collisions, add a unique MN_IRON_RANGE_MINER class and Minnesota-only registered party, publish one NPC, run the same compiled generic controller test with explicit source/pack/review directory and RUST_MIN_STACK=33554432. Require real win flag, save/load, follow-up and actual map/battle captures before NPC 4.

The accepted canoe-guide pack is `minnesota-original-npcs-02-v02.crystalpack`. Its final controller log is `target/npc-sprites/canoe-guide/controller-v02.log` (1 passed in 55.99s), and gameplay-v02 images were inspected after eye cleanup. Do not use its earlier v01 battle face for acceptance.
