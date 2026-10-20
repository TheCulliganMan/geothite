# Natural movement

Natural movement is the default whenever the 3D view is active, including
existing packs that omit `natural_movement`. 2D and text clients keep their
standard controls.

The left stick supplies continuous, camera-relative movement with a radial
deadzone and proportional walking speed. Arrow keys and the D-pad support
map-relative diagonals. Release movement to stop between gameplay cells;
reverse or turn without waiting for a tile animation to finish. Hold B to run
when the pack enables running shoes. Solid terrain and NPCs block movement,
with clearance at walls and corners.

The visible shell owns the fractional position. Every cell crossing still
passes through its existing input loop, so collision, encounters, trainers,
warps, and coordinate scripts retain their normal authority. Scripted movement,
ledges, downhill bike sections, and map transitions retain their authored presentation. A saved game
restores its gameplay cell; the fractional offset is session-local.

The Rust builder adds `natural_movement: true` and the `natural-movement`
manifest identity to an existing verified pack. It preserves game content,
runtime files, and the complete audio manifest. Generated packs stay ignored.

```sh
cargo run --locked -p crystal-assets --bin pack_natural_movement -- \
  . content-packs/core-modular.crystalpack content-packs/natural-movement.crystalpack
cargo run --locked -p crystal-assets --bin pack_natural_movement -- \
  . content-packs/core-modular.browser.crystalpack content-packs/natural-movement.browser.crystalpack
```

Both outputs were built and verified against their corresponding base packs,
including identical audio payloads, PCM hashes, frame counts, loop ranges,
runtime files, and unrelated game data. The native pack was also exercised by
the production renderer with offscreen static and walking captures.

Set `natural_movement` to `false` in the compiled dataset before compiling a
custom pack to retain tile stepping in 3D. This setting has no effect on 2D.
