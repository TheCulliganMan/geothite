# Repository rules

- The browser TUI uses pinned `ascii.rest` 0.2.1 from bas3line/ascii to paint
  the Rust Ratatui buffer as a custom colored canvas piece. This is a painter,
  not a second game engine. Keep its frame rate zero. Inputs/resize render the
  shared Rust buffer; ambient dot animation only updates cached Rust-produced
  radii on a visual timer, never the controller or requestAnimationFrame.
  Reduced-motion
  users must still see every game-state update. Keep the semantic terminal and
  WebMCP snapshots available; canvas pixels are not the accessibility/tool API.
  Build with `sh tools/tui-build.sh` (Node/npm plus wasm-bindgen required).
  The lockfile pins the painter; npm scripts are disabled. Its JS and MIT
  license are copied only into ignored build output. Docker performs the same
  install in a Node build stage. Include painter and adapter in the bundle hash.

- Keep implementations in this Geothite repository. Audio synthesis and export
  must use Rust. Only minimal browser integration JavaScript belongs in the
  website; do not introduce TypeScript or an external exporter dependency.
- Do not add pret disassembly, ROMs, raw or exported command dumps, duplicate
  audio source catalogs, generated PCM files, or new compiled artifacts to this
  repository or its shipped bundle. Game content packs must remain external and ignored, including regenerated
  packs. Never commit them or restore them from old Git history.
- Read audio programs from the user-supplied local content pack. Do not extract
  and commit another copy of those programs or add generated audit manifests.
- Build products and temporary verification output are not deliverables to
  commit. Keep them ignored and remove task-generated scratch artifacts when
  finished. Never stage them as part of an “all code” commit.
- Verify audio changes against the pack's PCM hashes, frame counts, and loop
  ranges. Preserve unaffected audio and unrelated game content.

## Crystalpack build and local runbook

- Painted is the default across every TUI: native/curl, browser desktop and
  browser mobile. Text is an explicit presentation toggle (or tiny-terminal
  fallback), not a second default. Start directly in the game, without launch
  chrome or successful-installer logs. Keep explicit user view choices and
  saves intact. A TUI deployment must verify both the public `/tui` WASM and
  every native target listed by `current.txt`; deploying one does not deploy
  the other. Additive browser releases must retain the silent installer,
  immutable native downloads, existing pack, graphical assets and save data.

### One-command terminal install

- `curl -fsSL https://geothite.ryanculligan.com/install.sh | sh` downloads a
  prebuilt native client and the existing external hosted pack, verifies SHA256,
  and reconnects stdin to `/dev/tty` before launching. Never compile on the
  user's machine or ship another pack/catalog inside the native download.
- Successful curl installs and reinstalls are silent: no download logs, install
  paths, progress bars or control banners before the painted game. Keep real
  failures readable on stderr; do not hide errors with blanket redirection.
  Require the actual PTY stream to start with the alternate-screen transition
  and require `--no-play` to emit zero bytes on success.
- `sh -s -- --no-play` installs without opening the game. Installation is
  user-local under `${XDG_DATA_HOME:-$HOME/.local/share}/geothite`; the launcher
  is `bin/geothite` there (supports `play`, `mcp`, and `dump`). Saves are keyed
  by pack checksum and retained across reinstalls. Do not edit shell startup
  files, require sudo, overwrite another `geothite` command, or delete saves.
- Native default themes must be compiled in, not loaded from
  `CARGO_MANIFEST_DIR` on the build machine. Explicit `--theme` remains supported.
  Curl-installed `play` must default to the pack-backed painted renderer too,
  not just local builds or the browser. Keep V text/paint switching available;
  standard 80x24 terminals must paint; below 40x24 retain the readable text
  fallback. Require painted
  pack-backed dot art and both V transitions in the actual downloaded PTY smoke
  before publishing a native release. Updating source alone does not update
  the immutable binaries referenced by the public installer manifest.
  Keep Crossterm's `use-dev-tty` portable poller: macOS's default kqueue source
  failed to initialize after the curl installer reopened the terminal. A frame
  painted successfully before that failure; require actual keys and saves in
  `node tools/terminal-curl-smoke.mjs`, not just startup pixels.
