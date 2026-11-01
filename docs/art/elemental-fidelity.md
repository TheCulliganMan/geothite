# Elekid, Electabuzz, Magby and Magmar: volume and posture audit

All four now have upright supporting shins in their orthographic profiles.
Forward toe projection belongs to the feet. Elekid has a rounder, deeper egg
body, two complete black waist bands, shorter open plug blades, dark shins,
broader feet and alternating raised/low fists. Electabuzz has more belly and
forearm mass, a flared bell-shaped skull, unequal tufts, broader open jaws and
both bent fists beside its head. Magby has a tapered deep head, broader pupils,
low upturned palms, low pear-shaped hips and three claws on each wider foot.
Magmar has short supports below fuller haunches, down/forward forearms, cuffs
following those curves, a broad golden bill and thick forked flames.

Original recipes remain in `tools/electric_pair_sculpt.py`,
`tools/magby_sculpt.py` and `tools/magmar_sculpt.py`. The four corresponding
`*_glb.py` exporters retain strict canonical readers, source parity, complete
skins, original authoring heights and existing loader limits. No downloaded
mesh, texture, rig or animation is an authoring/export input.

Full eight-angle Battle Revolution reference volumes were inspected for
[Elekid](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339548/),
[Electabuzz](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/303326/),
[Magby](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339549/)
and [Magmar](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339442/).
Captured BW front/back idle samples, Crystal front/back pictures and X/Y front
pictures were inspected through their [Elekid](https://pokemondb.net/sprites/elekid),
[Electabuzz](https://pokemondb.net/sprites/electabuzz),
[Magby](https://pokemondb.net/sprites/magby) and
[Magmar](https://pokemondb.net/sprites/magmar) galleries. Two front frames and the
back picture were also read from the completed local pack for each species.
Extracted neutral arm spreads establish anatomy rather than battle posture;
Crystal poses and captured idles establish the distinct authored arm carriage.
The opaque plane behind the reference Elekid plugs and reference Magby bill
texture artifacts were checked against captured pictures, not copied as anatomy.

Magmar's yellow lower torso wraps around its sides, back and underside. Clipping
the actual torso triangles removed the pinholes caused by an independently
triangulated color loft. Crown and tail flames have rounded profile sections
and forked outlines. Fitted front/rear cores clip to their actual triangles.
Broad round surfaces use focused sampling to retain these features inside the
unchanged 20,000-vertex, 120,000-index, 64-runtime-part, 256-accessor and 1 MiB
limits. All four keep their original production physical scale.

| Species | Logical/runtime parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Elekid | 33 / 33 | 11,517 | 19,024 | 9 | 148 | 452,776 |
| Electabuzz | 70 / 41 | 19,747 | 33,880 | 19 | 243 | 879,676 |
| Magby | 40 / 40 | 12,864 | 22,336 | 11 | 164 | 473,284 |
| Magmar | 55 / 39 | 19,024 | 29,200 | 17 | 219 | 803,244 |

Authoring heights remain 0.91, 1.28, 0.77 and 1.30. The corresponding external
pack heights remain 0.6096 m, 1.0922 m, 0.7112 m and 1.2954 m. Elekid and Magby
retain their chosen local-pack normal palettes. Later red Magby references
establish anatomy without replacing the Crystal purple-pink palette.

Source geometry pins:

- Elekid: `732c1d916e23c4165639128762c00e8ab323ab70903637e07aa455d8beaec635`
- Electabuzz: `6eb9b418dfe166fceee43b1e42c57025ca13801f5a37291b37781381854effcb`
- Magby: `6eaa05168ad1933e63dd038d097128c6d54c1808f892243e83c9c9e5703fedd4`
- Magmar: `6133e7ff64479745f7771a2dbc209d46a01e1c60a5147088889dc47bc63d3d82`

The losslessly packed runtime pins for Electabuzz and Magmar are
`39f624b2a9c8d396b06dad1492a0e2a9496451c6d8990946d379587ab1c538ab` and
`ae1bb9b0aaf6768366ed3276fae1e98f4d6742393f932f2f54e35444e48cc0f7`.
Every named source solid retains its order, geometry, color and weights.

Seventy-three focused Python checks pass across the final affected runs.
They cover closed outward connected solids, actual normals, canonical bytes,
source parity, hierarchy/binds, budgets, fixed battle feet, occupied neutral and
posed roots, fitted surface clearance, unfilled plug holes and mouths, winding,
edge strain and neutral clip endpoints. The new stored-volume guards reject
all six corresponding tests against the immutable prior assets: forward-sloped
shins, thin electric bodies, generic fist carriage, high Magby palms and flat
Magmar flames. Leg guards cut actual mesh sections. Flame guards measure actual
cross-sections, rather than the bounds of a tilted card. Magby's additional
mouth probes follow the moving head, derive the aperture from current lip
vertices and require occupied backing behind empty space against every part.

The exact test modules are `test_electric_pair_sculpt`, `test_elekid_glb`,
`test_electabuzz_glb`, `test_magby_sculpt`, `test_magby_glb`,
`test_magmar_sculpt`, `test_magmar_glb` and `test_elemental_fidelity` under
`tools/`. Run with `PYTHONPATH=tools python3 -m unittest` followed by those names.

All eight neutral authored and reference angles were inspected for every
species against immutable baseline orbits at shared 1.6 studio framing.
Magmar also received a complete underside orbit. Decoded stored skins were
sampled at idle 0.25, attack 0.34 and hit 0.18 at 45 and 90 degrees, retaining
the neutral height, origin and common camera. The retained ignored gallery is
`target/pokemon-fidelity-review-20261020-elemental/index.html`.
The reusable fidelity-audit skill now includes reference material artifacts,
wrapped-tissue triangulation and true cross-section thickness; its validator
passes. Only these four assets changed among 276 canonical variants; the other
272 are byte-identical to this phase's start. Nothing is staged or committed.

Thirteen focused Rust checks pass after verifying exact current compressed GLB
payloads in the test executable. They cover four generic GPU activations at
verified pack heights, four canonical owner/clip tests, three world-walking
checks, the full species schema and hip-centered biped steps. Both infants now
also receive 64 walking phases while every non-leg weight remains unchanged.

The test executable was rebuilt with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run
```

Both production examples were rebuilt with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example immersive_battle_3d --example render_at_location \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

Both executables contain the exact current compressed canonical assets, verified
by finding each payload and requiring decompression equality with its GLB.
Headless image captures use muted audio, no Winit window, the completed ignored
`content-packs/core-modular.browser.crystalpack` and original physical sizes.
No content pack was rebuilt. All twenty-four final native/classic PNGs were
individually inspected; twelve paired-state receipts preserve authoritative
checksum, LCD frame, virtual time and actor facing within each pair.

Battle captures use the real witnessed Route 29 walking encounter, ordinary
`RALA` inputs repeated twenty times, `CRYSTAL_RENDER_TEST_HOLD_BATTLE=0` and
`CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1`. Every admitted scene reports
`accepted=true`, the intended modeled species and no fallback. New Bark walking
starts at `(7,12)`, hour 12, with ordinary `RR` inputs and both hold flags off.
Supported cameras `2 1 35 0` and `5 1 35 0` provide wide and close views. Each
field receipt identifies the requested species as `PartyFollower`.

| Species / view | Frame | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: | ---: |
| Elekid / battle | 1290 | 895406696 | 1913 | 38.195983416 |
| Elekid / world | 441 | 3452823389 | 473 | 8.477885125 |
| Elekid / close-world | 398 | 3805874739 | 417 | 7.458012125 |
| Electabuzz / battle | 533 | 91828259 | 1111 | 19.404713125 |
| Electabuzz / world | 428 | 66068792 | 456 | 8.225569333 |
| Electabuzz / close-world | 393 | 3190541249 | 416 | 7.460660875 |
| Magby / battle | 1001 | 3013654649 | 2030 | 40.02228625 |
| Magby / world | 963 | 945927931 | 1032 | 20.10200825 |
| Magby / close-world | 379 | 4028709411 | 393 | 7.002420083 |
| Magmar / battle | 518 | 1636771565 | 1140 | 19.749778167 |
| Magmar / world | 348 | 3240101090 | 371 | 6.527383041 |
| Magmar / close-world | 332 | 2112043591 | 352 | 6.190641416 |

The close views show complete feet and distinct arm silhouettes. Faces, open
bills/jaws, electric markings, crown shapes and volume remain readable at
production sizes. Magmar's native battle fire presentation overlays the authored
flames; the studio profiles separately prove their thickness. The classic move
menu hides the player back picture, and classic field omits the added follower.
Those images establish state correspondence; separately inspected pack pictures
and full orbits establish the concealed front/back likeness. Different runs can
end at different interpolated positions; every individual pair preserves its
own exact state.

Fifty selected images and twelve receipts remain in the ignored gallery.
Reference-only archives, meshes and textures, draft exports, render scratch and
unused capture intermediates were removed after verification. Shared packs and
Cargo caches remain. The next Hitmon-family audit has only immutable baseline
orbits and a read-only triage; no anatomy was changed without game references.
This phase accepts sampled native normal-color views, not browser, shiny,
every-animation or whole-roster fidelity. No files were staged or committed.
