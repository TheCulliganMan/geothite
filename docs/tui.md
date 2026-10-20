# Web renderer views

Use the always-visible **2D / 3D / TUI** chooser to swap renderers immediately.
All three views share the running Rust game: position, trainer, dialogue pages,
menus, battles, and saves remain intact. A view change never creates a second
controller, reloads the page, or submits a game input.

Direct links are `/?view=2d`, `/?view=3d`, and `/?view=tui`. `/tui` and `/tui/`
open the same client with the terminal renderer selected. In TUI mode, **Text**
and **Paint** switch between the original painted dot surface and text cells.
The standard keyboard, controller, touch, audio, and save controls work in all
three views. The selected view is remembered in this browser.

Rust produces the shared Ratatui buffer from the live production controller's
visible snapshot. JavaScript only paints that buffer and selects the renderer.
The standalone text client reuses the same renderer crate; see
[the native text-client guide](../modpacks/text-tui/README.md).

Validate a running deployment headlessly with:

```sh
node tools/browser-view-smoke.mjs https://geothite.ryanculligan.com /tmp/geothite-views.json
```

This opens the direct TUI URL, starts CHRIS's game through the production input
path, checks nine in-place renderer swaps, retains Start-menu selection, walks
into Mom's dialogue, verifies input ownership and dialogue advancement, and
checks the phone chooser. Reports and screenshots stay outside the repository.
On macOS the headless test uses Metal; other platforms use SwiftShader.
