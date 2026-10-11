# Horsea and Seadra sculptures

`tools/seahorse_sculpt.py` replaces the generic ball-head, straight-neck,
angular-loop and slab-fin construction with original curved anatomical forms.
The continuous tapered chest and neck joins a rounded head and a swept inward
tail spiral. A fitted cream breast follows the actual body triangles, with
five shallow supported transverse folds. Both retain the 1.07 authoring height.

Horsea has six rounded tapering head rays, coral irises, narrow pupils and tiny
eye lights. Its two rounded ivory paddles have curved supported fin folds.
Seadra has a narrowed angular gaze, smaller serrated cheek webs, six small
rooted flank thorns and two broad cupped blue sails. Each sail's three cream
points fits both the actual front and rear surfaces and extends across the
fin thickness rather than disappearing in rear views.

The muzzles are real annular walls with dark bore linings and a chamber floor
well behind their openings. No opaque disk covers the mouth rim. The eyes fit
to the actual head, with separate dark sockets and inset ivory whites.
An intermediate preview showed faceted cream breast shading; the surface
normals were smoothed and reviewed again. The first Seadra sails were too flat
and their cream tips too small; their curvature, proportions and pigmentation
were revised before acceptance.

The main builder dispatches Horsea and Seadra through the pure recipe before
its Blender branches. Kingdra now uses its dedicated
[sea-dragon recipe](kingdra-sculpture.md).
These original sculptures are static; no external geometry was imported.

Horsea has 36 parts and 57,516 triangles; Seadra has 34 parts and 29,044
triangles. All nine tests in `tools/test_seahorse_sculpt.py` passed in 66.274
seconds against the final stored candidates. They check closed connected
outward solids, actual triangle shading, recipe parity, unique anatomy names,
species identity counts, grounding, height, budgets, real solid roots at the
body/head/tail/fins/armor/muzzle/bore/chamber, fitted details against stored
parent triangles, front clearance, rear clearance on the two-sided cream
tips, open axial snout rays with recessed floors, and the inward spiral.

Final front, oblique and rear software previews were inspected. Canonical
files match the tested candidate bytes exactly: SHA-256 prefixes
`4ab7be9411474868` for Horsea and `9aa388c8c44bdbb0` for Seadra. Layout validation
and storage validation for 569 raw JSON model documents passed. Temporary
artwork, candidates, renders, logs and authoring bytecode were removed. The
previous Rust all-251 loaded-model check passed before these two replacements;
a fresh check including Horsea and Seadra failed before running tests because
the separate foliage work does not compile. Kingdra was also replaced after
the last passing Rust check. The reported foliage calls subsequently changed,
and a fresh all-251 check including all three passed with zero failures. Its
scope is loaded bounds, grounding and finite unit normals, not appearance.

The [Horsea artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/116.png)
and [Seadra artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/117.png)
were inspected as temporary references and are not shipped.
[Tlal's Horsea](https://sketchfab.com/3d-models/pokemon-horsea-6c69c3ed63df446197ac125bded7bfab)
advertises a simple rig and idle animation. Its official API reports one
animation, an Editorial license and no available download. It is not an
accessible reusable open rig, and was not imported.

Native Bevy 0.20 appearance verification remains incomplete; see the
[native capture review](native-pokemon-review.md). Stored-mesh tests and software
previews do not establish native presentation or TUI/MCP gameplay. The full
251-species appearance goal and usable rig search remain active.
