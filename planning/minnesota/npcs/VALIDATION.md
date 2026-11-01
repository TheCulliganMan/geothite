# NPC pipeline validation

Acceptance requires the native source, pack, controller result and manual game captures for each individual NPC. The roster README and per-NPC ACCEPTANCE.md/manifest.json identify the accepted versions. Unaccepted candidates and diagnostic captures do not count.

The generic production controller regression is `original_npc_controller_talk_battle_turn_and_save` in `crates/crystal-bevy/src/bevy_shell/tests/original_ranger.rs`. It materializes the actual pack, checks production palette/art loaders, talks to the placed trainer, checks dialogue input ownership, spends move PP, saves/reloads a battle, wins through legal inputs, checks the defeated flag before capture and verifies follow-up conversation. Fixture setup uses three legal level-60 Minnesota starters. The fixture does not weaken opponents or disable inherited scaling. A legal defensive switch through the PKMN menu was added after the mechanic's first test party blacked out; its unchanged encounter then passed.

The current executable is `target/debug/deps/crystal_bevy-0e4f79a33ebb6d13`. Run it with `RUST_MIN_STACK=33554432`, the absolute accepted/candidate pack in `CRYSTAL_INTERIOR_PACK`, the single placement source in `CRYSTAL_ORIGINAL_NPC_SOURCE`, and a fresh per-version directory in `POKEGEAR_PC_RENDER_DIR`. Captures are actual production render compositions, offscreen.

Rust checks for the test helper:

- Edited file passes rustfmt check. Workspace-wide `cargo fmt --all -- --check` reports unrelated existing differences, starting in offset-allocator. Log: `target/npc-sprites/fmt-defensive-switch-v1.log`.
- Normal `cargo clippy --locked -p crystal-bevy --all-targets` fails in existing `crystal-core/src/multiplayer/mod.rs:1332` (`bad_bit_mask`). Log: `target/npc-sprites/clippy-defensive-switch-v1.log`.
- `--no-deps` also reaches the existing `crystal-bevy/src/bevy_shell/title_menu.rs:8661` `never_loop` error. Log: `target/npc-sprites/clippy-defensive-switch-focused-v1.log`.
- The focused `--lib --tests --no-deps -- -A clippy::never_loop` check exits successfully, with unrelated existing warnings and no diagnostics for original_ranger.rs. Both pixel-iteration warnings introduced by this task were fixed with `as_chunks::<4>().0.contains(...)`. Final log: `target/npc-sprites/clippy-defensive-switch-focused-v3.log`. This is a scoped check with the named unrelated lint disabled, not a claim that global Clippy passes.

The final pixel comparison cleanup compiled successfully in `target/npc-sprites/compile-pixel-contains-v1.log`; fresh baker/ranger/guide/miner hand-edit encounters passed using that executable. Before final delivery, build and smoke-test the production client against the final combined pack, and audit all 15 accepted source/art/gameplay records. User rejection of the earlier overworld quality remains authoritative; follow OVERWORLD-REWORK.md before final completion.

## Existing-NPC overworld revision

`pack_original_npc --revise-overworld BASE PLACEMENT NATIVE_ART_ROOT OUTPUT` replaces exactly one original NPC's overworld PNG and 2bpp bank. It requires a registered single-owner class, matching existing actor, exact native bundle and valid palettes/tiles, and rejects changes to battle art or palettes. Readback verifies unchanged gameplay data, audio and every unrelated runtime file.

- The public API is `revise_original_npc_overworld`. Focused external-pack test `original_npc_overworld_revision_guards` passed: exact two-file preservation, different valid identity, no-op rejection, unknown class rejection, incomplete bundle rejection and rejection of valid-but-changed battle PNG/tiles. Log: `target/npc-sprites/test-overworld-revision-guards-v2.log`.
- Test fixtures use `CRYSTAL_NPC_REVISION_PARENT`, `CRYSTAL_NPC_REVISION_SOURCE` and `CRYSTAL_NPC_REVISION_ART`. The executed parent was NPC 12 v04, source was the ranger's placement-v2.json, and art was ranger/hand-v1. Run with RUST_MIN_STACK=33554432 and the established crystal-assets dev opt-level=0 config.
- Build passed in `target/npc-sprites/compile-overworld-revision-v2.log`; all directly edited Rust leaf files pass rustfmt check. The workspace check still reports unrelated formatting differences. The module declaration order introduced by this task was corrected in place.
- Normal crystal-assets `--all-targets` Clippy remains blocked by the existing multiplayer bad_bit_mask error. `--no-deps` reaches 11 existing never_loop errors in mutation_protocol.rs. The focused `--lib --tests --bin pack_original_npc --no-deps -- -A clippy::never_loop` check passes with unrelated existing warnings and no diagnostics in the NPC updater, CLI or regression module. Logs: `clippy-overworld-revision-all-v1.log` and `clippy-overworld-revision-focused-v1/v2.log` under target/npc-sprites. No global lint suppression was added to source.

## Legal counterplay in the NPC encounter driver

Lark's first three reference-pass runs blacked out (`target/npc-sprites/photo/controller-reference-v01/v02/v03.log`). The actual diagnostic battle save showed two scaled REVERLOON with SHADOW_BALL, PERISH_SONG, HYDRO_PUMP and DESTINY_BOND. The test driver now chooses equally safe partners by available immediate attack/HP, penalizes charge moves, switches via PKMN before an active Perish countdown expires and chooses a legal safe turn after announced Destiny Bond. Fixture teams, levels, DVs, opponent data and production battle rules are unchanged. This is test input selection, not a gameplay patch.

Opt-in `CRYSTAL_NPC_BATTLE_TRACE` prints at most 80 changed HP/Perish/last-move snapshots in the ignored test. Default runs emit no added diagnostics. `photo/controller-reference-v04.log` passed in 9.48s; its trace shows a safe turn after Destiny Bond, the opponent changing to Shadow Ball, and victory with a living party member. `mechanic/controller-counterplay-v01.log` passed the unchanged scaled mechanic regression in 13.68s. Actual Ely capture was reviewed. Earlier failed logs remain evidence; do not report them as passes.

- Final driver compiled in `target/npc-sprites/compile-perish-counterplay-v1.log`. Edited leaf file passes rustfmt check.
- Workspace `cargo fmt --all -- --check` still fails in unrelated existing code, starting in offset-allocator (`fmt-offensive-tiebreak-v1.log`).
- Normal `cargo clippy --locked -p crystal-bevy --all-targets` still fails at existing multiplayer/mod.rs:1332 bad_bit_mask (`clippy-counterplay-all-v1.log`).
- Focused `--lib --tests --no-deps -- -A clippy::never_loop` passes with unrelated existing warnings and no diagnostics in original_ranger.rs (`clippy-counterplay-focused-v1.log`). No source lint suppression added.


## Final delivery

Final pack: content-packs/minnesota-original-npcs-15-v01.crystalpack. All 15 full encounters passed sequentially on this exact pack with the final counterplay-capable production-controller test. Logs/captures are under target/npc-sprites/final-audit; final-encounters.json indexes results. The standalone production geothite client built and passed fresh-game Start/Pack menus, real movement, NPC dialogue ownership, clean JSON-RPC and save/reload on this pack. All 15 registrations, 90 native art resources and active native evolution families were audited; original trainers/runtime resources/audio/map terrain/events/actors/scripts and every unaffected gameplay field match minnesota-trainers-v02. See FINAL-VERIFICATION.md and final-audit/verification.json for the completed audit. Existing global formatting/Clippy limits above remain unrelated and documented.
