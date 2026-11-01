Hosted overworld regression checks use the real Rust server, fresh browser WASM,
and three independent authenticated Chrome profiles. They require the external
`content-packs/core-modular.browser.crystalpack`, npm dependencies, installed
Chrome, the WASM Rust target, and matching `wasm-bindgen-cli`.

Run `sh tools/overworld-multiplayer-smoke.sh`. Build output, server data,
verification saves, logs, and screenshots stay under ignored `target/`. The
script stops its server on completion. `MP_TEST_OUTPUT` and `MP_TEST_PORT` can
select a separate verification directory and localhost port.

The browser checks cover remote player identity and walking, public speech
bubbles following actual rendered sprites, private whisper/reply delivery with
a third player present, mobile touch movement/chat, held-key chat focus, all
invitation modes, decline/cancel, busy menus, completed Trade Center and Time
Capsule exchanges, a synchronized link battle, save/reload/reconnect, real door
warps, map-scoped public chat, and disconnect cleanup. Inputs use physical
keyboard/touch, the visible social UI, and the same production joypad bridge.

`hosted_overworld_fixture` generates test-only saves against the exact composed
pack served by that server. Production does not load or manufacture them.
Hosted chat and WebSocket messages have no time-based rate limits. Native tests
cover rapid chat bursts and 320 real WebSocket messages within one second.
Meshtastic is outside this hosted regression's scope.

A pack-backed mirrored-turn regression checks shared RNG advancement, identical
HP/PP from both perspectives, equal-speed ordering, and opposite final results:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/core-modular.browser.crystalpack" \
  cargo test --locked -p crystal-runtime --test hosted_link_battle -- --ignored
```
