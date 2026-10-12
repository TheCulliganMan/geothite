# Short skirt motion trial

Current mapped Run views of Karen and the third motion batch show hems pulling
into knee-shaped scallops. The current closed-garment skin field gives the
lower hem 0.60 combined thigh influence and the remainder to the pelvis;
upper panels blend toward the fitted waist field.

A native people-review-only control,
`CRYSTAL_PEOPLE_REVIEW_SKIRT_LEG_SHARE`, permits comparison from 0 to 0.60 for
closed garments whose bind hem is at least 0.70. Long robes, kimono and open
coats retain their existing fields. The production default remains 0.60.
The next candidate is 0.28, requiring actual mapped comparisons and checking
that exposed lower legs do not pierce the less-following fabric. It has not
been rendered or accepted yet.

The existing review mesh exporter now also emits `ROLE.costume.json` with
positions, normals, UVs, indices and skin influences. The corresponding actual
production `ROLE.pose.json` palette can deform both body and costume surfaces
for intersection diagnostics. Export is opt-in, native and review-only; data
belongs in external scratch, not the shipped bundle.

The closed-garment shading regression now transforms normals using the inverse
transpose of the blended skin matrix rather than averaging separately
transformed vectors. It covers 25 closed-garment roles at 16 Run samples,
checking noncollapsed vertical panels and outward shading. Horizontal rim caps
remain excluded because they share side normals. The corrected regression
is pending in live session 70387; the control/export reviewer build is pending
in live session 14167. Neither is a completion claim or proof of collision-free
garments. The known Kurt robe/shoe crossing remains unresolved separately.

An external offline checker, `inspect-garment-crossings.py`, now deforms body
and garment exports with the actual column-major production palette and checks
strict non-coplanar triangle crossings using a spatial grid. Synthetic hit,
miss, excluded edge/coplanar contact and translated skin checks pass. No actual
garment export has been inspected by it yet; it deliberately cannot prove
absence of coplanar overlap or exact edge contact, or cover an unsampled pose.

Shared-target jobs remained live but queued. An owned copy-on-write clone of
the existing build cache was created under external scratch as
`isolated-garment-target`; it does not share file mutations or the shared build
lock. Reviewer build session 17523 is now actively compiling the current source
there, with the painting regression queued behind it in session 31986. The
original shared jobs have not been restarted or cancelled. This is temporary
verification output and must be removed after the isolated jobs and captures
finish, retaining only mapped evidence outside the repository.

## Actual comparison and rejection

The isolated reviewer finished with exit 0 in 1m40s. SHA256:
`46ee1858821815e2d31c0761fc344eff3ad0bd58736881d6dd14ec4b60e25704`.
The compiled executable contains both the short-skirt control and costume
exporter. Four capture batches exited zero; all twelve images / 48 views of
Karen, Jasmine and Lass at both shares and Run phases were individually
inspected in `captures/skirt-{060,028}-run{0125,0625}`.

The 0.28 share reduces the rounded scallop but leaves conspicuous leg-shaped
notches through the front hem in both poses. It is rejected; production remains
0.60. The actual mesh/palette diagnostic finds lower-body crossing pairs
(body triangles with all bind vertices below y=1.2) increasing from 86 in each
baseline pose to 132 at phase 0.125 and 123 at phase 0.625 in the trial, for
each of the three roles. Counts are triangle-pair diagnostics, not independent
visible holes. Baseline is also not collision-free. Upper-body crossings are
counted separately by subtraction; examples lie at the fitted waist, so total
counts must not be presented as lower-leg defects. Results and actual exports
remain external in `skirt-*-crossings.jsonl` and `meshes-skirt-*`.

The freshly linked isolated unit-test binary contains the skirt control.
The revised inverse-transpose shading regression was executed directly and
failed on Kurt at Run phase 4/16, triangle ids `[5155,5179,5180]`, bind y about
1.16–1.17. Its averaged shading normal opposes the animated geometric normal.
This is new evidence from the corrected calculation, not a passing test of all
25 roles. Kurt geometry/shading and existing shoe crossing remain unresolved.

Kurt was then captured silently at the failing phase through the actual
production update path (`captures/kurt-shading-run250/kurt.png`, four views
individually inspected; capture exit 0). The exported costume and actual
palette reproduce the same panel's shading disagreement: bind geometric/shading
dot 0.197347, actual animated dot -0.026434. Thus the failure is not confined
to the unit test's sampling path. The lower robe still curls into a foot-shaped
hem; the rear remains a plain fabric surface. No geometry fix is accepted yet.

## Pocket-edge diagnosis

The failing ids are in the appended apron pocket, not the main robe shell.
The full costume has 6582 vertices; the main cloth ends at 2706, followed by
1650 apron vertices, 1650 pocket vertices and 576 belt vertices. The failing
ids fall near the front pocket perimeter. An offline amplitude sweep that
recomputed normals over the entire merged costume was not a valid robe-fold
trial: it also changed apron/pocket normals. No production fold amplitude was
changed from that sweep.

