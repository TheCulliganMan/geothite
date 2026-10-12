# Nurse arm contour review

The initial Nurse-only candidate refines the sleeve and forearm together in two
joined passes and applies the existing exposed-arm shoulder/elbow weight field.
Five muted native captures completed and all five were individually inspected:
Idle and Run .125/.625 in four angles, plus larger front and side renders.
The elbow silhouette is rounder, but the shoulder/underarm opening becomes too
square. This candidate is rejected visually. The fresh outward-shading regression ran one test and failed at Nurse Run
15/32, with a corner-normal agreement of -0.03778. The four preceding roles
completed their checks. The next candidate preserves original shoulder weights
and applies only the elbow blend. Its fresh muted build, matching five captures
and the same unchanged animation criterion are pending in session 59793.

External evidence is in `pending-model-review/captures/nurse-rounded-arms-*`
and `nurse-rounded-arms-diagnostic`, under the character review artifacts root.
No browser deployment was performed and no complete character is approved.

A separate actual costume export diagnostic found 80 of 400 waist faces with
at least one opposed shading normal, minimum agreement -0.0995. This is a
possible contributor to the dark waist seam, not proof of the entire cause.
The fitted garment normals need review after the arm candidate is settled.


The elbow-only native build and five capture jobs completed successfully. All
five images were individually inspected. The square underarm opening from the
first candidate is removed, and the elbow/forearm contour is rounder. This is
still a local candidate pending the animation regression. Two attempts failed
before test execution on concurrently edited cave-mesh code: missing Vec2/Vec4
imports, then a slice `.map` in its test. The imports were fixed locally; the
slice expression was already corrected in the shared worktree when inspected.
A fresh test retry is live in session 79024. No stale binary result is credited.

Matching actual GPU vertices by complete position/normal/UV/joint/weight tuples
shows all 11,942 referenced vertices outside the arm band preserved. Raw prefix
index comparison is misleading because added arm vertices shift later UV copies.
Normals are finite and unit length; skin weights remain nonnegative with maximum
sum error 7.45e-8. The largest original arm displacement is 0.01151 rig units.
These checks do not establish full animated surface quality.

An offline prediction for the separate garment issue rebuilds normals from the
fitted side faces and gives flat top/hem rims separate normal islands. It leaves
geometry unchanged and predicts zero opposed faces across the 4,000-face Nurse
costume (minimum agreement .8176), compared with 160 in the current export.
This normal correction has not been implemented or rendered yet.


The final fresh retry (79024) executed one test and failed at Nurse Idle 0/32,
normal agreement -0.9502461 on the shoulder triangle near x=.315–.434. The
elbow-only appearance improvement does not resolve the existing shoulder
weight transition. The complete arm remains unapproved; the next arm work
must address that root transition without recreating the square underarm.

The Nurse fitted garment normal correction is now implemented separately:
side normals are rebuilt from the fitted faces with geometric welding across
the circular seam; flat waist/hem faces receive separate normal islands.
Positions, colors, UVs and skin weights are copied unchanged when making
those islands. Fresh muted native capture verification is pending (41344).


Actual exported Idle palettes independently reproduce the severe shoulder fold:
the pre-arm body has 14 opposed faces among 198 selected upper-arm/shoulder
faces, minimum agreement -0.9502463. The elbow-only body has 26 among 2,154,
with the identical minimum. Counts differ with subdivision and do not prove
quality by themselves. The unchanged severe defect is in the shoulder root,
so the next arm candidate needs a continuous shoulder weight field as well as
an acceptable underarm silhouette. No arm acceptance follows from these data.


The fitted-normal native build completed. Four capture jobs exited successfully;
the final side image was saved and inspected, but its process stalled after the
window closed. A one-second process sample showed the main thread waiting on a
condition variable. Only the verified task-owned process was stopped with
SIGTERM; the fifth job is recorded as -15, not a pass. All five images were
individually reviewed. Actual cloth per-face positions, UVs, joint IDs and
weights are exact versus the elbow-only baseline; body and pose exports are
exact. All 4,000 cloth faces now have outward shading agreement (minimum
.8175876), and normals are finite and unit length. This corrects shading data,
but the resulting bright waist band remains visually unfinished.

The next Nurse shoulder candidate shortens its transition from x=.25–.75 to
x=.25–.65, ending at the elbow field start. Other exposed-arm roles retain the
original .50 span. The prior Idle mesh prediction for this .40 span found no
opposed faces in its selected upper-arm/shoulder region (minimum .1469); that
is an offline prediction, not proof for the new native candidate or all motion.
Fresh muted build, five matching captures and the unchanged arm shading
regression are running in session 97713.


The front waist section independently explains the bright band: current cloth
radius rises from .27094 at y=1.62 to .33438 at y=1.47938, while the underlying
body radius barely changes (.25294 to .25717). The fit currently fades out over
only .16 rig units, creating a short outward shoulder on the skirt. A proposed
Nurse-only fade ending at y=1.22 spreads that flare over .40 units; a front-ray
prediction is saved externally. It is not implemented or accepted yet. Existing
waist attachment padding must stay intact, and the full surface and movement
need fresh checks after changing the contour.
