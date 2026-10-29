# Superseded: mechanic accepted

The fresh controller run passed, 1 test, 0 failures, with unchanged teams and legal defensive switching. Actual battle intro and post-win Duluth capture were reviewed. See ACCEPTANCE.md and manifest.json.

# Harbor mechanic: candidate, not accepted

TOBIN has six native overworld poses and a matched battle sprite, using the four-color cream/slate palette in `native-pixels-v1.json`. The native review repaired the two goggle lenses and face. Source art was generated with the built-in image tool; its exact prompt is preserved beside it.

Candidate pack: `content-packs/minnesota-original-npcs-10-v01.crystalpack`, built from the accepted NPC 09 pack. Placement: Duluth [54,59], approach [54,60]. Native encoding and pack preservation checks passed.

The first controller run failed the victory gate: the original three-Pokemon fixture blacked out against the inherited scaled team. It did not set the defeated flag. See `target/npc-sprites/mechanic/controller-v01.log`. The run is rejected; its captures are diagnostic evidence only.

The driver previously switched only after fainting. A test-only improvement now selects a healthier type matchup through the production PKMN menu before continuing normal moves. Neither the authored opponent nor fixture party has been changed, and the victory assertion is retained. Rebuild and a fresh complete controller run are required before acceptance. Do not advance to NPC 11 yet.

The rebuild was started with `cargo test --locked -p crystal-bevy --config profile.dev.package.crystal-assets.opt-level=0 --config profile.dev.package.crystal-bevy.opt-level=0 --lib original_npc_controller_talk_battle_turn_and_save --no-run`. Log: `target/npc-sprites/compile-defensive-switch-v1.log`. Execution session 21253 was confirmed live compiling crystal-bevy; poll it rather than starting another build. The edited test file passes rustfmt check. Workspace-wide formatting still reports unrelated differences; see `target/npc-sprites/fmt-defensive-switch-v1.log`.
