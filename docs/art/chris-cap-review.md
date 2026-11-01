# Chris cap candidate

The mapped Chris review shows a shallow oval crown and separate oval bill.
The replacement uses a scalp-fitted crown with a rounded profile, a curved
bill attached at forehead height, a narrow darker lower band and a small
cloth-covered top button. The bill has a darker underside. This branch applies
only to the trainer hat; other uniform caps retain their existing geometry
and colors. No generated painting or replacement character rig is involved.

Native build and actual standing/mounted render inspection are pending in the
external pending-model-review directory. The candidate is unaccepted and
undeployed. This does not resolve angular hair, clothing, body proportions or
other remaining player/NPC work.

Initial native build 40945 passed in 7m37s; reviewer SHA256
bc169a80d113663d049138c038f0e1d8d483d084f02d22ee2ede0f0f419ae29a.
Muted captures 62853 (standing) and 75895 (mounted phase .25) both exited zero.
Both actual images were individually inspected in four angles, eight views
total. The crown and band are clearer, but the bill remains too thin in side
view. The initial cap is not accepted as finished.

A source revision increases only the trainer bill's thickness from .028 to .065
and downward bend from .045 to .09, in head half-height units. Other uniform
hats retain their original parameters. Fresh build 75616 is active in
player-cap-bill-revision-build.log; this revision has not been rendered yet.

Bill revision build 75616 passed in 1m42s; reviewer SHA256
04b5e4c0cf71269b49b4219f2a4027e5863d4a16d27a0b73f0ef96318cec84b2.
Captures 91098 (standing) and 2233 (mounted phase .25) both exited zero.
Both images were individually inspected in four angles each. The thicker
bill shows a clearer edge and downward curve, without covering the eyes.
This local silhouette improvement is retained; the whole cap/player remains
unfinished, including hair, seams and clothing.

The offline reviewer now supports CRYSTAL_PEOPLE_REVIEW_SINGLE_VIEW=0..3,
requiring CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET. It centers one selected actor in
a 1000x1000 target with the ordinary camera height and a square projection.
Production pose, mesh and materials stay identical. This enables larger
inspection of face mapping, hair edges and seams. Fresh build 39269 is pending.
