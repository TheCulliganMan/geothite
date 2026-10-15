# Poliwag sculpture and skin

Poliwag's missing exact body is an original round blue tadpole with a fitted
white spiral belly, large eyes, forward pink lips, two paddle feet and a long
thin oval tail. Its sole canonical runtime source is
`crates/crystal-voxel-view/models/battle_species/poliwag.glb`.
`tools/poliwag_sculpt.py` authors its geometry; `tools/poliwag_glb.py` supplies
a five-joint standard glTF skin and idle/attack/hit rotations. Generic closed
surface helpers are reused; no evolved body or menu icon is relabeled.
Battle and explicit world species identity share the same neutral source.

The [official Poliwag artwork](https://sg.portal-pokemon.com/pokedex/0060/)
guides the blue body, ivory eyes with dark pupils and highlights, pink lips,
white belly with a continuous dark coil, long blue feet and pale oval tail
with blue centerlines. Initial neutral inspection caught lips hidden behind
the belly and stubby feet; a forward rounded lip volume and longer feet fixed
those defects. The first export exceeded runtime budgets; reducing small eye
patch subdivisions preserved their outlines and retained denser belly fitting.
The tail membrane uses the existing opaque matte material contract. Shininess
retains the existing flag and pack palette on the same geometry and rig.

The final GLB has 19 named closed parts, 15,265 vertices and 25,532 triangles,
681,504 bytes, with authoring height 0.90 and floor zero. Neutral digest:
`2914f4cf3bdb2b4ea6f4795e1586e3d93f4dee15f5b0050ca2981fdcc37c2f2d`.
Pack dimensions own physical size. Feet stay on the fixed root. Body, belly,
spiral and face form one rigid body assembly so the coil stays attached.
Tail stem, membrane and both centerlines share a three-joint longitudinal
weighting function. Joint pivots use the same normalization as the geometry.
Attack braces the body at 18% of the cue, the middle tail follows at 28%, and
the tip follows at 35%. Idle surveys and sways; hit recoils. Source programs
retain movement, audio, projectiles and timing ownership.

```sh
python3 tools/poliwag_glb.py
python3 tools/test_poliwag_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view poliwag --lib
```

Seven independent stored-model checks pass: exact regeneration, bounded
schema, closed outward topology and normals, anatomy and dimensions,
hierarchy/bind identity, normalized weights, rigid face/belly/spiral ownership,
fixed feet, floor clearance and strain throughout every clip, complete tail
chain, neutral endpoints and early brace with delayed tail response. The
coherent-belly check was extended and rerun to prove the pink lips project
forward of the actual stored belly surface. Spiral attachment also checks the
stored coil against the stored belly triangles. Native and runtime evidence
are recorded in `docs/art/animated-models.md`. Neutral software review alone
does not establish production animation, browser or world acceptance.
Seven independent Python model checks pass. The coherent-belly check was
extended and rerun to verify that the forward pink lips remain visible ahead
of the actual stored belly surface. Focused canonical routing passes. The
native rebuild passes; its embedded catalog reports 237 exact bodies out of
251 packed species, leaving 14 missing.

Initial neutral inspection found hidden lips and stubby feet; a rounded lip
volume projecting forward of the belly and longer paddle feet fixed those
features. The initial export exceeded budgets; fewer subdivisions in small
eye patches preserved their outlines while retaining denser belly fitting.
The final front and side neutral reviews were inspected.

The held native pair `target/pokemon-native-review/poliwag-3d.png` /
`poliwag-2d.png` was inspected. Receipt: `state_preserved=true`, hash
2589769990, LCD animation frame 42, virtual time 1.028859708 seconds. The belly
spiral, forward pink lips, ivory eyes, long planted blue feet and pale thin
oval tail with blue centerlines read at source size. The source view is the
back sprite. Native belly edges still show some faceting, and the blue is
lighter than the source back sprite's gray-blue shading. The tail remains
opaque under the existing material contract. Native source-cue motion,
browser and world review remain open; the full appearance/rigging goal is
unfinished.

The first native/test builds encountered a concurrent Ecruteak gate edit:
lintel bounds still referenced a removed `z`. The gate now uses the actual
`entry`/`front` doorway limits; the native retry passed. No pack or audio
content was changed by this model work.

Both Poliwag Rust integration checks pass: canonical neutral sharing and
fixed paddle feet throughout clips, plus five-joint GPU skin activation at
0.6096 m. The fresh test executable passes all 27 active species rig
regressions (one existing audit ignored), all 23 skinning regressions and the
237-species reusable model check. The existing shrine/ruin source-bounds and
outward-winding regression also passes after the gate compile repair.
Targeted whitespace checks pass. The full model/rigging goal remains active;
most exact bodies still need dedicated rigs and full production visual review.
