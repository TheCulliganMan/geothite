# Natu and Xatu sculptures

`tools/psychic_bird_sculpt.py` authors original canonical Natu and Xatu meshes.
The battle species builder dispatches both before Blender batching. Each keeps
its existing 1.04 presentation height and floor origin.

Natu has one round green body, calm ivory almond eyes with broad dark pupils and
soft silver reflections, a broad rounded golden bill and one swept red lance
crest. Two rounded dark wing shields have curved golden leading feathers and
three red trailing sections separated by dark bars on both faces. Two short
swept red tail feathers join the body. The asset has 46 closed parts and 44,692
triangles.

Xatu has a tall tapered green torso and broad green head, watchful almond eyes,
a long hooked golden bill and a fitted dark jagged neck ruff. Two red watchful
chest symbols have narrow dark pupil slits. Long folded ivory sleeves meet
below the chest and carry red-and-gold diagonal badges and red/dark hem bands
on both faces. Two long swept red head ribbons descend behind the torso, which
carries three fitted golden rear bars. The asset has 49 closed parts and
24,060 triangles.

Both birds have short supporting red ankles, three long splayed red toes and
ivory talons on each foot, plus curved rear toes. All facial and plumage marks
fit the actual stored surfaces; no separate planar symbols float in front of
the body or wings.

Silhouette and palette were checked against the
[official Natu artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/177.png)
and [official Xatu artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/178.png).
External research found CC Attribution listings for
[Pokemon Natu](https://sketchfab.com/3d-models/pokemon-natu-0926265c0b7f4b66a3cde29f6df9ab34),
described as made in Feather, and
[Xatu](https://sketchfab.com/3d-models/xatu-3bb721abe28e4b63937209fa9ef721e2).
Their metadata did not establish usable original rigs. No external meshes or
textures were imported. These are original static sculptures using native base
colors, with no rigging or animation claimed.

`python3 -m unittest discover -s tools -p test_psychic_bird_sculpt.py` checks
closed connected topology, outward winding, finite unit normals, deterministic
recipe parity, height, budget, anatomy counts, all fitted face/symbol/plumage
vertices and front triangle clearance, both wing faces, actual body/head/bill,
crest/streamer/tail/wing roots, and every supporting ankle, toe and talon join.
Root checks use actual occupied ray intervals rather than bounds alone.
Front/rear software views and native battle captures assess model presentation
rather than gameplay behavior.

Final verification passed all seven psychic-bird geometry regressions. Final
front/rear software views and both native battle captures were visually
inspected. The Rust catalog regression passed for the 220-species battle catalog; it verifies
grounding and volume rather than visual quality. Layout and storage validation
passed for all 571 JSON assets. Temporary reference artwork, review renders and
logs were removed after inspection.
