# Aipom prehensile tail-hand skin

The canonical runtime model is `models/battle_species/aipom.glb` under
`crates/crystal-voxel-view`, authored by `tools/aipom_glb.py` over
`tools/aipom_sculpt.py`. The duplicate static JSON was retired after an audit
confirmed that smoothing preserves every indexed triangle position, triangle
order and material across all fifty parts. Connected patch normals now smooth
across the face, eyes, belly and palms. This reduces 35,580 vertices to 17,804
without reducing the 30,016 triangles or changing fitted surface positions.
The subsequent palette correction is an intentional material change.

The supplied pack's front/back/normal palettes agree on RGB555 purple
`(12,4,14)` and tan `(29,23,12)`, expanded to RGB8 `(99,33,115)` and
`(239,189,99)`. The model uses those primary colors. Height digits `207`
establish 2′7″, or 0.7874 m, for production scale. The authored neutral span
remains 1.02 units, including the raised tail hand.

Twenty joints articulate torso, neck, head, both ears, arms and hands, four
stages of the prehensile tail through its palm, three radial fingers and three
crown tufts. Feet and toe folds remain root-owned. Upper supports and the belly
share the torso blend; fitted face parts share the head owner. Arm and tail
weights follow projected distance along their authored centerlines. Finger
roots blend from the palm into their individual joint, while each fingertip
stays rigid with that finger. Rotation-only Idle, Attack and Hit clips have
neutral endpoints and no root motion. Tail-hand and finger response follows
the body brace. These are authored anatomical responses; detailed source-frame
motion and walking gaits remain open.

The export has fifty anatomy groups, 17,804 vertices, 30,016 triangles,
twenty joints, 248 accessors and 760,964 bytes, within the existing loader
limits. The neutral digest is
`630e5b04d9e707d595af256a2d8f5fec236362a767aa7aae0541b262aacb8c83`.
Five stored-model checks and seven anatomy checks pass on the final canonical
bytes. Both fresh focused Rust registry/GPU checks pass, including every vertex at
dense off-grid clip samples against support height and culling bounds. All
78 registered variant schemas pass. A cross-runtime audit identified
only signed-zero patch-center differences. Normalizing exact zeros in the
belly and tooth band makes the actual production Blender builder output
byte-identical to the canonical GLB. Temporary audit source, audit GLB and
focused builder output were removed; only ignored QA logs remain.

Full art validation passes for 61 editable Blender sources, 609 runtime models
in 609 files and 754 embedded references. The final production native build
passes. Its first attempt exposed a shared-worktree toolbar compile error:
Bevy 0.20's legacy `Button` is a type alias, not a unit constructor. Explicit
`Interaction::None` and `FocusPolicy::Block` preserve that toolbar's existing
interaction components and allow the native build to finish.

Normal and shiny native pairs were inspected against 2D with the same state
and clock within each pair. Both receipts preserve state:

| Pair | Checksum (core frame 0) | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Normal | 3232759518 | 51 | 1.413380542 |
| Shiny | 172433161 | 54 | 1.211226667 |

The normal purple/tan and shiny magenta/pink surfaces follow the pack colors;
the tail hand and fingers remain attached, with grounded feet and no visible
skinning distortions. This verifies rig integration and palette correction.
Full visual acceptance remains open: the tail palm projects narrowly at the
oblique battle angle, and the face needs a clearer comparison. Tail-palm scale
and orientation, source-frame motion, walking gaits, browser review and the
broader all-species objective remain incomplete.

## Tail-hand rest orientation (2026-10-14)

The complete tail palm and all three fingers turn −35° horizontally about the
unchanged tail endpoint. Finger bind pivots use the same transform; their
weight fields use its inverse. Palm and finger animation quaternions are
conjugated by the rest turn, preserving their authored local curl axes.
Y coordinates, neutral height, palm/finger dimensions and the supporting tail
remain unchanged. An independent before/after audit verifies exact geometry,
normals, indices and weights for all 43 unaffected groups; the seven hand
groups retain triangle order and edge lengths within 0.000000213 units.

The current export is 761,020 bytes with neutral digest
`5dfead35f664f46a4a4ae8d8b59a8f22345dae52cf785d5d9a200fed882a0140`.
Six stored checks and seven anatomy checks pass, including decoded finger
pivot orientation and curl-axis alignment. Both fresh focused Rust registry/GPU
checks pass. Blender's production builder reproduces the GLB byte-for-byte.
The production build passes. Normal and shiny native pairs were personally
inspected on retained Route 29 terrain after ordinary grass encounters. Both
receipts preserve state:

| Turned-hand pair | Core frame | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: | ---: |
| Normal, versus Pidgey | 3059 | 1597936004 | 3765 | 76.585598167 |
| Shiny, versus Sentret | 2599 | 630686738 | 3348 | 71.004649628 |

The broad palm and all three fingertips read clearly in these retained-map
views; both feet remain on the ground and the hand remains attached. The new
camera also exposes the fitted face. These views accept the hand rest pose and
rig alignment under the retained-map camera. Earlier direct-arena screenshots
use a different camera and remain historical integration evidence. Detailed
face fidelity, source-frame motion, gaits and browser review remain open.

The old direct battle fixture timed out because it had no retained encounter
map. The Route 36 scripted fixture reached a real battle but its camera was
rejected as unreadable on checked ground; that QA process was stopped. Neither
failed attempt counts as visual evidence. The walking fixture obtains real
battle provenance and an accepted shot without weakening renderer guards.

## Retained-map model review

The disposable Route 36 and walking fixtures accept
`CRYSTAL_RENDER_TEST_PARTY_SPECIES` (default `CYNDAQUIL`) and
`CRYSTAL_RENDER_TEST_PARTY_SHINY` (`0` or `1`, default `0`). The chosen species
must exist in both the supplied pack and authored model catalog. These options
select the initial party lead only. Movement, encounter rolls, scripts,
battle introduction and the production controller remain authoritative.

The following normal command was executed successfully; use shiny `1` for the
also-verified shiny run. The route stops at the first visible battle. Opponent
species and encounter timing follow the ordinary game rolls.

```sh
CRYSTAL_PREVIEW_MUTE=1 \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=AIPOM \
CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic \
CRYSTAL_RENDER_TEST_HOLD_BATTLE=1 \
CRYSTAL_ENCOUNTER_TRACE=1 \
./target/pokemon-models/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack \
  --walking-encounter --walk RALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALA \
  --size 1000x700 \
  --screenshot target/pokemon-native-review/aipom-palm-grass-3d.png \
  --second target/pokemon-native-review/aipom-palm-grass-2d.png
```

Temporary before-turn GLB, profiler output and focused Blender output were
removed. Ignored QA captures, movement receipts and logs remain.
