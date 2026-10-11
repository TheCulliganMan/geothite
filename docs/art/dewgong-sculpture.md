# Dewgong sculpture

`tools/dewgong_sculpt.py` authors the canonical original battle asset
`models/battle_species/dewgong.mesh.json`. The battle-species builder dispatches
Dewgong through the pure geometry recipe. Seel keeps its separate reclining
sculpture and canonical asset. Authoring height remains 1.03, with normal runtime
species scale.

One closed continuous body runs through the rounded head, long neck, broad
belly, and curling tail. Smooth elliptical sections follow the curved centerline;
the lower bend has enough room for its thickness without folding into itself.
Two broad scalloped front fans carry supported flowing creases, and the raised
tail ends in two broad curved flukes. The small tapered horn, tall dark eyes,
soft brown reflections, bright eye glints, rounded white muzzle, small charcoal
nostril, gentle smile and two short ivory lower fangs distinguish the adult
from Seel. It has neither a cream muzzle nor a protruding tongue.

The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/087.png)
was inspected for silhouette and colors and is not shipped.
The downloadable [Dewgong listing](https://sketchfab.com/3d-models/dewgong-08600e523c6f4d889f707c2435f7588e)
advertises CC Attribution and zero animations, but its metadata does not
establish a usable original rig. Zero animations does not establish an absent
skeleton. Its official download endpoint returned HTTP 401 without
authentication, so no external geometry was imported. This original authored
mesh is static.

Stored-mesh checks cover closed outward geometry, finite unit normals,
recipe parity, preserved height, grounding, triangle budget, distinct adult
anatomy, actual solid intersections at roots, front marking clearance,
parent-triangle fitting, supported smile and fan folds, fan placement, and
signed ray crossings across the body bend. The latter distinguishes separated
sections of the curved body from nested overlapping or inverted surfaces.
Native captures verify presentation, not controller or MCP gameplay. The full
251-species visual goal and usable open-access rig search remain active.

The ten stored-mesh regressions passed with 22 parts and 23,740 triangles.
The initial underside bend was widened and thinned after rear-view inspection
showed a crease; the final signed-ray regression checks the stored surface
independently across 777 sample rays. The final eye highlights were fitted
inside their actual tapered eye parents. Front, rear, and oblique reviews are
complete. Canonical layout and storage checks passed (569 raw JSON documents
at verification time). The Rust `every_normal_species_has_a_bounded_reusable_visible_model` check
passed for all 251 actual loaded species meshes. The native battle example
built successfully and captured DEWGONG using the ignored local pack. Its
normal species scale, white fans, muzzle, and small fangs were inspected in the
actual battle scene. These checks do not establish completed visual review of
every remaining species. Temporary captures, reference artwork, and verification
logs were removed.
