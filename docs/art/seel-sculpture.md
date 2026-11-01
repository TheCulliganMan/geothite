# Seel sculpture and rig

`tools/seel_sculpt.py` authors the reclining ice-blue seal and
`tools/seal_glb.py` exports its canonical skin. The runtime, builder and checker
use `models/battle_species/seel.glb`; the legacy static JSON file is removed.
The continuous body narrows into a raised tail stem with two rounded
three-lobed fans. Broad low front paddles now carry four supported creases.
The rounded head retains its short horn, fitted glossy eyes with teal
reflections and bright glints, curved brows, cream bilobed muzzle, charcoal
nose, playful mouth, rounded lilac tongue and curved ivory tusks.

The [official Seel artwork](https://www.pokemon.com/us/pokedex/seel) informed
the shape and palette. No reference pixels or external geometry are shipped.
The closed rounded perimeter helper remains shared with Dewgong without
changing other species' assets.

| Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 27 | 13,304 | 21,680 | 10 | 137 | 559,936 | 1.03 | 1.0922 m |

The verified local pack supplies height digits 307 and classification SEA LION.
The neutral geometry digest is
`e1b358039ce8dba65f99bfde6d01438da48fa17f8532ea107117e290e7fae852`.

A floor root, blended body and coherent head carry the face. Two tail carriers
blend continuously along the rising stem; each fan has its own child carrier.
The front paddles have separate floor-rooted carriers that sweep around Y while
remaining level on the ground. Their creases share those carriers. The tongue
and its crease flex independently from the mouth. Idle, attack and hit use
rotation-only keys, neutral endpoints and the existing source-cue clocks.
World sliding preserves the low supports and authored appendages.

The combined Seel/Dewgong suite has 26 dedicated passing Python checks:
canonical bounded skins, hierarchy/binds/weights, anatomical owners, occupied
body/head/horn/muzzle/flipper/fan/canine/tongue roots through motion with a
detached negative control, floor contact, endpoints, winding, strain, local
articulation, actual posed gaze clearance, source cues, closed outward solids,
unit normals, recipe parity and size, fitted details, supported folds and
Dewgong's non-crossing body bend. The same run also passed 27 existing sea,
seahorse and Sentret checks. Four focused Rust checks pass for both GPU paths
at verified pack size, production routing/owners/floor/culling and independent
appendages, plus 64 phases of grounded world sliding. Three existing terrain
seam checks also passed in that run.

Front, side and rear poses of all three clips were inspected. Blender exports
are byte-identical. The existing 20,000-vertex, 40,000-triangle, 64-primitive,
256-accessor and 1 MiB limits remain unchanged. Shiny checks are skipped at the
user's request; grass height is untouched.

Rebuild and check:

```sh
python3 tools/seel_sculpt.py --output crates/crystal-voxel-view/models/battle_species
PYTHONPATH=tools python3 -m unittest test_seal_glb test_seel_sculpt test_dewgong_sculpt
```

The rebuilt production client passes normal Route 29 terrain admission at
original pack size. The inspected Seel/Sentret 3D/2D pair preserves core frame
4184, checksum 3435595935, LCD animation frame 4919 and virtual time
90.181717666. The complete raised tail and both fans remain in frame; the face,
cream muzzle, tusks, tongue and low paddles remain readable with the correct
SEEL HUD. The paddles meet the terrain. No shader errors were reported.
Final ignored evidence uses `target/pokemon-native-review/seel-normal`.

The existing supported -110-degree angle is selected with normal size, source
support, scenery omission and whole-body/frustum checks active. Both shared
all-species grounded/volumetric and exact source-cue pose timing regressions
pass. Browser visual acceptance remains open.
See also [Dewgong](dewgong-sculpture.md).
