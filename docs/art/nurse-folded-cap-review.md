# Nurse folded cap candidate

Current full-body Idle, Run .125 and Run .625 captures were individually
inspected for Nurse, Cal and Youngster: nine images, thirty-six views. They
use the retained terminal-domain hand geometry. Nurse's old white cap reads
as an oval pill resting on the hair, with a small raised red blob for its
cross. Her elbow contours, buns, jaw/nape, skirt, visible white shorts and
shoes also remain unfinished. Cal retains angular limbs, neck and footwear;
Youngster retains hair/nape, shorts/knee and shoe defects. No whole model is
approved.

The new Nurse-only cap uses a closed folded cloth crown with rounded corners,
a fitted lower edge and a flared front. Three non-overlapping red panels
follow the sloping front to form the cross. The former Nurse oval and red
ellipsoid implementation has been removed. Other characters' cap shapes
are unchanged by this source edit.

Fresh muted native compilation and all seven capture jobs completed successfully.
All thirteen candidate images were individually inspected: thirty-six quartet
views plus four larger Nurse angles. The ordinary cap geometry regression ran
one test and passed, including Nurse among its checks for finite unit normals,
valid non-degenerate faces and outward shading. The folded crown reads better
from front, side and rear, but the rest of Nurse remains unfinished.

Actual GPU body exports for Nurse, Cal and Youngster match the prior baseline
exactly in positions, normals, UVs, indices, joint IDs and weights. Their pose
exports and the two exported costume meshes also match exactly. This proves
body protection for this cap edit; the body export excludes head accessories.
The waist seam, faceted elbows, stiff skirt and thick shoes remain visible.
A separate Nurse-only UV correction removes the white trouser painting from
the exposed calves. Fresh muted build and all five capture jobs passed; all
five images were inspected (four angles in Idle and two running poses, plus
larger front and side views). Actual GPU per-face positions, normals, joints
and weights match the cap candidate exactly. Only exposed lower-body atlas
faces change UV; no changed corner is above y=.78501. The costume and pose
exports are exact. Skin samples are checked against the current painting.
This removes the visible white band, but retains the unfinished body contours,
waist and garment movement. No complete character is approved.

All captures and diagnostic exports remain external in
`~/.codex/artifacts/anime-character-review/pending-model-review/`.
The baseline is `captures/current-body-trio-{idle,run125,run625}/`;
the candidate is `captures/nurse-folded-cap-{idle,run125,run625,single0,single1,single2,single3}/`.
This candidate is not deployed in the served browser build.
