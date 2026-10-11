# Doduo and Dodrio sculptures

`tools/dodo_sculpt.py` authors original static runner birds with gently
scalloped feather bodies and heads, long individually curved necks, separate
tapered upper and lower bills, long shanks, three forward toes and a rear toe
on each foot, and rooted ivory claws. Doduo has charcoal necks and legs, black
oval eyes with pale glints, and two differently raised heads. Dodrio has ivory
necks and legs, angular watchful eyes, open bills with small pink tongues, a
charcoal lower feather apron, six dark crown quills and three broad rounded
pink tail plumes. The lower body and tail are actual closed geometry.

The [official Doduo artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/084.png)
and [Dodrio artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/085.png)
were inspected and are not shipped. The original coarse feather displacement
was softened after visual review. Tail feathers were turned to reveal their
broad faces. A separate pale oval breast marking was removed: it resembled an
owl marking rather than Doduo's evenly colored body with shading. All heads
share one authored local shape per species, then receive individual proper
rotations and names; no duplicate catalog or external asset is introduced.

Doduo has 37 parts and 42,084 triangles; Dodrio has 61 parts and 46,896
triangles. Both retain their existing 1.22 authoring height and ground at zero.
The main builder dispatches these pure recipes and bypasses its old callback.

`tools/test_dodo_sculpt.py` checks closed connected outward parts, finite unit
normals and actual face winding, exact recipe parity and grounding, height
and triangle budgets, distinct anatomy counts, actual solid intersections at
necks, heads, beaks, crests, tail, apron, legs, feet, toes and claws, plus eye
marking fit and front clearance against actual stored triangles. Eye geometry
is checked in the corresponding head's local frame, including its yaw and
pitch, rather than confusing turned faces with a shared global front plane.
The front-face check uses relative projected triangle area to distinguish
actual front caps from nearly vertical sidewalls after a quantized head turn;
the same strict clearance threshold applies to all actual front faces.

All eight stored-mesh tests passed in 41.035 seconds. Final software previews
cover front, rear and both oblique directions and were inspected. The
canonical files match the tested candidate bytes exactly. Layout validation
passed, and storage validation passed for 569 raw JSON model documents.
Temporary candidates, artwork, renders, logs and authoring bytecode were
removed.

Fresh native presentation remains incomplete: the rebuilt reviewer returned
blank battle captures, as documented in [native review notes](native-pokemon-review.md).
The separate Rust all-251 loaded-mesh check passed after these replacements.
Software previews do not establish native integration or gameplay. The full visual
goal and usable rig search remain active. Temporary species candidates,
reference artwork, software renders and test logs were removed.

Searches did not establish an accessible original reusable rig for
either species. No external geometry was imported. The full 251-species visual
review and usable open rig search remain active.
