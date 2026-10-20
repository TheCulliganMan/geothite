# Farfetch'd sculpture baseline

`tools/leek_duck_sculpt.py` authors an original gray-brown duck with a rounded
cream underside, upright hood, tall ivory eyes and fine dark pupils, fitted
eye lights, a dark forehead chevron, a broad flattened golden bill with a
curved seam, and three crown prongs. Short golden shins support rounded closed
three-point webbed feet with surface-fitted toe folds.

The raised left wing has three broad fan feathers. The other wing crosses the
body, with three rounded solid feather fingers and two curls that physically
intersect the curved leek stalk. Its pale stem, green sheath and three broad
closed leaf blades connect through actual solid roots. Three short tail
feathers attach to the underside. The cream belly is closed volume around the
lower body rather than an oval front marking. The first thin folded-feather
overlays showed visual artifacts despite passing the sampled fitting checks;
they were replaced with smooth rooted solid forms and reviewed again.

The original static model has 45 parts and 43,988 triangles, retains its existing 1.10 authoring
height and grounds at zero. The main builder dispatches the pure recipe and
bypasses its obsolete generic duck/leek branch.

`tools/test_leek_duck_sculpt.py` checks closed connected outward parts, unit
normals and actual face winding, recipe parity, grounding, height, budgets,
identity counts, actual solid roots at the body, head, bill, crests, wings,
feather fingers, tail, legs and feet, physical gripping contact with the leek,
stalk/sheath/leaf joins, fitted face markings and front clearance, plus foot
fold support against actual upward-facing web surfaces.

All nine stored-mesh tests passed in 42.802 seconds. Final front, oblique and
rear previews were inspected. The canonical file matches the tested candidate
bytes exactly. Layout validation and storage validation for 569 raw JSON model
documents passed. Temporary candidates, reference artwork, renders, logs and
authoring bytecode were removed. A fresh Rust all-251 loaded-model check
including this replacement passed on the shared Bevy verification cache.
Its scope is loading, bounds, grounding and normals.

The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/083.png)
was inspected and is not shipped. Searches did not establish an accessible
original reusable rig. No external geometry was imported. The original static
sculpture has since migrated to the [standard production skin](leek-duck-skin.md),
which preserves all anatomy names and original neutral bounds. Its native
headless front, world, moving and battle reviews are complete; the earlier
black-capture limitation is historical. The full 251-species model and
rig improvement goal remains active.
