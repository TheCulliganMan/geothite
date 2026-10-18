# Clefable sculpture

`tools/clefable_sculpt.py` authors the original canonical battle asset
`models/battle_species/clefable.glb`. The battle-species builder routes
Clefable through this recipe while preserving the remaining Wigglytuff recipe
and the other fairy sculptures. Clefable retains its authoring height of 1.0;
the runtime uses the normal species scale.

The replacement has a continuous round pear body, broad swept pointed ears,
muted brown tips fitted to both front and rear surfaces, a soft rolled forelock
with a supported curl, small dark eyes with ivory highlights, warm cheeks, a
gentle open grin, raised hands with pink fingers and ivory claws, grounded feet
with fitted toenails, broad three-point pink rear wing webs, and a thick curled
tail. The wing webs are continuous closed concave surfaces, with physical roots
in the body. The rounded ear surfaces reuse the repository's original pillow
geometry helper without modifying Jigglypuff's asset or recipe.

The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/036.png)
was inspected for shape and color. No reference image is shipped.
The downloadable [Clefable listing](https://sketchfab.com/3d-models/clefable-3e2a1055b4d24664a31fb032b20c146e)
advertises CC Attribution and zero animations. Its metadata did not establish a
usable original rig. Zero animations does not establish an absence of a skeleton;
the official download endpoint returned HTTP 401 without authentication, so the
source could not be inspected. No third-party geometry was imported. The initial local
sculpture was static; the current canonical asset uses an original standard skin
with the production verification recorded in [the skin guide](fairy-pair-skins.md).

The stored-mesh regressions check closed outward geometry, finite unit normals,
exact recipe parity, height and grounding, distinct anatomy, stored-triangle
fit of facial layers and both sides of the ear caps, visible triangle clearance,
physical appendage roots, and the fitted forelock curl. Multi-angle renders and
native captures establish presentation only, not controller or MCP gameplay.
The full 251-species visual goal remains active.

Final verification passed all eight sculpture regressions, the runtime-mesh
regression loading all 251 species, canonical layout checks, and storage
validation for the current 569 JSON documents. The reviewed candidate matches
the canonical asset byte-for-byte. Front, rear and oblique renders and the
rebuilt native battle capture were inspected with the ignored local
`content-packs/realtime-clock.browser.crystalpack`. The capture retained the
normal species scale. The final model has 32 closed parts and 32,136 triangles.
An initial check found ear-tip triangles below the curved support; the final
front and rear caps clear those surfaces. Temporary references, renders and
verification logs were removed after review.


## Current canonical skin

The neutral sculpture now has 15,603 vertices and 27,448 triangles. Four
brown ear caps conform to actual parent triangles, fixing an edge-clearance
defect missed by the original centre-only check. Numerical zero signs are
normalized for portable exports. An eleven-control canonical standard skin
adds independent ears, arms, forelock, wings and two tail carriers, while
retaining the coherent body/face and planted feet. Production registration and native review are complete for the scoped scenes. [The fairy skin guide](fairy-pair-skins.md)
records current geometry, source/skin checks, hashes and remaining gates; the
older verification figures above describe the initial static review.
