# Rapidash sculpture and rig

`tools/rapidash_sculpt.py` retains the athletic cream unicorn, long arched neck,
proportioned skull and long muzzle, narrow red gaze, pointed ears and ivory
forehead horn. The separate upper snout, lower jaw, dark oral chamber and pink
tongue retain a real empty mouth opening. Three dark blue hooves support the
resting model; the bent left foreleg holds its hoof above the floor. Seven
closed fire banks grow from crown, mane, tail stem and four lower limbs, with
fourteen fitted yellow core layers on both sides. No external geometry or rig
is imported.

`tools/fire_horse_glb.py` exports the canonical `rapidash.glb`: 54 logical
solids packed into 47 runtime material groups, 17,555 vertices, 26,744
triangles, 15 joints, 228 accessors and 724,584 bytes. Authoring height remains
1.30; verified pack height is 1.7018 m. Redundant surface sampling is reduced
within the unchanged skin limits. Adaptive fire-core fitting preserves actual
clearance; core shading follows the underlying fire normals.

Logical neutral geometry digest:
`8e17f3827417a173789716bfc7a5d2af03b236876a0fa606eea76b8bb726d54f`.
Packed runtime digest:
`c56d7e051ad01b2f418fca26b70baa6f9c682bc2048ad11e790fee0401ec156f`.

The continuous neck blends body, neck and head. Coherent gaze, muzzle and horn
follow the head, with separate ears and jaw. Tongue follows the lower jaw.
Tail stem and plume, mane and crown have independent carriers. Ankle fire and
both core faces share each matching leg carrier. Three supporting hooves stay
fixed in battle; the raised foreleg articulates independently. World walking
uses four complete chains and carries the ankle banks with their limbs,
retaining the authored elevated foreleg. Idle, attack and hit use source-cue
timing and return to neutral endpoints. Builder, asset readers and production
registry use the GLB; the obsolete static JSON is removed.

All 33 combined Python checks for both horses pass, including actual empty
jaw samples during motion, fourteen fitted core faces, occupied attachment
roots with detached negative controls, closed outward geometry, shading,
weights, local articulation, fixed supports and the raised forehoof, winding,
strain and clip contracts. Front, side and rear poses of all clips were
inspected, and Blender exports are byte-identical. Shiny checks are skipped
at the user's request. Browser visual acceptance remains open.

Four focused runtime checks for both horses pass for GPU activation at verified
pack sizes, independent carriers, animated culling and four complete world
chains through 64 phases. Both shared all-species geometry and exact source-cue
pose timing regressions pass.

The rebuilt native Route 29 Rapidash/Hoppip 3D/2D pair was inspected at
preserved state: frame 3192, checksum 1320301839, LCD frame 3901 and virtual
time 70.527144961. Complete horn, raised forehoof, open jaw, flowing fire and
correct RAPIDASH HUD remain readable; the three supporting hooves meet the
terrain. Normal terrain admission passes at the original pack size and the
existing -110-degree review angle. No shader errors were reported. Ignored
evidence is `target/pokemon-native-review/rapidash-normal`. Grass height is
untouched.