- The installer smoke must also resume the saved game through the installed
  `geothite mcp` launcher and prove movement and Start. The bedroom callback
  can retain `ToggleMaptileDecorations` as an `active_menu` visual fence in a
  save even though its decoration changes are already applied. The shared
  save-restoration controller completes that specific non-window fence through
  `close_active_menu`; never replay the decoration callback or blanket-clear
  genuine menus/scripts. Keep `installed_save_resumes_real_movement` passing.
- Build native targets, then use `tools/terminal-package.sh BINARY TARGET OUTPUT`
  to gzip each executable. Publish an immutable release directory containing
  `TARGET.gz` and `tools/terminal-play.sh` as `play.sh`. `current.txt` contains
  `release SLUG`, `TARGET SHA256`, `launcher SHA256`, and `pack SHA256` lines.
  Publish the manifest with its corresponding downloads atomically. Only list
  built targets; never claim unsupported systems work. Linux GNU builds require
  compatible glibc and shared libraries; Windows users need WSL.
- Downloads and deployment contexts stay ignored under `target/`. The additive
  `tools/terminal-overlay.Dockerfile` preserves the live server, browser clients,
  and externally supplied pack. Fresh Docker builds copy the installer; retain
  published native downloads when replacing the web image. Verify the actual
  curl pipeline in a PTY, actual downloaded MCP/controller gameplay outside the
  checkout, resume/reinstall preservation, and rejection of corrupt checksums.

### Browser WASM TUI at `/tui`

- `geothite`'s library compiles to `wasm32-unknown-unknown`. The browser uses
  `BrowserTui` and the same `VisibleShellController` as the native terminal.
  The selectable text view uses `TerminalUi::draw`; painted is the browser
  default on desktop and mobile. Rust renders Ratatui cells into colored HTML;
  browser JavaScript only loads assets, maps inputs, measures cells and paints.
  Do not replace this with a separate JavaScript game or menu dispatcher.
- Build with `sh tools/tui-build.sh`. Output lives in ignored `target/tui-web/`.
  The script copies the existing local browser pack into that ignored preview
  directory when present. Serve it with
  `python3 -m http.server 8080 --directory target/tui-web`, then open
  `http://localhost:8080/tui/`. Never commit generated WASM, glue, packs or saves.
- The production Dockerfile builds and content-addresses the TUI bundle and
  the Rust web server serves both `/tui` and `/tui/`. The TUI loads the existing
  `/realtime-clock.browser.crystalpack` endpoint; it does not need a second
  hosted pack. Browser saves use `saves/{modpack-id}-tui-local.crystalsave`,
  isolated from the graphical client's saves. Do not silently overwrite them.
- `window.geothiteTui.observe()` and `.press(button)` drive the current browser
  session. In WebMCP-capable browsers, `geothite_tui_observe` and
  `geothite_tui_press` register on `document.modelContext`. Never fake WebMCP
  support or expose engine internals through these tools. Native stdio MCP
  remains `./target/release/geothite mcp content-packs/text-tui.crystalpack`.
- `/tui` must auto-open the identity-matched save or start CHRIS; do not add a
  website launcher, toolbar, or separate controls around the terminal. F5 and
  the production Start menu save. Touch gestures stay on the TUI itself.
  Mobile-only Game Boy buttons are allowed inside the terminal's reserved
  control area. Keep all eight buttons on `BrowserTui::press`; no second
  dispatcher or synthetic menu state. Game Boy buttons stay mobile-only; the
  presentation toggle is available on both desktop and mobile. Targets must be at
  least 44px and remain on-screen in 320px portrait and phone landscape.
  Gate buttons on touch capability, not viewport width alone. A narrow
  non-touchscreen desktop must never acquire arrow/Game Boy buttons.
