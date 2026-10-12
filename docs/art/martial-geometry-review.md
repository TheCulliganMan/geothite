# Martial character geometry review

All captures are silent native renders of the production `PeopleGpu` and
`Person::update` path, including current head and hand proportions. Four
views per image are inspected individually; capture existence is not approval.

## Black Belt wrap and belt

The corrected baseline in `full-roster-fixed-idle/black_belt.png` reads as a
navy shirt with an X rather than a wrapped gi. The inner collar now terminates
at the outer wrap, above the old crossing. Two tapered belt ends follow the
sampled torso skin influences beneath the knot.

Fresh Idle and Run phases 0.125 and 0.625 were individually inspected:

- `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/martial-geometry-idle/black_belt.png`
- `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/black-belt-wrap-run125/black_belt.png`
- `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/black-belt-wrap-run625/black_belt.png`

The long crossing has gone and the two belt ends are visible beneath the knot
in front views. All three captures completed normally. The collar still looks
like strips on a shirt, belt ends are too subtle, and the straight shirt hem,
generic sneakers, hands, angular jaw and nape remain unfinished. The whole
model is not approved. The existing full uniform animation-fit regression passed (one actual test,
exit 0), checking fitted distance, face winding, valid skin influences and
16 Run samples across the supported uniforms. No browser deployment of this
geometry has occurred.

## Bruno exposed torso candidate

The corrected baseline retains the shirt silhouette on bare skin. The source
candidate adds selective refinement, paired chest contours and a narrower
waist with continuous fades at arm, hip and neck boundaries. UV islands and
skin influence arrays are interpolated by the existing refinement function;
the original bare-skin remap still runs afterward.

The first `martial-geometry-idle/bruno.png` was pixel-identical to the baseline
under read-only RGB comparison: that reviewer executable does not contain
the latest torso edit. It is not evidence about the candidate. A subsequent
reviewer build is queued behind the current fit test; fresh candidate Idle,
two Run poses and bare-body regression remain pending. Chuck is unchanged;
his broader anatomy requires a separate profile.

The freshly linked native test executable (mtime later than the torso edit) passed `bare_body::tests::supplied_bare_arms_and_torsos_use_skin_without_touching_faces_or_trousers` (one test, exit 0). This proves skin sampling, normalized influences and unchanged source face/trouser boundaries for the four bare-body roles. It does not prove the new torso looks good or that its full animation surface has no intersections.

Follow-up source inspection found duplicated shirt/trouser boundary positions with different UV selection at the waist. The torso displacement now pins any position shared with an unselected island. The bare-body regression additionally checks all original duplicate positions remain joined within 0.00002 units for all four bare-body roles. The revised test is queued; the earlier pass predates this seam guard and is not a result for the revised candidate.

The first actual torso candidate was rendered with reviewer SHA256 `9ff1eeb5d6ad68865a5ebf869a42367c35f4b807833cd9affd29bc864cc67ce4`. All four views in each of `bruno-torso-idle`, `bruno-torso-run125` and `bruno-torso-run625` were individually inspected. Waist/chest shape changed, but small dark gaps at the shoulder boundary and overly round pectoral contours make this candidate unacceptable. It has not been deployed. The next source revision broadens/flattens the chest and uses boundary-joined subdivision: adjacent unselected triangles split on each shared refined edge, retaining separate UV islands with matching interpolated skin influences. The original refinement path for other models remains unchanged. A fixture regression checks interior edges stay closed and duplicate UV copies coincide under sixteen different joint bends; the supplied hand contracts also run to guard the unchanged path. Revised captures and test results are pending.

The added original-vertex UV seam guard passed in the supplied bare-body regression (one actual test, exit 0, 0.26 seconds). That executable did not yet list the new joined-refinement fixture, so this pass proves the waist pinning revision only, not the latest boundary triangulation. Current revised reviewer and joined-refinement/supplied-hands builds remain live. Newly created UV boundary midpoint copies are also pinned during torso displacement, not just original source vertices.

