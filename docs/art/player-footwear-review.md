# Player footwear review

Offline review now supports `CRYSTAL_PEOPLE_REVIEW_DETAIL=feet` with the
image target and a single view (0 through 3). It changes only the close-up
camera; default full-body/four-angle views and production rendering remain
unchanged. This mode exposes ankle, outsole and lace details missed at the
normal review scale, along with unfinished hooked/faceted hand silhouettes.

Reviewer build 60786 passed in 4m18s, SHA256
`800ae1d4dfb53fdeadb4a51a81ec478ef553dd43fd64caf242aba382b13d6936`.
Front, side and rear close-ups of Chris and Kris all exited zero in serial
session 74473. All six actual images were individually inspected. The shoe
uppers are shirt-tinted, heels stretch the lace/crease painting, and outsole
edges remain angular. These shoes and whole models are unapproved.

Actual GPU sole UVs reach skin outside the shoe drawing: 106 lowest vertices
span U .63589 through .94157 and V .05427 through .48131. Many sample warm
skin RGB around (253,192,147). Both authored white shoe paintings have a
neutral plain toe panel near UV (.716,.465), RGB around 249 through 251.

The first candidate restored the painted upper palette and moved only low
sole vertex UVs to that panel. Build 13620 passed in 2m01s, reviewer SHA256
`b12e41bf3686ff36e80bb40deafa9f37efbc8f5e787d3af6c310e4d0248910eb`.
All six capture jobs exited zero (two players each) in session 43231. Only
both front close-ups were inspected before rejecting this variant: the right
sole interpolates through the lace drawing and gains stretched black stripes.
A good sampled vertex color is insufficient evidence for a clean mapped face.
This rejected variant was never deployed.

The current candidate gives rubber faces their own UV seam, copying contact
positions, normals and skin attributes exactly while retaining laced upper
UVs. Reviewer build 78574 passed in 6m32s, SHA256
`7a6d760bc61e3eb4f67e13dc99c9e24edaa6a7a8be41d9db5504894b4a715a64`.
Front capture 7305 and serial side/rear/Run .125/Run .625/mounted .25
capture 85617 all exited zero. All twelve actual images were individually
inspected: thirty views total. The stretched sole stripes are absent across
these poses. The sole mapping is retained locally; complete shoes remain
unapproved because rear artwork is still stretched and the outsole contour
is angular. Hooked, flat hand silhouettes also remain unfinished.

Ordered actual GPU triangles match the prior player builds exactly for
positions, normals, joint IDs and weights, despite 152 added UV seam vertices
per player. Only 160 rubber faces per player change UVs; upper and non-shoe
UVs are unchanged. The comparison is saved externally as
`player-footwear-seam-diagnostic/geometry-check.json`.

The regression samples interpolated UVs over entire rubber triangles and
checks unchanged geometry/skin plus upper UVs and neutral upper tint. Test
1511 exited zero, but its launch preceded the latest source repair and is
not treated as current-source proof. Fresh union Cargo test 97102 passed: the actual footwear test ran for
9.77s; other libraries had no matching tests. This proves the sole-only
source compiled before the later heel extension. The mapping is not deployed.

The separate verified head/hair/clothing/rider engine is now served locally,
SHA256 `28c8d5297be2d46df6af84626e6b1fe07544fcc8465ea75e2d1dc0db9c89853c`.
All five HTTP files match staged bytes; four rigs and seventy-five paintings
are present, with only Kris's painting changed. Browser opening was queued,
so no actual fresh WASM frame is claimed. Previews remain muted. This served
build does not include either shoe candidate.

## Heel counter candidate

The rear close-ups still show stretched lace/crease artwork around the heel.
The next candidate gives complete rear shoe faces (Z <= -.10, Y <= .235)
the same plain white leather sample as the sole. Front laces, side panels
and ankle collar remain painted. This extends the existing UV seam without
moving geometry or altering skinning. It is unrendered and unaccepted.

