# Dunsparce land-snake sculpture and standard skin

`tools/dunsparce_sculpt.py` authors the anatomy and `tools/dunsparce_glb.py`
exports canonical `models/battle_species/dunsparce.glb`. The focused species
builder now has a dedicated canonical-GLB branch and removes duplicate static
JSON. All 19 parts retain the previous asset's exact stored f32 positions,
normals and triangle order: 11,833 vertices and 12,248 triangles. The normalized
0.85 vertical span and floor origin remain unchanged.

The continuous rounded head and tapering trunk share exact seam vertices with
the blue belly. Fitted oval eye rings, closed eyelids and creases, pointed jaw
barbs, small feather wings, back ribbons and the raised three-turn drill retain
their actual surface support. Local Crystal front/back sprites and normal
palettes were inspected: RGB555 gold (31,24,6), blue (8,15,23), white and black.
Gold is now sRGB8 (255,198,49), blue is (66,123,189), feather wings and eyelids
are white, and the drill groove/fitted feather folds use source blue. Geometry
helpers perform the sole conversion to linear material colors.

Seven joints use the common GPU skinning path. Yaw-only body motion keeps the
whole belly at its authored supporting height; the fitted face, jaw and back
ribbons share its rigid owner. Each complete feather wing and fitted fold share
one wing joint. An initial blended feather chain inverted a thin edge during
attack; rigid feather ownership preserves the actual feather shape while
retaining flapping motion. Three tail stages bend the stem and turn the drill
around the authored tilted base-to-tip axis. The cone and its complete spiral
share one owner. Idle/attack/hit clips have an identity root, neutral endpoints
and rotations only, with authoritative battle cues owning playback timing.

The local pack's height field 411 is 4′11″ (1.4986 m). The existing shared
`battle_layout::reference_span` rule measures Dunsparce along nose-to-tail Z;
this migration preserves that length interpretation and ratio-preserving scale.
The listed size is not applied as a vertical-height override.

The GLB is 460,848 bytes, with 99 accessors and seven joints, within existing
loader limits. Its rest digest is
`97c7da81fb99f3363b601244efa047a94585dd0fa8c4b6faad88f5537b4fde4f`.
Seven independent stored-skin and eight migrated anatomy checks pass, including
bind identity, canonical regeneration, source colors, grounded belly height,
fitted owners, wing/tail motion, tilted drill axis, winding/edge strain, closed
topology and actual surface support. Focused Blender builder output is
byte-identical and emits no static JSON. Neutral front and side software views
were inspected. Both focused production Rust checks pass: GPU skinning at the pack length and
off-grid clip sampling for grounded support, fitted owners and culling bounds.
The native build and matched normal Dunsparce/Pidgey review pass: core 2245,
checksum 2951808166, LCD 2916, time 52.583664208, state preserved. The full
longitudinal silhouette, grounded belly, fitted back bands, both feathers and
tilted spiral drill remain readable without clipping. Native rest-pose/rig/color
integration is accepted; exact source-frame motion remains open.
Exact source-frame eye/body motion, field gaits and browser review remain open.
Shiny-specific checks and captures are skipped at the user's request.
