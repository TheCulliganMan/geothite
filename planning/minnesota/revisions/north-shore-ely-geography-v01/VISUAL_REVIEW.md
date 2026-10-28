# Actual published coast-to-Ely visual review

Reviewed pack file SHA256:
`65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17`.
This review covers the specific west-entry coastal correction and inland winter
publication. It is not acceptance of every Minnesota map or earned progression.

## Retained evidence

- `renders/north-shore-ely-geography-v01/native-winter-connected-v01/`:
  actual connected source-art atlas, complete source-pixel original, individual
  complete map originals, graph receipt, composition and Surf-boundary audits.
- `renders/north-shore-ely-geography-v01/native-player-cameras-v01/`:
  all31 production 2D screenshots, exact source references, individual logs,
  checksums and command receipt. Uses `native-camera-plan-v02.json`.
- `renders/north-shore-ely-geography-v01/native-reached-cameras-v01/`:
  seven production screenshots loaded from actual controller exports: closed
  and open northern gate, inward nurse care, Surf, collected campsite, reached
  Gunflint, and a one-way ledge landing. The receipt binds each original Save
  SHA and command. No position, party, badges or flags were overridden.

All captures used explicit headless image targets. No visible game window was
opened. Full originals and early comparison versions remain saved. The source
reference cameras use a classic160×144 view; current fullscreen presentation
shows more terrain. The `classic-frustum` comparison sheets therefore include
the centered160×144 source-pixel portion of the production capture at matching
scale. The unmodified full640×576 captures remain beside them and were also
inspected. These crops do not replace the originals.

## Findings

The actual connected atlas retains the north–south North Shore silhouette.
Its northern walking trail bends east into the west side of Grand Marais.
The town's western water aperture joins the same broad Superior water body;
there is no fabricated connecting terrain in the atlas. Both coastal maps
remain visibly green. The northern town opening leads through Cedar's warm
eastern pines to snow inland, then through the65-wide connector and substantial
R12/R13 winter regions. Existing neighboring regional footprints remain fixed.

The six coastal decision views show a narrow winding trail, grass spanning
useful parts of the route, and readable wooded turns. The fenced overlook
actually shows Superior beneath its reachable deck in the production player
view. The road and shoreline do not require an invented south-quay opening.
The town service forecourt, west officers, northern officers, lighthouse and
western lookout agree with their accepted source terrain. Guards remain
visible in closed reached states and disappear in the qualified reached state.

The17 paired winter views show the intended snowy native pine silhouette,
snow floor, pale encounter habitat and deliberately striped sliding ice.
Lake water stays visibly water. Ice entry/stop views, ordinary wooded paths,
portages, shelter approach and Cut approaches retain readable boundaries.
The three Cedar transition views show the warm town-side stand, inland
snowline and snowy western arrival. The new native scene at the R12 eastern
port shows its actual snowy connector; near Ely it shows the real adjacent
green rest camp. Older isolated source crops clamp differently at these map
edges and are not evidence of the connected production camera's framing.

The deliberate green Cut/Headbutt pads are still visible inside the winter
terrain. They are a known inherited visual compromise, explicitly retained
to preserve the accepted native gameplay recipes. This pass does not claim
they were recolored. The original source-review epochs and rejected designs
remain unchanged.

The bog-side sign/trainer pocket east of the R12 ice course is reached from
its existing lake bank: Surf at whole-route `(38,78)`, then dismount onto
Floor `(37,78)`. The ice course no longer permits its former lateral dry
entry. This is an optional water approach in the late regional area, not a
claim that the whole old Foot-only tour remains possible. The main continuous
coast/Gunflint walking journey still passes independently. Production tests
must retain and visit that sign/trainer approach through actual Surf input.

Native day terrain and source art agree by sight at the reviewed positions;
animated water and autonomous NPC poses need not be identical between a
static source crop and the running renderer. Reached-save screenshots retain
their own saved time and facing (including night), rather than being restaged
to imitate the noon source fixtures. The nurse picture confirms the actual
reached room visually; HP restoration is proved separately by controller
assertions, not inferred from that screenshot.

No further terrain or art revision was needed from this native visual pass.
Gameplay test outcomes, canonical promotion and Rust source reproduction are
recorded separately in `VERIFICATION.md`.
