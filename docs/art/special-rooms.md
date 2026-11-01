# Original special-room geometry

The special-room kit contains 23 independently authored, editable Blender assets.
It provides geometry for source families used by the Battle Tower battle room and lift,
Blackthorn Gym's upper puzzle, department-store lifts and rooftop terrace,
Celadon prize exchange and mansion roof, mobile battle/trade rooms,
Colosseum/Time Capsule/Trade Center, Dragon Shrine, Fighting Dojo, bicycle shop,
and Rocket Base B2F. Source maps, pixels, content packs, scripts, and collision
records are never copied into these assets or committed here.

The source scene retains separate named collections, objects, and materials.
Rebuild the canonical runtime exports with:

```sh
blender -b --python tools/build-special-interiors.py -- target/special-interiors
python tools/check-special-rooms.py target/special-interiors
```

Reconstruct `special-interiors.blend` through `tools/johto_art_sources.py`.
The runtime importer also accepts the repository's bounded, hash-checked,
lossless compressed model documents. Compression does not alter geometry.

## Identity and placement

`mesh/special_room_bindings.rs` holds compact, non-reversible fingerprints of
individually inspected complete drawings. The binding includes the exact map,
atlas, dimensions, metatile identity, native subtile column/row, and tile index
for every source cell. Runtime matching searches the whole visible source grid;
it does not hard-code an object's world position. Missing cells, changed art,
changed block identities, shifted native phase, unknown atlases, and unknown maps
refuse the complete object. Surface and modular enclosure fingerprints are
separate from compound apparatus, furniture, statues, and architecture.

The long shrine rails retain their original narrow floor plots. The two mobile
machines have separate asymmetric battle/trade face geometry and open footwells.
Bicycles have modeled rims, spokes, frames, saddles, forks, pedals, chain drives,
and three native orientations. The link rooms have complete octagonal horizontal
platforms, distinct apparatus, paired round pedestal stools, and a separate rear trade receiver.
The arena emblem is a horizontal inlay, never a sphere. Dojo guardians reuse the
original guardian sculpture only after the full four-course source statue is
verified. The terrace viewers are west-facing binocular fixtures with paired
barrels, a fork, a pedestal, and a rectangular foot. Their complete source plots
also retain the east parapet strip. Rocket Base's observed round instruments use
separate one-pair and two-pair closed housings with horizontal bands; no shelves
or filed cartridges are inferred. The central housings retain their preceding
two native rows of live track imagery. These shape corrections retain the
original whole-object fingerprints and source footprints. Their named material
groups contain only a source-pixel coordinate: runtime resolves each coordinate
through the current source tile UVs, preserving active palette colors without
embedding game artwork. Matching native 2D captures establish the observed blue
binocular housing and pale lenses, neutral circular instruments, and round pale
stool seats. Projected dimensions and native 3D appearance still require a paired
capture; a grayscale source reconstruction is not a color verification.

The bicycle shop's northern source ends inside two wheel drawings. Those
explicit boundary signatures emit only the visible lower wheel arcs, with no
inferred off-map bicycle frame; they are labeled source-clipped fragments.

## Openings and authority

The three Blackthorn drop shafts have actual open apertures and recessed side
walls; no polygon spans their top. Stair flights descend from the existing actor
datum. Doorways and cabin thresholds remain visually open. The central roof
inset's function is unknown: it retains its complete live source drawing on a
shallow floor and border, with no invented descending stair flight. The repeated
Rocket Base barrier likewise retains all twelve native cells and their live
pattern on a shallow continuous strip; it is not replaced by an upright door.
Room floors are shallow fitted paving, wood, mats, or corridor bands. Their
labels explicitly say `surface` and must not be reported as whole-prop coverage.
Meaningful source lettering and sign imagery is sampled from live UVs on framed
surfaces rather than copied into an asset catalog.

All binding and emission is render-only. Gameplay collision, warp locations,
script state, actors, and footing heights are unchanged. Tests separately cover
whole-object rejection, native phase, crop/unknown refusal, floor labels, open
shafts, descending stairs, triangle winding, finite normals, and untouched
footing. Validate the production per-map coverage and native camera views with
the ignored current pack after integration; a positive floor count alone does
not prove a room's objects are complete.
