# Rendering toolkit

## Iteration contract

Implementation is Rust. Build when a renderer capability changes, then keep
that executable and GPU session alive while editing geometry profiles. Cargo,
tests and watch-build commands do not belong in the art iteration loop.
Validate actual renders: a decoded, nonblank screenshot is capture evidence,
not proof of visual quality. Generated images stay outside the repository.

Worktree: `/Users/ryanculligan/GitHub/geothite-frontends`, branch `main`.
The original thread's `crystal-llm/rust` directory no longer exists.

## Start a review

```sh
CRYSTAL_VOXEL_PROFILES=modpacks/voxel-view/profiles.json \
  target/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map ElmsLab --x 4 --y 3 --view both --live \
  --screenshot /tmp/elm-north.png
```

If the executable lacks a required capability, install it once:

```sh
cargo build -p crystal-bevy --example render_at_location --features location-tester
```

Paired capture uses one game and GPU session. F3 toggles 2D/2.5D; F6 captures
again. Arrow keys walk normally. The default fixed hour is 12. For the prefix
above, results are `elm-north-2d.png`, `elm-north-2.5d.png`, and
`elm-north-2d.compare.png`. The comparison contains source 2D / previous 2.5D /
current 2.5D when map, center and camera match. Otherwise it omits previous.

## Edit and capture without rebuilding

1. Save `modpacks/voxel-view/profiles.json`.
2. Wait for `geometry profiles loaded: revision N` (accepted data) and
   `geometry profile mesh applied: revision N` (scene replacement).
3. Submit an external capture request or press F6. Empty requests preserve the
   camera; two numbers set zoom and orbit. Zoom clamps to 0–5; orbit wraps at
   8, with one step equal to 45 degrees. Fractions work.
4. Wait for `review ready`, then inspect the images before another request.

Write camera requests atomically so the renderer cannot consume a truncated
file while the writer is still filling it:

```sh
printf '0 0.5' > /tmp/elm-north-2d.request.next
mv /tmp/elm-north-2d.request.next /tmp/elm-north-2d.request
```

For an unchanged camera, `touch /tmp/elm-north-2d.request` is sufficient.
Invalid requests report an error and leave the session alive. Invalid profile
edits retain the last good geometry. The file watcher polls every 150ms. A
capture waits for the current geometry revision and 30 active 2.5D frames;
completion uses GPU readback and image-save callbacks. Previous output PNGs
are cleared when a request begins, preventing stale-file validation.

Avoid running unnecessary viewers concurrently. Verify a process is stopped
before replacing it. A slow log or observation timeout does not prove exit.

## Author profiles from source identities

Use the Rust helper to inspect the pack's exact block grid:

```sh
target/debug/examples/inspect_voxel_map \
  content-packs/core-modular.browser.crystalpack ElmsLab PlayersHouse1F
```

Its source is `crates/crystal-assets/examples/inspect_voxel_map.rs`. Install
with `cargo build -p crystal-assets --example inspect_voxel_map` when needed.
During this review it was compiled against the existing assets library and
run as `/tmp/inspect_voxel_map`, without rebuilding the renderer.

An object specifies `name`, `tileset`, optional `map`, `metatile`, `origin`,
`tiles`, `ground`, `top_pixels`, and `depth_pixels`. IDs are decimal integers.
`origin` is the subtile coordinate within the anchor block (0–3). `tiles` is
the exact rectangular tile-index matrix. Its extent including the anchor
origin must fit within 8×8 source tiles. Cross-block drawings also require
`metatiles`, the exact rectangular grid of block IDs covering the drawing;
its first ID equals `metatile`. Only complete matching drawings are replaced.
Overlapping object rules use the first match. Gameplay collision is unchanged.

The simple fold assigns the first `top_pixels` rows to its horizontal top
and folds the remaining rows upright at native pixel height. Depth controls
the footprint. The original drawing is replaced by floor. Zero top/depth
produces an upright card. Native file reload requires the environment variable
above; builds without it use the profile document embedded at compilation.

## Compound fixtures and masks

An object may specify `parts` containing disjoint source-pixel rectangles.
Each part requires `rect: [x,y,width,height]`, `top_pixels`, and `depth_pixels`.
Optional fields:

| Field | Effect |
| --- | --- |
| `base_pixels` | Raises the whole part; default 0 |
| `height_pixels` | Upright height; default is rows after the fold |
| `offset_pixels: [x,z]` | Placement relative to the source drawing |
| `mask: "ground"` | Removes boundary-connected pixels matching floor colors |

