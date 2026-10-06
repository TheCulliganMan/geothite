# Original volumetric actor kit

73 independently keyed runtime meshes: all 28 non-walking-human sprite families
(including both mounted trainers), all 38 resolved icon families, and seven
additional exact battle species. These contain authored geometry and linear
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
- `battle_species_mesh` selects only explicitly authored species. Seven new
  models cover Chikorita, Cyndaquil, Totodile, Pidgey, Rattata, Sentret, and
  Hoothoot. Other exact named species can reuse their dedicated creature model;
  generic `fish`, `bird`, `monster`, etc. never stand in for an arbitrary species

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

## Verification

```
python3 tools/check-actor-props.py
cargo test -p crystal-voxel-view actor_props
```

The Python check proves generator/registry/export agreement, 66 exact source
families, seven extra species, nonempty full-volume geometry, valid indices,
finite normalized vertex normals, consistent triangle winding, positive solid
volumes, floor grounding and bounded unit scale. Rust tests validate source
namespace separation, rejection of inferred aliases, exact-species fallback,
all imported meshes, and malformed-export rejection. These checks do not
replace gameplay, visual or interaction testing.
