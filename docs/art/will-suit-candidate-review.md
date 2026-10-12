# Will suit and mask candidate

## Current crown review

The lower side-part crown is retained locally after inspecting twelve actual
views: Idle, Run .125 and Run .625, each from front, three-quarter, side and
back. Volume .12 and sweep -.035 replace .38 and -.14 for Will alone. The
previous crown rose into a pointed hat silhouette; the lower crown follows
the head more closely. This is a crown improvement, not complete character
acceptance. The jaw, nape, hands, shoes and garment geometry remain unfinished.

The matching GPU comparison is saved externally as
`will-low-crown-diagnostic/protection-check.json`. Indices, UVs, joints and
weights are exact. All 32,318 changed positions are head-weighted; non-head
positions and normals are exact. No painting changed in this comparison.
The supplied-crown preservation regression ran one actual test and passed.
The native build and all three capture jobs passed. This change is not in
the served browser build yet.

The reviewed four-angle `torso-gutter-idle/will.png` still shows a plain red
shirt and bare hands. The repository's Will appearance specifies a suit,
mask and white gloves. A built-in imagegen edit adds those details to the
existing atlas layout, retaining the narrow drawn eyes and purple hair.

This is an unaccepted candidate. The canonical atlas is unchanged. It must
be inspected on the actual model in Idle and two Run phases, including the
rear garment and hand islands. A standalone atlas is insufficient evidence
of correct mapping. All complete models remain under review.