A `bevel_pixels` part slopes its top drawing down to ground at its perimeter.
It requires `top_pixels` equal to rectangle height, positive `height_pixels`,
zero `base_pixels`, and part mask `"none"`. Bevel width must be positive and no
greater than half the smaller rectangle dimension. Use a precise source
rectangle to exclude padding; ground masking cannot be combined with bevels.
Compound parts have their own mask and do not inherit the object mask.

A part with all rows assigned to its top is a horizontal surface. Source
rectangles cannot overlap or exceed the drawing. At most 32 parts per object
and 256 objects are accepted. The object replaces its whole footprint once.

The default mask is `"none"`. Ground masking preserves enclosed highlights,
but palette-shifted floor drawn inside furniture may differ from plain floor.
Use precise source rectangles for separate legs in that case. Do not mask an
opaque wood tabletop with matching wood-floor colors. Positive-depth cutouts
close exposed pixel edges. The lab desk uses separate tabletop, leg, book,
and monitor parts; the monitor removes floor-colored source padding.

## Visual checks

Inspect source/3D side by side, then orbit a half-step. Check silhouette,
top/front separation, texture stretch, closed sides, underside, hovering
geometry, duplicated source pixels on the floor, and actor depth ordering.
Walk around fixtures and inspect aisles, stairs and doors. Use full-object
framing before diagnosing clipping as a geometry defect. Save concrete
before/after evidence and record unresolved defects explicitly.

## Beveled surfaces

A compound part may specify `bevel_pixels` to map its complete drawing over
one raised surface with rounded sloping corners. Set `top_pixels` equal to
its source rectangle height, `height_pixels` to its rise, and `depth_pixels`
to its footprint depth. Bevel width must be positive and no greater than
half the smaller source dimension. This mode requires zero `base_pixels`
and `mask: "none"`; those restrictions keep its perimeter at ground level
and avoid open holes. Every source pixel maps once; no facade texture is
repeated down side walls. Height and bevel width remain live JSON controls.

## Capture execution modes

Live review retains the renderer and GPU session for profile and camera edits.
One-shot native capture keeps rendering on the main thread to avoid a Bevy
render-channel teardown hang. Paired captures and rendered walks were verified
to save, validate, and exit successfully. Frame timings from one-shot runs
are not directly comparable with pipelined live review.

## Review evidence and open work

See [Render review log](RENDER_REVIEW_LOG.md) for inspected locations,
before/after decisions, measured capture times, and unresolved findings.
Historical "live" labels are not current process state: inspect `ps` before
sending capture requests or replacing a viewer. The main open visual areas
are foliage refinement, wider Route 29 terrain edges, and complete building
approach/stair review. A nonblank image alone does not close those items.

For player or NPC orientation changes, capture the same stationary location at orbit 0,
1, 2 and 4 (0, 45, 90 and 180 degrees), with fixed zoom. The sprite's vertical
axis must follow world up, and its bottom pivot must stay on the same footing.
Select front/side/back source artwork relative to gameplay facing: a south-facing
NPC shows its back to a camera north of it. Upright directional sprites may
turn their drawing plane toward the view, but must never pitch toward it or
keep showing the same face through a full orbit. Check both upright geometry
and directional artwork; a single angle cannot prove this behavior. Transform
or source-frame plumbing changes require a renderer build; angle captures reuse
the live process. For the player, also capture movement after turning: retained
entities must refresh directional frames, preserve the walking phase, and use
the correct art for the current movement mode. Remote-player orientation remains
a separate path and is not covered by local-player evidence.

## Live outdoor atmosphere

User preference: no haze anywhere. The canonical profile omits `atmosphere`;
keep all maps clear. The settings below describe renderer capability only,
not an approved visual treatment.

Optional top-level `atmosphere` settings in `profiles.json` select exact maps:

```json
"atmosphere": {
  "maps": ["Route29", "NewBarkTown", "CherrygroveCity"],
  "start_tiles": 8,
  "end_tiles": 32,
  "opacity": 0.75
}
```

Distances are added to the camera-to-target distance in visual tile units,
keeping the nearby play area clear when zoom changes. Haze uses the outdoor
background color. Omit the block or set opacity to zero to disable it. Unlisted
maps, including interiors, remain unaffected. Distances must be finite with
`0 <= start_tiles < end_tiles <= 128`; opacity must be in `[0,1]`. Invalid edits
retain the last valid document. Atmosphere-only edits do not increment the
geometry revision or rebuild terrain; submit the usual capture request to
compare them. Changes to object profiles still rebuild geometry as before.

## Tree presentation preference

