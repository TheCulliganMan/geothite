# Rhydon sculpture and rig

`tools/rhydon_sculpt.py` authors a heavy gray biped with a continuous torso,
four cream belly bands, pointed neck armor, broad skull and muzzle, open
fanged jaw and pink tongue. Tall ears, narrow red gaze, six cheek points,
rounded shoulders, powerful arms, three claws per hand, stocky haunches and
two claws per foot retain the species silhouette. Its nasal drill has an
actual three-turn raised spiral. Three back points and fitted annular collars
finish the thick curved tail. No external geometry or rig is imported.

`tools/rhino_glb.py` exports the canonical `rhydon.glb`: 64 authored solids
packed into 48 runtime material groups, 18,156 vertices, 28,304 triangles,
15 joints, 242 accessors and 741,340 bytes. The source mesh previously exceeded
the skin budget; reducing redundant surface sampling preserves every authored
solid, the spiral turns and shared tail/collar curve within the existing
limits. Authoring height remains 1.40; verified pack height is 1.905 m.
The logical neutral digest is
`f65fdcc84a808bd430ba1ce4ed03602ec0301a6bb0905722850774e49f1248c9`;
the packed runtime digest is
`c0131e43f7235c93fc5a645ab0c3e69f63dc8c2f8d8aa36e0134cc89ab22b4f7`.

Independent jaw, ears, arms, hands and drill carriers accompany a coherent
head and blended body. The two tail carriers deform the tail and its collars
using the same source arc. Legs blend into the torso while foot pads and toe
claws retain floor support. Left and right foot-claw groups remain separate
for world walking. Idle, attack and hit use the source-cue clocks and return
to neutral endpoints. Builder, asset reader and runtime routes use the GLB;
the obsolete static JSON is removed.

All 31 combined Python checks for both rhinoceroses pass. These include true
open-jaw samples, inner collar occupancy and outer clearance through motion,
occupied attachment roots with a detached negative control, fitted markings,
weights, isolated joint motion, floor contact, winding and strain. Four focused
Rust checks pass for GPU activation at verified pack sizes, runtime owners and
world walking support chains. Front, side and rear poses of all three clips
were inspected, and Blender exports are byte-identical. Shiny checks are
skipped at the user's request. Browser visual acceptance remains open.

The rebuilt native Route 29 Rhydon/Pidgey 3D/2D pair was inspected at identical
state: frame 2374, checksum 3422758645, LCD frame 3085 and virtual time
55.014122498. The drill, open jaw, claws, belly bands and complete curved tail
remain readable; both feet meet the terrain and the RHYDON HUD is correct.
Normal admission at the original pack size passes with the existing
-110-degree review angle. No shader errors were reported. Ignored evidence is
`target/pokemon-native-review/rhydon-normal`. Both shared geometry and
source-cue timing regressions pass. Grass height is untouched.