Build 46684 (`player-heel-counter-build.log`) and fresh regression 75610
(`player-heel-counter-regression.log`) were queued behind sole-only test 97102, which has now passed. Reviewer
build 46684 is actively compiling; heel regression 75610 remains queued. The regression now requires over one hundred heel upper faces
per player and checks interpolated neutral albedo across their triangles,
while preserving instep/side/collar/non-shoe UVs and ordered geometry/skin.
After compilation, inspect actual rear close-ups first; do not infer art
quality from a test result. No footwear variant has been deployed.

Reviewer build 46684 passed in 6m52s, SHA256
`41df8f0a1abdb3d9fa89fbccc309dd2b794ebe6971284702ab79063cdc45468d`.
Rear capture 69537 is running. Fresh heel regression 75610 is active.

## Nurse lower-body evidence

During the build, serial capture 46859 produced front/side/rear close-ups
of the actual nurse. All three jobs exited zero and all three images were
individually inspected. Her calves and soles are visibly faceted, the rear
shoe artwork stretches, and the fingers remain flat and hooked. No nurse
geometry changed in this turn, and the whole model remains unapproved.
Per-job reviewer SHA256 is saved in the external
`nurse-feet-detail-captures.json` (the reviewer was rebuilt during the batch).

## Heel patch rejection and next candidate

Rear capture 69537 exited zero. Both actual rear images were individually
inspected and the isolated heel patch was rejected: its white area makes
sharp diagonal edges against the remaining painted side creases. Ordered
GPU geometry, normals and skin attributes were unchanged, with 386 remapped
faces per player; that invariant does not establish acceptable art. The
rejected patch was never deployed. Test 75610 remains live and covers only
this rejected revision, regardless of its eventual result.

The new candidate wraps plain leather continuously across rear and lateral
shoe faces, while keeping the upward-facing laced instep and ankle collar.
Its field uses rear Z <= -.10 or abs(face-normal Y) < .55, max Y <= .235,
inside the existing shoe mask; sole faces retain the verified plain sample.
The actual prior GPU mesh has 160 rubber and 552 selected upper faces per
player. Source geometry and weights are copied exactly across the UV seams.
Build 64848 and current-source test 36208 are queued behind 75610. This
candidate is unrendered, unaccepted and undeployed. Inspect rear and side
close-ups first, then the protected front and movement views if it improves.

The actual Chris GPU hand-tip analysis (index joints 32 and 43, total
weight > .85) finds 4,352 fully selected triangles and 2,241 unique
geometric positions per side, with 128 pinned boundary positions. This is
a dense surface with an unresolved flat silhouette. Another blanket
subdivision pass would increase cost without addressing its volume. The
next hand candidate should sculpt the fingertip/knuckle contours and volume,
then inspect mapped Idle/Run and actual mounted grip contacts. Analysis is
saved externally in `player-hand-tip-topology.json`.

## Face-masked revision rejected; continuous upper layout pending

Build 64848 passed in 9m56s, reviewer SHA256
`49d7e5486b6f63d29a99a7ba6909c39e7a797a1d44d69f6631a7c526cf0a0f53`.
Serial capture 46231 produced both rear and side images of both players.
All four actual images were individually inspected. The face-normal mask
still leaves isolated triangular remnants of the painted side panels; this
revision is rejected and was never deployed. Test 36208 remains active and
cannot establish acceptable art for the rejected revision.

The next layout maps all shoe upper faces through one continuous field
inside the second authored shoe island. Raised central instep vertices reach
the lace drawing; toe, heel and lateral edges reach plain leather. Adjacent
upper triangles share identical UVs at geometric seams, with a separate
neutral sole seam. Geometry/normals/skin attributes remain exact copies.
This changes the entire upper UV layout, rather than claiming unchanged
instep UVs. No bitmap painting changes.

Independent sampling of all 1,050 actual shoe faces per player, with fifteen
barycentric samples per triangle, finds no warm/unrelated-color samples
(channel range >=12), no dark soles, and 1,027/1,374 dark lace samples for
Chris/Kris. Max channel ranges are 9/10. This supports atlas placement only;
it does not prove the new rendered art. Results are external in
`player-parametric-shoe-albedo-analysis.json`.

Build 70234 (`player-parametric-shoe-build.log`) and test 13314
(`player-parametric-shoe-regression.log`) are queued behind 36208. The test
now protects non-shoe UVs and exact ordered geometry/skin, samples sole and
upper albedo, checks continuity across upper geometric seams, and requires
retained lace coverage. Inspect side/rear/front and motion after compilation.
The new layout is unrendered, unaccepted and undeployed.

