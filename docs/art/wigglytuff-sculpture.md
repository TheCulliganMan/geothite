# Wigglytuff sculpture

`tools/wigglytuff_sculpt.py` authors the original canonical battle asset
`models/battle_species/wigglytuff.mesh.json`. The battle-species builder routes
this identity through the pure geometry recipe. Other fairy assets retain their
own recipes and geometry. Wigglytuff retains its authoring height of 1.0; the
runtime retains the normal species scale.

The replacement has a continuous tall pink pear body, a fitted broad ivory
belly, long swept rabbit ears with rounded cross-sections and charcoal linings,
short rounded arms, grounded splayed feet, a rolled forelock with a fitted curl,
and large ivory-framed blue eyes with lower turquoise reflections and bright
highlights. Its open smile is fitted to the actual belly surface, and its pink
interior is fitted to the mouth. This prevents the white belly from hiding the
face. The rounded ear helper is reused without changing Jigglypuff's recipe or
asset.

The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/040.png)
was inspected for shape and color; no image is embedded or shipped.
The downloadable [Wigglytuff listing](https://sketchfab.com/3d-models/wigglytuff-f52a63c516dc40eaa899c6aa522177f2)
advertises CC Attribution and zero animations. Its metadata did not establish a
usable original rig. Zero animations does not prove an absence of a skeleton;
the official download endpoint returned HTTP 401 without authentication, so its
source could not be inspected. Other checked listings were
`331560a5efa646acbaaeeee735590ad3` (a Drawfee variant) and
`1bb0007369d74d0bad20b56ae377af1c` (Version1); neither metadata established a
usable original rig. No external geometry was imported. This local asset is
static and does not claim skeletal animation.

Stored-mesh regressions check closed outward surfaces, normals, exact recipe
parity, height and grounding, distinct anatomy, actual parent-triangle fitting,
visible triangle clearance, physical roots, the supported forelock curl, white
belly coverage over the actual front torso, and ear extension beyond the body.
Native captures establish presentation, not controller or MCP gameplay. The full
251-species visual goal and accessible-rig search remain active.

Verification completed with 24 parts and 48,060 triangles. All ten stored-mesh
regressions passed. The fitted belly uses four subdivision levels and a measured
0.008 authoring-depth offset so its front triangle interiors clear the torso;
checking vertices alone had missed small intersections. Front, oblique, rear,
and native battle views were inspected after the correction. The native example
built successfully and captured WIGGLYTUFF from the ignored local content pack.

The Rust `every_normal_species_has_a_bounded_reusable_visible_model` regression
passed and loads all 251 actual species meshes. Canonical layout validation and
storage validation passed (569 raw JSON model documents at verification time).
These catalog checks establish loadability and geometry invariants; they do not
replace individual visual review of the remaining species. Temporary previews,
logs, and downloaded reference artwork were removed.
