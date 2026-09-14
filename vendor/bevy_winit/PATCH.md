# Browser event-loop ownership

This is Bevy's `bevy_winit` 0.14.2 crate, distributed under its included
MIT/Apache-2.0 licenses. The browser runner uses Winit's `spawn_app`, transferring
ownership of `WinitAppRunnerState` to the browser callbacks. The native runner
continues to use `run_app`.

The upstream `run_app(&mut runner_state)` browser path throws a JavaScript
exception while retaining references to a Rust stack frame. Starting Bevy from
the game's asynchronous pack loader and then calling other WASM exports can
invalidate that borrowed state. The observed failure is a missing
`Events<WindowResized>` resource followed by an unreachable trap and a blank
canvas.

The upstream Winit API documents `spawn_app` for owned browser event loops:
https://docs.rs/winit/0.30.13/wasm32-unknown-unknown/winit/platform/web/trait.EventLoopExtWebSys.html

Regression: serve `web-client/browser-startup.test.html` with the built client
and open `/browser-startup.test.html`. It must report two successful live
game-loop observations after browser startup.
