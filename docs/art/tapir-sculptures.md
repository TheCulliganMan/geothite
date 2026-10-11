# Drowzee and Hypno sculptures

`tools/tapir_sculpt.py` now authors smooth curved body profiles, curved muzzles,
rounded palms, fingers and rooted thumbs, rounded shoulder/elbow/knee joints,
and raised heels with rounded yellow toes. Joint volumes cover the tube rims;
an intermediate preview exposed coincident rim shading despite passing the
contact tests, so the overlaps were adjusted and reviewed again.

Drowzee retains a continuous anatomical body with yellow and brown material
regions welded at the wavy waist. The two regions share exactly 64 boundary
positions and normals. Its rounded ears have surface-fitted brown linings.
Hypno retains pointed ears with fitted brown interiors, a closed ivory shoulder
ruff with a fine scalloped edge, raised hands and a thin hanging cord. The
pendulum has a genuinely open metal ring and a dark inner lip that physically
overlaps the metal rather than floating inside its hole.

Both species have dark eye sockets fitted to the actual body surface, inset
ivory eye surfaces, fine pupils fitted to those surfaces and fitted cheek
creases. Drowzee has 43 parts and 44,440 triangles; Hypno has 48 parts and
38,432 triangles. Both retain the existing 1.12 authoring height and ground at
zero. The main builder already routes these species through the pure recipe.

All nine tests in `tools/test_tapir_sculpt.py` passed in 19.695 seconds against
the final stored candidates. They check welded closed body regions, outward
closed anatomy, actual triangle shading, recipe parity, grounding, height,
identity counts, geometry budgets, actual solid contact at the ears, muzzle,
limbs, rounded joints, fingers, thumbs, feet and ruff, hand/cord/ring/lip
connections, the open ring and collar holes, surface fitting against stored
parent triangles and front-face clearance.

Final front, oblique and rear software previews were inspected. Canonical
files match the tested candidate bytes exactly: SHA-256 prefixes
`0a61c2bd2583a513` for Drowzee and `a79c2065302e6cbd` for Hypno. Layout validation
and storage validation for all 569 raw JSON model documents passed. Temporary
reference artwork, candidates, renders, logs and authoring bytecode were removed.

The [Drowzee artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/096.png)
and [Hypno artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/097.png)
were inspected and are not shipped. An advertised animated CC Attribution
Hypno candidate remains inaccessible through its authenticated download; see
the [rig research](open-rig-candidates.md). No external geometry was imported.
These original sculptures are static.

The fresh Rust all-251 loaded-model check passed after these replacements.
Its scope is loading, bounds, grounding and normals. Native presentation
remains incomplete because fresh
Bevy 0.20 captures were black, including classic and live controls; see the
[native review](native-pokemon-review.md). Software previews and mesh tests
establish neither native presentation nor TUI/MCP gameplay. The full 251-species
appearance goal and usable rig search remain active.
