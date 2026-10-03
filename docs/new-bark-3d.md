# Connected modeled Johto (local development)

The optional 3D renderer now has an original authored art slice spanning
**New Bark Town → Route 29 → Cherrygrove City → Route 30 → Route 31 → Violet City**.
It consumes the same `VisualWorldFrame` as the prior view. It never writes
collision, map coordinates, warps, dialogue, inventory, party, or scripts.

## What is modeled

- New Bark: houses, two-story player home, Elm's laboratory, trees, flowers and signs
- Cherrygrove: red-roof Pokémon Center, blue-roof Mart with striped awning, cottages,
  trees, coastal ground treatment and signs
- Routes 29/30/31: source-placed trees, flower beds, low folded-blade grass,
  signs, Route 30 cottages and Route 29's complete north-gate facade
- Violet: Pokémon Center, Mart, violet-roof gym, traditional cottages/academy,
  tiered Sprout Tower, trees, flowers, ground and signs
- Nine articulated character looks: male/female trainer, rival, youngster,
  teacher, lass, scientist, outdoorsman and elder

Unknown/clipped art retains the prior renderer. Interiors and the east-facing
Route 31 gate still use their existing view. Bikes, surf mounts, disguises,
nonhuman NPCs and effects retain their accurate published sprite art. Remote
players currently use a default trainer appearance because their presentation
frame does not publish a clothing/gender identity.

The new character rigs have independent joint meshes and a parent hierarchy.
Distance-driven gait uses two-bone legs with planted stance shoes, lifted swing
feet, opposing arms, small torso/head motion, idle breathing and blinks. Facing
comes from the host and is smoothed visually; map/camera changes cannot manufacture
walking distance. The root remains exactly anchored to the published foot.

## Spatial contract

Replacement requires a complete source-art signature and its native doorway.
Source-cell plot edges stay fixed; centered authored doors fit the actual
off-center threshold column. Violet's traditional roof overlaps and adjoining
tree rows are handled explicitly rather than guessed from repeated roof IDs.
Trees vary deterministically within the source plot. Ground shading and detail
are render-only; the original footing remains authoritative.

New Bark's four warps stay at lab (6,3), player home (13,5), neighbor (3,11),
and Elm's home (11,13). Cherrygrove's Center/Mart doors stay at (29,3)/(23,3).
Violet's Center/Gym doors stay at (31,25)/(18,17).

## Native local play

Use Rust 1.94 and a compatible external content pack, as documented in
`game-content.md`. Packs remain ignored and are not included in source bundles.

```sh
cargo run --locked -p crystal-bevy --example new_bark_3d --features location-tester -- \
  content-packs/core-modular.browser.crystalpack
```

The native preview starts a fresh real game in New Bark at (13,6), 16:00, without
loading/overwriting a save, granting a party or unlocking the town's scripts.
Arrows move, Z interacts/confirms, X cancels, Enter opens Start, Q/E orbit,
Page Up/Page Down zoom, and F3 switches the 2D/3D view.

Inspect other map presets with `--map CherrygroveCity`, `--map VioletCity`,
`--map Route29`, `--map Route30` or `--map Route31`. Override a preset using
`--x 29 --y 4`, `--zoom 1` and/or `--orbit -1`.

These are developer location previews, not a completed journey. They preserve
story gates and encounters; for an ordinary adventure use the normal title/new
game flow, obtain the starter and progress the story. A screenshot or location
fixture alone is not evidence of completing the adventure.

### Screenshots and actual-frame recordings

Append `--screenshot output/johto-3d/cherrygrove.png` for a settled one-shot GPU
capture. Append `--record output/johto-3d/cherrygrove-walk --seconds 30` for a
live-input recording. The recording folder must be empty and duration is 1–300
seconds. Screenshots and recordings are mutually exclusive.

The recorder reads the actual game framebuffer, at most 30 samples/second with
a bounded eight-readback pipeline. It never sends inputs or changes gameplay. Recorded
timestamps include real rendering stalls; this is not a fabricated smooth
animation. Convert the recorded sequence locally with:

```sh
ffmpeg -f concat -safe 0 -i output/johto-3d/cherrygrove-walk/frames.ffconcat \
  -fps_mode vfr -c:v libx264 -crf 19 -pix_fmt yuv420p \
  -enc_time_base 1:1000 -video_track_timescale 1000 -movflags +faststart \
  output/johto-3d/cherrygrove-walk.mp4
```

Recordings intentionally contain no audio. The capture helper is native-only,
behind `location-tester`, and absent from normal/browser builds.
For repeatable movement QA, optionally add `--walk LLRR`: the separate existing
walking harness sends ordinary direction keys through the production input path.
It does not teleport, bypass scripts or grant a party. The recording includes
map/position timestamps; the walking harness also writes its own movement trace.

## Browser build

With the official `wasm-bindgen-cli` 0.2.128 and wasm32 target installed:

```sh
sh tools/new-bark-3d-web.sh
CRYSTAL_DATA_DIR=target/new-bark-3d-data target/debug/crystal-web-server \
  --dir target/new-bark-3d-web --host 127.0.0.1 --port 3003
```

Open `http://localhost:3003/?multiplayer=off&preview=new-bark` for the feature-gated
fresh preview. Omit `preview=new-bark` for normal title/Continue flow. Browser
controls use Space for Start and the Settings menu for view/camera controls.
No remote deployment is performed by these commands.

## Editable art source

```sh
blender -b --python tools/build-new-bark-models.py -- target/new-bark-assets --skip-preview
blender -b --python tools/build-new-bark-player-house.py -- target/player-house-assets
blender -b --python tools/build-connected-johto-models.py -- target/connected-johto-assets
blender -b --python tools/build-johto-characters.py -- target/johto-character-kit
blender -b --python tools/build-johto-foliage-lod.py -- target/johto-foliage-lod
```

The Python scripts are original geometric authoring source, not extracted game
art. They produce editable Blender files and runtime JSON; environment exports
also include GLBs. Runtime data lives under `crates/crystal-voxel-view/models/`.
Exports use +Y up/front +Z and linear material colors. Rigs preserve joint pivots,
parent relationships, smooth normals and floor-centered alignment. Runtime
characters share mesh/material handles; scenery is combined into terrain meshes
instead of spawning an entity for every roof tile or grass blade.

Build products, packs, renders, recordings and test reports stay ignored under
`target/` or `output/`. Do not commit or bundle packs or compiled binaries.

## Verification

Native and Wasm builds, all 94 browser unit tests, and six real-pack story,
connection and doorway regressions pass. The real movement-publication regression
passes at 60/30/9 Hz. The battle-turn fixture was made reliably nonterminal and
passed ten fresh-process runs across the two local compatible packs, preserving
its PP/text/menu assertions. Focused model, source, doorway, prop and gait checks
pass. The full voxel suite retains four failures also reproduced on the original
base: two legacy tree-depth expectations, Ecruteak fence classification, and
remote-card transform equality. These known failures are disclosed in the PR.

A separate pre-existing renderer-neutral controller limit remains: terminal
battle/whiteout message settling can initially leave blank text once the active
battle disappears. This branch was not changed; native frame-driven play has a
different settling loop. This work does not claim a complete playthrough.

See `johto-3d-performance.md` for controlled native measurements and the actual
scrolling/gait fixes. See `../art/johto/README.md` for the five self-contained
editable Blender sources, kept as hash-verified compressed chunks in ordinary Git alongside the code
and runtime models. Run `python3 tools/johto_art_sources.py` to restore the
canonical editable `.blend` files. No LFS pointers or external downloads are required.

The cloud desktop uses software OpenGL and an app-local ALSA null sink because
no physical audio device is exposed. Dynamic-shadow fidelity and hardware
frame rate are not verified. Contact grounding is present without relying on
driver shadows. A native preview is not a remotely hosted playable link.
