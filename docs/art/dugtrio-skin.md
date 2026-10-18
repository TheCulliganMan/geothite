# Dugtrio: three emerging capsules and independent grounded articulation

The old static ellipsoid assembly is replaced by three continuous rounded
capsules: a tall rear mole, middle-height left mole and short foreground mole.
Each has narrow black eyes fitted to its actual stored body surface, fitted
white highlights, a rooted broad red oval nose and a small cap highlight.
Twelve individually varied faceted soil clods surround the shared bed. The
body, nose and soil palette follows the existing Diglett sculpture; full
Dugtrio sprite palette parity is not claimed.

`tools/dugtrio_sculpt.py` authors the anatomy. `tools/dugtrio_glb.py` exports
one canonical standard skin, consumed by the shared model reader, bounded
Rust registry and production battle/world renderers. The background species
builder and direct sculpture CLI produce byte-identical assets. No parallel
static JSON remains.

The rig has four controls: the fixed floor root and three independent capsule
joints, all parented directly to the root. Each body blends from its buried
base to its own capsule joint. Its complete fitted face and nose share that
joint; neighboring moles cannot drag each other's geometry. Soil stays rigid
on the root. Idle motion has different phases for each mole; attack and hit
clips stagger the three responses using source-relative cues. Clips rotate
only and have neutral endpoints.

The asset has 12,018 vertices, 13,584 triangles, 33 parts, four joints and
131 accessors in 416,564 bytes, within all existing model limits. Authoring
height remains 0.98. The local external pack reports `heightDigits=204` for
Dugtrio, preserving its physical height of 0.7112 m through uniform production
scaling. Full animated bounds, camera, collision and admission rules remain
unchanged.

Geometry SHA256:
`b3b89e9de63fe28defb7ee03d3cdb8a7efac2c97c9db3e9aae24d2f7bd5f790c`.
GLB SHA256:
`a97007d6b93a0f51b0f7de182400a8934152367e7a847b32d4f5019bc4ed4696`.

Eleven independently decoded Python checks pass in 8.584 seconds. They cover
canonical geometry and limits, closed outward geometry and shading, three
rounded dome profiles and distinct heights, family palette, real face fitting,
varied flat-faceted soil, coherent joint ownership, grounded supports and
bounded strain through all clips, neutral endpoints, independent controls,
and rejection of mutated geometry/bytes/species. Three focused Rust checks
pass in 0.45 seconds for canonical production geometry, fixed soil through
clips, original-size GPU admission and world binding/contact/full-bounds
preservation. Shared schema and exact elapsed-idle/current-attack-cue regressions pass in
106.04 and 116.11 seconds, bringing the batch to 16 scoped checks. Both native
capture clients build in 57.75 seconds.

The decoded before/after, front/rear/side and attack sheet was inspected inline
without opening a window. The production front pair completed in 83.441 seconds and the world pair in
78.830 seconds. Both preserve the same authoritative game state between 2D
and 3D. Front clearly shows all three coherent faces and ground contact; the
stationary world view is partly blocked by foreground foliage, while the
moving frame exposes the grounded capsules through the existing fade. The battle pair completed in 93.621 seconds with the full three-mole model,
grounded soil, readable DUGTRIO/SENTRET HUD names and no fallback. All three
paired captures preserve authoritative state. The stationary world obstruction
remains a visibility limitation. Both native binaries retain identical SHA256,
size and modification time before and after all captures, with the exact
current compressed GLB verified in each executable. All captures run headless, offscreen and
muted. No shiny, grass-height, scenery, AA, audio, pack or gameplay changes
belong to this batch. Full-roster improvement remains active.

Rebuild the canonical asset into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/dugtrio_sculpt.py \
  --output target/dugtrio-rebuild/dugtrio.glb
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/dugtrio-rebuild --only DUGTRIO
```
