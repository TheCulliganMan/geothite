# Jigglypuff shared sculpture

`tools/jigglypuff_sculpt.py` authors the original shared
`models/actor_props/icon_jigglypuff.mesh.json` asset used by the existing battle
and overworld identities. The actor builder routes this one identity through
the pure geometry recipe. Other fairy assets retain their recipes. Its authoring
height remains 0.82; the battle renderer retains the species scale.

A continuous pink balloon supports short arms, splayed feet, pointed ears with
rounded cross-sections and fitted charcoal linings, a large angled rolled
forelock with a fitted crease, and a gentle curved smile. Large fitted eyes have
ivory whites, blue irises, turquoise lower reflections, and ivory highlights.
The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/039.png)
was reviewed as a shape and color reference; no image is embedded in the model.

A CC Attribution candidate advertised as fully rigged was found:
[Eleanie's Jigglypuff](https://sketchfab.com/3d-models/jigglypuff-371fa15534b84b9d8975e41627662783).
The author describes an Auto-Rig Pro skeleton, expression controls, a squish
control, and Blender EEVEE shaders. The listing reports 174,200 triangles and
4096-pixel textures. Its official download API returned HTTP 401; clicking
Download in Chrome displayed a Sketchfab login modal. No authenticated download
was available, so its source rig, export compatibility, and attribution contents
could not be inspected. This remains a candidate for integration once the source
is accessible. The current local sculpture is static and does not claim a rig.

Other checked candidates included the explicitly unrigged
`76ab579e21b74697a90efe566a7293c3` and the artist-authored
`bc17db79c2804b60a36e2bd66b421294` (CC Attribution-ShareAlike, separate parented
eyes and microphones; its download also required authentication). The SSBB
listing explicitly names Nintendo content and was not imported. No external
geometry or textures were shipped in this pass.

Stored-mesh regressions check closed outward surfaces, shading normals, recipe
parity, height and grounded feet, distinct anatomy, actual parent-triangle
fitting and front-face clearance, physical joins, and supported forelock creases.
The full 251-species visual goal and the accessible-rig search remain active.

Final verification passed all eight sculpture regressions, the runtime-mesh
regression loading all 251 species, canonical layout checks, and storage
validation for all 571 JSON documents. The reviewed candidate was copied
byte-for-byte to the canonical asset. Front, rear and oblique renders and the
rebuilt native battle capture were inspected. The capture used the ignored
local `content-packs/realtime-clock.browser.crystalpack`; it verifies
presentation, not controller or MCP gameplay. The model has 22 closed parts and
40,716 triangles. The initial ear design exceeded the 50,000-triangle budget;
the rounded concentric ear surfaces improved the shape and reduced geometry.
Temporary references, renders, and verification logs were removed after review.