Latest source contracts passed via the freshly linked native test executable (mtime after both `bare_body.rs` and `hand_surface.rs` edits): joined-refinement fixture and supplied hand contracts, two actual tests, exit 0; supplied bare-body skin/UV/original-seam/boundary contract, one actual test, exit 0. The queued Cargo hand test is still live; these direct executable results establish the revised code contracts without implying visual approval. Latest candidate reviewer build and four-angle animation captures remain pending.

The revised reviewer and queued Cargo hand-test builds both terminated with exit 101 on an unrelated concurrent `dungeon_models::Model` field addition: `gym_scenery_occlusion.rs` omitted `gym_stone_pixels`. The importer now remaps that optional per-vertex metadata alongside surviving geometry rather than discarding it. A replacement reviewer build is live after this source correction. The direct character-test results above remain valid for the character source revision; the failed Cargo commands are not reported as passes.

The corrected reviewer build completed with exit 0 in 6m42s (including shared build queue). Reviewer SHA256 `b339b0c606b103c0b02700e5896bf4680feb1a7bb0bace9b5e1c9576f571b36e` produced `bruno-joined-torso-idle`, `bruno-joined-torso-run125`, and `bruno-joined-torso-run625`, all exit 0. All twelve views were individually inspected: dotted shoulder boundary cracks are gone and chest contours are broader/flatter; the remaining angular armpits, weak back/shoulder anatomy, generic hands/shoes, absent martial belt/wrist wraps, jaw/nape and overall silhouette remain unfinished. This is a verified incremental improvement, not whole-model approval.

The browser build completed with exit 0 in 2m48s. Bindgen completed and the local served engine and glue were checked against the generated bytes: engine SHA256 `e9392496e7ebd02ffebd424a7519a190e360fea0ed351a1a74c83432fcbf265d`, glue `e1f12d5d8b612b9beac9cbd94db27c6177a9ba69105908e0b43353a0eff82c92`. The department-store page loaded past its loading screen. Its sound control reported `aria-pressed=true`, `data-audible=false`. This is a loading/mute smoke check, not full character-context or gameplay validation. Paintings/bundle are unchanged.

## Chuck and male swimmer torso profiles

The individually inspected corrected Chuck Idle capture still has a straight,
shirt-like bare chest and abdomen. A source revision applies the joined torso
refinement to Chuck and the male swimmer with distinct silhouettes: fuller
chest/abdomen and back for Chuck, a narrower waist and subtle chest/back shape
for the swimmer. Bruno's displacement parameters are unchanged. Existing
skin, influence, original UV-seam and protected face/trouser checks now also
require Chuck's abdomen to gain width/volume and the swimmer's waist to taper.
The freshly linked test executable contains both new silhouette assertions
(checked in its binary strings) and passed the supplied bare-body test: one
actual test, exit 0, 0.19 seconds. This covers the new silhouette direction,
skin sampling, valid influences, protected face/trouser attributes and original
duplicated UV-seam positions. Fresh renders and visual acceptance remain pending.

Reviewer build completed with exit 0 in 23m30s including the shared queue;
SHA256 `ecab0ee247affb69673bf8947e6f4ed4c1dce633fc66f667f080d4397c21c5dc`.
Fresh `torso-gutter-idle`, `torso-gutter-run125`, `torso-gutter-run625`
captures of Chuck and Swimmer Guy completed normally. All 24 views were
individually inspected: Chuck gains a fuller front abdomen/chest and the
swimmer a tapered waist. Small dark shoulder-boundary gaps remain, so neither
candidate is deployed. The arm refinement now uses boundary-joined splitting
as well as the torso. Revised renderer is building; tests/captures for that
arm revision remain required. Belt/costume, beard, jaw/nape, hands and feet
still prevent whole-model acceptance.

