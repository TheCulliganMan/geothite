# Kingdra sculpture

`tools/kingdra_sculpt.py` authors an original sea dragon with a curved chest
and slender neck, a swept brow and lower jaw, a long tapered hollow muzzle,
rooted jaw horns, two tall blue crown stems with four curved branches, ruby
irises and fitted angular eye whites. Small rounded ivory neck paddles and
larger ivory dorsal sails have rooted blue support horns and forks, plus
surface-supported folds. A golden armored breast has five fitted plate seams;
three additional folds follow the actual rear torso surface.

The tail follows a smooth inward Archimedean spiral after a broad initial bend,
with its root hidden inside the body. An intermediate coil passed closure and
shading tests but showed a flat exposed root and a distorted bend in the
preview. The replacement passes an independent spatially culled triangle
self-intersection test. A negative-control check confirms that this test rejects
the earlier distorted coil. Refining the breast fitting resolved front-face
clipping caused by the narrowed tail transition.

The final model has 47 parts and 61,464 triangles, retains its 1.07 authoring
height and grounds at zero. The main builder routes Kingdra through the pure
recipe and removes the obsolete shared Blender seahorse construction. Horsea
and Seadra retain their previously tested dedicated recipe and model bytes.

All nine tests in `tools/test_kingdra_sculpt.py` passed in 13.133 seconds against
the final stored candidate. They check closed connected outward anatomy, actual
triangle shading, recipe parity, grounding, height, budgets, unique part names,
species identity counts, actual solid roots at the body/head/tail/muzzle/bore/
floor/jaw/crown/branches/neck fins/dorsal fins/support horns/forks, fitted details
against actual stored parent triangles, front-face clearance, a genuinely open
tapered snout with a deep chamber, crown proportions, an inward tail turn and
absence of proper nonadjacent tail-face crossings.

Final front, oblique and rear software previews were inspected. The canonical
file matches the tested candidate bytes exactly: SHA-256 prefix
`d42aece2354c8f99`. Layout validation and storage validation for 569 raw JSON
model documents passed. Temporary artwork, candidates, renders, logs and
authoring bytecode were removed.

The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/230.png)
was inspected as a temporary reference and is not shipped. Searches did not
establish an accessible original reusable rig. No external geometry was
imported, and this sculpture is static.

An earlier fresh Rust roster check failed on separate foliage compilation
errors. Those reported source calls subsequently changed, and a new all-251
check including Kingdra passed with zero failures. Its scope is loaded bounds,
grounding and normals. Native Bevy 0.20 captures previously returned black images;
see the [native review](native-pokemon-review.md). Neither stored-mesh checks
nor software previews establish native presentation or TUI/MCP gameplay. The
full 251-species appearance goal and usable rig search remain active.
