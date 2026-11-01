# Oddish sculpture and skin

Oddish's missing exact body is an original round blue bulb with two root feet,
five cupped green leaves and a twelve-joint standard glTF skin. Its sole
canonical runtime source is
`crates/crystal-voxel-view/models/battle_species/oddish.glb`.
`tools/oddish_sculpt.py` authors its anatomy; `tools/oddish_glb.py` supplies
weights, inverse binds and source-cue clips. The generic closed botanical leaf
helper is reused for blade and raised midrib construction. The arrangement,
body, face, feet and weights are authored specifically for Oddish.

The [official Oddish artwork](https://sg.portal-pokemon.com/pokedex/0043/)
guides the broad blue bulb, tiny red eyes and ivory glints, open happy mouth
with pink tongue, two blue root feet and five pointed green leaves. The first native review exposed a narrow leaf silhouette from the side:
spreading the blades through depth and lowering the outer leaves corrected
the fan in a side-angle software review. Final native verification is recorded
below. Matte materials preserve the simple silhouette. Battle and explicit world species
identity share one neutral source. Shininess retains the existing flag and
pack palette, using identical geometry and rig.

The GLB has 19 named closed parts, 8,845 vertices, 14,620 triangles and 440,592
bytes, with floor zero and authoring height 1.00. Neutral digest:
`b9c42e2be84290dfd4b02c68370d07d6f235fb3f7ea34826d677a1faaef29384`.
Pack dimensions own physical size. Both root feet stay on the fixed root;
the bulb blends out of its lowest contact area into the body joint. The fitted
face remains rigid on that joint. Each leaf has a base and tip joint; blade
and midrib share identical longitudinal weighting. Joint pivots use the exact
same normalization transform as the authored geometry. Attack braces at 18%
of the cue and leaf tips follow at 28%; idle sways and hit recoils. Source
programs retain movement, audio, projectiles and timing ownership.

```sh
python3 tools/oddish_glb.py
python3 tools/test_oddish_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view oddish --lib
```

Ten independent stored-model checks pass: exact regeneration, bounded
schema, closed outward solids and normals, anatomy and dimensions, hierarchy
and inverse-bind identity, normalized weights, rigid face, fixed feet, floor
clearance and strain throughout every clip, five complete leaf chains,
neutral endpoints and early brace with delayed leaf-tip response, uncrossed
neutral/posed leaf caps, crossing legacy-cap negative controls, unchanged
boundary positions and exact preservation of every other part. Native
integration and visual evidence are recorded in `docs/art/animated-models.md`.
Neutral software review alone does not establish native animation, browser or
world acceptance.

## Initial review history

Seven independent Python model checks pass on the final asset, including
fixed feet and deformation throughout all clips. A depth-span assertion was
added and verified to guard the wide fan from a side view. Focused canonical
routing passes. The final native rebuild passes; its embedded catalog reports
236 exact bodies out of 251 packed species, leaving 15 missing.

The first production review exposed a narrow upright tuft when the mostly
planar leaves were viewed from the battle camera. Spreading the five blades
through depth and lowering the outer leaves corrected that silhouette. The
final held pair `target/pokemon-native-review/oddish-3d.png` / `oddish-2d.png`
was inspected. Receipt: `state_preserved=true`, hash 3978529836, LCD animation
frame 37, virtual time 0.947527042 seconds. The fan is wider from the production
view, with blue bulb, small red eye and rooted feet visible at source size.
Some blades are edge-on or occluded from this camera. The 2D reference is the
back sprite. Native bulb is bluer, and leaves darker, than the source sprite's
gray-green bulb and bright leaves. Native source-cue motion, browser and world
review remain open. The broader appearance and rigging goal remains active.

Both final Oddish Rust integration checks pass: canonical neutral sharing and
fixed root feet across clips, plus twelve-joint GPU skin activation at source
size (0.508 m). The freshly compiled executable passes all 26 active species
rig regressions (one existing audit ignored), all 22 skinning regressions and
the 236-species reusable model check. Targeted whitespace checks pass. No
pack, audio program or unrelated game content was changed. The full model and
rigging goal remains unfinished; most exact bodies still need dedicated rigs
and complete production visual acceptance.

## Concave cap correction

A later shared-helper audit found proper triangle crossings in all five
leaf blades: each legacy vertex-fan cap cut through its concave cup's side
wall. Oddish now explicitly uses the botanical helper's ear-clipped section
caps. All boundary positions, full silhouette, dimensions, triangle count,
materials, weights, pivots and clips retain their contracts. Twenty redundant
vertices disappear because corrected cap normals require fewer splits.
Every non-blade part retains exact f32 geometry, normals, color and indices.
The runtime's strict reviewed-source digest and expected vertex count are
updated to the corrected canonical asset; parsing limits stay intact.

The ten stored-model checks pass in 32.632 seconds. Strengthening preservation
to include every other part's normals/materials, all joint nodes, per-position
weights and every animation time/quaternion also passes in 4.615 seconds.
Six sampled times including neutral endpoints across all three clips have
no proper triangle intersections in any blade or midrib; each regenerated
legacy blade remains a failing negative control. The software six-view sheet
shows the same broad fan and small fitted face before/after the cap correction.
The GLB SHA256 is
`89709c96c91c4b70d7d2a016366b69523e14b68f0a6ae88353c3906123630c89`.

Both focused Rust Oddish checks pass in 0.65 seconds: canonical registry
identity/fixed root feet throughout clips and 12-joint GPU admission at the
original 0.508 m pack size. Both native clients rebuild in 43.74 seconds.
The shared anatomy/order/physical-size schema check passes in 110.93 seconds,
making 13 distinct scoped checks including the ten Python checks. An initial
Rust compile encountered a concurrent furniture-test Source/string mismatch;
the corrected shared tree passes without this fix editing furniture files.

Three final same-state production pairs and a moving world frame were
inspected inline, using only headless image targets and mute. Two movement
traces contain 32 synthetic frames across five Route36 positions. The Route29
battle admits complete Oddish and Sentret models without fallback; Oddish
retains uniform scale, zero support and original dimensions/full bounds.
Both Pokémon names appear in the HUD. No window, app switch or focus change
was used.

| View | State frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| front | 1776 / 879107196 | 1779 | 30.357651042 |
| world | 1810 / 2457394389 | 1819 | 31.141370251 |
| battle | 2847 / 2009742427 | 3584 | 61.9774555 |

Complete ignored receipts:
`target/pokemon-native-review/oddish-caps-capture-receipts.json`. Both binaries
contain the exact compressed current GLB plus headless/mute controls, and
their streamed hash, size and modification time stay unchanged across review:

- `render_at_location`: `77cef99d4af003a580ba02cd710465e501c8b2272016461fdff82cc39f905027` (407,653,344 bytes; mtime_ns 1792331585633849952).
- `immersive_battle_3d`: `4453acd42b9133b28815284ff79f773f925e47774c897c44c8a82a390f1c837c` (407,418,792 bytes; mtime_ns 1792331584503557922).

The round blue bulb, five-leaf fan, small red eye and rooted feet read in
battle. Front views show the tiny fitted happy mouth. World distance and side
foreshortening hide much of the face and some edge-on leaves, as before.
Palette parity, broader camera/browser/material coverage and full-roster
visual acceptance remain open. This fixes the specific cap audit item; it
does not claim the full model/rig goal complete. No shiny verification, grass,
AA/filtering, audio, pack publication or gameplay changes belong to this fix.
