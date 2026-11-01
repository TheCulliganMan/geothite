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

Town and route pines have tall, irregular crowns inspired by Japanese red pine:
high spreading, upright and swept forms, open lower trunks, sparse short lower
limbs and warm upper bark. Their leaders rise above ordinary house roofs.
Horizontal crown fitting preserves that height; streaming bounds include the
complete treetop. Ancient forest and formal park trees retain their own forms.
The revised pines were checked in native headless image-target captures against
matching 2D at New Bark `(8, 8)` at noon and Cherrygrove `(26, 8)` at noon and
22:00; every pair preserved game state. A wider New Bark capture checked the
complete crowns above the houses.

Pine plots and eligible Ilex canopy junctions carry individual CC0 Quaternius
ferns, upright leafy stems, clover and occasional wildflowers. Curved leaf
silhouettes and painted vein gradients use the terrain's existing solid palette.
The imported purple clover and autumn foliage receive restrained moss/sage greens.
See `crates/crystal-voxel-view/models/understory/README.md` for provenance and the
Rust importer.

Ordinary Kanto shrub drawings use three distinct CC0 Quaternius broadleaf models:
a spreading crown, an asymmetric forked crown and an upright high crown. Source
coordinates choose exactly three shared meshes deterministically; stable height,
width, yaw and root offsets vary their planting. The Rust importer preserves
modeled branches and replaces square leaf cards with closed, cupped oval leaves.
Their roots stay inside the original blocked plots and use the same connected
understory as Johto. See `crates/crystal-voxel-view/models/deciduous/README.md` for
provenance and regeneration. Johto pines, Ilex ancient trees, formal park trees,
fruit trees and Cut trees retain their existing models.

Plant placement follows source cells across contiguous native tree plots.
Neighbor occupancy includes partial contacts and concave corners; interior
plot edges do not clip or restart the planting. Deterministic jitter, rotation,
height, species and exposed-edge setbacks create an irregular outline while
keeping leaves off authored walking surfaces. Individual plants share a bounded
species/palette library and keep their proportions on rectangular source plots.
Tree geometry, root positions and footing remain unchanged.

Planting is checked with matching 2D captures at Ilex `(10, 12)` and New Bark
`(5, 6)` / `(2, 6)` at noon, and Cherrygrove `(26, 8)` at 22:00, using headless image targets.
All pairs preserved game state. A 64-frame left/right walk in the installed
native client crossed five grid origins without additional terrain builds and
preserved its final paired state.

The deciduous/understory review compared matching production 3D/2D views on 21
maps: New Bark, Routes 29/30/35, Violet, Azalea, Ilex, Goldenrod, Ecruteak,
Cherrygrove, Pallet, Viridian, Pewter, Cerulean, Celadon, Fuchsia, Saffron,
Routes 1/2/15 and National Park. Cherrygrove was reviewed at 22:00; the others
at noon. Viridian was also checked at 22:00. Every pair preserved game state.
The review caught and fixed streamed
Kanto trees using Johto terrace levels: trees and plants now use the same
Kanto shelf as the ground, including retained positive terrace data without
local jump edges. Dense Kanto crowns use coverage reveal to keep the player
visible through overlapping leaves. The installed native build passed a
64-frame Route 2 left/right walk across four grid origins with no additional
terrain builds and preserved its final paired state. All 46 targeted tree, plant,
terrace, cache and terrain-worker checks passed with the external pack.

New Bark's four warps stay at lab (6,3), player home (13,5), neighbor (3,11),
and Elm's home (11,13). Cherrygrove's Center/Mart doors stay at (29,3)/(23,3).
Violet's Center/Gym doors stay at (31,25)/(18,17).

## Native local play

Use Rust 1.99.0 and a compatible external content pack, as documented in
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
