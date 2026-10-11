# Sunkern and Sunflora sculptures

`tools/sunflower_sculpt.py` authors original canonical Sunkern and Sunflora
meshes. The battle species builder dispatches these recipes before Blender
batching and rejects their old shared paths. Bellossom retains its existing
recipe. Both species keep their 0.95 presentation height and floor origin.

Sunkern has a rounded gold seed shell with eight serrated crown teeth, an
actual open rim and a recessed interior floor. A thin sprout grows from below
that floor into two broad green leaves with fitted midribs. Four broad dark
coat panels, two front and two rear, follow the seed surface and end in jagged
upper edges. Large black button eyes have ivory catchlights and a rounded pink
grin follows the composite shell/coat surface. The asset has 18 closed parts
and 16,524 triangles.

Sunflora has a continuous plump green bulb and rising stem, a broad pale-yellow
flower disk and a cheerful closed-eye expression with a rosy open grin. Eight
large curved pointed petals overlap eight smaller rear golden bracts. The
asymmetrical leaf arms include a raised curled tip and fitted midribs. Curved
root legs join two grounded broad green foot pads. The asset has 30 closed
parts and 13,696 triangles.

The shared `fitted_contour` helper now accepts an optional subdivision count.
Existing recipes retain its two-level default. Sunkern uses three levels for
its highly curved coat panels and tongue; stored-triangle clearance checks
caught and eliminated their initial shell/parent intersections. Existing Aipom
regressions verify unchanged default regeneration and geometry.

Silhouette and palette were checked against the
[official Sunkern artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/191.png)
and [official Sunflora artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/192.png).
External search found [Sunkern](https://sketchfab.com/3d-models/sunkern-be39b7cd8eaf4879a0a732ae24250f8d),
[Clay Sunkern](https://sketchfab.com/3d-models/clay-sunkern-b71cd4da1f5f437591529321ad427367)
and [Sunflora](https://sketchfab.com/3d-models/sunflora-3c78b210bcd749f28b89e859586146bd)
listings with CC Attribution licensing. Their search metadata did not establish
an original usable rig. No external mesh or textures were imported. These are
original static meshes on the native base-color path, with no rig or animation
claimed.

`python3 -m unittest discover -s tools -p test_sunflower_sculpt.py` checks
closed topology, winding, connectivity, unit normals, deterministic regeneration,
height, budget, actual crown/cavity geometry, sprout and leaf roots, fitted coat,
eye, mouth and tongue vertices and front triangle clearance, fitted midribs,
every petal/bract root and both supporting leg/foot junctions. Software views
and native battle captures check model presentation rather than gameplay behavior.

Final verification passed all ten sunflower geometry regressions and all seven
Aipom shared-helper regressions. Front and rear software views and both final
native battle captures were visually inspected. The Rust catalog regression
passed for the 220-species battle catalog; it checks grounding and volume, not visual quality.
Model layout and storage validation passed for all 571 JSON assets. Temporary
review images and build logs were removed after inspection.
