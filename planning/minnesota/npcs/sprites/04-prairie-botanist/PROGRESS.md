# Prairie botanist: accepted

Current acceptance is recorded in ACCEPTANCE.md and manifest.json. The accepted
pack is minnesota-original-npcs-04-v01.crystalpack. The following notes describe
earlier rejected work and are superseded by that acceptance.

Three NPCs are accepted. Use content-packs/minnesota-original-npcs-03-v01.crystalpack as parent. Do not start NPC 5 until this one passes.

New single-character battle source and built-in ImageGen prompt are saved as battle-source-v2.png and battle-prompt-v2.txt, based on the original elder reference sheet. Intended native palette is white/black, pale cream RGB555 23454, olive green RGB555 6632. The converted battle candidate is in ignored target/npc-sprites/botanist/battle-candidate-v1; it still needs visual review and deliberate refinement, especially preserving round glasses and light hair.

First native battle conversion was inspected and REJECTED: lens outlines largely disappeared, and pale hair/face highlights quantized to transparent ink zero, leaving holes. Rebuild the native head deliberately (light hair, closed contour, readable round glasses with visible cream lenses) or revise the source before acceptance. Keep background negative spaces around arms and flower transparent; do not flood-fill all exterior paper indiscriminately. Body/coat/flower/notebook silhouette is useful source, but this candidate is not a finished sprite.

Next: author six native 16x16 poses with fluffy cream hair, black-rim glasses, a green field coat, specimen bag/notebook and a small flower. Keep stable floor alignment and distinct walking phases; side profile should stoop slightly, with flower held in front. Back view must not show front glasses. Combine native battle rows and overworld frames into native-pixels-v1.json, encode via npc_sprite_candidate --native, inspect at 1x and enlarged. Mankato is a candidate prairie-town placement; inspect actual parent collisions and actors before choosing a free reachable tile. Use a unique class such as MN_PRAIRIE_BOTANIST, an explicit native behavior template, and a party from the actual registered Minnesota roster.

Publish using target/debug/pack_original_npc, preserving the three accepted NPCs. The generic controller executable is target/debug/deps/crystal_bevy-0e4f79a33ebb6d13, filter original_npc_controller_talk_battle_turn_and_save --ignored --nocapture. Set RUST_MIN_STACK=33554432, CRYSTAL_INTERIOR_PACK to the new pack, CRYSTAL_ORIGINAL_NPC_SOURCE to the botanist placement, POKEGEAR_PC_RENDER_DIR to its own ignored output folder. It uses legal type-aware moves, checks a full win flag before any render capture, and writes intro/first-turn/overworld PNGs. Require one actual passing test AND manual game-image review before calling record_acceptance.py. Do not recompile the engine for data-only additions.