The queued Cargo bare-body test failed on four stale test uses of
`Form::Room` after a concurrent roof-ownership bool was added. The two room
fixture constructors now pass `false`, and two pattern assertions ignore
that extra field. This preserves ordinary room fixture intent; it is not a
character test pass. The existing direct bare-body pass above predates the
latest arm refinement.

The arm revision additionally has a production-rig regression checking all
coincident upper-body UV copies stay joined under sixteen Run samples for
Bruno, Chuck and both swimmers. That new regression is queued, not yet a pass.
WASM build completed with exit 0 in 2m10s and bindgen completed normally.
The paired output is staged externally in `torso-gutter-wasm-staging`, engine
SHA256 `db730af79594ec6000c8ef8662e47dbb012ff8212ca2124e9ad71e08659778f8`,
glue `827b53b37566a7506ba8efc3d6e95daa011c40a734814d1610b1059a5556976e`.
It is held pending arm/shoulder visual review; the browser still serves the
previous engine. Native reviewer and new bare-body Cargo regression are live.

The latest linked native test executable lists the new production Run-seam
regression. Running `bare_body::tests --include-ignored` directly passed three
actual tests (exit 0, 0.36 seconds): sixteen-sample seam continuity for four
exposed-body roles, female swimsuit fabric/skin continuity, and skin/original
UV-boundary plus Chuck/swimmer silhouette contracts. This includes the joined
arm revision. The native reviewer build remains live; shoulder visual acceptance
and browser engine deployment remain pending.

## Latest shoulder diagnosis

The joined-arm reviewer finished successfully in 22m13s including the shared
build queue; SHA256 `bc966f29488371e068564c548ea83a94deea5fb732c3fbade6e804fabaa9b0a8`.
All 48 views of Bruno, Chuck and both swimmers in `joined-arms-idle`,
`joined-arms-run125`, and `joined-arms-run625` were individually inspected.
Small dark shoulder/upper-arm marks remain. The three passing bare-body tests
prove their stated seam and UV contracts, not removal of those visible marks.

Bruno and Chuck were additionally rendered unlit in Idle and Run 0.625.
All 16 views in `shoulder-diagnostic-unlit-idle` and
`shoulder-diagnostic-unlit-run625` were individually inspected: the dark
shoulder marks disappear, while each surface remains visibly continuous.
This isolates the marks to lighting-dependent behavior; it does not establish
whether normals, face orientation or another lighting input causes it.
The production body already has `NotShadowReceiver`, including in these
review captures. Original male GLB shoulder triangles do not oppose their
source vertex normals in the inspected height interval, but that check is
not evidence about the refined production surface.

The next diagnostic exports the final production body positions, normals,
UVs, indices and skin influences through the opt-in native-only
`CRYSTAL_PEOPLE_REVIEW_EXPORT_MESHES` directory. These are temporary external
review files, never shipped assets. Its native reviewer build is queued.
No global unlit material change or browser engine installation was made.
The staged engine remains held; generic anatomy, costume, hands and feet
also remain unfinished. All diagnostic previews are muted.

The initial exporter reviewer rebuilt successfully in 56.44s after concurrent
dungeon references were corrected elsewhere. Bruno/Chuck final-mesh captures
exited zero and emitted external diagnostic meshes. In each mesh, 1,868
nondegenerate upper-body triangles wholly within y=2.0–2.58 and |x|<1.2
had no corner normal opposing their rest-pose face normal. Coincident
upper-body UV copies also had no strong normal discontinuity (dot < 0.8).
An initial wider-height-only query included refined fingers; those results
are excluded from the shoulder diagnosis. This narrows the investigation to
animated deformation/lighting, without proving the actual animated normals
are correct. A reviewer-only export of the production skin palette after
`Person::update` is now added for that comparison; its fresh build is pending.

