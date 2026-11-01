# Tangela sculpture and rooted vine rig

`tools/tangela_sculpt.py` retains the round blue tangle, clear dark eye opening,
rooted white ovals with tiny high pupils and broad red shoes. All 41 closed
vines remain: individual front curls, layered front weave, four eye-frame
ropes, nine crossing rear wraps, four side curls and four rear curls. The side
curls turn farther around the body for a clearer profile. Hidden root centers
are seated inside the actual core, including lateral roots whose old caps
only grazed the surface. They blend back into the authored outer loops.
No external geometry or rig is imported.

`tools/tangela_glb.py` exports the canonical `tangela.glb`: all 49 logical
solids packed losslessly into 10 runtime material groups, 17,318 vertices,
31,236 triangles, 39 joints, 162 accessors and 896,856 bytes. Redundant radial
and path sampling is reduced within the unchanged skin limits. Smooth rope
normals preserve curved strands while the fitted dark opening adapts to the
stored core. Authoring height remains 1.00; verified pack height is 0.9906 m.

Logical neutral geometry digest:
`e8705e62e57b9a088c5df2f38d475e0ec2895b8311a504ef065d2e53f0527064`.
Packed runtime digest:
`9122c50cf61e0829b7d88147233af06cdfc8e5ba7ee98d917c864bc2b918b416`.

The body, face, eyes, pupils and four framing ropes share a coherent carrier.
Each of the other 37 strands has a separate carrier rooted on its own authored
path. Its influence blends from the fixed embedded root to the free curled
section. Idle phases differ across all 37 vines. Attack and hit follow source
cue timing and return to neutral endpoints. Red shoes remain fixed in battle;
world walking uses two shoe chains and preserves every body/vine influence.
Material packing keeps each logical solid's geometry, indices, material and
weights. Builder, asset readers and production registry use the GLB; the
obsolete static JSON is removed.

All 16 combined Python checks pass for exact geometry/packing and schemas,
closed outward strands and shading, uncrossed curves, fitted face, actual
occupied root witnesses, fixed root-cap owners, shoe/eye attachments, both
pupils remaining unobstructed by every vine through motion, fixed floor,
neutral endpoints, winding/strain, isolated articulation and distinct idle
phases. Detached-root controls remain active. Three focused Rust checks pass
for GPU activation at verified pack height, runtime owners and all 37 vine
carriers, animated floor/culling and two world support chains through 64
phases with all vine influences preserved. Front, side and rear poses of all
clips were inspected; Blender exports are byte-identical. Shiny checks remain
skipped at the user's request. Browser visual acceptance remains open.

Both shared all-species geometry and exact source-cue pose timing checks pass.
The rebuilt native Route 29 Tangela/Pidgey 3D/2D pair was inspected at preserved
state: frame 2551, checksum 3912378689, LCD frame 3214 and virtual time
56.338423042. The full tangle, clear pupils, red shoes and correct TANGELA HUD
remain readable; both shoes meet the terrain. Normal terrain admission passes
at original pack size and the existing -110-degree review angle, with normal
size/support/omission and whole-body/frustum checks active. No shader errors
were reported. Ignored evidence is `target/pokemon-native-review/tangela-normal`.
Grass height is untouched. Browser visual acceptance remains open.
