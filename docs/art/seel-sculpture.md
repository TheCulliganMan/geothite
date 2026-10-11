# Seel sculpture

`tools/seel_sculpt.py` authors the canonical original battle asset
`models/battle_species/seel.mesh.json`. The battle-species builder dispatches
Seel through this pure geometry recipe. Dewgong retains its existing asset and
recipe. Authoring height remains 1.03, with normal runtime species scale.

A continuous reclining body narrows smoothly into its raised tail stem, ending
in two rounded three-lobed flukes. Two broad rounded front paddles replace the
old triangular fins. The broad ice-blue head carries a short tapered horn,
glossy fitted dark eyes with teal reflections, supported curved brows, a cream
bilobed muzzle, a dark mouth, a rounded lilac tongue with a supported central
fold, and two short curved ivory tusks. The nose is rooted on the actual muzzle
surface. The existing rounded ear perimeter helper supplies closed paddle and
muzzle surfaces through proper transforms, without changing Jigglypuff's asset.

The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/086.png)
was inspected for silhouette and colors, and is not shipped.
Two downloadable listings were checked: [Seel](https://sketchfab.com/3d-models/seel-26f797546dc445b48a0b0f5203d215bd)
and [SEEL](https://sketchfab.com/3d-models/seel-6f70737cfde341a3b4076f854a75dc7b).
Their metadata advertises CC Attribution and zero animations, but does not
establish a usable original rig. Zero animations is not evidence of an absent
skeleton. Both official download endpoints returned HTTP 401 without
authentication. No external mesh was imported; this authored mesh is static.

Stored-mesh regressions cover closed outward geometry, finite unit normals,
recipe parity, preserved height, grounded paddles, a bounded triangle budget,
distinct anatomy, actual solid root intersections, front marking clearance,
parent-triangle vertex fitting, supported brows and tongue fold, and the raised
split tail and broad low paddles. Native captures verify presentation only.
The full 251-species visual goal and usable open-access rig search remain active.

Verification completed with 23 parts and 21,120 triangles. All nine geometry
regressions passed. The original nose position and narrow paddle roots were
corrected after independent solid-intersection tests exposed gaps; the final
asset was reviewed from front, rear, and oblique angles and in the native Bevy
battle capture at normal species scale. The native example built and captured
SEEL successfully using the ignored local content pack.

The Rust `every_normal_species_has_a_bounded_reusable_visible_model` regression
passed for all 251 actual loaded species meshes. Canonical layout and storage
validation passed (569 raw JSON model documents at verification time). These
checks establish loadability and geometric invariants, not completed visual
review of every remaining species. Temporary captures, reference artwork, and
verification logs were removed.