- Painted illustration lives in Rust `crystal-tui/src/painted.rs` and is the
  browser default on BOTH desktop and mobile. Provide a TEXT/PAINT toggle and
  V shortcut; switching views must not send gameplay inputs or reset the save.
  Keep the traditional `TerminalUi::draw` as the selectable text view, not as
  the desktop default. Use the pack's metatile art, collision table and actual
  species front/back PNGs; never ship another extracted catalog. Exported
  Pokémon PNGs are already palette-colored: do not infer grayscale from red
  or tint all species gold. Battles need the active Pokémon's BACK sprite,
  opponent's FRONT sprite, and live HP bars/numbers with low-health colors.
  Art must retain explicit blockers/ledges and the real player/NPC positions.
  Keep observed map bounds equal to painted map bounds. Prioritize dialogue
  and selected menu items on small screens, including landscape.
- Overworld art dithers the REAL pack tiles and composited player/NPC sprites
  with ordered halftone dots (` ·•●`). Preserve the current map/time-of-day
  palette and actual silhouettes. Alpine Dawn is a dithering reference, NOT a
  replacement color scheme or scenery generator. Do not infer roofs/forests
  from color thresholds, replace tiles with procedural material fields, or
  replace actors with generic figures. Those approaches lost the map's art.
  Rust `ink.rs` area-samples pack pixels into same-hue dot ink on flat paper;
  Bayer thresholds choose dot coverage, not colored rectangular backgrounds.
  Ink must keep source brightness; normalizing every shade's peak to 255 made
  Home's brown/tan shadows look neon yellow. The shared ink gently reduces
  chroma by neutral mixing, retaining hue families, source silhouettes and the
  authored indoor/time-of-day palette. Do not apply this to battle portraits.
  Browser art may use a denser square-cell canvas piece composited inside the
  same TUI, while captions and menus remain large. Painted maps must not have
  floating #/D/@ labels: real tiles/sprites carry the scene; ledge/barrier cues
  are dots. Keep explicit collision labels in text view and MCP snapshots.
  Both buffers come from Rust; minimal JS only paints them. Fine art must share
  the semantic camera bounds, never add a second map or invented scenery.
  Native terminals dither the same pack pixels at their available resolution.
  The native renderer consumes the SAME fine dot field and Rust-produced radii
  as the browser, not a second tile renderer. Ghostty/Kitty paint those circles
  through a Rust graphics transport (local shared RGB or direct PNG); other terminals pack the fine samples
  into 2×4 Unicode Braille cells. Keep menu/caption cells uncompressed and the
  observed camera bounds identical. `GEOTHITE_TUI_GRAPHICS=off` selects the
  portable adapter; `kitty` explicitly enables the graphics protocol. Auto
  detection must fall back in unknown terminals and multiplexers. No graphics
  transport or cosmetic timer may send gameplay inputs.
  Fullscreen owns graphics output too: quiet chunked payloads (4096-byte
  maximum, q=2, cursor preserved), two bounded image IDs, and deletion of ONLY
  owned images on text/battle/exit. Cache unchanged frames for reduced motion.
  Test with `node tools/tui-terminal-canvas-smoke.mjs` and
  `GEOTHITE_TUI_GRAPHICS=off node tools/tui-terminal-smoke.mjs` after rebuilding
  the release executable. The canvas smoke checks real executable/PTY PNG
  bytes, animation, reduced motion, V cleanup and production Start. It is NOT
  proof of a GUI terminal screenshot. Review actual terminal rendering when
  app access or a user screenshot is available; never claim that payload or
  headless buffer screenshots visually verify Ghostty.
  Native cosmetic animation uses monotonic deadlines, NOT the absence of
  keyboard events: held keys, resize events and unmapped keys must not starve
  dot breathing. Test both PNG canvas and portable Braille updates under
  continuous input and reduced motion. Rebuilding/reinstalling does not update
  a running executable; save and restart that session to pick up a new build.
  When a user reports a static window after an update, check the running
  process's launch time against the build before claiming the fix is visible.
  Never close their session or assume it can save: a local `play` invocation
  without `--save` has no configured save destination. Test a new client in a
  separate terminal instead. `TUI_CAPTURE_ANIMATION=1` on the canvas smoke
  records its actual emitted PNG sequence and timing under the ignored art
  output directory for visual review; changed hashes alone do not establish
  perceptible motion or that a GUI terminal displays those frames.
  `node tools/tui-terminal-animation-smoke.mjs` checks actual Braille writes
  while idle and under repeated unmapped keys; cursor-only traffic is not
  animation. Discard the trailing idle redraw before measuring busy input so
  the old idle-timeout-only implementation cannot falsely pass.
  Browser redraws must retain their pending cosmetic deadline too: cancelling
  and rescheduling on every input/resize freezes ambient ink during play. Both
  frontends pass elapsed seconds into the shared radius field, with bounded
  resume deltas. `tools/tui-motion-checks.mjs` measures actual dot-layer pixel
  changes over two seconds, both idle and during repeated R redraws, and requires
  unchanged authoritative observations. Sample the current offscreen canvas,
  not a detached canvas replaced by draw/preference changes. Reduced motion
  must remain still. Hash inequality alone accepted motion too subtle to see.
  Keep the browser's ambient motion gentle AND continuous: an approximately eight-second
  constant-speed travelling wave at 30 visual updates/second. Native idle art
  refreshes ONCE PER SECOND at the user's request; keys, movement, dialogue,
  menus, resize and V still redraw immediately, not behind a one-second wait.
  Keep finite battle replay timing separate. Preserve the same elapsed-time
  wave by subdividing the terminal's one-second cosmetic delta through the
  shared 250ms resume guard, without rendering intermediate frames or ticking
  gameplay. Native PTY checks require roughly 1fps idle art even during held
  unmapped input, plus responsive real Start/menu inputs. Pass fractional
  Rust radii through WASM and native adapters, not byte-quantized sizes. Stay
  below the radius cap; clipped peaks and sine easing created long holds then
  quick changes even when two-second pixel comparisons passed. Check every
  actual browser canvas compose across a full cycle for nonzero small changes
  and bounded frame gaps, plus the native one-second cadence. Regression tests bound radius
  range and per-update jumps; do not speed the wave up just to meet a short
  observation window. Preserve the current softened
  source colors and keep gameplay clocks independent of this cosmetic field.
  Native input polling must use at least 1ms even after a visual deadline
  overrun: Crossterm's use-dev-tty source skips reads with a zero timeout.
  Large/slow PNG frames otherwise animate while all keys appear ignored.
  Local Ghostty/Kitty use two private POSIX shared-memory RGB slots (`t=s`)
  instead of encoding and flooding the PTY with a PNG every cosmetic frame.
  Never overwrite an unread slot: discard stale cosmetic work under consumer
  backpressure, keep input polling, and clean only owned objects on exit.
  SSH/unknown clients retain quiet, compressed, chunked direct PNG transfer;
  `GEOTHITE_TUI_TRANSPORT=direct` explicitly selects it. If a local terminal
  never consumes shared memory, fall back to direct transfer. Shared memory
  must be sized and mmap'ed (read/write on shm descriptors fails on macOS).
  Cache the authoritative snapshot and fine pack scene until inputs/layout/
  replay change. Cosmetic frames update only the cached radii. Do not repaint
  hidden Braille under a graphics image or redraw on every unmapped key.
  Run `GEOTHITE_TUI_TRANSPORT=shared TUI_PERF_ASSERT=1 node
  tools/tui-terminal-performance-smoke.mjs` at the large-window test size,
  plus canvas smoke with BOTH shared and direct transports. Require continuous
  motion, zero hidden Braille traffic and responsive actual Start inputs;
  frame hashes alone missed a 42ms/frame encoder and 12 MB/s PTY bottleneck.
  The test-only `tui_shared_memory_consumer.py` reads/unlinks actual RGB slots
  like a protocol receiver; it is not a game renderer or Ghostty GUI proof.
  The real PTY smoke must resize then open Start and complete a battle; read
  its reconstructed character screen (`tui-pty-screen.mjs`), not concatenated
  Ratatui diffs or base64 graphics as if they were semantic text.
  V selects text/paint. Keep the map flat and north-up. The experimental 2.5D
  offset/compression pass and its P toggle were rejected and removed: do not
  bring back a diamond rotation or fake depth offsets without a new user request
  and a convincing camera/geometry treatment. Dot size data comes from Rust.
  Dot sizes vary continuously across world coordinates, not metatile IDs, so
  reused tiles do not repeat the exact halftone. Ambient breathing changes
  cached radii only; pause it in hidden tabs and honor reduced motion. Native
  circles or Braille density vary through the same world-space field. Never animate actor
  positions, trigger scripts, wait on audio, or advance gameplay from this clock.
  Keep texture seeded in world/tile
  coordinates, never viewport coordinates: walking must not make it swim.
  https://ascii.rest/alpine-dawn/ is the direct art reference; Obra Dinn's
  stable-dither discussion informs temporal coherence, not asset copying.
  Preserve colored ASCII battle portraits, not a generic gold tint. The canvas
  painter remains fps zero; ambient dot radii and finite, read-only battle
  replays may update on visual timers. Never advance gameplay from those timers.
  Native `play` uses the same painter by default (V toggles text); read art
  directly from the verified pack in memory, never the build checkout or a
  newly extracted source catalog. Tiny terminals fall back to text.
  Keep collision/warp badges and facing indicators readable over artwork;
  missing NPC art must still expose the actual blocking NPC, not erase it.
  For broad art evaluation generate ignored saves with
  `cargo run --locked -p geothite --example browser_overworld_fixtures --
  content-packs/realtime-clock.browser.crystalpack target/tui-art-eval/fixtures`
  and set `TUI_EVAL_OVERWORLD_FIXTURES=target/tui-art-eval/fixtures` when running
  the visual evaluator. It checks ten additional real towns/interiors/forest/
  cave/ice/coast/tower scenes at every size. These are verification-only saves;
  production must never import or manufacture them.
