# Ponyta sculpture and rig

`tools/ponyta_sculpt.py` retains the slender cream pony, arched neck, youthful
horse head, long modest muzzle, pointed pink-lined ears, fitted dark eyes and
white glints, subtle nostrils and closed smile. Four shaped legs end in
planted dark blue hooves. Ponyta has no horn or open mouth cavity. Seven
closed fire banks grow from the forehead, mane, tail and four lower limbs;
fourteen yellow layers follow both actual faces. No external geometry is
imported.

`tools/fire_horse_glb.py` exports the canonical `ponyta.glb`: 47 logical
solids packed into 40 runtime material groups, 15,002 vertices, 22,280
triangles, 14 joints, 200 accessors and 637,600 bytes. Authoring height remains
1.10; verified pack height is 0.9906 m. Redundant interior subdivisions are
reduced within the existing skin limits. Adaptive fitting refines the fire
cores and gaze where their actual parent needs it. Core shading follows the
underlying fire normals, avoiding diagonal highlights across the broad cores.
The shared adaptive helper retains its previous default subdivision level for
other sculptures.

Logical neutral geometry digest:
`e08134f1e01625a819c2a44fab4b227c4658caf579bd33ef1c4b4e52832d342e`.
Packed runtime digest:
`586c58b6e344ad355eed3521ceeb585a241b17255a01a5ae26bb2fb4f3b185ce`.

The continuous neck blends torso, neck and coherent head. Ears, tail stem,
tail plume, mane and forelock have independent carriers. Ankle fire and its
two core faces share the matching leg carrier. Battle hooves stay fixed;
world walking uses four complete leg chains, carrying each ankle bank with
its limb. Idle, attack and hit retain source-cue timing and neutral endpoints.
The builder, asset readers and production registry use the canonical GLB; the
obsolete static JSON is removed.

All 33 combined Python checks for both horses pass, including the existing
closed-solid, winding, fitted-layer, attachment-root, uncrossed-surface and
mouth-aperture checks. Decoded skin checks cover exact logical/packed geometry,
weights, independent articulation, roots through motion, both posed core faces,
floor contact, edge strain and source-cue clip contracts. Front, side and rear
poses of all clips were inspected, and Blender exports are byte-identical.
Shiny checks remain skipped at the user's request. Browser visual acceptance
remains open.

Four focused runtime checks for both horses pass, including GPU activation at
verified pack heights, independent carriers and animated culling, plus four
complete world leg chains through 64 phases. Both shared all-species geometry
and exact source-cue pose timing regressions pass.

The rebuilt native Route 29 Ponyta/Hoppip 3D/2D pair was inspected at preserved
state: frame 2063, checksum 971999260, LCD frame 2668 and virtual time
46.357391624. The complete pony silhouette, face, flowing fire and correct
PONYTA HUD remain readable; four hooves meet the terrain. Normal admission at
the original pack size passes with the existing -110-degree review angle.
No shader errors were reported. Ignored evidence is
`target/pokemon-native-review/ponyta-normal`. Grass height is untouched.