The pose exporter compiled and all twelve images / 48 views in
`hat-gutter-pose-{idle,run125,run625}` were individually inspected; captures
exited zero. The body revision is unchanged in these images and shoulder marks
remain. The production palette comparison uses the same inverse-transpose
of the blended skin matrix as Bevy 0.20's `skinning.wesl`. Rest-pose agreement
does not persist after animation: Bruno has 66/65/66 opposing upper-body
triangles in Idle/Run0.125/Run0.625; Chuck has 70/73/71. The worst shoulder
corners jump from UpperChest 0.4 + Shoulder 0.4 + Arm 0.2 to Arm 1.0 over
a short edge. The original unrefined male mesh also has 14 opposing faces
in the same production Idle palette, so subdivision alone cannot solve this.

A read-only weighting experiment smooths the shoulder bend across x=0.25–0.75
and fades back to existing weights below y=2.17 and toward the inner torso.
Both roles have zero opposing shoulder faces in the three sampled poses;
the wider upper-body query still detects 8/5 elbow-region faces in the two
Run samples. This is diagnostic geometry evidence, not a rendered fix.
The production candidate applies that field only to the four exposed-body
roles, deriving chest/shoulder/arm indices from each supplied hand ancestry.
It preserves position/UV/normal data and matching weights at UV copies.
The new regression checks shoulder shading against deformed face orientation
at 32 samples of both Idle and Run for all four roles. Fresh regression and
mapped captures of that candidate are pending; no browser deployment yet.

## Exposed arm motion follow-up

The shoulder-only revision passed four supplied bare-body tests (direct native
executable, exit 0, 0.34 seconds). All twelve images / 48 views in
`captures/shoulder-weights-{idle,run125,run625}` were individually inspected.
The shoulder bend is fuller, but small elbow creases remain. The full upper-body
palette comparison detects 8 opposing triangles at Run 0.125 and 5 at Run 0.625
for both Bruno and Chuck. Skin mapping and joined UV copies alone do not prove
the deformed surface is sound.

The first elbow extension used a smooth influence field from x=0.60 to 1.10.
The widened regression checks the arm through abs(x)<1.20 at 32 samples of both
Idle and Run for all four exposed-body roles. It FAILED at Bruno Run 0/32;
the other three bare-body tests passed (exit 101 overall, 0.45 seconds).
The fresh reviewer snapshot SHA256
`c40618711f29680be55894403f3e8ed6c568b6925dbe919e89e3e9421006d437`
exported the actual production mesh and skin palette in `meshes-elbow-run0`.
That diagnostic finds one remaining opposing upper-body triangle for Bruno
and Chuck, minimum normal agreement -0.0768. Bruno and female swimmer Run 0
captures were individually inspected; the four-role batch completed with exit
0. Chuck and male swimmer were subsequently individually inspected too,
completing all sixteen views in that four-role Run 0 batch. The remaining
elbow creases, generic hands and slab-like bare feet are visible.

A read-only broader x=0.55–1.15 influence experiment eliminates the opposing
triangles in the three available Run samples for both male roles. Minimum
agreement at Run 0 is only 0.0078, so full-cycle validation is necessary.
Production source now applies that wider field to existing Arm/ForeArm-only
vertices, retaining the same selected region and protecting wrists/fingers.
Its compiled full-cycle result and mapped appearance remain pending.
A later library snapshot emitted after the source edit still exported identical
weights to the earlier narrow revision; its newer file timestamp is not proof
of the new code. Do not approve or deploy it as the wider revision.

The swimmer review still shows squared/jagged swimsuit armholes and slab-like
bare feet with dark heel surfaces. Torso anatomy, role-specific clothing,
hands, feet and head silhouettes remain unfinished; no whole model is approved.

## Swimmer bare-foot volume candidate

The production exports show the bare sole spanning approximately z=-0.255 to
0.241 at an almost constant floor height. The upper foot retains nearly
parallel shoe-like side walls. The source candidate narrows the heel more
strongly than the forefoot and raises a shallow medial midsole arch, fading
to zero at heel/toe contact patches and above the lower sole. The ankle ring
remains outside both displacement fields. Skin UV sampling and welded normal
recomputation continue through the existing bare-foot path.

