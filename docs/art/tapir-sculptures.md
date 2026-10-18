# Drowzee and Hypno sculptures

## Initial static anatomy review

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
This initial version used static original sculptures; the canonical skins below replace it.

The fresh Rust all-251 loaded-model check passed after these replacements.
Its scope is loading, bounds, grounding and normals. Native presentation
remains incomplete because fresh
Bevy 0.20 captures were black, including classic and live controls; see the
[native review](native-pokemon-review.md). Software previews and mesh tests
establish neither native presentation nor TUI/MCP gameplay. The full 251-species
appearance goal and usable rig search remain active.


## Canonical skins, October 2026

`tools/tapir_glb.py` replaces both static JSON files with canonical GLBs.
All 43 Drowzee and 48 Hypno anatomical parts remain separate and named. Local
surface sampling reductions retain the silhouettes, fitted facial features,
wavy welded waist, pointed/rounded ears, scalloped collar, fingers and open
pendulum ring. Drowzee now has 19,170 vertices and 32,056 triangles; Hypno has
19,105 vertices and 32,320 triangles. Both fit the unchanged 20,000-vertex and
40,000-triangle limits at the original 1.12 authoring height.

Drowzee has 13 joints; Hypno has 14. Both have a floor root, blended torso and
neck, coherent head and fitted features, two three-joint arm chains, two leg
carriers and independent ears. Heels and toes stay fixed in battle. Hypno's
cord and both metal ring surfaces share a pendulum carrier parented to the
right wrist, pivoting at the held cord root. World walking uses two complete
leg/foot chains and preserves the authored pendulum weights.

Idle uses 81 keys over 3.2 seconds. Attack and hit use 65 keys over one second,
with source battle cues owning timing and neutral endpoints. The canonical
Drowzee file uses 188 accessors and 711,100 bytes; Hypno uses 204 accessors and
695,476 bytes. Geometry SHA-256 pins are
`1789c22bcf0b67308e90a00a00965c00c6e42b8f5bde57c782bd0d930c4e0859`
and `7e70103e4d8495bf4b78adeb41eca93860c7bde563ec30811f43098380997915`.

All 16 combined Python anatomy/skin tests passed in 17.631 seconds. They cover
closed anatomy and shading, fitted surfaces, real supports, welded body seam
under motion, fixed feet, winding and strain across all clips, independent
carriers, occupied hand/cord/ring/lip connections, and strict canonical readers.
Blender builds both canonical files byte for byte. Front, side and rear software
pose sheets for idle, attack and hit were inspected; these are art previews.
Shiny checks were skipped as requested. Native presentation evidence follows;
browser presentation remains open.


The focused Rust rig/GPU/world checks passed (3 tests, 1.41 seconds), as did
the shared all-species geometry and exact source-cue pose timing checks
(2 tests, 86.53 seconds). The freshly built native Drowzee/Hoppip Route 29 pair
was inspected at preserved state: frame 2928, checksum 2531053417, LCD frame
3543 and virtual time 64.29578896. The complete rounded body, fitted sleepy
face, wavy waist and fingers remain readable, with both feet on terrain and
the correct DROWZEE HUD. Normal admission passed at original pack size and
the existing -110-degree review angle with support, omission, whole-body and
frustum checks active. No shader errors were reported. Ignored evidence:
`target/pokemon-native-review/drowzee-normal`.


The native Hypno/Sentret Route 29 pair was also inspected at preserved state:
frame 1600, checksum 3201743356, LCD frame 2207 and virtual time 38.088585666.
The pointed ears, fitted gaze, ivory ruff, raised hands and connected open
pendulum remain readable; both feet meet terrain and the HUD says HYPNO.
Normal admission passed at original pack size and existing -110-degree review
yaw with all support/omission/whole-body/frustum constraints active, no fallback
and no shader errors. Ignored evidence: `target/pokemon-native-review/hypno-normal`.
Browser visual acceptance remains open. No shiny checks or grass-height edits
were made.