Keep trees upright in 2.5D as source-art cutouts. Do not flatten them onto the
terrain, round them into voxel hulls, or extrude their pixels into thick slabs.
Haze remains disabled. Furniture sharing grouped-mesh utilities is separate.

Movement review in fixed-camera rooms accepts a full source-tile span of player
translation relative to the map center, in addition to grid-origin/map changes.
Facing or animation alone must not satisfy this check. Inspect moving captures;
a successful exit does not prove an entire aisle is unobstructed.

Replacement ground lookup accepts flat terrain or water source tiles. A ground
ID must be present in the captured terrain; an unavailable or unsupported
source skips that profile. Use water ground for offshore objects to avoid
inserting a strip of land beneath them.

Keep one live review renderer open at a time. When moving to another map,
close the completed capture process after confirming its exact command/PID;
retained review windows continue consuming rendering resources even when idle.
Do not terminate unrelated game sessions.

## Breadth-first coverage

Object profiles may use either `"map": "Route29"` or a nonempty exact
allowlist such as `"maps": ["Route29", "Route46"]`. Omit both for all maps.
The parser rejects both selectors together and empty map lists/names.
Use map lists to share identical drawings without expanding their scope.
When consolidating, preserve each map's effective object order: first matching
profile owns overlapping source cells. Identical geometry alone does not make
reordering safe. Map-list edits reload through the existing profile watcher.

Rotate through batches of distinct maps and tilesets. Choose a clear source
path tile from the map grid before capture, then inspect both source and 2.5D
images. Record exact coordinates, artifacts and remaining defects in
RENDER_COVERAGE.md. A successful screenshot alone does not prove walkability
or traversal. Replace captures that spawn within tree, fence or building art.
Use the existing executable for coverage; rebuild only for native code changes.
Prioritize defects repeated across maps before revisiting local decoration.

For vertically stacked parts, account for each source rectangle's y+h when
aligning their front planes. A top-half16px part over a bottom-half16px part
needs a +16 depth offset to share its front plane. Base height alone does not
align depth. Diagnose texture defects with mask none before blaming masking.

### Coverage when a manual spawn is rejected

Use the existing batch selector even for one map. It selects a walkable
runtime tile and emits the same comparison artifacts without rebuilding:

```sh
CRYSTAL_VOXEL_PROFILES=modpacks/voxel-view/profiles.json \
  target/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --maps MahoganyMart1F --view both --output-dir /tmp/mahogany-auto
```

Inspect the resulting framing before calling a map covered. A valid spawn
only proves a walkable sample, not complete visibility or traversal.

For missing-ground failures, set `CRYSTAL_VOXEL_TRACE_GROUND=1` on a diagnostic
render to include sample-lookup backtraces. Omit it during normal iteration;
backtrace capture is deliberately outside the normal fast path.

### Verify profile placement

Set `CRYSTAL_VOXEL_TRACE_PROFILES` to a case-sensitive substring of the object
name when launching the renderer. Empty string traces all objects; omit the
variable for quiet normal iteration. Logs distinguish missing Flat/Water
source ground, drawings larger than the grid, and actual matched counts
with the ground cell used. A zero count means no unclaimed exact placement;
it does not distinguish absent artwork from ownership conflicts.

Example: `CRYSTAL_VOXEL_TRACE_PROFILES='Shared lab computer'` with the usual
render command. Document acceptance alone never proves an object matched.
This diagnostic is native and already included in the current executable;
changing the filter or profile requires no rebuild.

## Repeatable coverage positions

Use [verified render presets](RENDER_PRESETS.md) for a small cross-region pass
with reviewed street coordinates and a movement segment. These avoid known
misleading automatic starts and list the remaining visual limitations.

### Objects supported by live furniture

An object profile may set `footing_pixels` (finite0–64). This explicitly sets
actor support height over every source cell owned by that drawing, scaled from
source pixels into world units. Omit it to preserve existing support behavior.
Use only for a uniform support surface; the field does not infer varying part
heights or follow offsets. Match the tabletop height and render any occupants.
Changing this value reloads both geometry and cached footing without rebuilding.

## Location-render encounter fixture

The render_at_location example explicitly enables `render_test_party`, creating
one level30 Totodile with fixed DVs and a RENDER owner in its fresh location
session. This supplies a valid party for incidental encounters. The option is
restricted to location-tester builds and fresh runtime-tile starts; normal
new-game/save loading does not enable it. Adding the party changes RNG/state,
so old walk sequences need not trigger the same encounter. Inspect runtime
errors in capture logs: a nonblank PNG alone is still insufficient evidence.