- Visual verification uses `tools/tui-art-eval.mjs` for real game scenes at six
  sizes, then `tools/tui-art-compare.mjs` for side-by-side comparisons, both
  desktop/mobile frame bounds, view toggle and mobile 44px-target checks. Do
  NOT gate new desktop art on unchanged desktop pixels. Use actual
  WASM and an external fixture, assert its Route29 position and battle menu;
  a screenshot named "battle" is not proof it actually contains a battle.
  Wild encounters can differ with host-divider input; compare desktop hashes
  only when the visible battle state is identical. Review art separately from
  mechanical green metrics. Comparison output stays ignored under `target`.
- Text clients have no gameplay animation loop. Their controller settles battle
  presentation instantly through normal completion handlers, retaining HP/PP,
  rewards, capture results and player-owned text pages. Never require idle
  frames or repeated unchanged A presses to get past an invisible animation.
  Keep graphical timing unchanged and test long animations in the controller.
  Painted clients may opt into `take_battle_replays`: bounded frames from the
  production timeline, object VM, OAM and supplied graphics. They are not a
  second animation/game engine. Inputs interrupt replays immediately; observe,
  save and WebMCP always describe authoritative state, never an old frame.
  Browser reduced-motion skips replay (not state updates); native users can
  set `GEOTHITE_REDUCED_MOTION=1`. MCP/text-only clients do not generate frames.
  Never turn a replay failure or missing art/audio into an input fence.
