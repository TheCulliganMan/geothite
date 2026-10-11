# Clefairy shared sculpture

`tools/clefairy_sculpt.py` authors the original
`models/actor_props/icon_clefairy.mesh.json` asset. Both existing overworld and
battle identities already consume this asset, so no extra species copy or new
runtime dispatch is needed. `build-actor-props.py` routes this identity through
the pure geometry recipe and preserves the other fairy and Jigglypuff recipes.
Its established authoring height remains 0.82.

The replacement has a continuous round pear body, pointed pink ears with fitted
brown tips, a rounded rolled forelock and supported spiral crease, narrow eyes
with ivory reflections, warm cheeks, an open smile with a small fang, raised
hands with three ivory claws each, grounded feet with fitted toenails, six soft
scalloped rear wing feathers, and a thick curled pink tail.

The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/035.png)
provided a shape and color reference. No image is embedded in the asset.
Downloadable Sketchfab listings checked were `44e5a8793036494d922ac0c91d6a4cfa`,
`0f836aeb73a44a73bcd08b0305975de6`, `1705a11c380e4fea8faf08307f02e9c9`, and
`5ef2b20dd37b4b72b5032b6bba2301c0`. They advertise CC Attribution and zero
animations. Their metadata did not establish a usable original rig; zero
animations does not prove an absence of a skeleton. The first listing's official
download endpoint returned HTTP 401 without authentication. No external geometry
was imported. This sculpture is static, without skeletal-animation claims.

Eight independent geometry regressions cover closed outward surfaces, normals,
recipe parity, grounding and height, anatomy, actual stored-triangle fitting,
front-face clearance, physical joins, and the supported forehead curl.
The shared-species Rust regression additionally loads all 251 runtime meshes,
including the 31 shared assets outside the 220-entry battle catalog, and checks
bounds, geometry indexing, attribute lengths, finite positions and unit normals.
This does not establish visual completion of the full catalog.

Final verification passed all eight sculpture tests and the strengthened
251-species runtime-mesh regression, together with canonical layout and all
571 JSON storage checks. Front, rear and oblique software views and the rebuilt
native battle capture were inspected. The capture used the ignored local
`content-packs/realtime-clock.browser.crystalpack` and retained Clefairy's
species scale. Native capture verifies presentation, not controller or MCP
gameplay. The model has 35 closed parts and 38,760 triangles. Temporary
references, renders and logs were removed after review.
