# Original Magby live-coal sculpture and skin

`tools/magby_sculpt.py` authors the original sculpture;
`tools/magby_glb.py` exports and strictly reads the canonical standard GLB in
`crates/crystal-voxel-view/models/battle_species/magby.glb`. The duplicate static
JSON is retired. No downloaded model is used.

A compact pear body carries a pointed belly patch fitted to stored triangles.
A dark collar bridges the body and broad face. Five embedded rounded lobes form
a low cloud crown. Tall white eyes, small vertical pupils and tiny eye sparks
follow the face. Raised curved arms end in three white claws per hand. Shins
support small grounded feet with two claws each. The pointed curved tail has
no flame. Normal colors use the local pack's RGB555 palette: purple-pink
(189,16,90), warm orange (255,148,74), white and black.

The bill is a closed annular taper with an open front aperture and a recessed
dark oral backing. Stored geometry checks sample five positions against all
parts, require an unoccupied front recess, and reject a deliberately filled
negative. This establishes a bill recess, rather than a tunnel through the head.

The skin contains 38 parts, 19,764 vertices and 35,032 triangles in 766,584 bytes.
Twenty-nine parts preserve exact previous neutral f32 geometry. Four crown
lobes and the belly/pupil/glint surfaces use lower sampling to fit the existing
20,000-vertex skin limit; the central crown lobe preserves the authoring span.
Floor zero and authoring height 0.77 remain unchanged. Production uses the local
Pokédex height `204` (0.7112 m). The neutral geometry digest is
`aab2245cffeaa6b35e780fc79dee5fa83f7d6b482103151f3e1cfd6acec8c60b`.

Eleven joints articulate the torso, neck, head, coherent five-lobe crown, raised
arms, two-stage tail and leg middles. Feet and toe claws stay fixed. The fitted
belly follows the body field, and the fitted gaze, bill and mouth backing share
the head owner. Hand claws stay with the hands. Idle, attack and hit clips rotate
joints with neutral endpoints; production battle cues control attack/hit timing.

```sh
blender -b -t 2 --python tools/build-battle-species.py -- /tmp/geothite-magby --only MAGBY
python3 -m unittest discover -s tools -p test_magby_sculpt.py
python3 -m unittest discover -s tools -p test_magby_glb.py
```

All ten sculpture checks and seven skin checks pass. They cover closed connected
outward surfaces, normals, canonical bytes, dimensions, anatomy, face fitting
and clearance, actual neutral and posed attachment volumes, fixed feet/claws,
neutral clip endpoints, winding, edge strain, and open-bill/filled/detached
negative controls. Blender regeneration is byte-identical. Front and rear
software views were inspected. Both focused Rust tests, the native client build,
and the wider catalog grounding/elapsed battle pose regressions pass. The normal
paired review passes. Shiny verification is skipped at the user's request. These checks do
not establish TUI or MCP gameplay.

The matched normal native 3D/2D review against Sentret passes. Magby's planted
feet/claws, compact body, fitted bib, open bill, raised clawed hands, lobed crown
and pointed tail remain readable at the verified scale. The terrain is accepted;
the paired receipt preserves core frame 1123, hash 854216493, LCD frame 1693,
time 29.498578709 and `state_preserved=true`. Front/rear software views cover
surfaces concealed by the production profile. Shiny checks remain skipped.