An isolated calculation on the actual pocket adjacency removes the sampled
shading disagreement when the corner normal is constrained to face every
incident triangle with a small positive dot margin. Source apron normals now
apply twelve projection sweeps with margin 0.20 after the existing angle
average. It preserves topology, geometry, UVs and skin attributes by construction;
actual export equality, all affected mapped models and the corrected 25-role
Run regression remain required.

Native build 63239 passed in 1m54. Its reviewer SHA-256 is
`75aa8c2d7291b2de516d526eb21ab15adb0a1fb3adb6b6dc60d8e81827ac42c6`.
All five apron roles were individually inspected at Idle and Run phases 0.25
and 0.625 (15 images, 60 views). Kurt's exported positions, indices, UVs,
joints and weights match the previous export exactly; four vertex normals
changed. At phase 4/16, the original pocket triangle [5155,5179,5180] improves
from facing dot -0.026434 to +0.056322.

Cargo regression 25020 finished with exit 101: the next failure is Kurt phase
5/16, apron front triangle [3497,3521,3522]. A fresh silent mapped capture at
0.3125 was also individually inspected in four views:
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/apron-normal-run03125/kurt.png`.
The sharp knee fold and shoe/hem crossing remain visibly unacceptable.
No browser deployment or whole-model approval follows from this candidate.

Using that capture's actual exported palette, the failing front triangle has
facing dot -0.0012582. An offline angle-normal calculation that excludes the
thin closing rim from the front vertices gives +0.3006383 for this triangle.
This is evidence of a shared front/rim shading seam problem, not proof that
splitting the seam fixes the full cycle. A proper seam must retain matching
positions and skin attributes on both sides and preserve geometric closure;
changing topology needs corresponding closure checks, not dropping them.

The 25-role, 16-phase regression now collects reversed panels across the full
scope, reports counts by role and phase plus at most twelve detailed examples,
and still requires zero failures. Degenerate triangles still fail immediately.
Fresh aggregate run 86470 is in progress in the owned isolated target.
The browser retains the previously validated shirt-gutter build.


The fresh candidate apron fit/closure regression passed directly (1 test,
8.65s; 1303 filtered out). This confirms its own sampled closure/clearance
invariants, not overall body/garment collision freedom.

An independent diagnostic over all 15 exported costumes and actual palettes
reports zero opposing vertical faces in Idle for all five roles. Kurt and
KurtOutside also report zero at 0.25 and 0.625. Mom, RedsMom and PokefanF each
report one opposing face at both running phases. Kurt at 0.3125 reports seven,
including six apron hem faces and the earlier pocket triangle. This rules out
accepting the four-normal projection as a complete correction.
Diagnostics and exact ids are saved externally in
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/apron-normal-export-diagnostics.json`;
`inspect-garment-normals.py` uses the production blended matrix inverse
transpose and excludes only horizontal caps, matching the Rust check.
Direct run 73785 used the still-old test binary and repeated the phase-5
failure; it is not the aggregate result. Do not rerun until the newly linked
aggregate binary or Cargo run 86470 completes.


A further offline experiment recomputed angle normals from front fabric faces
only, preserving the candidate's back and rim normals elsewhere. On every
vertical triangle (including closing rims) in all sixteen exported poses,
opposing-face counts become zero: this includes the six short-apron failures
and all seven Kurt phase-5 failures. Exact results are saved externally in
`apron-front-only-normal-experiment.json`. This is stronger evidence for
separating the broad front shading field from the narrow closing rim, but
still covers only the exported poses. No production implementation or
full-cycle acceptance is claimed from this offline result.


Fresh aggregate direct run 53454 completed (exit 101, 10.85s, 1307 filtered):
25 roles were checked; only the five apron roles fail. Kurt and KurtOutside
each have 26 reversed panels across phases 5,6,7,8,13,14; Mom, RedsMom and
PokefanF each have 11 across phases 4 through 10. Total 85 opposing panels.

Production source now implements the offline front-only angle-normal field
for apron and pocket panels after the rounded closed-surface calculation.
The belt retains its prior calculation; topology, UVs, fitted positions and
skin weights remain unchanged. This is a new candidate, still requiring
fresh build, actual mapped review, fit/closure and full-cycle validation.
Native reviewer build is queued in the owned isolated target. Browser bytes
have not been changed.


Native build 19381 passed (exit 0, 2m24s). Reviewer SHA256:
`937439f7c127c7818b7db05b1fc4f7c4393ab9b7a233406f7176e5cd49a6ee8e`.
All five affected roles were silently recaptured at Idle, Run 0.375 and 0.625;
processes 10126, 25531 and 34785 exited zero. All fifteen images / sixty views
were individually inspected. Artifact directories:
`captures/apron-front-{idle,run0375,run0625}`. The pocket fronts read more
continuously, but small lit edge dots remain on the ivory pockets. Existing
Kurt robe/knee folds and shoe intersections, short-skirt leg-shaped folds,
angular jaws, rigid hair, generic hands/shoes and plain rear outfits remain.
No whole-model approval and no browser deployment.

