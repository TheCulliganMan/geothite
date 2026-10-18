# Dragon family sculptures

`tools/dragon_family_sculpt.py` supplies original, separate Dratini, Dragonair
and Dragonite geometry through the main battle-species builder. The obsolete
shared dragon routes reject these species. These are static sculptures; the
existing Pidgeotto rig and unrelated species are preserved.

Dratini has a continuous broad curved blue body, a white underside following
the actual tube rings, a round white forehead dot, white nose, violet fitted
eyes and folded ivory ear fins. Dragonair has a slender descending neck and
continuous coil, a forward forehead lance, feathered ear fans, and three
rooted blue pearls: one at the neck and two at the tail. Its white underside
also follows the actual curved tube geometry.

Dragonite has a continuous orange pear torso and neck, rounded head and broad
snout, fitted friendly eyes, a short crown point and two long curved cream
feelers. Eight seams cross a supported cream belly. Rooted arms and hands carry
six ivory digits; grounded feet carry six more. Teal membranes are visible on
both sides of the orange wing borders, with seated leading arms and ribs. A
continuous swept tail carries its own cream underside.

Final counts are 17 parts / 24,092 triangles for Dratini, 21 / 30,492 for
Dragonair, and 60 / 61,996 for Dragonite. Presentation heights remain 1.18,
1.18 and 1.40; all models ground at zero. SHA-256 prefixes are respectively
`47cf66ac5f266ba4`, `40a4f64cccb735f6` and `c15150578d070ec5`.

The candidate passed all twelve stored-mesh tests in 44.393 seconds. Checks
cover closed connected outward parts, finite unit normals and actual face
winding, recipe parity, grounding, height and triangle budgets, species-specific
identity, independent nonadjacent face crossings, actual anatomical roots,
fitted face and seam support, and triangle-center and edge-midpoint clearance.
Ventral ribbon support uses independent distance to actual stored triangles;
a detached ribbon is rejected by a negative control. Wing, limb, digit and
continuous-tail joins are tested against actual geometry.

All final front, oblique and rear software renders were inspected. Inspection
caught a tight Dratini bend and Dragonite belly shading artifacts. Dratini now
uses a broader arc. The belly now uses regular triangulation with sufficient
clearance, producing a smooth cream surface; the earlier uneven adaptive
triangulation was discarded. A grazing axis-ray support test was replaced by
actual point-to-triangle distance after a point 2.1 mm from its stored parent
incorrectly appeared 21 mm away along an axis.

Official [Dratini](https://sg.portal-pokemon.com/pokedex/0147/),
[Dragonair](https://sg.portal-pokemon.com/pokedex/0148/) and
[Dragonite](https://sg.portal-pokemon.com/pokedex/0149/) artwork was inspected
in temporary browser tabs. No artwork is shipped.

The [Eleanie rigged Dragonite](https://sketchfab.com/3d-models/dragonite-966370d8b1174c9ca20f2e34e7cdaeb7)
reports downloadable CC Attribution 4.0 geometry and 101,356 faces through
its [public metadata](https://api.sketchfab.com/v3/models/966370d8b1174c9ca20f2e34e7cdaeb7).
Its description identifies Auto Rig Pro and an EEVEE comic shader. The download
endpoint returned HTTP 401, so its geometry and skeleton could not be inspected.
Zero reported animation clips does not establish absence of a rig.
[KM Dragonite](https://blendrdragon.itch.io/km-dragonite) describes a Rigify rig
and a free name-your-price archive, but its page does not establish explicit
asset redistribution terms. Neither external model was downloaded or imported.
The broader 251-species appearance and usable rig search remain active.

The installed canonical meshes passed all twelve tests again in 42.422 seconds.
Layout validation and storage validation passed; the current shared model tree
contains 570 raw JSON documents. 

The fresh native battle example built successfully in 9m 40s, including shared
cache waiting. The fresh Rust loaded-model regression passed for exactly 251
species: one actual test passed, zero failures, in 2.21 seconds. That regression
covers loaded bounds, grounding, geometry and normals, not visual quality.

Three explicit GPU image captures used the ignored realtime-clock browser pack
and the freshly built production battle example. The same executable inode,
size and modification time were checked before and after each capture. All
three processes exited zero. Each actual 2000×1400 PNG was inspected at full
frame, showing the requested authored model, opposing Pidgeotto, arena scenery
and battle HUD. Dratini's broad continuous curve and white nose, Dragonair's
coil and all three pearls, and Dragonite's smooth segmented cream belly,
antennae, wing membranes and grounded claws are visible. The captures contain
37,592, 52,272 and 47,961 colors respectively.

This verifies production battle-preview rendering. It does not establish full
gameplay, animation completeness or ordinary native-window capture. Live window
inspection remains limited by the locked Mac reported during the previous
inspection attempt; no new unlock response has arrived. The successful explicit
GPU images do not resolve the separate ordinary-window capture issue.

All own verification jobs completed before temporary candidates, previews,
native images, logs and authoring/test bytecode were removed. Temporary reference
browser tabs were closed. The shared Rust build cache remains in place.

The later [canonical skin publication](dragon-family-skins.md) replaces these
static exports with bounded, independently controlled GLBs. Current captures
use the headless image path and never open a window; the historical window
limitations above do not describe that path.
