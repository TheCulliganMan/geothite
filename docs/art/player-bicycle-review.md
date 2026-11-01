# Mounted player review — 2026-10-12

The offline people reviewer now accepts `CRYSTAL_PEOPLE_REVIEW_BICYCLE=1`
with the explicit `chris,kris` filter. It sets the actual production Person
bicycle flag and drives Person::update, including bicycle::pose. Each rider
is shown together with the actual production bicycle::mesh. The prop's child
scale compensates the person root scale so the bicycle retains its separately
scaled runtime dimensions and does not inherit Kris's body width.

Native build 58638 passed in 4m44s. Reviewer SHA256:
`28f19f3f9bfe134ec2d7ffd6be99b1cd9f1b733ce4339a1eea92c35c63b06a67`.
Capture jobs 60705, 98372, 26857 and 6535 passed. All eight images were
individually inspected in four angles: 32 views across normalized cycle
phases 0, 0.25, 0.5 and 0.75. Every job used `CRYSTAL_PREVIEW_MUTE=1`, normal
production proportions, and the existing mounted pose. Artifacts remain in:
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/player-bicycle-{0000,0250,0500,0750}/`.

Neither rider passes. The handlebar grips are far above the open hands in
every phase; the frontal bar visually crosses the face. In side views the
hips sit forward/below the saddle and the shoes do not maintain convincing
pedal contact. The small alternating leg flex does not form a convincing
pedal cycle. Generic open hands, shoes, flat caps, rigid hair, angular jaw/nape
and plain clothing silhouettes remain. Chris's red bike and Kris's teal bike
are identifiable; wheel/frame readability does not validate rider fit.

At this baseline, production source confirmed that the bicycle pose applies fixed thigh,
knee and arm rotations plus a small sinusoidal leg change. It does not solve
hands to grips, pelvis to saddle or feet to circular pedal targets. The next
repair must establish these contacts on both supplied rigs and verify them
across the cycle, then repeat the mapped visual inspection. Baseline bike-local
anchors included saddle y=0.665,z=-0.12 and grips y=0.865,z=0.35. They must be
converted to each rider's mesh coordinates using its production height scale.

This is an offline model audit, not a controller/gameplay check or a live
browser observation. No bicycle pose change is accepted or deployed here.
The currently served engine contains the verified Bug Catcher/Rocker update.

## Proportional contact candidate (not deployed)

Native build 57713 passed (1m15s), reviewer SHA256
`fddc00e1345a9e1c5a72c2adcbb7a8c956d9489ba542e2fa883ba3f72882ace0`.
The contact regression 56366 completed successfully: both supplied rigs,
32 phases, hips/wrist/ankle targets, finite normalized rotations and preserved
bone translations/scales. It checks joint anchors, not physical palm/sole contact.
The first prototype failed its wrist target by 0.1257 world units; measured
source limb lengths showed the old bicycle anchors were unreachable.

The revised saddle is y=0.50, grips y=0.720,z=0.25; pelvis target is
(0,0.540,-0.12), with a 0.50-radian forward chest lean and upright head.
Two-joint CCD fits limbs without stretching bones. All eight images in
`captures/player-bicycle-proportional-{0000,0250,0500,0750}/` were individually
inspected in four angles (32 views). Both players sit more convincingly, with
bent elbows/knees and the bar below the face. Open fingers and fixed cranks
still fail. Phase 0 and .75 capture jobs have confirmed successful exits;
phase .25's old handle is missing and phase .5's handle was lost when tool
output truncated. Their actual images were inspected, but their job exits
are not claimed here.

A subsequent source revision splits the cranks and level pedal platforms into
independent child entities. The production actor sync updates each bike after
its rider's motion advances; the offline reviewer uses the same articulated
parts and phase. Mounted fingers retain the authored closed Idle grip rather
than the open bind pose. This revision is building and has not been visually
accepted. Actual grip wrap, shoe-platform contact, collision, culling and
performance remain open. Neither player is approved as a finished model.

## Articulated pedals and sole correction (not deployed)

Build 31153 passed in 1m36s; reviewer SHA256
`2766cae384b66d5f392b7742b44e4ea22694e31156e8b243ae8092739bc15555`.
Capture jobs 83859, 36589, 9804 and 97199 passed. Both players' images were
individually inspected in four angles at all four phases (32 views). Cranks
now follow the feet and platforms stay level. Closed hands read better than
open palms, but their actual surface fit was still unproven.

Regression job 81477 passed: two actual tests verify finite/outward bicycle
surfaces and actual articulated crank/platform meshes, floor clearance, level
platforms, hierarchy and phase independence between two riders. The supplied
rider test was ignored in this command, so this result does not revalidate IK.

Muted diagnostic capture 7261 exported the actual production GPU body and
Person::update skin palettes. Vertical triangle intersections at each pedal
platform center found sole penetration of 0.0217–0.0228 world units for both
players across 32 phases. The ankle offset above each pedal was raised from
0.055 to 0.0775. Build 21962 passed in 52.62s; reviewer SHA256
`80c4273e1ecfbc951a92e5376a2a412c4aeefb5028bb5bf6736373cf18b6d928`.

Diagnostic capture 66485 passed. For all 64 contacts per player, the corrected
actual shoe triangles intersect a vertical line through the platform center.
Signed clearance above the platform top is -0.0001901..0.0005700 for Chris and
-0.0000995..0.0005589 for Kris. This establishes local sole support at the
platform centers; it does not prove every shoe/platform triangle is collision
free. Evidence and analysis scripts remain outside the repository in
`bicycle-sole-diagnostic/` and `inspect-bicycle-contact.py`.

The separate hand-surface diagnostic contradicts a finished grip: actual hand
vertices enter the grip cylinder by up to 0.0222 world units. Retaining the
closed source Idle fist is not a fitted grip pose. This needs a deliberate
finger/wrist pose with a space around the bar; do not shrink the bar or weaken
the acceptance check to hide the defect. The new production-shoe regression
(job 13003) passed: one actual test in 4.94s after a 4m24s build. It uses the
production GPU body and Person::update for both rigs over 32 phases, requiring
platform-center sole clearance within 0.0015. A direct invocation of that
freshly built test binary also passed all three bicycle regressions, including
the supplied-rig contact test (see bicycle-final-geometry-contact.log).

Corrected capture jobs 66485, 2745, 45998 and 54433 all passed. All eight
images in captures/player-bicycle-sole-clearance-{0000,0250,0500,0750} were
individually inspected in four angles (32 views). The supported soles and
seated/bent limb pose are improved. Grip intersections, detailed collision,
culling, initial spawn pose, rider/prop alignment during turns and performance
remain open. All owned jobs for this revision are terminal.
No whole player or bicycle revision is accepted or deployed.

## Fitted hand candidate — 2026-10-12 (not deployed)

A source-hierarchy search over the actual production GPU hand mesh showed
that the original finger curl ran sideways. The fitted pose sets wrist
world orientation to Rx(1.05), mirrored by Ry(pi) for the right hand, and
blends the source bind/Idle finger rotations by 0.55. Wrist targets are
(±0.215,0.735,0.195). Build 61400 passed in 1m35s, reviewer SHA256
`2c875b81f622d37177ddd33edadeda10a054e05b0a6ab0b67da26d95756491ee`.
All eight images from jobs 18169,60594,64761,21782 were individually inspected
in four angles over four phases (32 views). Fingers now curl over the bars,
with a more convincing wrist/palm orientation. A full hand-triangle check
showed positive rubber-grip clearance, but the right thumbs still penetrated
the inner metal bar by about 0.007 world units. This candidate was rejected.

Named optional thumb-base attachments are now read from the supplied GLTF.
Mounted riders rotate RightHandThumb1 about world X by -0.27 radians after
finger blending. Build 2531 passed in 1m44s. Muted capture 4923 passed, and
both four-angle images were individually inspected (eight views). The
corrected right thumb has about 0.00198 clearance from the inner bar. The
left thumbs remain about 0.00075 clear. All four hands clear the rubber grip
cylinder by 0.0000544–0.0002342. These measurements clip each actual hand
triangle to the finite axial sections before measuring radial clearance;
they cover 9726 triangles per hand with >80% hand skin influence. They also
show a 3.061–3.100-radian angular span around the grip. Vertex-only searches
were used to propose candidates, never to approve the physical fit.

The source Idle animation also rocks the ankles over time. The mounted pose
now restores the source bind foot orientation so shoes remain level through
that animation. Build 21645 passed in 1m51s; reviewer SHA256
`a205ecd31361342d703f0c7a98347d8135dfe50c912136ce68689a56da0db2a9`.
Muted diagnostic capture 58291 passed. Hand clearances remain as above. Across
32 phases at Idle time zero, actual platform-center sole gap remains within
-0.000191..0.000570 for both riders. Evidence is external in
`bicycle-level-grip-diagnostic/`, including grip-surface-results.json and
contact-results.json; scripts are inspect-bicycle-grip-surfaces.py and
inspect-bicycle-contact.py.

The production regression is now
`mounted_production_hands_and_shoes_fit_grips_and_pedals_through_full_cycle`.
It evaluates actual GPU hand/shoe triangles through 32 pedal phases and 32
source Idle times, checks finite grip/inner-bar clearance and curl span, and
retains the platform-center sole-contact gate. Job 27475 passed: one actual test in 6.24s after a 7m41s build. The freshly
compiled test binary also passed all three bicycle regressions in 0.04s, and
one full-roster routing regression in 32.60s (all 75 resolved character sources,
mounted aliases and three map contexts). See bicycle-hand-shoe-surface-test.log,
bicycle-fitted-final-regressions.log and bicycle-rig-roster-regression.log.

All eight final images in captures/player-bicycle-level-grip-{0000,0250,0500,0750}
were individually inspected in four angles (32 views). Jobs 58291,93812,85867
passed. Job 34494 saved both valid images but remained alive and inactive after
capture. Its own PID 6630 and exact command were verified before SIGTERM cleanup;
it exited 143 and was not restarted. The images are reviewed evidence, but that
job is not reported as passing. A sample showed its main thread waiting on a
condition variable; stripped symbols did not establish the exact waiting function.

The local mounted hand/shoe contact fit is accepted. Fingers curl around the
bar, both right thumbs clear the inner metal, and shoes remain level/supported.
This does not approve either whole player or the complete bicycle revision.
No deployment or finished model approval is claimed. Runtime turn alignment, initial spawn proportions,
whole-body/frame clearance, culling and performance remain to verify.


Runtime source audit confirms a remaining alignment defect: props use
facing_yaw(facing) directly, while Motion::advance bounds/interpolates the rider's
yaw. These must share the same presented root orientation during turns. The
initial spawn path also constructs the sampled Idle pose before mounted pose
and head/hand proportions are applied on a subsequent update. Both issues
need production-boundary regressions and repair before deployment. This is
source evidence, not a claim of an observed live browser frame.


The runtime alignment candidate now passes the mounted flag into PeopleGpu::spawn
so the deferred joints already contain the seated pose and final head/wrist
scales. Animation buffers remain unscaled. Each bike copies its own rider's
presented root position and rotation after motion advances, while retaining
its authored scale and independent crank/platform phase.

Native build 85095 passed. Capture 79080 passed; both Chris and Kris images in
captures/player-bicycle-runtime-alignment-0000 were individually inspected in
four views each. The fitted grips and level shoe support remain visually
consistent. Hair edges, simple cap shapes and clothing still need improvement.
The first regression build failed on an unresolved crystal_assets reference
in a separate cave test. A test-only workspace dependency repairs that build
reference; no production dependency or asset bundle was added. Fresh regression
results are pending. This candidate remains undeployed and neither player is
approved as a finished model.

Fresh test binary crystal_voxel_view-dc3731eb4a201fc3 passed the 77-case
first-spawn regression in 34.04s (job 45992, terminal PASS0) and the two-rider
24-frame runtime turn regression in 7.04s (job 13128, PASS0). The physical
regression failed its newly added culling-envelope gate (job 51701, exit 101).
This establishes a camera-edge visibility risk; it is not a hand/sole pass.

The culling repair now includes final production body joint boxes over 64
pedal phases and eight independent Idle samples in the shared player GPU
bounds. Both walking and mounted instances receive this envelope regardless
of which instance created the cache entry first. Joint-box unions contain
weighted vertices without evaluating every body vertex at every sampled pose.
Fresh physical verification is pending in bicycle-runtime-culling-regression.log.
The repair is source-only and remains undeployed.

The pre-culling-repair test binary also passed costume instance independence
for six character variants in 9.03s (job 85344, PASS0). This verifies cached
wrist selection and first-spawn proportion changes, not the later culling fix.

The fresh culling-repair binary passed the 77-case first-spawn regression in
35.93s (job 40498) and the physical hand/shoe/culling regression in 6.47s
(job 21189). Both exited zero. The latter checks all production body vertices
against the actual cache envelope across 32 pedal phases and source Idle times,
then checks grip clearance/curl and shoe support. This is local sampled culling
and contact evidence, not a complete character approval or deployment claim.
