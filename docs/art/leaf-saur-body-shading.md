# Rounded bodies for the leaf starter family

This records the earlier shading-only pass. The
[October 21 family fidelity revision](grass-starter-fidelity.md) intentionally
changes anatomy, fitted surfaces, palettes and rig pivots. Its metrics and
reviewed shading contract supersede the preservation snapshots below.

A CPU silhouette survey of all 251 canonical species, paired with normal front
sprites from the local pack, exposed pronounced lighting panels on Chikorita,
Bayleef and Meganium. Their rounded necks, cheeks and haunches now use the
existing 35-degree crease-normal authoring helper. Leaves, collar buds, petals,
antennae, claws and fitted faces retain their authored edges and materials.

| Species | Previous / current vertices | Triangles | Joints | Original pack metres |
| --- | ---: | ---: | ---: | ---: |
| Chikorita | 7,463 / 4,989 | 3,220 | 11 | 0.8890 |
| Bayleef | 9,626 / 5,695 | 3,836 | 16 | 1.1938 |
| Meganium | 11,353 / 6,817 | 4,520 | 23 | 1.8034 |

This removes duplicate vertices at smooth body corners. It preserves every
expanded triangle position and its order, all six neutral bounds, complete
supports, material colors, fitted artwork, joint hierarchy, inverse binds,
quantized weights and every animation key. Full silhouette heights remain
1.00, 1.15 and 1.48 authoring units. Floor caps retain flat downward normals;
the shape, support areas and physical scaling are unchanged.

`test_leaf_saur_shading.py` pins the pre-refinement expanded geometry and
animation contract. It also checks every body corner normal against its actual
triangle within 35 degrees, unit normals, all 720 floor-cap corners per body,
and the reduced vertex budget. The existing closure, facial-fit, winding,
deformation, strain, fixed-contact and cue tests remain active. Their body
normal assertion now permits smooth normals instead of requiring each body
corner to equal one flat facet; non-body parts retain the original assertion.
Nine pre-existing Meganium collar corner normals differ only by signed
sub-1e-14 platform residues, normalized to geometric zero for the contract.

All 38 focused Python checks passed against the candidate GLBs, and the
installed assets passed the preservation checks. The labeled neutral source
comparison and decoded neutral/attack/hit poses were inspected offscreen.
All six focused Rust registry/contact and production GPU binding checks pass,
and the full 276-variant schema check passes: 45 distinct scoped checks.

Three close native 2D/3D pairs and three actual moving field frames were
inspected at Ecruteak City (18, 21), noon, with the same authoritative state.
The body panels are visibly smoother while eyes, smiles, collar folds, leaves,
flower petals and antennae remain readable. Feet and claws meet the ground;
the walking frames retain their supported steps. Each capture records 32
synthetic movement frames through five grid positions. An independent audit
matches the current GLBs, preserved geometry/rig contracts, selected images,
movement receipts and stable capture binaries. Close captures completed in
39.014, 54.568 and 78.643 seconds.

Review used headless image targets with audio muted. This refinement preserves
the previously reviewed battle geometry and stored animation keys; its new
native evidence covers close field views and moving field frames. The broader
roster goal remains active. Dedicated shiny checks remain skipped. Packs,
grass, global shaders and gameplay are unchanged by this refinement.

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest \
  test_leaf_saur_shading test_chikorita_glb test_bayleef_glb test_meganium_glb -v
```
