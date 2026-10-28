# Technical verification of the superseded ImageGen proposal

These checks prove conversion and storage correctness. They do **not** establish
user style acceptance. The user rejected large illustrated booths and requested
compact native tile-based construction with separate NPC vendors.

- Seven focused Rust tests passed, including reciprocal seams, exterior clipping,
  source-backed area sampling, alpha, foreground palettes and full base preservation.
- Independent packed-bitplane checks passed for all seven appended blocks.
  All 128 original Johto blocks match at morning/day/night. Every base runtime
  file and unrelated game-data field was preserved.
- Final proposal pack SHA-256:
  `85e44830650a9e13def59e1570964a0aec4a56e74d72035772644a5e9df5a493`.
- Rust desktop/audio verification passed: 544 maps, 251 species, 251 moves,
  377 audio programs; every PCM hash, frame count and loop range matches.
- A disposable gallery retained all campaign maps and runtime files. Native
  headless captures were made with the updated executable at
  `target/northstar-story/wasm-dev/examples/render_at_location`.
  The older runtime-fps executable predates foreground-color support; its
  brown-palette captures remain retained as rejected verification history.
- Every foreground pixel of all four objects matches an actual native capture:
  2,296 source pixels / 20,664 displayed pixels at 3× scale. Proof is retained in
  `planning/minnesota/renders/tiles/state-fair-v5/verification/native-pixel-proof.json`.
- Changed Rust files pass rustfmt. Normal all-target Clippy stops at the existing
  `bad_bit_mask` error in `crystal-core/src/multiplayer/mod.rs:1332`. All-target
  Clippy passes with the previously documented `bad_bit_mask` and `never_loop`
  allowances; no warnings refer to the changed tile importer or gallery example.
  Global formatting still reports unrelated existing differences.

All original images, atlas, prompts and conversion recipes are retained here.
All packs, native cutouts, captures and verification logs stay ignored under
`content-packs/` and `planning/minnesota/renders/tiles/state-fair-v1..v5/`.

The active fair-map implementation must follow the user's subsequent native
counter / native NPC direction, rather than treating this gallery as approval.