## Continuous upper rejected; retained source restored

Build 70234 passed in 8m04s, reviewer SHA256
`651e2e3b810fe5d38c2c72a5570aa0c91549e1ad9bebe4f2b2c95ff57221dae5`.
Serial 87653 produced side/rear/front images of both players; all three jobs
exited zero and all six actual images were individually inspected. The
field folds lace glyphs into concentric rings. This revision is rejected.
All 1,050 shoe faces per player changed UVs, with actual ordered positions,
normals and skin attributes exact and non-shoe UVs exact. Those invariants
and grayscale pixel checks do not prove recognizable lace artwork.

All rejected upper fields and face masks have been removed from source.
The retained source again uses the sole-only seam and white painted upper
palette, preserving the authored upper UVs exactly. This is the previously
inspected/tested local repair, not a runtime fallback. No bitmap changed and
none of the rejected revisions was deployed. Build 8025 regenerates the
retained reviewer; old candidate test 13314 is still active ahead of it.

The next upper detail needs an intentional lace/tongue layout rather than
further folded sampling of a top-down shoe glyph. Actual triangle ray samples
locate thirty points on each player's upper at Y .18142 through .21534,
using leg/foot skin joints 50/51/55/56; these can anchor modeled crosslaces
with interpolated source weights. Data is external in
`player-shoe-lace-surface-samples.json`. Proper mapped glyphs or attached
modeled laces must be reviewed in actual poses, along with unchanged contact
geometry and clear heel/side/collar surfaces.

## Modeled crosslace candidate

Player-only source now samples plain white leather for every supplied shoe
face and appends three crossed lace rows per foot in native Rust. Twelve
strand paths are vertically fitted to the actual upper before any tube is
added; barycentric source influences produce the strongest four normalized
skin weights. Each strand has nine eight-sided rings and two closed caps,
888 extra vertices total. Radius .0025 and offsets .0055/.011 keep the two
crossing layers distinct. The original body/contact positions, normals and
skin attributes remain exact. No bitmap or other character's shoe changes.

Current build 95446 (`player-modeled-laces-build.log`) and supplied-rig
regression 64060 (`player-modeled-laces-regression.log`) were dispatched after
the final source was written/formatted. Earlier queued build 8025 may now
compile this new source; it must not be called a retained-sole-only build.
Test 13314 still covers the previous revision. All handles remain active.

The new regression protects original geometry/skin, requires all tube edges
to have two opposite directed incidences, rejects degenerate triangles, checks
actual upper clearance .001 through .017, and validates normals and weights.
The existing albedo regression now compares the complete ordered body prefix
while allowing appended lace triangles, protects non-shoe UVs, and samples
all shoe faces for neutral plain leather. Fresh actual front/side/rear and
Run/mounted renders, exported GPU prefix checks, and the existing full-cycle
mounted contact regression are still required. This candidate is unrendered,
unaccepted and undeployed; full shoe contours and hands remain unfinished.

Old parametric test 13314 finished with one actual footwear test passing
in 14.86s. It does not validate the newer modeled-lace implementation, and
its rejected images remain rejected. Build 8025 is now compiling the current
modeled-lace code; fresh build 95446 and test 64060 remain queued.

An independent numeric reconstruction over the actual exported upper tests
864 strand ring vertices per player. Vertical clearance ranges from .00189623
through .01460377 for both, within the intended .001 through .017 range.
This is useful placement evidence, not a native test result or rendered art
approval. Saved externally as `player-modeled-laces-clearance-analysis.json`.

## Modeled lace review; conforming ribbons pending

Build 8025 passed in 14m51s and includes the modeled-lace code. Capture
84857 produced front/side/rear views of both players; all three jobs exited
zero and all six actual images were individually inspected. Capture reviewer
SHA256 is `c0dc8fa6eb9e2ab00e2cfea3b12d4f5a01340a5d101a08e0f5176768ba873805`.
Fresh build 95446 also passed in 11m17s, reviewer SHA256
`2995f33ec115e78b37045d90a9f772fb0ccd818185be9cce408f8089adcd373a`.
The captured plain uppers and soles no longer stretch glyphs or show painted
side/heel triangles. The round strands sit too high and look like wire in
side views, so complete laces/shoes are unapproved. Test 64060 still runs
against this earlier candidate. Neither build has been deployed.

