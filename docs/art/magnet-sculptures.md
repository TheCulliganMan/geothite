# Magnemite and Magneton sculptures

Original static models replace the thin rod magnets and oversized pupils with
smooth blue-silver bodies, broad white eyes, tiny pupils, cross-slotted face
bolts and threaded crown screws. Each magnet is a closed beveled horseshoe
with an outward opening and solid red and blue terminal blocks. Magneton uses
three physically intersecting bodies, with the lower units turned outward.

`tools/magnet_sculpt.py` authors both models without Blender or an external
asset. Magnemite has 18 parts and 19,316 triangles; Magneton has 54 parts and
57,948 triangles. Both retain their existing 0.96 height and ground at zero.
The main battle builder uses this recipe and bypasses its obsolete callback.

`tools/test_magnet_sculpt.py` independently checks closed, connected, outward
parts; finite unit normals and face winding; exact recipe parity, height and
triangle budgets; complete mechanical details; actual solid intersections
between screws, bodies, magnets and pole blocks; all three Magneton body
joins; surface-fitted eye and screw markings; and empty outward notches
between solid magnet arms. The first preview exposed incorrectly oriented
bevel walls; the topology test rejected them, and the wall winding was fixed.

All nine stored-mesh tests passed in 71.316 seconds. Final front, oblique and
rear previews were inspected. The installed canonical files match the tested
candidate bytes exactly. Layout validation passed, and storage validation
passed for 569 raw JSON model documents. Temporary candidates, reference
images, previews and logs were removed.

Fresh native presentation remains incomplete: the rebuilt reviewer returned
blank battle captures, as documented in [native review notes](native-pokemon-review.md).
The separate Rust all-251 loaded-mesh check passed after these replacements.
Software previews do not establish native integration or gameplay. The full visual
goal and usable rig search remain active. Temporary species candidates,
reference artwork, software renders and test logs were removed.

[EmblemSauce's rigged Magnemite](https://emblemsauce.itch.io/3d-rigged-pokemon-model)
advertises a freely downloadable rig but permits educational and portfolio
use and prohibits commercial use. Those terms do not establish permission to
redistribute its raw game asset in this repository. No source asset was
imported; the production sculptures remain original and static.
