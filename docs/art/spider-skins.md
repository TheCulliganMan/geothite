# Spinarak and Ariados six-limb skins

`tools/spider_sculpt.py` retains each species' original anatomy and improves its
surface construction. Spinarak has six walking legs, black eye lenses, coral
mandibles, an upright ivory horn, and fitted false eyes and a frown on its
abdomen. Ariados has four walking legs and two raised dorsal legs, purple eye
borders, white hooked mandibles, a forward horn and fitted dark chevrons.
The supplied pack's front and back sprites and official
[Spinarak](https://assets.pokemon.com/assets/cms2/img/pokedex/full/167.png) and
[Ariados](https://assets.pokemon.com/assets/cms2/img/pokedex/full/168.png)
illustrations informed the anatomy review.

Eye lenses and highlights now follow their actual parent surfaces. Walking
legs have rounded centerlines and contiguous colored bands; exterior normals
come from the complete limb before material splitting. Hidden cut planes retain
separate normals. Shared equal corners are indexed after assigning shading, so
this improvement stays below the existing 8,500-triangle and authoring-size
budgets. Mandibles use curved sweeps.

The supplied pack identifies heights of 1 foot 8 inches (0.5080 metres) for
Spinarak and 3 feet 7 inches (1.0922 metres) for Ariados. Normal body anchors
are RGB555 `(9,20,5)` / RGB8 `(74,165,41)` and `(27,10,8)` /
`(222,82,66)`. Ariados's purple is `(14,6,20)` / `(115,49,165)`.
Both authored full silhouettes are normalized to 0.90; production scaling
uses the pack height.

`tools/spider_glb.py` exports one canonical GLB per species, replacing each old
static JSON. Both rigs have eighteen joints: root, abdomen, head, horn, paired
mandibles and six two-joint limb chains. Ariados uses two of those chains for
its raised dorsal legs. Walking influence fields taper to the fixed floor
root at the tips. Band names select the limb, while spatial position selects
weights, so duplicated shared ring vertices deform identically. Fitted head
features follow the head and dorsal markings follow the abdomen.

Idle, attack and hit clips contain rotation keys, return to neutral at both
endpoints, and preserve the identity root. Attack braces the head and mandibles
early; raised legs lag behind the body cue. The existing production battle
cue owns playback timing. These are idle and battle clips; walking gaits still
need separate work.

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Spinarak | 43 | 7,932 | 7,684 | 18 | 222 | 379,116 |
| Ariados | 41 | 6,498 | 6,768 | 18 | 222 | 326,824 |

Pinned neutral geometry digests are
`b88a15aff01d0ca5d5362a8bb3bcfc8d86c389a1bc7c5d15c6d722bb194ee1d6`
and `b680cd4729af2101b86594f951817cb56f56a2c4e03b80209c2863345caa57f4`.
The strict readers reject changes to geometry, weights, hierarchy, clips or
metadata by reproducing and comparing canonical bytes.

Eleven stored-geometry and GLB checks, plus the canonical route check, passed after publication. They
verify topology, shading, anatomy, fitted dorsal marks, shared limb rings,
hierarchy, inverse binds, normalized weights, material owners, palette anchors,
floor support, posed seam closure, triangle winding, strain and neutral
endpoints. The real focused Blender build reproduced both canonical GLBs
byte-for-byte:

```sh
python3 -m unittest discover -s tools -p 'test_spider*.py' -v
blender -b -t 2 --python tools/build-battle-species.py -- \
  target/spider-focused --only spinarak,ariados
```

Two production Rust tests also passed: registry playback keeps walking tips
fixed while the face, horn, mandibles and six limbs move, and generic GPU
skinning uses both verified pack heights. Canonical battle mesh aliases use
the same neutral vertex order as each skin.

The full 73-variant schema check and all 47 production GPU skinning tests
passed. The complete art checker validated 60 editable Blender sources, 609
runtime models in 609 files and 737 embedded references. The canonical resolver
continues to expose all 251 species.

References, review projections and temporary geometry remain under ignored
`target/pokemon-native-review`. Task-created reference helpers, extracted sprite
images, draft geometry and focused-build output were removed after verification.

The freshly built native client produced four normal/shiny capture pairs using
`content-packs/core-modular.browser.crystalpack`. Both views of every pair were
inspected. Spinarak retains its small grounded silhouette, green body, false
dorsal face and gold/dark bands. Ariados reads as the larger red spider with
four walking legs, two raised purple/gold legs, a forward ivory horn, fitted eye
borders and hooked white mandibles. Both use matte materials, readable eye
highlights and continuous band seams. The shiny views retain the same geometry
and rig while the pack palette changes Spinarak to purple/pink and Ariados to
purple/blue. Secondary normal band colors follow the official artwork; the
classic Ariados sprite represents its light bands with the limited palette's
white shade.

All paired receipts report `state_preserved=true` at core frame zero:

| Capture | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Spinarak normal | 1892400081 | 53 | 1.419960584 |
| Spinarak shiny | 788614488 | 54 | 1.367141375 |
| Ariados normal | 180502018 | 61 | 1.587418667 |
| Ariados shiny | 95783334 | 63 | 1.593003584 |

Normal and shiny previews are separate source fixtures with different DVs and
stats; preservation is verified within each 3D/2D pair, not across fixtures.
Camera, facing, floor placement and gameplay state were not changed to improve
the screenshots. Walking gaits, source-specific motion for every Pokémon and
browser acceptance remain part of the wider unfinished model goal.