GPU comparison preserves every original ordered body triangle's positions,
normals and skin attributes exactly and all non-shoe UVs. The shoe region has
1,050 remapped faces and 1,728 added lace triangles per player; comparison is
external in `player-modeled-laces-diagnostic/geometry-check.json`.

The new strands are flat ribbons, halfwidth .0035 and halfthickness .0006,
neutralgray (190,194,194), with crossing layers .0018/.0032. Merely lowering
and widening the original path clipped a surface ridge: independent analysis
found minimum clearance -.00234453. That intermediate was not rendered or
deployed. Each ribbon vertex now fits its own ray-sampled upper point from
an immutable foot-only snapshot and inherits that point's skin weights.
Only added ribbon normals are rebuilt from triangle areas. Designed ring
clearance is .0012 through .0038, with a fixture requirement of positive
.00005 through .008. Original body/contact geometry remains exact.

Final source was written and formatted before build 47581 and test 1461 were
dispatched. Both are queued behind 64060. Actual new renders, current-source
closure/clearance/albedo checks, and the full-cycle mounted contact regression
are still required. Full shoe contours and hand anatomy remain unfinished.


### Fitted ribbon render review, October 12

The fresh native reviewer build (47581) finished successfully in 12m02s.
Muted capture batches 5214 and 77000 finished successfully. Both players were
individually inspected in six images each: front/side/rear feet closeups,
four-angle Run at .125 and .625, and four-angle mounted at .25 (30 views total).

The smaller layer offset removes the large floating-wire gap. The side view
still presents sharp zigzag strands, heavy shoe mass, irregular heel/collar
planes and flat hooked fingertips. This is not a shoe or whole-model approval,
and it has not been deployed. The next correction must address actual contours
rather than trying another UV field over the same shape.

Actual GPU exports in the external player-ribbon-laces-diagnostic directory
preserve every original ordered body position, normal, joint and weight, and
every nonshoe UV. Each player has 1728 added lace triangles. All 888 added
vertices per player have upper clearance .001199995 through .003800014,
confirmed against original upper triangles. These checks establish fit and
attribute preservation, not artistic quality.

The current regression (1461) is still live compiling the current voxel
library/test. The older regression (64060) passed its earlier lace fixture;
that does not replace current-source proof. After 1461 completes, run the shoe
albedo and full-cycle mounted grip/pedal regressions on that freshly built
libtest binary. Keep previews muted. All 75 whole models remain unapproved.


### Current fitted-ribbon regression results

The union command 1461 stopped on a contemporaneous unrelated client dependency
error (`VisualActor.model_palette`). Its voxel libtest finished compiling.
Direct batch 6662 ran that fresh binary: fitted-crosslace closure/clearance
passed (.05s), shoe albedo passed (5.68s), and full-cycle mounted hands/pedal
contact passed (8.77s), each with one actual test. This is current lace proof;
it is not proof of the subsequent fingertip shape candidate.

`hand_digits.rs` now replaces the three narrow Gaussian cuts with a continuous
cosine arch spanning each fingertip, including the two outer edges. The
connected mesh, existing joints/UVs, protected wrist/base selection and
existing taper activation remain. This targets the broad flat tip silhouette
seen in the actual closeups. It is a shape candidate, not an accepted hand.
Fresh scoped reviewer build 4465 is active; digit regression 6264 and a serial
muted six-job render batch wait on its terminal success. Pending state and
logs are external. Inspect every resulting frame and run fresh cycling
contacts before accepting the candidate. No current shoe/hand candidate has
been deployed; all 75 complete models remain under review.


### Sole-wall topology evidence

Actual fitted-ribbon GPU shoe geometry (Chris, right foot) has 30 unique floor
positions at Y .000684 and no vertices in Y .015 through .045. The next 44
positions are at Y .077121 through .085526. This leaves a coarse tall sole
wall below the already refined upper. Its width grows from .283098 at the
floor to .309229 at the next ring. The heavy silhouette in the reviewed closeups
is therefore a geometry issue as well as material shading. External evidence:
`player-shoe-contour-bands.json`.

