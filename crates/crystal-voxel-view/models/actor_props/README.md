# Original volumetric actor kit

68 independently keyed runtime sources: all 27 non-walking-human sprite families
(including the mounted trainers and Biker), all 38 resolved icon families, and three
canonical starter rig sources. These contain authored geometry and linear
material colors only. They contain no sprite pixels, extracted textures, ROM
content, or game logic.

- Object identity comes from the host's **resolved source**, after variable
  sprites and species-to-icon resolution. `monster` and `icon_monster` are
  distinct model keys; unknown sources retain the existing presentation
- Consoles have different cases, slots, switches, sockets and controllers;
  trophies have bowls/rims/handles; balls have separate shells/seams/buttons;
  strength boulders and breakable rocks have different fractured silhouettes
- Creature families have individually authored anatomy, appendages and face
  details. Shared family icons are deliberately family representations, not a
  claim that every species using one has a bespoke species model
- Both bicycle models use the original trainer rig geometry with hierarchical
  seated hip/knee and gripping shoulder/elbow poses, attached to complete
  bicycles. The current prop presentation is a fixed riding pose, not a
  pedaling animation
- Source meshes use +Y up, front +Z, actual lowest vertex at Y=0. All placements,
  facing, visibility, source changes, and floor height remain host-owned
- Battle and visible exact-species actors share the canonical battle-species
  registry. Missing species retain their original 2D art; smaller icon or
  overworld meshes never substitute for a missing canonical body. The retired
  Pidgey, Rattata, Sentret and Hoothoot alternative exports and builders were
  removed. Chikorita, Cyndaquil and Totodile GLBs remain canonical rig sources.

## Editing and regeneration

`tools/build-actor-props.py` is the editable procedural authoring source. It
requires Blender 4.3 or later and the repository's original trainer rig JSONs.
It does not read any game pack, images, external model library, or network data.

```
blender -b --threads 2 --python tools/build-actor-props.py -- /tmp/actor-props --skip-preview
```

The generator writes each runtime JSON and seven small self-contained editable
Blender scenes, `actor-props-01.blend` through `actor-props-07.blend`. Every prop
part remains named and editable, with one floor-root empty per asset. Batches
bound the scene's memory and dependency-graph cost. Without `--skip-preview`,
the generator also renders each source batch's studio preview.

Canonical runtime exports live here. Editable native `.blend` files are maintained by
the repository's shared art-source storage helper; update those files
after changing the geometry. Temporary previews are not build inputs.

The smashable `rock` uses the focused editable source
`art/johto/source/actor-props-rock-refinement.blend`. Its closed 98-triangle
body has a broad 0.64 × 0.54 footprint and retains its 0.43 height and floor
root, fitting within one source actor cell. Strength boulders retain their
taller, separate silhouette. All outdoor placements use this same canonical
rock with their current native palette; there are no town-specific meshes.
Rebuild it with `blender -b --python tools/build-actor-props.py -- target/actor-props-rock --only=rock --skip-preview`.

## Verification

```
python3 tools/check-actor-props.py
cargo test -p crystal-voxel-view actor_props
```

The Python check proves generator/registry/export agreement, 65 exact source
families, three canonical starter rig sources, nonempty full-volume geometry, valid indices,
finite normalized vertex normals, consistent triangle winding, positive solid
volumes, floor grounding and bounded unit scale. Rust tests validate source
namespace separation, rejection of inferred aliases, canonical exact-species selection,
all imported meshes, and malformed-export rejection. These checks do not
replace gameplay, visual or interaction testing.

Surf always uses the exact Pokémon retained by the field move, with the trainer
posed on its animated body. The generic Surf creature and preset surfing-Pikachu
board assets and generators are retired. Source Surf sprite IDs remain part of
the faithful 2D presentation.