The supplied two-swimmer regression now requires raised medial arch points,
unchanged heel/toe floor contacts, pinned ankles, valid skin mapping, normalized
influences and normals, and unchanged other body islands. Its Cargo run is live
but has not produced a result. An external read-only estimate using the prior
production export finds six arch floor points per side above 0.008 units,
maximum lift approximately 0.0181; this estimate is not the authoritative test
or a rendered appearance result. Fresh mapped captures are still required.

The previously queued bare-body test and native reviewer build terminated with
exit 101 on a missing space in the new foot-test negative comparison (`< -0.15`).
The worktree comparison is corrected; replacement bare-body and reviewer runs
are queued. The separate footwear test run remains live. These failed builds
provide no passing geometry result. Rustfmt syntax parsing alone did not catch
that compiler token error and is not reported as compilation success.

The fresh native test executable lists both the expanded arm diagnostics and
medial-arch assertions. Direct supplied bare-body checks passed: four actual
tests, exit 0, 0.42 seconds. This includes 32 samples each of Idle and Run for
all four exposed-body roles, joined UV copies, skin sampling and protected
source attributes. Direct footwear checks passed: two actual tests, exit 0,
0.20 seconds. The queued footwear Cargo command also passed after 13m25s
including the shared queue: two actual tests, 0.20 seconds.

Reviewer SHA256
`652df987df47517099ec7011b62263a7b1bc2d6da51de982c49b908fc1ebaa2c`
exports 848 changed Bruno weight entries compared with the narrow elbow revision;
the example at x=0.8174 matches the wider source field. Actual production Run 0
mesh/palette comparison has zero opposing upper-body triangles for both male
roles, minimum agreement 0.0078. This establishes the correct compiled revision,
unlike the earlier newer-timestamp but unchanged snapshots.

All twelve images / 48 views in `captures/arm-foot-{idle,run125,run625}` were
individually inspected; the three batches exited zero. Arm bends are fuller and
the large shoulder collapse is reduced. Anatomical creases remain visible and
the underarm silhouette still needs work. Narrower heels and shallow arches do
not yet produce convincing bare feet: toes and the low flat forefoot still read
as shoe-like slabs. Female swimsuit armholes remain jagged; torso, hands,
footwear and role-specific clothing are unfinished. No whole model is approved.
The browser build passed in 2m34s; bindgen/staging follow-up is pending here.

Bindgen completed with exit 0. The rebuilt engine and glue are now served by
the local browser preview, with three cache references updated and startup
mute preserved. HTTP bytes equal the generated files: engine SHA256
`5ebf4d8b0c977bbe94b4280e79cb6b5509ee0ef227652b4b161b9d4b35087d2e`,
glue `74ae011f6ed22ce17498442f6a5fb21a7e682f2aa1043750b45d2f5ea9a551d3`.
This deploys the incremental current geometry for further context review,
including previously verified cap/gutter changes. It does not approve whole
models. Owned linker, candidate artwork and obsolete staging directories were
removed after verification; mapped evidence and source paintings are retained.

A fresh background department-store browser tab loaded past the loading screen.
The sound control reports `aria-pressed=true` and `data-audible=false`.
The screenshot visibly shows Gentleman's tall top hat in the room. Characters
are small at the current camera distance and the strongly saturated floor makes
this insufficient for facial or overall art approval. This is a loading/mute
smoke check only, not gameplay or whole-roster verification.

Queued Cargo verification completed: the four bare-body regressions passed
(exit 0, 0.30s actual tests; 23m12 total queue/build time). The native reviewer
build also completed with exit 0 after 24m18. These supersede the earlier
pending entries; the later short-skirt reviewer build is a separate job.