Next shoe work should refine and bevel this connected wall while preserving
the floor contact surface and ankle join, with interpolated skin and joined
neighbor edges. The current hand candidate intentionally retains this shoe
geometry so its review comparison is attributable. No extra sole smoothing
or bevel has been implemented yet.


### Fingertip criterion checked against the old actual mesh

An independent analysis of both actual pre-contour GPU meshes uses the supplied
rig inverse binds and the same index-finger descendant groups to measure the
visible fingertip silhouette. For three of four lobes on each hand, the apex
is only .00232 through .00510 above the mean shoulders sampled .12 lateral
away. Those flat tips fail the new .015 curvature criterion. The outer lobe
is already rounded enough to pass (.02272/.02435). This establishes that the
new check detects the observed broad flat-tip defect; it does not establish
that the new candidate passes. Evidence is external
`player-old-fingertip-curvature-check.json`. New native rendered and animation
proof remains pending.


### Continuous fingertip candidate actual review

Reviewer 4465 passed in 37m44s. All twelve player images from batch 13176
were individually inspected (front/side/back detail, two four-angle Run
poses, and four-angle mounted per player: thirty views total). Curved tip
edges are visible, but the side/rear hands still read as a flat comb with
pointed lobes, and the palm/profile remains hooked. This is not accepted
as a finished hand. The next correction must address cap/profile volume.

Actual GPU comparison preserves every nonfinger position/normal and all
indices, UVs, joints and weights. Each player has 9499 changed finger
positions, maximum movement .1017426. Independent actualmesh measurements
show fingertip rises .0352..0501 above mean shoulders, confirming the
curvature change. Cap depth ranges .0287..0809; lateral curvature alone
does not establish convincing three-dimensional fingers. Evidence lives
in `player-rounded-digit-diagnostic/geometry-check.json` and
`player-rounded-digit-curvature-depth-check.json` externally.

Regression 6264 stopped before running on a transient Bevy Cascade import
error in contemporaneous shadow code. Current source already uses the
correct `cascade::Cascade` path, so no shadow edits were made by this task.
Fresh retry is queued behind the fitted Will reviewer build, then runs the
actual four-rig digit, full-cycle mounted contact and glove mapping checks.
Earlier dependent contact/handwear wrappers skipped because no fresh
libtest existed; they are not passes. Nothing new is deployed.


Fresh retry 35033 is now terminal PASS. It executed the actual supplied
four-rig fingertip/wrist/attribute/closed-surface/animation regression, then
the production full-cycle mounted hand/pedal contact regression and the
white-glove ordered geometry/skin/nonhand-UV preservation regression.
All three actual tests passed. Current source and binary hashes/results
are external in `player-rounded-digit-retry-results.json`. These are
geometry/animation/material guarantees, not whole-model artistic approval.
The visibly flat comb-shaped hand still needs a cap/profile correction.

## Mirrored fingertip depth and resting-pose review

Fresh reviewer session 62794 built successfully and captured front, side and
rear closeups of both players. All six images were individually inspected,
followed by all six four-angle images from Run phases .125 and .625 and the
mounted phase .25 (session 63147). The terminal pads round through depth,
but the idle palms still hook sharply and finger silhouettes remain comb-like.
Shoes retain heavy faceted heel and sole walls. Neither complete model is
approved, and these changes have not been deployed to the browser.

The actual exported GPU arrays preserve all indices, UVs, joints and weights,
and all nonfinger positions and normals versus the preceding cosine candidate.
Each player changes 9,503 finger positions, maximum displacement .054253.
Independent measurements find all eight terminal pads per player rise
.01996–.04255 above their depth-side samples, exceeding the .008 depth
criterion. These measurements establish local curvature, not art acceptance.
Evidence is external in `player-mirrored-depth-cap-diagnostic/`.

The preceding regression session 57811 failed at compilation on unrelated
waterfall API changes, before tests ran. The current source already includes
the missing API; fresh session 55346 is running. Its compilation started
before the subsequent resting-pose adjustment and cannot prove that change.