- Settle authored actor movement with `advance_visible_script_movement`, not
  only walk timers. Its queued programs and retained scene can survive
  `closetext`/script end (Mom's return walk was a real example). Leaving that
  scene alive retains stale text input ownership while the UI says Overworld;
  it can lock Start immediately and movement after crossing the first-floor
  doormat. Run the production movement completion handler in the shared
  controller; never clear scene/input fences in a frontend or special-case Mom.
  Test repeat conversations and exploration, not only the shortest house exit:
  `TUI_NATIVE_HOME=1 node tools/tui-stdio-smoke.mjs` and
  `TUI_NATIVE_HOME=1 GEOTHITE_TUI_GRAPHICS=off node tools/tui-terminal-smoke.mjs`.
  Browser smoke covers this same route with DOM lowercase A and real WebMCP,
  requires immediate Start after Mom and at `(5, 7)` left of the doormat, and
  authoritative movement back across the room without idle gameplay frames.
- Text clients also have no audio device or Bevy Update loop. Reveal the
  current printer page before polling WaitSFX; an unread paragraph/CONT must
  remain player-owned, and A/B must still advance it while the audio fence is
  retained. A busy fence must not starve the printer or spin the settle limit.
  Complete transient sounds through normal handlers, including phone rings,
  incoming calls, hangup, Pokédex search/not-found holds and radio furniture
  delays. Radio settles one authored program/print call per input, not an
  endless broadcast or an animation timer. Preserve graphical audio timing
  and PCM. Test `text_controller_settles_audio_without_skipping_unread_text`,
  `text_controller_settles_pokedex_search_and_phone_rings`, and
  `text_controller_pack_heals_and_pokegear_cards_remain_playable` against the
  external pack. A Potion must restore HP and consume exactly one item in
  field and battle; classify HP items from the core effect plan, not names or
  a hand-written item list. Map, untuned radio and Dex entry actions must have
  semantic text rows, not just graphical pixels.
  Also cover `text_controller_incoming_phone_completes_rings_and_hangup`,
  `text_controller_pc_item_switch_finishes_both_sound_fences`, and
  `text_controller_battle_pack_heals_through_authoritative_turn`. Native
  graphical Dex audits may read existing external artwork using
  `CRYSTAL_RENDER_TEST_ASSET_ROOT`; never extract/copy another catalog into
  this repository. That art path does not imply its pack export schema is
  compatible with `pack_core`.
- Menu end-to-end verification uses the test-only `browser_menu_fixture`
  example to create an ignored save. Set `TUI_TEST_MENU_FIXTURE` for browser
  smoke and `TUI_NATIVE_MENU_FIXTURE` for stdio smoke. Require real Potion
  HP/count changes, all four Pack pockets, Itemfinder feedback, Dex pages,
  area/cry/printer/options and successful/empty searches, Gear map selection,
  a two-ring authored phone callback plus hangup, radio tuning, and movement
  afterward. Use actual Chrome keyboard/WebMCP and mobile WebKit touch. Never
  import these fixtures into production or user saves.
- Keyboard A must confirm throughout battle/dialogue, not just while a menu
  exists. Otherwise lowercase `a` falls back to WASD-left on "Wild ... appeared!"
  and turn narration, making battles look frozen. Use
  `TextSnapshot::confirmation_input_owned` / `modalInput` and verify a real
  starter-level battle with DOM keyboard `a` presses, not only bridge/MCP A.
  Verify the native executable too with `node tools/tui-stdio-smoke.mjs`; native
  key mapping regressions must cover the same dialogue-only ownership.
  On macOS, `node tools/tui-terminal-smoke.mjs` drives a real PTY with lowercase
  A and requires battle completion and Start afterward (Python stdlib supplies
  the PTY only). Build the release executable before testing it. The optional
  `browser_battle_fixture PACK SAVE 5` makes a starter-level, multi-turn save.
  Pass it as `TUI_TEST_LOWLEVEL_FIXTURE` in browser smoke; `TUI_NATIVE_FIXTURE`
  selects it in stdio/PTY smoke. Production never imports fixtures itself.
- Render collision with `crystal_assets::resolve_collision_token` and the core
  collision attributes, not substring heuristics. `HOP_*` needs directional
  arrows, numeric walls need `#`, trees `T`, side barriers `║`, unknowns `?`.
  Missing collision data must never be shown as ordinary floor.
- Register WebMCP immediately on arrival, await session readiness in callbacks,
  handle cancellation and abort registration on partial failure/page exit.
  Tools are observe/press/move/save. The browser smoke uses installed Chrome's
  real `document.modelContext.getTools/executeTool` API with WebMCP enabled;
  a fake registration stub alone is not proof of WebMCP support.
- Verify the actual WASM, not just a native test or successful compilation:

  ```sh
  sh tools/tui-build.sh
  cargo run --locked -p geothite --example browser_battle_fixture -- \
    content-packs/realtime-clock.browser.crystalpack \
    target/tui-smoke/battle.crystalsave
  node tools/tui-browser-smoke.mjs
  ```

  Install Playwright Chromium and WebKit if needed. The smoke test requires
  keyboard and registered browser-tool inputs, Mom's interception/pagination,
  dialogue input ownership, New Bark's gate, save/resume, battle commands,
  move selection, victory text, post-battle movement and Start, plus mobile
  WebKit touch movement and menu display. The fixture is test-only; production
  gameplay must never use it. Verification output stays ignored under `target`.
  For deployment checks, set `TUI_TEST_URL` to the public `/tui` URL and generate
  the fixture against the exact pack served there, not another pack identity.

### Non-negotiable TUI/MCP architecture

- Native MCP `search`/`execute`, browser WebMCP `geothite_tui_search`/
  `geothite_tui_execute`, and `window.geothiteTui.execute(code)` use the same
  pinned Rust Boa code-mode interpreter in `crystal-tui/src/codemode.rs`.
  Accept an async JavaScript function BODY with sequential `await tools.*`
  (alias `codemode`), returning `{value,calls}`. Only visible observe/press/move/
  save capabilities are available; all game inputs use the shared production
  controller. Move is 1–20 taps, stopping at modal input, not exact tile travel.
  Never add browser eval, Node subprocess execution, filesystem/network globals,
  cheat capabilities, or a second game/menu dispatcher. Keep direct MCP tools.
  Fresh contexts and source/tool/VM/JSON/ArrayBuffer limits bound routine agent
  programs; do not describe Boa as a hostile-code sandbox (no total heap cap).
  Save only to the configured session destination. Errors preserve prior inputs,
  configured native autosaves and the browser's real rendered state; do not claim
  rollback or mid-batch cancellation. WebMCP checks abort before execution.
  Verify `TUI_CODEMODE=1 TUI_NATIVE_HOME=1 node tools/tui-stdio-smoke.mjs` and
  `TUI_CODEMODE=1 node tools/tui-stdio-smoke.mjs` after building release. Real
  browser smoke must discover all six registered tools and execute a complete
  starter-level battle/rewards/post-battle movement/Start in one code-mode call,
  not merely test a mock or a returned string. Retain keyboard/mobile regressions.

- The standalone text client must construct and drive
  `crystal_bevy::VisibleShellController`. This is the supported production
  controller boundary shared with the graphical client. It owns authored
  scripts, page-by-page text, input ownership, coordinate events, trainer
  sight, warps/connections, battles, Start/Pack/Party/PC/shop surfaces, and
  visible presentation settling.
- Never implement a second dispatcher in `crystal-tui`, and never drive the
  game by calling `crystal_runtime::RuntimeGameShell::tick()` alone.
  `RuntimeGameShell` is intentionally lower-level: a tile can move while Mom,
  the New Bark exit gate, warps, menus, or post-input script continuation are
  silently skipped. If a frontend needs behavior that the controller does not
  expose, extend `VisibleShellController` and add a regression there.
- `geothite dump` proves only that the pack loads and a snapshot renders. It is
  not evidence that input plays the game. Do not report the TUI or MCP as
  working from a dump, a screenshot, or a coordinate change alone.
- Keep MCP and keyboard input on the same `VisibleShellController::press`
  path. Renderer cursor state may mirror a production cursor for display, but
  it must never become authoritative game state.
- Fullscreen Ratatui owns stdout and stderr while the alternate screen is
  active. Never add unconditional `println!`/`eprintln!` calls to gameplay,
  dialogue, movement, audio, or controller paths used by `geothite play`.
  Diagnostic output must be opt-in behind a named trace environment variable;
  dialogue tracing uses `CRYSTAL_DIALOGUE_TRACE`. Keep event data in bounded
  in-memory logs when tests need it.

- Never hand a user a `/path/to/...` placeholder when a runnable local pack is
  requested. First check the ignored `content-packs/` directory and validate the
  chosen pack with the Rust client.
- The known-good base pack path in this checkout is
  `content-packs/realtime-clock.browser.crystalpack`. Build the dedicated TUI
  pack from it with Rust:

  ```sh
  cargo run --release --locked -p crystal-assets --bin pack_text_tui -- . \
    content-packs/realtime-clock.browser.crystalpack \
    content-packs/text-tui.crystalpack
  ```

  Packs remain ignored by Git and must never be staged or committed.
- To build fresh packs, use a compatible external export workspace. It must
  contain `vendor/pokecrystal/`, `apps/web/assets/`,
  `apps/web/assets/data/content-packs/core-modular.generated.json`,
  `asm-source.lock.json`, and the Rust-generated battle animation provenance
  consumed by `pack_core`. From this Geothite checkout run:

  ```sh
  cargo run --release --locked -p crystal-assets --bin pack_core -- \
    /absolute/path/to/compatible-export-workspace
  ```

  The builder writes `content-packs/core-modular.crystalpack` and
  `content-packs/core-modular.browser.crystalpack` inside the external
  workspace. Do not copy either into tracked source paths.
- `/Users/ryanculligan/repos/pokecrystal-python` currently contains a legacy
  export schema, not the compatible generated manifest. Do not claim a pack was
  built from it unless `pack_core` finishes successfully; its present failure
  starts with the missing explicit `MASTER_BALL` item identity.
- Validate a pack and the text renderer without entering fullscreen mode:

  ```sh
  cargo run --locked -p geothite -- dump \
    content-packs/text-tui.crystalpack
  ```

  This is a render smoke check only. Before calling the client playable, run
  both controller and stdio-MCP regressions with the freshly built external
  pack present:

  ```sh
  CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
    cargo test -p crystal-bevy \
    renderer_neutral_controller_exposes_battle_commands_and_executes_a_turn
  CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
    cargo test -p crystal-bevy \
    renderer_neutral_controller_moves_after_wild_battle_exit
  cargo test -p crystal-bevy \
    renderer_neutral_controller_plays_mom_and_new_bark_gate
  cargo test -p geothite mcp_drives_real_movement_and_production_start_menu
  cargo test -p geothite mcp_battle_exit_restores_movement_and_start_menu
  cargo test -p geothite --test stdio_clean
  ```

  The controller regression must prove all of the following: the game starts
  as the requested trainer in `PlayersHouse2F`; Start exposes the production
  menu; the bedroom/ground-floor warp fires; Mom intercepts the player and A
  changes the visible dialogue page; movement is rejected while dialogue owns
  input; leaving the house enters New Bark; walking to the west exit shows
  `Wait, CHRIS!`; and completing that scene returns the player to `(5, 8)` in
  `NewBarkTown`. The MCP regression must prove its `move` tool changes the same
  authoritative position and its `press start`/`observe` path exposes the
  production Start menu. The battle regression must use the external pack,
  retain the core battle phase, expose the production FIGHT/PKMN/PACK/RUN and
  move menus, execute the selected move through the authoritative battle
  engine, observe its PP mutation, and expose the resulting battle dialogue.
  Battle-exit verification must finish retained narration, EXP/reward frames,
  and map reload, then prove a directional input changes the authoritative
  tile and Start opens the menu. Test both victory and RUN. Core becoming
  Overworld is insufficient: battle messages and reward animations can still
  own input after core ends the battle. Advance their production presentation
  clocks in the shared controller, and require MCP movement after battle too.
  A battle HUD or input log without those state changes is not a passing test.
  Any failure is a gameplay blocker, not a renderer polish issue. The stdio
  regression walks into Mom's dialogue in a spawned client and requires stderr
  to remain byte-for-byte empty, preventing trace lines from corrupting the
  fullscreen terminal.

- Build the `geothite` executable and launch the interactive terminal UI from
  this checkout with:

  ```sh
  cargo build --release --locked -p geothite
  ./target/release/geothite play content-packs/text-tui.crystalpack
  ```

- Run the same local game session as a stdio MCP server with:

  ```sh
  ./target/release/geothite mcp content-packs/text-tui.crystalpack
  ```
