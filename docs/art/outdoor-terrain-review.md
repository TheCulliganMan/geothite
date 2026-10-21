## Current art direction — 2026-10-20

Make outdoor routes look amazing as 3D environments from every angle while
preserving the original game's feel. The original colors, time-of-day mood,
recognizable shapes, scale and route readability guide the art direction.
Translate those qualities into sculpted forms, crafted surfaces and lighting
that hold up through full camera orbits, low views and close views. The current
colors are approved; source heights are not a target. Mountains need tall,
continuous bodies whose interiors rise above their outer edges. A raised
perimeter around a sunken middle is unacceptable. The latest tall canyon
candidate is also rejected: reduce the overall relief while retaining rising
interiors and the approved colors. Avoid towering walls along ordinary routes.
Same-state 2D references remain useful for identity and readability, but cannot
establish 3D shape quality. Preserve gameplay and navigation throughout the work.

### Steep cap corner normals — 2026-10-21

Shared cap corners now fall back to their angle-weighted geometric normal when
neither steep adjoining face passes the crease filter. The former empty result
retained an authored upward normal, producing bright triangular marks on
sloping Route9 and Route46 shoulders. The regression covers both world scales
and preserves positions, indices, pigment and source UVs. An actual 49,534-vertex
Route9 window has identical positions and topology; 68 normals are corrected.

All 215 distinct selected checks pass, including the eight external-pack checks
listed below. Logs are `target/outdoor-mineral-normal-final-verified-tests.log`
and `target/outdoor-mineral-normal-final-native-build.log`. Eight final paired
views were personally inspected: two each on Routes9 and 46, plus Routes3, 26
and 44 and National Park. Their state receipts are preserved. The complete capture directory is
`target/outdoor-mineral-normal-final-review/`; seven additional pairs there were
captured but not personally inspected. Broad physical shoulders and some edge
teeth remain. The user accepted the current mountains as good enough and moved
the current work to National Park trees. Subsequent park work uses code and
geometry checks without render previews, as requested.

### Whole mountain bodies and entrance ground — 2026-10-21

The mountain surface now shapes the complete body. It no longer adds a crown
to a native two/four-tile shaft or spreads a distant source course through the
connected mass as a geometry target. The former regional fill and original
relief remain paint evidence, preserving the current pigment recipe. The
geological surface uses source-owned mineral beds, rising from low margins
into unequal interiors. The ordinary crown has a 3.2–4.8-cell upper envelope;
small footprints rise only as far as their width allows. Native pixel height
does not establish that envelope. Caves and raised water retain the additional
body needed for their real openings and banks.

The horizontal inset now spans approximately 0.28–0.36 source cells. The former
0.78–1.02-cell inset would leave a broad low apron under the complete-body
profile. Cap/wall copies still share one ordered horizontal field, and walking
plots retain their full source grid. Nearby proven standing ground extends
under the body with a finite linear grade, rather than disappearing one vertex
inside an exposed edge. Building roofs do not become that ground.

Wall-only columns use their actual lower and upper geometry extents and the
same horizontal fit as cap columns. The former native-height fallback retained
chipped horizontal ribs. Distinct native courses meeting at one source column
now keep separate wall-fit references. Using the lowest cap for both courses
collapsed the taller course's upper height rings into a triangular fan in the
unequal-course regression fixture. Cap
vertices themselves meet the complete geological surface, including where an
old upright rim stood above a lower cap. The former column fill still supplies
the same pigment calculation independently of these new fit references.
Fit references follow each face's source owner and actual upper geometry
extent. Neighboring courses can receive the same resolved terrain tier, so
that nominal height cannot identify their original wall extent.
Neighboring float-UV samples within a subpixel column share one representative
source point. The rooted bottom sixth retains its contact rounding; the upper
height rings follow one flank. Short banks outside proven permanent mineral
keep their original shape. Wall construction now encodes the analytic source
before chipping. Projecting a chipped diagonal face back onto its plane had
retained tangential drift from the shared-axis snap, separating successive
height rings into different source columns.

Walking mineral publishes the new upper surface, including its within-cell
triangle interpolation. Actor support reads the same geological calculation
and actual neighboring standing-ground heights. Ordinary source terrain
offsets apply afterward to both geometry and support. The slope correction now
also honors a source datum when the current frame has no visible jump edges;
the old early return was hidden by the former compensated mineral fill.

A complete cave mouth now receives its real zero-height floor before terrain
offsets. Its four source cells formerly retained the roof drawing's support
height. Counting that as surrounding ground inflated the first complete-body
cave preview. The cave's claimed cells are seated explicitly; other cells keep
their support. The shared ground calculation also recognizes complete entrance
plots. Standing clearance, the dark lining, closed threshold and native warp
coordinates retain their existing policies.

All 214 distinct selected checks pass, including eight explicitly executed
external-pack checks. The new wall-only diagonal fixture verifies the complete
top, straight upper height rings, seated roots and unchanged short ground
banks. The production Route26 directional-mineral regression checks actor
contact against actual cap triangles at twelve fractional positions. The
datum-only regression checks both geometry and actor support without visible
jump edges. Existing cap closure, triangle ordering, cave clearance, shore,
footing, renderer, terrain batch and worker checks pass, including the
eleven-region worker roundtrip. Native compilation succeeds. Logs are
`target/outdoor-whole-mountain-verified-tests.log` and
`target/outdoor-whole-mountain-native-build.log`.

The external cap/wall guard groups original source points by their face owner.
Every cap copy coincides, and each wall face's actual upper endpoint must meet
that cap within 0.001 world units. A lower native cap could previously intersect
a taller wall halfway up. Reparameterizing the complete body moves that cap to
the new wall top; the former middle height ring no longer represents the join.
The unequal-course fixture separately requires distinct increasing wall rows,
straight shared edges, and coincident roots and upper endpoints, even when
the two courses have the same nominal terrain tier.

Twenty-nine final native views and all their same-state 2D counterparts were
personally inspected across fifteen maps: Routes2, 3, 9, 10South, 14, 20, 26,
44, 45 and 46, Blackthorn, Cianwood, Cinnabar, MountMoonSquare and NationalPark.
Twenty-six views use eye height; the two cap views and the Route26 movement
view are elevated. Two views use 22:00 lighting. Every paired receipt preserves
state, and every final image postdates the native binary built after the final
terrain changes. The evidence is in `target/outdoor-whole-mountain-review/`.
The two additional movement stills were also inspected. Both 32-frame keyboard
walks retain active terrain, one build, no pending work, zero fade and no warp:
Cianwood moves `(10,20)` to `(6,20)`, and Route26 moves `(9,28)` to `(13,28)`.
The latter also shows the trainer's contact on the reshaped walking mineral.
The static Route45 mineral pose remains an art diagnostic, not gameplay proof.

The continuous lower bodies remove the broad aprons and tall underlying shafts
from the previous height candidate. The improvement is clearest at Cianwood,
Cinnabar and the smaller Seafoam island. Visual acceptance remains open.
Repeated triangular corner marks are still visible in Route9 `eye4` and
Route46 `eye6`, despite the new row-fit checks. Projecting shoulders remain in
Route46 `eye2` and Route3 `eye5`; those checks do not establish the native
artifacts' visual resolution. Long straight ridges and faint horizontal bands
remain on Routes44 and 45. Some cuts and pointed feet remain coarse on Route26
and at Blackthorn. Cinnabar retains its water-side platform, and distant small
islands remain square or pyramidal. Rigid pond/shore margins, repeating rock
and buoy courses, Seafoam's bright paving gaps, coarse and porous upper
foliage, Route12's distant land wedge, and rectangular ground patches remain.
These checks establish neither browser visual acceptance nor general
performance. Earlier whole-body candidates remain in the ignored first-probe,
column-before, course-before and course-sanity capture directories. Temporary
geometry exports, trace probes, diagnostic source hooks and task helpers were
removed; no generated game content was published.

### Moderate summit height and low foothills — 2026-10-21

The latest height direction is “Maybe not that tall.” The natural added
mountain relief now uses 65% of the previous height. Its original 0.85 field
still supplies pigment, while geometry, mineral footing and within-cell actor
support share the reduced field. Required cave-roof growth stays full height:
the first reduction also shortened low entrances, which the standing-clearance
regression rejected. Native courses, terrace offsets and water levels keep
their existing roles. This is a 35% reduction of added natural relief, not a
35% reduction of every mountain's total height.

Unreachable outer courses now start as low foothills, approximately 23–33% of
their local native height above the neighboring ground. Source-owned mineral
beds vary the amount. These low heights are sampled at shared grid corners
before interpolation; looking up a new ground height independently at every
surface vertex could otherwise leave a low ring beside raised ground. The
existing two-cell transition restores the filled interior. Walking plots,
complete entrances and raised-water contacts retain their native course while
following the reduced natural relief. An adjacent building's standing ground,
rather than its roof height, sets the foot of the slope.

The old mid-height horizontal bevel is removed above the rooted bottom sixth.
Compressing it into a much shorter outer face produced a projecting shoulder.
A first attempt to shrink each cap's horizontal inset with its lowered height
also folded a Blackthorn cap triangle. The final cap keeps one shared inset
field; vertical walls approach it from their unchanged ground contacts. The
topology regression is retained. No filler walls, new textures, extra entities
or new mesh subdivisions are introduced.

All 210 distinct selected checks pass, including eight explicitly executed
external-pack tests. A new small-body fixture checks the low outer course,
strictly rising samples toward the middle, the reduced center height, fixed
ground contacts and an adjacent tall roof over both ordinary and raised ground.
The existing cap/wall closure, outward winding, horizontal gradient, complete
walking-triangle contact and cave-clearance checks remain. The height threshold
for added summit relief follows the intentional 35% reduction. Terrace, shore,
footing, renderer, batch and worker checks pass, including the eleven-region
worker roundtrip. Native compilation succeeds. Logs are
`target/outdoor-low-foothills-verified-tests.log` and
`target/outdoor-low-foothills-native-build.log`.

Twenty-five final native views and every same-state 2D counterpart were
personally inspected across Routes2, 3, 9, 10South, 14, 20, 26, 44 and 46,
Blackthorn, Cianwood, Cinnabar, MountMoonSquare and NationalPark: fourteen maps.
Twenty-four views use eye height; one elevated Route44 view exposes the cap
join. Two views use 22:00 lighting. Every paired receipt preserves state, and
every final image postdates the final native binary. The 32-frame Cianwood
keyboard walk reaches `(6,20)` from `(10,20)` with terrain active, one build,
no pending job, zero fade and no warp. Its movement still was also inspected.
Final evidence is in `target/outdoor-low-foothills-review/`.

The smaller Seafoam island now has a lower, more continuous flank. The
previously folded Route9 shoulder no longer projects over its face. Whole-route
visual acceptance remains open: long straight faces and crests on Route44,
Cinnabar and Route46 still read as walls in close views. Some native courses
retain coarse cuts and angular feet, including Routes9 and 46 and Blackthorn.
The wider low aprons remain too uniform, and Cinnabar's water-side foot still
reads as a platform. Distant islands remain square or pyramidal. Rigid pond and
shore margins, repeated rock/buoy courses, Seafoam's bright paving gaps, coarse
foliage, porous upper leaders, Route12's distant land wedge and rectangular
ground patches also remain. These checks establish neither browser visual
acceptance nor general performance.

Rejected early height/inset previews remain ignored in
`target/outdoor-low-foothills-rejected-probe/` and
`target/outdoor-low-foothills-bevel-probe/`. The temporary profile hook, raw
diagnostic output and capture/verification scripts were removed. Packs and
authored assets were not regenerated.

### Lower outer courses and rock texture on slopes — 2026-10-21

A production Route44 trace showed a native two-tile course, 64 world units at
the current scale, being filled to roughly 129–130 units by a higher point in
its connected rock mass. The fill now transitions from the local native course
at exposed margins to the existing regional fill two source cells inward.
Complete walking plots, cave entrances and raised-water contacts retain their
published datum. The high interior and 0.85 relief scale remain. Ground contacts
stay fixed. Existing pigment uses the previous filled-height calculation, so
lowering the outer course does not choose a different native paint recipe.

The first preview exposed broad horizontal bands and stretched texture. The
material still used the original flat X/Z coordinates after the rock had been
sculpted; an upward-facing portion of a sloped wall could consequently sample
one texture row along its height. A final mineral-paint pass now registers detail
on the finished 3D surface after geometry fitting. It preserves the existing
material family and source-map coordinate basis through scrolling and connected
paint. Atlas materials, other paint domains and distant repeated donor art are
excluded. This stage changes UVs only. It adds no textures, geometry or entities.

The shared flank profile also now follows one plane between its endpoints.
Universal bends at 22% and 70% of every face's height made unrelated masses
repeat the same horizontal shoulders. Lower mineral weathering remains, while
those fixed-height bends are removed. The final side views retain angled
slopes, and their paint has less directional stretching. This does not resolve
the broader rectangular mountain silhouettes.

All 209 distinct selected checks pass. Four new checks cover a high native
course filling the center without doubling a lower outer face, unchanged
native pigment and walking datum, detail coverage on actual slopes at three
scales/material families, scroll/connection stability and excluded material
domains. The existing filled-mass regression now checks the retained high
interior beyond the transition and requires lowered wall samples to remain
below their joined cap, preventing an upright perimeter above it. Its contact,
gradient and walking-surface checks remain. Rock, cave, terrace, shore, footing,
renderer, batch and worker checks pass, including eight explicitly executed
external-pack tests and the eleven-region worker roundtrip. Native compilation
succeeds. Logs are `target/outdoor-local-cliff-fill-verified-tests.log` and
`target/outdoor-local-cliff-fill-native-build.log`.

Thirty final native views and every same-state 2D counterpart were personally
inspected across Routes2, 3, 9, 10South, 14, 20, 26, 44 and 46, Blackthorn,
Cianwood, Cinnabar, MountMoonSquare and NationalPark: fourteen maps. All receipts
preserve state. Twenty-nine views use eye height; one elevated Route44 view
exposes the cap/face join. Three views use 22:00 lighting. Route44 includes four
opposing headings, and Route46 adds two side views beside its close northern
face. The 32-frame Cianwood keyboard walk reaches `(6,20)` from `(10,20)` with
terrain active, one build, no pending job, zero fade and no warp. Its movement
still was also inspected.

Final evidence is in `target/outdoor-local-cliff-fill-review/`, with matching
earlier views in `target/outdoor-retained-flanks-review/`. Three rejected banded
previews remain in `target/outdoor-local-cliff-fill-banded-probe/`. The temporary
source trace hook, diagnostic output and capture/verification scripts were
removed. Packs and authored assets were not regenerated.

Whole-route acceptance remains open. Route44 and Cinnabar still read as broad
walls front-on. Route9 and Route46 expose block-like masses and coarse cuts;
Seafoam's isolated island retains tiers, and other isolated islands remain
square or pyramidal. Rigid shore/pond rims, repeated rock/buoy courses and
Seafoam's bright paving gaps remain. Close Route3 foliage is coarse and crowded;
porous upper broadleaf leaders and broad pine sprays still need work. Route12's
distant green land wedge and rectangular ground patches on Routes2 and 9 also
remain. These native checks do not establish browser visual acceptance or
general performance.

### Inward mountain slopes at the existing height — 2026-10-21

Permanent native mineral bodies now have a shared horizontal slope field. Upper
flanks recede into their existing rock footprints through broad angled sections
above a short rooted toe. Source-owned mineral beds vary the amount along a
run; the influence fades over four source cells. A cropped frame cannot create
a new geological edge. Corner samples and interpolation are shared by cap and
wall copies. This reshapes the existing mesh without adding triangles, filler
volumes, textures or entities.

The filled height field remains unchanged, including its rising interiors and
0.85 relief scale. Native paint, material UVs, ground contacts, claims and
walking heights retain their existing policies. Complete walking plots keep
the source surface used by actor support; a shared two-cell fade protects their
neighbors. Complete cave entrances and elevated standing-water contacts also
retain their source outline. Ordinary dry ground, water, buildings and foliage
are not displaced by this mineral field.

Simply retaining the old upper weathering restored only a small bevel and left
Cinnabar's broad face too vertical. Combining that old corner warp with the new
slope also folded some cap triangles. The rock-contact regressions caught this
before the final review. The final mapping first puts upper samples on one
common horizontal surface, then applies the inward slope. Lower weathering
eases into it. The formerly projecting Cianwood shelf beside the house now reads
as a sloping body; Route26's nearby corners and Cinnabar's exposed flank also
have more modeled shape.

All 205 distinct selected checks pass. Four new regressions cover inward
upper slopes, ordered cap triangles, unchanged filled heights and pigment,
fixed ground contacts, joined cap/wall copies, walking-grid protection, source
crops and connected-map placement. Existing rock-base, cave, terrace, footing,
shore, renderer, batch and worker checks pass. Eight external-pack tests were
explicitly executed, including the Blackthorn and Route44 cap/wall seams, five
Kanto cap layouts, directional walking mineral and the eleven-region worker
roundtrip. Native compilation succeeds. Logs are
`target/outdoor-retained-flanks-verified-tests.log` and
`target/outdoor-retained-flanks-native-build.log`.

Twenty-four final native views and all same-state 2D counterparts were
personally inspected across Cinnabar, Cianwood, Blackthorn, Lavender,
MountMoonSquare, NationalPark, Routes2, 10South, 14, 20, 26 and 44: twelve maps.
All receipts preserve state. Twenty-three views use eye height and one elevated
Cinnabar view exposes the upper body and pond. The set includes opposing
headings, two 22:00 views, cave entrances, town/building context and inland,
forest and coast controls. The 32-frame Cianwood keyboard walk reaches `(6,20)`
from `(10,20)` with terrain active, one build, no pending job, zero fade and no
warp. Its movement still was also inspected.

Final evidence is in `target/outdoor-retained-flanks-review/`. Matching earlier
Cianwood, Cinnabar and Route26 views remain in
`target/outdoor-beach-bank-review/`; the first weak bevel preview is in
`target/outdoor-retained-flanks-probe/`. Temporary capture and verification
scripts were removed. Packs and authored model assets were not regenerated.

Whole-route acceptance remains open. Route44's close northern face still reads
as a long straight cut, as do some protected Route26 faces. Cinnabar remains
large and wall-like close up despite its improved flank. Seafoam's isolated
mineral island has coarse tiers, and other isolated islands remain square or
pyramidal. Rigid pond/shore rims, repeated mineral and buoy courses, Seafoam's
bright interior paving gaps, crowded broadleaf foliage, porous upper leaders,
broad pine sprays, Route12's distant green land wedge and rectangular Route2
ground patches still need work. These native checks do not establish browser
visual acceptance or general performance.

### Sloping sand at coastal waterlines — 2026-10-21

Eligible native sand caps now meet the lowered coastal water through closed,
sloping sand shoulders. The dry cap stays at its previous height. Each shoulder
extends 0.30–0.44 source cells into proven neighboring water, reaches the
existing liquid level and closes twelve percent farther beneath it. Shared
source-owned corner widths keep adjacent sections joined across connected-map
handoffs. Rounded outer corners require water on both sides and diagonally.
Only the exposed upper slopes share normals; hidden closure faces do not flatten
their shading. The geometry joins the existing terrain batch.

The new faces sample the completed cap's pigment, source coordinates and
material role. Actual Route20 `$1f/$33` border caps use native sand role 11;
the simpler fixed-palette sand fixture uses role 6. Both retain their finish.
Wet shading darkens the received pigment by at most twenty percent, preserving
the native day/night hue. Earlier previews added no native bank because their
source and material guards were too narrow; those previews were rejected.
A temporary source probe identified the actual role before the final change.
Physical bank geometry also remains identical with paused water or native
level-plane source shapes. It does not depend on animation metadata.

Ground must have an authenticated sand source, an existing zero-height sand cap
and an eligible coastal owner. Raised, priority, paved, indoor and other claimed
surfaces are excluded. Source caps, liquid surfaces, actor support, claims and
navigation remain unchanged. An existing cap-continuation regression also
exposed the flat `BANK_GROUND_UV` marker being omitted from adjacent-ground
finishing; that selection now includes it. The existing rock-local wave wash
remains visible around the foreground Seafoam rocks. Mountain relief remains
0.85, and the approved colors are preserved.

All 133 distinct selected checks pass, including five new sand-bank regressions
and the explicitly executed eleven-region external-pack worker roundtrip.
Checks cover closed outward geometry, bounded growth, unchanged original cap
and liquid bytes, native role-11 paint, rejected sources, paused-water
equivalence and connected-map handoffs. Shore, water, footing, renderer, batch
and worker checks also pass. Native compilation succeeds. Logs are
`target/outdoor-beach-bank-verified-tests.log` and
`target/outdoor-beach-bank-native-build.log`.

Twenty-one final native views and every same-state 2D counterpart were
personally inspected across Route20/Seafoam, Cinnabar, Cianwood, Olivine,
Cherrygrove, Vermilion, Routes2, 14, 19, 26 and 40, and NationalPark: twelve
maps in total. All receipts preserve state. Twenty views use eye height; one
elevated Seafoam view exposes the rounded corners. Opposing views, two 22:00
views, town harbors and inland/forest/mountain controls are included. Some sea
positions are waterborne diagnostics. The 32-frame Cianwood keyboard walk
reaches `(6,20)` from `(10,20)` with terrain active, one build, no pending job,
zero fade and no warp. Its movement still was also inspected.

Final evidence is in `target/outdoor-beach-bank-review/`; three original pairs
remain in `target/outdoor-coastal-contact-before/`. The ineffective previews
are in `target/outdoor-beach-bank-probe/` and
`target/outdoor-beach-bank-border-probe/`. Temporary source hooks, diagnostic
output and capture/verification/inspection scripts were removed. Packs and
authored models were not regenerated.

The sand toe and outer corners hold together, but whole-route acceptance remains
open. Seafoam rotations 5 and 7 expose bright pinholes along the protected
interior paving margin. The opposing Cinnabar view exposes a large flat grey
volcanic wall; Route26 still has coarse planar cut faces. Rocky Johto rims,
mineral overhangs, square or pyramidal isolated islands, rigid pool rims and
repeated rock/buoy courses remain. Narrow or porous upper broadleaf leaders,
crowded close foliage, broad pine sprays, Route12's distant green land wedge
and rectangular Route2 ground patches also need work. These checks establish
native evidence for this change, not browser visual acceptance or general
performance.

### Living growth along exposed broadleaf boughs — 2026-10-21

The tallest broadleaf model's leafless side limbs and long bare supports made
its lower crown read as a forked skeleton. Rust now identifies complete authored
wood components once when loading the three models. It retains every component
reaching the foliage and its complete lower support chain, then trims whole
leafless terminal limbs. The supporting trunk surface remains intact; no kept
limb is sliced into an open stump. Unreferenced bark vertices are removed from
the rendered mesh as part of its index remap.

Small leafy shoots now graft directly onto exposed living boughs and upper
trunks in mature trees. Their roots use actual authored bark vertices and
deformed surface normals. Existing leaf coverage excludes crowded attachment
sites. Each curved stem has closed sides and both end caps; its twelve closed
leaflets use the existing bounded leaf scale and native pigment. Short stems
and varied roll keep the sprays from reading as parallel flat fronds. Young
trees retain their existing crown clearance. The twelve shared age prototypes,
planted plots, root contacts, height range and colors remain. Mountain relief
remains 0.85. No crown hulls, filler walls or sky masks were introduced.

Additional shoots are capped at one per three existing crown sprays. Each adds
36 stem triangles and 144 leaf triangles; pruning also removes unused bark
geometry. This is additional modeled growth, not a general performance result.
The first candidate widened upright crowns but worsened close crowding while
leaving bare supports obvious; that width change was reverted. A second preview
had excessively flat sprays and was revised before the final review.

All 136 distinct selected checks pass. New regressions retain bare parent limbs
under leafy children, preserve all shipped foliage supports and roots, reject
partial limb cuts, verify actual shoot attachment, enforce the growth budget
and young-tree policy, and check closed outward stem geometry. Shared groves,
footing, renderer, batches and the explicitly executed eleven-region external
worker roundtrip also pass. Native compilation succeeds. Logs are
`target/outdoor-living-boughs-verified-tests.log` and
`target/outdoor-living-boughs-native-build.log`. Two existing test-fixture compile
issues in `arm_skin.rs` were corrected with an explicit float type and default
mesh fields; its production behavior was not changed.

Nineteen final native views and every same-state 2D counterpart were personally
inspected across Routes2, 13, 14, 15, 20, 25, 26 and 31, ViridianCity,
IlexForest and NationalPark. All receipts preserve state. Eighteen views use
eye height; one elevated view exposes crown coverage. The set includes opposing
headings, Route15 at 22:00, and pine, ancient-tree, formal park, coast and
mountain controls. The 32-frame Route14 keyboard walk reaches `(13,15)` from
`(13,10)` with terrain active, build count one, no pending job and zero fade.
Its final pair enters trainer dialogue; it does not prove traversal past that
trainer. The movement still was also inspected.

Final evidence is in `target/outdoor-living-boughs-review/`. Four original pairs
remain in `target/outdoor-living-boughs-before/`; rejected width previews are in
`target/outdoor-living-boughs-rejected/`, and flat-spray previews are in
`target/outdoor-living-boughs-flat-sprays/`. Temporary capture/verification
scripts and the scratch source copy were removed. Packs and authored model
assets were not regenerated.

Bare lower forks are less dominant, but whole-route acceptance remains open.
Some isolated upper leaders are still narrow, straight and porous; close
foliage remains crowded. Broad pine sprays, repeated stone/buoy courses, coarse
mineral overhangs and cutaway planes, isolated island shapes, rigid pool rims,
Route12's short distant green land segment and rectangular Route2 ground
patches still need work. These native checks do not establish browser visual
acceptance or general performance.

### Broader crowns and finer broadleaf sprays — 2026-10-21

Kanto broadleaf crowns now spread more with maturity and turn continuously
around their planted centers. Mature foliage starts lower on the existing
branch systems. A growing canopy allowance and fitting against the actual root
outline prevent diagonal poses from squeezing the entire crown to fit a root
bounding rectangle. The three authored branch architectures, twelve shared age
prototypes, original planted trees, root contacts and roughly 2.6–12.2 tile
height range remain. Pine and ancient-tree instance poses retain their previous
rules. Mountain relief remains 0.85 with the approved colors.

Each selected authored foliage site now carries twelve smaller closed, cupped
leaflets with six-edge outlines and individual rolls. Alternating source sites
limit overlapping sprays; this does not remove planted trees. Physical leaf
growth stops at 1.35 times the stem scale while branch spread can continue to
grow. The native pigment calculation remains. The first candidate still looked
like large pointed shards close to camera and was rejected. The final leaves
read as finer clusters, although close crowns remain crowded. No opaque crown
hulls, filler walls or sky masks were added.

The final spray uses 144 triangles at each selected site, approximately 2.25
times the previous total leaf triangles after selecting alternate sites. The
shared prototype capacity remains 144. All 133 distinct selected checks pass,
including closed leaves, projected leaf area at the bounded physical scale,
normals, continuous deterministic poses, actual-root fitting, shared grove
placement, footing, renderer/batching checks and the explicitly executed
eleven-region external-pack worker roundtrip. Native compilation succeeds.
Logs are `target/outdoor-rounded-broadleaf-verified-tests.log` and
`target/outdoor-rounded-broadleaf-native-build.log`.

Twenty-one final native views and their same-state 2D counterparts were
personally inspected across Routes2, 8, 13, 14, 15, 20, 25, 26 and 31,
ViridianCity, IlexForest and NationalPark. All receipts preserve state. Twenty
views use eye height; one elevated view exposes crown coverage. The set includes
opposing headings, two 22:00 views and pine, ancient-tree, coast and mountain
controls. A 32-frame production keyboard walk on Route14 reaches `(13,14)` from
`(13,10)` with terrain active, build count one, no pending terrain job and zero
fade. The final pair includes trainer attention; this is not evidence of
uninterrupted traversal past that trainer. The movement still was also inspected.

Final evidence is in `target/outdoor-rounded-broadleaf-review/`; five original
pairs remain in `target/outdoor-rounded-broadleaf-before/`, and two rejected
first-candidate pairs are in `target/outdoor-rounded-broadleaf-rejected/`.
The capture script was removed. Packs and authored model assets were not
regenerated. Whole-route acceptance remains open: upright trees still expose
excessive lower forks, some tall coastal crowns remain narrow or porous, and
close foliage remains crowded. Broad pine sprays, repeated stone courses,
coarse mineral overhangs and cutaway planes, isolated island shapes, rigid pool
rims and Route12's short distant green land segment need further work. These
native checks do not establish browser acceptance or general performance.

### Molded offshore floats — 2026-10-21

The shared Johto/Modern float producer now has a rounded shoulder, tapered
crown and small closed recessed cap. Its white body and blue collar/cap sample
the same native atlas swatches. Six body rings and two closed quad fans retain
the previous 96-quad budget. Gentle differences in tilt and proportions stay
inside the source footprint and the previous `-0.18..+0.30` tile envelope
relative to water. Source centers, course layout, water beds, footing and
claims retain their existing policies. Resting poses use the owning map's
coordinates, so connected-map handoffs cannot reseed the same float.

A separate live-paint tag gives only these floats neutral face shading and
a restrained plastic response. The game palette still supplies time-of-day
color. The first night candidate lost most normal shading; this was corrected
before the final review. Both day and night now retain modeled volume without
a second moon tint. This does not add bobbing or another water material.

All 150 distinct selected checks pass, including the external worker roundtrip
expanded from eight to eleven source regions with Cianwood, Olivine and
Route40. New checks exercise 64 poses for closed volume, outward faces and
normals, nondegenerate triangles, footprint/envelope bounds, native paint and
the unchanged geometry budget. A connected-map ownership regression compares
the actual appended mesh across a handoff. The paint-role regression verifies
coordinate roundtrips and separation from the other material domains. Final
buoy, paint and renderer tests were rerun after the night-shading correction.
Logs are `target/outdoor-molded-buoy-verified-tests.log` and
`target/outdoor-molded-buoy-native-build.log`.

Twenty final native views and all same-state 2D counterparts were personally
inspected across Cianwood, Olivine, Cherrygrove and Routes13, 19, 26, 30, 40
and 41. All receipts preserve state. Eighteen views use eye height; two use
65-degree elevated cameras to expose the closed caps. The set includes
opposing coast views, two 22:00 views, land/forest/mountain controls and a
32-frame Cianwood keyboard walk from `(10,20)` through `(6,20)`. Its terrain
stays active, build count stays one, no terrain job is pending and fade stays
zero. The final walking pair and movement still were inspected. Route41,
Cherrygrove and Olivine include waterborne diagnostic positions.

Final evidence is in `target/outdoor-molded-buoy-review/`; three original coast
pairs remain in `target/outdoor-molded-buoy-before/`, and the initial flat night
pair is in `target/outdoor-molded-buoy-probe/`. The capture script was removed;
packs and authored assets were not regenerated. Whole-route acceptance remains
open: courses still dominate some distant views, foliage is crowded/repeated,
and coarse mineral overhangs, isolated island shapes and rigid shore/pool rims
need further work. Mountain relief remains 0.85 with the approved colors.
These native checks do not establish browser visual acceptance.

### Open sea beyond mixed coastal frames — 2026-10-21

The tan Route13 and green Route14 ribbons were distant ground continuation,
not sky paint. Route13 was absent from the coastal eligibility list. Route14's
reviewed frame had 7,001 live-water cells out of 14,400, so the old whole-frame
majority test selected ground despite wet top, bottom and right edges.

Routes12–15 now participate in coastal continuation. Frames that already
qualify as open sea retain their four water faces. Otherwise, each outer edge
requires its own majority of verified animated water. Mixed continuation uses
eight disjoint perimeter sectors; a sea edge's two outer corners also continue
as water, preventing a dry strip beyond it. Other sectors retain their ground
source. The water faces remain separate through source-cell fitting, then join
the ordinary surface before the common quarter-tile waterline fit. No new
footing, collision, walls, sky masks or authenticated source area is introduced.
The approved colors and mountain relief of 0.85 remain.

All 124 selected checks pass, including the external eight-region worker
roundtrip. The new mixed-coast regression covers 64 combinations of route,
edge direction, tile scale and instancing. It verifies sector coverage, closed
corners, outward winding, native live-water UVs, the completed water level,
unchanged source/ownership/support behavior and outside-domain classification.
Negative cases reject inland maps, enclosed dry boundaries, unanimated art,
single-frame art and priority scenery. Logs are
`target/outdoor-sea-continuation-verified-tests.log` and
`target/outdoor-sea-continuation-native-build.log`.

Twenty-two final native eye-height views and all same-state 2D counterparts
were personally inspected across thirteen maps: Routes12, 13, 14, 15, 19, 26,
30 and 40, Cianwood, Cherrygrove, Olivine, LakeOfRage and NationalPark. All
receipts preserve state. The set includes diagonal and land-facing controls,
two 22:00 views, both inland pools, the lower mountain scale and a Route14
keyboard walk. That 32-frame walk reaches `(13,15)` from `(13,10)` while the
source origin scrolls; terrain stays active, build count stays one, no job is
pending, and fade stays zero. Its final pair and movement still were inspected.
Route12, Cherrygrove and Olivine include waterborne diagnostic positions.
Final evidence is in `target/outdoor-sea-continuation-review/`; the two original
paired coast comparisons remain in `target/outdoor-sea-continuation-probe/`.

The full-width Route13/14 ribbons are gone. Whole-route acceptance remains
open: Route12 retains a short distant green land segment requiring further
boundary inspection; bright buoy courses, repetitive stone rows, crowded or
repeated crowns, broad pine sprays, coarse mineral overhangs and rigid pool
rims still need work. These native checks do not establish browser acceptance.
Temporary trace hooks, count traces and capture scripts were removed; content
packs and authored assets were not regenerated.

### Complete cliff bodies beside modeled roofs — 2026-10-21

The opening beside Cianwood's house persisted with player reveal disabled and
with flat gray mineral paint. The source layout identifies the cause: cliff
cells `(13..15,31)` and the roof immediately south both have a native height
of 16. The terrain builder treated that roof as adjoining geological fill and
omitted the shared cliff faces. The modeled roof does not cover their full
area, leaving visible sky below the cap.

Both outdoor terrain builders now distinguish raised object tops from their
ground support. An adjacent roof, tree or other raised object supplies its
standing floor as the cliff's lower contact, rather than suppressing the face
at its old top. The common rule covers the connected mineral producer and
the separate native rock-platform path across Johto, modern Johto and Kanto.
The existing mineral footprint and source ownership remain; this repairs the
rock's own side. Mountain relief stays at 0.85 with rising interiors, and the
approved mineral palette remains. Source footing, ramps, water, cave corners
and authored openings retain their separate policies.

The new regression checks projected face coverage and outward winding for
96 combinations of source family, terrain builder, direction, ground level
and equal/taller roof. It also preserves the standing-height buffer and the
neighboring model's ownership. An explicitly executed external-pack test
verifies complete side faces at the three actual Cianwood source contacts.
All 223 focused checks pass in `target/outdoor-house-rock-verified-tests.log`,
including ten external-pack checks, the eight-region worker roundtrip and
the existing cave-return regressions. Native compilation succeeds in
`target/outdoor-house-rock-native-build.log`.

Twenty final native eye-height views and all their same-state 2D counterparts
were personally inspected across Cianwood, Blackthorn, Cinnabar, Lavender and
Routes8, 26, 29, 31, 44 and 45. Cianwood includes the original view, three closer
headings, a sea-facing control, 22:00 and a production keyboard walk. The set
also includes opposing mineral/route views, Blackthorn at day/night, Cinnabar's
volcanic pond and pine/Kanto controls. All twenty receipts preserve state.
Route45 `(7,16)` and Cinnabar `(15,6)` remain static mineral diagnostics rather
than ordinary walking proof.

The 32-frame Cianwood walk records `(10,20)`, `(9,20)`, `(8,20)`, `(7,20)` and
`(6,20)` through production keyboard input. Terrain remains active, no terrain
job is pending, fade stays zero and build count stays one. Its final paired
views and a movement still were inspected. Final evidence is in
`target/outdoor-house-rock-review/`; the original comparison and negative
reveal/paint probes remain in `target/outdoor-house-rock-probe/`. Temporary
source-role probes, review shaders and duplicate candidates were removed.
No content pack or authored model asset was regenerated.

The reviewed opening is closed in the rock geometry. Whole-route acceptance
remains open: close mineral overhangs and cutaway planes, Cinnabar's rigid pond
rim, crowded foliage, repeated crowns and stone courses, bright buoys and
distant coast strips still need improvement. These native checks do not
establish browser visual acceptance.

### Seat hidden ground below weathered mountain faces

Cianwood's thin bright mountain line was source-ground backing at the old
mineral standing height. That cap datum left a horizontal ground surface
crossing the recessed rock face. Both Johto terrain families now use their
authenticated neighboring ground's support height, as the separate Kanto
backing already did. Existing rock vertices, normals, pigment and texture
coordinates remain unchanged. The 0.85 mountain relief and rising interiors
remain, as do source footing, ownership and navigation.

Flat-material, normal and material-role probes identified the exposed ground.
A hide-only fragment probe retained its depth mask and was unsuitable for
judging a physical opening; the corrected production render supplies that
visual evidence. Temporary geometry snapshots and their source hook were
removed. The original paired comparison and diagnostic views remain in
`target/outdoor-mineral-join-probe/`.

Twenty final native eye-height views and all their same-state 2D counterparts
were personally inspected across Cianwood, Blackthorn, Cinnabar, Lavender and
Routes8, 26, 29, 31, 44 and 45. The set includes five Cianwood headings, two
22:00 views, opposing mineral views, raised paths, connected ridges, volcanic
paint and pine/Kanto controls. All twenty receipts preserve state. Route45
`(7,16)` and Cinnabar `(15,6)` are static mineral diagnostics, not walking
proof. Three obstructed initial shots remain in the probe's `rejected/`
directory: Lavender's `(3,2)` building overlap, its `(5,10)` door-facing view
and Cinnabar's occluded `(9,6)` view. The final Lavender view uses `(5,10)` at
rotation step 2; the final Cinnabar view uses `(15,6)` at step 4.

The 32-frame Cianwood production keyboard walk records `(10,20)`, `(9,20)`,
`(8,20)`, `(7,20)` and `(6,20)`. Terrain stays active with no pending job, zero
fade and a build count fixed at one. Its final native/source pair and a
movement still were inspected. Final views and the movement receipt are in
`target/outdoor-mineral-join-review/`.

All 168 focused checks pass in `target/outdoor-mineral-join-verified-tests.log`.
The regression distinguishes neighboring ground from the higher mineral seat
in both Johto families, including raised ground, while retaining the original
rock and authoritative support. Existing clipping, ownership, water exclusion,
footing, terrace, renderer and worker checks pass. Nine external-pack checks
were explicitly executed, including shared cap/wall samples on Route44 and
Blackthorn and the complete eight-region worker roundtrip. The production
native build succeeds in `target/outdoor-mineral-join-native-build.log`.
Duplicate candidates and temporary review shaders were removed; no content
pack or authored model asset was regenerated.

The crossing bright line is resolved in the inspected views. A separate
opening behind Cianwood's house remains visible at rotation step 0; the older
lower-mountain comparison already shows it. Whole-route acceptance remains
open, including that opening, crowded foliage, repeated crowns and stone
courses, bright buoys, distant coast strips and elevated cutaway joins.
These native checks do not establish browser visual acceptance.

### Remove painted woodland sectors from the sky

The dark rectangular horizon patches came from the sky material's directional
woodland overlay. It sampled the farthest source cells into 64 azimuth sectors
and painted their tree coverage onto empty sky. Native probes with modeled
shadows, modeled occlusion and tilt shift individually disabled retained the
patches. Removing that overlay clears the rectangles above the Route8 ridge
and behind Route25's trees. The CPU sampler, cache and matching shader uniforms
were removed with it. Actual trees supply the silhouettes; no backdrop geometry
or replacement sky mask was added.

Distance fog on real woodland geometry retains its existing source coverage,
water exclusion and clock response. The original sky palette, clouds, sun,
moon, stars and camera following remain. Cave and room sky exclusion and the
open roof policy also remain. Mountain relief stays at 0.85, with interiors
rising above their outer slopes and the approved source colors retained.

Twenty final native eye-height views and all their same-state 2D counterparts
were personally inspected across fourteen maps: Routes2, 8, 13, 14, 25, 31, 36
and 45, Cianwood, National Park, Ilex Forest, Dark Cave's Violet entrance,
PlayersHouse1F and CeladonMansionRoof. The set includes diagonal ridge views,
two 22:00 comparisons, broadleaf and pine coverage, open coasts, the formal
park and enclosed/open-roof controls. Ilex retains its authored dark palette.
Dark Cave's unlit view is a sky-exclusion control, not geometric acceptance.
Route45 `(7,16)` remains a static mineral diagnostic. All twenty paired-state
receipts preserve state. The 32-frame Route36 production keyboard walk records
`(30,8)`, `(31,8)` and `(32,8)`, active terrain, zero fade, no pending terrain
and a build count fixed at one.

All 73 focused checks pass in `target/outdoor-horizon-verified-tests.log`;
one unrelated GPU blur comparison remains explicitly ignored. The checks cover
the actual published sky material across forest, coast, water and land source
frames, viewport origin changes, day/night continuity, geometry fog, room/roof
visibility, lighting and renderer integration. The production native viewer
build succeeds in `target/outdoor-horizon-native-build.log`. Final pairs and
the movement receipt are in `target/outdoor-horizon-review/`. The original
comparison and three negative diagnostic probes remain in
`target/outdoor-horizon-probe/`; the duplicate candidate batch was removed.
No content pack or authored model asset was regenerated.

The painted horizon rectangles are resolved in the reviewed native views.
Whole-route acceptance remains open: crowded close foliage, bare forks and
repeated crowns, regular coast and barrier courses, bright buoys and elevated
cutaway joins still need improvement. The coastal views expose a thin distant
ground strip, and Cianwood retains a thin bright mineral join; neither is
resolved by this sky change. These checks do not establish browser visual
acceptance.

### Natural barrier-stone shoulders and mineral surfaces

Kanto's dry barrier stones no longer receive the flattened hand-cut crown.
Their three existing cached derivatives now have unequal sloping shoulders.
Each planted stone also receives deterministic heading, width, offset and rise
variation inside its native plot. Dry-stone peaks span 1.35–1.75 source tiles,
centered on 1.55. Fitting the actual sculpted height prevents the shoulder cuts
from silently shortening the stones. Ground contacts, native underlays,
authored ownership and gameplay footing retain their source domains. The
separate path drawing keeps its existing treatment.

Native light ink now grades continuously over the upper stone instead of
forming an opaque white lid. Duplicate positions share the same pigment and
sculpted normal across material partitions. The source palette and its night
exposure remain authoritative. Dry stones opt into the same existing
world-anchored mineral detail as the sea rocks, using a separate dry paint
marker; they do not enter water optics or acquire water support. The retained
solid-material registration already supplies scale and translation on dry maps.
No new texture asset, geometry entity or model-cache entry was added, and the
stone topology retains its existing triangle count.

Twenty final native low-angle views and their same-state 2D counterparts were
personally inspected across Routes2, 8, 14, 15, 20, 25, 31 and 45, Viridian and
Cianwood. The review includes four opposing Route8 and Route14 views, nearby
and distant barrier rows, two 22:00 views, sea-rock controls, the pine control
and the user's lower Route45 mineral plot. All twenty paired-state receipts
preserve state. Route45 `(7,16)` remains a static mineral diagnostic rather
than ordinary path evidence. The 32-frame Route8 production keyboard walk
records authoritative positions `(15,6)`, `(15,8)` and `(15,10)`, active terrain,
zero fade and a build count fixed at one. Its final native and source views
and an inspected movement still are retained.

All 164 focused checks pass in `target/outdoor-land-stone-verified-tests.log`.
They cover closed stone volumes, planted plots, actual posed peak height,
outward winding and normals at square and rectangular scales, stable asset
selection across camera crops, continuous native pigment, day/night refresh,
unchanged footing and claims, matching sea-stone optical contacts, terraces,
water registration and renderer integration. The explicitly executed external
worker roundtrip completes all eight source regions. The production native
viewer builds in `target/outdoor-land-stone-native-build.log`. Final views and
the movement receipt are in `target/outdoor-land-stone-review/`; four original
comparison pairs remain in `target/outdoor-land-stone-before/`. The duplicate
candidate batch was removed. No pack or authored model asset was regenerated.

The previously plain barrier surfaces now read as weathered stone. Whole-route
acceptance remains open: regular barrier courses, dense close foliage, dark
rectangular horizon patches, bright buoys and elevated cutaway joins still
need improvement. The Cianwood control also exposes a thin bright mineral
join that requires closer review. The approved palette and 0.85 mountain relief
remain. These native checks do not establish browser visual acceptance.

### Broadleaf branch spread and finer solid leaves

Kanto route trees retain their three authored branch architectures and wide
height variation. Mature upper branches now spread with age instead of keeping
a narrow crown as the trunk grows. The upright architecture's mature foliage
starts lower over its forks. Young trees retain their higher crown placement
and original spread; the first unrestricted candidate crowded Route2's nearby
grass-patch viewpoint. Root flares remain fitted inside the native planted
plots. Heights, source placement, palette treatment and the lower 0.85 mountain
relief remain unchanged.

Each original oversized leaf site now carries four smaller closed pointed
blades. Their aggregate projected area matches the original, and the original
branch and foliage sites still determine the crown. Each blade has a raised
midrib, a closed underside and normals from its actual triangles. Leaves are
placed with their own proportions rather than stretching with the entire
tree's trunk. Branch normals use the inverse transpose of the actual radial
and vertical deformation. No scenery wall or opaque crown shell was added.

The shared library now uses three broadleaf branch models in four age classes,
chosen with the same age field as the actual trees. This keeps instance height
ratios within 0.60–1.70 across the sampled world grid. The bounded library holds
the 120 existing pine/ancient recipes and twelve broadleaf recipes in 144 slots.
Pine, ancient and formal park geometry retain their own builders. The additional
leaf detail costs 32 triangles per original leaf site, up from 20; the three
broadleaf prototypes contain 15,856, 25,718 and 35,065 triangles before placement.
These are shared meshes, not new meshes per planted tree.

Twenty final native low-angle views and all their same-state 2D counterparts
were personally inspected across Routes2, 8, 13, 14, 15, 20, 25 and 31,
Viridian and National Park. The review includes four opposing Route14 views,
multiple Route13 and Route2 views, two 22:00 controls, the pine and formal park
controls and the repaired distant Route20 border. All twenty paired receipts
preserve game state. The Route14 production keyboard walk runs 32 frames from
`(13,10)` through `(13,11)`, `(13,12)`, `(13,13)` and `(13,14)` with active
terrain and the build count fixed at one. Its final 2D view shows a trainer
attention marker; this is movement evidence, not an uninterrupted traversal
past that trainer. A requested Route25 point `(43,8)` was rejected as
unwalkable by the production shell; that capture uses the verified `(14,5)`
position instead.

All 105 focused checks pass in `target/outdoor-broadleaf-verified-tests.log`,
including rooted scale and determinism, projected leaf area, closed leaf faces,
outward winding, deformed branch normals, bounded shared instances, source
family identity, footing and renderer integration. The explicitly executed
external-pack worker check completes all eight source regions and preserves
the exact prepared output. Native compilation succeeds in
`target/outdoor-broadleaf-native-build.log`. Final views, native placement
traces and the walk receipt are in `target/outdoor-broadleaf-review/`; original
comparisons are in `target/outdoor-broadleaf-before/`. Two rejected geometry
comparisons remain in `target/outdoor-broadleaf-rejected/`. Superseded batches
were removed. No pack or authored model asset was regenerated.

The broadleaf silhouettes are fuller and the leaf edges are finer, but this
does not establish whole-route acceptance. Route2's close foliage still crowds
some views. Bare forks and repeated crown clusters remain visible; the pale
plain barrier rocks, square coast courses, dark rectangular horizon patches
and elevated cutaway joins also need work. Browser visual acceptance remains
unproven.

### Complete distant drawings and authored coastal swimmers

The distant green/black fragment in Route20 was native Kanto tree artwork,
not a floating actor. A temporary terrain-only material probe established
that distinction. The retained source window cut through four-by-four source
blocks, and the nearest-border selection could assign different maps to cells
of the same block. Aligning the window alone still left six unclaimed tree
cells at a diagonal source boundary. Unknown continuation now chooses one
border owner per complete source block. Actual map and connection drawings
keep their original selection. The final Route20 trace has zero unclaimed
tree cells, and the native view contains complete 3D trees at that location.

Expanded 3D windows retain complete blocks with a fixed 120-by-120 extent,
up from 116-by-114: approximately 8.9 percent more source cells. Their explicit
origin is shared by live publication and destination preparation. Camera,
actors and traversal account for the resulting asymmetric padding. All source
phases retain the previous minimum halo without resizing the window on each
phase change. Classic scroll windows retain their existing size and origin.
The complete-grid validator now checks the published dimensions rather than
the previous constants. No scenery mask or filler wall was added.

Seven authored male swimmers on Routes40 and 41 share the variable sprite
token used by Olivine's rival scene. Fresh location previews can still carry
its initial rival bitmap. The model publisher and destination preparation now
use the pack's compiled `SWIMMERM` trainer header to identify those objects as
male swimmers. The adaptation requires the exact shared token, initial rival
assignment and trainer role; custom replacements retain their selected model.
Classic pixels, story variables and gameplay state are unchanged. An opt-in
human placement trace confirms Water support at -6 world units and the
existing 60.0576-unit immersion for both affected Route40 objects. Land actors
remain at zero support and immersion.

Eighteen final native views and their same-state 2D counterparts were
personally inspected across Routes13, 20, 31, 36, 40, 41 and 45, Cianwood,
Olivine and Cerulean Gym. Seventeen use true eye height, two use 22:00 paint,
and all eighteen receipts preserve state. Route45 `(7,16)` remains a mineral
plot diagnostic, not walking evidence. Its two opposing views preserve the
lower 0.85 mountain relief, rising centers and approved palette. The adjacent
swimmer checks start in Surf; the Olivine default position faces a shop wall
and is a publication control rather than a useful outdoor composition.

The Cianwood production keyboard walk runs 32 frames through `(10,20)`,
`(11,20)` and `(11,21)`. The Route36 walk runs 32 frames through `(30,8)`,
`(31,8)` and `(32,8)`. Both retain active terrain and a build count of one.
The first Route36 capture used the wrong camera-relative key and did not
translate; the final capture uses `ArrowDown` and proves actual movement.
Final views, traces and movement receipts are in
`target/outdoor-source-border-final-review/`. Three earlier diagnostic views
and the two six-cell traces remain in `target/outdoor-source-border-before/`;
superseded capture batches and the temporary probe shader were removed.

All 29 focused checks pass in `target/outdoor-source-window-verified-tests.log`,
including the actual seven compiled trainer roles, replacement guards,
classic actor art, preload identity, complete source blocks, authoritative
player placement, published grid metadata, exact movement at 60/30/9 Hz and
destination preparation gates. The movement regression accounts independently
for source-window padding before comparing the original interpolation bounds;
its displacement and cadence requirements are retained. The initial fixed-size
validator failure, rejected native frame and old camera-origin expectation are
preserved in the `outdoor-source-window-rejected-*` and
`outdoor-source-window-camera-origin-tests.log` diagnostics. Unoptimized tests
use `RUST_MIN_STACK=16777216`; the initial default-stack overflow remains in
`target/outdoor-swimmer-source-initial-tests.log`. The native viewer build
succeeds in `target/outdoor-source-window-native-build.log`. No pack was
regenerated.

These source and placement defects are corrected; whole-route art acceptance
remains open. Square shoreline courses and barrier rows, bright buoys, rough
broadleaf crowns and regular elevated cutaway cells still need improvement.

### Outdoor waterline and bank depth

The native half-pixel water recess left beaches looking like thin sheets.
Sea-level outdoor water now sits a quarter tile below dry ground, giving the
existing closed bank faces visible depth. A final monotone vertical fit keeps
dry ground and the moderate mountains at their existing heights. Shoreline XZ
coordinates, materials, palette colors, animation bindings and source gameplay
domains are retained. Faces whose shape changes receive normals from their
completed triangles, preserving the original hard creases.

Rooted sea stones, aquatic plants and authenticated native buoys lower rigidly
with the liquid. Resolved water-cell actor support lowers by the same amount,
preserving its existing immersion relative to the surface. Raised inland
water and its rooted scenery keep their own positive datum. Interior maps
remain outside this outdoor fit. The source profile continues to identify
native art; the completed mesh supplies the final rendered support.

The first elevated native review exposed a separate old-height ocean apron.
The retained water and distant continuation now receive the same final fit
before publication. A regression requires both surfaces to meet at the new
datum, preventing a bright rectangular horizon seam.

Twenty final native views and their same-state 2D counterparts were personally
inspected across fourteen maps: Routes19, 20, 21, 31, 40, 41 and 45, Cianwood,
Cherrygrove, Olivine, both ports, Cerulean Gym and Dragon's Den. Eighteen use
true eye height; the other two are an elevated Route20 control and a Cianwood
keyboard walk. Two captures use 22:00 source paint. All twenty paired-state
receipts preserve state. Route41, Cherrygrove and Olivine start in Surf, as
their source images and actor receipts show; they are water-level composition
checks, not dry-ground walking evidence. The Route19 and Route45 mineral
positions likewise remain static geometry diagnostics.

The Cianwood walk uses 32 production keyboard frames and reaches `(10,20)`,
`(11,20)` and `(11,21)` with the terrain build count fixed at one. Its final
source image places the trainer on the dry shoreline. The eight-second
Route20 recording contains 240 native frames at 30 Hz; inspected samples show
foam changing around the seated rock feet without reopening the ocean apron.
The video is `target/outdoor-bank-depth-motion.mp4`; four inspected samples
remain in `target/outdoor-bank-depth-motion-frames/`. Final stills, movement
trace and paired receipts are in `target/outdoor-bank-depth-review/`. Six
additional baseline pairs are in `target/outdoor-bank-depth-before/`; the
rejected old-height apron is retained in `target/outdoor-bank-depth-apron-before/`.

All 176 focused checks pass in `target/outdoor-bank-depth-verified-tests.log`,
including the new bank/liquid closure, rigid water roots, actor immersion,
raised-pool and interior exclusions, matching ocean continuation, water,
piers, terraces, renderer integration and the explicitly executed eight-region
external worker roundtrip. The production viewer builds in
`target/outdoor-bank-depth-native-build.log`. No content pack was regenerated.

Visible bank depth is improved, but whole-route acceptance remains open.
Shoreline contours and barrier rows still follow conspicuous square source
courses. Some distant actors float or appear as fragments, distant broadleaf
forms remain rough, buoys are too bright, and the elevated walking view still
exposes regular cutaway cells. The current palette and 0.85 mountain relief
remain; none of these remaining defects is accepted as finished art.

### Coastal stone placement and damp native banks

Kanto sea stones now vary their heading, horizontal proportions, offset and
height within their original source plots. The same deterministic transform
places the closed mineral shell and its underside water-contact triangles, so
the foam mask follows the actual stone. Normals use the world-space inverse
transpose, including rectangular tile scales. Water beds, source collision and
walking support retain their existing positions.

The first placement candidate made the nearest stones too upright. The final
range is broader and lower: nominal rise is 1.25–1.85 tiles, centered on the
previous 1.55, with restrained offsets and full rotation. Rotation fits the
actual sculpted envelope into its original plot rather than shrinking a square
around the model. The three existing closed shoulder forms remain shared;
there are no additional models or scenery entities. Land barrier posts and
the lower mountain relief of 0.85 remain unchanged.

Kanto's authenticated natural sand and native bank caps now receive the damp
shore transition. Treating only ordinary sand missed the separate bank cap at
the actual waterline. The wet target scales the received pigment, retaining
the source day/night hue instead of introducing a daylight brown at night.
This changes color, not shoreline geometry or elevation. The production bank
cap regression verifies that the transition reaches the water-facing vertices
while preserving positions, indices and footing.

Fifteen final native views and their same-state 2D counterparts were personally
inspected across Routes20, 21, 19, 13 and 31, Cinnabar and Cianwood. They include
multiple true-eye headings, an elevated Route20 view, two 22:00 views and inland
controls. Every paired-state receipt preserves game state. Route21's position
is on the authored pier. Route19 and the elevated Cinnabar mineral position
are static geometry diagnostics, not evidence of walking those approaches.
The location tester accepts explicit positions; a `chris` actor alone does not
authenticate walkable ground. Final evidence is in
`target/outdoor-coast-balanced-review/`; eight before views remain in
`target/outdoor-coast-pose-before/`. Superseded candidate captures were removed.

All 147 focused checks pass in
`target/outdoor-coast-balanced-verified-tests.log`, including deterministic
placement, closed-volume bounds, exact rendered/contact-mask agreement,
native bank paint, water, terraces, footing, renderer integration and the
explicitly executed eight-region external worker roundtrip. The native viewer
build succeeds in `target/outdoor-coast-balanced-native-build.log`. No content
pack was regenerated.

The coast still needs further work. Beach land has a thin physical edge; the
new damp paint does not give it depth. The authored straight barrier rows are
still evident despite varied stones. Route20's distant green/black fragment,
Route21's rough distant tree/fog forms and Cianwood's bright buoy courses remain
visible. This pass preserves the approved palette and moderate mountains;
it does not establish whole-route visual acceptance.

### Route pine branch volume and age-matched instances

The shared pine library grouped trees by an older random height recipe while
their actual stems used the newer age field. A bounded source calculation over
32,768 sparse/dense placements found prototype vertical scales from about
0.227 to 5.027. That could flatten a mature crown into a young tree or stretch
its needle bundles into long upright blades. Pine prototype selection now
uses the same age field as the actual tree. The executed Rust regression gives
0.620746 through 1.516377 on the same placement range. The requested stem-height
variation, source roots and plot bounds remain; ancient and deciduous trees
retain their original prototype seeds.

Needle fascicles now spiral around the complete twig instead of occupying two
shallow opposed planes. Each spray retains its ten closed bundles and forty
triangles. A full-orbit geometry regression checks all four radial sides and
projected area through sixteen views, including the supported vertical scale
range. Young trees keep their lower crown layer; mature trees retain sparse
lower trunks and shortened woody limbs. The existing crown test now measures
those woody limbs separately from needles hanging down from the crown.

Thirteen final production views were personally inspected on Routes31, 36, 37,
32, 13 and 20, including multiple true-eye headings, a high Route36 view and
two 22:00 controls. All thirteen paired 2D receipts preserve state. Evidence is
in `target/outdoor-pine-depth-review/`; the eight before views are in
`target/outdoor-pine-orbit-before/`. Route36's production keyboard walk changes
six authoritative positions through 32 input frames with no additional terrain
build. Its final screenshot retains the trainer aperture and surrounding crowns.

All 170 foliage, shared-library, reveal, batch, renderer and worker checks pass
in `target/outdoor-pine-depth-verified-tests.log`, including the explicitly
executed eight-region external worker roundtrip. The library still has 120
pine/ancient combinations and three deciduous meshes within its existing bound.
The production viewer builds; no content pack or model bundle was regenerated.

This improves branch volume without establishing whole-route acceptance.
The closest needle bundles remain broad and angular, and the coast controls
still show thin land edges, repetitive boulder rows and bright buoy courses.
The lower mountain height and current palette are retained.

### Connected-route mineral coordinates

Connected scenery keeps material coordinates in its owning map, while its
geometry is placed in the active map. Mountain filling and the later terrace
fit now recover that placement from the frame's authenticated source regions.
Both producers use the same coordinate helper. Material UVs and palette recipes
are retained, as are the current 0.85 relief scale and source walking domains.
This closes the large distant openings in Route45's elevated view. Cianwood's
largest proposed source-grid displacement falls from about 1,946 to 36.8 world
units, within the existing local shoulder envelope.

Seven production views on Route45, Route42, Route26, Route44, Cianwood and
Cinnabar were personally inspected; all seven paired 2D receipts preserve game
state. Captures and the bounded placement trace are in
`target/outdoor-connected-mineral-review/`. All 169 focused and external-pack
checks pass, including a mixed-coordinate fixture that requires identical
fitted geometry, colors and walking support for active-map and connected-map
material registration. The log is
`target/outdoor-connected-mineral-verified-tests.log`. The full-region worker
roundtrip was not run in that pass; the entrance verification below includes it.

The subsequent entrance review exposed a separate Ice Path defect above the
tunnel. Complete Johto entrances lacked Kanto's weathering protection, and
independently fitted portal columns could separate the closed arch. The
entrance work below addresses that additional defect.

### Closed entrances at moderate mountain height

Complete native Johto, Johto Modern and Kanto cave drawings share the same
weathering protection. A complete entrance retains its source footprint,
threshold and dark vertical recess while its connected roof fits the adjoining
mountain. The dark lining extends inside the roof; it cannot expose a separate
black horizontal cap. The ordinary per-column mountain deformation skips the
closed portal volume, which instead uses one continuous vertical mapping.

Scaling the entire entrance with the mountain made Ice Path a narrow, tall
slit. Its existing opening now retains its useful height while only the rock
above it grows to meet the mountain. Short native drawings need standing
clearance: their roof receives a local shoulder that tapers into adjoining
mineral over two tiles. The standing opening is bounded by the closed roof.
Source walking floors stay seated; neighboring accessible mineral uses the
same shoulder field for its triangular surface and within-cell footing.
The route-wide relief remains 0.85 and the current palette is retained.
The original sky tears are recorded in `target/outdoor-cave-front-review/`;
`target/outdoor-seated-portal-review/ice-path-approach.png` retains the rejected
whole-entrance stretch for comparison. Superseded intermediate captures were
removed after the final review.

Thirteen final production views were personally inspected on Route42, Route45
and Blackthorn, including front and side views at true eye height, trainer-scale
controls, elevated views and Ice Path at 22:00. All thirteen same-state 2D pairs
preserve game state. The final captures are in
`target/outdoor-integrated-portal-review/`. Mount Mortar and Dark Cave now have
standing openings rather than low crawl holes; Ice Path retains its original
useful opening height. The 32-frame production keyboard walk enters IcePath1F;
the settled interior screenshot and paired-state receipt confirm the completed
warp rather than relying on the CSV's intermediate fade-in frame.

All 171 focused and external-pack checks pass. The final log is
`target/outdoor-integrated-portal-final-tests.log`. The entrance regression
exercises short and tall roofs on level and raised ground, tests front-facing
roof closure and standing headroom, and verifies real approach-path triangles
and footing beneath the overhead rock. Neighboring walking caps retain their
triangle/footing agreement. The external footprint check now distinguishes
authenticated cave-roof influence from walking ground; the earlier failed
zero-ceiling assumption is retained in
`target/outdoor-integrated-portal-verified-tests.log`. All nine external-pack
checks were explicitly executed, including exact worker/direct prepared-output
roundtrips on eight source regions. None of the selected checks remains skipped.

The broader art goal remains open: Mount Mortar's oblique shoulder still exposes a
thin straight mineral strip, some cliff courses remain too regular, coastal
land edges remain thin and close pine sprays still need more volume. This work
does not establish whole-route visual acceptance.

### Mountain interiors and moderate relief

The perimeter-only candidates are rejected. Native plateau art often carries
ordinary or directional land permission, so excluding walking mineral from
the mountain field left low interiors behind rising sides. The mountain body
now includes those tops. Connected cap/wall columns rise together, and
accessible mineral footing follows the actual triangular upper surface.
Paths, cave mouths, planted plots and water retain their source footprints.

The subsequent very tall Route45 candidate is also rejected. Added relief is
reduced from 1.85 to 0.85 times the interior distance field. The current color
and day/night treatment remain. Wall chips retain the original wall plane in
their source coordinates so every height ring can find its matching cap.
Material tags also need the actual geometric position to disambiguate them:
large retained route grids cross their 10,000-unit tag ranges. Decoding the
tag by threshold alone could pull a restored cap 20,000 units across the map.
Connected scenery also retains its own map's material coordinates. Those
coordinates cannot place a face in the active map. The production trace found
proposed displacements of about 1,946 world units in Cianwood. Source-grid
recovery is now limited to the existing local shoulder envelope; distant
material registration cannot move the geometry. This closes the stretched
bands in both Cianwood's low coast view and the user's Route45 view.
Ordinary walking follows the mountain slope; actual source ledge jumps retain
their existing arc and the runtime still owns all movement permissions.

The production native preview builds successfully. Twelve final native views
were personally inspected across Route45, Route44, Route26, Route19, Cianwood,
Cinnabar and MountMoonSquare, including true eye height and two night controls.
Ten same-state 2D pairs preserve game state. Final captures are in
`target/outdoor-mountain-lowered-final-review/`. Route45 `(7,16)` is the user's
rock-plot diagnostic, not ordinary path evidence; Route19 `(4,16)` is a water
view. The Route26 keyboard recording starts at `(9,28)`, changes authoritative
positions through 128 input frames and crosses raised mineral tops with no
additional terrain rebuild. The eight-second video is
`target/outdoor-mountain-walk-final.mp4`; retained sample frames and movement
receipts are in `target/outdoor-mountain-walk-final/`.

All 165 focused and external-pack checks pass. They include actual raised-cap
triangle/footing intersections, walking and ledge arcs, ground contacts,
mineral winding, source-coordinate reuse, source-tag boundaries, terraces,
water registration, renderer integration and worker metadata. The external
layouts include 1,681 shared Route44 cap/wall samples, 2,199 in Blackthorn and
the Kanto mountain controls. One existing worker benchmark remains ignored.
The verified log is `target/outdoor-mountain-final-verified-tests.log`.

This is a lower mountain candidate, not whole-route visual acceptance.
Route45's elevated view still exposes distant cave/land joins; some connected
faces retain straight shelves, coast edges remain thin and ordinary rounded
land posts remain too uniform. Earlier records below describe their respective
revisions; the policy that flattened walking mineral is superseded here.

### Coastal rock shoulders and rock-local wash

Kanto boundary shore rocks retain their closed base and source footprint but
use three unequal shoulder envelopes. Their source pigment and night palette
remain; weighted shared normals describe the closed volume. Ordinary land
posts retain their separate geometry. The shared sea-stone material now receives
the retained grid registration even when the animation phase does not advance,
keeping its painted mineral detail at the intended world scale.

Rock-local wave wash uses the existing shoreline distance field and received
water phase. The distance gradient identifies each rock's outward shore;
incoming crests spread broken white wash around exposed feet, with quieter
water on the lee side and a small reflected slope. The actual land-depth field
still owns shallow water. No separate clock, water height or collision changes
are introduced. The eight-second motion review is
`target/outdoor-rock-wave-motion-review.mp4`; Route19's final eye-height control
also retains the embedded production shader. These changes do not resolve the
remaining ordinary post silhouettes or thin coastal land edges.

### Coastal buoy volume and the water horizon

The bright blue/white fragments along the Johto buoy courses came from open
collars with contrasting patches across their upper faces. Each float now has
a complete rounded crown and a quiet blue lower band, using the same native
paint samples and footprint. Smooth modeled normals describe its volume. The
closed six-face construction uses 96 quads instead of the previous 128.
Source claims, water beds, support and the live time-of-day palette are retained.

Route40's pale patterned horizon was a separate continuation defect. Unknown
flat scenery could supply the repeated distant ground, while verified water
was excluded. Ground donors now require a known lawn, path or paving source.
On coastal frames dominated by authenticated animated water, with a wet outer
edge, four level continuation faces use the ordinary world's water material,
source palette and received animation phase. They are published after terrain
fitting, outside the finite source grid, at the existing water datum. No extra
water material, shader, collision or footing is introduced. Inland maps,
enclosed basins and unverified still drawings cannot acquire this ocean apron.

The production preview built successfully. Eleven final native views were
personally inspected on Route40 `(6,11)`, Cianwood `(10,20)`, Route41 `(3,22)`,
Cherrygrove `(9,8)`, Olivine `(34,31)`, Route19 `(10,11)` and a Route30 `(7,40)`
forest control. Eight same-state 2D pairs preserve game state. The coastal
views include true eye height, elevated controls and Route40 at 22:00. The
Route41, Cherrygrove and Olivine positions are surfing views, not walking-path
evidence. Route40's eye-level horizon strip is gone in both daylight and blue
night. The final receipts are `target/outdoor-coastal-continuation-review/`;
`target/outdoor-buoy-shape-review/route40-eye.png` retains the earlier horizon.

All 53 focused buoy, background, live-water, shoreline and worker checks pass;
one existing worker benchmark remains ignored. The verified log is
`target/outdoor-coastal-continuation-verified-tests.log`. It retains the initial
fixture failures and their passing repairs: the apron test now supplies known
Johto lawn, and the pier test uses Port08's full-width timber phase rather than
repeating Port14's left-only face across four columns. Those fixture repairs
change no production pool behavior. The temporary source-inspection example
and copied atlas images were removed; packs remain untouched.
The extended coastal check also proves that ordinary authentic-domain
partitioning marks every continuation triangle as outside the source grid,
so the ocean apron cannot become a battle surface.

The broader art goal remains open. The shoreline still has repetitive rounded
rocks, thin land edges and overly geometric rock islands; Route42's cave arch
still exposes joins. Some low pine sprays remain too flat. These buoy and
horizon repairs do not establish whole-route visual acceptance.

### Joined rock at native map borders

Route44 `(38,3)` exposed a long sky-colored slit across the northern mineral
mass and a distant hole through its cap. Coincident rock vertices could first
receive different sculpting decisions. The later terrace pass then sampled
adjoining synthetic and native caps on different sides of the map boundary,
separating an otherwise joined edge. Sculpting now evaluates each shared
source sample once. Terrace fitting samples the same source point on continuous
mineral land, retaining owning-side behavior at actual authored drops, raised
water and cliff/bank contacts. Walking support and source ownership stay intact.

The final native render closes both Route44 openings. All four cardinal
eye-level views, three third-person headings and controls on Routes 42 and 45,
Cianwood, Route19 and the protected Route26 ledge were personally inspected:
15 native views across six maps, including Route42 at 22:00. Nine paired 2D
captures preserve the same game state. The final receipts are
`target/outdoor-route44-final-rim*` and `target/outdoor-joined-rim-review/`;
`target/outdoor-route44-seam-before*` records the initial defect.

All 117 focused checks pass: 109 terrain, rock, terrace, datum and worker
checks plus eight explicitly executed external-pack regressions. The Route44
pack check exercises 1,681 shared cap/wall samples and 107,136 cap triangles.
A separate border fixture proves both a joined cap and retained authored
drops. Raised-pond ownership, tilted mineral walls, ordinary footing and the
directional mountain walking cells retain their existing checks. One worker
benchmark remains ignored. The verified log is
`target/outdoor-joined-land-verified-tests.log`.

These repairs close visible terrain gaps, but the broader art goal remains
open. Route42's cave arch has exposed joins in eye-level views. The coastline
still has repetitive rounded rocks and thin land edges. The subsequent buoy
and horizon corrections are recorded above. Broad mountain beds and some pine
sprays also retain overly regular silhouettes.

### Route forest heights and low views

Route tree blocks need overlapping crowns at different heights, including
young trees below the mature canopy. The former Johto range still left long
open corridors between equally tall bare trunks. Ordinary pines now span
approximately 2.6–16.5 source tiles at full scale; ordinary Kanto broadleaf
trees span approximately 2.6–12.1. The coordinate-seeded distribution favors
shorter trees while retaining tall leaders. Direct meshes and shared runtime
instances use the same height recipe. Existing tree models, planted positions
and blocked plots are retained. Ancient and formal park trees keep their
separate scale policies. The rejected forest fill and smooth bough blobs have
been removed; neither is part of this implementation.

The final height mix was personally inspected in 11 offscreen 960×720 noon
views on Routes 2, 30, 31, 38 and 43. Every route includes a true eye-level
capture; Route30 and Route2 also include elevated views. Five same-state 2D
pairs preserve the game state. Evidence is in
`target/outdoor-height-variation-review/`, particularly `route30-eye.png` for
the user's open-trunk sightline. Shorter crowns now overlap beneath taller
leaders and substantially reduce that horizontal opening. Some needle sprays
still read flat in close low views; this height correction does not establish
whole-route visual acceptance.

The native preview built successfully. All 100 focused checks pass, covering
foliage, shared instances, cached groves, local reveal, grass, understory and
worker metadata; one pre-existing worker benchmark remains ignored. Mature
crown checks retain their shape requirements for mature trees, while young
trees are checked for complete leaders, stable roots and valid geometry.
The verified log is `target/outdoor-varied-height-verified-focused-tests.log`.

Foliage reveal now clears the trainer locally rather than dissolving whole
crowns far above the trainer. Overworld grass uses a 0.45 default height
multiplier; explicit battle normalization retains its existing scale. These
changes address the earlier route views in which grass hid the trainer and
whole-tree fading opened artificial holes in the forest.

The wider review also covered Routes 12, 17, 19, 38, 42, 44 and 45, Cianwood,
and ordinary route forests at true eye height. It exposed remaining coastal
defects: thin shore edges, repeated rock/post silhouettes and patterned distant
water edges. Route44's border seam is corrected by the joined mineral mapping
above; the water-horizon correction is also recorded above. Thin shores and
repeated coastal rocks remain open. Requested low third-person pitch can
be raised automatically by
terrain clearance, so first-person captures are required to judge sightlines
at trainer height. A spawn inside a tree block or water is a diagnostic view,
not evidence of normal walking composition.

### Original feel in 3D and protected walking caps

The first neutral granite candidate drifted away from the original feel.
The corrected surface shader keeps its fractured painted detail, restores
warm ochre pigment, adds restrained shelf lichen and lights the actual 3D
faces. DayCycle retains the original blue night tint. Source-painted materials
keep their existing protection against a second night tint. The original
bitmap and its generation prompt are in `assets/terrain/README.md`; cave
bedrock keeps its separate image. No pack content changed.

Decorative relief previously buried the trainer at Route26 `(9,28)` and
`(10,28)`. The external pack identifies those cells as blocks `6a` and `70`
with permission `b2`: directional UP_WALL land, rather than impassable rock.
The existing complete-source terrace datum now carries those accessible
mineral cells through native and browser-worker preparation. Mountain relief
treats them as openings, preserves their walking support and retains distant
inaccessible ridges. The runtime still owns all movement permissions.

The optimized native executable built successfully. All eight Route26
headings at `(9,28)`, noon, zoom 0, pitch 35, 960×720 were personally inspected:
the trainer stands clear on the ledge throughout the orbit. The lower view
at `(10,28)`, heading 5, pitch 20 and the eye-level view at `(10,27)`, heading
0, pitch 5 were also inspected. Same-state 2D pairs preserve position, facing,
clock and game state. Receipts are ignored verification output:
`target/outdoor-og-feel-orbit/route26-{0..7}.png`,
`outdoor-route26-og-feel-protected*`, `outdoor-route26-og-feel-low-retry*` and
`outdoor-route26-og-feel-eye-level*`. The first low-view capture timed out;
its completed retry used the existing 300-second capture allowance.

The production MountMoonSquare `(20,12)` pair at 22:00, zoom 1, heading 0.5,
pitch 45 retains blue-purple night and warm cabin windows. Accessible upper
rock remains a plateau and its cave entrance is visible. Its paired game
state is preserved (`target/outdoor-moon-og-feel-production-night*`). Earlier
shader-only noon/night studies remain material diagnostics. The Route31
`(14,9)` control pair at noon, heading 5, pitch 35 also preserves game state.

All 76 focused checks pass: 37 terrain and worker checks, four explicitly run
external-pack regressions, five clock/lighting checks and 30 render API checks
(one pre-existing API test remains ignored). Coverage includes native source
ownership, camera crops, connected origins, protected walking support, retained
ridges, bounded worker pages and the external pack's actual `b2` permissions.
The lighting integration fixture now publishes an outdoor map; its former
empty frame exercised the indoor fallback. Logs are
`target/outdoor-og-feel-{focused,external,clock,api}-tests.log`.
The focused log retains the initial empty-frame lighting failure; the clock
log records its corrected fixture and passing rerun.

Whole-route visual acceptance remains open. The orbit still exposes overly
sharp ridge shapes and a gray surface at the water edge in heading 2. Route31
and the Route29 control retain busy needle foliage and dense tall grass.
Walking views, further route families and those remaining art defects still
need work. Successful material and walking-cap reviews do not complete the
outdoor art goal.

## Outdoor mountain relief and mineral contrast — 2026-10-16

Outdoor permanent rock now has unequal planar ridges and broken spurs, with
cooler mineral beds and lighter seams across the existing layered faces.
Cinnabar uses a grayer volcanic finish and now reaches the outdoor mineral
producer; Route10North/South now reach the mountain relief pass. The relief
changes only vertex height and pigment, retaining horizontal coordinates,
triangle counts, walking support, ground contacts and cave openings. Cave
interiors, grassy jump ledges and tree support geometry remain unchanged.

Final production 2D/native pairs were inspected at noon with matching position,
facing and preserved game state: Route10South `(8,5)` outside Rock Tunnel toward
Lavender, LavenderTown `(3,2)`, CinnabarIsland `(9,6)`, and CianwoodCity `(10,20)`
on the open shoreline path. Broad fractured faces remain readable; small peaks
are restrained, the cave mouth stays open, and Cianwood retains its beach width.
Route29 `(20,7)` is the control for the existing grassy banks and vegetation.

All 52 focused regressions pass: 6 mountain tests, 24 outdoor mineral tests,
21 terrace tests and the Kanto outdoor material regression. This includes the
external pack's Lavender, Cinnabar, Cianwood and Route29 footprints, Route26
relief, the wider route cap-winding checks and Blackthorn's shared contacts.
Packs and generated verification output are not source deliverables.

Reproduce a matching-state pair with the completed local pack:

```sh
cargo run --locked -p crystal-bevy --features location-tester \
  --example render_at_location -- --pack content-packs/core-modular.crystalpack \
  --map Route10South --x 8 --y 5 --hour 12 --view both \
  --screenshot target/route10-cliff-review.png
```

## New Route22 perimeter defects — 2026-10-14

All eight perimeter source/native pairs are inspected with preserved state
under `target/outdoor-route22-perimeter/`. East retains pale path-cell patches;
south and southeast cliffs hide most of the trainer. Some shallow caps and
brick courses retain flat source ink, and outer rock plateaus lack visual
interest. The path classification now includes blank Kanto23 as well as
speckled39, matching their shared source ground; its regression and all7
focused Kanto ground tests pass. Final production day/morning7/night22 east
pairs are inspected with preserved state (`target/outdoor-route22-quiet-path-final-*`):
the pale day / bright blue night patchwork is gone, leaving a continuous path.
This scoped repair is verified. Night brick courses remain too royal blue;
some shallow caps retain flat ink and foreground cliffs still hide the trainer.
An exterior body-reveal experiment was rejected and removed: it exposed sky
through the rock, while its tightened boundary clipped boots. This route is rejected
pending those repairs; perimeter coverage is evidence, not acceptance.

## Buoy cards and legacy bank seams removed — 2026-10-14

Final production pairs at Route41 close/wide and Cianwood southeast are
personally inspected against matching2D, all with preserved state. Rounded
low native floats replace the omitted buoy families, and both textured cards
and old solid brown bank seams are gone in these views. The whirlpool funnel
and mostly submerged female swimmer remain readable. All6 buoy regressions
pass; source audit covers all152 formerly raster cells. Receipts:
`target/outdoor-buoy-solid-final-*`. Water remains too teal against the blue
source; broad rock caps and angular coasts are still rejected. Route46's two
distant ground gaps remain unresolved. Route22 perimeter coverage is underway
to extend inspection beyond the current ocean and cliff fixtures.

## Cliff contacts closed in reviewed views; grove and ocean fixes pending — 2026-10-14

The expanded cap/wall mapping passes15 terrace and21 mineral tests (one ignored).
Native build succeeded in13m58. Matching-state Route46 east and Blackthorn
south pairs are inspected: their large cliff-contact slits are closed. Two
distant Route46 ground gaps and Blackthorn's black roof faces remain open.
The Route46 grove underlay correction is now verified in production: all961
plots select height0, the bright forest raster strip is gone, and the paired
state is preserved (`outdoor-grove-datum-route46*`).
New Route41 QA verifies female swimmer submersion and exposes flat whirlpool
cards / placeholder buoys. The existing guarded native funnel/current producer
now supports exterior Johto whirlpools; all12 water tests pass. Combined build31008 succeeded in10m23; the fresh72-map/2457-jump audit has
zero errors. Remaining ocean raster is buoy art in omitted `$07/$3a` cases,
not a failed whirlpool clock guard. The expanded adapter and old-bank cleanup
pass6 buoy tests and are awaiting final native proof. Floats retain their
footprint, use a lower broader pale shoulder and thin submerged dark rim, and
add no vertices. See `outdoor-buoy-bank-cleanup-native-build.log`. See the latest handoff for receipts and exact pending actions.

## New route and city rejection fixtures — 2026-10-14

All eight Blackthorn perimeter pairs are inspected and state-preserved. They
expose roof and cliff occlusion, rock contact gaps and incorrect roof hues;
these remain open. Route46's fresh fallback-ownership pair retains both its
pixel strip and sky gaps. Its trace has no fallback tree-card events, ruling
that path out for this view. The background identity and fallback ownership
regressions pass, but neither is visual acceptance of the remaining strip.
The mineral corner correction now passes14 terrace and21 mineral tests
(one ignored). Its native build succeeded; fresh Route46 and Blackthorn pairs
remain visually rejected because sky gaps persist. Route46 clear-color pixel
count decreases7642 to6433 at the same pose, which is partial improvement, not
terrain acceptance. The global72-map/2457-jump audit has zero errors. Route44
now has guarded mirrored small-crown bindings with a passing four-variant test;
its production day/morning/night proof is now inspected and state-preserved,
with four more native tree cells verified by the refreshed source audit.
Route46 has zero unclaimed interior tree glyphs; only clipped outer-row art
remains. A new sculpted cap/wall mid-edge repro fails21.60 vs21.36. Its expanded
contact-edge correction now passes15 terrace and21 mineral tests (one ignored),
with native visual proof pending build34529. The next trace records native
grove ground source/height to locate the remaining bright strip. See the handoff for
receipts and exact current status.

## Flat small-tree strip repaired — 2026-10-14

Azalea's four flat `$3b` small-tree crowns are replaced through the shared native
grove pipeline, with exact phase/art guards and no source cards underneath.
Final day/morning/night pairs are inspected and state-preserved; source audit
confirms all16 newly modeled cells. The source-painted well now avoids the
second night tint and retains source purple stone. See the latest handoff for
receipts. Whole-scene acceptance remains open: new Cianwood perimeter checks
show roof/coast/water defects, and Route46 still exposes rock slits and a border
pixel-tree strip. A broader tree suite has three failures reported to the
outside-art owner; the focused new binding regression passes.

## Slowpoke Well native replacement — 2026-10-14

The complete flat exterior drawing is replaced by a guarded square stone
well, recessed courses, source-painted pale steps and an open shaft. Three
focused tests pass. The final source audit confirms36 native well cells, and
hour12/7/22 Azalea `(33,9)` pairs are inspected with preserved state. Circular
shape, hidden courses and stairs covering the aperture were rejected and
corrected before these final captures. Night grading still makes the well and
stairs too cobalt compared with the source; this is a material/lighting
rejection, not completion of time-of-day fidelity. See the handoff for final
receipts and remaining regional defects.

## Junction proof and new route diagnostics — 2026-10-14

Fresh preserved-state Route13 `(54,6)` and Route24 `(2,9)` pairs are inspected.
The Route13 `$78/$31` raster junction strip is replaced by the native mineral
bank, and the repaired graded brick cap remains clean. Route24 still exposes
a separate checker patch and pale water rim; whole shoreline acceptance stays
open.

New hour12 matching-state diagnostics cover Route36 `(30,8)`, Route37 `(9,9)`
and Azalea `(20,9)`. Route36's trainer is obscured by pine crowns, Route37 has
flat flowers, and Azalea's architecture/materials remain unfaithful. A fresh
Route36 fitted-pine pair confirms that bounds alone do not solve occlusion.
These rejection fixtures were shared with the active outside-art goal, which
owns further pine finishing. See the handoff for exact ignored receipts.

The nearly horizontal mineral-wall shoulder now has a source-axis/contact
regression, and all13 terrace tests pass. The full mesh suite passes780 tests
with zero failures and six ignored. The native shoulder build succeeded, but its inspected fresh Route26 pair
still shows tiny sky-blue slits. Horizontal wall-ring matching and shared-corner
remapping diagnostics are now under investigation; no continuity acceptance
is claimed. Slowpoke Well now has a guarded native replacement pending tests
and paired production inspection.

## Native brick decks and graded-bed correction — 2026-10-14

Kanto `$05`/`$4b` decks now use live source-painted staggered bricks and
water-facing masonry without changing LADDER permissions or footing. The
first native Route13 view exposed mortar triangles through the graded caps
despite green tests. Splitting the caps at the source terrain's diagonal
eliminates the overlap; the new folded-grade regression passes. The corrected
Route13 `(54,6)` source/native pair is inspected and state-preserved, with the
previous white patch visibly gone. The bridge walk still reaches `(8,14)`
from `(8,10)` in Route24. Fifteen new paired edge views in Route13/Route24 are
inspected and state-preserved; source drawings still expose repetitive stones,
pale trees and wrong gray post shafts.

The residual junction strip belongs to shoreline `$78`/`$31`, not brick `$4b`.
Its exact guarded coast extension passes the focused tests and current 774-test
mesh suite, with six ignored. Native rebuild/visual proof remains pending;
the live handle and receipts are recorded in the handoff. Whole exterior
acceptance remains open, including nighttime grading and remaining props.

## Native Kanto shoreline and new edge review — 2026-10-14

The complete native pond/coast drawings now replace the retained gold checker
caps and shallow water faces with source-painted mineral geometry. Exact art,
phase, floor/water geometry and ownership guards preserve unrelated scenery;
water levels, collision and footing remain unchanged. The finish covers 1,500
cells across 22 maps in the 77-map audit, with all 2,003 jumps still descending.
All three shoreline regressions and the full 764-test mesh suite pass; six
tests remain ignored. The production native build succeeds.

Five close hour12 pairs in Fuchsia, Route12, Route17, Viridian and Vermilion
are inspected with preserved source/native state. Fifteen distinct new edge
pairs cover Fuchsia and Route12; Route12's south and southeast targets resolve
to one walkable position. The checker stripe is gone and mineral contacts sit
at the original shallow water drop. Fuchsia morning/night pairs preserve that
geometry and follow the live pigment, but whole nighttime grading remains too
blue and the water too bright compared with 2D. Extensive brick surfaces,
gray/faceted posts, oversized/boxy scenery, pale trees and phase/view-dependent
missing flowers still fail whole exterior acceptance. Exact receipts and next
work are recorded in the outdoor-terrain handoff.

## Native Kanto paving and new edge review — 2026-10-14

Nine complete ordinary-FLOOR road drawings now use source-painted native
beveled flags and rounded curb junction inlays. No collision, input, standing
height or source footprint changes occur. Complete art/phase/geometry guards
preserve unfamiliar, cropped, claimed or elevated drawings. Morning/night
paint uses the live source palette. The first render's crowded tile grid and
dark junction backing were rejected; the refined flags are larger and the
junctions have pale centers, gray rims and shaded chamfers.

The refined Route11 `(10,7)`, Saffron `(18,7)` and Route17 `(8,16)` source/native
pairs are inspected and state-preserved. Production Route11 curb walking
reaches `(11,7)->(12,7)->(13,7)->(14,7)->(15,7)`. All eight Route11 and eight
Saffron perimeter pairs are inspected, with sixteen preserved-state receipts.
All three paving regressions and the full 761-test mesh suite pass; six tests
remain ignored. The 77-map audit retains 2,003 descending jumps and zero errors,
with 4,592 native paving cells across seven maps. Commands, coordinates and
receipts are recorded in the outdoor-terrain handoff.

Whole exterior acceptance remains open. Route11 hour7 follows the source
yellow temperature, but hour22 is overly cobalt blue versus lavender source
art and has poor character readability. The lighting task is archived; its
notice failed and it was not reopened. Saffron building scale/occlusion and
retained yellow brick strips, checker plazas, repeated posts/pale trees and
Route17's retained gold shoreline pixels still need work.

## Kanto boundary-stone and expanded edge review — 2026-10-13

The exact land, paving and shore boundary models now use cached geometry
fairing with pinned ground contact and smooth welded normals/pigments. This
removes the triangular cap patches observed after the first smoothing pass.
No triangles, game content or support changes are introduced; the closed-volume
and under-500-triangle checks pass. Current full mesh suite: 757 passed, zero
failures, six ignored. Native build and close paired Pewter `(2,18)`, Route15
`(20,2)` and Route19 `(10,11)` views are validated and inspected. Rounded volume,
smooth cap shading and seated contact are verified for these component views.

All eight Pewter and all eight Route15 edge pairs are also inspected, with 16
preserved-state receipts. Whole scenery acceptance remains open: flat paving,
retained cliff ink, sparse flowers, pale repeated ordinary trees, northern
building occlusion, mismatched shoreline edging and separate angular capped
posts remain. Exact locations, receipts and next work are in the handoff.

## Current macOS terrain continuation — 2026-10-13

The old-host winding FAIL below is historical. Current production mesher tests
pass it along with the full 728-test mesh suite. The shared directed-terrace
renderer now follows the human's permitted-jump high-to-low rule, including
consecutive courses and joined corners. Current paired Route6/Route4 movement
views confirm lower landings and closed cliff/corner contacts. Route8's alternate
underground entrance is now modeled with its live cyan-blue roof; Cerulean
Mart and Route6 flat roofs now match the source gray. Cerulean's eight
perimeter pairs are all validated and inspected. The shared-grove cache now preserves exact source coordinates, directional
neighbors and completed bank geometry; both exact preservation regressions
pass. The latest combined directed-terrace, foot-boundary and directional-face
snapshot passes 753 mesh tests with zero failures and six ignored. Actual jump
position auditing exposed additional opposing-course, landing-boundary and
tower-course-end defects; those now have focused regressions. The compiled outdoor audit verifies all 2,003 jumps in 77 maps, and the
broader app audit verifies all 2,457 jumps in 72 authored-jump maps, both with
zero errors. The tower jump contact is visually inspected; its retained pixel
floors and floor/wall distortion remain unaccepted. The final timber-support
material pair is inspected and confirms removal of the outdoor grass rim. Viridian's eight additional standable
perimeter pairs are inspected and expose remaining paving, flower and fence
shape defects. These are
concrete improvements, not acceptance of every exterior. Current receipts,
paired images, outstanding source-road borders and city-edge coverage are in
`docs/handoff/outdoor-terrain.md`; keep the full visual scope described below.

## Final rock normal pipeline — 2026-10-12

The actual production order explained the weak fracture-normal result: `rock_bases::round` recomputed smooth normals for every tagged rock after the cap sculptor had assigned fracture-aware normals. The final renderer therefore erased those cap distinctions whenever a native base corner qualified. Geometry stages now compose through `outdoor_stone::finish_rock_geometry`: sculpt mineral rims, round native base contacts, then calculate mineral normals once. Source cap/wall classification remains intact until that final calculation. The existing 25-degree crease rule is retained; the brief tighter-threshold experiment was superseded by this integration correction before visual acceptance.

A new regression calls that same production finishing function with a native rounded base and two joined fractured cap planes. The original normal/contact version passed, along with 19 stone tests and six base-contact tests. The extended winding check FAILED, including its corrected per-source-tile topology rerun (`target/outdoor-final-rock-pipeline-native-topology-test.log`): the combined sculpt/base pass inverted a cap triangle at positions (9.220304,4,14.89865), (8.822326,5.3333335,13.378186), (8.905096,4,13.626882). The failing regression is retained as an explicit unresolved blocker; earlier passing tests do not establish the combined geometry is correct.

The corrected native build completed in 12m16s. Its muted Route26 45-degree capture completed in 202.16s and was inspected (`target/outdoor-route26-final-rock-pipeline-45-muted.png`): fracture shading survives, but cliffs remain swollen and lumpy, and floor-Y seam coverage is incomplete. The muted Lake clearing capture at (23,30), orbit6/135 degrees completed in 228.83s and was inspected (`target/outdoor-lake-final-pipeline-clearing-135-muted.png`). The cyan roof has quieter curved courses and this angle leaves the clearing visible; the straight shore, rectangular water glints and repetitive crowns remain below the target. This angle does not resolve the intrusive canopy seen at 45 degrees. A prior Lake attempt terminated before producing an image and supplies no evidence.

The user and coordinating agent requested a transfer checkpoint. No further improvement cycle follows this diagnostic. See `docs/handoff/outdoor-terrain.md` for ownership, restart commands and unresolved work. The whole outdoor goal remains incomplete.

## Route26 fresh shape review — 2026-10-12

The muted current native/source pair at Route26 (10,28), orbit2/45 degrees completed and was inspected (`target/outdoor-route26-current-shape-45-muted.png`, 1280x960, 83.08s). A second native side view at orbit4/90 degrees completed and was inspected (`target/outdoor-route26-current-shape-90-muted.png`). The bank caps visibly rise toward the drop in the side view, but the broad earth steps and rounded, lumpy mineral caps remain below the requested standard. Neither view establishes that every reported floor-Y seam is fixed.

The stone normal pass previously averaged every adjacent mineral face at a shared position, rounding the lighting across actual fractured geometry. A new mineral-cap crease threshold retains distinct normals across cuts above 25 degrees and shares angle-weighted normals across shallow source seams. The first wider run caught unwanted sharp shading at cap/wall shoulders; those shared native shoulder junctions now retain the previous smooth average. Rock-base smoothing retains its existing behavior. Positions, indices, source ownership and floor datums are unchanged by this pass. The new regression passed both sides of this distinction and exact geometry preservation. All 19 corrected stone tests now pass, including cap/wall shoulder continuity, cap winding, raised-floor support, closed corners, tree seats and the new fracture-normal check (`target/outdoor-mineral-crease-stone-tests.log`). All seven current ledge-approach tests also pass (`target/outdoor-route26-current-bank-tests.log`), covering the closed sawtooth reset, floor/cliff joins and shared cap normals. These are structural checks, not verification of all visible floor seams. The native build completed in 9m42s. The muted 45-degree native capture completed in 31.22s and was inspected (`target/outdoor-route26-mineral-crease-45-muted.png`, 1280x960). Its appearance is almost unchanged; the normal correction alone does not resolve the lumpy caps. The frozen capture executable was removed. Visual acceptance remains pending.

The next geometry pass removes the short-period secondary height ripple and enlarges the primary linear mineral bed field. Cap depth now samples the original shared source coordinate rather than a bevel-displaced coordinate, keeping the large bed planes from bending with the shoulder. The existing support bounds, closed corner fields and tree-seat fades remain in effect. All 19 stone tests pass with this geometry (`target/outdoor-broad-mineral-plane-tests.log`), including resolved clefts and closed/support-preserving cap/wall joins. The native build completed in 1m43s. The muted native 45-degree capture completed in 52.25s and was inspected (`target/outdoor-route26-broad-mineral-planes-45-muted.png`, 1280x960). Broad cap slopes and silhouette changed, but painted fine bumps still dominate the scene. The frozen capture executable was removed. This is not visual acceptance.

Inspection of the shared bedrock asset and shader found outdoor caps sampling it at 40 world units while regular cave rock uses 144. The outdoor top projection now reuses that existing 144-unit scale; vertical wall sampling, asset identity and cave rendering remain unchanged. The native build completed in 2m08s (`target/outdoor-broad-cap-paint-native-build.log`). The muted 45-degree capture completed in 26.92s and was inspected (`target/outdoor-route26-broad-cap-paint-45-muted.png`, 1280x960). The larger cap slopes and less dense painted detail are visible, but the rock tops still read as rounded humps. This is a diagnostic checkpoint, not reference-level acceptance. The frozen capture executable was removed. Further work must address the remaining rounded cap form, ledge presentation and all-map floor-seam verification; the whole outdoor scope remains incomplete.

## Damp grass-to-water blending — 2026-10-12

The ordinary soil lip alone still leaves a straight, abrupt lawn/water transition. A new source-scoped shoreline color field blends a narrow irregular strip of actual Johto lawn toward the same existing soil pigment used by the bank face. It uses stable map coordinates, finite edge segments and cached local candidates. It creates no alternate asset, changes no source occupancy, and does not paint water interiors, unknown atlases, indoor routes, raised surfaces or raw tree drawings. Shared vertices exactly on a water-side grid boundary retain the land bank color, avoiding a green seam at the half-open cell boundary. Nearby lawn tessellation is refined only to resolve the color band; the native floor and its corners remain fixed.

All three new regressions pass: bounded/continuous color with viewport translation and source-grid scrolling, rejection of unsupported contacts, and production finish preservation of native floor corners, footing heights and authored claims. The five existing path/lawn blending tests and both shared soil relief/perimeter/winding tests also pass. The native build completed in 4m26s. The muted paired LakeOfRage (23,30), orbit6/135-degree, zoom1 review completed and was inspected (`target/outdoor-lake-damp-shore-blend-135-muted.png`, 1280x960; 47.15s 3D write). A narrow softened brown transition is visible at the grass/water boundary, replacing the abrupt green edge; the geometric shore remains straight and rectangular water glints remain. This verifies a modest color-blending improvement, not whole-scene acceptance. The frozen capture executable was removed after completion. The previous soil-lip screenshot predates this color field.

## Remaining outdoor acceptance — current scope

The requested goal covers the whole outdoor presentation, not only the newest cabin or a passing terrain regression. Treat the supplied forest remake image and saved Park concept as visual targets. Current native evidence still contradicts completion.

| Requirement | Evidence needed before acceptance | Current state |
| --- | --- | --- |
| Reference-level outdoor color, texture, detail and blending | Inspected native scenes compared with source 2D and reference art | Lake views still show intrusive translucent crowns and coarse repeated foliage; incomplete |
| Route26 saw-blade ledges and consistent floor Y | Fresh 45/90-degree views plus exact source cap/contact checks | Fresh 45/90-degree views inspected; side caps rise, but shape quality and all-seam verification remain incomplete |
| Stone, canyon and cliff shape/material quality | Source-scoped native route/city views and source comparisons | Color/foundation improvements exist; broad visual acceptance outstanding |
| All New Bark-to-Cherrygrove outdoor areas | Current native reviews covering representative transitions and source layouts | Earlier scoped audits are not current whole-scene acceptance |
| National Park modern remake | Saved reference retained; inspected source-faithful outer trees, benches, two grass types and centerpiece shape | Separate Park review outstanding |
| Lake of Rage and cabins | Both cabins, source roof colors, clearings, banks, water and lake scene reviewed at useful angles | Roofs/corner bindings, soil lip and narrow damp-edge blend visually verified; shore shape, water glints, canopy and creature coverage incomplete |
| Cherrygrove fake lake removal | Source override and current native/source view | Earlier scoped repair documented; confirm in current broad review |
| Consistent shared outdoor assets across routes/cities | Required canonical dispatch and current area coverage/render evidence | Required-loader/ownership checks passed; full visual coverage not proven |
| Preserve intended forest and distinct tree families | Current Ilex/source comparison and no unintended global substitution | Canonical root mismatch diagnosed; Ilex visual acceptance outstanding |
| Modern water appearance | Continuous water field in inspected native scenes, retaining source footprint and gameplay | Earlier Lake field is improved; appearance and blending not accepted across all areas |
| No silent alternate 3D asset | Required-loader and source-2D missing-coverage tests plus inspected scene | Scoped tests pass; Lake exposes missing creature coverage in permitted 2D |
| Muted reviews and worker coordination preference | Preview mute configuration; no unsolicited worker update/reply requests | Current captures explicitly muted; workers told to stop messaging this thread |

## Ordinary lawn shoreline soil — 2026-10-12

The exposed vertical wall from ordinary Johto lawn to water previously retained a cropped green checker texture. The production outdoor finish now reuses the existing shallow-bank soil-and-sod relief for those exact contacts. This adds no separate asset or soil palette. Recognition requires a named outdoor map, a same-atlas actual lawn source, an unclaimed native land cell, an adjacent actual water cell, and a complete vertical face from the original water datum to floor zero. Model/tree foundations, unknown continuations, altered heights and unrelated art are excluded.

The dedicated production regression passed for all four facing directions and eligible LakeOfRage/Route26/Route43/CherrygroveCity/NewBarkTown scopes. It proves unchanged exact perimeter corners, floor heights and authored claims, and rejection of route interiors, owned tree cells, shifted heights, unknown atlases and non-water neighbors. The bank remains closed at the existing datums. The dedicated native build completed in 3m37s. The muted paired bank review at LakeOfRage (23,30), orbit6/135 degrees, zoom1 completed and was inspected (`target/outdoor-lake-lawn-soil-bank-135-muted.png`, 1280x960; 111.98s 3D write). The ordinary lawn/water boundary now has a brown soil lip in place of the green vertical strip. The shore still reads as a straight cut and the water carries rectangular bright glints, so whole-scene shoreline blending is not accepted. The separate rock-ledged source corner remains its own drawing. The frozen bank capture executable was removed after completion. The earlier roof build began before this shoreline edit; its screenshots do not prove the new bank finish.

## Lake cabins and corner crowns — 2026-10-12

Muted paired source/native reviews found lavender panels on the northern Hidden Power cabin's charcoal/ochre roof, terracotta on the Magikarp cabin's cyan/blue roof, two flat source vegetation squares beside the northern approach, and intrusive translucent canopy overlap. Roof color is now sampled from the live source drawing and applied only to named solid roof-material vertices. Geometry, wall/timber colors, normals, UVs and source claims remain unchanged by recoloring. Traditional houses consume their individual source palette; the Lake Magikarp cottage binding is source scoped and does not recolor every cottage globally. Missing palette coverage retains exact 2D source drawing.

The first north paired corrected capture, `target/outdoor-lake-north-cabin-source-roof-45-muted.png`, proves the brown/ochre mapping but still shows broad flat roof panels and the old clearing gaps. The first Magikarp correction, `target/outdoor-lake-magikarp-cabin-source-roof-45-muted.png`, instead exposed a palette coverage bug: it retained 2D because the source roof uses four entries. The extractor now accepts that complete native palette, treating its brightest border separately from its field. The fresh muted Magikarp pair completed and was inspected (`target/outdoor-lake-magikarp-cabin-four-palette-45-muted.png`), proving the cyan/blue 3D cottage returned. Its roof still has loud alternating shade panels and is not visually accepted. Recoloring now derives a common field from the source mid/light swatches and keeps only a small difference for lighter courses, preserving the source hue while reducing the checker pattern; all nine cabin tests pass again after that shade adjustment; its fresh capture remains pending.

Source inspection of the supplied pack identified the clearing squares as complete Johto corner crowns in blocks $3c at (0,0) and $3e at (0,2). Both now use phase-aware 2x2 bindings to the unchanged shared grove generator and finished lawn underlay. The regression passes for LakeOfRage, Route43 and CherrygroveCity, proving one four-cell placement, flat footing, no retained sprite square after the production outdoor finish, and rejection of altered source phase. Source inspection helpers and their temporary output were removed.

The traditional-house panels now reuse the existing closed roof-course refiner, retaining named source paint. The refiner adds bounded curvature inside each panel while retaining its perimeter and lower geometry. It does not reshape trees. All nine cabin regressions pass, including the four-color source fixture and both cabin model recoloring invariants. All three roof-course tests pass, now including the actual traditional-house model, fixed perimeters, normalized normals, bounded positions and finite colors. The latest native build completed in 3m22s; its captured revision must be distinguished from the subsequent roof-course edit. The north clearing pair completed and was inspected (`target/outdoor-lake-north-cabin-corner-fix-45-muted.png`, 1280x960). Both square sprite remnants are gone and their exact intended corner trees are present, with the surrounding source layout retained. The broad translucent canopy overlap remains conspicuous. Temporary frozen capture executables were removed after both pairs completed. The curved-course/quiet-field native build completed in 10m00s. The muted north pair was inspected (`target/outdoor-lake-north-cabin-quiet-courses-45-muted.png`, 1280x960): roof courses are visibly curved, broad checker contrast is reduced and the repaired corner trees remain present. The large translucent crown overlay still obscures the approach. The wider Magikarp pair completed and was inspected (`target/outdoor-lake-magikarp-cabin-quiet-courses-wide-45-muted.png`, 1280x960): cyan/blue roof colors now form a quieter common field with curved courses. The canopy overlap remains, and the lake also shows a permitted 2D marker for missing creature coverage. The frozen roof capture executable was removed after both pairs completed. These scoped improvements do not establish visual acceptance of the entire Lake scene. The canopy overlap is unresolved, and the outdoor goal is incomplete.

At the user's request, all five other workers were instructed to stop sending updates, questions and coordination replies to this thread. Their independent authorized work continues; no further reply/status requests are issued here.

## Required assets and visible missing coverage — 2026-10-12

The user explicitly instructed deletion of fallback assets and permits original 2D to reveal missing coverage. All five active workers received that directive. `open_models::load_required` returns None for a missing or invalid selected asset, with no embedded alternate. Native required assets use an explicit root or the canonical checkout asset directory; browser required assets use the installed bundle. The old generic fallback loader is removed.

Deleted the old embedded New Bark tree, Johto tree LOD and world forest-conifer JSONs and their runtime catalog entries. Forest trees use the exact required tree/variant; a missing variant emits exact flat source tiles and reserves those cells against legacy voxel extrusion, without an authored claim or changed support. Ordinary canonical procedural grove trees were not reshaped. The old New Bark tree prototype generator and tree-LOD generation were removed; the grass LOD authoring tool retains only its intended grass output.

The interiors worker deleted five obsolete embedded alternatives and migrated their designated external families to the required loader. Missing interiors retain source art; the worker reports its scoped tests passed. The Pokemon worker removed battle-species substitutions with smaller overworld models and made arena trees optional exact-asset lookups. Other intended authored catalog assets remain their designated assets, not alternate substitutes.

The final voxel source check passed after the shared migration and exclusive-bundle correction (1m54s). Python storage/authoring tests passed (5), and all edited authoring scripts parse. An earlier loader-test compile was deliberately canceled because it predated the final selected-source semantics; a fresh current forest regression build uses unoptimized renderer test code while retaining dependency optimization. The production test binary subsequently passed all four required-loader tests and the missing-forest regression; the new production forest regression verifies source UVs, no solid/scenery substitute and unchanged support/claims in a fresh process with an explicitly missing root. Post-deletion native/browser review is still required. The canonical Ilex image predates this deletion and remains visually unaccepted: pale crowded foliage and transparency/card artifacts need further work. This is not completion of the outdoor goal.

## Shared asset coordination and Ilex rejection — 2026-10-12

The user rejected the Ilex Forest appearance and requested coordination with all workers. Redo Outside messaged all five other active Geothite workers. Outdoor tree geometry and map dispatch belong to this pass; lighting/effects owns the continuation of shared outdoor water appearance. The Pokemon scenery worker handed over its outdoor water mask/palette additions and keeps its source-scoped forest ledge finish and shore ecology. The cave worker retains cave geometry/materials and supported crown cutaway guards. Shared files must be edited narrowly and existing producers reused.

The user explicitly rejects preserving 2D water tiles as the 3D appearance. Source footprints, collision/events and animation identity remain input data; continuous water coloration, depth, highlights and movement must replace pixel atlas colors and rectangular glints. Earlier UV-preservation checks prove source integrity, not this visual standard.

Inspected saved Ilex final/forest-fixed views both show the same repeated tiered conifers; their difference is ledge finishing. No worker has reported replacing Ilex tree geometry in that pass. A possible source mismatch requires verification: the browser build bundles assets/open-models by default, while native loading requires CRYSTAL_OPEN_MODEL_ROOT. Forest dispatch substitutes the detailed Johto cutout tree only when that external source loads; otherwise it uses ForestConifer. The scenery worker confirmed its Ilex captures omitted CRYSTAL_OPEN_MODEL_ROOT. A muted native capture at Ilex (15,27), 45 degrees, with assets/open-models explicitly selected completed successfully (25.70 s image write), and inspection shows tall branching textured trees instead of tiered cones. This verifies different asset selection; it does not prove the user's accepted prior appearance or a completed forest fix. The detailed trees are still too pale and crowded against the supplied reference. All five workers received the finding and the requirement to select the same asset root for comparable native/browser art previews. No global tree geometry was reverted.

The compact shared sign native build finished successfully. Its muted paired Lake capture also completed (105.44 s for 3D), and inspection shows the taller, narrower panel; water-bank blending remains abrupt. The overall outdoor goal remains open.

# Outdoor terrain review — 2026-10-10

## Lake visual comparison and shared sign shape — 2026-10-12

The muted timeout-verified shared executable completed the LakeOfRage
south close-up and original 2D pair in 75.35 and 76.07 seconds. Both readbacks
were inspected (960×720; 34,177 and 16 colors). The water now has continuous
tonal variation and highlights, and sparse reeds soften parts of the edge.
The lawn-to-bank transition still reads abruptly. Tree shapes were preserved.
This is useful intermediate visual evidence, not acceptance of the complete
outdoor standard or a final current-source audit. The temporary executable
copy was removed after the capture completed.

The original sign in the paired 2D view has a compact notice panel; its 3D
replacement read like a bench backrest. The existing shared RouteSign model
now has a taller panel and inset, with the original posts, lettering,
materials and floor anchor retained. Its source geometry was resized with
the normals corrected for the height change; all 252 nondegenerate triangles
retain outward winding. The editable generator specifies the new proportions;
Blender regeneration was not run in this environment.

Verified Johto sign placements fit the panel inside the original two-by-two
source plot at 1.65 tiles wide and two tiles high. The early-town renderer now
uses this same existing model, removing its separate procedural sign drawing.
Recognition, source ground, claimed cells, interactions and collision stay
unchanged. The existing native-sign regression now passes in the integrated
Cargo command: one passed, zero failed, 1,424 filtered out. The other selected
client test binaries also compile successfully. The authoring script applies
the same five-thirds vertical scale to the original panel/inset/lettering/
fixing geometry, retaining the authored bevel treatment on regeneration;
its Python syntax was checked without generating scratch assets. A fresh
sign render is required before visual acceptance; its native build is running.

## Authored grove shoreline foundations — 2026-10-12

Preserve the existing tree shapes, size, foliage and source placement while
repairing these contacts, as explicitly requested. This foundation restoration
does not edit the foliage generator or tree assets. Both ordinary Johto tree
adapters continue to call the same grove generator with their source bounds,
height, coordinates and neighbor information; Park and conifer assets remain
distinct. Do not reshape trees to fit Lake of Rage's shoreline or cabin gaps.
The fresh production test binary also passes
`tree_variants_preserve_horizontal_plot_and_ground_anchor`, covering the
shared generator geometry and ground anchor across the expanded map scope.
This does not replace visual comparison of the source layouts.

The fresh full-map production audit completed on 65 named outdoor maps
(all `RouteN`, `*City`, `*Town`, and LakeOfRage entries in the supplied pack),
plus 39 route interior negative controls. All 265 eligible authored tree-water
contacts have actual wall-triangle intersections at the halfway water datum:
zero gaps. Nonzero cases include Cerulean 50, Fuchsia 3, LakeOfRage 19,
NewBark 3, Route12 64, Route27 51, Route30 6, Route32 11, Route34 6,
Route42 14, Route44 20 and Violet 18. The audit used full map bounds at native
32-unit tile scale and the production mesh builder with actual pack textures,
not fixture terrain. Interior controls had no eligible contacts. Temporary
Rust source, executable, copied library and orchestration script were removed.
The result proves this contact invariant, not complete outdoor visual quality.

A full-map production mesh audit of Lake of Rage cast horizontal rays midway
between the native lawn and water datums at every `johto/tree` claim bordering
native same-atlas water. All 19 tested contacts were physically open. There
were 3,340 candidate subfloor triangles in the mesh, so this was not a failure
to inspect existing terrain. The cause is shared: modeled foliage supplies a
new lawn plane, then claimed-tree cells skip ordinary exposed-side meshing.
The supplied source layout and collisions are not the cause.

The shared authored terrain path now appends the missing tree-ground water
contacts. It uses existing canyon wall geometry/pigment with fixed perimeter
vertices between zero and the native water datum. Recognition is limited to
proven tree claims with zero footing, a known outdoor tileset, and adjacent
same-atlas native water; unknown viewport continuation, custom claims and
elevated floors are left alone. Johto, Kanto, conifer and Park tree producers
share the restoration without duplicating their assets or palettes.

The new regression casts rays through eight contacts around a two-by-two
modeled grove in all four directions at 8- and 32-unit tile scales. It also
requires unchanged footing/ownership and rejection of custom claims,
elevated floors and foreign water sources. The focused regression passes with the production contact function, canyon
wall builder, pigment, quad primitive and test body read directly from source
in a temporary Rust harness. It checks actual 8- and 32-unit geometry; the
fixture helper itself hardcodes 8-unit tiles, so this regression explicitly
scales both geometry dimensions. Temporary source/executable were removed.
The fresh full-map production audit now passes all 19 Lake of Rage contacts
with zero gaps (3,796 candidate subfloor triangles). Cherrygrove also passes
all 96 native rock-water rays at both sampled depths with zero gaps (31,488
candidate triangles), verifying the signed-course fix through the actual
pack renderer. Temporary audit source/executable were removed.

The native client build finished successfully in four minutes after a
configuration ownership fix: the location-test shiny flag is copied before
its configuration moves into initialization, matching the other capture
flags. A transient duplicate Vec2 import in another edited module had
already been removed by the time the build was retried. Integrated grove
regression now passes directly in the freshly compiled production renderer
test binary: one test passed, zero failed, 1,406 filtered out. This verifies
the actual module, beyond the earlier focused temporary harness. The full
integrated Cargo command subsequently completed successfully, including the
production renderer regression (one passed) and compilation of the selected
Bevy, render API and text client test binaries. Fresh muted visual review
remains pending.
The subsequent independent static image-target capture also produced no
usable image: it reached its 60-second deadline while still building authored
terrain. The original movement capture was stopped after confirming it could
not obtain sustained window focus; no failed output is accepted as visual
evidence. A fresh native build subsequently finished in the shared
`target-bevy20-verify` cache. However, its static Lake attempt again timed out
at the old deadline during terrain construction: inspection of the executable
found no `CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS` setting. That build began
before the new capture module was edited, so completion time alone did not
prove the newer setting was included. A fresh native build in the owned
`target-outdoor-verify` cache is now running after the integrated test command
finished. Its timeout-setting inclusion must be verified before the next
static attempt. Both failed static captures remain excluded from visual proof.
The initially queued native build was stopped while it was still waiting for
the shared build lock, before compiling anything. Its replacement uses the
same cache and feature union with a command-only package override,
`profile.wasm-dev.package.crystal-bevy.opt-level=0`, to reduce preview-host
compile time while retaining optimized terrain generation and dependencies.
No Cargo manifest or tree/rendering source changes are involved. The native
build cleared the shared lock and is compiling. A newer shared executable
then became available in the same cache; its binary contains the explicit
timeout-setting name that was absent from the failed build. A fixed temporary
copy is running a muted, image-target LakeOfRage close-up at 45 degrees,
zoom 2, with the bounded 300-second timeout and paired native 2D capture.
The dedicated host build remains live. This intermediate shared executable
must not stand in for a final current-source completion audit.
The current worktree's separate shoreline-plant module also passes both
production renderer unit tests: roots/vertices remain in the water cell,
placement is deterministic in map coordinates, footing/ownership remain
unchanged, and paths, interiors and claimed water reject accents. These tests
do not establish that the plants look good; the capture above predates this
module and cannot visually validate it.

## Lake of Rage and consolidated outdoor grove coverage — 2026-10-12

A Rust audit of the supplied pack found 301 complete 2×2 tree crowns inside
Lake of Rage's mixed Johto blocks (`$5b`–`$67`). The detailed mixed-tree path
was restricted to the original six-map slice; Route26 also bypassed mixed
recognition despite having a dedicated tree adapter. This left the same
source drawings on later routes and cities outside preferred mixed-tree
ownership and contact shading; the generic renderer could still draw trees.

Outdoor grove coverage now uses one route/city/town scope, with Lake of Rage
and the two named outdoor areas included. Building coverage remains separate.
Every mixed crown still requires its exact tile drawing, subtile phases,
complete bounds and a real same-atlas ground donor. Complete groves reserve
their native cells before live-profile masking and use the existing branching
broadleaf asset. Park trees, Kanto foliage, conifers, incomplete/custom art,
collision and source layouts retain their identities.

Ground contact now consults the same generic tree-ground eligibility as the
renderer and uses the shared trunk offset for Johto/Johto Modern crowns.
Johto routes share the existing sunlit ground finish; this is not a new
texture or tree catalog. Source audit also found previously omitted mixed
crowns on Route27 (12), Route32 (34), Route35 (3), Route36 (83), Route37 (8),
Route38 (24), Route39 (3), Route42 (39), Route43 (31), Route44 (21),
EcruteakCity (12), and MahoganyTown (3).

The three focused regressions pass in the newly compiled renderer test
binary: mixed crown recognition/contact, exact shared tree geometry with
unchanged footing, and signed bank-to-water foundations. A fresh full-map
production mesh check passes ownership and zero native footing for all 301
Lake crowns, 31 Route43 crowns, 39 Route42 crowns and 12 Ecruteak crowns.
Temporary probe source/executable were removed. Integrated Cargo completion
and fresh muted native renders remain pending. Two Lake captures reached the
60-second limit while terrain was building, so they supply no visual approval.
A muted production movement capture completed successfully in 248.85 seconds
with paired 3D and native 2D images. It remains a baseline from the earlier
binary, not approval of the consolidated changes. Inspection confirms that
trees were present through the generic path; weak ground contact, right-angle
shore seams and bare water-to-lawn edges still need work. The 32-frame move
changed four grid origins without rebuilding terrain; no playability claim
is inferred from this render check.
The whole lake (including northern cabins/woods) and other cities still need
visual review, not just source-signature coverage.

## Native bank-to-water foundation gap — 2026-10-12

The latest full-map Cherrygrove headless audit inspected the production mesh
from the supplied local pack: all 120 source buoy cells have course claims,
and no sub-floor vertical buoy atlas faces remain. The only retained low
vertical atlas faces are 24 native path faces (block `$01`, tile `$06`).
The temporary probe source/executable were removed.

A second full-map probe cast 96 horizontal rays through native `$0a` rock
contacts with adjacent `$14` water, sampling source-cell centers at heights
-1.5 and -0.5 in native eight-pixel units. All 96 rays found gaps. Inspection
of `append_bank_run_side` identified the cause: its first course was clamped
to zero, skipping the wall between the native water datum (-2) and zero.
This is an actual open foundation, not merely a bad shoreline pigment.

The shared bank mesher now retains signed foundation courses. Above-ground
course indices still use the original nonnegative offset, so the extension
cannot reorder the authored wall art. The new regression checks four face
directions at both eight-pixel and native 32-unit capture scales, requiring
a complete foundation and identical upper positions, UVs and normals.
The focused foundation regression passes in the newly compiled renderer
test binary and the integrated Cargo run completed successfully. Fresh
native build and all 96 full-map rays now pass; muted visual review remains
pending. The pale waterline is not declared fixed until those pass.

## Shared native buoy-course trial — 2026-10-12

`buoy_course` now handles Johto/Johto Modern tile `$58` in native water blocks
`$30`–`$39` through one producer. It replaces only complete unclaimed planes
at the native water datum, with a same-atlas `$14` water donor. The float's
placement/radius follow the largest connected foreground component in the
existing tile. Detached wave marks are excluded. Both paint swatches sample
that existing tile's palette, so no palette, texture or exported source is
introduced. Missing art/water donors, unknown identities, cropped planes and
reserved/custom objects retain the existing rendering path.

The float is closed beneath the water and uses real planar quads compatible
with the shared ground finisher. Ownership enables the existing water finish
only below this exact source identity at its original datum; floor heights,
source layout and collision remain unchanged. Temporary Rust tests compiled
the actual silhouette/geometry routines and shared quad primitive: both
silhouette isolation and closed/outward/non-degenerate mesh checks passed.
The integrated course/water-finishing regression is pending.

The fresh native build completed with exit 0 in 4m46s. The actual-45-degree
muted Cherrygrove capture completed with exit 0 in 22.11s and was inspected:
`target/outdoor-cherrygrove-shared-buoy-course-45-muted.png` (1280x960).
The blue/white checkerboard course is gone and the original two-row course
remains. The first floats are too bright and read as flat white marks, so this
appearance is rejected. The follow-up confines the original white swatch to
the small crown, uses the original middle swatch over the broad body, and
reuses the existing native prop directional shading. `solid_color` retains
its original factors through that shared helper; no new palette is added.
All four integrated buoy regressions passed through Cargo and directly in the
fresh renderer binary, including the shared-water finish and unchanged floor
heights. The shaded native build is still running.
The pale straight waterline remains an independent open issue.

## Reuse native Cherrygrove stone — 2026-10-12

Follow-up tracing found an additional map-name exception: the shared stone
mesher selected canyon pigment for `$0a` on Route26 but regular stone elsewhere.
The identical Johto/Johto Modern `$0a` drawing now selects the existing canyon
finish in every town/route. No new palette or texture is introduced. The parity
regression now compares actual stone vertices, colors and UVs across
Cherrygrove, Route26 and Johto Modern, and first proves that its fixture matches
the shipped coastal profile. The fresh direct renderer binary passed that
strengthened regression and the custom-profile check. Its compiled diagnostics
were also checked for all three new assertion messages, confirming this was
the current fixture rather than the earlier weaker one. The integrated Cargo
process is still running. This latest pigment change requires a fresh muted
capture; the gray captures described below precede it.

The integrated Cargo parity run subsequently completed with exit 0 (20m50s,
including queued compilation). The first preview build hit a missing café
helper during a concurrent interior edit. Reinspection confirmed that helper
and its include were present; the retry completed with exit 0 in 4m28s.
`target/outdoor-cherrygrove-shared-canyon-stone-45-muted.png` was captured
muted at actual 45 degrees and inspected. The coastal `$0a` masses now share
the warm ochre/canyon cap and side courses. The previous gray boulder asset
and its plinth are absent. Pale straight waterline strips and the blue/white
flat buoy course remain obvious. The rounded caps are still below the stated
shape standard, so this color/asset consistency improvement does not close
the outdoor goal.

The buoy trace found no existing 3D buoy producer. Both Johto and Johto Modern
classify tile `$58` in water blocks `$30`–`$39` as a plane at `WATER_HEIGHT`.
A temporary Rust probe read the supplied pack and confirmed Johto's buoy
sample resolves to bank 0, slot 88 of 192 source tiles. The float silhouette
is distinct from the background and small wave marks. Future modeling should
preserve the silhouette's placement/proportions and the existing course,
reuse the common water finish, and use one shared source identity adapter.
The temporary probe source/executable were removed; no pack or audio source
was copied into tracked paths.

The user identified duplicated treatment of blocks already rendered elsewhere.
The shipped `Cherrygrove coastal boulder` live profile was claiming the exact
Johto `$0a` block before the native bank mesher could handle it. Ordinary Johto
and Johto Modern already render that block as a connected stone platform with
the common outdoor mineral finish. Canonical copies of that profile now defer
before reserving source cells, so Cherrygrove uses the same geometry, pigment,
texture coordinates and floor support. User-edited profiles retain their
explicit live renderer; Kanto's distinct shoreline identities are unaffected.

The separate coastal sculpture, palette and water-bed exception from the
unaccepted trial were removed. Its first captured image still showed the old
sphere/plinth model and cannot verify the trial. The experimental buoy rewrite
was also removed from this pass; native buoy artwork remains a separate open
visual issue. A regression compares native and profile-enabled meshes,
including colors, UVs and footing. The fresh native build completed with exit
0 in 7m00s. Muted actual-45 and 90-degree captures were inspected:
`target/outdoor-cherrygrove-shared-native-stone-45-muted.png` and
`target/outdoor-cherrygrove-shared-native-stone-90-muted.png`. They confirm the
sphere/plinth override is gone. The common stone still looks too grainy; the
pale waterline and raw buoy strips remain visible. This is consistency work,
not visual acceptance. Integrated parity and custom-profile checks are pending.

## Shared ordinary tree assets — 2026-10-12

Ordinary Johto town and route trees now call `append_grove_tree`, sharing
branching trunks, forked limbs, asymmetric crowns, leaf materials and stable
source-coordinate variation. The separate town crown recipe and its leaf
branches were removed. New Bark, Route29, Cherrygrove, Route30, Route31,
Violet and Route26 use the same mesh for matching source plots; the generic
Johto exterior path uses that renderer too. Small ordinary trees use the same
family at reduced scale. National Park's garden trees and separate conifer
assets retain their distinct authored species.

Trunk contact shadows now follow the shared offset in each supported town
and route. New Bark's path uses a cooler gravel palette, carried through
ground blending so the old tan edge does not reappear at grass transitions.
Source ground ownership and walking support remain separate from crowns.

The fresh muted Cherrygrove capture at actual 45 degrees was inspected:
`target/outdoor-cherrygrove-unified-grove-muted.png` (1280x960).
The previous tall pillar crowns are gone and the town shares the route's
tree family. This is an asset consistency improvement, not acceptance of
the reference's finish: crowns remain too rounded and uniformly lime-green;
ground is sparse and water remains flat. Fresh tests and the New Bark/Route29
visual comparison are pending at this checkpoint.

The New Bark–Route29–Cherrygrove baseline review also found coastal rocks
that read as gray balls on square pedestals, raw shoreline strips, repetitive
forest rows, and a pixel facade at the Route29 gate. Central captures do not
prove complete map coverage. These findings remain open until independently
fixed and reviewed.

Fresh direct renderer checks subsequently passed: 79 New Bark/exterior-ground
checks (2 ignored), 7 ordinary foliage checks, 9 Park foliage checks, and
36 modeled exterior checks. The cross-map test compares complete solid meshes
and unchanged floor heights, not merely a tree count. The New Bark capture
timed out and produced no image, so the cooler path palette has not yet had
visual acceptance.

The user explicitly requested a mod over the unintended lake behind the
Cherrygrove Center. The wider camera exposes the original repeating sea border
outside the map. The first native review showed the border filled, but its 2D
comparison revealed additional water through the Route30 connection, hidden
under the new 3D canopy. That first capture is not acceptance of the lake fix.
`exterior_border_block` replaces that off-map border with complete Johto
tree block 0x05 on the east side and north of the land-side region (source
x >= 24). The western and southern sea retain the original water border.
Connected-map selection precedes this fallback, preserving Route29/Route30.
The same source block and art feed 2D and 3D. The border fallback does not
change playable-map collision.

The follow-up runtime mod replaces the original matched shoreline pairs at
Cherrygrove block (17,0)/(18,0) and Route30 block (9,25)/(9,26) with tree block
0x05. It runs after pack verification and before runtime map catalogs and
sessions are constructed, so authoritative collision and both renderers agree.
Only original Johto maps of the verified dimensions and exact shoreline pairs
are changed; independently modified layouts are preserved. Original external
pack bytes, audio, scripts, doorways and western ocean blocks remain unchanged.
The actual source function passed its focused Rust regression, including
unchanged non-lake blocks, repeat application and modified-pack guards.
Fresh native rebuild and source-level/2D/3D visual checks remain pending.

The first source-mod capture removed the connected Route30 water in 2D, but
still showed city water at its right edge. Cherrygrove's tile callback invokes
LoadBlockData, whose `overworld_map` reconstructed the original split payload
and overwrote the modified module's blocks. `overworld_map` now uses an assembled
runtime module when present, retaining split-payload assembly for build inputs
without one. A regression compares a modified module against the original split
payload and checks both paths. This reload fix still needs a fresh native review.

The pack-backed source check consumed the verified external pack in memory,
applied the actual mod function, and found exactly four changed blocks across
all maps. All 16 changed collision quadrants resolve to tree block 0x05 and
WALL; a western-sea sample retains WATER. Temporary probe source and executables
were removed. The native western-coast capture was inspected:
`target/outdoor-cherrygrove-coast-preserved-grove-muted.png` (1280x960).
The sea and island remain. The spherical gray boulders, square bases and raw
wave strips still fail the outdoor art target; the grove change does not fix them.

The shared-tree Route29 middle view was also inspected:
`target/outdoor-route29-unified-grove-middle-muted.png` (1280x960).
Its tree family matches both towns. The gate remains a pixel-textured facade
and the ground remains sparse; these are open visual findings.

The final reload-fixed native build finished with exit 0. Both muted final
captures were inspected at actual 45 degrees:
`target/outdoor-cherrygrove-lake-removed-reviewed-muted.png` and
`target/outdoor-cherrygrove-lake-removed-native-2d-reviewed-muted.png`.
The remaining water behind/right of the Center is now replaced by trees in
the 2D view as well as the 3D view; the prior first-pass captures are superseded.
The pack-backed check against the fresh production asset library also confirmed
that `overworld_map` retains both tree replacements in each of the two maps.
The runtime mod's integrated Cargo regression passed (1 check), as did the
integrated border regression (1 check). This verifies this lake correction,
not the overall outdoor art target or gameplay completion.

The full asset-test crate could not compile: six `include_str!/include_bytes!`
references require missing unrelated radio oracle fixtures under
`tools/asm-oracle/fixtures/`. No fixtures or exported source were added.
Consequently the new asset reload regression has not run in that full crate.
Attempts to compile an isolated test harness after switching dependency feature
sets could not resolve the cached production asset crate, so they provide no
additional passing result. The successful pack-backed reload check and the
fresh native 2D/3D captures above remain the direct evidence for the fix.
All owned preview/build/test processes for this checkpoint have exited.

The fresh New Bark view was inspected at actual 45 degrees:
`target/outdoor-new-bark-unified-grove-reviewed-muted.png` (1280x960).
It shares the same broadleaf family with Cherrygrove, and the old tan plaza
is now cooler gravel without an obvious tan blending strip. Ground detail is
still sparse, and crown repetition remains below the forest reference.

## Camera-angle correction — 2026-10-11

`new_bark_3d --orbit N` initializes rotation_step=N×0.5, and the camera
turns rotation_step×45°. CLI orbit 1 is therefore 22.5°, orbit 2 is 45°,
orbit 3 is 67.5°, orbit 4 is 90°, and orbit 6 is 135°. Earlier entries
in this review that call CLI orbit-1 captures 45° or orbit-3 captures 135°
mislabel their angles; those angle claims are withdrawn. The same correction
applies to park captures made with CLI orbit 1. Their actual image/geometry
observations remain evidence only at the corrected angles. The half-step CLI
initialization predates these captures (committed 2026-09-30).


All subsequent trials, tests, previews and screenshot runs must be silent, per
the user's 2026-10-11 instruction. Native preview commands must set
`CRYSTAL_PREVIEW_MUTE=1` and use an executable rebuilt with the sink-level mute
support in `bevy_shell.rs`; older executables do not honor this switch. Browser
previews must use `crystal_set_muted(true)` before playback or mute their own tab.
Never change system volume or another application's audio. No outdoor preview
process remained running when checked after the request.

The user's forest illustration remains the quality target. The user accepted
the improved stone texture and requested better shapes and correction of visible
ledge/floor height mismatches. The broader rendering goal remains unfinished.

Compared Route26's original 2D rendering with native 3D views at 45 and 135
degrees, using the ignored local realtime-clock browser pack. Original rock
platform footprints remain recognizable. Stone and canyon now use original
built-in imagegen material assets embedded by Rust; exact prompts and saved
asset names are in `assets/terrain/README.md`. Browser execution was not tested.

Rock and canyon rims are eroded along exposed source boundaries, with stable
map-space variation and shallow wall relief. Internal terrain joins and center support remain unchanged. The later contact
pass described below rounds free basal corners while preserving source
footing heights. Painted rock corner caps are
completed before erosion so their rims meet the side walls. Restored corner
side sections also reach the neighboring ramp's boundary height, closing the
narrow side opening seen during the 135-degree review.

Jump banks rise across their four authored rows and retain a sharp south drop.
Consecutive drawings previously shared a flat bank run with no internal faces;
sloping them exposed the background at each reset. Those resets now receive
closed earthen faces. Ground details above the cap follow its slope. Protruding
cliff triangles retain their high vertices while their bases meet the bank,
including triangles whose centroids fall inside the neighboring bank cell.

Native captures confirm the large horizontal gaps and cliff-base slots are
closed in the reviewed Route26 views. Regression coverage checks continuous
ramp heights, a ray through an internal drop, preserved cliff tops, offset
ground details, shared eroded cap/wall corners and original embedded textures.
The latest full voxel-view library run passed 1,064 tests, with 32 ignored.
These are rendering checks, not evidence of gameplay correctness.

Remaining visual work includes broader cliff silhouettes and less regular tree
crown shapes. This pass does not claim the reference's overall finish. Content
packs, audio and authoritative gameplay were not changed. Review captures and
logs are temporary verification output, not source deliverables.

The following shape pass adds recessed weathering through the middle of exposed
stone/canyon faces, above the jump-bank datum. Segment endpoints stay attached
to native ledge closures; bases and walking support retain their original
heights. Wall tessellation uses common world-height rings even when adjoining
faces start on different floors. Cyclic source-vertex ordering is normalized
before subdivision, retaining both horizontal-first and top-to-bottom winding.
Regressions cover shared rings, winding, middle-wall shaping and fixed closure
endpoints. The final 45/135-degree Route26 review showed the thin cracks at
repeated ledge resets closed. Viewport boundary exposure remains visible at the
edge of the 135-degree capture and needs separate review.

Route trees now use one connected, uneven crown with deterministic scattered
leaf placement. The first preview's spiral pattern was rejected and replaced;
final previews have a less regular surface. Crowns and cliff outlines still
need further work against the forest illustration. One intermediate full run
failed the canonical Mareep model-byte check while that unrelated source asset
was changing; two subsequent full runs passed 1,064 tests with 32 ignored.

A further shape pass rounds convex stone/canyon corners across the full upper
shoulder, with radius limited by the continuing native rock footprint. Isolated
stone edges use a wider circular cap-to-wall fillet than canyon edges. The
first wider polynomial preview folded its cap triangles into a visible lip;
that preview was rejected. Circular projection now retains ordered cap
vertices, and positional normal averaging removes separate patch lighting.

One map-space warp is shared by stone and canyon vertices regardless of their
material tag. The nearest adjoining floor constrains the entire warp, including
perpendicular rims at corners; regression coverage proves that geometry below
a higher neighbor floor stays fixed. Additional assertions check identical
stone/canyon seam positions, shared cap/wall normals, an ordered cap cross
section, wider shoulders and preserved low ledge contact.

Reviewed the final native Route26 build at its default walkable location and
at (10,28), each at 45 and 135 degrees. Rocks have continuous rounded shoulders
without the rejected folded lip. The reviewed repeating ledge run remains
closed. The faint wider-view floor line near the houses presents as the bank's
earthen drop face from the reverse angle, rather than an exposed background
slot. The final library run passed 1,068 tests with 34 ignored; these checks do
not establish gameplay or browser correctness. The viewport boundary remains
visible, and the lower stone/canyon corner in the wide canyon capture merits
further close review. Square base outlines, the overall canyon silhouette,
canopy/trunk shapes and broader forest finish still fall short of the target.

The contact pass now rounds free basal corners of native Johto/Modern $0a
rocks. Each corner requires its matching local 2x2 rock neighborhood at the
same height and three outside lawn/path cells at height zero. Claimed authored
surfaces and higher/unknown floor contacts retain their existing joins. The
radius remains inside the corner source cell and varies by authored map
coordinates, avoiding a uniform rounded-box outline. Existing normal averaging
runs again after the contact warp.

The revealed corner receives a level ground patch using the same GroundFinish
color, path/lawn blending and grain as the surrounding map. Source footing
heights and rock-center support do not change. A first whole-plot eligibility
check was replaced: requiring a distant fourth source row/column caused a
visible corner to change shape when that distant part left the viewport. Local
source proof keeps the visible verified corner consistent.

Regression checks cover shared wall contact vertices, ground coverage beneath
the cut corner, unchanged footing heights/center support, anchored ledge
neighbors, altered/authored local neighborhoods, equivalent source viewport
scrolls and distant viewport cuts. The final full library run passed 1,074
tests with 36 ignored. The final native build was reviewed near the houses and
at (10,28), both at 45 and 135 degrees. Rounded free contacts meet the floor,
and the reviewed ledge/cliff-floor joins remain closed. Border exposure and
broader canyon/canopy/trunk forms still prevent a claim of reference fidelity.
Browser and gameplay correctness were not established by these render checks.


The next shoulder pass widens exposed sandstone fillets from 0.48 to 0.90
source tiles above ordinary ground. Contacts with native rocks at different
heights retain the tighter profile: the first unrestricted widening exposed a
thin opening at the smaller rock in the 45-degree Route26 view. The constrained
version closes that reviewed contact while preserving the broader free rim.
Ground contacts and jump-bank support remain fixed. A regression verifies the
broader inner-cap rounding, identical duplicated cap/wall positions, ordered
cap vertices and unchanged floor contacts.

The presentation apron now retains its selected VisualTileSource instead of
recovering a material from the first matching image handle. Equal source counts
use the earliest grid cell deterministically, avoiding material changes caused
by HashMap iteration. A regression covers tied counts and shared image handles.
This addresses source selection, not the still-visible cyan/pale foreground
boundary in the 135-degree view, whose cause remains unproven.

Final native Route26 (10,28) reviews at 45 and 135 degrees show closed reviewed
bank/cliff joins and no reproduced thin rock opening after the contact guard.
The renderer suite passes 1,080 tests, with 37 ignored; git diff --check passes.
These views do not establish that every outdoor map has correct joins or that
the wider environment matches the supplied forest reference.


A subsequent boundary diagnostic changed the outdoor clear/fog color to magenta
for a native Route26 (10,28), 135-degree capture. The conspicuous pale/cyan
foreground region retained its colors. It is therefore not exposed clear color;
its specific source/material remains to be audited. The temporary diagnostic
was removed and the normal native executable rebuilt. The original 2D view at
(10,28) was also captured, but its narrower viewport does not cover that region.

Route grove crown depth now follows canopy spread, capped at the previous
height-based depth. Tall narrow boundary trees no longer receive disproportionately
tall ellipsoidal crowns just because their trunks are tall. Town crowns retain
their established proportions. The renderer suite passes 1,083 tests with 37
ignored in the current shared checkout. Native visual review is recorded below.

Native Route26 (10,28) at 135 degrees and Route29's default walkable viewpoint at
45 degrees confirm flatter spreading grove crowns rather than upright eggs.
The reviewed Route26 ramp/cliff floor contacts remain closed. These scenes
still differ visibly from the reference: trunks are too uniformly tall and
exposed, crowns remain regularly placed, and a pixel-art shrub card survives
in the Route29 clearing. Those are subsequent exterior presentation issues,
not evidence that the overall rendering goal is complete.


The next grove pass lowers route crown centers from 0.72 to 0.64 of trunk-height
scale and lowers their branch endpoints by the same amount. This reduces the
exposed stem below the already flattened crowns, keeping the branch fan inside
the canopy. Native Route29 and Route26 (10,28) reviews at 45 degrees show this
lower canopy without an obvious detached branch or new reviewed floor opening.
Current renderer verification passes 1,084 tests with 38 ignored.

The small pixel-art card in the Route29 clearing remains unresolved. Temporary
actor tracing found no fruit-tree actor in the published viewport. The complete
Johto $63 cut-tree drawing elsewhere in the halo passed all four-cell identity,
clear-ownership and ground-sample checks and already received its model. No
unclaimed FacadeBand cells occur inside the halo's four-cell perimeter in this
view, so the visible card cannot be explained by a rejected nearby tree facade.
Its actual surface/instance still needs identifying. All temporary tracing was
removed and the normal native executable rebuilt. No pack, gameplay or audio
content was changed by this pass.


On 2026-10-11, a temporary textured-terrain tint proved the Route29 clearing
card belongs to textured terrain. Nearby upright textured face metadata traced
it to Johto $3c, tiles $1e/$1f, whose original compiled shape is Flat. The shipped
“Route29 small tree 3c 0,0” live profile creates its 18-pixel-high original-art
card. This explains why the previous facade and Cut-tree audit missed it.

Canonical Route29 and NewBarkTown small-tree live profiles now append compact
procedural broadleaf trees at half tile scale, using stable global map coordinates
for variation and preserving the existing native ground and footing. Authored
ownership is labeled johto/small_tree. Exact profile equality remains required;
customized profiles retain their original renderer. A regression proves native
source resolution, leafy solid geometry, flat ground, unchanged footing, exact
four-cell ownership, customized-profile rejection, and incomplete-art rejection.

Native Route29 at 45 degrees confirms the pixel-art card is replaced by a small
leafy tree. NewBarkTown at 45 degrees was also reviewed, with no obvious new
floor gaps at the reviewed small tree placements. The current renderer suite
passes 1,086 tests, with 38 ignored. All temporary tints and source tracing were
removed, the normal native executable rebuilt, and scratch captures/logs removed.
The broader environment still differs from the supplied reference and the pale
water/ground boundary remains under review; browser rendering is unverified.


The NewBarkTown shoreline was compared directly with original 2D art at (17,9).
The native drawing has an ochre rocky bank; the existing GroundMaterial::Shore
finish had replaced it with a broad pale gray strip. Verified Johto/Johto Modern
shore caps and their exposed water faces now share the existing painted outdoor
canyon/bedrock finish. The stone appender adds cap subdivisions and bounded wall
relief while preserving cap height zero, water height, and native face endpoints.
Other tilesets and water surfaces retain their existing materials.

The shoreline regression now includes both cap and exposed water face, verifies
the original endpoints remain present, bounds every height to the original
water/ground range, and checks the shared ochre material tags. The current full
renderer suite passes 1,086 tests with 39 ignored. A first full run hit the shared
checkout's canonical-model byte test during unrelated model edits; the fresh
final full run passes, without changes to those model files from this task.

Native NewBarkTown (17,9) at 45 degrees and Route26 (10,28) at 135 degrees confirm
that the conspicuous pale foreground strip is a shoreline-material mismatch,
not a gap exposing clear color. It is now ochre stone adjoining the existing
cyan water. The reviewed floor/ledge contacts remain closed. The source-authored
rectangular shoreline layout remains, and wider exterior form, foliage variety,
and browser verification remain incomplete. Scratch render/test outputs were
removed after review; no pack, gameplay or audio data was changed.

### Rounded rock cap sampling — 2026-10-11

Route 26 at (10, 28), 45 degrees exposed thin bright cracks on the upper-right native rock cap. Its clipped polygon fan and restored triangular corner had unequal edge samples beside regularly subdivided walls. Coincident endpoints alone did not preserve the curved edge after shoulder erosion. The verified exposed 0a corner now replaces the referenced fan cap with a regular full-cell cap before the shared sculpt, keeping sixth-cell samples aligned with the walls. The existing rounded shoulder and basal contact projection remain; textures, source heights, collision, and gameplay are unchanged.

The corner regression now checks referenced cap perimeter sampling, in addition to restored floor support and finite positions. Full renderer suite: 1,086 passed, 0 failed, 39 ignored. Native build succeeded. Reviewed fresh Route 26 captures at (10, 28), 45 and 135 degrees, plus the default southern location at 45 degrees. The previously visible bright cap cracks are absent in the new 45-degree capture, and inspected ledge/floor contacts remain closed. This is a sampled visual audit, not proof that all maps are aligned. Broad canyon silhouettes, forest forms, and browser verification remain unfinished against the reference. Temporary verification captures and logs were removed after inspection.

### Continuous cliff wall forms — 2026-10-11

The previous wall erosion returned to zero at every source-cell endpoint, leaving a repeated grid of small dents on long canyon walls. Collinear touching edges now share a run only when their native height, adjacent floor height, normal, and shoulder profile agree. Broad world-space weathering continues across internal tile seams; real run ends, height transitions, and verified adjoining jump-bank south lips remain anchored. Wall inset is bounded to 0.32 source tiles above the maximum floor/jump datum, with a 2.4-tile noise scale. Mineral texture and authored terrain/support heights are retained.

A first broad profile exposed narrow slits at jump-lip contacts in the native 45-degree view. Source-authenticated lip anchors closed these during iteration; they remain explicit run ends rather than being mistaken for internal rock seams. The final geometry regression checks erosion across a shared internal wall seam, identical duplicate vertex displacement, unchanged floor/bank contacts, finite geometry, and separation at differing heights or authored lip anchors. Full final renderer suite: 1,087 passed, 0 failed, 39 ignored. Earlier full runs encountered concurrent embedded-model byte changes; a fresh final run passed without editing those model assets.

Fresh final native Route26 (10,28) captures at 45 and 135 degrees show broader softened wall contours and closed inspected ledge junctions. The effect improves wall form but does not establish the complete reference standard: plateau silhouettes are still broad rectangles, forest crowns remain too regular, and browser/all-map coverage remains incomplete. Temporary verification artifacts were removed after review. No packs, gameplay, or audio data were changed.

### Recessed plateau crests — 2026-10-11

Straight canyon tops now receive a broad, map-anchored recess shared by the cap and wall. The field fades toward level interior terrain and the maximum adjacent floor/jump-bank datum. Source-verified exposed corners continue the recess rather than retaining pointed terminal tips. The maximum lateral field is 0.22 tiles, sampled over 2.8 tiles with a 1.2-tile inward fade. It is evaluated after the existing fillet and corner rounding, including noise sampling at the rounded coordinates. This ordering avoids reversing the compressed border vertices on narrow rock caps.

The initial larger pre-fillet field failed the existing narrow-cap ordering regressions and was corrected rather than weakening those checks. A new plateau regression checks that the crest recedes beyond the old narrow bevel, cap/wall duplicates coincide, floor contacts remain fixed, and the plateau interior retains its native level plane. The final geometry tests pass. Final native build succeeded; inspected fresh Route26 (10,28) views at 45 and 135 degrees show softer upper outlines and closed checked ledge contacts. Disk exhaustion interrupted test compilation during iteration; own obsolete captures/logs were removed, and compilation resumed when space became available. A full-suite run of the final coordinate-sampling adjustment passed 1,087 tests but encountered concurrent Wobbuffet model-byte changes; the fresh final full suite passed 1,089 tests, with 0 failures and 41 ignored.

This improves the upper silhouette but does not achieve the complete reference standard. Canyon footprints remain visibly broad and rectangular, forest canopies retain regular repeated forms, and browser/all-map evidence is still incomplete. Textures, authored floor/support heights, packs, audio and gameplay are unchanged.

### Branch-supported route crowns — 2026-10-11

Route woodland crown shape now uses five unequal angular leaf masses joined by one connected skin, replacing the sine-only fluted oval. The three lower and two higher clusters vary with the existing stable source-coordinate phase. Four supporting limbs now each fork into two tapered branches. Lower limb roots and fork tips expose the branching beneath the canopy, and route trunk radius increases from 0.105 to 0.125 source-plot spacing. Existing leaf count, pigment, native trunk placement, ground vegetation, and cutaway ownership remain. Town and small-tree crown paths are unchanged.

The route geometry regression now includes all twelve bark sections in its blocked-plot bounds check, rather than only the original four limbs. Existing finite geometry, outward winding, deterministic placement/color, and cutaway ownership checks pass. Final full renderer suite: 1,090 passed, 0 failed, 41 ignored. Native build succeeded; fresh final Route29 and southern Route26 45-degree views show visible lower branch forks and unequal crown contours. The earlier Route26 (10,28) capture also retains the checked closed ledge contacts. Own verification captures/logs were removed after review.

The reference standard is still incomplete: groves retain regular source rows and rounded crown masses; town forest forms and browser evidence need work. The southern Route26 capture also shows a thin jagged foreground grass/path transition line beside the low ledge. Its cause remains unproven and warrants the next floor-alignment inspection rather than assuming it is a hole or acceptable detailing. No packs, gameplay, audio, or authored floor heights changed in this pass.

### Ground blends follow rendered ledge edges — 2026-10-11

The ground blend previously compared constant native cell heights, although the four-row jump approach renders as a slope. It therefore excluded a joined zero-height toe beside ordinary path/lawn. The blend now compares both endpoints of the shared edge, using rendered approach corner heights and the same verified terminal taper as the terrain. Matching toes and continuous slope edges may blend; a front drop or partly mismatched side stays distinct. Geometry, collision/support heights, pigments, and jump behavior are unchanged.

A new regression checks continuous color across a sloping toe, no blend across its six-pixel front drop, and rejection when only one side-edge corner matches. The existing four-row approach regression now checks the blend's reported corner heights against the actual warped cap vertices. Final full renderer suite: 1,091 passed, 0 failed, 42 ignored. Native build succeeded. Fresh southern Route26 views at 45 and 135 degrees show improved grass/path edge transitions.

The prominent foreground line was not fully explained by the first color-seam diagnosis: it remains visible, and the reverse view exposes the earthen jump face at that outline. Thus the toe-blend change must not be reported as removing that entire line or proving all floor alignment. Its silhouette still needs shape review against the reference. The broader cliff/forest standard and browser/all-map checks remain incomplete. Own verification captures and logs were removed after inspection; no packs, audio, or gameplay data changed.

### Shared jump-lip outline — 2026-10-11

Continuous, untapered, unclaimed Johto jump-bank interiors now receive a shallow lateral rim recess, at most 0.10 tile. It fades to zero at ground level and at walking-cell centers; terminal cells, modeled contacts and all Y coordinates remain fixed. The same map-anchored linear half-cell knots shape both the coarse cap and finely divided earthen face. Sampling a smooth curve independently on their different subdivisions would leave cracks, so the shared piecewise outline is deliberate. Existing rise/drop geometry and native support heights remain.

A regression checks cap/face coincidence with different subdivisions, unchanged ground and walking centers, unchanged Y, finite geometry, source-stable translation and modeled ownership rejection. Final full renderer suite: 1,093 passed, 0 failed, 43 ignored. The final native build succeeded after waiting for the shared artifact lock. Southern Route26 native views at 45 and 135 degrees show joined checked rim/face geometry, but the thin line along the rear approach remains visible at 45 degrees. The earlier capture is not treated as a reliable before/after baseline because a build was running when it started. This pass must not be reported as resolving every floor mismatch or meeting the complete reference standard. No packs, audio or gameplay data changed.

A fresh final-build Route26 (10,28) view at 45 degrees also shows closed checked jump-bank/rock/canyon contacts. The repeated bank silhouettes and broad canyon footprints still need refinement against the reference. Own scratch captures and verification logs were removed after inspection.

### Remove collapsed bank-back relief — 2026-10-11

The southern clearing's thin jagged line came from finishing an obsolete north-facing wall before converting the first bank row into its zero-height toe. The Y warp flattened the wall, while its outward earth relief and rim strands could still draw on the level floor. Source-verified Johto first-row north walls now disappear before finishing. The guard requires the actual cell's bank identity, north boundary, north-facing normal, and shallow original height. Front drops, side contacts, taller cliffs and later rows remain.

Fresh native southern Route26 views at 45 and 135 degrees no longer show that jagged strip. The slope-to-floor boundary still has a broad color/lighting transition; removing the erroneous strip does not prove that every outdoor floor contact is aligned. A fresh Route26 (10,28) 45-degree capture retains the checked front drops and closed canyon/rock contacts. The native build succeeded. The regression also exercises the finish pipeline, proving the obsolete face produces neither earth relief nor rim grass and does not mutate footing heights. Initial full suite passed 1,094 tests, with 0 failures and 43 ignored; the extended regression's next full run passed all terrain checks but encountered a concurrent Bellossom canonical-byte mismatch. The fresh final full-suite rerun passed 1,094 tests, with 0 failures and 43 ignored. Own captures and verification logs were removed after review.

The reference forest, canyon shapes, and browser/all-map verification remain incomplete. No packs, audio or gameplay data changed.

### Crown lighting follows leaf-mass shape — 2026-10-11

Route woodland previously displaced its crown skin into lobes but still shaded the skin and leaf orientations with enclosing-ellipsoid normals. The shared crown sampler now returns both radial mass and its analytic tangent gradient. Inverse-radius scaling converts the displaced surface normal to the actual ellipsoid dimensions, and both skin lighting and leaf orientation use that direction. The five branch-supported masses are more distinct: base radius 0.78, lower mass weights 0.42, upper weights 0.48, retaining the existing small ripple. Town/small-tree sampling retains its previous radius and enclosing normals. Leaf count and pigment palette remain; displayed shading changes with the corrected directions.

A new regression compares analytic normals with finite differences of the actual displaced crown over three phases and an angular grid, checks finite unit normals, and proves a material departure from enclosing-oval directions. Existing winding, source trunk bounds, deterministic geometry/color and cutaway ownership checks pass. Final full renderer suite: 1,095 passed, 0 failed, 43 ignored. Native build succeeded. Fresh Route26 and Route29 45-degree views show uneven leaf masses and local lighting changes; the repaired southern Route26 toe strip remains absent. Own verification captures and logs were removed after inspection.

The reference standard is still incomplete. Forest edges expose regular rows of long trunks and the crown clustering remains repeated; canyon footprints and broader browser/all-map evidence need further work. No packs, audio, authored support heights or gameplay data changed.

### Lower foliage around route forks — 2026-10-11

Route crowns now sit at 0.57 of nominal tree height rather than 0.64. Vertical radius is limited by the smaller of 0.30 height and 0.65 spread, replacing 0.245 height / 0.38 spread. This brings the lower foliage around the upper trunk and supporting forks while retaining the lateral dimensions, source trunk/branch placement and ground ownership. Town/small-tree proportions remain.

The route geometry regression now checks lower crown coverage, a readable lower trunk and bounded mature canopy height for isolated, partially connected and interior grove placements, alongside existing winding/source-plot/determinism/cutaway checks. Full renderer suite: 1,096 passed, 0 failed, 43 ignored. A normal native build reached linking but failed with disk exhaustion. A subsequent verification build using `cargo rustc` with linker debug-symbol stripping succeeded after free space became available; shared build caches were preserved. Fresh Route26 and Route29 45-degree views show shorter exposed trunks and fuller foliage around the forks. The repaired Route26 toe strip remains absent. Own captures and verification logs were removed after inspection.

The broader reference goal remains incomplete. Some enlarged crown masses look column-like, regular tree rows remain visible, and more irregular leaf silhouettes are needed. Canyon shapes and browser/all-map verification also remain open. No packs, audio or gameplay data changed.

### Scatter route trunks with matching contact shade — 2026-10-11

Route trunk offsets now use 0.14 of source-plot spacing rather than 0.065, from the same stable source-coordinate variation. Primary limb and fork reach decreases from 0.38 to 0.33 spacing so the complete bark remains within the original blocked plot. A shared offset sampler drives the physical tree and its local trunk-contact shade. The broad bounded canopy shade and lawn underlay retain their source bounds; towns, signs and buildings keep their existing contact centers. No walking-cell or source ownership change accompanies the visual offsets.

A new regression samples thirty positive/negative source-coordinate pairs, checks every trunk/limb/fork vertex against the blocked plot, and proves an offset span exceeding the old nearly aligned grid. The contact regression checks that the dark local shade follows a displaced trunk while the original source bounds remain intact. Final full renderer suite: 1,097 passed, 0 failed, 43 ignored. Native build succeeded. Fresh Route26 and Route29 45-degree views show less uniform visible trunk spacing and shaded displaced bases; the repaired southern Route26 toe strip remains absent. Own verification captures and logs were removed after inspection.

The full reference standard remains unfinished: repeated crown masses and residual row structure still show, and canyon shapes plus browser/all-map evidence need further work. Packs, audio, authoritative heights and gameplay data remain unchanged.

### Use Ice Path fracture fields for outdoor rock shape — 2026-10-11

Compared the native IcePath1F rocks directly after the user's request. Outdoor shaft recesses now sample the same broad planar slab function used by the ice-cave rocks, rather than smooth noise alone. Outdoor crests share Ice Path's actual height sampler at 0.75 strength, restricted to downward erosion and faded at the floor/support datum, authored joins and level plateau interior. Cap and wall use the same deformation. The accepted sandstone and bedrock pigments/textures remain intact. The initial 0.28 crest strength was visually too subtle; unrestricted positive peaks at 0.75 also defeated the existing shoulder regression, so outdoor erosion now cannot raise the authored crest.

Extended geometry checks cover finite positions/normals, source-stable deformation when the viewport shifts, and caps never exceeding their authored top plane. Existing shared cap/wall closure, fixed floor/support, level interior and nonfolding shoulder checks pass. Final full renderer suite: 1,098 passed, 0 failed, 44 ignored. The native verification build succeeded with linker debug-symbol stripping after an earlier normal link failed with disk exhaustion. Shared caches were preserved. Fresh Route26 canyon views at 45 and 135 degrees show additional broken crest relief and no visible openings at the checked stone/floor contacts.

The southern Route26 45-degree view also retains the earlier toe-wall fix: the obsolete jagged floor strip remains absent. Own comparison captures and verification logs were removed after inspection.

This is a partial shape improvement. Long canyon plateaus remain too rectangular beside Ice Path, and isolated outdoor rocks still need stronger individual silhouettes. The complete forest reference and browser/all-map verification remain open. No pack, audio or authoritative gameplay data changed.

### Broader fractured crests and angled wall shoulders — 2026-10-11

Continuous outdoor crest recession now reaches 0.48 source-tile width rather than 0.22, with amplitude sampled from the shared ice-cave planar slab field at 0.45 map frequency. The inward fade, datum fade, plateau center and authored junction anchors remain. Shaft recession now has linear lower/upper shoulders around a broad mineral face, replacing the sinusoidal rounded recess. The existing downward Ice Path crest-height field and accepted pigments/textures remain.

Extended the canyon shoulder regression to reject inverted cap triangles at the original source phase and three additional positive/negative coordinate phases, while checking fixed lower contacts and finite geometry. Native build succeeded; fresh Route26 canyon views at 45 and 135 degrees show wider crest breaks and less rounded shaft profiles, with the checked ledge/cliff contacts remaining closed. A full rerun encountered a separate cave-hatch assertion comparing an elevated recess to zero; corrected the assertion to compare against its authored datum, consistent with that test's existing exact depth checks. Final full renderer suite: 1,099 passed, 0 failed, 44 ignored. Own verification logs and captures were removed after inspection.

The broad plateau outlines are still too regular, and the forest reference plus browser/all-map evidence remain unfinished. This pass does not prove the full outdoor standard achieved. No packs, audio or gameplay data changed.

### Fracture whole outdoor rock tops — 2026-10-11

The shared Ice Path height field now shapes blocked outdoor rock interiors as well as exposed crests. Interior strength is 0.55, with the existing stronger 0.75 crest blend; a 0.8-tile downward bias avoids large unchanged flat patches. Relief fades above the highest floor/support datum and cannot lower a point by more than 35 percent of its remaining height above that datum. Horizontal interior footprints remain fixed. The whole-mass pass also works where a cropped view contains no exposed rim, but requires qualifying Johto rock sources so unrelated cave/coast-only surfaces do not gain this deformation. The source drawings containing planted trees ($6a/$6c) retain a level seat with a one-cell smooth transition around them. Accepted texture/pigment assets remain.

Replaced the former fixed blocked-interior expectation with checks for actual interior relief, unchanged X/Z, level planted-tree seats and source-stable viewport translation. Added a cropped-rock-interior regression that requires varied cap heights without inventing a viewport-edge recession. Existing winding, cap/wall shared-point closure, fixed lower contacts and support checks pass. Final full renderer suite: 1,103 passed, 0 failed, 44 ignored. Native compilation initially failed from disk exhaustion; after space recovered, the verification build with linker debug-symbol stripping succeeded. Fresh Route26 canyon views at 45 and 135 degrees show height variation across the complete rock tops. The southern 45-degree view shows irregular gray-rock crowns and retains the earlier floor-strip fix. Checked contacts remain closed. Own captures and verification logs were removed after inspection; shared caches were preserved.

The overall goal remains incomplete. Some canyon corners still read as authored right-angle masses, forest crowns remain too repeated and column-like beside the reference, and browser/all-map verification is still missing. No content pack, audio or authoritative gameplay data changed.

### Broader route canopies with unequal leaf masses — 2026-10-11

Route boundary crown spread now uses 0.90 plus 1.3 times grove density, replacing 0.82 plus 1.2 times density; interior grove spread remains 2.0 plot spacing. Crown X/Z radii use 0.62/0.58 spread, with Y limited by 0.26 nominal height or 0.42 spread. Crown centers sit at 0.54 nominal height. This replaces the upright oval proportions with broader overlapping crowns. Route trunk tips and branch/fork elevations lower with the foliage so supporting wood stays inside the leaf masses. Cluster dominance now varies with the stable tree phase, rather than every tree sharing identical lower/upper cluster weights. Root positions, bark plot bounds, source ownership, pigment palette, leaf count and town-tree proportions remain.

Added an actual core-geometry breadth check across thirty positive/negative source placements: both horizontal spans must exceed vertical span by 15 percent. Existing source bark bounds, analytic-normal finite differences, winding, deterministic geometry/colors, canopy depth and cutaway checks pass. The first test compilation encountered a missing cutaway field in a concurrently edited apron fixture; that fixture was corrected in the shared checkout before the fresh run. Final full renderer suite: 1,103 passed, 0 failed, 45 ignored. Native build succeeded. Fresh Route26 and Route29 45-degree views show broad overlapping canopy masses and shorter exposed trunks; Route26's repaired toe strip remains absent. Own captures and logs were removed after inspection.

The forest still falls short of the reference: rows remain apparent and the leaf surface detail is too coarse. Canyon corner shapes and browser/all-map verification remain open. The overall goal is not complete. No packs, audio or gameplay data changed.

### Pointed folded route leaves — 2026-10-11

Route crown leaves now have elongated pointed blades, a raised center fold and unequal tip curl. Their normals are area-weighted from the actual eight fan triangles instead of approximated by radial edge directions, so lighting follows the fold. The projected blade area remains close to the previous oval, with the same 1,920 leaves, nine vertices and eight triangles per leaf. Crown proportions, source scatter, pigment palette, bark anchors and cutaway ownership remain; town/small-tree ovals retain their previous shape and normals.

Added a leaf-geometry regression across four surface orientations, checking elongated silhouettes, fold/curl ordering, retained projected coverage, outward triangle winding and finite unit normals. Existing whole-tree winding, crown gradient, source bark bounds, deterministic geometry/colors and breadth checks pass. Full renderer suite: 1,104 passed, 0 failed, 45 ignored. Native build succeeded. Fresh Route26 and Route29 45-degree views show pointed foliage detail replacing the previous rounded patches; Route26's repaired toe strip remains absent. Own verification captures/logs were removed after inspection.

The goal remains incomplete: grove rows and repeated crown clustering still show, and the reference's richer leaf/lighting detail is not yet matched. Canyon corner shapes and browser/all-map verification remain open. Located the existing production browser preview build in tools/new-bark-3d-web.sh for subsequent cross-platform inspection; no browser build or visual verification is claimed by this pass. No pack, audio or gameplay data changed.

### Browser outdoor previews and mesh allocation fix — 2026-10-11

Added Route26 southern clearing (10,54), Route26 canyon (10,28), and Route29 (50,9) production-browser art presets at 45 degrees. They require multiplayer=off and disable save persistence. The builder accepts an explicit WASM_BINDGEN executable and rejects versions other than Cargo.lock's 0.2.128; it retains the core-named server input and also writes the browser-named static copy. The server derives the clock/customization endpoint from the core input. An initial diagnosis that the server pack filename was wrong was corrected after inspecting that derivation.

The fresh browser build initially crashed. A name-preserving diagnostic WASM stack identified std::alloc::rust_oom inside occluder_fade::select. Selection now remaps its owned index buffer in place and allocates vertex attributes at their exact selected count. Preparation releases the full source and owner/remap buffers before building cutaway trees and transparent leaf meshes. Geometry, leaf count and ownership remain. A regression proves index-buffer reuse, exact attribute order/capacities, ownership ranges and remap reset. Full renderer suite after this fix: 1,110 passed, 0 failed, 47 ignored. Browser JavaScript tests: 94 passed, 0 failed. Fresh WASM/bindings, Rust server and artwork bundles built successfully.

Actual live browser screenshots then verified the southern Route26 clearing and canyon at 45 degrees with detailed foliage and stone present. The canyon screenshot still showed broad rectangular masses, so this is cross-platform render evidence, not approval of the art standard. Route29 navigation was initiated but its observation was interrupted; no Route29 browser success is claimed. The server and tool handles subsequently disappeared on continuation and process inventory confirmed no server or build jobs remained. Browser/all-map and alternate-angle verification remain incomplete. No content pack, audio or gameplay data was committed.

### Diagonal outdoor corner breaks and softer material transitions — 2026-10-11

Compared a fresh native IcePath1F view directly. Exposed outdoor convex corners now use Ice Path's diagonal compression (0.4 strength), replacing circular corner projection. Radius varies in source coordinates from 1.4 to 1.9 tile widths and is bounded by 0.49 of each available same-height run. Run inspection extends to six cells. The initial larger cuts overlapped across a narrow two-cell mass and inverted a cap at source phase (17,43); the half-width limit fixed that verified failure. Existing floor/support datums and common cap/wall position field remain. A new four-by-four canyon fixture requires broad actual corner displacement, positive area for every cap triangle across four source phases, and varied map-position silhouettes. All nine outdoor-stone regressions pass.

The user requested an Ice Path target image through imagegen, rejected the first uniform chip-heavy result as lacking taste, then directed continued outdoor work toward the second image with better blending. These generated images are concept references, not evidence of implementation. Outdoor shading now uses triplanar weights squared rather than fourth-power weights, mixes 30 percent sandstone paint into canyon cap pigment, retains 82 percent painted texture against the mineral palette, and reduces small-scale grain/fissure contrast. Accepted embedded texture assets remain; cave material shading is unchanged. Fresh Route26 canyon views at 45 and 135 degrees verify the implemented broader corner breaks and softer cap/shaft transitions, with checked floor contacts still closed. A fresh southern Route26 view retains the removed obsolete toe-wall strip.

The southern capture exposed a remaining bank-to-lawn lighting boundary. Cap normals now blend from level-lawn Y at the north toe to the actual slope through the first source row. Positions, linear rise, footing heights and sharp south earth face remain. The ledge regression now explicitly proves toe normals meet level lawn and the front face stays sharp; all five ledge tests pass. A fresh southern 45-degree render confirms a softer lighting join, although a broad tonal band remains and needs further material/ground work.

Native builds succeeded. Intermediate full-suite runs passed the outdoor checks but failed separately in cave-corner and department-store fixtures. A later shared interior edit temporarily referenced a nonexistent Document revision field; current source resolved that mismatch before retry, without an edit here. The final full renderer run passed 1,120 tests, 0 failed, 48 ignored. Own comparison captures and terminal verification logs were removed after review; shared build caches were preserved.

The goal remains incomplete: isolated rocks retain some rectangular outlines, the remaining bank tonal band and broader ground/material blending need refinement, woodland still has repeated rows/clusters, and these latest shape/shader/normal changes have not yet been rebuilt and checked in the browser or across all outdoor maps. No packs, audio or authoritative gameplay content changed.


### Ground feathering and National Park masonry — 2026-10-11

A regression reproduced the lawn/path blend jumping back to unblended pigment at its physical band cutoff: map-coordinate wobble kept mixed paint alive at the cutoff. Blend color now feathers to the owning material across the outer half of the band. All nearby verified edges contribute with distance-squared weights, avoiding nearest-edge switches at path corner bisectors. Both new continuity regressions and the three existing ground-blend regressions pass. Fresh Route26 southern and Route29 native views were reviewed. The Route26 bank tonal band remains: its raised caps receive coarser finishing than Route29's special-cased caps. This mismatch is identified but not fixed in this pass; terminal slopes must retain their actual corner heights when receiving finer detail.

At the user's National Park steering, a fresh native 45-degree center view exposed checkerboard pavement, neon blue pool borders, flat blue spray tiles and overly uniform grass beds. Only park tile 00 had previously received pavement finishing; native 06 and 16 remained atlas pixels. Hand-reviewed exact source masks now identify exposed pavement and pool coping in twenty native park drawings, excluding props, water, spray, unknown art and changed source positions. A read-only comparison against the ignored pack confirmed all twenty masks. The pool's recessed vertical coping faces now share the masonry finish only within their original zero-to-water depth. No collision, footing, ownership, layout or animation data changes.

Ten focused park regressions pass, including all plaza samples, unchanged pool-wall geometry/normals/footing, source identity guards and existing fixture ownership tests. Native builds succeeded. Fresh NationalPark captures confirm continuous restrained paving and the removal of the neon border, including its recessed faces. The new regression fixture initially had an incorrect UV tuple order; corrected to the production geometry contract before the passing run. Final full renderer suite: 1,112 passed, two failures, 48 ignored; failures are a cropped-rock relief fixture and bike floor-rooting. Earlier this pass the full suite instead reported arcade seating and bike floor-rooting. The cropped-rock failure requires investigation before claiming the outdoor checks are fully clear.

National Park still needs substantial work: the central live spray appears as flat bright-blue atlas cells around a small basin, pavement is too uniform, planting edges remain rigid, and the wider tree/bench/fence surroundings need camera review. The broader outdoor goal remains active, including Route26 cap detail, stone shapes, woodland repetition and current browser/all-map verification. Temporary capture/log files were removed after review; shared build caches and ongoing work were preserved.


### National Park concept selected as a goal reference — 2026-10-11

The user explicitly requested a nuanced, alive modern 3D remake of the original 2D National Park, then emphasized the different outer trees and benches. Built-in imagegen used faithful native 2D views of the northern plaza and southern tree/bench promenade, plus the current wide 3D pool view as a spatial reference. The generated target and exact prompt are saved in `map-concepts/national-park-remake-v1.png` and `map-concepts/national-park-remake-v1.md`. This is now a reference for the active outdoor rendering goal. It depicts broad umbrella park crowns with rooted trunks, pale slatted benches, warm masonry, a stepped pool with live-looking spray/ripples, hoop fencing, flowerbeds and nuanced planting/light. Its composed camera is an aesthetic reference; authoritative source positions must remain. No claim that the renderer implements this quality yet.

After the earlier full-suite failure, an attempted smooth bounded conversion of the shared Ice Path height field retained downward-only outdoor erosion, but the cropped-rock relief regression still failed. The attempted conversion was reverted after that failure; a proper correction still needs a focused check. The follow-up full suite reported 1,111 passing, three failing and 48 ignored. The other failures were bike floor-rooting and an embedded Wigglytuff raw-model byte mismatch during concurrent asset work. Do not interpret the generated concept as verification of those changes.


User review of the saved National Park concept: close enough as the visual target, but the Poké Ball centerpiece shape is wrong and must preserve the original two distinct grass types. Retain the existing render as art reference; correct the source silhouette and grass distribution in implementation rather than copying the generated layout. These corrections are also recorded beside the saved image.


### National Park's two grass types — 2026-10-11

Previous goal turn made progress by saving the user-requested concept and recording the required correction to the original Poké Ball centerpiece shape and two grass types. Current work retains the original source-cell distribution: park block 03 receives the short-grass style, block 13 the long-grass style, only after the existing native grass identity check succeeds. No map geometry, collision, encounter identity or footing changes.

Short grass now uses finer folded blades, a cooler deep-green palette and height 0.82 tiles; long grass uses broader, more curved leaves, warmer olive/lime pigment and height 1.25 tiles. Boundary blades fade to 72 percent height at the outermost root distance, reaching full height inward, while verified shared encounter edges retain connected foliage. Ordinary meadow generation keeps its existing parameters and colors. A regression checks three map phases for finite contained geometry, floor-rooting, visibly different heights, palette contrast and broader long-grass leaf sections. All twelve grass regressions pass.

A fresh native build succeeded after a shared build-lock wait (7m15s total). Reviewed fresh NationalPark center and southern promenade (20,40) views at 45 degrees. The center render visibly distinguishes cool short grass and warm long grass in the existing source footprint. It still has uniform lawn underlay and a tiled/rigid planting impression compared with the reference. Southern review exposes remaining flat pixel artwork under benches and along fences, chunky clipped hedges, small repeated park crowns, wood-colored benches against the pale-slatted source/reference and a coarse pedestal fountain. The outer large park trees currently reuse the KantoCanopy model; they still need a dedicated spreading park silhouette. The original bench selector claims only the upper two rows of block 0e; its remaining 27/28/29/2a row deserves source review for the visible leftover backing before changing coverage.

The goal remains active. Current work does not resolve fountain spray, park furniture/fencing/trees, Route26 bank detail, stone relief/shape issues, forest repetition or current browser/all-map verification. The full suite was not rerun for this grass-only pass; earlier cropped-rock and other shared-area failures remain to revalidate. Own terminal verification logs/captures were removed after review; the saved concept and shared caches remain.


### Complete National Park benches and pale slat materials — 2026-10-11

Previous goal turn made progress with source-distinct grass geometry/colors and native review of the southern promenade. The next source review corrected an old bench assumption: block 0e's drawing is 4x3, not 4x2. Tiles 27/28/29/2a form the feet/shadow row, while the fourth row is plaza backing. The exact source selector now includes all three drawing rows and rejects out-of-range columns. The existing grouped placement claims those twelve cells; the model fitting bounds explicitly remove the extra row so the actual bench size and position stay unchanged. The formerly exposed painted feet receive the same verified plaza ground as the upper drawing. No collision, footing, actor, map or encounter data changes.

The existing bench runtime mesh now has pale gray seat/back slats and a dark charcoal frame, replacing its warm timber palette in line with the 2D source and saved reference. All positions, normals, indices and bounds remain. Named park-specific materials in the original modeling recipe reproduce this palette without recoloring other kit assets. The recipe and JSON asset pass parsing, finite vertex/color and valid index checks. The existing Blender archive was not regenerated in this pass; the local Blender executable is unavailable, and the edited recipe records the intended material update.

All twelve focused park tests pass. A new production append regression proves twelve drawing cells are claimed, the final plaza row remains unclaimed, clean paving replaces all claimed rows, footing is unchanged, the model stays inside its previous 4x2 fitting bounds, and mutating any lower feet cell rejects the whole bench. Native builds succeeded after verified waits on shared Cargo jobs (12m51s and 12m04s including lock/dependency work). A fresh NationalPark southern (20,40), 45-degree native screenshot confirms pale slats/dark frames, unchanged bench placement and the absence of the former black/white painted feet strip. This does not prove the broader reference standard.

The full renderer suite was run directly from the exact test executable produced by the passing Cargo park run: 1,114 passed, five failed, 48 ignored. Failures: cave rim bounds, cave shelf side bounds, shallow cave terrace face bounds, cropped outdoor rock relief and bike floor-rooting. These remain open; no overall green-suite claim. The park tests passed within that full run. The reference goal remains active: flat park fence art, coarse repeated park trees/hedges, sparse flowerbeds, fountain/water detail, terrain/stone issues, woodland repetition and current browser/all-map checks still need work. Own terminal test/build/capture scratch output was removed after review; shared caches and ongoing jobs were preserved.


### Native park hoops replace incorrectly classified hedges — 2026-10-11

The source investigation made progress by reading the exact ignored-pack park atlas and reviewing a faithful 2D southern promenade capture. The former “light/dark hedge” drawings (25/26/35/36 and 23/24/33/34) are pale hoop rails with low gold posts. They are not foliage. A read-only native map scan finds 38 north and 42 south complete drawings in both NationalPark and NationalParkBugContest. The source selector is now named hoop_fence_local. Its complete grouped placement checks each quadrant, source block and phase rather than accepting a repeated or displaced glyph.

The production exterior append replaces the old dense green hedge model with Rust-generated open hoops and muted gold masonry posts. Curved eight-sided rails share rings at every bend; the two native overlap variants use their appropriate front/back border position within the existing 2x2 source footprint. Native lawn backing and existing ownership remain; no map, encounter, collision or footing changes. The historical ParkHedge catalog key remains for compatibility, but these placements bypass that old asset and use the park_hoop_fence presentation label.

Fourteen focused park regressions passed, including finite nondegenerate open geometry, floor-rooted posts, four-cell source coverage, unchanged footing/backing, duplicate-quadrant and phase rejection, and duplicate-append refusal. A fresh native build finished successfully after eight minutes including shared Cargo waits. A new NationalPark (20,40), 45-degree capture shows pale open rails in place of the prior bulky green blocks, beside the unchanged benches and trees. The separate longitudinal fence strips and flowerbed edging remain flat atlas drawings; the screenshot clearly identifies them for the next pass. Trees still need wider park-specific crowns, and the flowerbeds/fountain/material blending remain below the saved concept.

The broader suite ran from the exact executable produced by that passing Cargo park run: 1,120 passed, three failed, 49 ignored. Remaining failures were shallow cave terrace bounds, cropped outdoor rock relief and bike floor-rooting. This is not an overall passing-suite claim. Afterward, the fence grouping was hardened further so a valid outer-tree quadrant cannot substitute for a missing fence quadrant; the append also revalidates the drawing before writing geometry. The final focused recheck (tool session 46847) subsequently terminated with exit 101 before reaching this crate: twelve compilation errors in crystal-render-api, including streamed-image imports, MessageReader and GpuImage fields. Its diagnostics reference Bevy 0.14.2 dependencies while the current shared manifest now selects Bevy 0.20; the migration changed during its artifact-lock wait. That final guard expansion remains unverified. The source-geometry checks and native review above predate this shared migration. Revalidate the current dependency/source state before another run; there is no live handle left to poll.

Completed diagnostic source atlas/captures and earlier test/build logs were removed after review. The final recheck log was removed after recording its terminal failure. The saved concept and shared caches/jobs are preserved. The full outdoor goal stays active: remaining park border geometry, trees, flowerbeds, fountain detail, source centerpiece/two-grass fidelity, Route26 and stone shape/relief issues, woodland variation and current browser/all-map verification remain required.

A new focused park check was started against the current Bevy 0.20 manifest after that terminal failure. Tool session 41643 is live and waiting on the shared Cargo artifact lock; target/outdoor-park-hoop-bevy20-test.log is the only retained task scratch. Resume by polling that handle and inspecting its output, rather than replacing a still-live job. This new run has not yet proved the final source guard or current migration compiles.


### Upright park side fences and source-complete corner turns — 2026-10-11

The preceding goal turn made progress by correcting the false hedge interpretation and reviewing native open horizontal hoops at 45 degrees. This pass replaces the separate flat side-strip drawings in native blocks 10/12/14/16/18/1a with upright rails and low posts from the same kit. Hand-reviewed complete block guards include the native lawn and horizontal corner art. Each placement reserves only its eight strip cells (2x4); the other half of the block, including horizontal hoops, remains available to its own renderer. North/south turns add the native inner corner post. The outer strip receives verified park paving, while its inner strip receives the specific native green lawn sample. No source-map, encounter, collision, actor or controller changes. The append leaves footing data untouched.

Selection requires complete source identity and world block phase, both native ground samples, an unclaimed strip, and no raw custom profile anywhere in the guard block. Incomplete or modified blocks retain their original drawing. The placement revalidates the whole guard before appending, and refuses a second append. New renderer fixtures cover all six variants, eight-cell ownership, native underlay, unchanged footing, every changed source quadrant, phase, duplicate append, custom backing and missing ground. These integrated fixtures have not yet run successfully.

An independent checker compiled and executed the actual production source matcher against the ignored local pack: all six block drawings agree, and each National Park variant contains 41 eligible source blocks. Its source-guard unit test passes. A separate temporary harness compiled the actual production fence geometry with the available Bevy 0.20 math library and the original surface/quad bookkeeping. Three isolated checks pass: the production source guard, original horizontal hoop geometry and all six rotated side variants. Side positions/normals/colors/UVs have equal finite lengths, normals are unit length, triangles have positive area, posts reach zero, and geometry stays inside the 2x4 footprint. These checks prove the local geometry/source code, not a full rendered frame, integration ownership or gameplay. Both harnesses and their generated executables were automatically removed from external temporary directories.

The prior Bevy 0.20 park recheck (session 41643) terminated before tests with 34 compilation errors elsewhere in the migrating renderer, including capture extraction, moved render imports and a retired shader include in a test. No passing full renderer claim. The native side-fence build (session 65856) waited on the shared build queue and then began compiling Bevy 0.20 dependencies. Actual available disk fell to about 3 GiB during the ongoing migration/disk cleanup. The owned Cargo process was stopped with SIGINT and its handle confirmed terminal exit 130. No owned Cargo/native job remains to poll. Defer another full build until the migration and disk environment are ready. A fresh 45-degree native side-fence view is still required before accepting the corner placement, color and scale.

Only this task's obsolete target/outdoor-browser-preview bundle (209 MiB, old pre-migration browser output) was removed after checking for open files and matching serving processes. Source, saved art, original content-packs, shared target/debug and target-bevy20, the small browser data directory and wasm-bindgen tools were preserved. Completed task test/build logs were removed after recording the failures.

The full outdoor goal remains active. Side-fence native review, flowerbed edging, dedicated spreading park trees, fuller flowerbeds, fountain/spray and water, ground blending and the corrected native Poké Ball/two-grass centerpiece still need verification/improvement. Route26 cap/ledge/stone defects, cropped rock relief, broader woodland variation and current browser/all-map rendering also remain.

### Flowerbed timber edging and raised-cap finish — 2026-10-11

The source-complete flowerbed renderer now recognizes the complete native 3c/3d/3e drawing and gives its perimeter low beveled warm timber beams with four corner posts. It owns only border cells; the native flower/lawn interior remains available to its own renderer. Both park variants contain one eligible 20-by-4 bed: 44 border cells and 36 interior cells. Native paving outside and native green lawn inside meet beneath the timber, at the original floor datum. Whole-drawing source identity, phase, custom-profile priority, ground samples and duplicate-append guards remain mandatory.

Earlier isolated checks executed the production matcher against the ignored pack, checked the production timber geometry with Bevy 0.20 math, and checked the production ground split: 92 quads cover 2816 square world units, all at Y=0, with 20,480 coverage samples finding no holes or overlaps. These are local checks, not evidence of a fresh integrated render. Integrated park checks and native review remain pending. The previous test/build handles 18813 and 60774 are now missing; their retained logs contain no completed result. A new focused park run is live as tool session 73092 (Cargo PID 4481), waiting on the shared build directory. Another chat owns the live native example build (PID 3955, target/cave-review/native-render-build.log); do not cancel or duplicate that build.

Raised terrain had a separate material-detail inconsistency: the fine lawn/path finish was allowed on raised surfaces only for Route29. Other native caps, including Route26, retained coarse four-corner color sampling. The finish now also accepts exact native shallow-bank sources at their verified positive cap height. Original cap corners and footing stay unchanged; walls and sloped faces retain their existing path. A new regression creates a Route26 native raised path and checks subdivision, original corners, unchanged footing and no geometry below its cap datum. Its focused run is live as session 45912 (Cargo PID 4925), waiting on the shared build directory. This change addresses material continuity; it does not establish that the reported ledge Y misalignment or silhouette is fixed.

The saved National Park art and recorded original Poké Ball/two-grass correction were re-opened and verified present. Dedicated spreading park trees, richer planting, fountain detail, remaining ledge/rock shape issues and fresh native/browser review remain part of the full goal.

### Dedicated outer park trees — 2026-10-11

The prior turn made progress by extending fine ground detail to verified raised caps and adding positive/negative native-datum checks; the live Cargo runs still await the shared build queue. The current park tree branch no longer fits the generic Kanto canopy into the source drawing. It calls a dedicated Rust garden-tree builder: a fluted, flared warm trunk, five forked supporting limbs, and a single broad flattened asymmetric crown using the existing articulated broadleaf skin and folded leaves. Stable world-coordinate variation changes crown phase and proportions. No route/Kanto model or existing tree style changes. Original tree-plot center, ground ownership, native backing and footing remain authoritative.

The append revalidates all sixteen source quadrants before claiming ground or adding geometry. An integration fixture checks the park-specific presentation label, sixteen-cell ownership, unchanged footing and refusal of every single altered tree quadrant without partial output. The existing live park_ test run (session 73092) will include this test and the park-tree geometry test when it acquires the lock; do not duplicate it.

A temporary isolated harness compiled the actual new park-tree module and its actual production branch/crown/variation helpers against the shared Bevy 0.20 math library. Its geometry test passes for three coordinate phases: finite positions, no vertex below the supplied base, rooted trunks, unit normals, valid indices, distinct shapes and a single complete cutaway range. The temporary source and executable were automatically removed. This does not establish successful renderer integration or visual acceptance. The native example build owned by the cave-review chat remains live (PID 3955); a fresh park 45-degree image is required to judge crown spread, trunk scale, blending and color against the saved concept before accepting this tree design.

### Outdoor crest fractures without clipped ice peaks — 2026-10-11

The previous turn made concrete progress by adding dedicated park-tree geometry, source revalidation and integration coverage. Current build/test handles remain live in the shared queue. This pass addresses the known cropped outdoor rock regression: the previous field borrowed frozen_point's height, subtracted a constant and clipped positive ice peaks to zero. Crops entirely inside a positive peak could therefore lose all height variation even with no exposed rim.

Outdoor stone now uses its own strictly downward depth field with the same world-anchored broad planar slab primitive and smaller smooth noise. Cap and wall vertices still share one position field. Existing relief fades around planted-tree seats and native support, and the downward limit of 35 percent of available height remains. The Ice Path renderer and its raised crowns are unchanged. A viewport crop still gains no invented horizontal rim. Actual nearest-floor handling for isolated low rocks remains an open issue; this change does not prove that all reported stone shapes are finished.

An isolated Rust harness executed the actual production crest_depth, slab and meadow_noise functions. Two checks pass: bounded downward depth across 40,000 map samples, and retained variation in the existing failing crop phase. The latter produces approximately 0.489 world units of crest-height variation at the cap's 0.55 relief scale, exceeding the existing 0.4-unit requirement. The first candidate amplitude produced only 0.302 and was rejected before settling on the current field. Temporary harness source/executable were automatically removed. The full existing cropped_rock_interior regression is queued as session 64589 (target/outdoor-cropped-rock-test.log); full cap/wall/contact checks and fresh native review remain pending. Park tests 73092 and raised-cap tests 45912 remain live; preserve their handles and poll rather than restarting.

### Hollow northern fountain basin — 2026-10-11

The previous turn made progress by replacing clipped positive ice-crest heights with a bounded downward outdoor field. The current park, raised-cap and cropped-rock jobs remain live in the shared build queue. This pass replaces the production source-complete 3f+33 northern basin kit's filled atlas cap with a hollow stone shell, worn bevel, coping and recessed blue-green water center. Thirty-two shared angular samples connect its outer wall, bevel, coping and inner wall; the closing sample wraps to the first angle exactly. The production kit's source-derived center, 0.88-by-0.65-tile radii, bottom -2 and maximum height 4 stay unchanged. Water stays at the prior cap datum 3, below the coping. Footing, map positions, pool borders, collision and animated spray ownership are unchanged. This is a basin-shell improvement, not completion of the animated water/spray or pool work. The initial integration mistakenly targeted the samples-only diagnostic path; that function and its test were restored before integrating the production kit.

The production integration test still requires four solid source cells, no water/spray ownership and native water backing at -2; it now also checks water at 3 and coping at 4 without a mixed atlas cap. The diagnostic grouped-fountain test remains unchanged. A temporary isolated harness compiled the actual basin module with Bevy 0.20 math and a minimal quad bookkeeping adapter. Its final geometry regression passes: finite contained positions, recessed center, unit normals, positive primary triangle area and four-vertex terrain-face alignment. Water fan triangles use the existing repeated-center quad contract, with intentionally degenerate secondary triangles. The adapter is not full renderer integration evidence. Temporary files were automatically removed. The live park_ run (session 73092) includes the new basin regression and updated production fixture once it acquires the shared build lock. Native build PID 3955 remains live; no fresh basin screenshot yet. Materials, shape, spray and overall park composition must be judged together against the saved reference before visual acceptance.

Build coordination update: the first shared native executable is built, but the screenshot example and this task's three test handles remain live in the queue. Future Cargo verification should use the coordinator's full package union `-p geothite -p crystal-bevy -p crystal-voxel-view -p crystal-render-api --all-features --profile wasm-dev --jobs 2`, with `CARGO_TARGET_DIR=target-bevy20-verify`, `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_WASM_DEV_DEBUG=0`. Adding geothite changes hashbrown/allocator/indexmap features and can rebuild Bevy despite identical renderer flags. Do not replace existing live jobs solely for this cache change. The coordinator will provide compiled test executable paths for direct filtered checks. No current test executable or screenshot example was present at the last filesystem check. Preserve shared caches until all consumers finish; clean only owned completed temporary output.

Latest native-process check: the old cave-review build PID 3955 is no longer present. Its owner has already started replacement PID 13039 using the full four-package union; the new log is `target/cave-review/joined-rim-native-build.log`. That replacement is confirmed live, so do not start another screenshot build. Own tests 73092/45912/64589 remain live. There is still no new_bark_3d executable. The WASM build is progressing through game crates, but its currently present renderer rlib predates the latest basin files, so that artifact cannot yet prove those changes compiled.

Subsequent WASM verification: `target/bevy20-reapply/resume-wasm-game-build.log` now reports a successful optimized game build (17m52s). The new renderer artifact is `target-bevy20-wasm/wasm32-unknown-unknown/wasm-dev/deps/libcrystal_voxel_view-c285ca68d3059444.rlib`, timestamp 12:09:44, after the latest basin files (12:00:09/12:00:23); the older d561 artifact is not the current one. The finished game WASM is timestamp 12:13. This proves the current production outdoor code compiled for the browser build, including trees, basin and stone field. It does not run cfg(test) regressions or establish a rendered frame. A disposable browser preview additionally requires the current generated people/model bundles and audio support; these were not found among the present owned preview outputs, so no partial browser startup is claimed. Native example PID 13039 and own focused test jobs remain live in the shared queue.

Native park run 73092 has now terminated with exit 101 before executing tests. Its only compiler error was an unrelated new Blackthorn channel regression in dungeon_extension.rs: a Vec of four vertex UV pairs was compared directly with GridGeometry::uv's bounds tuple. Corrected the assertion to the actual ordered four-vertex UV array; the production channel append and all gameplay remain unchanged. The raised-cap run 45912 encountered the same pre-fix compiler error but its tool handle is still live while Cargo waits for other jobs to finish; do not restart it before terminal confirmation. Cropped-rock run 64589 remains live and queued.

The replacement park check is session 94025, log target/outdoor-park-current-test.log, using the full canonical four-package union and two jobs. Native screenshot build PID 13039 remains live. No focused renderer test pass or fresh native screenshot yet. Preserve these current handles; old park handle 73092 is terminal.

Raised-cap handle 45912 subsequently confirmed terminal exit 101. Its replacement is session 66940, log target/outdoor-raised-cap-current-test.log, using the same canonical four-package union and two jobs. Cropped-rock handle 64589 is still live; do not replace it. Completed old park/raised-cap test logs and the obsolete owned bed-build log were removed after recording the compiler failure. Current logs, source, saved art and shared caches are preserved.

Additional verification opportunity: another chat's live renderer test compile (Cargo PID 12619, rustc PID 23190 at the last check) writes to `target/handheld-final-verification/wasm-dev/deps`. Its package set includes crystal-bevy/crystal-voxel-view/crystal-render-api with all features. Once its crystal_voxel_view test executable is complete, confirm it includes the new park-tree/basin/raised-cap fixtures and is newer than current relevant source before using direct outdoor filters. Do not assume an rlib or incomplete executable proves tests passed. Preserve this other chat's cache/job. Our canonical park 94025 and raised-cap 66940 jobs and cropped-rock 64589 remain live; screenshot build PID 13039 is still queued.

### Direct outdoor regressions and seam correction — 2026-10-11

The completed renderer test executable in `target/handheld-final-verification/wasm-dev/deps` ran the current park fixtures: 24 passed, none failed. These include dedicated park trees, hollow production basin, side fences, timber flowerbed and source-ownership guards. This establishes integration behavior, not visual acceptance against the saved concept.

Direct checks also exposed one raised-cap fixture failure and two stone failures. The Route26 fixture called the town-only finish wrapper, which does not process Route26; it now calls the production world-surface entry point. Cropped stone relief was still too shallow after the actual height-dependent fade. The downward crest field now uses a bounded tanh mapping of the signed fracture field. An isolated check using the actual production field and the fixture's 16-pixel height/fade yields 0.4507256 world units of relief, exceeding the existing 0.4 requirement without reaching the support clamp. Full cropped-mass integration remains pending; the earlier full-scale 0.489 estimate did not account for this fade.

Adjacent walls starting on different floor heights reconstructed a shared height ring through different floating-point lerps. The wall sampler now assigns the common world-ring Y directly, clamped to the actual face bounds. An isolated Rust harness compiled the actual production append, pigment, noise and quad functions with Bevy math and ran the unchanged adjoining-wall regression: one passed. Exact shared-edge equality, horizontal-first input and reversed winding all pass. Temporary source and executable were automatically removed. The earlier direct binary predates these cap/stone corrections and must not be reused as evidence that they pass full integration.

The queued cropped-rock build subsequently reached an unrelated test compilation error in apron.rs: its final Elder assertion referenced a loop-local rig outside the loop. That assertion now uses the already loaded male rig. Production clothing behavior is unchanged. Current canonical park/raised-cap jobs and the native screenshot build remain live in the shared queue. Fresh full stone/cap tests and rendered Route26/NationalPark review are still required.

The old cropped-rock handle 64589 is now confirmed terminal with exit 101. Its replacement checks the entire outdoor-stone test module using the canonical four-package union, two jobs and shared native cache: session 87335, log `target/outdoor-stone-current-test.log`. Park 94025, raised-cap 66940 and stone 87335 were all re-polled and remain live; native example PID 13039 is also live, with no finished screenshot executable yet. Preserve and poll these jobs rather than duplicating them.

The existing completed renderer binary also ran all 12 grass-filtered checks successfully. The relevant grass.rs/grass_blades.rs source files are unchanged since 10:40, preceding this binary. This includes distinct park grass height, blade width and palette, rooted edges, scrolling consistency and no spill onto paths. Native park short grass remains source block 03 and long grass block 13. These results do not verify the centerpiece's complete 2D silhouette or visual balance in the new park composition; those remain native capture requirements. The binary is still not accepted as evidence for the later stone/cap corrections.

Fresh renderer test executable e61197694d02a097 finished after the stone corrections and apron compile fix. Direct execution passes all nine outdoor-stone regressions, including the previously failing cropped mass and exact shared-height-ring checks. All 24 park checks also pass again on this binary. These are integrated renderer tests, not a visual-quality completion claim.

The corrected Route26 fixture now reaches the production finish and passes its native-height/detail/corner/footing assertions. Its displaced-height subcase exposed an incorrect fixture expectation: the ordinary fallback converts source paint to a coarse vertex-colored quad rather than retaining the atlas quad. The fixture now requires exact original solid positions and exactly six indices at both displaced heights, proving finer cap tessellation is not authorized there. Production finishing behavior is unchanged. This latest fixture correction still needs compilation and execution; do not report the full Route26 test as passing from e61197694d02a097. Existing canonical test handles remain live and should consume the updated source once they acquire the cache.

### Bank perimeter and scrolling stability — 2026-10-11

The broader latest voxel run reports 1145 passed, six failed and 49 ignored; it is not a green full suite. Two additional outdoor failures are the shallow-bank scroll-color regression and native ledge vertex/normal preservation. The bank relief now returns its unchanged perimeter directly: f32 sin(PI) is not exactly zero and previously displaced some boundary coordinates by a few bits. Short normal derivatives are evaluated relative to the bank anchor, and pigmentation samples the corresponding stable source coordinates, preventing viewport translation from altering derivative precision and lighting. The unchanged scroll/perimeter/winding regression passes in an isolated harness executing the actual production append/noise/quad functions. The fixture additionally now requires exact original corner normals and exact scroll-normal equality; full integrated execution of the latest changes is still pending. Temporary harness output was removed automatically.

The shared new_bark_3d native example completed. Actual NationalPark captures at (20,40), 1280x960, modeled 45-degree view and classic 2D both failed the existing screenshot validator: each saved image is a uniform blank frame with one color and zero luma range. Logs are `target/outdoor-park-current-native.log` and `target/outdoor-park-classic-native.log`. These are failed capture diagnostics, not usable review images or proof that park art is rendered. The native binary predates the latest bank-relief correction. Read-only inspection of the upgrade chat confirms another worker is investigating the same uniform main-window screenshot problem in classic and modeled battle views; preserve that worker's shared capture edits. Browser terrain is reported rendering after its Bevy shader fix, so a current browser preview is the next available visual-review route while native capture diagnosis continues.

The strengthened production bank harness now runs both module regressions successfully: exact support-corner position/normal pairs, exact translated normal equality, existing color stability, cardinal winding and support checks. This remains isolated production-function evidence; the full shallow_ledge_finish regression must be rerun after compilation.

Added `national-park` (20,40) and `national-park-pool` (20,24) to the existing Rust location-tester preview dispatcher and browser 3D-start list. Both use orbit 1 (45 degrees), existing multiplayer=off requirement and no preview save persistence. Existing cave preview edits were preserved. Another chat's current WASM build (Cargo 31910, renderer rustc 31980 at the last check) is live; do not start a duplicate or assume these new presets are available in its served bundle before inspecting completed output.

An actual Chrome Route26-canyon preview was opened at the existing local server on port 3007. Loading reached the game page, but subsequent state/screenshot observations timed out. Chrome's own resource warning identified this Geothite tab; making this failed review tab inactive released 2.2 GB. No usable browser terrain frame was captured. No other tabs were changed. Native screenshot diagnosis and current WASM build are still active; neither capture route establishes visual acceptance yet.

### Current browser bundle and retained vertex normals — 2026-10-11

The cave owner's WASM build finished in 4m03s; its retained bindgen output is `target/cave-review/browser`, newer than the park preview source additions. The shared raw WASM cache was cleaned after that build, so no duplicate build was started. Prepared a complete owned disposable preview in `target/outdoor-browser-review` using those bindings, current web-client files and the existing verified model/people/audio/content assets. All remain ignored. Literal-byte searching is not a reliable preset check: even established Route26/cave preset strings are absent from the optimized WASM. Actual runtime selection still requires verification. The owned loopback server runs on port 3013 as session 92502.

Actual Chrome `preview=national-park&multiplayer=off` finished its loading screen, but its canvas is black. Console diagnostics show the Apple M5 WebGL adapter and subsequently `RuntimeError: unreachable` at 18:05:31 UTC. This is an actual failed browser render, not visual acceptance or proof of the chosen map. No successful park frame exists yet.

The newer integrated renderer binary passes both bank-module regressions, including exact scrolling normals. The broader shallow-bank test still fails. An external Rust diagnostic using a matching completed production rlib and the exact existing fixture identifies eight native corner-normal pairs lost during material finishing; their positions are identical. The ordinary paint fallback replicated the first source normal across each quad, discarding sawtooth-approach corner normals. Both unchanged-geometry fallback branches now copy all four original source normals after assigning color. The existing full shallow-bank regression must be rerun on a newly compiled binary; its requirements were not weakened. Temporary diagnostic source and executable were removed automatically.

The Route26 wrong-height subcase also showed that -0.5 and +0.5 cap offsets can legitimately choose different paint domains (solid versus retained atlas). Its assertion now requires exact combined original positions and six total indices across both domains, preserving the requirement that only an exact native-height cap gains fine tessellation. This latest fixture update and vertex-normal fix remain pending full integration tests. Current park/raised-cap/stone Cargo handles are still live; preserve them rather than duplicating the builds.

Park handle 94025 has now terminated with exit 101 on an unrelated crystal-bevy Bug Contest test matched by the broad park_ filter: it loads a nonexistent external core pack via the parent checkout path. Raised-cap handle 66940 has also terminated with exit 101 before tests because the concurrently added dev_tools.rs module references a missing diagrams.rs. These outcomes do not supersede the direct 24 park passes, and they are not passes for the latest cap/normal corrections. Stone handle 87335 remains live. Future focused runs should use renderer-qualified filters to avoid the unrelated contest fixture; preserve the canonical package feature union and current shared development-tool edits.

The development-tool diagrams file is now present. Started the renderer-qualified material-finish regression group (including the cap and native-normal tests) as session 88156, log `target/outdoor-finish-current-test.log`, with the canonical four-package union and two jobs. This is a replacement after terminal confirmation, not a duplicate of either failed job. Stone 87335 remains live.

### Shared approach lighting across detail levels — 2026-10-11

The fresh integrated binary now passes the complete Route26 native-cap regression, including both displaced-height coarse-geometry subcases. The full shallow-bank corner-normal check still fails after the paint fallback correction. Inspection of the actual build order shows that material finishing precedes approach/lip shaping; the later shape pass recalculated corner normals from triangle areas. Thus detail tessellation changed the normal at otherwise identical native corners.

Approach caps now transform their source normals by the differential of the same linear four-row height warp used for positions, with the existing toe-light blend retained. Lip normals now use the cofactor of the existing source-coordinate recess warp instead of averaging differently sized triangles. The lip's positions, amplitudes, half-cell knots, support anchors and footing are unchanged. Non-cap cliff/side geometry retains its existing geometric normal refresh during the height pass. This aims to remove shading seams between coarse and detailed banks; it is a production lighting correction, not weakening the existing native-corner assertion. The latest implementation needs the full shallow-bank and approach regressions on a newly compiled binary; current live finish/stone handles are preserved.

Stone handle 87335 has completed successfully with exit 0: all nine outdoor-stone regressions pass in the canonical Cargo run. Finish handle 88156 remains live. The current unrelated renderer compile started before the latest ledge_approach.rs edit, so its result must not be accepted as proof of that edit without a subsequent rebuild/current source check.

An isolated Rust harness executed the latest production shape_surface, refresh_normals, soften_lips and noise functions on the known native block56 drawing. It supplied minimal terrain/grid bookkeeping and the matching existing source-profile types; eligibility was limited to this known south-bank fixture. Across cap heights 6 and 6.012 and map origins (0,0) and (20,-11), coarse and six-subdivision caps have exactly matching shared position/normal pairs and unit normals. One check passed; temporary source/executable were removed automatically. Added the equivalent behavioral regression to ledge_approach.rs as shared_cap_normals_do_not_depend_on_material_tessellation, using the actual repository fixture helpers. This is regression coverage for detail-independent lighting, not broad map/render proof.

The differential was additionally corrected for tiny grain offsets above the cap (which translate with the ramp rather than compress vertically) and for relieved faces beyond the lip (whose recess is clamped along Z). Geometry and footing remain unchanged. Full repository compilation/execution of the latest test and the existing shallow-bank check remain pending behind live finish handle 88156.

Native capture recovery: read-only inspection of the interiors chat confirms its explicit image-target GPU test now passes after preserving the full ImageRenderTarget scale factor in screenshot readback. The shared render_capture.rs provides opt-in `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, retaining production camera projection, viewport and order and leaving separately owned image cameras alone. Ordinary window capture remains a separate failing/diagnostic path; the image mode is not an automatic fallback. Another chat's fresh new_bark_3d build is confirmed live as Cargo PID 38702, using the canonical package union. Reuse its finished executable with the explicit image mode once its timestamps include this capture fix and the latest outdoor source. Do not duplicate or cancel that build. No actual park/Route26 image-target render has yet been produced.

### Integrated ledge verification — 2026-10-11

The native renderer test executable rebuilt at 13:30:40, after the latest approach normal-field source at 13:22:07. Direct execution passes the full shallow-bank native position/normal preservation regression, all six ledge-approach regressions (including shared cap normals across material tessellations), the Route26 raised-path cap regression, both ledge-finish regressions, and all nine outdoor-stone regressions. Every command returned exit 0. This closes the previously failing shared-normal check without relaxing its exact corner requirements. Actual park/Route26 art acceptance still requires fresh rendered frames; the new_bark_3d build remains live and the existing executable is still the older 12:54:16 build.

The same fresh executable additionally passes the whole new_bark test group (47 passed, 2 ignored) and all 24 renderer park tests. The park checks include distinct source grass shapes/palettes, complete outer-tree plots, benches, flowerbed ownership, fountain basin rings and coping. These support geometry/source fidelity, not a claim that the visual target has been met.

### First recovered park image and paving revision — 2026-10-11

An ephemeral locally linked new_bark_3d executable using the newly completed native libraries captured NationalPark (20,40), orbit 1, with explicit image-target mode. Link and capture both returned exit 0. The native validator reported 2560×1920, 35,999 colors and luminance range 252.9; actual image inspection confirms a visible scene. Temporary executable/source workspace was removed automatically. The ignored review image is target/outdoor-park-image-native.png. This is the first usable current park capture, not accepted final art.

Compared with the saved reference, paving remains broad uniform gray squares, tree trunks lack grounded roots, grass is too regimented, flowerbeds are sparse, and the distant pool still has a bright blue atlas spray. Park paving now reuses the existing world-anchored limestone flag partition with warm varied colors and worn edge strips, restricted to proven park masonry source cells in NationalPark and its contest variant. Source layout, heights and claims remain unchanged. The existing plaza regression additionally checks exact total source-floor coverage. This newest paving edit needs rebuilt tests and an actual after-image.

Finish session 88156 has completed with exit 0 (47 renderer material-finish tests passed, 2 ignored), before the new paving revision. A second direct capture attempt for the northern pool/Route26 failed to link because dependencies changed concurrently (E0463 crystal_bevy); it produced no new frames and was not retried against guessed artifacts. The canonical new_bark_3d build is still live as PID 38702. A focused plaza test is queued in target/outdoor-park-paving-test.log after terminal confirmation of the previous owned test job.

### Park roots and recovered canyon review — 2026-10-11

The canonical new_bark_3d build completed (executable 13:37:51). Its explicit image mode successfully captured Route26 (10,28), orbit 1: exit 0, 2560×1920, 99,881 colors, luminance range 227.6, 41.40 seconds. The ignored frame target/outdoor-route26-canyon-current.png was visually inspected. Canyon rocks remain too block-like; gray shoulder surfaces abut ochre cliff bodies abruptly, and the regular bank/floor transitions still need source comparison. This frame is diagnostic evidence, not visual acceptance.

Park trees now include six closed tapering buttress roots contained within the native tree plot and existing cutaway range. No source placement, ownership, footing or crown was changed. An initial root helper incorrectly called a private mesh function; the focused paving run caught this transient compile error and exited 101. The helper now appends its own indexed faces with geometric normals. Replacement test session 83924 is running in target/outdoor-park-roots-paving-test.log. Added a root geometry check for exact floor contact, contained bounds and matching nondegenerate face normals; it may require another build if its source changed after compilation began. Actual post-root/post-paving rendering remains pending.

The completed canonical executable also captured and visibly rendered the matching Route26 classic 2D frame and NationalPark northern pool (20,24), both exit 0. The 2D canyon confirms the little projecting rock shoulders are authored features, so their footprints should remain while their form/material joins improve. The northern park image visibly includes the new staggered limestone paving and both grass types. The blue cross-shaped live spray is still exposed around the hollow basin, and pool water reads as a flat opaque plane. Those are concrete remaining fountain problems; neither this capture nor the successful paving appearance completes the park target. The executable predates the root revision, so its trees cannot validate the roots.

### Source-complete fountain spray replacement — 2026-10-11

The guarded northern fountain now owns its exact 12-cell cross (four basin cells plus eight native spray cells), with all 32 backing cells still required to match before replacement. Ordinary pool-water cells remain unclaimed. An isolated execution of the actual source-index function passes exact cross ownership and preserves bin/pedestal counts; temporary files were automatically removed. Native atlas spray is replaced with a fine central jet and eight curved water threads, with shared ring vertices along each curve and muted water highlights. Added full geometry coverage for finite positions, basin-contained endpoints, above-bowl rise and unit normals. These newest source/geometry changes await rebuilt tests and an actual render; streams are currently static geometry, so living motion remains unfinished.

Session 83924 ended with exit 101 on the unrelated crystal-bevy external-pack Bug Contest fixture. Direct inspection of its renderer executable e7f760a35a1e6c2b confirms the generic park-tree geometry/cutaway check passes with the roots, but its added root-specific test was not included. The plaza coverage assertion failed when accumulating hundreds of small face areas in f32; its area calculation now uses f64 coordinate differences, retaining the same 0.001 tolerance. This change still needs execution before accepting that diagnosis. A fresh canonical no-run test build is live as session 89037, log target/outdoor-fountain-current-test-build.log; subsequent direct renderer tests will avoid the unrelated contest test. No new visual acceptance is claimed.

An isolated harness executing the actual city_paving.rs append/clip/partition functions over the exact 4×4 plaza confirms f64 area 1024 and f32 accumulated area 1023.99774. It used minimal geometry bookkeeping and the production lattice function; temporary source/executable were removed. This confirms the precision diagnosis without relaxing the coverage tolerance. The existing full city-paving module also passes all three tests in the completed renderer binary, including exhaustive interior coverage probes and scrolling phase. Full post-fountain integration and imagery remain pending.

### Actual root/spray imagery and underlay correction — 2026-10-11

The newly linked native executable (latest completed crystal_bevy 2e368ffdc6ee52a2 library) successfully captured both NationalPark pool (20,24) and southern trees (20,40), explicit image mode, orbit 1. Both link/capture commands returned exit 0; temporary executable was removed. Visually inspected target/outdoor-fountain-spray-current.png and target/outdoor-park-roots-current.png. Buttress roots and arcing spray are visibly present. However, the blue cross persists as the replacement's water underlay, not the original spray hull: ordinary water material validation rejects water painted under formerly non-water spray/basin sources.

Added a narrowly guarded park basin water-underlay binding: NationalPark/contest maps, park water sample 0x14, existing complete-fixture ownership label, basin source blocks 0x3f/0x33 and exact basin/spray tile identities. An unclaimed spray source cannot acquire this binding. Added a regression for owned versus unowned cells and unrelated map rejection. This latest correction needs execution and an after-image; the captured blue cross is not accepted or hidden.

The updated renderer executable at 13:46:51 passes both root tests, both basin/spray tests and all three fixture-ownership tests. Its plaza failure still reports the old single-precision assertion line, preceding the 13:44:18 source edit at compiler start; its late executable timestamp alone is not proof that the f64 edit compiled. Fresh canonical session 89037 remains live and will provide a current source build. Park water motion/depth, foliage and canyon form remain unfinished.

### Fountain binding verification and canyon material continuity — 2026-10-11

The refreshed renderer executable at 13:53:21 now passes all three park material tests: complete-fixture water underlay binding, recessed pool coping, and exact plaza area with f64 accumulation. The new underlay test actually exists and ran, so the earlier stale-executable failure is superseded. A fresh after-image is still required to confirm that the blue cross disappears in production.

Route26's proven native block0x0a rock platforms now use the same ochre canyon mineral family as their adjoining cliffs. Other maps retain their existing rock finish. Restored clipped cap corners and their closing side sections inherit the existing source cap's mineral family instead of inserting gray repair faces. The material switch preserves the same tessellation, footprint and support; added a regression comparing Route26 and Route29 positions, normals and footing while requiring their intended different mineral tags. This new regression is absent from the 13:53:21 executable, so the latest canyon edit is still pending rebuild and imagery. Canonical no-run build session 89037 remains live; no duplicate build was started.

### Park water motion and current verification — 2026-10-11

Canonical no-run session 89037 completed with exit 0. Its completed renderer passes all 27 park tests and nine outdoor-stone tests. A newer renderer executable at 14:03:03 also contains and passes the Route26 shoulder pigment/support regression, the new park water glint coordinate/floor regression, and all four park material tests. These results establish geometry/source/tag invariants, not GPU shader validity or final appearance.

Verified park water now emits dedicated source-coordinate tags for a restrained moving blue-green pigment field, replacing sparse static surface stripes only in NationalPark/contest. Bowl water carries bowl-relative tags; water threads carry their arc distance and per-stream phase for flowing glints. The surface shader reads Bevy20's existing globals.time (confirmed in local mesh_view_bindings.wesl) without changing geometry or authoritative gameplay. Native cave water retains its original mask and authenticated phase path. No refraction or live canopy reflection is claimed yet.

The full water/shader build is live as session 85451, target/outdoor-water-glints-test-build.log. Attempts to link an ephemeral native capture executable returned E0463 and produced no frames. Fingerprint inspection now identifies the precise stale dependency: crystal-bevy-2e368ffdc6ee52a2 still expects a crystal_voxel_view fingerprint which no longer exists, while its other observed dependency fingerprints match. The parent library must rebuild after the changed renderer; waiting on the live canonical job avoids more direct-link guesses. Temporary link workspaces were automatically removed. The latest bowl tag edit may postdate the currently completed test binary and still needs fresh geometry/GPU verification. Blue-cross removal and water motion remain visually unverified.

### GPU water/canyon review and temporal capture mode — 2026-10-11

Session 85451 completed with exit 0. The current completed renderer passes all 28 park tests and the Route26 shoulder pigment/support regression. The fresh NationalPark classic capture (20,24) was visibly inspected: upper grass is long, lower grass short, with the stepped original pool outline. The source layout remains authoritative over the saved concept's incorrect centerpiece.

The canonical native executable rebuilt at 14:15:14. Actual explicit image-mode NationalPark capture returned exit 0 and a visible GPU image, 2560×1920, 27,424 colors, luminance range 240.4 (target/outdoor-park-water-current.png). The blue cross is gone, the bowl and thin spray are visible, and the water shader renders without a GPU shader error. Highlight spacing is too regular, so the latest shader revision introduces slowly drifting continuous noise to break the ripples, lower glint strength and add restrained green tonal variation. Those revised pixels have not been rendered yet.

The second image from --second is the tool's default 2D comparison (21 colors), not temporal evidence. Added opt-in CRYSTAL_RENDER_TEST_SECOND_VIEW=same for one-shot capture pairs, leaving the default toggle behavior intact and rejecting same-view live-review mode. Both views use the existing camera/readback settling path; this mode permits an actual same-camera water-region comparison. Updated the capture test fixture's field initialization. Rebuild and actual paired captures remain required to verify motion.

Actual Route26 canyon capture with the 14:15 executable returned exit 0, 2560×1920, 82,978 colors, luminance range 227.1, 45.26 seconds (target/outdoor-route26-ochre-current.png). Visible inspection confirms the prior gray shoulder seams are gone. The masses remain too block-like and the strata texture too strong; the canyon is not visually accepted. Another chat's canonical new_bark_3d build is confirmed live as PID 58363 after the latest shader/capture edits. Reuse its completed tool rather than starting a duplicate. The separate indoor shader range [80000,81000) being added by its owner is preserved; park tags remain in their own [40000,60000) and [70000,70020) ranges.

### Same-view GPU motion evidence — 2026-10-11

PID 58363 completed and the canonical native executable rebuilt at 14:25:18, after capture source 14:21:43 and shader source 14:22:23. Explicit image mode with CRYSTAL_RENDER_TEST_SECOND_VIEW=same successfully captured target/outdoor-park-water-temporal-a.png and -b.png, exit 0. Both are actual 3D GPU views of NationalPark (20,24), orbit 1, 2560×1920, with 27,507/27,317 colors and equal luminance range 240.4. Captures completed at 16.93/17.54 seconds in the same process. Both images were visibly inspected; blue cross removal remains confirmed, the new irregular glints render, and source grass/pool layout and camera are stable.

A central-region pixel comparison restricted to blue-green water in both images selects 362,977 pixels. 173,917 change by more than three summed RGB levels; mean summed-channel difference is 4.208. This demonstrates live GPU water shading rather than inferring motion from shader text. No image was modified. Water still lacks convincing depth/reflections and remains below the saved reference, while grass/flowerbed density and canyon shape/material restraint remain unfinished. No goal completion is claimed.

### Park flower planting refinement — 2026-10-11

Replaced the generic route flower model only for native park flower tiles on NationalPark/contest with five offset rosettes per cell, folded broad basal leaves, varied head heights, cupped pink/red five-petal flowers and restrained warm centers. Native lawn cells, bed border ownership, original centerpiece/grass layout and support heights are unchanged. Variation is keyed to absolute source-map coordinates in both production animated-surface and audit paths, preserving planting while scrolling. Park flower ownership is labeled park/flowers.

The actual new geometry module passes its isolated Rust test for cell containment, height limits, unit normals, coordinate translation, neighboring variation and both blossom pigments. Temporary harness files were automatically removed. Added a production dispatch regression for ordinary/contest equality and rejection of native lawn. Integrated canonical test session 58414 (target/outdoor-park-flower-test.log) is confirmed live, waiting on other active Cargo builds. The current native executable predates these edits; visual acceptance requires a freshly built southern flowerbed capture. No finished appearance is claimed from geometry tests.

### Park flower GPU review and restrained canyon pigment — 2026-10-11

The canonical native example rebuilt at 14:41:56 after the initial park flower module and both dispatch paths. Explicit image-mode capture of NationalPark (20,40), orbit 1, returned exit 0 in 10.44 seconds: target/outdoor-park-flowers-current.png, 2560×1920, 34,386 colors, luminance range 252.6. Visible inspection confirms the clusters render on the source flower positions, but the five-plant clusters still read as isolated bouquets and diamond petals appear angular. Trees, grass and paving remain far below the saved concept. No visual acceptance is claimed.

Based on that actual image, the current flower module now uses seven offset plants and eight-segment rounded, gently cupped petals with restrained tonal variation. Its isolated Rust geometry test passes again after this refinement. This newer revision still requires a fresh GPU capture. Integrated test session 58414 remains confirmed live and waiting on other Cargo work; no duplicate job was started.

The actual prior Route26 ochre image was re-inspected: pigment seams are gone, but sediment contrast and cap grain are excessive. The canyon-only shader now blends paint at 0.60 on walls and 0.42 on caps, retaining broad continuous ochre variation. Ordinary gray outdoor stone remains at its previous 0.82 paint strength. The separate indoor branches are preserved. The shader edit postdates the native binary and needs a fresh build/GPU comparison; canyon forms and ledge joins still require further work.

### Park trunk surface lighting and integrated flowers — 2026-10-11

The canonical flower test session 58414 completed with exit 0. The fresh 14:51:13 renderer test executable passes all six park_flower tests, including both new cluster/source dispatch regressions and the existing complete-bed border/ground checks. These cover the seven-plant rounded-petal revision, but its fresh GPU appearance remains pending.

Park trunk normals now follow the actual fluted taper, rooted flare and lean using local parametric derivatives; exact duplicate seam vertices/normals remove the angular wrap discrepancy. Derivatives exclude scene translation to avoid camera-dependent cancellation. Branch roots start lower with staggered heights, exposing more of the spreading fork structure. Source tree plot, floor and cutaway ownership remain intact. The exact current park tree module plus the actual parent variation/branch/crown helpers passed three isolated Rust tests: closed rooted geometry, varied trees with one cutaway owner, and the new flare-normal/seam check. Temporary harness removed automatically. This is geometry evidence, not visual acceptance.

The newest tree source postdates the integrated renderer test executable. A current canonical new_bark_3d build is confirmed live as session 2639, target/outdoor-park-tree-native-build.log, waiting on existing Cargo jobs. It will also include the pending canyon pigment refinement. No duplicate live job was restarted. Next visual reviews must use a binary newer than the current park_tree.rs and voxel_surface.wesl, then inspect the park southern planting and Route26 ledge/canyon forms at 45 degrees.

### Southern Route26 shape review and free-rock lower faces — 2026-10-11

The 14:41 native binary produced an explicit image-mode Route26 (10,54), orbit 1 capture, exit 0 in 55.15 seconds: target/outdoor-route26-south-joints.png, 2560×1920, 118,440 colors, luminance range 238.5. Visibly inspected: native isolated rocks still read as rounded boxes; the view does not prove all canyon stair joins. Current integrated renderer executable at 14:54:14 passes the new park trunk normal/seam test and all six ledge_approach tests, including cap/face tessellation, sharp sawtooth reset, bank floor centers and cliff joins. These are geometry checks, not whole-route visual acceptance.

The isolated rock lower wall retained a global six-unit bank datum despite adjoining level ground. Changed only the source-proven 0x0a rock wall profile (distinct existing 0.95 rim scale) with zero neighboring floor to weather down to its actual support. Existing higher-support constraints, canyon bank datum, end anchors, corner/cap fields and footing remain intact. Added a stronger free-rock regression requiring deformation below six units while exact floor contacts and shared wall samples stay fixed. This new source postdates the current renderer tests and requires fresh integrated checks and visible review; existing contact assertions were not weakened.

Canonical native build session 2639 remains confirmed live, waiting on active Cargo jobs; no restart on timeout. It will include the current flowers, park trunk, canyon pigment and free-rock refinement. Further source-map/angle review remains required, especially the canyon staircase at Route26 (10,28).

### Current integrated stone and park verification — 2026-10-11

The renderer test executable rebuilt at 15:06:44 after outdoor_stone.rs 15:01:33. All 10 outdoor_stone checks pass, including the new free-rock lower-face regression plus unchanged higher-floor support, unequal-floor wall rings, cap/wall closure and cap winding assertions. All 31 park_ checks pass, including rounded flowers, dispatch/source ownership, rooted/flared trunks, both grass types, original floor and water/basin guards. No contact assertions were weakened. This confirms the intended lower deformation without substituting visual acceptance.

Re-inspected the original 2D Route26 canyon capture: projecting rock shoulders and ledge runs are authored layout and must be retained while improving their forms. The canonical native example still predates these source changes. Build session 2639 is confirmed live, waiting on the shared active build queue; a crystal_bevy test compilation is actively consuming CPU. No restart or duplicate owned build was started. Pending actual screenshots are the current NationalPark southern planting, Route26 southern isolated rocks and Route26 (10,28) staircase at 45 degrees. Entire outdoors goal remains incomplete.

### Fresh park and Route26 GPU review — 2026-10-11

Canonical native build session 2639 completed exit 0 in 16m30s; executable rebuilt 15:08:52, after the rounded flower/trunk/free-rock source and canyon pigment edit. Fresh NationalPark (20,40), orbit 1 explicit image capture returned exit 0: target/outdoor-park-planting-reviewed.png, 2560×1920, 34,538 colors, luminance range 252.6, 5.81s, plus the original 2D comparison (16 colors). Both visibly inspected. Rounded flowers and clearer forks render, with native flower/bench/tree positions preserved. Bed remains sparse and flower heads sit low relative to the edging; pavement and grass/tree silhouettes remain too uniform. Raised only flower heads from 0.32 to 0.48 tile, reduced center offset, leaving basal leaves/floors/source locations fixed and keeping the original 0.65-tile envelope. The exact geometry module passes its isolated Rust test after this edit; its new GPU view remains pending.

Wide Route26 (10,28) capture hit the preview’s hard 60-second timeout and exited 1 before writing a frame. After confirmed termination, retried at 960×720 with the same position/orbit. Exit 0: target/outdoor-route26-canyon-reviewed.png, 1920×1440, 55,283 colors, luminance range 228.8, 59.67s, plus the original 2D comparison at 60.01s. Both visibly inspected. The canyon grain/strata is quieter and source projections remain recognizable; visible staircase joins appear closed, but masses still read too rectangular and bank grass detail is thin/regular. This is not full-route seam acceptance.

Fresh Route26 (10,54) orbit 1, 960×720 capture returned exit 0 in 49.09s: target/outdoor-route26-free-rock-reviewed.png, 1920×1440, 91,476 colors, luminance range 238.5. Visibly inspected: softer pigment and lower wall facets render without an obvious floor gap, but isolated rocks still have square toes/corners. Existing generic corner anchoring below the bank datum limits the change. Further shape work should distinguish actual native 0x0a corner phases over level ground from real ledge/higher-floor contacts, retaining the latter.

Only owned current build is session 48099, target/outdoor-flower-height-native-build.log, confirmed live and waiting on other Cargo jobs. It contains the pending raised flower heads and latest shared indoor shader work. No partial render is treated as completion of the outdoors goal.

### Native rock toe refinement and current GPU result — 2026-10-11

Native 0x0a corner phases over level cardinal/diagonal floors now distinguish a free toe from a bank closure. Other source fragments and raised-floor contacts keep their existing conservative datum. Added a full native-phase fixture checking inward toe compression on both walls at three source phases, exact floor vertices, shared wall samples and unchanged corners beside an adjoining six-unit bank. Existing contact assertions remain intact.

Build session 48099 completed exit 0; canonical native example rebuilt 15:30:32. Fresh Route26 (10,54), orbit 1, 960×720 explicit image capture completed exit 0 in 49.50s: target/outdoor-route26-corner-reviewed.png, 1920×1440, 91,563 colors, luminance range 239.5. Visibly inspected: the new lower form renders but isolated rocks still appear too box-like and upper facets remain harsh. The current 15:33:21 renderer test executable passes all 11 outdoor_stone tests, including the new full-phase corner/bank fixture. This does not establish visual acceptance.

After reviewing that image, bounded the new corner toe facet below the bank-height ring, smoothly returning to zero at its endpoints and preserving the established upper shoulder field. Strengthened the native-phase fixture to compare all original upper wall samples against the conservative fragment profile. This newest revision postdates the GPU capture and integrated test executable and still needs fresh checks/rendering. Owned canonical test session 95569 remains live waiting on other Cargo jobs; no duplicate test/build job was restarted.

### Bounded toes verified; raised flowers and warm park limestone — 2026-10-11

Canonical stone test session 95569 completed exit 0. Current renderer test executable rebuilt 15:41:41 and passes all 11 outdoor_stone tests, including the strengthened native toe fixture which compares every original upper wall sample against the unchanged conservative profile, preserves both corner walls across source phases, and retains exact contacts beside a six-unit bank. No old contact assertions were weakened.

Native executable rebuilt 15:39:24. Fresh NationalPark (20,40), orbit 1, 960×720 explicit image capture returned exit 0 in 9.40s: target/outdoor-park-flower-height-reviewed.png, 1920×1440, 29,236 colors, luminance range 252.9. Visibly inspected: raised flower heads read more clearly relative to the edging, but the bed is still sparse and pavement nearly white/uniform. Trees/grass/water still remain below the saved art target.

Added a park-only 0x40 finish option to the existing disjoint limestone paving partition: warmer buff/ivory stone, restrained darker joints, broader per-stone tonal variation, and low-amplitude source-coordinate weathering across each stone. Only source-proven park pavement selects it; Goldenrod’s existing treatment remains unchanged. Geometry, normals, floor datum and indices retain the same partition. Source-coordinate quantization makes the same pigment exact across viewport translations. The actual city_paving module plus its exact parent lattice/noise/bounds helpers passes all three isolated Rust tests, now including the park option in the exhaustive coverage/phase check. Temporary harness was automatically removed. New paving requires integrated/GPU review.

A fresh owned native build is active (target/outdoor-park-paving-native-build.log), following inspection of live Cargo work. It will include current limestone and bounded toes; no existing live native example build was duplicated. Whole outdoors completion remains unproven.

### Warm paving GPU result and clustered bank grass — 2026-10-11

Owned native build session 58319 completed exit 0 in 3m24s; executable rebuilt 15:52:15 after city_paving.rs 15:47:20. Fresh NationalPark (20,40), orbit 1, 960×720 explicit image capture returned exit 0 in 10.41s: target/outdoor-park-warm-paving-reviewed.png, 1920×1440, 29,537 colors, luminance range 253.3. Visibly inspected: warmer buff limestone and restrained tonal variation render without changing layout/floor; the park still lacks the saved reference’s lush planting, canopy structure and blending. No final acceptance is claimed.

Replaced the shallow bank’s fixed every-third strand pattern and identical three-blade fans with source-coordinate clustered tuft selection, two to four blades, wider bases, varied lean/heights, darker basal colors and sparse hanging grass/exposed roots. Each fan is contained between its existing bank ends; height is limited to 0.32 of face height and native cap/floor/perimeter geometry is unchanged. Final ledge_finish.rs revision at 15:53:07 passes both direct Rust perimeter/scrolling and four-cardinal winding/support tests using the exact parent noise helpers. Temporary harness removed automatically. The current scene warp still requires actual edge/terminal review; isolated decorative geometry is not evidence for every bank-to-floor join.

Another canonical native example completed 15:54:19 during this work, but its source snapshot for the final height reduction is not established. Do not claim a fresh GPU review of the final bank cluster revision from that timestamp alone. A focused owned integrated ledge_finish test build is now active (target/outdoor-bank-cluster-tests.log). No live job was duplicated or restarted. Next review should confirm the current bank clusters against the actual Route26 staircase and terminal floors at 45 degrees.

## Bank grass source ownership — 2026-10-11

An isolated Rust reproduction using the current `ledge_finish` geometry and actual `shape_surface`/normal refresh code found 467 of 677 decorative vertices failing the expected four-row bank rise. Outward blade triangles were resolved through neighboring floor cells or the tall-wall fallback, retaining their old Y. The generated bank surface now carries its known source-cell owner through a temporary UV marker consumed and cleared by the slope pass. Genuine adjacent cliff walls retain their existing inference and height guards. Source floor positions, X/Z coordinates, and gameplay footing remain unchanged.

After the change, an expanded isolated Rust check verified 14,490 earth and decorative vertices on east/west sides, all four source rows, and two viewport translations against the analytic rise; all temporary markers were cleared. The integrated regression is `relieved_bank_grass_follows_its_source_even_outside_the_grid`; the first integrated execution failed a fixture assumption that every face has upright tufts. Deterministic tuft selection intentionally leaves some faces bare. The test now requires elevated blades across the complete fixture while retaining every vertex-height and X/Z assertion; a fresh integrated run is pending. Actual GPU review remains required; the isolated geometry result is not visual acceptance.

### Source-owned bank GPU review and rim follow-up

Fresh native build completed successfully at 16:10:53. The actual 45-degree Route26 capture at (10,28), `target/outdoor-route26-bank-ownership-reviewed.png`, completed in 49.92 seconds, 1800×1350, 53,144 colors, luminance range 226.0. Reviewed the real image: the bank grass follows the visible sloping approaches without obvious disconnected old-height strips. This view does not establish every Route26 terminal contact. Broad canyon shoulders and ledge faces remain too smooth and regular for the art target.

The south bank lip recession now varies more visibly at shared half-cell knots through a broad source-coordinate wave and smaller continuous noise, bounded below one tenth of a source tile. It retains anchored run ends, floor vertices, cap centers, source Y heights, and the shared cap/face warp. This rim change postdates the capture; integrated closure/normal checks and a fresh GPU comparison are required before accepting it.

An isolated Rust execution of the actual updated `soften_lips` function passed coarse-versus-detailed cap/wall sample and normal matching, fixed floor contacts and cap centers, unchanged Y/X, sub-0.8-unit recession, and unit normals at two viewport translations. This is geometry evidence; the updated rim still needs GPU review.

The 16:14:38 integrated renderer test executable passed all seven `ledge_approach` tests, including the corrected source-owned grass regression and the six existing sawtooth/drop/cliff/floor/normal checks. This executable was compiled before the newer rim-recession field; it validates the ownership fix, not the latest rim variation. The current rim tests and fresh native build remain live.

## National Park ground blending follow-up — 2026-10-11

The reviewed warm-paving park capture still has very uniform lawn beneath the outer trees. NationalPark and NationalParkBugContest now opt into the existing source-anchored sunlit-grove floor treatment: restrained moss/dry-soil variation and filtered canopy contact light. This affects ground colors; source geometry, encounter grass ownership/types, centerpiece outline, paving, and water stay on their existing paths. The existing scene build queued for rim review will include this change only if its renderer compilation starts after the edit. A fresh park capture is required for visual acceptance.

Park canopy colors now temper the yellow-green highlights and enrich the blue-green shade body through a park-only linear palette adjustment. Geometry, branches, source placement, cutaway ownership, and route groves are unchanged. The change requires a fresh park GPU comparison against the saved concept; no visual acceptance is claimed from the palette edit alone.

The integrated renderer executable built at 16:21:20 passed all seven ledge-approach regressions with the updated recession field. The 16:18:50 native renderer also produced `target/outdoor-route26-rim-variation-reviewed.png` at the same Route26 (10,28), 45-degree view: 1800×1350, 53,222 colors, luminance range 227.1, 58.16 seconds, successful capture. Reviewed the actual image: subtle shallow edge variation and no obvious cap/face opening in this view, but the broad canyon mass silhouette is still too regular. The native build predates the park-ground and canopy-color changes; it is not park visual evidence.

## Park ground/foliage GPU comparison and oval-leaf follow-up

After temporary shared-checkout Dragon’s Den module/import build errors were resolved, the native renderer built at 16:31:52 produced the park promenade at (20,40), 45 degrees, and its original 2D counterpart. `target/outdoor-park-ground-foliage-reviewed.png`: 1920×1440, 53,267 colors, luminance range 253.3, 12.39 seconds. The classic frame completed at 12.63 seconds with 16 colors. Reviewed both real images: warmer varied lawn and cooler park foliage are visible, and the southern trees, benches, fence and flowerbed retain source placement. Broad empty lawns, sparse flowers and paper-chip leaf shapes remain below the saved concept. The 16:26:27 renderer test executable passed all 31 park regressions (compiled after the ground/palette edits).

Park crowns now select a dedicated cupped oval leaf rather than the route’s long pointed blade. Source tree roots, trunks, crown skeleton, map variation and cutaway ownership remain unchanged. Town and woodland crowns select their original paths. The revised park leaf has an actual folded geometric normal field and a modestly wider area for layered coverage. This leaf change postdates the GPU comparison and requires shape/winding checks plus a fresh park render before acceptance.

An isolated Rust run of the actual park-tree module and its actual parent crown/branch/variation helpers passed all four current module tests, plus a complete leaf-canopy face check across three map phases: 15,360 nondegenerate outward leaf triangles per tree, finite unit normals, bounded colors, rooted trunks, sealed buttress faces and one cutaway owner. The added `park_oval_leaves_keep_outward_winding_through_curvature` regression checks the cupped oval at different surface orientations and sizes. These are geometry checks; fresh native visual review is still pending.

## Outdoor mineral-course geometry follow-up — 2026-10-11

The reviewed Route26 walls remain too flat beneath detailed painted strata. Their bounded inward weathering now follows shallow piecewise mineral courses, adapted from the cave rock’s shoulder/slab pattern and sampled continuously in source X/Z/Y. The original maximum coefficient (0.32 source-tile width), end anchors, cap/floor datums, adjoining support guard, and join field remain intact; the raised beds use a smaller recess rather than expanding the wall. This changes actual wall relief, not source map geometry or gameplay footing. Existing stone contact/cap/winding tests and a new Route26 capture remain required; no acceptance is inferred from the formula.

## Oval-leaf GPU review and low root planting

The 16:45:54 native renderer produced `target/outdoor-park-oval-leaves-reviewed.png` at the same promenade camera: 1920×1440, 52,884 colors, luminance range 253.3, 9.38 seconds. Reviewed the actual image: rounded leaf shoulders are visibly softer and more coherent than the old pointed chips, but the canopy masses and surrounding ground remain less nuanced than the saved concept. The 16:48:10 integrated renderer test executable passed all 32 park tests, including the new oval winding regression, and all 11 outdoor-stone tests. Its renderer compilation started after the mineral-course edit at 16:43:32; this is geometry regression evidence for the new course field, not GPU acceptance.

The park-tree generator now adds six low basal rosettes beside the buttress roots, entirely within the authored tree plot. Leaves have real folded triangles and follow the source-stable tree phase. Their maximum height is below one quarter of a source tile, preserving readable walking space; gameplay collision/encounter state is untouched. They share the tree’s existing cutaway owner. An isolated Rust run of the actual current park-tree module and its parent helpers passed all five module regressions, including new planting bounds, triangle winding and unit normals. This root planting postdates the screenshot and needs a fresh native image. The owned native build and stone Cargo regression jobs subsequently completed successfully (both exit 0). The resulting native executable at 16:50:12 includes the mineral-course work and predates the root planting at 16:52:22.

### Mineral-course capture performance follow-up

The native Route26 mineral-course captures at 900×675 and then 720×540 both hit the fixed 60-second capture timeout (exit 1) without a validated image. Neither attempt is visual acceptance. Inspection found that the wall field evaluated mineral slabs for remote edges whose influence was exactly zero. The pass now skips zero edge/end weights and lazily shares its source-coordinate mineral fields between participating edges at one vertex. This preserves the intended coefficient/shape while avoiding unnecessary field evaluations. The optimization postdates the successful 11-test run; fresh regressions and an actual Route26 capture are required. The root-planting native build subsequently completed successfully (exit 0, 5m26s; executable at 16:59:47). It predates compilation of the newer wall-cache optimization, which still requires a fresh native build for Route26.

The root-planting retry saved `target/outdoor-park-root-planting-reviewed.png`
before termination (exit 143). The saved PNG was fully decoded and visually
reviewed: 1800×1350, 51,023 colors, luminance range 253.3. Low rosettes are visible
around the trunks, but the planting remains subtle and the broad lawn and crown
masses still fall below the concept. The saved frame supports this visual
assessment; the capture process did not complete normally.

## Muted mineral-course review and planar mass follow-up

The fresh integrated run passed all 11 outdoor-stone regressions after both
wall and crest field caching; the same renderer executable passed all 33 park
checks, including low root planting. A muted native Route26 capture at (10,28),
45 degrees completed normally: `target/outdoor-route26-crest-cache-muted-reviewed.png`,
1800×1350, 53,259 colors, luminance range 226.4, 54.57 seconds. The preceding
muted attempt still timed out while building terrain. The successful frame was
visually reviewed: course relief is subtle and broad cliff tops still read as
smooth blocks, so this is evidence of remaining shape work rather than visual
acceptance.

The top-mass field now retains continuous linear mineral planes at two scales
instead of saturating them through `tanh`. Deeper unequal beds should articulate
the plateau without raising source caps. The existing shared wall/cap warp,
support-relative erosion bound, floor datums and planted-tree seats remain in
place. This edit postdates the successful capture; fresh geometry regressions
and a muted native image are required before judging its result.

Fresh renderer tests compiled after the first planar-field edit passed ten
stone regressions but failed the cropped-interior variation requirement. That
failure is retained as evidence that the initial field was too weak. The
planar field now uses stronger unequal beds with positive bounded depth, so
source caps are never raised. The original variation
assertion remains unchanged. This revised field is awaiting fresh regressions
and a muted GPU comparison.

Cap restoration now short-circuits non-stone triangles and triangles outside
the cap's height plane before loading all points and calculating centroids.
The native cap selection predicate and resulting indices are unchanged; this
avoids repeatedly doing geometric work on foliage/ground for each rock corner.
It is included in the pending strengthened-field verification.

A subsequent renderer test snapshot still failed cropped-interior variation:
its shallow minimum-depth clamp had flattened the fixture's negative mineral
patch. The current field removes that minimum clamp and uses a positive bias
larger than the combined bounded noise amplitudes. Its depth stays positive
while retaining linear planes throughout each mineral bed. The original
support-relative safety bound remains. This latest field postdates the failed
snapshot and still needs integrated tests and muted GPU review.

The current field and cap-filter optimization passed the fresh integrated
outdoor-stone run: all 11 regressions, including the original cropped-interior
variation assertion (Cargo exit 0, renderer test executable at 19:48:56).
A temporary headless probe called the public production authored terrain
builder on a cropped Route26 source fixture: 1,296 stone vertices, heights
12.681368 through 14.308224. This is full terrain-builder geometry evidence,
not a GPU image or complete Route26 acceptance. The first probe used a default
map texture handle and correctly failed frame validation; it was corrected to
use nonempty fixture handles. Both probe scratch directories were removed.
The native build and muted visual comparison remain pending.

The native build completed successfully (exit 0, executable at 19:51:28).
Muted Route26 (10,28), 45-degree GPU captures completed normally:
`target/outdoor-route26-planar-mass-muted-reviewed.png`, 1800×1350, 53,529
colors, luminance range 226.3, 40.78 seconds; its original 2D companion completed
at 41.11 seconds with 13 colors. The real 3D frame was reviewed against the 2D
layout: broader unequal top cuts are visible, but shoulders still look softly
rolled and the stone remains below the target. The same fresh renderer test
executable also passed all seven ledge-approach/contact regressions.

The stone-rim projection now uses convex octagonal bevel planes instead of a
circular fillet. It retains one shared radial deformation for cap and wall,
the support/end guards and the original bounded footprint. This changes
shoulder geometry and postdates the reviewed image. Cap-area, seam, floor and
normal regressions plus a fresh muted image remain required before acceptance.

An isolated Rust execution of the actual current outdoor-stone module passed
all 11 existing regressions with octagonal bevels. It used the actual corner,
quad, lattice and slab helpers, with fixture shapes forwarded from the
production source-profile dispatcher. This proves the tested geometry
invariants without claiming a complete native-client or visual result. Its
temporary source/executable directory was removed; the integrated run remains
live.

## Deeper park crowns — 2026-10-11

Comparison of the reviewed park frame with the saved concept showed compact
high crowns over long bare stems. Park crowns now sit lower around the branch
forks and have a deeper vertical spread. Roots, source tree plots, branches,
palette, native grass types and cutaway ownership remain on their existing
paths. An isolated execution of the actual park-tree module and its actual
parent geometry helpers passed all five module regressions. Its scratch
directory was removed. This crown change still requires integrated checks and
a muted park GPU comparison before visual acceptance.

The integrated bevel run completed successfully: all 11 stone regressions,
Cargo exit 0. The fresh renderer executable at 20:07:56 also passed all 33 park
checks. Native build completed successfully (executable at 20:05:33). Muted
Route26 bevel captures were reviewed at 45 and 135 degrees:
`target/outdoor-route26-mineral-bevel-muted-reviewed.png` (1800×1350, 53,461
colors, luminance range 226.4, 24.15 seconds) and
`target/outdoor-route26-mineral-bevel-135-muted-reviewed.png` (1800×1350,
54,865 colors, luminance range 227.7, 36.38 seconds). Both processes exited 0.
No obvious bank/cliff opening is visible in these views, but the broad stone
masses and soft-looking shoulders remain below the reference standard; this
does not establish every Route26 floor terminal.

The muted deeper-crown park capture completed successfully:
`target/outdoor-park-deeper-crowns-muted-reviewed.png`, 1800×1350, 49,126
colors, luminance range 252.5, 7.37 seconds. Reviewed the real image against the
saved concept: lower crowns connect better with the stems, but enlarged leaves
still form bulky, uniform shells. Park leaf dimensions now follow horizontal
canopy span rather than vertical depth, and individual leaves have modest,
source-stable angle variation. Other crown styles retain their existing paths.
An isolated execution of the actual current park module and parent helpers
passed all five module regressions plus full fan winding/unit-normal checks on
15,360 leaf triangles per canopy at three phases. Scratch output was removed.
This leaf change postdates the image and requires a fresh muted GPU review.


## Park bough silhouettes — 2026-10-11

The leaf-angle native rebuild completed successfully (20:29:38 executable).
Reviewed the fresh muted GPU frame
`target/outdoor-park-leaf-angles-muted-reviewed.png`: 1800×1350, 49,756
colors, luminance range 252.5, capture exit 0 in 7.79 seconds. Smaller leaves
help the surface scale, but the crowns still read as rounded solid shells.
The focused integrated leaf run passed all five park-tree module tests; it
predates the next bough-shape edit.

Park crowns now use a separate seven-bough radial field with a smaller inner
body, unequal side and upper masses, and deeper saddles between branches.
The field supplies its analytic tangent gradient to both crown and leaf
normals. Town and woodland crown fields keep their existing paths. Source
plots, trunks, roots, native grass types and cutaway ownership are unchanged.
An isolated execution of the actual park-tree module and its parent geometry
helpers passed all six tests, including a new finite-difference normal and
closed-seam check over three phases. Temporary harness output was removed.
A muted native rebuild is queued in
`target/outdoor-park-cluster-muted-build.log`; a fresh GPU comparison and
integrated checks of this seven-bough change are still required. This is not
visual acceptance of the park or the full outdoors goal.

The complete new crown was additionally checked at three phases: all core
faces and all 15,360 leaf triangles per canopy remain nondegenerate and
outward relative to their surface normals (seven isolated checks passed).
The full crown face check is now retained in the park-tree module.


## Canyon cap pigment — 2026-10-11

Reviewed the previous real Route26 135° bevel frame against the original
2D drawing. The cap pigment is too close to the sediment faces, making
shoulders read as a single muddy mass. Canyon caps now use a lighter warm
weathered bed color, blended into the existing face pigment by the actual
surface normal; cap texture strength falls from 0.42 to 0.28. Wall texture
strength remains 0.60 and non-canyon stone retains its existing pigment.
This shader change still needs a fresh muted GPU comparison for both color
and pipeline validation.

The first bough native rebuild failed on a concurrent gym importer schema
change (missing `gym_stone_pixels`). Other work supplied its metadata remap;
a duplicate initializer was removed without changing that remap. The retry
in `target/outdoor-park-cluster-muted-rebuild.log` is confirmed live. The old
failed build is not evidence for either the new crown or cap pigment.


## Park root planting blend — 2026-10-11

The latest leaf-angle scene has nearly bare lawn below the tree canopy.
Expanded the existing source-owned root planting from six to twelve low
rosettes, staggered over three radii. Leaves are broader and longer, with a
higher folded middle, to create a visible transition from buttress roots to
lawn. All planting remains within the original 4×4 source tree plot; no
promenade, encounter grass or placement data changed. Updated the deliberate
planting height bound to below 0.40 tile (previously 0.25) while retaining the
floor, plot, outward winding and unit-normal assertions.

An isolated execution of the actual current park-tree module and actual
parent functions passed all seven retained tests. Scratch output was removed.
The muted native retry has progressed from its confirmed queue into compiling
crystal-bevy after the foliage edits. The GPU result remains pending.


The native retry completed successfully, executable 20:53:27, Cargo exit 0
in 7m09s. Reviewed two fresh muted GPU captures, both process exit 0:
`target/outdoor-park-bough-root-blend-muted-reviewed.png` (1800×1350,
50,891 colors, luminance range 252.5, 8.04 seconds) and
`target/outdoor-route26-weathered-cap-muted-reviewed.png` (1800×1350,
57,736 colors, luminance range 225.4, 54.84 seconds).

The park planting is now visible around roots. Seven boughs break the previous
rounded shell, but upper lobes are too pointed in the real image. Broadened
the lobes (field exponent 13 to 10), reduced their amplitude, and filled the
inner body slightly. Park core sampling is now 18×36 instead of 12×24;
town and woodland retain 12×24. All seven actual-module isolated tests pass
for this softened revision, including full shell/leaf winding and numerical
normal checks. Scratch output was removed. A new muted native build is live
in `target/outdoor-park-soft-bough-muted-build.log`; the image above predates
this softening. The integrated park test run in
`target/outdoor-park-bough-root-muted-tests.log` is also confirmed live.

The Route26 render validates the revised shader pipeline and visibly lighter
cap beds. Broad cliff masses remain too uniform and the ledge silhouette does
not yet prove the requested saw-blade profile or every floor terminal. Neither
scene meets overall visual acceptance.


## Fresh softened crowns and actual Route26 banks — 2026-10-11

Softened-crown native build completed successfully, Cargo exit 0 in 8m49s,
executable 21:05:04. Reviewed the actual muted GPU frame
`target/outdoor-park-soft-bough-muted-reviewed.png`: 1800×1350, 48,903
colors, luminance range 252.5, capture exit 0 in 9.31 seconds. The pointed
upper lobes are reduced and root planting remains visible, but canopy bodies,
bare lawn, pale paving and water still fall below the saved art reference.

The integrated park run failed on an unrelated test initializer using an
unimported `default()` in modeled_dungeons. Replaced it with
`Default::default()` and started the focused retry in
`target/outdoor-park-soft-bough-muted-tests.log`; its handle is confirmed live.
No integrated pass is claimed for this revision yet.

Executed a temporary Rust headless probe using the verified local pack,
actual in-memory tileset pixels and the public production terrain builder for
Route26 source columns 4..36 and rows 32..80. All 112 bank cells in this crop
were central native $57 drawings. Their four source-row footing heights are
exactly 0.75, 2.25, 3.75 and 5.25 world pixels, maximum expected-height error 0.
This proves the crop's footing buffer follows the rise, not its visible cap
triangle continuity or all terminal joins. A triangle-level surface check is
still needed to distinguish cap defects from visual readability.

Early probe compilations encountered concurrently replaced Cargo libraries;
no geometry ran in those attempts. The successful probe used a temporary
snapshot of the current production rlib for a consistent link. All temporary
source, executable and snapshot files were removed automatically. No pack
content was written out or added to source control.


## Actual bank cap and floor-contact geometry — 2026-10-11

The same real Route26 pack crop (source columns 4..36, rows 32..80) passed
1,008 triangle ray samples across 112 central native bank cells: zero missing
or incorrectly elevated caps, maximum expected-height error 0.0000009536743
world pixels. Samples cover X fractions 0.15, 0.50, 0.85 and Z fractions
0.10, 0.35, 0.50 in each source cell. The recessed lip region is deliberately
outside this linear-rise assertion.

A second production-builder check passed 252 rays through the south drop
faces at X fractions 0.15, 0.50, 0.85 and heights 0.01, 3.0, 5.9 world pixels.
Zero holes were found, including the near-floor samples. This covers the
sampled central runs, not all terminal shapes or every route location.
Temporary snapshot/executable directories were removed; the ignored probe
source is retained only until the final related checks finish.

Reviewed `target/outdoor-route26-bank-side-muted-reviewed.png`: CLI orbit 2,
actual 45°, zoom 1, 1800×1350, 55,128 colors, luminance range 229.3,
34.87 seconds, muted capture exit 0. Cliffs still read as overly uniform
masses. A true side capture uses CLI orbit 4 (90°); that job is live.


## Stone shoulder normal weighting — 2026-10-11

Reviewed fresh muted captures at the corrected angles:
`target/outdoor-route26-bank-90-muted-reviewed.png` (CLI orbit 4, actual 90°,
zoom 1, 1800×1350, 54,040 colors, luminance range 230.2, 33.84 seconds,
process exit 0) and `target/outdoor-park-actual45-muted-reviewed.png`
(CLI orbit 2, actual 45°, zoom 0, 1800×1350, 47,686 colors, luminance range
240.6, 9.58 seconds, process exit 0). Both scenes remain below the reference;
the actual 90° rock view exposes particularly harsh shoulder lighting.

The production builder's real Route26 crop contains 107,190 nondegenerate
stone triangles. On 159 triangles, the geometric face normal points opposite
the average of its assigned vertex normals (dot < -0.01). This is evidence of
bad face/lighting agreement; it is not by itself proof of inverted geometry.
A temporary angle-weighted normal candidate on that same completed mesh
reduced this count to 82. The 1,008 cap-ray and 252 drop-ray bank checks still
passed. Temporary executables and rlib snapshots were removed.

Changed actual outdoor stone normal smoothing from face-area weights to
corner-angle weights. Shared position ownership and geometry remain intact;
large cap triangles no longer dominate finely sampled shoulder normals solely
through their area. A new retained regression varies face size and fan split
while requiring the same dihedral normal at duplicated shared corners. An
isolated execution of the actual smoothing function and this actual test
passed. The native build in `target/outdoor-stone-angle-normal-muted-build.log`
and integrated stone run in `target/outdoor-stone-angle-normal-muted-tests.log`
are live; a fresh GPU comparison and the remaining bad-face investigation are
still required. The 82-face count describes the temporary candidate, not yet
a rebuilt production result or visual acceptance.

The focused integrated park retry subsequently completed successfully, Cargo
exit 0. Its park-tree module checks passed; no full-gameplay or full outdoors
visual acceptance is inferred from this result.


## Residual cap folding investigation — 2026-10-11

A freshly linked production-builder probe now observes the angle-weighted
normal implementation: 82 of 107,190 nondegenerate stone faces have negative
agreement with their average assigned normal. Reapplying the angle-weighted
candidate gives the same count. This confirms the production geometry result,
without yet claiming a new GPU result.

Projection into the original source-coordinate UV plane identified 11
final downward-facing triangles among 54,764 strong original cap-projection
samples. Inspected examples have the native cap-grid UV determinant near
-1.78 and upward average normals, with horizontal warped vertices overtaking
one another. Thus part of the remaining defect concerns actual cap geometry,
not only normal averaging. The 1,008 bank-cap rays and 252 drop-face rays
still pass for the central Route26 crop.

A temporary local rewind candidate made this worse: 269 shared points were
adjusted and 19 projected cap triangles remained folded; bad normal agreement
increased to 90 faces. Rejected it and removed it from the probe. It was never
added to production source.

Reduced the production secondary crest recession from 0.48 to 0.28 tile,
after the existing bevel and convex-corner compression. Broad vertical erosion,
mineral fields, support bounds and source footprints keep their existing paths.
The intent is to prevent this additional inset from overtaking compressed cap
samples. This requires the actual pack probe and fresh muted GPU review before
claiming the fold issue is fixed. Both owned native/test jobs are confirmed
live and still queued; the source edit precedes their compilation. The current
probe source remains ignored for this pending verification only.


The shared build cache was held by another live test for over twenty minutes.
Created an ignored APFS clone at `target-outdoor-verify` for independent
outdoor iteration. Confirmed and stopped only our queued build/test PIDs
36515/36630 (both terminal exit 130); other work was not interrupted.
The isolated native build completed successfully in 2m59s.

Its real-pack probe disproved the shallower-crest hypothesis: 83 bad
face/normal agreements and 12 downward cap-projection triangles, compared
with 82 and 11 before. Central bank cap/drop rays still passed. Reverted the
0.28 recession to 0.48. Added a temporary, opt-in
`CRYSTAL_OUTDOOR_GEOMETRY_TRACE` for the specific affected source vertex;
it is silent unless explicitly enabled. The trace rebuild is live in
`target/outdoor-stone-warp-trace-muted-build.log`. This diagnostic must be
removed after isolating the warp defect; no successful cap-fold fix is claimed.

## Finite-edge bevel correction — 2026-10-11

The opt-in geometry trace identified a genuine cap fold at Route26 source
samples (190.66667,32,293.33334), (190.66667,32,294.6667), and
(192,32,294.6667). Their bevel shifts were respectively 1.24948, zero, and
2.01781 pixels. The third sample overtook its unchanged neighbor. Membership
used distance to a finite edge segment, whereas deformation used distance to
the extended wall plane; this introduced an abrupt deformation beyond an edge
endpoint. `mineral_bevel` now uses finite-edge distance for the inward radial
coordinate at clamped endpoints. Interior wall-plane projection is retained.
The temporary stderr geometry trace has been removed.

The regression `finite_rim_endpoint_preserves_route26_cap_triangle_winding`
uses those actual source samples and the production noise/bevel functions.
Isolated execution passes with the correction and fails with the old endpoint
calculation, reproducing the traced vertex inversion. This is local evidence,
not proof that every cap or ledge is aligned. The preceding integrated run
passed all 12 earlier stone tests. The updated integrated run initially failed
to compile because newly edited dungeon guardian dispatch referred to an
unimported `Model`; those two references were qualified with their existing
`crate::dungeon_models::Model` type. The updated 13-test run is live in
`target/outdoor-stone-finite-endpoint-muted-retry.log`. A fresh actual-pack
mesh probe and muted GPU frames remain required before visual acceptance.

The rebuilt production library's actual-pack probe completed successfully:
107,190 nondegenerate stone triangles, 54,764 strong original cap-projection
samples, and two final downward-facing cap triangles (previously 11). Faces
opposite their averaged assigned normals fell from 82 to 64. The 112 central
bank cells still pass all 1,008 cap rays (maximum height error 9.536743e-7)
and all 252 drop-face rays, including floor-level samples. These samples
cover source columns 4..36 and rows 32..80, not the entire route or all
terminals. Two remaining folds lie at source cap corners near (192,288)
and (64,416); further corner work is required. Evidence:
`target/outdoor-stone-finite-endpoint-pack-probe.log`. The build retry also
needed an explicit existing `[usize;2]` UV closure argument type in the
guardian dispatch; no dungeon behavior was changed.

The updated integrated run completed with exit 0: all 13
`mesh::outdoor_stone` tests pass, including the actual Route26 finite-endpoint
regression. Log: `target/outdoor-stone-finite-endpoint-muted-checks.log`.
This validates the checked geometry invariants; it does not close the two
remaining actual-pack cap folds or establish the requested visual quality.
The muted native build remains live in
`target/outdoor-stone-finite-endpoint-muted-build.log`.

The muted native build completed exit 0 in 5m55s. Reviewed the fresh actual
45-degree GPU frame `target/outdoor-route26-finite-endpoint-45-muted-reviewed.png`:
1800x1350, 55,132 colors, luma range 229.5, capture exit 0 in 39.11s.
The endpoint correction reduces lower foreground rim pinching, but overall
cliff masses remain too broad, uniformly ochre, and slab-like. Bare lawns
and sparse grass continue to miss the painterly forest reference. This frame
is not visually accepted as the target. An actual 90-degree muted capture
is live for the side/corner comparison.

Reviewed the matching actual 90-degree muted GPU frame
`target/outdoor-route26-finite-endpoint-90-muted-reviewed.png`: 1800x1350,
54,012 colors, luma range 235.0, capture exit 0 in 38.40s. Side view still
shows dark protruding slivers at the lower foreground cap edge and a sharp
inner shoulder pinch near the trainer. Those visible defects remain open,
as do the two cap folds measured above and the overly uniform cliff masses.
Both owned capture processes have exited normally. Removed the temporary
probe runner; the ignored probe source remains for the unfinished corner
investigation. No generated content-pack assets or build output were staged.

## Reentrant shoulder contribution — 2026-10-11

A temporary opt-in three-vertex trace showed that the remaining corner at
source (192,288) folds during the first rim stage. Wall inset, corner shoulder
cut, and crest inset are all zero at these samples. The two perpendicular
bevels each contribute the same full offset: 1.3745648, 1.3331273, and
2.189051 pixels at the three adjacent source vertices. Their combined
diagonal gradient inverts the cap.

`combined_rim_shift` now gives the overlapping contributions the strongest
face's radial length, retaining direction and approaching the single-face
field continuously. Single-face offsets remain the same. The new actual
corner regression passes with this calculation and fails with independent
full offsets; an additional assertion checks continuity as the second face
vanishes. Temporary trace code was removed. Integrated 14-test validation and
a fresh muted native build are live in
`target/outdoor-stone-combined-rim-muted-tests.log` and
`target/outdoor-stone-combined-rim-muted-build.log`. The actual-pack probe and
GPU review remain pending; no visual acceptance is claimed.

The fresh production-library actual-pack probe now finds zero inverted cap
triangles among 54,764 strong original cap-projection samples in the same
Route26 crop. All 1,008 bank-height rays and 252 drop-face rays still pass;
maximum cap-ray height error is 9.536743e-7. Added an explicit zero-cap-fold
assertion to the ignored probe and reran successfully (exit 0). Log:
`target/outdoor-stone-combined-rim-pack-probe.log`. The normal comparison
increases from 64 to 68 opposing wall faces among 107,190 nondegenerate
stone triangles, so wall slivers/shading still need investigation. This
proves removal of the detected cap inversions in this crop, not every
terrain defect or achievement of the art target. Integrated tests and
native capture review remain pending.

Integrated validation completed exit 0: all 14 stone tests pass, including
finite-endpoint and perpendicular-bevel winding regressions. Log:
`target/outdoor-stone-combined-rim-muted-tests.log`. The native preview
rebuild remains active; it recompiled shared renderer edits made while the
test build was running, so the older executable is not used for acceptance.

The fresh native rebuild completed exit 0 in 8m32s. Reviewed both muted
GPU frames: actual 90 degrees
`target/outdoor-route26-combined-rim-90-muted-reviewed.png` (1800x1350,
54,025 colors, luma range 231.7, 36.29s) and actual 45 degrees
`target/outdoor-route26-combined-rim-45-muted-reviewed.png` (1800x1350,
55,089 colors, luma range 228.3, 35.86s). Both capture processes exited
normally. The detected cap folds are removed in the measured crop, but
dark protruding wall slivers remain at the lower foreground corner in the
90-degree frame. The large ochre slab masses and bare lawns remain far
from the reference. Neither frame establishes visual acceptance.
Removed the temporary corner-probe runner; no opt-in trace code remains
in outdoor_stone.rs. The ignored actual-pack probe source remains useful
for the unfinished wall investigation; external content packs stay ignored.

## Wall height join — 2026-10-11

The detailed actual-pack probe records all 68 wall faces opposed to their
averaged assigned normals. An opt-in trace at source (224,544) found that
heights 12..16 receive an outward bevel from a shorter 16-unit rock; at
height 16 the radial components are 2.4058654 each. Height 17.333334
receives zero such bevel because it exceeds the shorter cap's height. The
taller wall's corner compression then resumes at that sample, producing
a large inward/outward kink. Support is zero throughout this particular
vertical trace, ruling out a floor-support switch there. Logs:
`target/outdoor-stone-wall-detail-pack-probe.log` and
`target/outdoor-stone-wall-stage-pack-probe.log`.

The mineral bevel now continues the shorter cap's deformation above its
height, with a smooth fade to zero over one bevel radius. This preserves
the lower-cap value and removes the hard cap-height cutoff at a shared
taller wall. The focused actual-function regression passes in isolation
and checks continuity at the old cutoff and termination of the influence.
The temporary opt-in wall trace was removed. The updated integrated run
is live in `target/outdoor-stone-height-join-muted-tests.log`; actual-pack
checks and GPU review are still required. No visual acceptance is claimed.

The height-join candidate's fresh actual-pack probe passes cap winding
(zero folds among 54,764 samples), all 1,008 bank rays, and all 252
drop-face rays. However, wall faces opposed to averaged normals increase
from 68 to 81. This candidate alone is not accepted as the wall fix.
The trace also showed the taller corner cut disappearing whenever the
shorter bevel moves a shared original corner sample slightly outside
its rounded corner planes.

A follow-up now determines the corner's wall ownership from its original
position, including the bounded native wall-relief skin, and clamps the
rounded projection at the corner planes. This avoids dropping the full
corner cut as the neighboring lower bevel moves the point across a plane.
The updated integrated run is queued behind the still-live height-join
run in `target/outdoor-stone-corner-ownership-muted-tests.log`. Actual-pack
comparison and muted GPU review remain required; this is an unaccepted
candidate, not a visual-quality claim.

The height-only integrated run completed exit 0 with all 15 stone tests
passing. The following original-corner-ownership candidate's fresh actual
production-library probe also passes zero cap inversions, 1,008 cap rays
and 252 drop-face rays. Opposing averaged-normal wall faces decrease from
81 to 75, but remain above the earlier combined-rim version's 68. This
metric does not prove the visible slivers are fixed; both new changes are
candidates pending GPU comparison. Probe log:
`target/outdoor-stone-corner-ownership-pack-probe.log`. The updated integrated
run remains live in `target/outdoor-stone-corner-ownership-muted-tests.log`;
a muted native rebuild is queued behind it in
`target/outdoor-stone-height-corner-muted-build.log`. No temporary trace
code remains. Removed the temporary wall-probe runner after the completed
probe; the ignored probe source remains for ongoing terrain investigation.

## Height/corner GPU comparison — 2026-10-11

The original-corner-ownership integrated run completed exit 0 with all
15 stone tests passing. The fresh muted native build completed exit 0
in 5m21s. Reviewed actual 90 degrees
`target/outdoor-route26-height-corner-90-muted-reviewed.png` (1800x1350,
53,995 colors, luma range 241.4, capture exit 0 in 36.41s) and actual
45 degrees `target/outdoor-route26-height-corner-45-muted-reviewed.png`
(1800x1350, 55,068 colors, luma range 231.7, exit 0 in 33.44s).
Compared with the prior combined-rim side frame, the kink beside the
trainer is smoother and the two large foreground spikes are substantially
reduced. A smaller triangular wall sliver remains near the lower foreground
corner. Retain the new transitions as a visible improvement, not complete
wall repair. The 75 opposing averaged-normal wall faces remain a diagnostic
concern despite zero measured cap inversions and passing bank/drop rays.
Both owned captures exited normally; no preview was left running.

The current park frame was also re-reviewed: crowns still read as solid
lettuce-like green bodies around long stems, and broad bare lawns/pale
paving lack the reference's layered foliage and richer planted ground.
These broader outdoor art requirements remain unfinished, as do cliff
mass variation/color and the remaining wall sliver. No goal completion
or overall visual acceptance is claimed.

## Layered park canopy candidate — 2026-10-11

Re-reviewed the saved park concept alongside the actual 45-degree park
frame. The actual trees retain a solid lettuce-like crown around long
stems; the concept/reference has overlapping branches, layered leaf
clusters and heavier rooted trunks. The new park canopy replaces one
radial crown with five lower overlapping boughs around the existing
authored forks and an offset higher leader. Bough heights, spread and
leaf tint vary from the same stable source phase. Trunk radius increases
from 0.085 to 0.11 of tree spacing. Each smaller bough uses 960 leaves
(previously one 1,920-leaf crown), with leaf size adjusted to the smaller
bough span. Town and route canopies retain their existing leaf counts.
This increases total park canopy density; performance requires native
review. Source plots, collision and the single cutaway owner remain.

Isolated execution of the actual park-tree/crown/branch functions with
geometry buffers passes all seven existing park tests (normals/winding,
roots, low planting, stable variation and cutaway ownership). Temporary
harness files were removed automatically. Integrated checks and a muted
native build are live in `target/outdoor-park-layered-bough-muted-tests.log`
and `target/outdoor-park-layered-bough-muted-build.log`. A GPU frame is
required before accepting the candidate's appearance. The concept's
incorrect centerpiece/grass distribution remains non-authoritative.

The integrated layered-canopy run completed exit 0 with all seven park
tests passing. Log: `target/outdoor-park-layered-bough-muted-tests.log`.
The same-angle muted native preview build is still active. This validates
checked geometry only; the new silhouette and increased canopy density
remain visually unaccepted until the fresh frame is reviewed.

The first layered-canopy native build completed exit 0 in 8m50s. Reviewed
the fresh muted actual 45-degree park frame
`target/outdoor-park-layered-bough-45-muted-reviewed.png`: 1800x1350,
49,276 colors, luma range 241.1, capture exit 0 in 20.92s. The capture
time is higher than the prior single-crown frame (9.58s), consistent with
the greater foliage density, though this is not a controlled FPS benchmark.
The new canopy has better depth and a heavier trunk, but still reads as
stacked broccoli; dark bough bodies contrast sharply with the leaf cards.
This version is not accepted as the final art target.

A follow-up lowers the leader from 0.78 to 0.71 of tree height, spreads
lower boughs farther and broadens/deepens them into an umbrella silhouette.
Park bough-body colors blend 35% toward their leaf color; park per-leaf
brightness variation narrows from 0.80..1.12 to 0.88..1.08. Route/town
color paths remain unchanged. Updated seven-test validation and a muted
native build are live in `target/outdoor-park-spreading-bough-muted-tests.log`
and `target/outdoor-park-spreading-bough-muted-build.log`. This follow-up
requires fresh GPU review before appearance is accepted. Both earlier
owned jobs and the first capture exited normally; no preview is running.

The spreading-canopy integrated run completed exit 0: all seven park
geometry tests pass. The native build remains active. Source inspection
also found a ground-treatment gap: the dedicated park masonry append in
`GroundFinish::append_surface` returns before applying `GroundFinish`'s
canopy/trunk darkness, while the color shading pass only includes lawn
and path materials. Park paving therefore bypasses that contact treatment.
This is a source finding, not yet a rendered fix; verify its visual effect
separately after accepting/revising the spreading canopy.

## Spreading canopy review and paving shade candidate — 2026-10-11

The spreading-canopy native rebuild completed exit 0 in 9m49s. Reviewed
the fresh muted actual 45-degree frame
`target/outdoor-park-spreading-bough-45-muted-reviewed.png`: 1800x1350,
48,011 colors, luma range 240.8, capture exit 0 in 15.90s. The canopy
now has a broader umbrella silhouette and softer leaf/body contrast.
Retain this improvement over the first layered-bough version, but the
clumps still look too solid and the trunks/planting/ground remain below
the saved reference. This is not overall visual acceptance. The owned
capture exited normally.

Further source inspection showed that existing tree contact shade is
bounded to its source lawn plot. Merely multiplying paving by that shade
would not create the missing overhanging canopy treatment outside the
plot. A separate park-only shade field now uses modeled-tree footprint
centers, a daylight-direction offset, soft elliptical coverage, and stable
map-space filtered-light noise. Small sign contacts are excluded. The
maximum shade is restrained to 22%, with a slight cooler color response.
The dedicated park masonry append applies it to newly appended colors;
it does not remap lawn/paving cells, change the masonry partition, add
geometry, or move any floor. This is a stylized presentation cue, not
proof of improved real-time shadow rendering.

The new regression checks that a modeled-tree canopy darkens eligible
park masonry while positions, indices, normals, and the shade bound are
preserved; distant paving receives zero influence. Integrated validation
is live in `target/outdoor-park-masonry-shade-muted-tests.log`. A fresh
muted GPU comparison is required before accepting this candidate.

## Native park canopy registration correction — 2026-10-11

The prior masonry shade regression passed and the muted native build completed
successfully. Its fresh 45-degree capture,
`target/outdoor-park-masonry-shade-45-muted-reviewed.png`, completed exit 0
in 15.66s, but did not establish a visible shade improvement. Inspection found
that GroundFinish accepted only the Johto modeled-tree ground path, so actual
4×4 park trees never entered its shadow list. The previous regression manually
inserted a shadow and therefore missed this production integration gap.

Park canopy registration now shares the exterior renderer's complete native
tree and ground-sample validation. Original source bounds remain unchanged;
only the separate canopy color field extends onto adjacent paving. The revised
regression constructs a complete native 4×4 tree beside pavement and checks
automatic registration, masonry geometry preservation, restrained color
darkening, and rejection after each of the 16 source cells is altered.
Integrated verification is running in
`target/outdoor-park-native-canopy-muted-tests.log`. A fresh muted rendered
comparison remains necessary; neither the prior frame nor a unit test is
visual acceptance.

The corrected native preview build completed exit 0 in 4m24s. Reviewed
`target/outdoor-park-native-canopy-45-muted-reviewed.png` (actual orbit 2,
45 degrees, 1800×1350, 50,826 colors, luma range 240.6), capture exit 0
in 27.03s. Source-plot lawn contact darkening now appears. However, three
fixed image patches on the left tree-adjacent paving, the paving north
of the right trees, and the open plaza had exactly unchanged RGB values
against the previous frame. Other warm-pixel differences are confounded
by moving NPCs and do not establish successful paving blending. Retain
the registration correction, but the canopy projection still needs a
better visual treatment. Trunk silhouettes, dense leaf clumps, sparse
flower planting, and opaque pool water remain below the reference.

The integrated retry completed exit 0 with the production canopy-registration
regression passing. The freshly rebuilt renderer test binary also passed the
complete-native-tree ownership regression and the original-floor park plaza
partition regression. The initial combined test attempt failed on an unrelated
array field access in a gym model test (`max.y`); that access was corrected to
`max[1]` before retrying. These checks establish the guarded source connection
and preserve floor geometry; the rendered paving limitation above remains.

## Fuller, lower-forked park stems candidate — 2026-10-12

The current 45-degree park frame still shows long tapered stems under dense
canopy clumps. The saved concept instead has shorter, full stems with visible
branching and a localized root flare. Park tree height now varies from 5.25
to 5.6 tiles rather than 6.34 to 6.76; the stem retains 78% of its radius at its upper
end, and the flare falls away near the roots instead of tapering the whole
exposed trunk. Main forks start lower, follow the leaning stem center, and
have fuller branch bases. Other tree families and map ownership are unchanged.

All seven geometry tests passed in an isolated execution of the actual current
foliage functions and park source module, using Bevy's real vector types and
plain mesh buffers. Temporary harness files and executable were removed.
Log: `target/outdoor-park-full-stem-source-tests.log`. The first integrated
attempt failed on an unrelated missing helper in a dungeon test; that test was
already corrected in the shared checkout before the retry. Integrated retry
and native build are live in `target/outdoor-park-full-stem-muted-retry-tests.log`
and `target/outdoor-park-full-stem-muted-build.log`. This remains an unaccepted
visual candidate until a fresh muted capture is reviewed.

The first stem rebuild completed exit 0 in 3m39s. Its muted actual 45-degree
capture, `target/outdoor-park-full-stem-45-muted-reviewed.png`, completed exit 0
in 20.70s (1800×1350, 50,672 colors, luma range 240.6). Trees have a shorter,
broader proportion, but their forks remain largely hidden. Main limbs were
still bowing upward into the canopy, so shortening the stem alone did not
establish the reference's visible branching.

The main park boughs now sweep outward through a quadratic curve before rising
into their crowns. Their surface normals follow the curved, tapered geometry,
their duplicated ring seams match exactly, and their bark palette matches the
stem. All eight isolated actual-source geometry checks pass, including a new
outward-winding and closed-seam regression for the spreading boughs. Temporary
harness files were removed. Log:
`target/outdoor-park-spreading-forks-source-tests.log`. This further shape edit
postdates the first stem frame and awaits its own muted native comparison.

The first stem integrated retry completed exit 0 with all seven park geometry
tests passing. Further source inspection found NationalPark and its contest
variant still using the distant one-cascade shadow profile, while Route26,
Route29 and NewBark use tighter grove cascades. Both park variants now use the
same near-foliage profile (three native cascades, first bound 256, far bound
2048; one WebGL cascade with far bound 1024). The transition regression covers
both parks and restoration of the distant profile outside those daylight maps.
This lighting change and the spreading-bough geometry are queued together in
`target/outdoor-park-spreading-forks-muted-build.log`, with updated geometry
tests in `target/outdoor-park-spreading-forks-muted-tests.log`. Actual leaf
shadow improvement remains unproven until the new frame is inspected.

The spreading-bough/shadow native rebuild completed exit 0 in 6m53s. Reviewed
`target/outdoor-park-spreading-forks-shadow-45-muted-reviewed.png` (actual 45°,
1800×1350, 56,529 colors, luma range 240.6), capture exit 0 in 23.35s. Main
forks now read beneath the crowns, and native cast shadows appear clearly
across the lawn. Retain this shape and grounding improvement. The six opaque
canopy bodies still cast large solid shadows; this is not the reference's
filtered leaf light. Broad pale paving, sparse flower beds and flat pool water
also remain below the concept. Integrated eight-test validation is still live.

Further source inspection found that park terrain is excluded from the
warm-exposure/cool-shadow daylight material selection at terrain publication
(`cave_reveal.z`); only NewBarkTown, Route29 and Route26 receive that treatment.
The new frame does not contain a fix for this separate color-selection gap.
Secondary fork anchors also still follow a straight root-to-tip interpolation;
they should follow the new main bough curve when that geometry is refined.

The fresh integrated renderer test binary passed all eight park geometry tests
and the grove shadow transition regression (one matched test, including both
park variants and the first native cascade bound). Logs:
`target/outdoor-park-spreading-forks-direct-tests.log` and
`target/outdoor-park-shadow-transition-direct-tests.log`. An initial exact
filter used the wrong module path and matched zero tests; it was corrected to
`renderer_tests::grove_shadow_detail_resets_when_leaving_the_daylight_maps`
before recording this pass. The combined Cargo parent is still compiling
other packages with no matching geometry tests, so its eventual exit is not
needed to broaden these focused results into an overall rendering claim.

## Park daylight color and complete-bed planting candidate — 2026-10-12

The previous spreading-bough combined test run completed exit 0. Park and
contest terrain now receive the same warm-exposure/cool-cast-shadow material
selection as the existing exterior groves, while cave reveal ownership remains
unchanged. Secondary forks now attach to the actual curved main bough center,
rather than a straight root-to-tip chord.

Native flower heads remain on their existing source cells. Low overlapping
leaf rosettes now fill the two interior rows of a source-complete native bed;
unknown or altered bed drawings still refuse replacement. Planting is confined
to each interior cell, skips an already claimed cell, adds no ground planes,
and claims no additional floor or flower ownership. Leaf height stays below
0.30 tile and phase uses absolute source coordinates retained by the placement.
The existing complete-bed regression now requires this planting to be present
inside the bed while preserving its 44 border claims and original footing.

Ten isolated actual-source geometry checks passed (eight tree, two flower),
including bounds, low height, normals and source-stable translation. Temporary
harness files were removed. The initial integrated build and tests failed on
the new helper's module visibility; the shared checkout added a wrapper before
the retries. Current verification logs:
`target/outdoor-park-daylight-planting-muted-retry-build.log` and
`target/outdoor-park-daylight-planting-muted-retry-tests.log`. The isolated log
is `target/outdoor-park-daylight-planting-source-tests.log`. Color and density
still require fresh muted visual review; neither is accepted from these checks.

The daylight/planting native retry completed exit 0 in 3m13s. Reviewed
`target/outdoor-park-daylight-planting-45-muted-reviewed.png` (actual 45°,
1800×1350, 68,874 colors, luma range 241.1), capture exit 0 in 24.09s. The
bed now has visibly fuller low greenery between its original flower samples,
and exposed terrain has a warmer daylight palette. Retain this improvement,
but the low planting still reads as coarse rosettes rather than the concept's
rich continuous garden. Canopy clumps and cast shadows remain too solid.
The bright raster-like patch at the upper-left fringe also needs inspection.

The fresh integrated renderer binary passed all 38 matching park regressions,
including native bed ownership, added interior planting and unchanged footing
(`target/outdoor-park-daylight-planting-direct-tests.log`). The combined Cargo
parent completed exit 101 because its broad `park_` filter also selected an
unrelated controller contest test that requires the absent external pack
`/Users/ryanculligan/GitHub/apps/web/assets/data/content-packs/core-modular.crystalpack`.
No pack was copied or restored to satisfy it. This does not contradict the
38 renderer passes and does not establish gameplay verification.

## Open park canopy candidate — 2026-10-12

Park boughs now omit their closed inner bodies and retain the existing leaf
geometry. Route and town crown bodies remain unchanged. Nine isolated
actual-source geometry checks pass, including a new directional-ray check:
each sampled bough retains substantial leaf coverage while permitting real
light rays through gaps that a closed body would block. This proves individual
bough permeability, not full-tree filtered shadows or visual acceptance.
Temporary harness files were removed. Log:
`target/outdoor-park-open-canopy-source-tests.log`. Native rebuild and integrated
foliage checks are live in `target/outdoor-park-open-canopy-muted-build.log` and
`target/outdoor-park-open-canopy-muted-tests.log`. The daylight/planting frame
predates this canopy edit.

The open-canopy native build completed exit 0 in 5m29s. Reviewed
`target/outdoor-park-open-canopy-45-muted-reviewed.png` (actual 45°,
1800×1350, 69,488 colors, luma range 240.6), capture exit 0 in 24.57s. Real
gaps now separate the leaves and the lawn receives broken leaf shadows instead
of only solid body shadows. Retain this improvement over the closed-core frame.
The bough layout remains repetitive, planting remains coarse, paving blending
and pool water remain unfinished, and the bright upper-left fringe persists.
This is not overall park or outdoor visual acceptance.

The ignored Route26 surface probe now distinguishes wall-plane winding from
averaged vertex-normal disagreement, using original source X/Z and final Y
to reconstruct the vertical reference plane. Its first launch failed before
compilation because the current library was temporarily absent during the
live integrated test rebuild; only older artifacts were present. Those older
libraries were not used. No new wall-fold result exists yet.

The fresh integrated renderer binary passes 16 foliage checks and 39 park
checks, including open-bough ray permeability, native bed planting/ownership,
source mutation refusal and preserved floor geometry. Direct logs:
`target/outdoor-park-open-canopy-direct-tests.log` and
`target/outdoor-park-open-canopy-ownership-direct-tests.log`. The combined
foliage-filter Cargo run also completed exit 0.

The corrected Route26 audit compiled against a snapshot of the fresh library
and completed exit 0 using the verified external pack. In the same 32×48-cell
crop it found 107,190 nondegenerate stone triangles, zero downward cap
projections among 54,764 strong samples, zero mismatches on 1,008 bank-height
rays (maximum error 9.536743e-7), and zero holes on 252 drop-face rays.
The new wall metric found 80 reversed projections among 9,260 original
wall-plane samples; 75 faces still oppose their averaged surface normals.
This projection metric uses final Y to reconstruct a source-space plane and
does not alone prove a 3D fold or self-intersection. Early examples involve
source corners around (24,36) and (20,44), suitable for targeted wall review.
Log: `target/outdoor-route26-wall-plane-muted-probe.log`. Temporary library
snapshots and executables were removed. No current preview remains running.


### Full Route 26 audit and terminal taper follow-up

The smooth adjoining-floor support field passed all 16 outdoor-stone unit
checks in the muted isolated build. Its actual 90-degree capture changed the
right bank contact, while the foreground inside-corner sliver remained unchanged.
The actual 45-degree frame also remains below the cliff shape/colour target.
Captures: `target/outdoor-route26-support-field-90-muted-reviewed.png` and
`target/outdoor-route26-support-field-45-muted-reviewed.png`.

The probe was expanded from a 32x48-cell crop to the entire source map.
The first full-map run found zero downward cap triangles among 218812 strong
original cap projections. Its 36 bank-cap mismatches and four drop misses were
at a terminal taper: that first probe incorrectly expected untapered heights.
These numbers therefore do not establish four physical holes. The 129 reversed
wall-plane projections likewise remain an indirect diagnostic, not proof of
self-intersection.

Inspection found that the composed east/west end taper and southward bank rise
were represented by one large cap quad per cell. That interpolation makes a
diagonal dip rather than the intended composed surface. The candidate now uses
4x4 subdivisions for the Johto 0x57 end caps, preserving native UVs, footprint
and other east/west ramps. The terminal regression checks the composed rise at
all cap vertices. The probe now scales both cap expectations and front-face
sample heights by the end taper. Verification of this candidate is pending;
no visual acceptance is claimed.


The fresh full-map probe completed successfully against the external browser
pack: Route26 is 40x216 source cells. All 3024 cap rays across 336 bank cells
(including the tapered end) passed, with maximum height error 0.02249714 world
units. All 756 drop-face rays passed, including samples 0.01 units above the
floor. This is geometric coverage evidence, not visual acceptance or proof that
all wall surfaces are free of self-intersection. The fresh renderer unit binary
also passed all 23 tests matching `ledge`, including the terminal-cap regression.
Logs: `target/outdoor-route26-terminal-taper-muted-probe.log` and
`target/outdoor-terminal-taper-direct-muted-tests.log`.

The integrated Cargo terminal-cap run also completed with exit 0. The native
example build remains in progress (owned session 30113); visual review of this
end-cap candidate is still pending. The separate foreground inside-corner sliver
is unresolved and the outdoor quality goal remains active.


### Muted terminal-end visual review and corner localization

The terminal-taper native build completed with exit 0 (4m56s). The muted Route26
capture at runtime (15,70), orbit 2 (actual 45 degrees), zoom 0 completed with
exit 0 in 40.11s. The 1800x1350 frame has 80539 colours and luma range 238.4:
`target/outdoor-route26-terminal-taper-45-muted-reviewed.png`. It was inspected:
the bank end meets lawn in this view, but its sharp footprint, coarse planting
and repeated tree crowns still do not establish the requested visual quality.
This view is separate from the unresolved foreground inside-corner sliver at
the earlier (10,28) framing.

A named opt-in capture diagnostic, `CRYSTAL_RENDER_TEST_TRACE_FRAME`, records
only map/camera/grid metadata when a screenshot is requested. It remains silent
without that environment variable and does not change authoritative gameplay.
The purpose is to locate the visible corner in source coordinates before
altering its shape. An ignored Rust ray probe uses the actual pack-derived
terrain and original mineral UVs; no game content is exported or committed.


The traced native build completed with exit 0. A muted actual-90-degree capture
at (10,28), zoom 1 completed in 35.22s with exit 0:
`target/outdoor-route26-corner-source-90-muted-reviewed.png` (1800x1350,
54008 colours, luma range 236.5). It was inspected; the sliver remains.
Capture metadata is authoritative: center (0,0), grid origin (-38,-1), grid
116x114, tile size 32x32, viewport 640x576, rotation step 2. Earlier probes used
8-unit tiles. The probe now supports the actual tile scale before normalizing
its outputs for comparisons, retaining the native shape/relief sampling.

At native 32-unit tile scale, the full 40x216-cell map still passes all 3024
bank-cap rays (maximum normalized error 0.02249714) and all 756 drop-face rays.
There are zero downward triangles among 218786 strong original cap projections.
Log: `target/outdoor-route26-native-scale-corner-ray-muted-probe.log`.

Camera-derived rays through the foreground sliver localize the join near source
(28,56): the adjacent source drawings are 0x6a (west-facing band north of the
join), 0x70 and 0x71 (bank caps south of it). At screenshot pixel (900,1220),
the first two surface hits are near normalized height 27.84 and 27.79 at that
corner. Pixel (880,1210) crosses an upper wall near height 27.64, then lower
surfaces near 14.22 and 11.32; pixel (910,1220) hits the level floor. These rays
identify the relevant geometry but do not, by themselves, prove self-
intersection. Inspect the resolved tiers and shared corner warp at this join;
do not treat a texture change or the earlier support-field check as a fix.
Log: `target/outdoor-route26-native-corner-edge-rays-muted.log`.
All owned previews and diagnostic runs completed; no preview is left running.


### Shared inside-shelf corner constraint

Resolved footing at the traced join is 32 normalized world units in all three
shelf quadrants, with ordinary ground at (27,55). The raw source classifier's
16-unit heights in the earlier ray-neighbor output were not resolved elevations;
this is an inside corner of one shelf, not a lower cap meeting a taller cap.

The sculptor now detects that same-height, three-quadrant topology. Both walls
and the cap retain the common vertical corner edge, with one source-position
smooth fade over two cells into the surrounding erosion. Different-height
neighbors are excluded. This removes the endpoint roll created by applying
perpendicular finite bevels through the shared inside corner. The regression
checks both 8- and 32-unit tile scales, common cap/wall vertices, and retained
weathering away from the join. The final integrated run passed all 17 stone
checks with exit 0: `target/outdoor-inside-equal-shelf-muted-tests.log`.

The final native build completed with exit 0 in 1m31s. Its fresh actual-scale
full-map audit retained zero downward cap triangles (218786 strong projections),
zero mismatches over 3024 bank-cap rays, and zero misses over 756 drop-face rays.
Wall-plane projection reversals decreased from 129 to 23; this remains an
indirect diagnostic, not a complete self-intersection proof. Log:
`target/outdoor-inside-equal-shelf-native-scale-muted-probe.log`.

Muted captures at (10,28), zoom 1 were inspected at actual 90 degrees (orbit 4,
34.69s, 53581 colours, luma range 234.3) and actual 45 degrees (orbit 2,
24.67s, 54258 colours, luma range 231.0):
`target/outdoor-route26-inside-equal-shelf-90-muted-reviewed.png` and
`target/outdoor-route26-inside-equal-shelf-45-muted-reviewed.png`.
The identified folded strip is gone in the 90-degree frame; the 45-degree join
also reads as a continuous shelf. The matching classic 2D capture was inspected:
`target/outdoor-route26-inside-equal-shelf-2d-muted-reviewed.png` (13 colours).
This validates that source comparison, not the overall visual standard. Broad
uniform cliff slabs, cap shape, sparse grass, repeated tree crowns, and the
remaining Park/reference gaps still require work. All owned runs are terminal
and no preview remains open.


### Rejected normal-crease candidate

A trial split mineral normal averaging at steep face angles, then used
connected face-direction groups to preserve gradual bevels. It changed no
vertex positions or terrain support. Both versions violated the existing
rounded cap/wall normal continuity regression. The first version also selected
inconsistent per-patch smoothing cones. The connected version passed 17 of 18
checks but still failed the rounded-shoulder join, so it was rejected rather
than weakening that blending requirement.

Its muted 45-degree native capture failed authoritatively at the 60-second
startup limit (exit 1, `target/outdoor-route26-connected-mineral-creases-45-muted-reviewed.log`),
leaving no verified image. The trial initially allocated local normal vectors
for all scene vertices; a mineral-only allocation was tried, but this did not
resolve the lighting regression. The timeout's exact cause was not isolated
from concurrent compilation/load, and no frame-rate conclusion is drawn.

The candidate and its new crease test have been removed. The angle-weighted
smoothing and original angle/area regression are restored; the previously
verified inside-corner constraint and terminal-taper fixes remain. A fresh
muted regression/build is pending for the restored source. Do not launch the
last candidate executable as a review result; use the rebuilt restored source.
Broader shape improvements must come from the actual mineral planes while
preserving the shared cap/wall lighting and floor-contact constraints.


### Cap clefts between broader mineral planes

The restored smoothing passed its fresh direct 17-check run with exit 0 and
then the integrated Cargo run completed with exit 0. The normal-crease trial
remains removed. Logs: `target/outdoor-restored-smoothing-direct-muted-tests.log`
and `target/outdoor-restored-smoothing-muted-tests.log`.

A geometric cap trial adds narrow linear clefts through the shared erosion
field, with quieter small cap undulations around the existing broad mineral
planes. It retains the common floor bounds, tree seats and inside-join fade.
The new mesh regression samples actual cap vertices across four source phases:
resolved valleys must exceed 0.08 tiles relative to both neighbours, and the
cap triangles must retain upward winding. This is stronger than asserting a
nonzero fissure function. The first version passed all 18 stone checks.

The first version's muted actual-45-degree Route26 frame was inspected:
`target/outdoor-route26-cap-clefts-45-muted-reviewed.png` (1800x1350, 54264
colours, luma range 227.9, exit 0 in 51.45s). It shows shallow geometric lines
across the cap, but the overall cliff masses remain too uniform. Its full-map
native-scale probe retained zero downward cap triangles and all 3024 cap /
756 drop-contact rays passed. Wall-plane projection reversals increased from
23 to 38, with 87 triangles opposite averaged normals (previously 73).
Log: `target/outdoor-cap-clefts-native-scale-muted-probe.log`.

The revised candidate samples the cleft from authored coordinates before the
bevel moves them, while broad crest erosion still uses the rounded coordinates.
This keeps a fissure stationary through successive wall rings. It is pending
fresh checks and native review; neither its geometric quality nor the overall
outdoor goal is yet accepted. Build/test logs:
`target/outdoor-source-anchored-cap-clefts-muted-build.log` and
`target/outdoor-source-anchored-cap-clefts-muted-tests.log`.

The source-anchored revision's fresh full-map probe completed with exit 0 at
native tile size 32: 218786 cap-projection samples contained zero downward
triangles, all 3024 bank-cap rays matched (maximum normalized error 0.02249714),
and all 756 drop rays found contact. Wall-plane projection reversals decreased
from the first cleft trial's 38 to 28, still above the pre-cleft baseline 23.
Triangles opposite averaged vertex normals increased to 105 (first trial 87,
baseline 73). These indirect diagnostics do not prove self-intersection, but
the revision is not visually accepted from contact checks alone. Log:
`target/outdoor-source-anchored-clefts-native-scale-muted-probe.log`.

The revised actual-45-degree native capture exited 1 after the capture
timeout and produced no image. No visual conclusion is drawn from that run:
`target/outdoor-route26-source-anchored-clefts-45-muted-reviewed.log`.
The integrated stone test process remains pending at this checkpoint.

The integrated source-anchored cleft run subsequently finished with exit 0:
all 18 outdoor-stone checks passed in the freshly linked renderer test binary
`crystal_voxel_view-dc3731eb4a201fc3`, including resolved cap valleys, cap
winding, shared inside-shelf edges, native-scale joins, and floor support.
The smaller capture also exited 1 with no image. A third muted 45-degree
capture was started only after this owned build finished; it is pending at
this checkpoint (`target/outdoor-route26-source-anchored-clefts-45-muted-after-build.log`).

The post-build muted 45-degree capture completed with exit 0 in 55.16s and
was inspected: `target/outdoor-route26-source-anchored-clefts-45-muted-after-build.png`
(1280x960, 43746 colours, luma range 226.3). The visible inside corner and
bank joins are continuous. The clefts are too subtle to make the broad tops
read as convincing fractured stone: the caps still look padded, uniform
masses, especially the foreground plateau. This is not visual acceptance.
A muted actual-90-degree capture is pending to examine the exposed wall joins.

The muted actual-90-degree view completed with exit 0 in 56.29s and was
inspected: `target/outdoor-route26-source-anchored-clefts-90-muted-reviewed.png`
(1280x960, 43583 colours, luma range 233.1). The previously identified inside
corner remains closed; the foreground face meets its floor. Both current
views nevertheless show rounded, padded cliff crowns rather than distinct
broken mineral beds. The next shape iteration should resolve clefts across
several mesh samples, rather than increasing narrow undersampled pits; the
upper silhouette needs broader angular breaks. Colour/material blending and
the sparse grass floor remain separate unresolved visual requirements.
All owned processes started or resumed in this checkpoint have finished.
No overall outdoor acceptance or gameplay claim is made.

The next cap trial widens the authored-coordinate linear cleft band from
0.05 to 0.12 and increases its maximum depth from 0.65 to 0.9 tiles before
the existing support bounds and erosion fade. It changes neither the source
footprint nor mesh density. The regression now also compares actual interior
vertices against the broad-bed-only surface, requiring adjacent samples to
retain a resolved cleft rather than isolated depressions. Native build
completed with exit 0 in 3m02s. Fresh tests, full-map contact verification and
muted visual review are pending; this candidate is not accepted yet. Logs:
`target/outdoor-broad-cap-clefts-muted-build.log` and
`target/outdoor-broad-cap-clefts-muted-tests.log`.

The wider-cleft muted actual-45-degree capture completed with exit 0 in
52.99s and was inspected: `target/outdoor-route26-broad-cap-clefts-45-muted-reviewed.png`
(1280x960, 43702 colours, luma range 228.9). Cut bands are broader, but
the dominant upper masses still read as lumpy rounded plateaus. The visible
inside-corner and bank contacts remain closed. This remains a partial shape
trial, not visual acceptance of the cliffs or the outdoor goal.

The wider-cleft native-scale full-map probe completed with exit 0:
390246 stone triangles, 218786 cap-projection samples with zero downward
triangles, 27 wall-plane projection reversals, and 102 triangles opposed to
averaged vertex normals. All 3024 bank-cap rays matched (maximum normalized
error 0.02249714); all 756 drop rays found contact. The fresh renderer binary
`crystal_voxel_view-dc3731eb4a201fc3` passed all 18 stone checks directly,
including the new adjacent-sample requirement. The integrated Cargo process
remains pending. Logs: `target/outdoor-broad-cap-clefts-native-scale-muted-probe.log`
and `target/outdoor-broad-cap-clefts-direct-muted-tests.log`.
The wider-cleft 90-degree capture exited 1 at its timeout and produced no
image; no side-view conclusion is drawn. User priority now expands the visual
review to the complete New Bark–Route29–Cherrygrove outdoor stretch.


## Forest palette separation and sheltered water — 2026-10-14

The normal 45-degree IlexForest `(10,12)` review exposed a second blue nighttime light over its fixed night source palette. Native forest lighting now uses neutral fill with restrained warm direct light, retaining the source frame and classic palette unchanged. Verified forest floors use cooler humus/moss modulation and broad filtered warmth through the existing texture read; mild directional response gives exterior folds and raised surfaces more form. Sheltered forest water has lower normal strength, little breaking wash, reduced open-sky reflection and a subdued blue-green depth palette. Outdoor reflected streaks and caustics are softer. Verified Johto path cells adjoining water receive a wider damp-sand transition without painting water, changing heights or moving source support.

Native build passed. The exact test executable passed 37 distinct targeted lighting, water, shoreline, forest, material-registration and cave-restoration tests; one existing opt-in forest benchmark remains ignored. Matching source-state 2D/native captures were inspected at IlexForest `(10,12)` and CherrygroveCity `(9,8)`, hour 12, normal 45-degree camera. A first forest render was too yellow; the final shader reduces that floor cast and cools the pond. The final shader was compiled and inspected through the existing native shader-review override on the current optimized client. Opt-zero capture attempts timed out under concurrent build load; those attempts are not visual or performance evidence.

No new lights, draw passes or texture reads were introduced in this pass. Damp shoreline cells reuse bounded floor subdivisions; other ground geometry is unchanged. FPS remains unbenchmarked. Forest canopy repetition, sparse-looking needle crowns and rigid bank outlines still limit composition and source-scenery fidelity; whole-scene/reference acceptance remains open. Review scratch outputs were removed; content packs and other build caches were preserved.


## Shared smashable rock readability — 2026-10-17

The canonical smashable `rock` has broader shoulders and a 0.64 × 0.54
footprint, retaining its 0.43 height, floor root, closed 98-triangle body and
shared native-palette material. It stays within one source actor cell and
remains smaller than the separate Strength boulder. Its focused editable source
is `art/johto/source/actor-props-rock-refinement.blend`; the original generator
reproduces the canonical runtime JSON with `--only=rock --skip-preview`.

A dark live palette previously selected only its nearly black dominant tone,
then shaded that tone again across the facets. Stone props now select a real
significant highlight from that same palette when the dominant tone is very
dark. Day pigment is preserved, and absent highlights are never invented.

The native build and all 10 focused Rust checks pass: seven actor-prop registry,
geometry and palette checks plus native palette/movement, cave, and Strength
boulder controls. The broader Python kit audit still stops at an unrelated
retired `slowpoke.mesh.json` reference; no full Python-kit pass is claimed.

Matched 3D/2D production pairs were personally inspected offscreen at
CianwoodCity `(10,18)` at hours 12 and 22 and Route40 `(9,6)` at hour 12, with
the normal orbit-1/zoom-0 camera. All paired states are preserved. The night
Cianwood and day Route40 checks each record 64 frames, two positions and an
LR return, with terrain active and never pending. The first maroon night
candidate was rejected; the accepted stone has readable muted-purple facets.
Captures and the receipt remain ignored under `target/outside-art-*-stone-final*`
and `target/outside-art-smashable-stone-verification.txt`. All captures explicitly
used headless image targets and muted audio. This is a shared-prop improvement,
not acceptance of the complete outdoor scene goal.


## Continuous native Kanto road fields — 2026-10-17

Cycling Road's blank native pavement was modeled as one inset slab per source
cell, producing a conspicuous mint tile grid. Authenticated Kanto road drawings
now give only their blank `$23` fields a continuous warm mineral plane with
map-registered paving grain. Authored curb strips, circular covers, brick bonds,
water-edge geometry and standing heights retain their existing construction.
The complete-drawing and opaque-palette guards are unchanged; this is one shared
source finish across maps, not a Route17-specific asset.

All 14 focused paving regressions pass. The new regression checks full area,
edge coverage, native standing datum and material registration after translation.
Existing controls cover all ten outlined road drawings, live palettes, malformed
art, source ownership, raised pier/bridge contacts and the tournament brick path.
The native client build passed.

Matched 3D/2D production views were personally inspected offscreen at Route17
`(9,45)` at noon, Route17 `(8,45)` at 22h, and the Route18 preview position at
noon. All pairs report preserved state, using orbit 1, zoom 0 and 640×640.
The night Route17 UD check records 64 frames and a return through `(8,44)`
to `(8,45)`, with terrain active and never pending. The first moving native
sample was also reviewed. The initial automatic Route17 pose was in the canal;
that image is a material diagnostic, not Surf or movement evidence.

Fuchsia's central `(20,14)` source/native review showed coherent existing
architecture, paving and post composition; no separate town redesign was made.
Route18's grass-covered NPC deserves a closer actor/priority comparison before
calling its readability a defect. These checks do not prove complete outdoor
scene acceptance. Captures and the receipt remain ignored under
`target/outside-art-route1*-continuous-road-*` and
`target/outside-art-continuous-road-verification.txt`. Every capture used a
headless image target and muted audio; no visible preview was opened.


## Johto side-entrance cloth coverage — 2026-10-17

The Ruins of Alph research entrance still showed a pixel-checkered floor mat.
The shared cloth finisher now authenticates both complete Johto side-approach
drawings: `$0b` has two mat columns followed by two path columns, while `$0e`
mirrors that layout. Both use the existing 16×32 woven finish, live averaged
source pigment, binding and stitching. No shader, asset or town-specific clone
was added. Original submitted triangles, contour normals, walking support and
source ownership guards remain authoritative.

Both cloth regressions pass, including the two atlases and both side orientations,
full weave extents, contoured floors, source phase/claim guards and live pigment.
The three connected north-gate controls and native doorway control also pass:
six focused tests total. Native client and audit builds passed. The freshly
built audit confirms 24 cloth cells at Alph (eight existing plus 16 side cells),
zero remaining `$40` mat fallback and no mesher errors. Route39's source-family
counts remain unchanged.

Production 3D/2D day and night pairs were personally inspected at the Alph
preview pose, orbit 1, zoom 0, 640×640; both preserve state. The side mat now
reads as a woven surface rather than enlarged source pixels. These were static
material checks, not movement or whole-scene acceptance. All captures remained
headless and muted. Source-audit JSONs were temporary and removed after the
counts were recorded in `target/outside-art-johto-side-cloth-verification.txt`.

The Route18 close comparison also cleared the suspected grass-height defect:
its native 2D grass substantially covers the same characters, while 3D retains
heads and clothing. Grass height was preserved. Route39's farmhouse appearance
was retained after confirming its shared native house/gate drawing; the separate
asset named `JohtoBarn` was not treated as authority to replace that picture.
Overall outdoor quality acceptance remains open.

## Outdoor rooftop sky and clock — 2026-10-17

Celadon Mansion, Goldenrod Department Store and Tin Tower roofs now share an
explicit map-and-atlas outdoor classification. Their finite-map border filler
reveals the sky rather than forming a saturated blue ground plane. Tin Tower's
internal sky blocks additionally require a complete native drawing with uniform,
opaque live source pixels. Authored scenery claims take priority over sky removal;
changed art, foreign atlases, missing map bounds and enclosed rooms retain their
existing fallback. Native deck geometry and standing heights were preserved.

These fixed-palette maps now publish the authoritative live RTC for exterior sky,
lighting and sky reflections. Their native 2D palette remains unchanged. The
enclosed Celadon rooftop house stays indoors. One shared identity predicate serves
the presenter and renderer, without separate town assets or additional cubemaps.

Eleven distinct focused tests pass: roof identity, two source/bounds masks, two
skybox checks, two cave-light controls, environment reflections, clear-color
classification, the mansion deck datum and retained-frame RTC publication. The
client build passed. A full-source coverage audit retained every existing authored
family count on all three roofs, added 432 authenticated Tin Tower sky cells, and
reported no mesher errors. This checks coverage rather than complete geometry
equivalence. Temporary source-audit files were removed after recording the result.

Matched production 3D/2D pairs were personally inspected at noon on all three
roofs and at 22h on Celadon Mansion, using orbit 1, zoom 0 and 640×640. All four
pairs report preserved state. The first night candidate incorrectly retained a
daytime sky; the live-RTC correction was rebuilt and verified with a new capture.
Every capture used a headless image target and muted audio, without opening a
window. Evidence remains ignored under `target/outside-art-*-rooftop-*` and
`target/outside-art-rooftop-sky-verification.txt`. This accepts the background and
clock correction only; rooftop architecture/material polish and overall outdoor
scene acceptance remain open.

## Native tower rooftop ceramic paint — 2026-10-17

The Tin Tower roof's authored slopes and ridge retained hard-coded blue-gray
paint, losing the warm ceramic identity visible in the native drawing. The shared
roof finisher now samples the live opaque palette of complete `$1c`/`$23` slope
drawings and paired `$50`/`$51` ridge tiles. Body tones supply the ceramic field;
existing facet brightness is preserved, with the established ceramic grain and
reflection role. No separate town model, texture, shader or roof geometry was
added. Missing pixels, altered source identities, transparency or an unsupported
palette retain the previous finish. Ordinary scenery keeps its connected-body
append path.

Three new focused regressions pass. They check positions, normals, triangle
indices, existing vertex prefixes, textured geometry and standing support;
live pigment changes; incomplete drawing and invalid paint rejection; ridge
volume; and material registration after translation. Translation uses a
`0.00001` UV tolerance for floating-point subtraction of faceted centers.
The existing slope endpoint/datum check and both rooftop sky guards also pass:
six focused checks total. The native client build passed.

The production day and night pairs were personally inspected against native 2D
at the same Tin Tower rooftop pose, orbit 1, zoom 0, 640×640. Both report preserved
state. The roof now reads as warm gold at noon and follows the source's purple
night palette at 22h; the walking deck, slope silhouette and ridge stay in place.
These are static material checks, not movement or complete scene acceptance.
All captures were muted, headless image targets. Evidence remains ignored at
`target/outside-art-tin-rooftop-ceramic-{day,night}{,-2d}.png` and
`target/outside-art-tower-roof-paint-verification.txt`. Wider outdoor review
remains open.

## Docked ferry proportions and continuous sea — 2026-10-17

Vermilion's port showed two linked defects: the ferry's complete model was
vertically compressed into a shallow barge, while its native sprite rectangle
seeded a pale, square shoreline beneath the hull. The shared ferry fit now
derives vertical scale from ship length and aligns the actual authored gangway
top (`1.1075` model units) to the quay. Its north docking seam and horizontal
plot are unchanged; the keel sits below the sea and the cabin, bridge windows
and rails retain their authored vertical proportions. The canonical ferry asset
and editable source were retained without modification or town-specific copies.

Both ports also give the complete, authenticated eight-block ship drawing a
continuous optical sea backing. The model resolver and shore preparation share
one source signature, including the same sixteen bow, cabin and keel anchors.
Incomplete, clipped, foreign-atlas or altered drawings retain their original
shoreline fallback. Real quay cells remain dry. This changes optical shore
distance only: standing water stays at its existing height, and source art,
collision, gameplay support, ledges and terrain geometry are unchanged.

Twenty-one distinct focused checks pass: four ferry checks, fourteen water-surface
checks (including one repeated ferry check) and four port controls. They cover
the actual gangway vertices, length-derived vertical scale, unit normals, retained
indices, the existing recessed-window geometry, both ports' sea continuity,
real quay contacts, altered source rejection, partial ships, buoys and rock
contacts. The native client build and whitespace checks passed.

Matched production 3D/2D views were personally inspected at Vermilion's default
berth pose at noon and 22h, and Olivine's berth approach `(7,22)` at noon. All
report preserved state, at orbit 1, zoom 0 and 800×640. The default Olivine pose
was also inspected as a quay control; it does not show the ferry, so it was not
counted as vessel evidence. Vermilion City's original street/waterfront view was
retained after review. These are static presentation checks, not boarding or
movement regressions. All captures were headless image targets with muted audio.
Evidence remains ignored under `target/outside-art-*-ferry-*` and
`target/outside-art-harbor-ferry-verification.txt`. Overall outdoor acceptance
remains open.

## Shared port stair landing and material clarity — 2026-10-17

The Olivine close review confirmed a modeled flight rather than a flat sprite,
but its darker treads and green ground-colored nosings/handrails made it read as
a grille. The shared harbor-stair treatment now uses the native flight's light
gray for stone, middle gray for metal and darkest gray for its recess. An external
pack regression checks those actual PNG swatches, including the old ground
swatch. No source palette, shader or canonical stair asset was changed.

The flight also placed its highest treads against the eastern water barrier.
It now turns within its original square opening to meet the dry western quay.
The resolver additionally authenticates all four west-landing source cells;
altered or unavailable landing art retains fallback. Three recessed concrete
faces close the shaft's north, east and south sides, preventing sky/sea from
showing behind the lower steps. The west landing stays open. Source claims,
opening bounds, tread heights, water level, collision, warp coordinates and
authoritative support remain unchanged. Both ports use the same treatment.

Seven focused checks pass: external pack swatches, complete flight/landing
identity and geometry, the shared stair material/cache control, and four port
controls. The geometry regression checks the west-facing grade, original bounds,
source-paint UV ownership and unchanged support. The client build and whitespace
checks passed. A south-facing trial was rejected because it hid the flight at
the gameplay camera angle; the final west-facing version was rebuilt and reviewed.

Matched production 3D/2D pairs were personally inspected in Olivine at `(10,8)`
at noon and 22h and in Vermilion at `(8,6)` at noon. All preserve state, using
orbit 1, zoom 1 and 800×640. These are static scene checks, not movement or
passage-warp regressions. Every capture was headless and muted. Accepted images
remain ignored under `target/outside-art-{olivine,vermilion}-stair-final-*`, with
the receipt at `target/outside-art-harbor-stair-verification.txt`. Rejected capture
trials were removed. Overall outdoor quality acceptance remains open.

## Shared Kanto front-entry cloth — 2026-10-17

Route 5's standard front gate carpet still displayed enlarged source pixels.
The shared cloth finisher now recognizes the complete Kanto `$04` drawing:
its two ground courses followed by eight `$4f` carpet cells. It reuses the
existing 32×16 weave, binding, stitching and live averaged source pigment.
No town-specific asset, texture or shader was added. Submitted floor triangles,
contour normals, support and custom ownership remain authoritative; incomplete
or altered drawings retain their native fallback.

All three cloth checks and fourteen paving controls pass: seventeen focused
checks total. The new front-entry regression checks exact contour triangles and
normals, unchanged support, full weave extents, live pigment changes, all sixteen
source cells' identity guards and custom ownership. The client and audit builds
passed. A fresh full-source audit reports no mesher errors and no remaining
unclaimed `$04/$4f` carpet cells in Route 5, Saffron or Celadon. Cloth totals are
eight in Route 5, eight in Celadon, and twenty-four in Saffron including its
existing side mats. Route 5's earlier authored-family counts are preserved;
Saffron's footing range remains exactly zero. Temporary source audits were
removed after recording those results.

Matched production 3D/2D pairs were personally inspected in Route 5 at `(8,16)`
at noon and 22h, Saffron at `(16,32)` at noon, and Celadon's rear entry at `(16,2)`
at noon. All preserve state, using orbit 3, zoom 1 and 800×640. The Celadon view
has the retained building cutaway active; it verifies the mat rather than full
architecture acceptance. These are static material checks, not warp or movement
tests. Every capture was headless and muted. Route 5's long white bypass strips
were retained after the native art and walking permissions confirmed their
horizontal role. Accepted captures and the receipt remain ignored under
`target/outside-art-*-cloth-*` and
`target/outside-art-kanto-front-cloth-verification.txt`. Overall outdoor quality
acceptance remains open.

## East Kanto exterior verification — 2026-10-17

Fresh full-source audits of Routes 6, 7, 8, 9, 10 North, 11, 14 and 15 report
no mesher errors. The remaining raw source entries were used to choose review
positions, not treated as proof of visible missing scenery. Route 6 demonstrates
the distinction: its host-only source audit has a clipped north-gate frontage,
while the production 3D/2D pair includes the neighboring map context and renders
the complete brick gate, roof, door, windows and steps. Existing bindings were
preserved.

The Power Plant was inspected at the automatic Route 10 North pose and from its
actual front approach `(3,10)`, at noon, orbit 1, zoom 0 and 800×640. The closer
default view legitimately fades the tall frontage to expose the player; the
door approach retains the opaque source-bound building. Production occupancy
validation already checks walking permissions, so the initial wall-spawn
diagnosis was rejected and the preview spawn code was preserved. The retained
building reveal keeps a faint whole silhouette, opaque ground backing and a
clear player aperture. Both focused reveal/aperture controls pass.

The Route 6 north-gate pair was inspected at `(10,2)` with the same capture
settings. All three production pairs report preserved state. These checks
verify specific views, not every audited route or complete outdoor acceptance.
Every capture was headless and muted. The source-audit JSON and log were removed
after recording the result in `target/outside-art-east-kanto-verification.txt`;
accepted images remain ignored under `target/outside-art-*power*-day*` and
`target/outside-art-route6-north-gate-day*`. No renderer, asset, controller or
preview-placement change was justified by this review. Overall outdoor quality
acceptance remains open.


## Native Kanto sand correction — 2026-10-17

The alternating $30/$39 speckles in native Kanto blocks $79/$7b are sand,
not a flagstone plaza. Their shared ground binding now uses the existing warm,
grain-finished path surface and no longer enables city paving. The separate
prop quadrant in $79 retains its original source faces. Source phase guards,
standing heights and authored claims remain unchanged. Genuine bridge/street
masonry retains its existing producer.

The Power Plant front approach (Route10North, 3,10; noon; orbit 1; zoom 0;
800×640) was inspected in a fresh headless, muted production 3D/2D pair.
The former staggered forecourt slabs are gone, replaced by continuous textured
sand. Focused sand and prop-preservation regressions pass. The broader Kanto
filter reports 278 passed, one ignored and one separate failure:
`kanto_directed_bank_replaces_owned_detail_and_grains_caps_without_support_changes`
expects a closed mineral-lip UV marker on its block-$07 fixture. No ledge code
was changed as part of the sand correction.

The FuchsiaCity (13,10; noon; orbit 1; zoom 4; 800×640) matched pair
also shows continuous sandy paths, with preserved source layout and state.
Both pairs report `state_preserved=true`; all captures were offscreen and muted.
Evidence: `target/outside-art-power-plant-sand-final-day*` and
`target/outside-art-fuchsia-sand-day*`.

The pending shared generic flower refinement was also inspected in the Fuchsia
pair: rounded closed petals and curved closed leaves replace the pointed petals
and flat leaf cards. All five original head/stem placements, material colors and
exact bounds remain unchanged. The canonical flower mesh has 940 triangles;
its editable flower-only source is
`art/johto/source/new-bark-flowers-refinement.blend`. Geometry checks verified
closed, consistently wound components with positive volume; the flower-filter
suite reports 35 passed, one ignored. No town-specific duplicate was added.
The temporary Blender export workspace was removed after publishing the shared
mesh and editable source.


## Seafoam coast and ground-supported approach — 2026-10-17

Route 20's Seafoam beach must meet sea level. It no longer inherits cumulative
terrace elevation from its one-way collision edge. The shared map policy is
used by terrain generation, deferred finishing and actor-support correction;
the native cave mass and collision remain intact.

A native Kanto brick strip enclosed by source-$31 sand can be ground-supported.
Its connected drawing now follows the actual ground datum instead of acquiring
the suspended pier's rise. Opposing water contacts on any part of the connected
strip preserve the original crossing behavior, including connected approaches.
Unknown or missing contacts retain the existing pier fallback. This is shared
geometry and source-context classification, with no map-specific model copy.

The external-pack pier regression verifies all 32 Seafoam approach cells have
zero-height ground support while the separate 32-cell offshore bridge retains
its original raised deck. It also covers ten other pier maps, including Route25.
Seven pier controls and the mesh/actor-support flat-floor control pass. The
latter now explicitly covers Route20 while retaining ordinary outdoor terraces.
The initial overly broad ground-contact candidate was rejected by the Route25
bridge check; final code preserves its raised support.

The Route20 (37,11), noon, orbit 1, zoom 0, 800×640 production 3D/2D pair
was inspected offscreen and muted. The tall artificial shoreline wall and pale
blue ramp wedge are gone; the sandy beach and brick approach meet the cave at
the coastal datum. The pair reports preserved state. Evidence is retained in
`target/outside-art-seafoam-level-day*`. Audits of Route20, Route21 and Cinnabar
report no mesher errors and unchanged authored-family coverage; these host-only
audits are not used as proof of finished terrace heights. No water level,
waterfall, cave-mouth asset, tree asset or source game data was changed.

The separate offshore platform was also inspected at Route20 (13,9), noon,
orbit 1, zoom 1, 800×640. Its matched native drawing retains the raised deck,
frame and pilings over level water. This pair also preserves state and was
captured headlessly and muted (`target/outside-art-seafoam-offshore-bridge-day*`).


## Softer coastal sky reflections — 2026-10-17

The ocean's broad dark checker pattern came from the cloud-reflection field,
whose sharp contours repeated across the short-wave normals. A diagnostic
capture with that layer disabled isolated the problem while retaining ripples,
shore wash and glints. The production wind-water branch now uses a wider sky
field, a softer cloud transition and lower cloud contrast. Existing wave
motion, source palette and scene-light exposure remain authoritative. The
change affects reflected color only; no surface heights or shore geometry
were edited.

Fresh production 3D/2D pairs were inspected at Route20 (37,11), noon and
midnight, orbit 1, zoom 0, 800×640, plus OlivinePort (10,8), noon, orbit 1,
zoom 1, 800×640. Seafoam retains the corrected coastal ground level and level
cave approach. Its foreground water has quiet blue depth with scattered
highlights instead of blocky dark cloud stamps; night retains the native
palette and dim exposure. The harbor preserves quay, stair opening and post
contacts. All three pairs report `state_preserved=true`. Captures were muted,
headless image targets using the rebuilt embedded shader, with no review
shader override.

All 14 `water_surface::tests` pass, covering source-clock motion, scrolling
registration, shoreline distances, swimmer contacts, pier/ferry coverage and
separate water styles. Production captures also verify shader compilation.
Evidence: `target/outside-art-seafoam-water-{day,night}*`,
`target/outside-art-olivine-water-day*` and
`target/outside-art-water-cloud-verification.txt`. Diagnostic shader copies
and their captures were removed after production verification. Twelve retired
tree JSON paths remain absent in both model roots; no tree assets were edited
or restored. Overall outdoor quality acceptance remains open.


## Shared cottage ceramic course refinement — 2026-10-17

Bill's cottage exposed oversized roof sheets in the shared native cottage
renderer. The existing closed barrel-tile primitive now forms five shorter,
staggered courses instead of three aligned courses. Clipped half-tiles finish
alternating row ends, and stronger rounded shoulders give the ceramic more
physical depth. Native roof pigment, silhouette, support deck, ridge, chimney,
facade and independent window roles remain intact. No per-town asset was added;
the changed append arm is the authenticated `Asset::KantoHouse` path, separate
from Pokémon Centers and Marts.

All five `kanto_house`-filtered tests pass. Existing controls cover complete
narrow/extended source bindings, independent paint roles, closed primitives,
translation invariance, bounds and the existing triangle limit. Fresh production
3D/2D pairs were personally inspected for Route25 (47,6), noon and midnight,
orbit 1, zoom 0, 800×640, plus the compact cottage at PewterCity (29,14), noon,
orbit 1, zoom 1, 800×640. The roof reads as individual ceramic courses at both
scales. The native day blue and night purple remain distinct; Bill's lower
windows retain their nighttime light. All three pairs preserve state. Captures
were headless and muted, with no shader review override.

Evidence: `target/outside-art-bill-cottage-roof-{day,night}*`,
`target/outside-art-pewter-cottage-roof-day*` and
`target/outside-art-cottage-roof-verification.txt`. The initial Route22/24/25
source audit reported no mesher errors; those host-only counts are not visual
acceptance of the three complete maps. Its JSON/log were removed after recording
the result. No tree assets, ledges, sea levels or source game data were edited.
Overall outdoor acceptance remains open.


## Kanto canyon cutaway backing — 2026-10-17

Route22's southeast canyon cutaway exposed the pale horizon through the
removed foreground rock. The existing shared mineral backing now has a late
Kanto pass after final terrain support. It projects only within actual sculpted
cap triangles, borrowing authenticated nearby dry-source support and native
mineral material. It excludes water, custom ownership and volcanic surfaces.
The Johto pass and cutaway shader remain unchanged. Narrower shader feathering
was rejected because it made a hard pale crescent; that diagnostic was removed.

Eight outdoor-mountain tests and three external-pack mountain footprint
regressions passed. Production 3D/2D pairs were inspected for Route22 (37,15),
noon and midnight, and (20,15), noon, all orbit 1, zoom 0, 800×640. State was
preserved in all pairs. The pale halo is gone while the player and mineral
backing remain readable. Evidence: `target/outside-art-route22-canyon-backed-*`
and `target/outside-art-route22-south-backed-day*`. All captures were offscreen
and muted, without shader overrides. This is local cutaway acceptance; overall
outdoor review remains open.


## Enclosed Kanto pond datum and bank grading — 2026-10-17

Route22's pond stayed at the base water datum while its surrounding land rose,
creating a deep rectangular pit. A shared source-based pass now gives each
complete, unclaimed native Kanto pond one level at its lowest authenticated dry
rim. The whole water surface and its existing actor/Surf support offset move
together. Native rocky bank caps and relieved support faces slope from the
existing outer terrace contact to that rim; the tall retaining wall is gone.
The pass preserves source footprints, source collision, dry outer contacts and
existing rocky detail. It excludes open/clipped seas, waterfalls, custom banks
and owned water. No per-map asset or sea-level exception was added.

Three focused regressions cover flat water geometry/support, bank contact and
relieved-wall grading with actor footing, plus unchanged open water, cascades
and custom controls. All pass, alongside eight footing and three native Kanto
shore regressions. The production client builds successfully and diff whitespace
checks pass. Production 3D/2D pairs were personally inspected at Route22 (24,10),
noon and midnight, orbit 1, zoom 1; Route22 (20,15), noon, orbit 1, zoom 0; and
the Seafoam coast, Route20 (37,11), noon, orbit 1, zoom 0. All are 800×640 and
preserve paired state. The pond is locally accepted without the deep wall; the
coast remains low with a continuous level ocean. Captures were offscreen and
muted, with no shader override. No tree assets or game content packs were edited.

Evidence: `target/outside-art-route22-pond-graded-{day,night}*`,
`target/outside-art-route22-pond-wide-day*`,
`target/outside-art-pond-sea-level-control*` and
`target/outside-art-enclosed-pond-verification.txt`. Overall outdoor review
remains open.


## Native shoreline night pigment and neutral grain — 2026-10-18

Fresh same-state surveys extended review to southern Cycling Road, Route17
(8,68), and the Tohjo Falls approach, Route27 (33,9), both noon, orbit 1, zoom 0,
800×640. Cycling Road's canals remain level, the biker and path readable, and
the native outlined road strips remain walkable curbs. Their all-FLOOR source
collision does not justify raising guardrails. Route27 retains its source house,
mountain mass and open approach. These are reviewed views, not whole-map or
gameplay acceptance. Captures: `target/outside-art-route17-south-survey*` and
`target/outside-art-route27-house-survey*`, both paired-state preserved.

Pallet's Oak's Lab review at (12,13), noon and 22h, exposed nearly black shoreline
rocks at night. The shared native bank producer already derives its pigment
from the current atlas palette; the ordinary scene lighting then applied a
second night tint. Its material coordinates now use a separate -70000 Z band.
The shader decodes the same map-space grain and uses source-paint lighting only
for that band. Mineral texture supplies neutral value detail rather than
replacing the native pigment with brown bedrock. Fixed-color mineral lips,
stairs and canyon backing keep their original -20000 band and lighting.
No vertex position, normal, topology, rock silhouette, support or water datum
changed in this pass. Kanto and Johto share the same native-bank producer.

The direct native-bank scrolling/material-band regression passes, alongside
34 shore-filtered checks, three fixed mineral-lip controls and four outdoor
stair controls: 42 distinct passing checks. The production client builds and
diff whitespace checks pass. Final embedded-shader 3D/2D pairs were personally
inspected for Route22 (24,10), noon and midnight, orbit 1, zoom 1; NewBarkTown
(13,10), noon, orbit 1, zoom 0; and PalletTown (12,13), 22h, orbit 1, zoom 0.
All are 800×640 and preserve paired state. Native stone texture remains visible
at night, daylight banks retain natural mineral variation, and the Route22
pond stays level with its accepted graded banks. The New Bark view is a
material control, not acceptance of its complete waterfront geometry.

Evidence: `target/outside-art-route22-source-shore-{day,night}*`,
`target/outside-art-new-bark-source-shore-day*`,
`target/outside-art-pallet-source-shore-night*` and
`target/outside-art-source-shore-verification.txt`. The broader all-shore
source-lighting diagnostic was rejected because fixed-color terrain shares
the old material band. Its brown-pigment result was also rejected. Diagnostic
shader copies and captures were removed. Final captures were headless, muted,
and used the embedded production shader with no override. Trees and game
content packs were not edited; twelve retired tree paths and the removed
renders directory remain absent. Overall outdoor quality remains open.


## Mt. Moon Square outdoor domain and border-water continuation — 2026-10-18

The preceding shoreline-material turn was verified progress. New Bark's fresh
waterfront angle and Route39's Moomoo Farm corral were inspected against paired
2D. New Bark's immediate edge is shallow; its distant connected-route bank was
not mistaken for a local pit. The farm's shared Tauros models and white corral
fencing retain their native sprite identities. No art changes were justified
in those two views. Evidence: `target/outside-art-new-bark-waterfront-east*`
and `target/outside-art-moomoo-corral-review*`, paired-state preserved.

Mt. Moon Square exposed a pale untextured rock enclosure and a regular grid of
false islands beyond its native bounds. Its source declares a Kanto outdoor
route site, but the named map was absent from the shared land/water domain.
It now receives the existing mountain relief, mineral grain, outdoor light and
sheltered source-water optics. MountMoon and MountMoonGiftShop retain their
indoor classification. No new mountain or building asset was created.

A complete native $2d border block repeats one shore corner among fifteen
animated water cells. Outside this isolated map's authoritative extent, those
corners now borrow the live water slot from their own block. Exact atlas,
metatile, all sixteen phases, original tile indices, complete outside placement
and a genuine animation family are required. Actual map cells, foreign atlases,
changed/cropped drawings and missing metadata retain their original treatment.
The render-only replacement has a water surface and water depth mask; the
source map, native 2D and gameplay collision remain untouched. The late terrace
backing pass also honors this proven water continuation, preventing it from
recreating walls around the retired corners or raising their support. Genuine
raised shores keep their backing. The distant apron donor selector now rejects
flat native shore corners as repeatable ground. Beyond-map water remains
presentation-only continuation of the existing native border.

The production trace identified 648 complete border corners in the reviewed
116×114 source grid, origin (-44,-39), with a real 30×18 core-tile map extent.
The first replacement still left late terrace walls; the added regression
reproduces that failure while preserving a genuine raised shore and flat water.
Diagnostic traces and interim captures were removed after the final check.
A concurrent shared Model schema addition also required carrying link-material
roles through the existing planter vertex remap; both planter occlusion controls
pass. No source export or content-pack publication was needed.

All 67 distinct focused checks pass: two border-source/depth tests, 25 terrace
checks (including the new late-wall/support regression), six apron controls,
16 water-role/palette checks, 14 water-surface checks, one named-site material
check, the external-pack four-site mountain footprint check and two planter
controls. The production client build and diff whitespace checks pass.
Final embedded-shader pairs were personally inspected at MountMoonSquare (7,9),
noon and 22h, and (17,9), noon, plus the sea-level control Route20 (37,11), noon.
All use orbit 1, zoom 0, 800×640 and preserve paired state. The rock grid is gone,
the mountain mass is articulated, garden/shop/cave source layout remains clear,
and the Seafoam coast retains its low level, continuous ocean. These are static
scene checks; no weekday Clefairy scene or whole-outdoor acceptance is claimed.

Evidence: `target/outside-art-moon-square-final-{day,night}*`,
`target/outside-art-moon-square-approach-day*`,
`target/outside-art-moon-sea-level-control*` and
`target/outside-art-moon-square-verification.txt`. The initial same-state
`target/outside-art-moon-square-review*` pair supplies before evidence. Every
capture was headless and muted; final captures have no tracing or shader
override. No tree assets, Center/Mart models, audio or content packs were edited.
The twelve retired tree paths and removed renders directory remain absent.
Overall outdoor quality review remains open.

## Route23 League facade: shared twin entrances — 2026-10-18

The Route23 approach at (9,8) exposed the League's entire repeated-story
facade painted on the ground. Its native Kanto drawing starts with $68/$7f/$69
story rows at the map boundary and ends with adjacent $53/$3a entrance bays.
The shared brick-building producer now accepts that complete drawing, retains
every native window and both door positions, and provides the existing closed
roof, masonry, recessed glass and vents. The two doors share one threshold.
There is no map-specific building asset. Both exact native doorway glyphs are
required; damaged drawings cannot fall through to a different tower model.
Ordinary single-door buildings retain their existing branch. Center and Mart
source bindings, trees, land ledges, water and content packs were not edited.

The production client build succeeds. Twenty-four targeted checks pass: sixteen
brick-building controls, the new twin-door/source-ownership regression, the
shared civic-binding control and six city-light checks. The new regression
checks every source pane, separate door lights, unchanged source/support,
complete building ownership and rejection of a damaged second door.

Paired production captures preserve state at (9,11), noon and 22h, orbit 1,
zoom 0, 800×640. Both doors and the native lit/dark window pattern are visible.
The tall building's roof is cropped in the ordinary street view. A separate
(9,8) noon capture uses the supported 90-degree diagnostic field of view to
inspect its full roof and silhouette; this is not ordinary-camera acceptance.
All captures are headless, muted and use the embedded shader. The raw red
native approach strip remains a separate unresolved visual issue.

One ordinary Up input from Route23 (9,6) and (10,6) also commits both authored
door warps into IndigoPlateauPokecenter1F, respectively (5,13) and (6,13).
Both final trace rows have matching runtime/terrain maps, no pending terrain,
no warp phase and zero fade alpha. Their arrival 3D/2D pairs preserve state.
The headless QA input copies keys into the existing production dispatcher in
process; it sends no desktop keyboard events. A focus-gated keyboard attempt
was stopped before the successful in-process checks. Evidence:
`target/outside-art-route23-{west,east}-door-walk*`.

Evidence: `target/outside-art-route23-approach-review*`,
`target/outside-art-route23-building-final-{day,night}*`,
`target/outside-art-route23-roof-wide-review*` and
`target/outside-art-double-entry-final-*.log`. Overall outdoor quality remains
open; this does not resolve the user's unlocated water-pit report.

## Shared paired Kanto approach carpet — 2026-10-18

The preceding Route23 entrance verification completed both real warp checks;
its remaining bright red source-pixel strip was then investigated. Native
Kanto blocks $46/$4a contain a central four-by-two $4f carpet flanked by
round-rock drawings. The shared outdoor entry finisher now recognizes all
32 cells of that complete pair before transferring only the eight carpet
cells to its existing woven finish. The same source pair occurs at Fuchsia's
south exit and Vermilion's waterfront approach; no town-name branch or new
asset was added. This resolves the raw-strip issue recorded above.

The existing 32×16 weave, binding, stitching and live averaged native pigment
are reused. Floor positions, normals, indices' triangle coverage, walking
support and source tiles remain authoritative. Rock/cave producers retain
their cells. Partial/altered drawings and custom-owned carpet retain fallback.
Four entry-surface tests pass, including the new complete-pair regression that
alters each of the 32 source cells in turn. The production client builds and
diff whitespace checks pass. No shader, water, land-ledge, tree, service-building,
audio or content-pack changes were made in this pass.

Final embedded-shader 3D/2D pairs were personally inspected at Route23 (9,11),
noon and 22h, orbit 1, zoom 0, and FuchsiaCity (8,33), noon, orbit 3, zoom 1.
Every 800×640 capture is headless, muted and preserves paired state. Route23's
native red/pink drawing now reads as continuous cloth by day and night;
Fuchsia verifies the same shared treatment on another native threshold.
These are material/static-scene checks, not new movement or water-pit proof.

Evidence: `target/outside-art-route23-entry-cloth-{day,night}*`,
`target/outside-art-fuchsia-entry-cloth-day*`,
`target/outside-art-cave-entry-{tests,build}.log` and
`target/outside-art-paired-entry-verification.txt`. The wider outdoor objective
and unlocated water-pit report remain open.

## Coastal pit diagnosis: terrace field crosses water — 2026-10-18

A fresh Route27 west-inlet review at (5,8), noon, orbit 1, zoom 0, 800×640,
reproduces a boxed, tall coastal wall. Its paired native frame preserves state.
This is a Surf-state static capture; the faint/invisible 3D player is a separate
unresolved presentation defect, not accepted actor fidelity.

Temporary opt-in counters established that all 136 authenticated native shore
cells in this source halo already carry positive terrace support: 69 at 24,
two each at 27, 36 and 45, and 61 at 48 render units. Standing water retains its
original datum. NewBarkTown (13,10), noon, likewise has native shoreline support
from 48 through 72. The gray walls are therefore not just the native shallow
two-source-pixel water recess. Inland terrace elevation is reaching the coast.

`terrain_ledges::collect` supplies directed edges from all resolved source maps;
`terrace_levels::Levels::build` solves and normalizes their ground field without
native water/land connectivity evidence. The terrace pass skips standing-water
height, then closes the resulting gap with shore backing. Thus level water
alone does not prevent a coastal pit. The next correction must constrain the
land field to the actual source land domains and a stable coastal datum,
independently of camera cropping, while preserving authored drops, actor/prop
support, map continuity and waterfall-owned levels.

A low-bank grading prototype passed its synthetic geometry/support checks
and 94 distinct focused controls but affected zero production banks in this
view because they were already raised. Its first production capture remained
boxed. The prototype, its temporary trace code and test additions were removed;
no shoreline geometry or material change from that attempt is retained.
Those passing synthetic checks are not evidence of a fixed production pit.
The restored production client builds successfully; whitespace checks pass.
Evidence: `target/outside-art-coastal-investigation-restored-build.log`.

Evidence: `target/outside-art-route27-west-coast-current*` and
`target/outside-art-coastal-pit-investigation.txt`. This diagnosis is progress;
the coastal defect, the user's exact reported location and the overall outdoor
objective remain unresolved. No water heights, trees, ordinary Centers/Marts,
source content or game packs were changed.

## Shared coastal land fields and low-water contact — 2026-10-18

The New Bark/Route27 inlet's extra vertical coast wall is removed. The original
shallow water recess, native shore footprint, varied mineral finish and level
water remain. The land-side shelf now grades to its ordinary low-water contact
instead of carrying remote inland terrace elevation into a retaining wall.

The renderer derives a source-complete corner field from the active map and
its resolved connections. Native land/hop collision evidence separates walking
domains; blocked mountain/tree art cannot link them. Real jump landings remain
connected across their authored barriers. Each coastal domain uses its own
directed ledges. Shared corners stay continuous except at actual jump cuts,
and fixed low-water contacts constrain the added land elevation. This changes
presentation support, never movement permissions or source 2D content.

`VisualTerraceDatum` carries the same calibrated field through terrain fitting,
rigid building roots, foliage roots, animated ground details and production
actor footing. It is independent of camera cropping and active-map coordinate
translation. Map-identity retention and a bounded complete-input cache avoid
recalculating it for ordinary camera/animation changes. The optional worker
boundary transfers corner data in bounded typed pages, preserving float bits
without making the full raster an inline metadata header. Kernel identity
continues to guard worker compatibility. Legacy frames can omit the field.

Johto's closed bays/lakes retain their fixed low-water datum. The existing
Kanto enclosed-pond producer keeps its independent level and bank grading;
waterfall-connected pools keep their authored ownership. Forest and interior
maps retain their legacy treatment. The terrace pass leaves standing-water
geometry and support unchanged. No shoreline palette, bank asset, tree asset,
Center/Mart facade or roof, audio, content pack or gameplay dispatcher was
edited. Foliage only follows the corrected common ground field.

All 158 distinct focused checks pass: 131 voxel/worker checks and all 27 render
API checks. They include the remote-island failure, blocked dry connections,
real hop links, source-overlap priority, stable crop/map-origin sampling,
enclosed-pond and waterfall guards, exact water preservation, production actor
support, cache invalidation and a 512×256 corner-field paged round trip. The
production renderer and authored-asset auditor build successfully. A fresh
external-pack audit passes CinnabarIsland, NewBarkTown, Route20, Route22,
Route26, Route27 and SaffronCity with zero mesher/jump errors and 93 actual
authored jump cases. Route22 keeps the existing pond branch; Saffron's support
range remains exactly zero. This is geometry/input-evidence verification,
not a new claim that the gameplay clients were certified end to end.

Final embedded-shader 3D/2D pairs were personally inspected at Route27 (5,8),
noon and 22h, orbit 1, zoom 0; NewBarkTown (13,9), noon, orbit -3, zoom 0; and
Route20 (37,11), noon, orbit 1, zoom 0. All are 800×640, headless, muted and
state-preserving, with no camera/FOV/shader override. The bay's extra wall is
gone, its water stays level, New Bark's channel has the same low contact, and
Seafoam retains its low island/continuous sea. The faint Surf actor in the
Route27 review remains a separate unresolved presentation issue; these static
captures do not establish actor fidelity or whole-outdoor acceptance.

Evidence: `target/outside-art-route27-coast-final-{day,night}*`,
`target/outside-art-newbark-coast-final-east*`,
`target/outside-art-coast-seafoam-final-control*`,
`target/outside-art-terrace-final-tests.log`,
`target/outside-art-datum-final-*.log`,
`target/outside-art-terrace-api-all.log`,
`target/outside-art-terrace-pack-final.log` and
`target/outside-art-coastal-field-verification.txt`. The same-position
`target/outside-art-route27-west-coast-current*` pair supplies before evidence.
Rejected field prototypes, temporary traces and generated audit JSON were
removed. The twelve retired tree paths and removed renders directory remain
absent. Overall outdoor quality review remains open.

## Offshore boundary stone finish — 2026-10-18

The Cinnabar coastal review showed level water, but its shared offshore rocks
read as white capped posts. The Kanto sea-rock model now retains its authored
sloping shoulders instead of receiving the land/path post's flattened crown.
Its existing native pigment blends continuously from a darker wet foot into
a muted mineral highlight. The existing rooted-water paint domain adds broad
weathering and pixel-filtered fine pores in stable world coordinates. Shared
closed topology, seated contacts, placement variants and source footprints
remain intact; land/path boundary styles retain their prior crown treatment.
No new assets, map-specific copies, tree edits, source collision changes,
water-height changes, pack exports or service-building changes were made.

Thirty boundary checks, one shared variant topology/footprint check and all
fourteen water optics checks pass. The production native renderer rebuilt with
its embedded shader. Personally inspected same-state 3D/2D pairs cover
CinnabarIsland `(12,12)` at noon and 22h and Route20/Seafoam `(37,11)` at noon,
orbit 1, zoom 0, 800×640. All pairs preserve state; captures used the headless
image target and muted audio, without shader/camera/FOV overrides. The offshore
stones read as weathered rocks, and the low island coast and level sea remain.
The existing Cinnabar mountain and Center are outside this finish's scope.

Evidence: `target/outside-art-cinnabar-coast-field-review*` before;
`target/outside-art-cinnabar-sea-stone-{day,night}*` and
`target/outside-art-seafoam-sea-stone-day*` after;
`target/outside-art-sea-stone-{tests,variants,water,build}.log` and
`target/outside-art-sea-stone-verification.txt`. The twelve retired tree paths
and removed renders directory remain absent. Overall outdoor acceptance remains
open.


## Shared Kanto office slate material — 2026-10-18

The Saffron `(18,7)` production street review showed large, uniformly gray
roof grids. Its existing diagonal stone flags were receiving ceramic glaze.
Shared Kanto brick-office roof decks and closed diagonal flags now use a
separate matte mineral-slate material, with restrained cleft/grain detail,
nonperiodic per-slate value variation and narrower physical joints. Native
roof outlines, story counts, coping, vents, windows, entrances and ground
support retain their prior construction. Ceramic roofs and service-building
styles keep their existing material domains. No new asset or per-city copy
was introduced, and no source collision, trees, water or terrain heights changed.

A first night review exposed a new material-tag overlap with legacy city
emitters; that rejected capture was replaced after moving slate into a separate
reserved band. A regression now excludes all legacy window/cream/neon roles,
source-window tags, glass and the original four town-paint domains. All 24
focused checks pass: sixteen source/building/topology checks, one domain check,
one civic-asset selection check and six city-light checks. The production
native client rebuilt successfully. Scoped diff whitespace checks pass.

Final same-state 3D/2D pairs were personally inspected at SaffronCity `(18,7)`,
noon and 22h, and CeladonCity `(16,2)`, noon, orbit 1, zoom 0, 800×640. All
preserve state. Captures used the headless image target, muted audio and the
embedded production shader without camera/FOV/shader overrides. Final night
roofs remain dark and textured while their original windows emit. The Celadon
view verifies the same finish on its taller, partially occluded office form.
The pale Saffron street surface, overall building scale and nearby foreground
NPC occlusion still need broader scene review; this is roof-material acceptance.

Evidence: `target/outside-art-saffron-street-review*` before;
`target/outside-art-saffron-slate-roof-{day,night}*` and
`target/outside-art-celadon-slate-roof-day*` after;
`target/outside-art-slate-roof-{tests,domain,civic,lights,build}.log` and
`target/outside-art-slate-roof-verification.txt`. The twelve retired tree paths
and removed renders directory remain absent. Overall outdoor acceptance remains
open.


## Shared blank-road mineral aggregate — 2026-10-18

Saffron's `(18,7)` street review showed an excessively pale native road field
beside its modeled curbs. The complete native `$23` pavement already had a
continuous plane; it now has a separate shared mineral-road finish with a
lower source-paint gain, restrained wear and fine neutral grain sampled from
the existing painted-bedrock resource. Map-registered, gradient-filtered detail
stays continuous across source cells and camera crops. A coarse procedural
aggregate prototype read as frosted patches and was rejected; its captures
were overwritten after reducing broad mottling and using finer mineral detail.
No joints or brick bonds were invented in the blank source field. Actual sand,
brick paths/decks, curb strips, circular covers and buildings retain their
existing finish. No geometry, collision, standing heights, water, trees, audio,
source game content, new assets or per-map copies changed.

Fifteen Kanto paving checks and four elevated-pier checks pass. Coverage includes
full blank-field area at the original zero datum, material registration through
translation/crops, native palette changes, malformed/claimed/elevated controls,
and actual external-pack pier support. The external test used the absolute
completed `content-packs/text-tui.crystalpack` path. The initial Saffron source
lookup found one map and zero mesher errors; it was a baseline source lookup,
not changed-material acceptance, and its generated audit JSON was removed.

The production native renderer rebuilt with the embedded shader. Final
same-state 3D/2D pairs were personally inspected at SaffronCity `(18,7)`, noon
and 22h, and Route17 `(9,45)`, noon, orbit 1, zoom 0, 800×640. Every pair
preserves state. The final roads read as muted mineral pavement by day and
retain their dark source-night value. Saffron remains flat; Cycling Road's
continuous road/curbs and level canals retain their prior geometry. Captures
were headless image targets and muted, without shader/camera/FOV overrides.
Scoped diff whitespace checks pass. Broader building scale and nearby foreground
NPC occlusion remain open; this is road-material acceptance only.

Evidence: `target/outside-art-saffron-slate-roof-day*` before;
`target/outside-art-saffron-road-aggregate-{day,night}*` and
`target/outside-art-route17-road-aggregate-day*` after;
`target/outside-art-road-aggregate-{tests,piers,build}.log` and
`target/outside-art-road-aggregate-verification.txt`. The twelve retired tree
paths and removed renders directory remain absent. Overall outdoor acceptance
remains open.


## Nearby outdoor NPC building sightlines — 2026-10-18

Saffron's `(18,7)` street review showed only the nearby NPC's head above the
foreground office roof. Outdoor building reveal now considers up to two close,
source-backed human object actors as well as the player. It tests their actual
body sightlines against the existing original triangle surfaces, using the
rendered rig's visible stature and calibrated standing support. Interest is
bounded to 2.25 gameplay tiles in three dimensions and follows moving actors
and support changes independently of the player's camera/body cache. A nearby
actor on a raised land shelf remains eligible. Swimmer targets use the visible
head-and-shoulders height above the waterline, matching the existing pose.
Props, effects, remote players, distant actors and missing support do not add
interest. Only groups already using outdoor building reveal receive these
additional targets; room walls, foliage and forest maps keep their prior rules.

No meshes, opacity constants, shaders, assets, building styles, water, terrain,
collision, audio, packs or gameplay ownership changed. Existing fade settling,
restoration, linked captions, shadows and retained building form remain shared.
No old pixel card or duplicate character body is drawn over a modeled actor.

All 43 focused checks pass: the complete 42-test occluder-fade suite, including
two new NPC cases, plus the existing swimmer-pose test extended to the sightline
height. Tests prove a real roof can miss every player ray yet block the nearby
NPC, enforce bounded nearest-character selection, reject unrelated source
roles and support, and track movement/height while preserving room/forest
exclusions. The production native client rebuilt successfully; scoped diff
whitespace checks pass.

Final same-state 3D/2D pairs were personally inspected at SaffronCity `(18,7)`,
noon and 22h, and FuchsiaCity `(20,14)`, noon, orbit 1, zoom 0, 800×640. Every
pair preserves state. Saffron's NPC is visible from head to foot while the
foreground office retains its faint form. Fuchsia's clear actors and opaque
building remain intact. Captures were headless image targets and muted, using
the embedded shader without camera/FOV/shader overrides. This verifies the
nearby building sightline correction; broader scene scale, actor appearance
fidelity and whole outdoor acceptance remain open.

Evidence: `target/outside-art-saffron-road-aggregate-{day,night}*` before;
`target/outside-art-saffron-nearby-npc-{day,night}*` and
`target/outside-art-fuchsia-nearby-npc-control*` after;
`target/outside-art-nearby-npc-{tests,fade-tests,swimmer-tests,build}.log` and
`target/outside-art-nearby-npc-verification.txt`. The twelve retired tree paths
and removed renders directory remain absent. No task-generated audit manifest
or exported game data was retained.


## Shared Sprout Tower pagoda roof — 2026-10-18

Violet City's `(23,7)` approach showed three broad gabled awnings on the tower.
The single shared Sprout Tower model now has three swept, four-sided hip roofs,
staggered closed ceramic courses, physical overlapping lips, curved timber
fascia and exposed corner brackets. The source drawing still owns its live
gold/day and dark-purple/night roof paint. Initial long aligned tile strips
were rejected after the normal-camera review; staggered joints and course
steps now read as ceramic rows. Existing authored-course metadata bypasses the
legacy slab refinement, preserving the new closed geometry in the renderer.
This is a source-pattern/model-type change, with no per-town duplicate.

The original authoring script replayed the current shipped tower exactly.
Its original fit was retained before replacing only the roof parts. Bounds
remain `[-2,0,-4]..[2,6.39,4]`, and the door anchor remains `[0,0,4]`. The complete
stone, light stone, plaster, light plaster, glass, door-detail and finial-detail
primitive groups compare byte-for-byte with that baseline. Walls, windows,
steps and doorway keep their prior placement. No water, ledges, trees, actor
appearance, service buildings, audio, packs or controller behavior changed.
The authoring loader now stops at the explicit vocabulary boundary rather than
accidentally running unrelated New Bark builders during a selected export.

All 33 focused Rust checks pass: 23 shared-model checks, four legacy-roof
adapter checks and six city-light checks. The legacy adapter fixture now
contains only its three actual legacy models; authored Gym/pagoda courses have
separate raw-geometry preservation checks. The new tower regression covers
fixed bounds/threshold, upward corners, all four roof slopes on each tier,
unchanged authored ceramic vertices and bounded runtime geometry. All 591 new
roof/support parts have closed positive-volume shells and no zero-area
triangles. They contribute 10,644 triangles; the complete authored tower has
12,308 triangles within its selected 13,000-triangle export limit.

The production native renderer rebuilt successfully. Final paired 3D/2D
views were personally inspected in VioletCity at `(23,7)`, noon, orbit 1, and
`(22,7)`, noon and 22h, orbit 2, all zoom 0 and 800×640. Every pair preserves
state. The approach verifies the entrance and lower roof hierarchy; the side
views expose all three tiled hips/tiers, with only the unchanged finial tip
partly clipped by the normal frame. Captures were headless image targets and
muted, using the embedded shader without FOV/camera/shader overrides. Wider
path framing and early roof prototypes were discarded. Scoped whitespace
checks pass. Overall outdoor acceptance remains open.

The existing `art/johto/source/connected-johto.blend` now contains the updated
685-object tower collection. Geometry/material/transform snapshots preserve
all 2,145 unrelated source objects. Only its existing manifest identity changed,
and its compressed/expanded hashes verify. The general source-store helper
refused publication because `gym-scenery-kit.blend` already mismatched its
manifest before this work. A scoped replacement updated only the connected
scene and its own identity, preserving every unrelated manifest record and the
gym source bytes. Sixty-six source identities verify; the standard global
`python3 tools/johto_art_sources.py --check` still reports that pre-existing gym
identity mismatch. No validator was weakened or unrelated identity refreshed.

Evidence: `target/outside-art-violet-tower-approach-review*` before;
`target/outside-art-violet-pagoda-approach-day*` and
`target/outside-art-violet-pagoda-side-{day,night}*` after;
`target/outside-art-sprout-{model-tests,roof-tests,lights-tests,build}.log`,
`target/outside-art-sprout-{closed-parts,source-merge,source-check,canonical-source}.log`
and `target/outside-art-sprout-{source-verification,verification}.txt`.
Task-owned authoring directories, temporary GLBs, baseline model copies,
working Blender backups, merge script and generated statistics were removed.
The twelve retired tree paths and removed renders directory remain absent.


### Enclosed Johto lake level — 2026-10-18

Blackthorn Gym's enclosed lake was treated as open sea while the surrounding
terraces rose above it. This produced a deep artificial water recess and
stretched bank caps. The complete native source field now distinguishes closed
Johto lakes from open water and assigns each lake its lowest surrounding terrace
level. Water vertices, ripple accents, shoreline contacts and Surf support share
that constant level. The complete body determines the level before camera
cropping; a partial view cannot invent a different rim. Open oceans remain at
sea level. Waterfall pools and the existing Kanto enclosed-pond producer retain
separate ownership. This changes shared terrain fitting, without adding town
asset variants or altering collision, native jumps, forests or building models.

Production headless paired captures were inspected at BlackthornCity `(20,12)`,
orbit 1, zoom 0, 800×640, at hours 12 and 22:
`target/outside-art-blackthorn-lake-level-{day,night}{,-2d}.png`.
The lake now meets the Gym island's rocky bank; its surrounding native mountain
walls remain. The NewBarkTown `(13,9)` sea-level control uses orbit -3 at hour 12,
with `target/outside-art-new-bark-lake-control{,-2d}.png`.
All paired states are preserved and no windows were opened.

Validation: 105 distinct Rust checks across terrace, shore and water-surface
suites pass. Read-only production pack audits retain 47 native Blackthorn jumps,
12 on Route27, and 3 on CinnabarIsland; NewBarkTown remains at zero land support
height. The scalar receipt is `target/outside-art-lake-level-verification.txt`;
generated audit JSON and the rejected blocked-position capture were removed.
These checks establish this water-level correction, not acceptance of every
outdoor scene.


### Steep shoreline-cap mineral finish — 2026-10-18

The corrected Blackthorn lake still exposed stretched grass and pale path paint
on steep native bank caps beneath its cliffs. Authenticated bank caps now retain
separate grass/earth material roles through terrace fitting. Gentle caps keep
the previous floor finish. Steep caps blend into muted earth-brown mineral paint, using the
existing stone texture projected across three axes rather than stretched along
the source floor. Ordinary grass/path floors, the surrounding cliff models,
rocky edge geometry, water levels and building materials are unchanged. Forest
maps and forest-source frames retain their previous bank material tags.

The production renderer was checked headlessly at BlackthornCity `(20,12)`,
12h and 22h, orbit 1, zoom 0, 800×640, with matching native 2D state:
`target/outside-art-blackthorn-bank-earth-{day,night}{,-2d}.png`.
The pale and green strips now read as exposed stone continuous with the cliffs;
the Gym's approach and the lake's corrected level remain intact. NewBarkTown's
coastline control is `(13,9)`, 12h, orbit -3, zoom 0:
`target/outside-art-new-bark-bank-earth{,-2d}.png`.

The bank ownership regression preserves positions, normals, indices, footing,
other assets' UVs/colors, and forest-map tagging. It and the 37 shoreline checks
pass (38 distinct checks). The shader compiled in the production captures.
No new texture assets, town variants, source packs or visible windows were
introduced. This material correction does not establish acceptance of every
outdoor scene.


### Shared Kanto flat-hall proportions — 2026-10-18

Cerulean's compact doorless halls had a wall height of 4.60 source units and
narrow glass nearly three times as tall as its width. The shared authenticated
`kanto_flat_house` producer now uses the cottage's 3.08-unit single-story eave,
with a lower closed roof assembly. At unit 8 the maximum height changes from
42.656 to 30.496. Source panes keep their original columns and native lit/dark
identity, with an aspect near 1.275 and a slender horizontal meeting rail.
Roof pans, coping, vent, native pigment and footprint stay intact. Source
recognition, service buildings, collision, water and forest assets are unchanged.
No town-specific model or new asset file was introduced.

Three focused regressions pass across widths 8–32, verifying complete source
recognition, opacity rejection, pane proportions/locations, native lamps,
material translation stability, bounded geometry and day/night geometry
identity. The shared-producer regression covers Cerulean, Vermilion and an
otherwise unnamed town. Read-only pack audits before/after preserve the same
family counts, coverage, source bounds, native pane identities, footing range
and actual jump cases: Cerulean has 192 hall cells, 19 panes and 11 jumps;
Vermilion has 32 hall cells, 3 panes and no jumps. Temporary audit JSON was
removed; the receipt is `target/outside-art-flat-hall-verification.txt`.

Production offscreen native 2D/3D pairs, all personally inspected with preserved
state, normal camera settings and 800×640 image targets:

- Cerulean `(23,18)`, orbit 1, zoom 0, 12h:
  `target/outside-art-cerulean-hall-road-{before,day}{,-2d}.png`.
- Cerulean `(16,17)`, orbit 1, zoom 0, 12h/22h:
  `target/outside-art-cerulean-hall-close-{day,night}{,-2d}.png`.
- Vermilion `(19,31)`, orbit 1, zoom 0, 12h:
  `target/outside-art-vermilion-hall-control{,-2d}.png`.

The close Cerulean view confirms that the compact hall now sits at cottage
scale, its square sash bays retain native night lighting, and its roof profile
stays distinct. Initial exploratory positions were removed from acceptance.
All captures were muted and headless, with no opened windows.

The Vermilion control exposed a separate unresolved scene defect: water-surrounded
barriers sit on square grass platforms absent from native 2D. The current
boundary adapter chooses a family-specific remote floor sample, and its land
family supplies lawn. Before changing that path, trace the exact owners, source
metatile phases and live atlas backgrounds at this harbor. Preserve the native
barrier locations, neighboring actual land, standing-water datum and source
fallback guards. The image is evidence of the defect, not acceptance of the
harbor. The full outdoor goal remains incomplete.


### Harbor barrier water backing — 2026-10-18

The Vermilion defect recorded above is corrected in the shared boundary
adapter. The same native barrier glyphs occur on dry land, so their metatile
IDs cannot globally select water. Complete coherent barrier components now
use surrounding native water/ground evidence to resolve their backing. An
immutable, validated source-coordinate mask travels with the terrace datum,
cache and bounded worker pages; moving the camera cannot change the decision.
No map allowlist or town-specific model was added.

At Vermilion, 120 source cells (30 barriers) use the existing shore-rock model
and water underlay instead of raised lawn platforms. The entry mat, path,
actual island, native positions and standing-water datum are retained. Land
barrier coverage changes from 264 to 144 cells and shore coverage from 292 to
412; total modeled coverage remains 2194. Borrowed water beds prefer an
authenticated animated sample with inset texels.

The verification receipt is `target/outside-art-harbor-domain-verification.txt`.
There are 120 distinct passing checks, including complete/partial source
recognition, dry reuse across actual Kanto maps, shifted-camera mesh identity,
mask validation, worker page roundtrip and mask-only cache invalidation.
PewterCity, CinnabarIsland and Route17 preserve exact family counts, mesh
counts, footing ranges and jump samples against the pre-change audits.
Temporary source/audit JSON and rejected prototype captures were removed.
All 12 retired tree assets remain absent from both asset roots.

Production offscreen pairs at 800×640, orbit 1 and zoom 0:

- VermilionCity `(19,31)`, 12h and 22h:
  `target/outside-art-vermilion-harbor-final-{day,night}{,-2d}.png`.
- PewterCity `(2,18)`, 12h, dry reuse control:
  `target/outside-art-harbor-pewter-control{,-2d}.png`.
- CinnabarIsland `(12,12)`, 12h, existing coastline control:
  `target/outside-art-harbor-cinnabar-control{,-2d}.png`.

The paired-state receipts preserve native state. These checks address the
incorrect backing of harbor barriers. Dense rock groups still show some
rectilinear shallow-water bands in daylight; this is not acceptance of all
harbor optics or the broader outdoor goal. Captures remain muted and headless.


### Mineral plateau ponds and source-owned cliff contacts — 2026-10-18

Cinnabar's native pond sat below its surrounding mineral plateau. Decorative
relief also grew across Kanto's walkable plateau caps and folded cliff courses.
That pass initially removed Kanto mountain relief and lowered Johto ridges.
The user rejected that flattening; the mountain restoration below supersedes
those geometry and pigment changes. The source-owned pond calibration below
remains in place.

The new shared `mountain_ponds` source pass recognizes complete mineral-enclosed
ponds and their authenticated narrow banks. A complete mixed barrier drawing
can continue the adjacent mineral apron when it belongs to such a pond.
Isolated stone paths retain their original level. The pond and bank receive one
level at the lowest surrounding plateau contact. Open water stays at sea level;
unknown, altered and incomplete boundaries do not authorize this calibration.
The existing immutable terrace snapshot carries the result through caching and
worker transport, independently of the camera crop. No town allowlist or new
model/texture variant was introduced.

The final contact corrections preserve the original geometry's source owner:

- Mineral sampling decodes the existing volcanic UV band and selects the
  original rock-side cell. Folded courses recover their authored grid plane
  before choosing that side; their narrow UV thickness cannot select water.
- A resolved pond's natural bank receives one offset across each sculpted
  surface. A mesh trace found neighboring vertices receiving offsets 16 units
  apart, which stretched the bank into needles. Uniform bank translation
  preserves its source palette, footprint and varied rock detail.
- The shallow shore backing includes a neighboring mineral plateau's native
  base in addition to its calibrated terrace. This closes the front contact
  without a tall retaining wall or a water-height transition.
- The ordinary pond fallback skips already resolved water, preventing a second
  lift of Surf support. Existing jump-contact compensation remains in place.

An initial apron-only fallback was rejected after the control audit found it
flattened Pewter's unresolved terrain field. Aprons are now committed only for
a proven closed pond, and unresolved nonempty ledge fields retain their original
fallback. The final nine-map controls preserve model families, source coverage,
footing ranges and 169 native jump takeoff/landing cases without audit errors:
Blackthorn, Cerulean, Cinnabar, Pewter, Route15, Route23, Route26, Saffron and
Vermilion. The final production-build Cinnabar/Pewter recheck also reports no
errors, with modeled coverage unchanged at 825/2186 cells.

There are 141 distinct passing checks across terrace/source calibration,
external-pack footprints, mineral geometry, boundary recognition, water masks
and pond support. Regressions cover all 42 native volcanic-pond water cells,
unchanged ocean levels, dry/altered/cropped apron rejection, protruding caps,
folded wall ownership, uniform sculpted-bank translation and no double support
lift. Receipt: `target/outside-art-mineral-pond-verification.txt`.
All 12 retired tree assets remain absent from both asset roots. This pass adds
no source pack, audio data, service-building asset or forest redesign.


Production captures use muted headless image targets, normal camera settings,
orbit 1, zoom 0 and 800×640 output. Matching native 2D/3D states are preserved:

- Cinnabar `(15,6)`, noon, the native walkable mineral shelf beside the pond:
  `target/outside-art-cinnabar-pond-shore-day{,-2d}.png`.
- Cinnabar `(12,12)`, noon, overview and unchanged Center control:
  `target/outside-art-cinnabar-pond-finished-day{,-2d}.png`.
- Pewter `(2,18)`, noon, dry boundary/floor control:
  `target/outside-art-mineral-pond-pewter-final{,-2d}.png`.
- Blackthorn `(20,12)`, 22h, retained lake and broader ridge control:
  `target/outside-art-blackthorn-ridge-final-night{,-2d}.png`.
- Route26 `(9,28)`, noon, native walkable mineral shelf and ridge control:
  `target/outside-art-route26-ridge-final-day{,-2d}.png`.

The close Cinnabar pair confirms a level native pond meeting its walking shelf,
with a complete rocky lip and no stretched bank or cliff needles. The overview
retains the existing Center and open ocean. No visible windows were opened.
Temporary source/audit JSON, mesh traces and rejected prototype captures were
removed after recording scalar results. This is acceptance of the corrected
pond/contact behavior, not the complete outdoor target.

The close Cinnabar view also exposes a separate offshore yellow panel and pale
rectangular outlines to the northeast. Their source/connection ownership still
needs investigation before any geometry change. Overall outdoor acceptance
remains open.

### Water pit final check — 2026-10-18

The production Cinnabar `(15,6)` noon pair confirms the mineral pond meets
its shelf without a deep pit. Open ocean remains at sea level. Captures
`target/outside-art-cinnabar-pond-shore-day{,-2d}.png` use muted headless image targets;
the paired receipt confirms `state_preserved=true`.

A separate experiment generalized connected bridge ownership. Although six
checks and a five-map audit passed, the Celadon production control exposed an
unwanted pale edge. The experiment was rejected and removed; the verified pond
correction remains. Temporary source/audit JSON and source-usage traces were
removed. No asset, content pack or forest changes were needed. The offshore
panel and outlines remain unresolved; this is not full outdoor acceptance.

### Native-map gaps cannot lift the ocean — 2026-10-18

The large pale offshore rectangles were gaps in the rendered water, not a
shoreline pigment defect. A diagnostic surface isolation and camera ray trace
located four affected Cinnabar halo cells at `(40,-7)`, `(41,-7)`, `(42,-2)` and
`(42,-1)`. The complete native-map rectangle contained no source samples there,
but nearest-land domain propagation still supplied varying elevations. Water
in the repeated camera halo inherited support offsets of 12/24/24/12 at the
production scale, opening gaps between neighboring water tiles.

The shared datum builder now leaves unauthored gaps at zero. Standing water
also refuses a clamped land-edge sample outside the native rectangle. Ordinary
land continuation retains its previous edge behavior; complete elevated ponds
still consume their authenticated level. All four traced problem points now
have zero support offset and intersect the same ocean surface at `y=-8`.
No shoreline texture, model, source pack or forest asset changed.

The production build passes 61 terrace, datum and pond checks. New regressions
cover gaps between native maps, all sides of the repeated outer halo, retained
raised-pond support and geometry, unchanged UVs/topology and existing land-edge
sampling. Nine native-map audits report zero mesh errors across 135 jump
takeoff/landing samples. Saffron remains flat. Scalar receipt:
`target/outside-art-ocean-halo-verification.txt`.

Final 800×640 muted headless image captures were personally inspected against
native 2D with `state_preserved=true`, normal camera settings, orbit 1, zoom 0:

- Cinnabar `(15,6)`, noon: `target/outside-art-ocean-halo-final{,-2d}.png`.
- Route20, runtime-selected walkable spawn, noon:
  `target/outside-art-ocean-route20-final{,-2d}.png`.
- Blackthorn `(20,12)`, 22h: `target/outside-art-ocean-lake-night-final{,-2d}.png`.

Cinnabar's large offshore gaps are gone, and its pond/rocky lip remain intact.
Blackthorn retains its night lake and native ridge. Route20 still shows a
smaller bright plot corner beneath a shoreline rock, a distinct unresolved
contact defect. The flat offshore deck panel also remains. Temporary surface
probes, source traces, raw audit JSON and diagnostic captures were removed.
All 12 retired tree assets remain absent. Full outdoor acceptance remains open.

### Shared outdoor cliff finish — 2026-10-18

Review correction: the user subsequently rejected using the Cinnabar cliff
result as the direction for mountains throughout the app. The mountain
restoration below supersedes that part of this intermediate review. Preserve
mountain masses and their own pigment; do not flatten them into shore ledges.

The user rejected the shoreline-only ledge treatment. Town and route shore
faces now use the shared fractured cliff generator and the same mineral
shader projection/scale as inland cliffs. Separate shoreline pebble crowns,
the raised-pond backing crown and shoreline-specific texture overrides were
removed. Source ownership tags still serve terrain shaping, then resolve to
the shared material instead of being cleared to an untextured face.

A cliff's dry cap continues its actual adjacent ground. Applying cliff paint
to every horizontal bank cell made a separate paved-looking band, which was
rejected during production visual review. The final material pass inherits
the completed neighboring floor's texture coordinates and pigment at the
same elevation. It accepts verified native ground/mineral floors, excludes
buildings and unrelated higher shelves, and caches the choice per source
cell so shared triangles cannot acquire different materials. It changes no
positions, normals, indices, source claims, collision or walking support.

Recessed modeled sea-rock beds also close the exposed underside of adjacent
flat ground. Route20's flat-map policy remains authoritative even when an
imported neighbor has elevation. Only the missing mineral support face is
added; neither a new rim nor a water-height transition is introduced.

Forest presentation is explicitly retained through its existing unpainted
finish. No town-specific assets or new terrain texture files are introduced.
The old tests requiring a pebble crown are replaced by checks for shared
cliff relief below the unchanged cap, alongside all existing contact,
water, source-ownership and footing assertions.

Faces also inherit the common cliff pigment and lighting, including at night;
verified mineral floors retain their native stone pigment. There is no separate
shoreline color treatment.

The production build and 110 distinct terrain/material regressions pass. The
four final muted headless 800×640 production/native-2D pairs were personally
inspected with `state_preserved=true`: Blackthorn `(20,12)` at 22h, Cinnabar
`(15,6)` at noon, Route20's runtime-selected walkable spawn at noon, and
Route26 `(9,28)` at noon. All use orbit 1, zoom 0 and the normal camera.
The shoreline-only cap band is gone; Blackthorn's bank now shares the night
cliff finish, and Route20's exposed rock-side corners are closed.

The nine-map control audit reports zero mesh errors across 132 jump samples.
A final four-map recheck of Cinnabar, Route20, Ecruteak and Saffron also reports
zero errors, retaining coverage and footing ranges. Scalar receipt and final
image pairs: `target/outside-art-unified-cliff-verification.txt` and
`target/outside-art-unified-cliff-*{,-2d}.png`. Temporary probes, source traces,
raw audit JSON and superseded captures were removed. All 12 retired tree
assets remain absent. The offshore deck panel remains a separate defect;
full outdoor acceptance remains open.

### Connected bridges retain their shared structure — 2026-10-18

The yellow offshore panel visible from Cinnabar was a connected Route21 deck
whose pier renderer depended on the host map name. The exact native Kanto
`05/4b` deck now uses the existing shared deck, frame, pilings and support
height across that connection. The native pack confirms all sixteen cells
of this metatile are the same deck drawing. Ambiguous `78/31` edge art is
recognized outside its original host only beside an actual authenticated
deck; standalone city-edge and level classifications remain unchanged.
This avoids the earlier rejected global edge-classification experiment.

The build and 34 distinct bridge, paving and datum checks pass. The new
external-pack regression compares complete Route21 pier geometry, ownership,
water and footing under Route21 and Cinnabar hosts and rejects isolated city
edges. Five native-map audits report zero mesh errors across 40 jump samples:
Celadon, Cinnabar, Pewter, Route20 and Route21.

Four muted headless production/native-2D pairs were personally inspected at
noon, orbit 1 and 800×640 with normal camera settings and preserved state:
Cinnabar `(15,6)` and Celadon `(16,2)` at zoom 0; Route20 `(13,9)` and Route21
`(16,22)` at zoom 1. A fresh Celadon before capture confirms no new pale edge;
Route21's actors stand on the deck, and native 2D confirms its brick drawing.
The distant Cinnabar panel now shows the same suspended structure. Ocean
levels, the island pond, the shared cliffs and city buildings are retained.

Evidence: `target/outside-art-connected-deck-{cinnabar,celadon,route20,route21}*`
and `target/outside-art-connected-deck-verification.txt`. Raw audit JSON was
removed after recording scalar results. No assets, packs, forests or service
buildings were changed. All twelve retired tree paths remain absent. The
offshore panel's missing structure is resolved; its strong source-yellow
paint and the repetitive coastal boulder rows remain broader visual-review
candidates. Full outdoor acceptance remains open.

### Mineral bridge finish and tall-grass bank contact — 2026-10-18

The shared Kanto/Johto brick deck now uses the existing live mineral pigment
instead of reproducing saturated gold/purple source paint with white mortar.
Recessed joints derive their exposure from the brick pigment. The three-course
bond, bevels, frame, pilings, source authentication and walking support are
retained. City paths, granular sand and tournament paving keep their existing
material branches. This introduces no assets or map-specific model variants.

The Route32 control exposed a separate sky-colored gap at the north landing.
The shore-contact recognizer expected glyph `03` for Johto tall-grass block
`03`; the native pack proves its sixteen cells actually use glyph `04`.
The ignored bank stood at render height 48 beside a deck fixed at 16. Correcting
that shared identity allows the existing sixteen-source-cell approach to meet
the bank. No bank, cliff or water datum was changed. The regression checks
native art, the joined edge, continuous grade, actor support, unchanged water,
rejection of the wrong glyph and protection against double terrain lifting.

The build and 21 distinct bridge/paving regressions pass. Five muted headless
800×640 production/native-2D pairs were personally inspected with preserved
state, orbit 1 and normal camera settings: Route21 `(16,22)` at noon and 22h,
zoom 1; Route20 `(37,11)` at noon, zoom 0; Route32 `(8,48)` at noon and 22h,
zoom 1. The Seafoam sand remains granular, the actual cave approach remains
brick, and Route32's bank join is closed in both lighting conditions. Evidence:
`target/outside-art-bridge-mineral-{day,night,sand-control,johto-final-day,johto-final-night}*`
and `target/outside-art-bridge-mineral-verification.txt`.

Final Route26/27/32 audits report zero mesh errors across 68 native jump
takeoff/landing samples, retaining modeled coverage at 5164/3412/4602 cells.
Temporary contact probes, raw audit JSON and rejected diagnostic captures have
been removed after recording scalar results. The user is directing tree and
forest style with another agent; this pass makes no tree or forest edits.
Full outdoor acceptance remains open.

### Shared painted timber posts — 2026-10-18

The Fuchsia east-street review at `(30,26)` showed swollen mint-colored caps
on narrow shafts. The existing shared Kanto capped-post asset now has a flat
cut crown, a small bevelled shoulder and a sturdier timber shaft. Its original
576-triangle closed topology, native fitting bounds and height are retained:
`[-0.28205,0,-0.28205]` to `[0.28205,1.212,0.28205]`. The cap is less than
1.2 times the shaft radius instead of a large dome. There are no town-specific
models, added rails, changed post gaps or placement/support changes.

Live native pigment now gives the shaft a weathered warm timber finish and
the cap a more neutral painted white. The existing model-local wood grain and
paint finish remain stationary through camera movement and RTC palette changes.
The authoring script, single runtime mesh, dedicated editable Blender source
and that source's manifest entry were updated together. Only the capped-post
record was replaced; unrelated source records and forest assets were retained.

All ten capped-post regressions pass, including closed edges, positive volume,
winding, flat crown normals, crisp shaft facets, unchanged fitting envelope,
four source blocks/fourteen courses, open course gaps, support and source
immutability, live day/night paint and unknown/cropped-art rejection. The
Blender rebuild reports one closed component, eight materials, 576 triangles
and volume `0.2289288821`. The owned source hashes validate. The global
`tools/johto_art_sources.py --check` still fails on the pre-existing unrelated
`gym-scenery-kit.blend` identity mismatch; that source was not changed.

Three muted headless 800×640 production/native-2D pairs were personally
inspected with preserved state, orbit 1, zoom 0 and normal camera settings:
Fuchsia `(30,26)` at noon and 22h, and Pallet `(10,10)` at noon. The caps read
as cut painted timber rather than rounded ornaments, retain night readability,
and keep their native spacing in both locations. Fuchsia/Pallet/Route13 audits
report zero mesh errors across 32 native jump samples, consuming 200/32/800
source cells with the same shared post family. These are post-specific visual
checks. Parallel tree edits produced black canopies in these captures; tree
assets remain with the user's other agent and are outside this acceptance.

Evidence: `target/outside-art-timber-post-{fuchsia-day,fuchsia-night,pallet-day}*`
and `target/outside-art-timber-post-verification.txt`. Raw map audits and the
temporary authoring workspace were removed after recording scalar results.
Overall outdoor acceptance remains open.


### Softer shared house-roof paint — 2026-10-18

Ordinary Kanto hip roofs and flat-house pans now share a muted blue-gray
mineral paint response to the authenticated live source pigment. This reduces
the dominant cyan seen in Pallet/Fuchsia while retaining the native hue and
night exposure. The existing hip-only desaturation was replaced by that shared
response. Roof geometry, physical ceramic courses, ridge/height, source
guards, facade/window roles and placement/support are unchanged. The helper
has exactly two callers: the Kanto house and flat-house appenders. It is not a
global shader or service-building material change.

Eight existing house/roof/flat-house/service-binding checks and the production
build pass. Four muted headless 800×640 production/native-2D pairs were
personally inspected with preserved state, orbit 1, zoom 0 and normal camera:
Fuchsia `(30,26)` at noon and 22h; Pallet `(10,10)` at noon; Cinnabar `(12,12)`
at noon. Roof relief remains readable by day and night, with native lamps
retained. Cinnabar's Center was compared to the approved prior pond-overview
capture: its red roof, awning, doors, windows and walls retain their form.
The native Center/Mart binding test and the two-caller scope guard the service
assets; no new Mart-specific visual acceptance is claimed.

Evidence: `target/outside-art-house-roof-{fuchsia-day,fuchsia-night,pallet-day,center-control}*`
and `target/outside-art-house-roof-verification.txt`. No assets, packs, terrain,
water or trees were edited in this pass. Fuchsia/Pallet foliage is green in
the latest captures during the parallel tree work, which remains with the
user's other agent. All twelve retired tree paths remain absent.

Full outdoor acceptance remains open. The cottage foundation lawn strips
beside native quiet paths in Pallet/Fuchsia deserve a source-underlay check;
that is separate from this roof material correction, and genuine flower-plot
lawn must be preserved.


### Restored mountain masses and Cerulean river level — 2026-10-18

The user rejected flattening the mountains behind northern Cerulean. Kanto's
permanent rock recognition, original ridge profile and rise-weighted mainland
pigment are restored. Source-owned cave mouths and ground contacts retain their
openings, and walking support remains independent of decorative mountain relief.
The shoreline shader no longer promotes every bank into the canyon material.
Cinnabar's volcanic pigment stays local rather than becoming a mountain default.

The river needed a complete-source height calculation, not another cliff restyle.
Rock-lined inland bodies now resolve whether the connected maps leave them open
or fully enclose them. Each connected body has one level at its real walkable
rim. Open inland continuation carries explicit, validated water ownership through
native gaps, camera borders, worker messages and terrain-cache invalidation.
Separate closed bodies keep their own levels. Oceans retain sea level and
waterfall pools keep their separate authored treatment. Finite inland shelves do
not borrow the route-wide northward course extrapolation from a distant ledge.

Both the bridge deck and its water underlay use that same river datum. Structures
already fitted to their deck cannot receive a second land-height lift merely
because the river places them above an old fixed-height cutoff. Native deck
corners remain valid; offshore continuation cannot borrow a high land-edge level.

Production headless, muted 3D/native-2D pairs were personally inspected at
Cerulean `(20,4)` and `(31,11)`, noon, orbit 1, zoom 0, 1280×960. Both preserve
state. The final files are `target/outside-art-cerulean-corrected-{bridge,cottage}`
with `.png`, `-2d.png` and `.paired-state.txt` suffixes. A Cinnabar `(12,12)`
control retained the ocean baseline and Center model while showing restored
volcanic mass; this is not full-island visual acceptance. All captures remained
offscreen and no tree/forest assets were edited.

All 91 focused Rust checks pass, including complete closed-river recognition,
open continuation, bridge clearance/underlay, high-deck fitting, source masks,
worker bulk transport, cache invalidation and invalid-metadata rejection. Six
native map controls preserve 82 actual jump takeoff/landing checks with no audit
errors; the final Cerulean/Route24 recheck preserves 12. Saffron remains flat and
Route26 retains its directed drops. Raw catalog/source diagnostics were removed.
The scalar receipt is `target/outside-art-cerulean-river-verification.txt`.

Route 24 river-bank joins were then checked at `(4,8)`, noon, orbit 1,
zoom 0. The raised river had incorrectly activated the rigid mineral-pond
bank fit, separating the bank from the graded grass shelf beside the stairs.
That fit now requires an actual native mineral rim. Ordinary river banks retain
their land-corner heights; the water datum and cliff finish are unchanged.
The corrected `target/outside-art-route24-shelf-fixed` 3D/native-2D pair closes
the visible openings. The matching Cerulean `(31,11)` cottage control is
`target/outside-art-cerulean-bank-control`. Both pairs were inspected offscreen
with state preserved. All 31 terrace tests pass, including the new graded-bank
regression and the existing rigid mineral-pond regression. No forest assets
or implementation were edited.

## Shore backing at bridge corners — 2026-10-18

Fresh Route 25 views at `(20,8)` and Bill's cottage `(47,6)` exposed a pale
triangle below the bank at the cottage bridge's west corner. Final production
mesh inspection showed the cap at height 48 while one backing endpoint sampled
the neighboring bridge's zero terrain datum. The shared shoreline backing now
samples both endpoints just inside its actual neighboring bank cell, matching
the cap's ownership. Face positions and materials stay in place; bridge support,
water levels, buildings, posts and forests are unchanged. Exploratory return-wall
and cap-material changes did not close the opening and were removed.

The final headless, muted production 3D/native-2D pairs were personally inspected
at Route 25 `(47,6)` and Route 24 `(4,8)`, noon, orbit 1, zoom 0, 1280×960.
Both preserve state. The triangle is closed, and the earlier Route 24 shelf
repair remains intact. Evidence is `target/outside-art-route25-bank-endpoint-fixed*`
and `target/outside-art-route24-bank-endpoint-control*`. All 43 focused checks
pass: 32 terrace tests, seven elevated-pier tests and four enclosed-pond tests.
The new regression directly reproduces the high bank/zero bridge endpoint,
checks both bank contacts and preserves the water datum and flat-bank control.
Temporary geometry probes and raw audit catalogs were removed. The retired
tree assets and deleted renders directory remain absent. Overall outdoor
acceptance remains open.

## Flat shoreline-cap contacts beside sea rocks — 2026-10-18

Vermilion `(20,24)` exposed pale triangular openings at two walkway/rock
contacts. The recessed-rock closure pass admitted ordinary flat lawn and path
donors but excluded authenticated native shoreline-bank glyphs. These caps now
receive the same concealed rocky side from the existing sea level to the dry
cap. The shared predicate still requires a modeled recessed rock bed, a flat
neighbor and an authenticated bank drawing. It adds no cap, rim, water plane,
height change or town-specific asset. Buildings and forest work are untouched.

All 33 terrace tests pass. The new native-bank regression checks closure within
the existing rock relief envelope, exact top/toe contact, and unchanged water
geometry, footing and ownership. The initial test's planar-face tolerance was
corrected to admit the existing fractured rock relief; the implementation does
not flatten that relief.

Production headless, muted 3D/native-2D pairs were personally inspected at
Vermilion `(20,24)`, 12h and 22h, plus Route 20/Seafoam `(37,11)`, 12h,
orbit 1, zoom 0, 1280×960. All preserve state. The Vermilion openings are closed;
Seafoam retains its coastal datum, mountain mass and cave approach. Evidence:
`target/outside-art-vermilion-bank-contact-{fixed,night}*` and
`target/outside-art-harbor-bank-seafoam-control*`. Dense harbor rock groups still
show square-looking shallow-water shading in daylight. That optical issue is
separate from the repaired geometry and remains open, along with overall
outdoor acceptance.

## Sea-rock contact and land-depth optics — 2026-10-18

The remaining broad pale patches beneath sea-rock groups were isolated with
an offscreen shader diagnostic: replacing shore distance with open water
removed them. The retained field was rasterizing small model feet into binary
dry samples and using those same contacts to imply broad shallow ground.

Complete authenticated sea-rock drawings now seed signed distances to their
fitted model's outer footprint. Internal triangulation edges do not become
shorelines. The same four-sample RGBA texture carries contact distance in R
and actual land distance in G. Contact still damps waves and carries a thin,
broken wash; broad depth tint and caustic strength follow real land. Small
rock contacts have a narrower, quieter wash than a bank. Ordinary banks keep
identical contact/depth values. Source fallback guards, native rock placement,
mesh geometry, water height, swimming support, source clocks and live palette
selection remain intact. No town-specific assets or forest changes were made.

All 31 focused checks pass: 15 shoreline tests and 16 native water/palette/phase
tests. New coverage checks the actual model-contour distances at retained
samples, signed distance without internal triangle seams, open sea depth under
isolated rocks, preserved real-land shallows, and both channels across a
streamed crop. Existing worker/background image preparation comparisons,
incomplete/custom source controls, piers, ferries and source animation checks
also pass. Texture dimensions and image count are unchanged; preparation uses
an additional CPU distance buffer in the existing background path.

Final muted headless production 3D/native-2D pairs were personally inspected
at Vermilion `(20,24)` at 12h and 22h, and Route 20/Seafoam `(37,11)` at 12h,
orbit 1, zoom 0, 1280×960. Each preserves state. Rock rows no longer carry the
former broad beach-like halos offshore; real bank shallows remain visible.
The prior bank-contact geometry repairs remain closed. Evidence:
`target/outside-art-vermilion-rock-wash-{day,night}*` and
`target/outside-art-seafoam-rock-wash-control*`, with test logs
`target/outside-art-rock-optics-tests.log` and
`target/outside-art-rock-wash-palette-tests.log`. Diagnostic shaders, images
and source probes were removed. Retired tree paths and the removed renders
directory remain absent. This is acceptance of this optical refinement;
overall outdoor quality acceptance remains open.

## Northern Rock Tunnel source binding — 2026-10-18

The Route 10 north approach showed a flat pixel-art cave mouth and adjacent
rock strip. The source belongs to Route 9 in the connected scene: native block
`70` embeds the same four mouth tiles as block `06`, but neither its portal
nor surrounding cliff had a complete geometric binding. The existing shared
Kanto portal producer now admits that exact mouth variant. Verified cells in
the rest of block `70` use the existing plateau and east/south cliff profiles;
the four mouth cells retain flat walking support. Changed ink and foreign
atlases are rejected. No town-specific asset, new cliff style, building change,
forest change or water/elevation policy was introduced.

All 30 focused checks pass: 16 Kanto cliff tests, the portal matrix covering
four authenticated drawings, 12 mountain controls and the outdoor Kanto
material/footing control. The new external-pack regression checks every native
block phase, the four level mouth cells and custom/foreign-source rejection.
Native audits report no mesher errors. Route 9 gains exactly 16 modeled cells
(2300 to 2316), with no block-70 cells left as flat source artwork. Native Route 10
north/south family counts are unchanged; all three maps preserve their footing
ranges and their 59 directed jump checks. Those ranges do not claim that the
newly corrected mineral cap cells kept their former flat height. The scalar
receipt is `target/outside-art-route10-portal-scalars.txt`; raw catalogs were
removed.

Production headless, muted 3D/native-2D pairs were personally inspected at
Route 10 north `(9,0)`, noon, orbit 1/zoom 0 and orbit 0/zoom 1, plus the
southern cave `(8,5)`, noon, orbit 1/zoom 0, all 1280×960. Each preserves state.
The northern arch now joins the mineral body without a remaining pixel strip;
the existing southern entrance stays intact. Final evidence is
`target/outside-art-route10-complete-portal-fixed*`,
`target/outside-art-route10-portal-front*` and
`target/outside-art-route10-southern-portal-control*`. The mouth-only
intermediate was removed. Retired tree assets and the removed renders
directory remain absent. Overall outdoor acceptance remains open.

## Shared Johto framed notice boards — 2026-10-18

Violet's street signs used the older compact board with three generic raised
strokes. Three complete Johto source signatures (`45`, `47`, `78`) now use the
existing modern framed-sign mesh and its live native inscription inset. The
posts, projecting sill, header and recessed panel read as a physical fixture at
the normal camera scale. No model, texture or town-specific asset was added.
Actual source lettering and its changing palette remain in the scene atlas.
Custom profile ownership retains priority, including New Bark's existing signs.

All eight framed-sign regressions pass across the 13 supported source
signatures. Coverage includes complete identity and backing, changed/cropped
drawings, custom ownership, atomic four-cell claims, live text coordinates,
footing, raised-pier placement, and preserved roots/coverage around clearances.
Two existing controls now select the intended Kanto/park signatures by identity
rather than their former list positions. The native Violet audit replaces
24 legacy sign cells with 24 framed sign cells; all other family counts, total
modeled coverage, footing range and three jump checks are unchanged. Native
New Bark still reports its 16 profile-owned legacy sign cells. Both audits have
no mesher errors. The scalar receipt is
`target/outside-art-johto-framed-sign-verification.txt`; raw catalogs were removed.

Matched production 3D/native-2D pairs were personally inspected at Violet
`(18,18)` at 12h and 22h, and Cherrygrove `(23,8)` at 12h, all orbit 1,
zoom 0, 1280×960. Every pair preserves state. Evidence is
`target/outside-art-violet-framed-signs-{day,night}*` and
`target/outside-art-cherrygrove-street-sign-control*`. The initial Cherrygrove
shore-only view was removed from sign evidence. Violet's initial northern
channel view remains an exploratory terrain view, not a repaired water defect;
this pass changes only sign bindings and their tests. Buildings, water/ledge
implementations and the other agent's forest work are untouched. Captures stayed
muted and offscreen. Retired tree assets and the removed renders directory
remain absent. Overall outdoor acceptance remains open.

## Actual Surf users and body immersion — 2026-10-18

The Route 24 location preview at `(4,8)` contains a source Surf sprite but no
retained field-move user. Its missing 3D rider is not evidence that production
should invent a Pokémon or restore a generic Surf card. The location was also
confirmed against the northeast Cerulean cottage at `(31,11)` with an offscreen,
state-preserving 3D/native-2D pair. The tall river-bank composition remains open;
this pass does not change its height or finish.

For valid Surf review, the existing production-controller regressions created
settled Route 44 QA saves by actually using Surf with a non-lead Lapras and with
Goldeen. These are seeded QA fixtures, not a user's playthrough. Capture must use
the same pack identity as the save: these fixtures use the completed text-TUI
pack, whose graphics also render through the production graphical client. The
browser pack correctly rejects their different modpack identity.

The real Lapras capture exposed a separate defect: its lowest body vertex sat
on top of the water, leaving its belly and flippers exposed as if it were walking.
The shared Surf fit now derives a local waterline from the same authenticated
torso/shell used to find the rider's seat. Forty percent of that body's vertical
range is immersed, bounded below the carrying surface. Only `SurfMount` receives
this offset. The rider continues to follow the mount's animated skin; ordinary
followers, actual Pokédex dimensions, water planes and terrain support stay on
their existing paths. No species asset or town-specific clone was added.

Five focused regressions pass: the two actual-field-move controller fixtures,
the body-seat/animated-skin check, the both-trainers/five-species turning and
dismount check, and the no-generic-avatar-or-mount-card check. The turning test
now verifies that the body-local waterline meets the water plane while the seat
remains above it, preserving uniform physical scale and animated rider contact.

Fresh headless, muted production pairs were personally inspected at Route 44
`(38,4)` for both Lapras and Goldeen, 1280×800, retained save time/facing and
default saved-session camera. Both report `state_preserved=true`. Lapras now
sits partly in the water, and Goldeen retains its smaller physical body with
the trainer seated above it. Evidence is
`target/outside-art-surf-{lapras,goldeen}-waterline-{2d,3d}.png`; the Lapras
before image is `target/outside-art-surf-lapras-review-3d.png`. Build and test
logs remain ignored under `target/`; temporary QA saves were removed after
verification. Forest implementation, building assets and cliff geometry were
not edited. Full outdoor acceptance remains unproved.

## Inland banks retain their own authored ledges — 2026-10-18

The river-bank investigation found a real height-ownership defect. Sea-facing
land already filtered the complete connected-source ledge list to its own dry
domain, but inland river domains inherited every ledge in that list. A ledge
across the water could therefore raise an unrelated bank. The new regression
reproduced the old failure on a flat bank, yielding roughly `0.79` terrace levels
instead of zero despite that bank having no ledge.

Rock-lined inland river domains now use only ledges whose actual takeoff and
landing belong to that dry domain, expressed in its local coordinates. The
regression proves that the opposite bank stays flat, the real ledge retains its
directed drop, the connected water retains one level, and translating the
complete map inputs preserves both banks and the water datum. Existing ordinary
inland/forest treatment remains on its prior path. An exploratory rim-outcrop
change was removed; this repair does not replace the shared cliff finish.

All 63 focused checks pass: 56 terrace/datum/course/mineral-pond checks and seven
elevated-pier checks, including their external-pack controls. Six native audits
retain all 82 jump samples with no mesher errors. Source/model cell counts stay
the same. Cinnabar's height range and ocean ownership, Route 26's directed
terraces, and Saffron's flat zero datum retain their previous scalar results.
Cerulean's native maximum footing changes from `27.1106` to `16`, Route 24's
from `21.8098` to `17.2480`, and Route 25's from `21.25` to `20.5`. Their native
river controls now resolve to zero, rather than Cerulean and Route 24 acquiring
different nonzero river offsets from unrelated land constraints.

Production headless, muted 3D/native-2D pairs were personally inspected at
Cerulean `(31,11)`, Route 25/Bill's cottage `(47,6)`, and Route 24 `(3,8)`, noon,
orbit 1, zoom 0, 1280×960. All preserve state. The Route 24 review starts on dry
ground and retains its actual human actor rather than an unmounted Surf preview.
Evidence is `target/outside-art-inland-bank-{cerulean,bill,route24}{,-2d}.png`;
the scalar receipt is `target/outside-art-inland-bank-verification.txt`. Raw
audit catalogs and discarded experiment logs were removed. No tree/forest
implementation, building, species asset, cliff texture or water shader edits
were made in this pass.

This proves a domain-isolation repair, not complete acceptance of the bank's
appearance. Bill's cottage and the northbound route still show substantial
straight bank faces, and the mountain backdrop still needs a stronger visual
review. Overall outdoor quality remains open.

## Native Kanto cliff ink no longer spawns boulders — 2026-10-18

The repeated `$24` ink in Kanto block `$29` is an east-facing cliff beside a
plateau, not a drawing of round barriers. Removed its modeled boulder and card
fallback bindings, restored its native mineral plateau/side geometry, and
removed the obsolete flat mineral apron compensation that otherwise doubled
Cinnabar's cliff lift. Genuine native land and shoreline round barriers retain
their existing bindings. The legacy path asset enum remains unbound.

Source-proven permanent Kanto mountains now use the existing shared mineral
shoulder shaping, retaining their existing pigment and texture. Complete native
Kanto cave mouths pin their arch seating joint, fading back into the shared
sculpting outside the opening. The regression checks that the actual portal
vertices stay seated while remote mountain faces still receive shaping.

The production examples build and 159 unique focused tests pass. Six native
map audits remove 264 false source-cell boulder claims while preserving real
land/shore barrier counts and all 34 authored jump samples, with no mesher errors.
Cinnabar's maximum footing remains 22, Saffron stays at zero, and the native
Cerulean/Route 24/Route 25 water controls stay at zero. Route 24's blocked mineral
maximum changes to 21.9387 as the actual native cliff is recognized.

Personally inspected production headless, muted 3D/native-2D pairs at Route 24
`(3,8)`, Cinnabar `(12,12)`, and Route 10 North `(9,0)`, noon, orbit 1, zoom 0,
1280×960. All preserve state. Route 24's false left boulder row is gone; the
final tunnel view retains a closed seating joint around its open cave mouth.
Evidence is `target/outside-art-kanto-no-false-boulders-{route24-day,cinnabar,tunnel}{,-2d}.png`.
The compact receipt is `target/outside-art-no-false-cliff-boulders-verification.txt`;
raw audit catalogs were removed after verification. No forest implementation,
building assets or water shaders were edited. Full outdoor acceptance remains
unproved; this pass does not establish final visual acceptance for all six maps.


## Native map heights replace the false Route 24 slope — 2026-10-18

The connected Route 24 solve inherited 36 ledges from distant maps. Its dry
right bank reached seven extra terrace levels, falling from `[7,7,0,0]` into
water within one source cell. A material-only experiment did not repair that
geometry and was removed.

Production now supplies complete native regions in rendered ownership order.
Kanto maps compute their own translation-normalized height profiles, joining
only adjoining dry corners at map connections. Authored jump cuts remain
separate. Standing water components retain one level, with explicit waterfall
pools excluded from that flattening. Saffron's entire native border stays flat;
its neighbor owns any approach. Non-Kanto primary maps keep the existing path.
The shared brick bridge fascia also uses beveled masonry with live muted deck
pigment; internal cell boundaries do not receive exposed edge ornament.

The production examples build. Focused checks pass: 59 terrain tests, 15 paving
tests and seven pier tests. The terrain suite reproduces the former seven-level
false rise, verifies a flat native bank and water, preserves all seven actual
remote jump drops, checks local connection ramps and translation invariance,
and anchors the flat city boundary. A separate single-map six-map audit has no
mesher errors and retains 82 authored jump samples; it is a native-map control,
not evidence for the connected compositor by itself.

Personally inspected offscreen, muted production/native-2D pairs at Route 24
`(9,8)` (noon, orbit 1, zoom 0, pitch 30, 1280×960) and Bill's cottage/Route 25
`(47,6)` (noon, zoom 0, pitch 45, 1280×960). Both preserve state. Route 24's bank
is flat beside the bridge and no longer descends through a huge grass slope;
Bill's courtyard and low pond banks remain closed. Evidence is
`target/outside-art-native-terrace-{route24,bill}{,-2d}.png`, with scalar results
in `target/outside-art-native-terrace-verification.txt`. The exact rejected
Route 24 before view remains `target/outside-art-flat-kanto-route24.png`.
Raw catalogs and temporary diagnostic/experiment output were removed. No forest
implementation or building assets were edited. Other map joins and overall
outdoor visual quality still require review; this is acceptance of the reported
bank-height repair, not complete outdoor acceptance.


## Connected route follow-through — 2026-10-18

The next review follows the corrected bank to Route 24's north end `(9,0)` and
Route 25's western turn `(3,6)`. Personally inspected paired production/native
2D captures at noon, zoom 0, pitch 45, 1280×960, all headless and muted. Both
preserve state. The bridge approach and adjacent native shelves remain flat;
no second broad descent to water appears at this turn. Evidence is
`target/outside-art-native-join-{route24-north,route25-west}{,-2d}.png`.

Added a production integration regression using the external pack and the real
renderer/controller app, not a second terrain fixture. It renders Route 24 at
`(9,8)` and `(9,0)`, requires all corners of source bank cells `(22..27,16)` to
remain zero, verifies water at `(30,16)` is zero, validates water/barrier metadata,
and compares the complete published height fields across both positions. The
new regression passes (one test, zero failures), with its build/run output in
`target/outside-art-native-join-production-test.log`. The generated render
pairs and tests prove these route controls; overall outdoor acceptance remains
open. No forest or building assets were changed.

A third paired control at Route 26 `(9,28)` (same offscreen settings) preserves
state and retains successive native jump terraces. Its immediate foreground
mineral contact shows a thin patterned edge that warrants closer diagnosis;
this is not final mountain-art acceptance. Evidence is
`target/outside-art-native-join-route26-control{,-2d}.png`; the compact review
receipt is `target/outside-art-native-join-verification.txt`.


## Outdoor rock dissolve banding — 2026-10-18

The thin patterned Route 26 contact was not an open seam: an offscreen
zero-radius diagnostic retained a closed opaque mineral face. The player-reveal
aperture's interleaved-gradient threshold produced conspicuous diagonal stripes
on that grazing face. Outdoor canyon apertures now use a shared pixel-anchored
integer hash, retaining fine grain without those diagonal bands. Other reveal
callers retain their previous coverage. Geometry, heights, source support,
shadows, foliage and architecture remain unchanged.

The viewer builds and the final GPU capture has no shader compilation errors.
Personally inspected its production/native-2D pair at Route 26 `(9,28)`, noon,
zoom 0, pitch 45, 1280×960, headless and muted, with `state_preserved=true`.
Before is `target/outside-art-native-join-route26-control.png`; after is
`target/outside-art-mineral-dissolve-route26{,-2d}.png`. The diagonal striping
becomes fine stable grain while the player and enclosing rock remain readable.
Two existing aperture specialization/source-contract tests pass. The CPU reveal
tracking fixture still fails its foot-position expectation (`Z=-8` versus zero),
reproduced identically in the old pre-change test binary and the new binary;
this is not a passing full-suite claim. Diagnostic images were removed after
the A/B review. The compact receipt is
`target/outside-art-mineral-dissolve-verification.txt`. Overall outdoor visual
acceptance remains open.


## Conforming mineral wall edges; Mt. Moon light leak remains — 2026-10-18

The shared sixth-cell wall lattice did not include six-pixel ledge floors.
A shorter wall acquired its floor through clamping while its taller neighbor
spanned past that point. Mineral walls now include the same authored floor
multiples, snapping nearby regular rings to those exact values. Relief also
returns boundary points directly instead of relying on approximate `sin(PI)`.
The producer regression compares full XYZ float bits on adjoining tall/short
edges at three scales and four ledge courses, including a zero-coordinate edge.
Walking heights, source footprints, materials, and portal models are unchanged.

The final production viewer builds. All 27 rock-geometry and 12 mountain tests
pass (39 unique), including external native-route winding/cap/support and
Route 26/Lavender/Cinnabar/Johto footprint controls. Personally inspected
headless, muted production/native-2D pairs at Mount Moon Square `(20,12)`
(noon, orbit 1, zoom 1, pitch 45, 1280×960) and Route 3 `(52,2)` (same settings,
zoom 0). Both preserve state. Evidence is
`target/outside-art-mineral-exact-edges-{square,route3}{,-2d}.png`; results are in
`target/outside-art-mineral-exact-edges-verification.txt`.

This establishes edge conformance, **not repair of the visible Square light
leak**. Its mouth region still contains 63 cyan candidate pixels; the nearby
grass corner changes from 186 to 184. Disabling player reveal did not remove
those pixels, and a flat-color fragment diagnostic retained uncovered edge
pixels. Sampled coincident mineral points after terrace fitting showed no
height split; that check does not cover all noncoincident or nonmineral edges.
The remaining coverage/assembly defect needs further diagnosis. Additional
Route 4 and Square mouths were confirmed against native pack warps and retained.
Rigid doorway, backing-overlap, and height-welding experiments were removed;
no forest, landmark, building or source-pack changes remain from those trials.
Raw catalogs, traces, temporary shaders and rejected captures were removed
after recording scalar evidence. Overall outdoor visual acceptance stays open.

## Solid cave thresholds close most doorway light leaks — 2026-10-18

The shared cave threshold sat slightly above its source floor as an open,
paper-thin quad. It now has four narrow side faces and an underside, closing
that existing volume without changing the entrance, arch, source claims,
walking heights, warps, or mountain profile. All four native portal variants
pass the existing complete-drawing/footing regression, extended to require a
solid front riser and bottom. The production viewer builds.

Personally inspected muted, offscreen production/native-2D pairs at Mount
Moon Square `(20,12)` and Route 3 `(52,2)`, noon, orbit 1, pitch 45, 1280×960;
Square uses zoom 1 and Route 3 zoom 0. Both preserve game state. Route 3 retains
its authored forecourt and recessed opening. At Square the fixed mouth region
`(830,540)..(980,650)` falls from 63 to 20 cyan candidate pixels using the same
`B > R + 15 && B > 120` check. This is a visible partial repair: the narrow
side seam remains, and the separate grass/rock corner still contains 184
candidate pixels. Neither remaining gap is accepted as finished scenery.

Evidence is `target/outside-art-threshold-volume-{square,route3}{,-2d}.png`,
its paired state files, build/test logs, and scalar verification receipt.
Two-sided rendering and cap-backing experiments did not improve coverage and
were removed. No forest, building, water-level, or pack assets were changed.

## Cianwood shoreline follow-through — 2026-10-18

Reviewed a new land pose at Cianwood City `(9,18)`, noon, orbit 1, zoom 0,
pitch 45, 1280×960. The muted headless production/native-2D pair preserves
state (`target/outside-art-cianwood-land-review{,-2d}.png`). The sandy forecourt,
native house and movable rocks retain their source arrangement; open water
remains level beside the low shore. This is not final art acceptance. The
small upper rock return exposes a rough contact, while the offshore repeated
buoy course reads as flat white discs. Source inspection subsequently confirmed
these are already shallow capped meshes from the shared `buoy_course` producer,
not unconverted raster planes. The classifier includes Johto/Modern `$58` in
water blocks `$30..$39`, already seated at the water datum. Improving that
existing producer must preserve its datum and native silhouette placement.

An initial water-start fixture lacked a modeled Surf mount/rider and is
excluded from visual acceptance; the land pair is the retained scene control.
No actor, forest, shoreline or elevation implementation was changed for this
review. Overall outdoor quality remains unfinished.

## Shared offshore buoy collars — 2026-10-18

Improved the existing `buoy_course` producer instead of adding a second buoy
asset. Its shallow capped discs are now rounded collars with open water through
their centers, two bindings using the native secondary swatch, and smoothly
varying normals. The source silhouette still owns each center and radius.
The old footprint and vertical envelope remain exact: `-0.10..+0.20` source
tiles relative to water. Existing source recognition, palette sampling, water
backing, authored claims and walking/swimming heights are unchanged. Wrapped
ring indices keep both geometric seams coincident. No town-specific meshes,
new assets, water elevations, forests or building models were introduced.

All 11 focused buoy tests pass, covering closed winding/nondegenerate geometry,
the open center and both native paint roles, preserved footprint/envelope,
Johto/Modern source guards, cropped/reserved/custom-art fallback, water backing,
footing preservation, no square shoreline, and exclusion from land terraces.
The production viewer builds. Personally inspected headless, muted production
and native-2D pairs at Cianwood `(12,18)` at noon and Olivine `(29,29)` at noon
and 22h, orbit 1, zoom 0, pitch 45, 1280×960. Every pair preserves game state.
The Cianwood before/after comparison shows a clear open collar in place of a
bright closed disc; Olivine verifies the same shared mesh across the harbor
course. At night the source palette dims the collars while their blue bindings
and holes remain readable, without a white luminous cap.

Evidence: `target/outside-art-johto-buoys-before{,-2d}.png` and
`target/outside-art-buoy-collar-{cianwood,olivine,olivine-night}{,-2d}.png`,
paired-state files, final build/test logs and scalar verification receipt.
The duplicate trial implementation and its logs were removed. The rough
Cianwood rock return and remaining Mt. Moon contacts are still open issues;
this buoy improvement does not establish overall outdoor visual acceptance.

## Native mineral corners no longer create false planted seats — 2026-10-18

Traced Cianwood's flat rectangular rock shelf to two blanket exceptions for
Johto/Modern blocks `$6a/$6c`. The rock passes assumed these drawings contained
planted trees, suppressing both mountain relief and mineral erosion. The native
drawings instead contain mineral courses. A source-phase-and-tile guard now
recognizes those actual courses; they continue the shared mountain finish.
Non-native planted ink retains its previous exclusion and level seat. Tree
placement, forest assets, materials, building models, collision and water levels
are unchanged. This is a shared source correction, with no Cianwood special case.

The new regression reproduces the false interior opening in the mountain
field, verifies native corners receive continuous relief in both atlases,
preserves footing/ownership, and retains the custom planted-art guard. All 56
focused outdoor tests pass with the completed external pack and ignored tests
explicitly included: winding, caps, contacts, open paths, native footprints and
the existing level-seat controls. The production viewer builds.

Personally inspected headless, muted production/native-2D pairs at Cianwood
`(12,18)` and Route 26 `(9,28)`, noon, orbit 1, zoom 0, pitch 45, 1280×960.
Both preserve game state. Cianwood's former rectangular inset shelf now joins
the adjoining mineral silhouette; the beach nook, movable rocks, house and
shoreline retain their source layout. Route 26's matching control replaces
the same false corner shelves with joined rock while preserving its visible
terrace corridor. Evidence is `target/outside-art-native-corner-{cianwood,route26}{,-2d}.png`,
the before controls `outside-art-buoy-collar-cianwood.png` and
`outside-art-mineral-dissolve-route26.png`, and the final build/test logs and
scalar receipt. A thin lower Cianwood seam and the Mt. Moon contacts remain
unresolved; this is not complete cliff or outdoor visual acceptance.

## Hidden mountain backing stays inside the mineral surface — 2026-10-18

The pale line across Cianwood's lower cliff came from its hidden reveal floor
protruding through descending rock. A native geometry A/B that omitted only
that floor removed the line. The initial mineral-only fragment diagnostic was
misleading because discarded surfaces still participated in the depth prepass;
it did not establish an open cliff mesh. No visible cliff recut was needed.

The backing producer now retains authored cap ownership before mineral normals
are recomputed, excluding upward-facing weathered wall facets from floor
projection. This ownership is temporary build metadata and is cleared before
the mesh leaves the terrain builder. Projection also requires one source pixel
of mineral clearance above the existing hidden floor. Its walking datum and
seating bias are unchanged. The visible rock, palette, water, collision,
forest assets and buildings remain untouched by this backing repair.

Two new regressions cover wall facets falsely supplying backing and descending
caps touching the floor. They retain the visible mesh and footing, require an
interior backing boundary, and keep its original height. All 58 focused outdoor
tests pass with the external pack and ignored tests included. The final
production viewer builds.

Personally inspected headless, muted production/native-2D pairs at Cianwood
`(12,18)`, Route 26 `(9,28)` and Mount Moon Square `(20,12)`, noon, orbit 1,
pitch 45, 1280×960. The first two use zoom 0; Square uses zoom 1. Every pair
preserves game state. Cianwood's pale horizontal line is gone; Route 26 retains
its floor beneath the player reveal and its native terrace corridor. Square's
separate mouth-side and grass/rock gaps remain: the same candidate-pixel checks
still measure 20 and 184 respectively. Those defects remain open.

Evidence: `target/outside-art-backing-pixel-clearance-{cianwood,route26,moon}{,-2d}.png`,
their paired states, final build/test logs and scalar receipt. The earlier
normal Cianwood view remains in `outside-art-native-corner-cianwood.png`.
Temporary shaders, the native backing-omission switch and diagnostic captures
were removed after recording the A/B evidence. Overall outdoor quality is
still unproven beyond the inspected fixes and controls.

## Route 15–14–13 approach and fitted bank/deck closure — 2026-10-18

Inspected fresh muted headless production/native-2D pairs at Route15 `(12,10)`,
Route14 `(6,28)`, and Route13 `(50,6)` and `(54,6)`, noon, orbit 1, zoom 0,
pitch 45, 1280×960. Every pair preserves game state. The western Route15
ledge and Route14 junction follow the native layout without a broad coastal
slope. These views expand the older component reviews; they do not establish
whole-route visual acceptance. Repeated pale boundary posts and paving still
need a separate quality review. Forest assets and styling were left alone.

Route13 exposed a bright open slit between the raised grass bank and the
sloping boardwalk. The bank closure used the nominal four-pixel pier height,
although the real deck approaches have independently fitted corners. A trial
one-pixel recess reduced the slit but did not close it and was superseded.
The shared pier producer now supplies its actual fitted underside corners,
using the same frame depth as its generated structural sides. Terrace closure
interpolates that actual contact edge. It changes neither walking surfaces,
water levels, the visible ledge crown nor source footprints, and has no
Route13-specific asset or exception.

The inspected Route13 join is closed in both views. In the fixed before/after
rectangle `(140,670)..(400,775)`, cyan candidates (`B > R + 15`, `B > 120`)
fall from 662 to zero; this is a local visual diagnostic, not a whole-scene
hole count. The geometry regression compares the emitted bank toe against
the actual generated sloping deck frame at two scales, including approaches
below the old fixed cutoff, and preserves the original bank lip. All 37
terrace tests, seven pier tests and 15 paving tests pass with the external
pack and ignored tests included. A fresh Route24 `(9,8)` paired control keeps
the previously repaired flat bank and level river.

Evidence is in ignored `target/outside-art-route15-west-before*`,
`outside-art-route14-south-before*`, `outside-art-route13-coast-before*`,
`outside-art-route13-bank-contact*`, `outside-art-route13-deck-contact*`,
and `outside-art-bank-contact-route24-control*`.

The preceding Route31 review at `(14,9)` and the Dark Cave approach `(32,6)`
also has preserved-state production/2D pairs. Its pond/terrace relationship
remains open for investigation; no Johto height change was retained before
the user redirected the review east of Fuchsia. The rejected Moon legacy-wall
trial retained 336 old triangles but changed neither observed gap count
(20 mouth-side, 184 grass-side); its diagnostic code and scratch captures
were removed, leaving a scalar receipt.

Final current-source viewer rebuild succeeds. Both Route13 pairs were recaptured
from that build and personally inspected again; the slit remains closed, the
local cyan diagnostic remains zero and both paired states are preserved.