The Idle/Run relaxation factor is now .20 instead of .40 to open the hooked
resting hand. Mounted poses still use their separate production grip solver.
Fresh reviewer session 1782 is building this candidate and will capture the
same six closeups. This pose is unreviewed and requires fresh animation,
mounted-contact and glove-mapping regressions. Every command remains muted.

The fresh 55346 test compiled and ran, then failed the outer-finger silhouette
assertion: human-male outer apex .3967085 versus adjacent valley .38106775
(gap .01564, required .035). The depth criterion was not weakened. The local
valley height field was corrected from .080 to .105; the .035 depth rounding
and mirrored depth center remain. Session 21450 now queues fresh regressions
for this contour and the .20 resting pose. Session 1782 began before the
valley correction and represents the intermediate .080/.20 candidate only.

Direct tests from the freshly compiled 55346 binary passed mounted physical
contacts through the full production cycle and exact glove geometry/skin/UV
invariants (session 58604). They cover the preceding .080/.40 candidate,
not the later contour/resting-pose changes. The current candidate remains
unapproved and has not been deployed.

## Broader finger pads

All six intermediate .080-valley/.20-resting-pose closeups from session
1782 were individually inspected. The less hooked pose exposes thin pointed
fingers, particularly from the side; this is not an approved final hand.
All eight four-angle Idle/Run .125 images for Will, Nurse, Youngster and
Black Belt from session 84195 were also inspected (32 views). Their hands
are readable at game scale, but the closeup defect remains. Nurse's moving
skirt hem and exposed leg transitions also remain unfinished.

Session 21450 freshly compiled and passed the four-rig digit depth,
separation, closure, attributes and animation regression for the .105
linear-wave valley and .20 resting pose. The following Cargo pose-test
command failed before execution on unrelated new_bark.rs test fixtures
passing UV tuples to append_quad_colors. No outdoor source was changed here.
Direct use of the first freshly compiled valid test binary passed the
resting-pose regression; mounted and glove tests are running in session
79635. This binary predates the next pad-shape change.

The current height field squares the wave before applying the valley,
concentrating the cut between fingers while leaving broader shoulders on
each pad. This keeps the .105 valley, .035 depth rounding and mirrored
centers. All existing regression thresholds remain intact. Session 42787
is building and will render six actual closeups plus exported player meshes.
This new shape is unreviewed, untested and undeployed; it does not approve
any complete character. Every run remains muted.

All three direct tests in session 79635 passed: resting-pose attributes and
animation envelope, full-cycle production mounted contacts, and glove
geometry/skin/UV mapping. They prove the preceding .105 linear-wave/.20
pose candidate only. The two unrelated new_bark.rs fixture calls were
repaired by expanding the UV rectangle into four coordinates; production
world rendering and tree/water dispatch were not edited. Fresh broad-pad
regressions run in session 68398 and will include that repaired fixture.

Session 42787 built the squared-wave broad-pad candidate in 2m28s. All six
actual player closeups were individually inspected. The fingers remain
flat and thin from side/rear views, so widening the tip outline alone is
insufficient and this hand remains unapproved. Exported GPU comparisons
preserve all indices/UVs/joints/weights and nonfinger positions/normals;
9,492 finger positions change, maximum .024912.

The current variant also rounds each finger across its width: the depth
scale uses the circular-section envelope sqrt(1-wave), fading from the
original palm into a connected thin web between pads. Both sides center
on their measured source cap depths; the prior hand-length approximation
is removed. No loose pieces, new joints, source UV changes or nonfinger
changes are introduced by the field. Session 16230 is building and will
capture six closeups plus actual exported meshes. This new volume field
requires fresh current tests and actual render inspection; it remains
unapproved and undeployed. Audio stays muted.

Session 68398 compiled and ran the squared-wave broad-pad regression,
then failed lateral pad curvature: first human-male pad apex .45516825,
shoulders .43379784 and .4489404, rise .0138 versus required .015. The
other tests in that pipeline were not run. Direct execution of the repaired
outdoor UV fixture from this freshly compiled binary passed.

