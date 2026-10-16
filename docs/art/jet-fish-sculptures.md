# Remoraid and Octillery sculptures and skins

`tools/jet_fish_sculpt.py` authors the anatomy and `tools/jet_fish_glb.py`
exports canonical standard glTF skins. The battle builder dispatches both
before generic Blender batching. The duplicate static JSON assets are removed.
Every original neutral f32 position, normal, color and triangle is retained at
the 1.00 authoring height. Raw recipe geometry is cached for normalization and
skin pivots.

Remoraid retains its continuous tapered mint body, swept upper and scalloped
lower tail lobes, forward dorsal crest, lower pelvic fin, pale pectoral paddles,
fitted lateral bars and large ivory eyes. Seven joints give it a floor-height
preserving body yaw, a caudal body bend, coherent rigid tail lobes and independent
fin strokes. Its fitted bars, eyes and mouth follow the body. Rigid tail lobes
avoid folding the thin scalloped fan during a stroke.

Octillery retains its continuous head, neck and arm mantle, eight curled arms,
sixteen open annular yellow sucker rims and recessed gold wells, three fitted
crown spots, heavy eyelids and hollow upward siphon. Eleven joints articulate
the upper mantle, aiming siphon and eight individually phased tentacle tips.
The lower supports remain on the identity floor root. Arms and their suckers
share a smooth spatial weight field; the upper curls bend while their lower
contacts stay fixed. The broad transition and bounded curl angles avoid local
stretching. The siphon and inner lining share a longitudinal deformation field,
keeping the mouth open while the base remains connected. Facial details and
crown spots follow the upper mantle.

Both skins use the existing idle loop and source-relative attack/hit clips,
rotation-only animation and literal neutral endpoints. The verified local pack
heights are 0.6096 m for Remoraid and 0.889 m for Octillery.

| Model | Parts | Vertices | Triangles | Joints | Bytes | Accessors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Remoraid | 18 | 6,822 | 7,724 | 7 | 263,080 | 89 |
| Octillery | 52 | 19,798 | 21,740 | 11 | 741,696 | 229 |

The existing 1 MiB, 20,000-vertex and 256-accessor limits remain unchanged.
Neutral f32 digests:

- Remoraid: `02f2a0f1061633dd1b6772fc886c1ccce95ddc5237d63a8e3ca29a280a6d0514`
- Octillery: `11c7a88eb486c7a59ae19980866dcfc9a9d0f25510be9472abfaa419a8a2c36f`

Official [Remoraid artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/223.png)
and [Octillery artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/224.png)
were used for the original sculpture pass. No external mesh, texture, extracted
game model or reference image is shipped.

```sh
python3 -m unittest discover -s tools -p 'test_jet_fish*.py'
```

Eight anatomy tests retain the closed topology, shading, height, proportions,
fitted surfaces, arm contacts, sucker support and hollow siphon checks. Eight
skin tests independently decode the actual keys and weights to check coherent
ownership, floor contacts, physical posed joins, open posed suckers, an open
aiming siphon, winding, strain, neutral endpoints and visible movement of every
arm and fin. Shiny checks are intentionally skipped.

All 16 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration produced byte-identical GLBs. The production native client built successfully; final paired 3D/2D battles were visually inspected with preserved game state, accepted terrain and visible ground. Shiny checks were skipped as requested.

Remoraid against Sentret: core 2552/hash 4071485440, LCD 3222, virtual time 58.125843916; `state_preserved=true`.

Octillery against Pidgey: core 2956/hash 819259712, LCD 3720, virtual time 68.988857374; `state_preserved=true`.

Final ignored QA prefixes are `target/pokemon-native-review/remoraid-normal` and
`target/pokemon-native-review/octillery-normal`. Temporary helpers, builder
parity outputs and software previews were removed after inspection.
