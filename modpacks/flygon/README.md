# Flygon

A local Rust/WASM connectome controller and interactive 3D anatomy viewer for
Geothite's Pokémon Crystal browser game. This is an experimental preview, version
0.2.0. Elm arrival is verified under online reward shaping; Cherrygrove and
reward-off story mastery are not established. See [measured results](../../docs/FLYGON_RESULTS.md).

## Architecture

| Component | Responsibility |
| --- | --- |
| `crates/crystal-flygon/src/lib.rs` | Full retained graph, LIF dynamics, synaptic plasticity, configuration validation |
| `operant.rs` | Situation/action cues, actual MBON readouts, dopamine conditioning, configurable outcome teacher |
| `view.rs` | Independent anatomy model, 3D perspective, depth shading, projection, picking and connection rendering |
| `interface.rs` | Structured sensory interface, source lookup and compatibility APIs |
| `checkpoint.rs` | Graph/model-checked brain memory and dynamics |
| `campaign.rs`, `conditioning.rs` | Reference controller ledger and controlled learning experiments |
| `bin/flygon-package.rs`, `preview.rs` | Native release assembly, hash verification, loopback preview and live source serving |
| `web-client/flygon*.js` | Thin DOM, pointer, worker and game-bridge transport; no neural or geometry engine |

The simulation and viewer run in separate workers using the same small WASM
module. The viewer receives real soma coordinates once, and sparse measured spike
indices after each neural step. Camera gestures do not wait for neural trials.
All projection, picking, connection extraction and pixels are computed in Rust.

The import retains 166,700 neurons, 25,582,938 directed pair edges and 124,177,617
contacts. There are 139,662 source soma coordinates. Missing somas remain simulated
but are not drawn. The optional profile dropdown changes experiments; it does not
silently substitute a smaller neural graph.

## Use the viewer

Open `/flygon.html`; the brain loads automatically. The default is the story action-memory
profile. **Run brain** starts decisions; **Control game** enables their submission.
The game remains in its own pane. Human game input and hiding the tab pause the
controller. Repeated presses are ordinary game inputs, never scripted route steps.

- Drag to orbit; scroll or pinch to zoom. Mouse and touch are supported.
- Arrow keys rotate a focused canvas; `+`/`−` zoom; `0` resets; Escape clears selection.
- Click a soma to inspect its source identity and strongest incoming/outgoing links.
- Purple arrows are incoming; blue arrows are outgoing. Green links mean their
  source spiked in the current measured window, not verified synaptic transmission.
- At most 48 links per direction are drawn, ranked by anatomical contact count.
  Selection reports visible totals and omissions. Lines join real somas; they are
  schematic connectivity, not reconstructed axons. Spiking cells are a luminous
  overlay so activity remains visible within the depth-shaded anatomy.
- Population selection focuses activity without reducing the simulated graph.

Research tools are collapsed beneath the main view. Save brain and Export run
record retain learned state and evidence. Save the Pokémon game separately when
its save control is available. These are not atomic combined checkpoints.

## Tight source loop

Prerequisites: the repository Rust toolchain, `wasm32-unknown-unknown`, and
`wasm-bindgen-cli` **0.2.126**, matching Cargo.lock. An existing game bundle is
required; this workflow does not compile Pokémon.

```sh
# Only when Rust changes (small crate, not the game):
sh tools/flygon-build.sh
cargo build --locked -p crystal-flygon --bin flygon-package --profile web-release

# Native Rust source server; defaults shown, no compiler invocation:
FLYGON_GAME_ROOT=target/3d-web \
FLYGON_DATA_ROOT=/tmp/flygon-data/prepared \
FLYGON_PORT=33006 sh tools/flygon-dev.sh
```

Open `http://127.0.0.1:33006/flygon.html`. The loopback server uses the real local
UTC clock and disables multiplayer in the embedded game. CSS and `view.json`
refresh live without resetting the brain or game. **Reload tuning** reads config
changes without rebuilding or erasing learned synapses. HTML/JS changes require
page reload; save game and brain first. Rust changes require the small WASM build.
No rendering or configuration iteration invokes Cargo automatically.

`FLYGON_TOOL` can select an already built native packager binary. The old Node
preview server has been replaced by this Rust tool. Playwright is only used for
browser validation; it is not a public runtime dependency.

## Data preparation

Source files are pinned and hash-verified. Downloads and compiled data stay outside
Git. Preparing them is a one-time operation; visitors fetch only the prepared graph.

```sh
sh tools/flygon-data.sh /tmp/flygon-data/source
cargo build --locked -p crystal-flygon --features import --bin flygon-import --profile web-release
target/web-release/flygon-import /tmp/flygon-data/source /tmp/flygon-data/prepared
```

