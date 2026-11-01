# Slugma and Magcargo sculptures and skins

`tools/lava_sculpt.py` authors the anatomy and `tools/lava_glb.py` exports the
canonical standard skins. Both duplicate static JSON assets are removed. The
battle builder dispatches these rigs before generic Blender batching. Every
original neutral f32 position, normal, color and triangle is retained at the
0.98 authoring height. Raw recipe geometry is cached for normalization and pivots.

Both retain their continuous longitudinal mantle, raised face and tapered tail,
scalloped low edges, flattened underside, rounded eye stalks, yellow eyes,
slim black pupils, fitted smile and molten cheek droplets. Slugma retains its
two fitted sinuous mantle folds. Magcargo retains its broad stone shell,
complete raised spiral on each side, molten fissures, hollow chimney, inner
lining and recessed orange floor.

Five Slugma joints and six Magcargo joints articulate the mantle, raised neck
and independent eye stalks. The low contact layer keeps its original height;
a spatial neck field bends only the raised mass. Each stalk carries its eye
and pupil, and the fitted face follows the raised neck. Slugma's mantle folds
follow the low body. Magcargo's shell, both spirals, fissures and all chimney
parts share one rigid owner, preserving their shapes while the shell rocks
above its seat. Standard rotation-only idle/source-cue timing and literal
neutral endpoints are retained.

| Model | Parts | Vertices | Triangles | Joints | Bytes | Accessors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Slugma | 12 | 2,290 | 4,108 | 5 | 114,632 | 63 |
| Magcargo | 22 | 5,554 | 10,276 | 6 | 247,368 | 103 |

Existing loader budgets remain unchanged. Verified pack heights are 0.7112 m
and 0.7874 m. Neutral f32 digests:

- Slugma: `035d33384e2d2375f1a6ad51fee8bc97efe63d2449179ac3fcc79f83522d8ade`
- Magcargo: `47cde235b8a7adf55ab9c3c7eeaeb86d824ec44959fe3bf46e7764a8b74cbe5b`

Ordered fitted-patch centroids preserve the original Python 3.9 geometry under
Blender's newer Python. The complete GLBs regenerate byte-identically in both
environments. No external mesh, texture, rig or extracted game content is shipped.

```sh
python3 -m unittest discover -s tools -p 'test_lava*.py'
```

Six anatomy checks retain topology, shading, bounds, fitted surfaces, shell
anatomy, chimney depth and body/stalk/shell contact. Eight skin checks decode
the actual keys and weights to verify fixed contact height, coherent owners,
neutral endpoints, visible motion, physical posed joins, fitted details,
winding and strain. Five rays through the posed vent check every model part:
the recessed molten floor remains the highest center hit, more than 0.085
units below the lip. Shiny checks are intentionally skipped.

All 14 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two
wider catalog/timing regressions passed. Blender regeneration is byte-identical
for both canonical GLBs. The production native client built successfully.
Front/rear and fixed-camera neutral, idle and attack views were inspected,
followed by normal paired 3D/2D battles with preserved game state and visibly
rendered terrain. Both bodies meet the floor. Shiny checks were skipped as
requested.

Final normal pairs (`state_preserved=true`):

- Slugma versus Sentret: core 2536/hash 1639352788, LCD 3164,
  virtual time 56.093004792; QA prefix `slugma-normal`.
- Magcargo versus Pidgey: core 1966/hash 2484868359, LCD 2659,
  virtual time 47.260232457; QA prefix `magcargo-normal`.

Evidence remains under ignored `target/pokemon-native-review`.
