# Ledyba and Ledian six-limb skins

The existing shared builder gives both species the same round shell spots and
six white hand-like ends. The new recipe in `tools/ladybird_sculpt.py` separates
their proportions and anatomy using the verified pack sprites and official
[Ledyba](https://assets.pokemon.com/assets/cms2/img/pokedex/full/165.png) and
[Ledian](https://assets.pokemon.com/assets/cms2/img/pokedex/full/166.png) artwork.
The back sprites show five round spots for Ledyba and four for Ledian.

Ledyba has a broad gold abdomen, red head, fitted eye lenses and lower facial
mask, curved antennae, four white fists and two grounded white glove-like feet.
Ledian has a slimmer abdomen, a small mouth, four white fists, two red boots,
a fitted rear head spot, and pale flight membranes behind its red wing cases. Both use six insect limbs
and a fitted central breast seam. Glove finger lines follow the actual fists.

The verified source heights are 3 feet 3 inches (0.9906 metres) and 4 feet
7 inches (1.3970 metres). Their normal palettes contain white, black and
RGB555 red `(27,6,5)`, expanded to RGB8 `(222,49,41)`. Ledyba's gold anchor
is `(26,19,8)` / `(214,156,66)`; Ledian's is `(31,22,4)` / `(255,181,33)`.
The normalized draft heights are 0.95 and 1.05.

The first Ledyba draft exceeded the 20,000-vertex limit. Reducing excess spot
subdivisions preserved fitted-surface clearance and brought it within budget.
Rear inspection then exposed an inflated double-lobed shell. Each species now
uses two closed halves of one continuous dome, with identical seam positions
and exterior normals. The hidden center cut has separate plane normals.
Ledyba's fifth spot is split between the independently riggable wing cases.

The current Ledyba draft has 36 parts, 18,127 vertices and 28,940 triangles.
Ledian has 38 parts, 19,044 vertices and 30,356 triangles. All four independent
geometry checks pass for both models:

```sh
PYTHONPATH=tools python3 -m unittest test_ladybird_sculpt -v
```

They verify closed outward geometry, shading, shared dome seams, actual parent
surface clearance for faces, gloves and rear spots, source palettes, grounded
feet, budgets, six-limb anatomy and the distinct spot counts. Front and rear
authoring renders were inspected. References and temporary geometry stay in
ignored `target/pokemon-native-review`.

`tools/ladybird_glb.py` exports fourteen-joint Ledyba and sixteen-joint Ledian
skins. Both have abdomen/thorax/head chains, paired antennae, upper and lower
arm pairs, supporting legs and two wing-case joints. Ledian adds two flight-wing
joints. Fitted face and head-spot layers follow the head; glove seams follow
their fists; shell spots follow their exact cases. Both supporting feet remain
on the identity floor root. Idle, attack and hit use rotation-only keys and
return to neutral at their endpoints.

The first playback check exposed imperceptible idle case movement and inward
case rotation. The revised keys open the rear seam outward, then return to the
closed dome without pushing the halves inward during hit recovery. An
independent posed-seam regression verifies the opening direction.

Ledyba has 171 accessors in 723,344 bytes, with neutral digest
`38b13162e17a6677a6d3c6cc6561f870210a342cbb00e226a8688ee5d5811e2a`.
Ledian has 189 accessors in 779,796 bytes, with neutral digest
`d8b776f761bf9f6287d975cc158c3ebf2b9f06b2b67479a98ef19e881f202031`.
All nine stored-GLB checks passed for both species, followed by ten combined
canonical and route checks after publication. The real focused Blender builder
produced both canonical GLBs byte-for-byte. Two production Rust tests passed:
registry playback keeps feet fixed while the face, fists, feelers, cases and
Ledian's wings move, and GPU skinning uses the verified pack heights.

The canonical GLBs replace both static battle JSON bodies and are registered
in the standard renderer. The schema check passed for all 71 standard variants,
and all 46 GPU skinning checks passed. The full authoring check passed with
60 editable Blender sources, 609 runtime models in 609 files and 733 embedded
references. Catalogue coverage remains 251 Pokémon species.

The preserved-state native baselines were inspected: Ledyba checksum
1684412370, LCD frame 61, time 1.524055624 seconds; Ledian checksum 4290352828,
LCD frame 55, time 1.392580083 seconds. Both have core frame 0. The old models
show protruding eye/spot geometry, a missing facial mask, identical limb ends,
and insufficient distinction between species.

A fresh native build produced four paired 3D/2D captures from the production
battle renderer. All were visually inspected and preserve their own state:

| Species | Appearance | Checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- | --- |
| Ledyba | Normal | 501422619 | 72 | 2.379653875 |
| Ledyba | Shiny | 4167295922 | 69 | 2.428784917 |
| Ledian | Normal | 1895862126 | 95 | 2.697896792 |
| Ledian | Shiny | 658199492 | 73 | 1.816596208 |

Every receipt has core frame 0 and `state_preserved=true`. Each pair is an
independent fixture with its own DVs and stats. The full silhouettes fit,
feet remain on the arena floor, eye lenses and facial masks stay fitted, and
Ledian's red boots and pale membranes distinguish it from Ledyba's white feet
and broader abdomen. Normal red/gold and shiny orange/yellow palette anchors
match the paired classic views. The native profile naturally hides most rear
markings; inspected rear authoring views show the five/four shell spots and
Ledian's head spot. No camera-facing marker override was added.

This accepts the current Ledyba/Ledian battle-model milestone. The standard
registry now covers 46 species through 71 variants, with legacy Pidgeotto
separate. Most species still need dedicated rigs, and broad world motion and
browser fidelity remain open. The full all-Pokémon goal is not complete.
Task scratch geometry, source sprite references and helper executables were
removed after acceptance. Final native and authoring review captures stay ignored.
