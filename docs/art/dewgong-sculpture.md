# Dewgong sculpture and rig

`tools/dewgong_sculpt.py` authors the adult white seal and `tools/seal_glb.py`
exports its canonical skin. The runtime, builder and checker use
`models/battle_species/dewgong.glb`; the legacy static JSON file is removed.
One continuous surface runs through its rounded head, forward-swept neck,
broad belly and hooked tail. The head and crown now sit farther forward for
a softer seal profile. Two broad scalloped front fans carry supported creases,
and the tail ends in two raised scalloped fans. Tall dark eyes, brown lower
reflections, bright eye glints, a white muzzle, small charcoal nostril, gentle
smile and two small ivory lower canines distinguish Dewgong from Seel.

The [official Dewgong artwork](https://www.pokemon.com/us/pokedex/dewgong)
informed the shape and palette. No reference pixels or external geometry are
shipped. The adult retains a white muzzle and has no protruding tongue.

| Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 22 | 15,206 | 23,740 | 9 | 117 | 649,548 | 1.03 | 1.7018 m |

The verified local pack supplies height digits 507 and classification SEA LION.
The neutral geometry digest is
`07d7f5f2e844cd65944949d81004e5de86d9493fb99bf220dbc9c552830ae5e9`.

A floor root anchors the lowest belly vertices. The continuous body blends
into a coherent head and two tail carriers. The front flippers articulate
independently with their creases, and each terminal fan has its own child
carrier. Eyes, reflections, glints, muzzle, nose, smile, canines and crown
follow the coherent head. Idle, attack and hit retain rotation-only keys,
neutral endpoints and source-cue clocks. World sliding preserves belly contact.

All 26 dedicated combined Seel/Dewgong Python checks pass, including actual
occupied appendage roots through motion with a detached negative control,
posed gaze clearance, fixed low support, winding and strain. The original
777 signed body rays still distinguish disjoint curved sections from nested
surfaces; a separate posed triangle-intersection test checks the adult body
in all three clips. Four focused Rust checks pass for both verified-size GPU
paths, routing/owners/floor/culling and independent appendages, and 64 phases
of grounded world sliding. Front, side and rear poses were inspected for all
clips. Blender exports are byte-identical. Existing geometry, primitive,
accessor and byte limits remain unchanged. Shiny checks are skipped at the
user's request; grass height is untouched.

Rebuild and check:

```sh
python3 tools/dewgong_sculpt.py --output crates/crystal-voxel-view/models/battle_species
PYTHONPATH=tools python3 -m unittest test_seal_glb test_seel_sculpt test_dewgong_sculpt
```

The rebuilt production client passes normal Route 29 terrain admission at
original pack size. The inspected Dewgong/Hoppip 3D/2D pair preserves core
frame 3470, checksum 2172026011, LCD animation frame 4201 and virtual time
75.831987126. The full curved body, raised stem, both tail fans and both front
flippers remain in frame. The gentle face, white muzzle, crown and lower
canines remain readable with the correct DEWGONG HUD. The belly meets the
terrain. No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/dewgong-normal`.

The existing supported -110-degree angle is selected with normal size, source
support, scenery omission and whole-body/frustum checks active. Both shared
all-species grounded/volumetric and exact source-cue pose timing regressions
pass. Browser visual acceptance remains open.
See also [Seel](seel-sculpture.md).