The current height profile is wave^1.5, between the former linear and
squared profiles. It retains broader shoulders without fully flattening
the arch; the .015 curvature and .035 separation requirements are unchanged.
The circular depth section remains. Session 16230 started before this
height-profile correction and renders the intermediate squared-height
volume candidate only. Current-source build/render/tests are still needed.

## Sequential volume deformation

Session 16230 built successfully and all six intermediate squared-height
volume closeups were individually inspected. The hand remains thin and
claw-like, so this shape is unapproved. Analysis of actual exported geometry
and production pose matrices finds 121 opposed shading faces among 19,118
finger triangles per player, versus 76 in the preceding broad-pad variant;
bind-pose counts are 115 versus 69. These normal comparisons alone do not
prove self-intersection. Independently, the old coupled tip/depth mapping
can have a negative local Jacobian in a deep valley: both contractions use
the pre-shaped longitudinal coordinate. That is a real fold mechanism.

The current field computes volume activation from the already reshaped
longitudinal position. Tip shaping and positive depth scaling now apply
in sequence, keeping the local maps monotone for the supplied hand extent.
The current height profile remains wave^1.5; no regression thresholds were
weakened. Source UVs, skin palette, original wrist/palm base and body geometry
remain outside this field. Actual render and geometry checks must still
verify the result.

Session 98818 builds and captures the preceding pre-staging wave^1.5
variant, then invokes Cargo regressions. Its following tests start after
the sequential correction was written and should compile the current
source; the separate regression-source-note records expected test hashes.
Its original launch hashes describe the native pre-staging build only.
Session 20883 builds and captures the current sequential variant. Keep
source frozen until these proofs complete, inspect each current image and
compare actual GPU arrays before any approval. No browser deployment or
complete-character approval has occurred. All runs remain muted.

## Current sequential-map evidence and mounted fit

Session 20883 built the sequential-volume deformation; all six player
closeups were individually inspected. The connected hand remains too
thin and claw-like, so no hand or complete model is approved. The actual
GPU arrays preserve all indices/UVs/joints/weights and nonfinger positions
and normals. Bind opposed-shading faces fall to 19; posed counts are 22
using the installed Bevy 0.20 inverse-transpose skin-normal transformation.
There are no near-zero-area finger triangles. This is local geometric
progress, not proof of perfect topology or final art acceptance.

Session 98818 freshly compiled the current sequential source and passed
the four-rig geometry/attributes/depth/separation/animation, resting-pose
and glove tests. Its mounted full-cycle test failed at Chris phase0 side0,
actual radial gap -.0020952374 (minimum allowed -.0015). The original
combined-launch hashes describe the earlier native build; the result now
explicitly records separately confirmed current test-source hashes.

Session 48743 exported both current mounted player meshes and production
pose matrices. Independent triangle clipping reproduces Chris's gap
-.002095232 and finds Kris -.002145677. Closest contact points are above
and forward of the bar center. Translating each wrist +.0015 Y,+.0015 Z
predicts left near-contact (+.000025 Chris, -.000025 Kris) and right gaps
about +.000620. This prediction is not a production-cycle pass.

The production wrist target is now (.215,.7365,.1965), mirrored in X.
The reviewer also accepts `CRYSTAL_PEOPLE_REVIEW_DETAIL=hands` with a
single image-target view, focusing on standing hands or the mounted bar.
Production cameras are unchanged. Session 10616 builds and captures six
mounted player hand details and six idle player/NPC details, then runs a
fresh production full-cycle mounted test and the four-rig/resting/glove/
joint checks. These current captures and checks remain pending.
Every build/test/capture is muted; no browser deployment occurred.

The exact installed-shader normal analysis confirms the old squared-volume
variant has 115 bind and 121 posed opposed-shading faces, versus 19 and 22
in the current sequential variant. Protected GPU arrays remain exact.
These comparisons are indirect topology evidence and do not approve art.

Session 10616 failed before captures or tests on a shared scenery change:
its textured-only Model adapter omitted the newly required solid_materials
field. The initializer now explicitly uses an empty vector, consistent
with this adapter containing a textured surface only. No outdoor shape,
water, shader or dispatch changes were made here. Session 14788 retries
the same current muted native captures and production regression batch.
The hand, rider and reviewer sources stay frozen until that proof finishes.
