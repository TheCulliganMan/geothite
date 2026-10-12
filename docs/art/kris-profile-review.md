# Kris profile review

The actual front, side and back baselines were individually inspected
(capture sessions 41837, 17134 and 56714). The front painting reads clearly,
but the side jaw was vertical, the cap was a shallow oval, and the rear hair
and pigtails were broad and rigid. Outfit, hands and shoes remain unfinished.

## Profile correction

The TrainerFemale mesh now uses the rig-relative small nose and receding chin
while retaining the authored smooth anime normals. No painting or rig was
replaced. Build 41099 passed; the native reviewer SHA256 was
`a9c802fa68048411f5b16e09a82f61394fc2fbb8e1073f8f8cef6c52faa178d3`.

Captures 58135 (front), 83980 (side), 68528 (Run .125), 79664 (Run .625)
and 61591 (mounted .25) all exited zero. Each actual image was individually
inspected, covering fourteen views. The front remains clean and the chin
recedes through the sampled poses. This local profile correction is retained.

The actual GPU comparison preserves UVs, skin IDs and weights, triangles and
normals exactly. Only 537 of 15,463 vertices moved in Z, at most .0941764 source
units. Body, mixed neck, eye plane and crown were preserved. The external
evidence is `kris-profile-candidate-diagnostic/geometry-check.json`.

## Cap and hair review

Kris's generic oval cap has been replaced with a fitted soft blue crown and an
attached narrow darker lower band. Build 41160 passed; reviewer SHA256 was
`dd6e6f0805a6ffaf830de59a90292f94dc799dab32a26969a439dade82aa4382`.
Front, side and rear captures (77943, 46469 and 63645) exited zero and were
individually inspected. The band sits against the head and the forehead stays
open. Motion and mounted validation of this cap are still pending.

These views expose the pigtails as broad leaf-like forms with a brighter color
than the painted scalp. A Kris-only candidate now samples the actual painted
hair strip and uses three overlapping tapered locks per gather. Other tied
hairstyles retain their existing geometry. Build 56870 passed in 1m47s; reviewer SHA256 was
`865c88d28bc96e1a704bbb03658d221f530e9d4400259c85edbac69e6cd16714`.

Seven actual captures were individually inspected: front, three-quarter,
side, rear, Run .125, Run .625 and mounted .25, totaling sixteen views.
Capture sessions 49128, 28625 and serial batch 54016 all exited zero.
The pigtails now taper into separate overlapping tips, match the painted
scalp and retain their attachment through the sampled poses. This local
cap and hair improvement is retained. Broad rear scalp and harsh printed
clothing folds remain visible; neither character nor outfit is approved.

Whole character approval, current mounted envelope validation and deployment
remain outstanding. All previews remain muted.

## Rear scalp candidate and validation

The sixteen-view cap/gather review still exposes a broad flat rear scalp.
A new candidate applies bounded rear-hair refinement to the supplied female
rig, preserving face, skin boundary and crown support. The dedicated mapping
regression now covers both player rigs and explicitly protects skin and ears.
Build 33661 is active; appearance remains unverified.

Mounted regression build 85351 failed to compile before executing the test:
eleven room test calls lacked the interior renderer's new map argument.
Those calls now use their existing fixture map identities. The concurrent
interior-model moved-value error was already repaired in current source.
Fresh current-source geometry compilation is queued behind the reviewer build;
current mounted and gathered-hair regressions still need passing results.

Rear scalp build 33661 passed in 5m45s including the prior build wait; reviewer
SHA256 was `2a4566671684705356f30cf94aa613ada60784fb433eaf0e45b132fd64f54ecd`.
Serial capture batch 9487 exited zero for front, side, rear, Run .125, Run .625
and mounted .25. All six actual images were individually inspected, fifteen
views total. Rear curvature is modestly improved and the sampled cap/gather
attachments remain intact. This local correction is retained; the painting's
rear hair bands, nape, outfit, hands and shoes still need further work.

The actual GPU comparison has 20,855 vertices versus 15,463 before. Canonical
directed triangle attributes match for all 21,210 body/mixed-neck triangles
and 1,508 front-head triangles, at six decimal source-unit precision. The
comparison includes positions, normals, UVs, joint IDs and weights. External
evidence is `kris-rounded-scalp-diagnostic/region-check.json`.

Fresh capped-player regression 98877 is compiling. It must pass before running
the newly compiled tied-hair and mounted contact/culling regressions. No test
pass, deployment or whole character approval is inferred from these renders.

The fresh character lib binary (mtime 2026-10-12 10:40:25) now passes all three
direct regressions in serial session 89978: both supplied player scalp rigs
(.09s), all sixteen closed gathered hairstyles (8.24s), and current player
mounted hand/sole contact plus body culling over 32 phases (8.22s). The union
Cargo command 98877 is still compiling other selected crates; its completion
is not inferred from these direct results. These checks precede the proposed
new clothing painting.

Union Cargo command 98877 completed successfully in 8m33s. It ran one actual
capped-player regression, covering both supplied rigs, in .09s; the other
selected crates had no matching tests. The earlier direct gathered-hair and
mounted tests remain separate passing evidence.

The calmer clothing painting was subsequently installed after fifteen mapped
candidate views and four canonical-root views. The canonical Run image matches
the candidate's decoded pixels exactly. See kris-clothing-review.md for the
asset path, prompt, pixel changes and limitations. Engine and asset-bundle
preparation for local deployment are active; no served update is yet claimed.

The verified player browser engine is now served locally with hash
`28c8d5297be2d46df6af84626e6b1fe07544fcc8465ea75e2d1dc0db9c89853c`.
Index, glue, engine and JSON/gzip bundle bytes all match the staged files over
HTTP. The three cache references use its new prefix, and startup stays muted.
The browser open request was queued; actual fresh browser frame inspection
remains unverified. Completed staging/backup scratch was removed. Subsequent
shoe candidates are separate and are not in this served build.