Actual exports confirm every non-normal costume attribute unchanged for all
five roles at Idle and Run 0.625; 224 normals change per role. Independent
production-palette diagnostics report zero opposing faces at Idle and 0.625.
At 0.375, only one pocket triangle fails for each short-apron role. Exact
results are external in `apron-front-export-verification.json`.

Fresh direct full-cycle regression 56148 failed (exit 101, 17.71s, 1309
filtered): only Mom, RedsMom and PokefanF phase 6/16 remain, each with ids
[4499,4523,4524] and facing -0.16948673. Total failure count decreases from
85 to 3; Kurt/KurtOutside and the other twenty roles pass all sampled phases.
Fresh apron fit/closure regression 91609 passed (one test, 9.16s). Cargo run
2865 still completes dependent crate compilation; do not report it as passed.
The residual pocket triangle has a posed geometric z component below zero;
this needs deformation/shape diagnosis, not a weaker facing threshold.


The remaining pocket face sits at the last grid row/right edge. Its quad is
[4498,4523,4499,4524]. In the actual exported female phase-6 palette, original
triangles have facing +0.985818 and -0.169521. The alternate diagonal yields
+0.964598 and +0.746924 with the existing vertex normal field. Bind double
areas improve from [0.000053916,0.000032001] to the more balanced
[0.000041879,0.000041623]. These values diagnose the thin rounded corner;
they do not prove whole-cycle acceptance.

Panel triangulation now chooses the diagonal maximizing the smaller triangle's
scale-independent aspect score (squared cross area divided by squared sum of
edge lengths squared). Front and back use matching diagonals with opposite
winding; ordered closing rim is retained. The rule applies to apron/pocket
panels, never the belt or underlying robe. Because the apron sampling
triangles change, pocket fitted skin influences can also change through
barycentric interpolation; do not claim every skin attribute remains exact
for this newer candidate. Native build 1712 and a fresh full-cycle test are
queued in the owned isolated target. Fresh mapped render and fit/closure
validation remain mandatory. Browser bytes are unchanged.


Cargo run 2865 subsequently finished (exit 101; total build 10m02s, actual
regression 15.71s) with the same three short-apron phase-6 failures. It tests
the front-normal candidate, not the newer triangulation candidate. Build
1712 is now actively compiling the latter; regression 81556 is waiting on
that owned target's lock. No job is restarted merely because compilation is
quiet. All mapped reviews this turn preceded the triangulation edit; it has
no fresh render yet.


Triangulation candidate native build 1712 passed (2m23s), reviewer SHA256
00c369453b6fe538c8c8b5ae9053c872da4cc7485a3eb3721767c361f908583b.
All five apron roles were recaptured silently at Idle, Run 0.375 and 0.625;
all fifteen images / sixty views were individually inspected. Capture jobs
33720,95773,32781 exited zero. `captures/apron-aspect-{idle,run0375,run0625}`
contains the actual mapped views. Independent production-palette diagnostics
confirm zero opposing vertical faces in all fifteen exported poses
(`apron-aspect-export-verification.json`).

Fresh direct full-cycle regression 95674 passed (one test, 18.71s), with all
25 roles at 16 phases and zero failures. Apron fit/closure regression 14344
passed (one test, 9.33s). Full Cargo run 81556 also passed (one actual shading
test, 18.78s; total build 10m05s). The prior 85 failures are resolved within
this sampled shading scope; no claim of collision-free garments, acceptable
silhouettes or complete character quality follows. Small lit pocket-edge dots
remain visible on ivory pockets. Kurt knee-shaped folds and shoe/robe
crossings, short-skirt leg-shaped folds and generic hands/shoes remain open.

The newer reviewer build 96239 passed (10m41s), SHA256
e44307d6258fddab00b75cc6a0a057d7151a414320bf4a95615217cdf23b63a5.
It includes the separate Cooltrainer M gutter correction. Browser build 17798
passed (3m23s), and bindgen 46873 passed. Browser installation is recorded
below; no refreshed live browser frame or gameplay check is claimed.

Validated engine and glue installed in the local served root; HTTP bytes
match both files. Engine SHA256:
0784e8847bafa3990d6e08a8d0cdbc223504fbe8ba5a7d22a423b265e973601f.
Glue SHA256:
d98b8d5814a24b6d33e8b1162927d9c25db66b290688b714850ceb9f69da0a03.
All three served cache references use 0784e8847bafa399. Startup mute is
preserved. The four-rig/75-painting bundle remains installed with the
Pharmacist face replacement. Temporary WASM staging removed. No live browser
refresh was performed; this proves served bytes, not a rendered browser frame.
