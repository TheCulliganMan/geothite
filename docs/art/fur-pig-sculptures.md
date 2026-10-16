# Swinub and Piloswine sculptures and skins

`tools/fur_pig_sculpt.py` authors the anatomy; `tools/fur_pig_glb.py` exports the
canonical standard glTF skins. Both duplicate static JSON assets are removed.
The battle builder dispatches the rigs before generic Blender batching.
Every original neutral f32 position, normal, color and triangle is retained at
the 0.98 authoring height. Raw recipe geometry is cached for normalization and
skin pivots.

Swinub retains its low elongated coat, four fitted dorsal wedges, sleepy curved
eyes, projecting pink snout and rear wisps. Piloswine retains its tall humped
mantle, scalloped hem, long drooping ears, hidden eyes, fitted front fur creases
and ivory tusks curving down and forward. Both retain fitted tilted nostrils,
four supporting shanks and grounded dark oval hooves.

Eight Swinub joints and nine Piloswine joints provide a gentle body sway,
articulated sniffing head and supported leg middles. Hooves stay on the identity
floor root. Snouts, nostrils, eyes, front fur creases and tusks follow the head;
the front coat reaches the same owner before those fitted details. Swinub's
stripes remain on the body. Piloswine's ears and Swinub's rear wisps have their
own coherent owners. A broad vertical shank blend follows each leg's centerline,
avoiding weight steps around the tiny polar rings and keeping the hooves fixed.
Idle, attack and hit retain the shared rotation-only source-cue timing and
literal neutral endpoints.

| Model | Parts | Vertices | Triangles | Joints | Bytes | Accessors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Swinub | 22 | 7,044 | 8,496 | 8 | 278,932 | 97 |
| Piloswine | 22 | 5,062 | 9,432 | 9 | 211,732 | 90 |

The existing loader budgets are unchanged. Verified pack heights are 0.4064 m
and 1.0922 m. Neutral f32 digests:

- Swinub: `99b43c8174c27d71d246f211d8ca6a0ce2764cb9d6e60cc2153b1fe25ffc0591`
- Piloswine: `b6b3ef74640643f89b461d55f276157aad0cfb2f7a39d17214b9a9eb8d3ea0b9`

Fitted patches use the existing ordered-centroid helper to preserve the original
Python 3.9 geometry under newer Blender Python. The dorsal-patch helper accepts
that same patch function for Swinub's stripes; its default remains unchanged.
Blender regeneration is byte-identical to both canonical GLBs.

The original anatomy was checked against official
[Swinub artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/220.png)
and [Piloswine artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/221.png).
No reference image, external mesh, texture, rig or game content is shipped.

```sh
python3 -m unittest discover -s tools -p 'test_fur_pig*.py'
```

Six anatomy checks cover exact f32 regeneration, dimensions and budgets, closed
connected surfaces, shading, coat proportions, tusk direction, fitted details,
grounded hooves and actual body/leg/hoof overlap. Seven skin checks decode the
actual keys and weights to verify fixed feet, coherent features, posed physical
joins, fitted surfaces, winding, strain, neutral endpoints and visible motion.
Shiny checks are intentionally skipped.

All 13 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration is byte-identical to both canonical GLBs. The production native client built successfully. Final paired 3D/2D battles were visually inspected with preserved game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Swinub against Sentret: core 1762/hash 3963713493, LCD 2338, virtual time 41.149872167; `state_preserved=true`.

Piloswine against Rattata: core 1919/hash 3115544402, LCD 2543, virtual time 44.664665584; `state_preserved=true`.

Final ignored QA prefixes are `target/pokemon-native-review/swinub-normal` and
`target/pokemon-native-review/piloswine-normal`. Fixed-camera neutral, idle and
attack views and rear art views were also inspected. Temporary helpers, software
previews, movement frames and builder parity outputs were removed afterward.
