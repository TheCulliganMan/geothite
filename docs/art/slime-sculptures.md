# Grimer, Muk and Ditto sculptures

`tools/slime_sculpt.py` authors three original canonical battle JSON assets.
The battle-species builder dispatches each identity through this pure geometry
recipe, replacing the old shared stacked spheres. Each retains its 0.96 authoring
height and the normal runtime species scale. Other species remain unchanged.

Each species has one closed continuous body with flowing rippled contours and
a broad flat underside. Ripples blend gradually into that underside instead of
producing an abrupt seam. Grimer has a tall open mouth, a raised true annular
lip, pale eyes with tiny pupils, a blue-gray lower tongue, two forward arms,
and rounded three-finger hands. Muk has a broader sagging puddle, darker muted
color, a wide mouth, narrow angled eyes, one long sagging arm and a shorter
opposite hand. Both have soft supported folds on the front and rear.

Ditto has an asymmetric rounded blob with three upper lobes, a soft spread
base, tiny fitted eyes, a gently curved smile with a pink interior, and three
soft lavender highlights. It has no separate ears, arms, sphere skirt or sludge
mouth rim. Its smile interior follows the actual concave smile outline rather
than assuming an ellipse will fit inside the thin curve.

The official [Grimer](https://assets.pokemon.com/assets/cms2/img/pokedex/full/088.png),
[Muk](https://assets.pokemon.com/assets/cms2/img/pokedex/full/089.png) and
[Ditto](https://assets.pokemon.com/assets/cms2/img/pokedex/full/132.png) artwork
was inspected for shape, anatomy and color. No reference image is shipped.
Downloadable [Grimer](https://sketchfab.com/3d-models/grimer-6b86d9e0fa824ebd81aef8d7ab9d43cd),
[Muk](https://sketchfab.com/3d-models/muk-512043dacaf74c548834455ac922b450),
and [Ditto](https://sketchfab.com/3d-models/ditto-47a04e8d40dd49689bcc6158f1f8a821)
listings advertise CC Attribution and zero animations, but their metadata does
not establish usable original rigs. Zero animations does not prove an absence
of a skeleton. The official download endpoints returned HTTP 401 without
authentication. Other Ditto listings checked were
`5a67ff026dac49818f4b0e63be8f5384` and `fb5b9e441d60453da296b6e12643fcba`;
the latter is a plush scan. No external mesh was imported. These authored
meshes are static.

Stored-mesh regressions check closed outward geometry, finite unit normals,
exact recipe parity, heights, broad flat grounding, budgets, distinct anatomy,
actual solid intersections at arms, palms and individual fingers, fitted face
vertices, front triangle clearance, true open mouth-rim centers, supported
front and rear folds, and Muk's asymmetric arm span. Native captures establish
presentation, not gameplay. The full 251-species visual goal and usable
open-access rig search remain active.

All eleven stored-mesh checks passed for the final assets. The spatial ray index
uses stored triangle projections and keeps the original barycentric inclusion
test. A separate comparison checks it against the original unindexed oracle
across 378 body and mouth sample rays; all marking vertices and triangle
centroids retain their original verification coverage. Ditto's asymmetry check
measures its upper side bulge rather than the broad puddle sole.

| Species | Parts | Triangles |
| --- | ---: | ---: |
| Grimer | 26 | 42,244 |
| Muk | 26 | 27,844 |
| Ditto | 8 | 13,980 |

The Rust `every_normal_species_has_a_bounded_reusable_visible_model` regression
passed for all 251 actual loaded species meshes. Canonical layout and storage
checks passed (569 raw JSON documents at verification time). The final meshes
were reviewed from front, rear and oblique angles. Native battle verification
is complete for this batch: the fresh Bevy example built successfully and
captured GRIMER, MUK and DITTO with the ignored local content pack. Each actual
2000×1400 battle capture was inspected. Mouth rims, fitted eyes and fingers,
Muk's long arm, and Ditto's small blob and face were reviewed at normal species
scale. The canonical meshes stayed unchanged from the eleven passing model
checks. These captures establish presentation, not gameplay. These catalog checks do not establish completed visual review of
every remaining species.

Temporary reference images, software reviews and verification logs were removed.
