# Exeggutor sculpture

`tools/exeggutor_sculpt.py` authors an original coconut palm with a rounded
continuous trunk, supported front and rear segment creases, two stout ankles,
wide foot stumps, pale fitted pads and six rooted ivory claws. Three golden
coconuts have a tired slack mouth, an open fanged mouth and a pink-lined smirk.
Pale eyes, fine pupils, lids, tongues and fangs fit actual stored surfaces.
Eleven curved closed fronds have a folded central keel and supported narrow
midribs. Authoring height remains 1.22 with normal runtime species scale.

The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/103.png)
was inspected for canopy, trunk, feet and the three expressions and is not
shipped. Searches did not establish an accessible usable original Kanto
Exeggutor rig. No external geometry was imported. This authored mesh is static.

Eight stored-mesh checks cover closed outward parts, unit normals and actual
winding, recipe parity, heights and grounding, budgets, anatomy counts,
actual solid intersections at heads, ankles, feet, claws and fronds, parent
fitting and front marking clearance, plus supported trunk bands and midribs.
Initial tongue and smirk-lining shapes extended outside their mouths; fitting
uses the actual curved parent outlines rather than their rectangular bounds.
All eight stored-mesh regressions passed. The model has 70 parts and 56,984
triangles. Already dense crease and midrib paths use their actual samples
without another fourfold subdivision; this reduced the initial 87,404
triangles while retaining the blade curves. The initially long trunk was
compressed with the inverse scale applied to its normals, and the coconut
heads and canopy were lowered without distorting their shapes. Final front,
rear and oblique views were inspected.

The canonical copy matches the tested candidate bytes exactly. The main
builder dispatches both Exeggcute and Exeggutor through their respective pure
recipes. Layout and storage validation passed for 569 raw JSON model documents.
Temporary candidates, reference artwork, software renders and logs were
removed. Fresh native presentation remains incomplete: the rebuilt reviewer returned
blank battle captures, as documented in [native review notes](native-pokemon-review.md).
The separate Rust all-251 loaded-mesh check passed after these replacements.
Software previews do not establish native integration or gameplay. The full visual
goal and usable rig search remain active. Temporary species candidates,
reference artwork, software renders and test logs were removed.
