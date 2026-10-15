# Quagsire padded-foot amphibian skin

Quagsire retains the existing continuous body, broad head, tiny eyes, curved
smile, hanging arms and flattened rising paddle from `tools/axolotl_sculpt.py`.
Its pads now have more volume and two fitted toe folds each. Palm folds follow
the actual arm surface instead of a fixed front depth. Wooper's geometry,
materials and canonical GLB bytes are unchanged.

The supplied pack's front/back PNGs contain the normal body color RGB8
`(115,173,231)` / RGB555 `(14,21,28)` and dorsal purple `(115,24,165)` /
`(14,3,20)`, plus white and black. The recipe now uses these Quagsire anchors.
The verified Pokédex height is 4 feet 7 inches, or 1.3970 metres. Its authored
silhouette remains normalized to 1.13. The pack sprites and official
[Quagsire illustration](https://assets.pokemon.com/assets/cms2/img/pokedex/full/195.png)
were inspected for proportions, digit folds and dorsal anatomy.

`tools/quagsire_glb.py` exports a thirteen-joint standard skin: root, pelvis,
torso, head, paired arms, palms and supporting legs, plus a three-joint paddle
chain. Foot pads and toe folds remain on the identity floor root. The fitted
face follows the head. Palm folds use the same arm/hand field as the underlying
flesh. Body, paddle and dorsal ridge share a spatial influence field across
their join; the fin's rear section follows the paddle chain.

Idle, attack and hit are rotation-only clips that return to neutral at both
endpoints. Attack braces the head and arms early, with later palm and paddle
settling. The existing source battle cue controls playback timing. Walking
gaits and full source-specific facial motion remain separate work.

The first native inspection exposed hard silhouette facets on the pads. A
second cage subdivision rounds their edges. Lower-belly sampling was reduced
while the upper head retains its denser contour, keeping the same 7,000-triangle
and authoring-size budgets. The refined geometry and skin checks passed again.

The canonical GLB replaces `quagsire.mesh.json`. It contains 22 parts, 4,380
vertices, 6,836 triangles, thirteen joints and 127 accessors in 236,652 bytes.
Its pinned neutral geometry digest is
`d97c02d58f80145da2df67f1902083c13ccc862253c75e9332e5d8b04e5e36ba`.
Strict decoding reproduces the canonical bytes, including skin and clips.

Twelve stored anatomy, skin and route checks passed after publication. They
verify closed geometry, shading, adult pad volume, real parent-surface contact
for facial and digit marks, limb overlap, flattened paddle anatomy, budgets,
palette anchors, hierarchy, inverse binds, normalized weights, feature owners,
dorsal influence coverage, fixed floor support, winding, strain, neutral
endpoints and early head/late paddle timing. The canonical resolver still
contains 251 species.

The first focused build exposed Wooper's fallthrough to the static recipe.
The builder now dispatches both canonical exporters explicitly. The actual
focused Blender run reproduced both GLBs byte-for-byte: unchanged Wooper is
218,156 bytes and Quagsire is 236,652 bytes.

```sh
PYTHONPATH=tools python3 -m unittest test_axolotl_sculpt test_quagsire_glb \
  test_battle_model_assets.CanonicalSources.test_quagsire_resolves_one_canonical_skin -v
blender -b -t 2 --python tools/build-battle-species.py -- \
  target/quagsire-focused --only wooper,quagsire
```

Two production Rust tests passed for fixed pads with articulated face, palms
and dorsal paddle, and GPU skinning at the verified pack height. They passed
again after the foot refinement. The final 74-variant schema check, all 48 GPU
skinning tests, and the shared animation-bounds preservation regression passed.
The complete art check validated 60 editable Blender sources, 609 runtime
models in 609 files and 747 embedded references.

The fresh native normal and shiny 3D/2D pairs were inspected using
`content-packs/core-modular.browser.crystalpack`. The refined pads have rounded
edges and readable toe folds, with real floor contact. The body remains a
continuous broad-headed amphibian with tiny eyes, a fitted smile, hanging
paddle-like palms and a flattened tail. The purple ridge stays attached along
the back and tail. Matte shading keeps the simple anatomy readable. Normal
blue/purple and shiny pale-purple/pink appearances use the same geometry and
rig. The native profile naturally exposes one eye; camera and facing were not
changed to force a frontal view.

Both final receipts report `state_preserved=true` at core frame zero:

| Capture | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Quagsire normal | 2950442125 | 96 | 2.570708375 |
| Quagsire shiny | 958744436 | 69 | 2.548014541 |

The normal and shiny scenes are separate source fixtures with different DVs
and stats. Preservation is verified within each 3D/2D pair. Extracted sprites,
reference helpers, draft geometry and focused build products were removed;
review images and verification logs remain ignored under
`target/pokemon-native-review`. Walking gaits, remaining species rigs,
source-specific facial motion and browser acceptance keep the wider goal open.