`dataset.json` declares source hashes, retained counts and the combined SHA-256
identity of graph.bin followed by metadata.json. The simulation validates that
identity. Attribution is in [ATTRIBUTION.md](ATTRIBUTION.md) and in the public UI.

## Assemble a public preview

```sh
# Uses existing game and neural bundles; no builds, downloads or deployment:
target/web-release/flygon-package \
  --workspace . --game target/3d-web \
  --data /tmp/flygon-data/prepared --out target/flygon-release-0.2.0

target/web-release/flygon-package --verify target/flygon-release-0.2.0
target/web-release/flygon-package --preview target/flygon-release-0.2.0 33008
```

Assembly validates runtime identities and profiles, verifies the pinned graph,
copies a standalone distribution without symlinks, and records every shipped file's
SHA-256 and size. It refuses existing output, missing assets and stale bindings.
An interrupted assembly never exposes a half-built release directory. Brain saves,
run captures, development tests and raw source tables are excluded.

The output contains the existing game inventory, the optional Flygon page, thin UI,
WASM, prepared graph, profiles, dataset attribution and a release manifest. It does
not add a ROM, duplicate game content or upload visitors' brain state. Open
`/flygon.html`; root `/` is the existing game. Serve the assembled directory through
the existing `crystal-web-server` or the existing hosted game origin/clock. Preserve
WASM MIME types and same-origin asset paths. The Rust preview server is loopback-only
and supplies real local time; it is not a public production server. Deployment is a
separate action. This optional distribution does not alter the base Docker image.

Initial neural data is about 216 MiB. The simulation's measured WASM memory is
552 MiB, with additional viewer, game and browser memory. Recommend at least 1 GiB
free; mobile memory capacity is not guaranteed. The preview does not claim offline
service-worker support. After assets are available, local preview requires no
external simulation service.

## Reward curriculum and evidence

`operant.json` contains all curriculum targets, map order, goal map, pre-starter
targets and eligible dialogue object names. These are evaluator inputs only: they
are not supplied to the action decoder. The current experimental target is Route30,
through Cherrygrove. The recorded verified milestone remains Elm arrival.

Each context/button pair addresses 8 real KCs. All eight buttons receive 500 ms
reward-free probes with frozen plasticity and reset transient dynamics. Measured
MBON01/MBON11 changes relative to naive calibration determine probabilities, with
10% declared exploration. The calibration is not an external learned Q table.
Selected actions use 16-frame holds. After an outcome, six cue/DAN pairings recruit
PAM01 for reward or PPL101 for aversion. Actual DAN spikes and KC eligibility modify
existing KC→MBON synapses. The viewer shows actual activity from these trials;
reported summed trial time is not continuous brain time.

Forward locations, receiving a starter, shorter observed-terrain distance and new
eligible dialogue can earn reward. Retreat, irrelevant prompts and no progress can
earn aversion. If the observed terrain does not provide a path, distance falls back
to Manhattan. This engineered shaping does not establish natural fly perception.
The older reference profiles have separate reward-strength controls; those controls
and unpaired manual pulses are disabled/hidden where they do not apply.

Mechanism validation after neural changes:

```sh
cargo build --locked -p crystal-flygon --bin flygon-operant-assay --profile web-release
target/web-release/flygon-operant-assay /tmp/flygon-data/prepared modpacks/flygon/operant.json
```

The assay checks naive probabilities, reward preference, aversion suppression,
frozen learning and memory erasure separately from gameplay. End-to-end learning
still needs reward-off rollouts and matched episode controls. A rendered spike or
changed weight alone does not demonstrate learned story completion.

## Render review

With Playwright and its browsers already installed, review the **assembled** preview:

```sh
node tools/flygon-review.mjs
FLYGON_BROWSER=firefox node tools/flygon-review.mjs
FLYGON_BROWSER=webkit node tools/flygon-review.mjs
```

The harness opens a separate browser session, waits for the full graph to autoload, takes a real
neural decision with game submission off, selects source MBON01, and renders its
actual incoming/outgoing edges. It checks that orbit changes pixels while neural
state remains fixed, captures desktop and narrow layouts, and exercises Chromium
touch orbit/pinch. It records missing assets and browser exceptions. Output goes to
`target/flygon-evidence/release`; use `FLYGON_EVIDENCE_DIR` to override. The default
is a visible browser; `FLYGON_HEADLESS=1` is optional where the game renderer supports
it. No compiler runs in this loop. No neural data or decisions are generated by JS.
