# Pokémon authoring portability and roster checks

A complete read/export survey covered 276 canonical variants in 175 authoring
families: all 251 species and the additional Unown letters. It found five
variants whose source path failed on Python 3.12 despite valid reviewed GLBs.
Three local numerical fixes restore the existing bytes without regenerating
any runtime model:

- Gyarados belly coordinates retain the reviewed positive zero after rounding.
  Python's changed summation could otherwise produce six negative-zero entries
  and change the source digest while leaving the numerical mesh unchanged.
- Onix's centerline length uses explicit left-to-right accumulation. This keeps
  the reviewed metadata value `2.8677406192405615`, rather than changing its last
  bit through compensated summation. Physical size and geometry are unchanged.
- Closed Unown strokes use an explicit scaled tangent length. This preserves
  the reviewed frames when `hypot` rounding changes between Python versions;
  B, O and V no longer fail their geometry pins. All 26 forms retain their
  canonical buffers, glyph shapes, skin weights and animation keys.

The corrected Gyarados, Onix and all 26 Unown readers/exports were checked on
both installed Python 3.9.6 and 3.12.14 and reproduce the reviewed GLBs exactly.
Thirty-nine existing anatomy and decoded-skin regressions pass on Python 3.12,
covering topology, normals, actual contacts, supported poses, coherent eyes,
distinct glyphs, animation endpoints, winding, strain and source-cue metadata.

```sh
python3 -m unittest discover -s tools -p 'test_gyarados*.py'
python3 -m unittest discover -s tools -p 'test_onix*.py'
python3 -m unittest discover -s tools -p 'test_unown_glb.py'
```

After targeted follow-up, all 276 source reads pass. Verification modes are
explicit: 271 default authoring exports match canonical bytes; Gengar,
Raticate, Pidgeotto and Totodile reproduce their canonical bytes using their
stored geometry as the authoring source. Meganium retains its pre-existing
reviewed-container/expanded-geometry equivalence path. Its expanded positions,
joint IDs, weights, hierarchy and decoded keys are identical; the host export's
largest normal-component difference is about `7.11e-19`. Its reviewed GLB and
strict SHA guard are preserved. This does not claim every sculpture can be
resynthesized without its canonical GLB.

The current runtime test binary was checked for all 276 exact compressed GLB
payloads before the full-roster motion test. That test passed at sixteen phases
per entry, including the base Unown alias, checking hierarchy order, support
chains, floor penetration, excessive floor correction and exact split seams.
The same current payloads and unchanged binary were verified again after the
authoring corrections. These are numerical consistency gates, not a replacement
for species-specific attachment tests or matched-state production visual review.

Ignored local evidence uses `target/pokemon-native-review/roster-authoring-`,
`authoring-platform-` and `roster-motion-runtime-` prefixes. The reviewed assets,
palettes, rigs, sizes and rendered appearances were preserved. No windows were
opened, content packs modified or shiny variants tested. Broader model-fidelity
acceptance remains separate from these source and motion checks.

The full geometry validator now reads the canonical GLB/actor source routing,
accepts the explicit base Unown-to-I mapping, and checks all 26 Unown files once.
It reads storage limits from the Rust loader rather than retaining an obsolete
22,000-triangle cap. Packed vertex/index counts enforce the actual loader
budgets; logical anatomy still receives finite-attribute, unit-normal,
outward-winding, positive-volume, floor-root and unique-geometry checks. No
runtime budget was raised. Ten focused tests cover valid routing and negative
controls for identity, geometry and storage limits.

```sh
python3 -m unittest discover -s tools -p 'test_check_battle_species.py'
python3 tools/check-battle-species.py --progress
```

Both commands passed. The full run checked 251 species and 276 variants,
including 5,244,489 triangles and 3,436,678 logical source vertices. Source
vertex totals include logical anatomy expansion, rather than packed GPU size.

A fresh windowless GPU review also captured all 276 current normal-color
variants. All 46 labeled contact pages were visually inspected, with no
blocking defect observed in the sampled rest, travel (20% and 80%) and attack
(40%) poses. The exact embedded model payloads and test executable stayed
unchanged during capture. The test build excludes Winit and audio, uses an
image render target, and opens no application windows. This studio review uses
normalized scale and one oblique angle; it supplements the existing matched
2D/3D production reviews without claiming every angle, move, map or browser
presentation. Shinies were not tested.

Local pose evidence uses the ignored `roster-pose-gpu-current` and
`roster-pose-gpu-pages` directories under `target/pokemon-native-review`, with
capture and visual-review receipts alongside them.