The generated candidate is saved outside the repository at
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/will-suit-candidate-root/people/npc-textures/will.png`.
The candidate review root links the other canonical paintings and supplied
rigs, so comparisons alter only Will's painting.

## Exact prompt (built-in imagegen)

Use case: precise-object-edit. Asset type: existing square UV albedo atlas for a stylized anime game character, not a character portrait or scene. Edit the supplied atlas for Will. Preserve every UV island position, boundary, scale and overall layout exactly. Keep the purple swept hair, the drawn narrow purple eyes, eye positions, ears, face proportions and flat anime skin shading. Add a clean dark purple-black domino eye mask around the EXISTING eyes, with angular elegant outlines, the eye apertures preserving the existing eye artwork and a narrow bridge across the nose; do not move or enlarge the eyes. Replace ONLY the plain red garment rectangle in the lower left with a tailored red suit-jacket albedo: restrained curved fabric folds, small dark red lapels framing a narrow ivory inner-shirt V at the top center, a clean center fastening seam and subtle waist shaping. Keep all garment details inside that existing red rectangle; no whole body illustration. Keep the dark burgundy trouser rectangle in the bottom right unchanged. Change ONLY the hand UV island rectangle in the right middle, approximately x=62.5%..100%, y=51%..74.7%, into ivory white gloves: subtle cool gray seam/knuckle shading, following the existing two hand crease forms. Preserve the skin-colored arm strip on the upper right, the narrow skin strip at the very top right, and both shoes in their existing upper-right shoe strip unchanged. Preserve gutters. Crisp restrained cel-shaded anime albedo, no realistic pores, no glossy plastic, no photographic skin, no new islands, no text, no labels, no grid, no backdrop, no geometry rendered into the atlas. Square opaque PNG.


Protected-region analysis confirms the candidate remains 1254 square. It does
not preserve protected pixels exactly: mean RGBA differences are 1.24 in upper
hair, 1.37 in lower face, 1.38 in shoes, 1.70 in arms and 1.80 in trousers;
maximum differences reach 64 at shoe detail edges. Small mean differences
do not establish unchanged mapped appearance. The actual render review must
check these regions as well as the new mask, lapels and gloves.

Muted serial candidate capture batch 64517 waits for the player batch 13176
to finish before using the GPU. Existing canonical Will painting is unchanged.

## Actual mapped rejection and revised candidate

Existing-reviewer capture 56785 completed successfully; its four-angle Idle
image was individually inspected. The ivory opening appeared on Will's back
and the palms stayed bare. Single-view capture 94500 also completed and was
inspected, confirming the front still read as a plain buttoned shirt. The
first atlas is rejected and remains external as `will-suit-rejected-back-collar.png`.

Actual GPU UV analysis in `will-suit-uv-region-analysis.json` places front
torso UVs near v .7518 through .9818; the rear runs from .51 to .6740.
Hand UVs span .1662 through .6715, including palm islands in the bare-arm
strip. Painting only the lower hand rectangle cannot cover both palms.

The candidate was regenerated twice with the exact prompts below. The first
correction still placed the ivory opening at v .6930, and the precision
follow-up placed it at .7097 rather than the requested .753. Neither atlas
alone establishes correct front mapping. The current painting's measured
ivory bounds are u .2640 through .3509 and v .7097 through .8134.

Will's front panel now uses a face-level UV mapping to that measured collar
region in `painted_torso.rs`; rear and noncloth islands are preserved.
`handwear.rs` splits complete hand faces onto the authored glove sample
(.87,.69), retaining every position, normal, color, joint and skin weight.
It applies only to the explicit `whitegloves` accessory, currently Will.
The new ignored supplied-rig regression verifies complete glove-face UVs
and preserved nonhand mapping and ordered geometry/skin attributes.

Fresh build/render job 34403 waits for the earlier GPU batch before rebuilding
and capturing this complete revised candidate in Idle and two Run phases.
Handwear regression 73676 waits for the freshly compiled libtest. Earlier
queued batch 64517 is intermediate evidence and may lack the revised front
mapping; do not use it as final-candidate proof. No candidate is accepted or
deployed. Canonical Will remains unchanged.

### Garment correction prompt (built-in imagegen)

Use case: precise-object-edit. Asset type: existing square anime game UV albedo atlas. Correct ONLY the red suit garment rectangle in the lower-left of this supplied atlas. Preserve the dark purple domino mask, drawn eyes, purple hair, entire face/ears/neck, both shoes, upper-right skin arm strip, existing white glove rectangle in the right middle, burgundy trouser rectangle bottom-right, and all gutters unchanged. Exact atlas UV layout and dimensions unchanged. Critical mapping instruction: the BACK torso uses the UPPER part of the lower-left red rectangle (image normalized y=0.50 through 0.735). Erase the existing ivory opening, lapels, seams, buttons and pockets there; make this upper portion uninterrupted restrained red jacket cloth with soft broad folds and a subtle red rear seam, no white and no V. The FRONT chest is the LOWER central part of the same red rectangle, x=0.245..0.385, y=0.75..0.985. Put all front suit detail ONLY there. The neckline is centered at x=0.315,y=0.753; small ivory inner-shirt opening spans x=0.28..0.35 at this neckline and tapers to a V point at x=0.315,y=0.845. Place narrow dark red lapels around that small ivory V, confined to x=0.25..0.38, y=0.753..0.87. A restrained red center fastening seam continues from V tip to y=0.98, with two small dark red buttons. Small understated red pocket seams at y=0.92 if space allows, staying in that front chest region. Keep the rest of the red rectangle simple red fabric, including its left/right sides. This is a precision flat UV texture correction, never a body portrait. Clean subdued cel-shaded anime cloth, no photoreal fabric, no text, no diagram, no outline around UV patches, no transparency. Preserve all other elements exactly.

### Precision follow-up prompt (built-in imagegen)

Precision UV texture edit of this 1254x1254 atlas. Keep every pixel outside the LOWER-LEFT RED GARMENT RECTANGLE visually unchanged. The small ivory V and dark-red lapels are still too high and too far left. Move ONLY that ivory V and its enclosing lapels EXACTLY 36 pixels to the RIGHT and 76 pixels DOWN, keeping their current size and shape. Fill their vacated location seamlessly with the surrounding red fabric. The ivory neckline's top edge must end up at image row 944 (normalized y=0.753), centered at image column 395 (normalized x=0.315). There must be NO ivory garment pixels above row 935. Realign the jacket front fastening seam to column 395, running from the moved V apex down to row1230. Erase all existing garment buttons and pockets, then add only two restrained dark red buttons centered at (395,1110) and (395,1175). Keep the upper half of the red garment rectangle uninterrupted red cloth. Preserve the mask, eyes, face, hair, shoes, arm strip, white glove patch, trousers and gutters exactly. This is a flat albedo UV atlas; no portrait, no scene, no labels. Do not reinterpret or recenter the UV layout.


### Intermediate corrected material renders

All three images from batch 64517 (four-angle Idle, Run .125 and Run .625)
were individually inspected. The mask reads, the front opening and buttons
are visible, the back is plain red, and both hands now read white, including
palms. These are clear material improvements over the rejected first atlas.
The hair/head/jaw profile and flat hooked hands remain unfinished. The
fresh build 34403 remains required to establish the current fitted source
without relying on source/build timing. New pixel invariant measurements
for the final bitmap are in external `will-fitted-suit-invariants.json`;
earlier invariant figures apply to the rejected first image. Canonical
painting remains unchanged.


### Current local retained material

Fresh build/capture job 34403 passed, with all three actual Idle/Run .125/
Run .625 images individually inspected (twelve views). The mask and front
opening/buttons read correctly, the rear is uninterrupted red and the
complete hands read white. This material correction is retained locally in
`assets/open-models/people/npc-textures/will.png`, SHA256
`18f75cc8d31497a8638e5458458b7ef2d7d905972bb0b07db8ec78c1a94b79aa`.
Earlier statements that canonical Will is unchanged describe the rejected
and pending stages, not current state. An external backup preserves the
previous canonical painting. Built-in imagegen supplied this retained
bitmap; all prompts above are exact.

Canonical capture 48126 passed and was individually inspected. Its decoded
1600x600 RGB pixels exactly match the reviewed fresh candidate Idle frame.
Actual GPU comparison with the initial Will export preserves ordered joints
and weights, all nonfinger positions/normals, and UVs outside hand/cloth
regions; hand shape differences are the separately reviewed finger candidate.
External evidence: `will-fitted-suit-diagnostic/geometry-check.json` and
`will-material-installation.json`. This proves the local mapped material
correction, not a finished character.

The current bitmap is not pixel-identical outside edited regions. Protected
mean RGBA differences vs the previous canonical image range 2.12..3.27,
with maximum difference 127 at shoe-detail edges; rendered protected regions
were inspected. Hair/head/jaw/nape, flat hooked hands, footwear and cloth
geometry still require work. Fresh digit/contact/glove checks are running
in retry 35033 after a transient unrelated shadow-import compile failure.
No browser bundle or engine was deployed, and no whole model is approved.


Fresh retry 35033 is now terminal PASS. It executed the actual supplied
four-rig fingertip/wrist/attribute/closed-surface/animation regression, then
the production full-cycle mounted hand/pedal contact regression and the
white-glove ordered geometry/skin/nonhand-UV preservation regression.
All three actual tests passed. Current source and binary hashes/results
are external in `player-rounded-digit-retry-results.json`. These are
geometry/animation/material guarantees, not whole-model artistic approval.
The visibly flat comb-shaped hand still needs a cap/profile correction.
