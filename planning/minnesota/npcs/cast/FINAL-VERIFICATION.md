# Final trainer cast verification

Final pack: `content-packs/minnesota-trainer-cast-36-v01.crystalpack`.
SHA-256: `c7a16077f5c056f60ea248bde2446e704a4bd1154e97c9d2e9b10724ec644f17`.

## Passed

- All 36 character encounters rerun on this exact combined pack through production `VisibleShellController` inputs. Directional/stride art, dialogue ownership, move selection with PP mutation, battle save/load, legal victory, repeat dialogue and saved victory state checked. Eight leaders additionally assert the native badge and exactly one correct TM; four Elite Four assert League progress.
- Final Rust asset audit: 36 unique private art classes, 216 byte-exact native resources, 24 new trainer actors and 12 existing boss bindings. Existing trainer data changes only at the twelve intended class fields. Active Minnesota species and every reachable evolution checked. Original resources/audio, map terrain/events/scenes, old dialogue and all unrelated gameplay fields preserved. Raw scripts differ only at the selected trainer-class operands; split scripts and NPC payloads agree with compiled maps.
- Evidence audit: accepted JSON sources, dimensions, six distinct walking/idle poses, four-color RGB555 palettes, independent native tile roundtrips and acceptance file hashes. Original fifteen identity/party/dialogue unchanged.
- Actual final game battle/field contact sheets visually inspected. Authentic Crystal sprites compared at matching pixel scale, common palette and native 1×; own-palette and game captures reviewed separately.
- Production client built, then three fresh stdio-MCP processes exercised real Start/Pack navigation, movement, dialogue blocking and paging, autosave and fresh-process reload. All exited successfully with empty stderr and clean JSON-RPC stdout.
- Changed Rust leaves pass edition-2024 rustfmt. Focused crystal-assets all-target no-deps and crystal-bevy lib/tests no-deps Clippy pass, with the existing never_loop lint allowed. Focused asset preservation/stale-binding rejection regression passes.

## Verification scope

Controller fixtures place three legal level-60 Minnesota starters adjacent to each character and set explicit prior-access events/badges/League memory. Opponents, legal counterplay, native rewards and production scripts remain intact. These are local encounter/progression/save proofs, not intended-level balance tests or a continuous earned campaign. No Champion/Hall-of-Fame completion or six-scene rival integration is claimed.

The terminal play command is provided for interactive use. Automated rendering stayed headless; no visible window was launched. No deployment or active-pack promotion occurred.

## Existing check failures

Workspace `cargo fmt --all -- --check` reports pre-existing differences, beginning in offset-allocator and other unrelated files. Required full all-target Clippy attempts for crystal-assets and crystal-bevy stop on an existing crystal-core `bad_bit_mask` comparison that is always false. No unrelated formatting or gameplay lint repairs were made. Focused no-deps checks and the actual production build succeeded.

## Evidence

- `target/trainer-cast/final-audit/asset-audit.log`
- `target/trainer-cast/final-audit/evidence-audit.log`
- `target/trainer-cast/final-audit/controller-status.json` and per-character gameplay/log directories
- `target/trainer-cast/final-client/*-transcript.json`, `*-stderr.log`
- `target/trainer-cast/build-production-client.log`
- `target/trainer-cast/clippy-assets-all.log`, `clippy-assets-focused.log`, `clippy-bevy-all.log`, `clippy-bevy-focused.log`
- `target/trainer-cast/fmt-workspace.log`, `fmt-own-leaves.log`

Encoder verification.json intentionally reports its own manual/runtime stages as pending: visual/game acceptance belongs to each character manifest and this final combined proof, rather than being fabricated by the encoder.
