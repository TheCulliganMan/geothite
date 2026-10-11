# Human character review

Current art direction supersedes the earlier face paintings below: see [anime prototype and intermediate conversion record](stylized-character-faces.md). The user rejected both realistic faces and the subsequent rounded storybook direction; the roster is not approved or complete.

The goal remains individual player and NPC art that reads clearly in the game,
with clean maps, smooth silhouettes and recognizable costumes. A successful
import, a rendered lineup or passing geometry tests does not approve the cast.

## October 10 baseline

The production `PeopleGpu::spawn` reviewer captured 75 appearances, each with
front, three-quarter, side and back views, from `assets/open-models`. All 75
3200×1200 PNGs decoded successfully. Front/side/back contact sheets were
visually inspected for the whole cast; detailed four-angle inspection began
with Chris, Beauty, Bugsy, Clair, Cooltrainer F, Elm, Granny, Karen and Mom.
This is an initial triage, not a completed individual art pass.

Concrete remaining issues observed in this baseline:

- Chris has a zipper and triangular fabric treatment repeated on his back;
  the backpack/rear jacket pass below addresses these two baseline defects.
  Kris has a generic face and an undifferentiated outfit; the tied-hair pass
  below replaces her ball-shaped pigtails.
- Beauty, Bugsy, Clair, Daisy, Erika, Jasmine, Karen, Link Receptionist, Mom,
  Receptionist, Rocket Girl and Sabrina have disconnected oval side hair.
  The new continuous rear curtain addresses this particular shape; scalp
  joins, individual fringes and character-specific outlines still need review.
- Dresses, skirts, robes and coat hems have a sharp waist ledge and straight
  cone profile. The new multi-ring shell rounds the hem and closes its rims;
  the subsequent weighted-garment pass below addresses locomotion clipping.
- Lass, Cooltrainer F, Nurse, Gramps and Super Nerd repeat front costume
  details on their backs. Granny's painted fabric and plain modeled hem have
  a visible material transition. Painted faces improve identity but do not
  repair UV mapping or geometry by themselves.
- Blue, Brock, Bruno, Elm, Koga, Lance, Oak, Rival, Surge and Will do not show
  their distinct authored hairstyle silhouettes. The appearance names exist
  but several have no corresponding added geometry.
- Chuck and Outdoorsman had a round mouth-covering beard blob; the jaw fitting
  pass below replaces it. Elder, Pharmacist, Blaine, Kurt and Pryce had
  detached oval side hair; the scalp pass below removes these additions.
  Glasses looked like separate beaded rings, without a bridge or temples;
  the continuous frame pass below addresses those construction defects.
- Rocket and Rocket Girl lack the chest insignia. Black Belt and Bruno lack
  the martial belt; Clair lacks her cape (Lance resolved below); Kimono Girl, Erika and
  Falkner lack wrapped garment details. Swimmers retain full trouser/shirt
  geometry, and children labeled with shorts often still wear full trousers.
- Biker, Bill, Clerk, Cooltrainer M, Fisher, Fishing Guru, Gameboy Kid,
  Gentleman, Gym Guide, Morty, Officer, Pokefans, Red, Rocker, Sailor,
  Standing Youngster, Teacher and Unused Guy remain mostly color variants.
  Their role accessories and clothing construction need individual work.
- Bug Catcher, Cal and Captain have hats but otherwise generic costumes.
  Janine, Misty, Old Link Receptionist, Reds Mom, Twin and Whitney had simple
  ball/oval tied hair, replaced in the tied-hair pass below, and still lack
  their individual outfit details. Sage and
  Scientist have only a generic robe/coat silhouette.
- Angular base cheeks, hands, elbow outlines and shoes remain visible across
  the cast. Evaluate topology and shading against the painted faces rather
  than hiding these with more accessories.

Map readability and moving poses remain separate required checks. The studio
camera does not prove face readability at game scale, unobstructed buildings,
correct footing, or absence of leg/hair/garment intersections in motion.

The quarter-cycle Run capture of Beauty, Mom, Elm, Granny and Cooltrainer F
exposed knees through the original rigid garment shell. This baseline defect
motivated the weighted-garment pass below.

The retained curtain was checked at all four angles on all twelve affected
appearances. It removes the detached oval hair pads and joins the back hair,
but still needs a better scalp transition and individual lock shapes. Separate
crown experiments were rejected after GPU inspection because they looked
like helmets. Four appearance tests pass, including shell winding, finite
normalized normals and the open face area. The supplied-rig regression also
passes for all four shared rigs, preserving bind geometry and animated clips.

## Weighted garment pass

All 29 appearances with a modeled skirt, dress, robe or coat now use the
production GPU skin palette. The waist blends weights transferred from nearby
body vertices; lower fabric blends thigh and shin joints. Mesh-space bind
positions remain intact, and each actor owns its live skeleton while sharing
cached garment geometry and materials. Coats have an open front. Dense vertical
sections let the garment bend around the knees instead of bridging the entire
leg with one large triangle. The original seven profile rings alone were
insufficient: a full-cast moving review still showed knees breaking through
long robes. Those sparse intermediate versions were rejected.

Visually inspected all 29 appearances at idle and Run phases 0, 0.125, 0.25 and
0.75, each from four angles: 145 GPU captures, 580 views. The large knee holes
and waist tears seen in earlier attempts are resolved in these sampled views.
This sampling does not certify every frame or approve each outfit. Long robes
still have bunched, angular rear hems, lack individual drape, and share generic
tubular silhouettes. Several faces, hair attachments and painted rear costume
details remain visibly wrong, as catalogued above.

PlayersHouse1F and VioletPokecenter1F were also inspected with the local base
pack through the production map viewer. Mom and Nurse remain readable at room
scale with the new garments. These two views are limited art checks; they do
not approve all maps, all occlusions or gameplay.

Verification: all 1,000 enabled renderer library tests passed. External-rig
tests passed for all four body rigs, the production player path, all 29
garments' bind geometry and eight sampled gait phases, and separate local and
remote GPU instances. Garment checks cover finite normalized weights, valid
joint indices, animated bounds, shared immutable assets, independent live
joint palettes and recursive retirement. Screenshot and contact-sheet files
are temporary evidence and are removed after review.

## Backpack and rear jacket pass

Chris, Kris, Cal, Red, Outdoorsman and Brock now have torso-attached modeled
packs with rounded rectangular fabric bodies, outer pockets, recessed zipper
lines and pulls. Shoulder straps follow a fitted curve around the chest and
over the shoulders. Early circular straps floated in front of the body and
were rejected; the retained version was inspected at idle and opposing Run
phases 0.25 and 0.75, from four angles for all six appearances (72 views).
Pack geometry and materials are cached per appearance, and each instance
attaches to its own Chest joint. Hiking packs are taller than trainer packs.
An oblique PlayersHouse1F capture with the local base pack confirms Chris's
yellow pack remains legible at game scale. This is one room-scale art check.

Chris's rear jacket now samples a plain red region of his existing painting.
Rear atlas faces have separate UV vertices with the original positions,
normals, joint indices and weights. Moving only individual UV corners produced
stray zipper/collar lines from interpolation across the texture atlas; those
intermediate versions were rejected. Front jacket, face, hair, trousers and
shoes retain their original UVs. No new texture or converted rig is required.

The external geometry regression checks all six appearances, preserved body
skin attributes, unchanged front/unrelated UVs, finite normalized pack normals
and mesh-space attachment bounds. The four supplied-rig regression and the
production player GPU reuse/animation regression pass. All 1,005 enabled
renderer library tests pass (nine external-asset tests remain ignored in the
default suite).

These additions improve rear silhouettes without approving these characters'
complete art. Kris still has an undifferentiated outfit; her ball-shaped tied
hair is replaced in the following tied-hair pass. Red and Cal still need
distinct faces/clothing; Brock lacks his authored
hairstyle; Outdoorsman's beard is addressed in the next pass. Chris's hands,
cheeks, shoe geometry and cap profile also need further individual review.

## Jaw fitting pass

Chuck, Outdoorsman and Elder now have a fitted lower-face beard surface instead
of a round ellipsoid over the mouth. Its upper edge dips below the lips at the
center and rises toward the cheeks. The contour narrows around the chin;
Elder's lower contour extends farther than the two short beards. Geometry
samples the supplied rig's jaw triangles and is generated once per cached
appearance, then follows the existing Head attachment.

The first fit used a constant depth outside the projected jaw and produced
jagged lower edges. That version was rejected. The retained fit uses the
nearest projected triangle edge beyond the jaw silhouette and a narrower
lower contour. All three were visually inspected from four angles at idle
and quarter-cycle Run. The mouth and nose are visible, and the former round
pad no longer projects ahead of the face. Fine edge thickness, individual
hair texture and jaw faceting remain unfinished. Elder's side-hair pads are
removed in the following scalp pass.
This is a facial readability improvement, not approval of these complete NPCs.
All 1,006 enabled renderer library tests pass.

## Older NPC scalp pass

Elder, Kurt, Pryce, Blaine, Gramps, Pharmacist, Pokefan M, Kurt Outside and Sage
no longer have detached ellipsoids at the sides of the head. Sparse-haired
appearances use their existing fitted scalp surface with corrected hair color;
bald appearances have skin-colored crowns and retain side/back hair. The
protected central face region preserves eyes and eyebrows. Gramps retains his
existing painted texture. An added scalp band was visually rejected because
it looked like a headband.

All nine were inspected from four angles at idle and Run phase 0.25 (72 views).
Detached side pads are removed; angular scalps, jagged hairline transitions,
and the difference between crown and face skin still require improvement.
All 1,009 enabled renderer tests passed after the retained changes.

## Continuous glasses frame pass

Blaine, Elm, Gramps, Granny, Pharmacist, Pokefan M, Scientist and Super Nerd now
have continuous smooth wire rims, a bridge and curved temple arms. Rim depth
uses the supplied rig's face at eye height instead of its foremost nose vertex.
Pokefan M and Super Nerd retain round frames; the other frames have a slightly
flatter outline. Frames use the existing Head attachment and cached geometry.

All eight were visually inspected from four angles at idle and Run phase 0.25
(64 views). The eyes remain readable, and the former disconnected beads and
missing arms are resolved. Fine fitting at the nose and ear, individual frame
styles and room-scale legibility still need review. Granny's formerly detached
bun is addressed in the next pass;
repeated back costume markings, generic faces and garment deformation remain
visible unrelated defects; this pass does not approve the complete characters.

The external-rig regression covers all eight frames' finite normalized normals,
head bounds and triangle winding. It caught a tube-frame discontinuity at a
temple bend; transporting the cross-section along the curve fixes that twist.
The regression passes, and all 1,011 enabled renderer library tests pass
(ten tests require external assets and remain ignored in the default suite).
Final GPU captures were inspected again after the tube correction. Temporary
review captures and logs were removed after recording the results here.

## Tied hair pass

All sixteen appearances with buns, topknots, ponytails, pigtails or rolls now
use continuous cubic gathers with tapered cross-sections and subtle lengthwise
grooves. Roots begin within the head bounds; knots and tails are no longer
separate ellipsoids. Buns have rounded ends, ponytails taper, and rear, high,
side and paired gathers use separate control curves. Kris has wider paired
tails, Misty has an asymmetric side tail, Nurse has shorter rolls, and Bruno
has a compact topknot. Granny's added hair uses a warmer gray to reduce the
difference from her existing painting. Production caching and Head attachments
remain shared with the game.

The cast covered here is Bruno, Cooltrainer F, Granny, Janine, Kimono Girl,
Kris, Lass, Misty, Nurse, Old Link Receptionist, Pokefan F, Red's Mom, Swimmer
Girl, Teacher, Twin and Whitney. Each retained appearance was inspected from
four angles at idle and Run phase 0.25 (128 final views). Early tight bun curves
folded around themselves and failed winding checks; those versions were
rejected. Bun contours were then rounded and reinspected at both poses.

The detached ball/oval construction and knot-to-tail gaps are resolved in
these samples. Buns still have dark, sharp undersides. The colored forehead
fringe and temple mismatches were addressed in the following color passes.
Scalp transitions, painted strand detail, character-specific bangs and moving
hair behavior remain unfinished. Generic faces, incorrect swimsuit/shorts
geometry, repeated rear costume details and garment bunching remain visible.
This pass improves silhouettes; it does not approve these complete characters.

The supplied-rig regression covers all sixteen gathers' finite normalized
normals, outward winding, closed seams/caps, face clearance, size bounds and
roots within the original head bounds. It passes. All 1,013 enabled renderer
library tests pass; eleven external-asset tests remain ignored in the default
suite. Temporary captures and logs were removed after review.

## Female forehead color pass

The human-female atlas's forehead hair extends through row 47. The procedural
recolor now includes that region for all 22 female appearances using it.
The four painted female textures retain their existing painting. Male fringe
recolor behavior remains unchanged.

All 26 female appearances were inspected at idle from four angles (104 views).
Janine, Kris, Misty and Whitney were also inspected at Run phase 0.25 from
four angles (16 views). Their forehead hair now follows the intended hair
color. Small brown temple patches below the corrected region remained visible
at this stage, as did blunt scalp transitions and repeated generic faces and clothes. This
is a color correction, not approval of the complete cast.

The supplied-atlas regression checks each affected appearance's hair pixels,
unchanged alpha, byte-for-byte preservation of the eyes, brows and mouth region,
and unchanged male fringe pixels. It passes. All 1,019 enabled renderer library
tests pass; twelve external-asset tests remain ignored in the default suite.
Temporary review images and logs were removed after inspection.

## Female temple color pass

Inspection of the actual source atlas located the remaining temple locks at
rows 48–55, outside the central face. The mask now includes those locks,
including the right edge at columns 100–102. It stops above the ear interiors
and leaves the central eyes and brows unchanged. All 22 procedural female
appearances were inspected at idle from four angles (88 views). The colored
temple patches now match the scalp in those renders. The four painted female
textures continue to bypass this procedural recolor.

The supplied-atlas regression now requires every dark hair pixel in the added
region to change and preserves the central upper face and the full face/ear
region below row 55 byte-for-byte. It passes, as do all 1,019 enabled renderer
library tests. Male fringe behavior remains covered by the same regression.
Long-hair curtains still have abrupt joins and blunt ends; generic facial
identity and inaccurate clothing remain unfinished. Temporary atlas extracts,
rendered captures and logs were removed after inspection.

## Rounded long-hair and bob pass

The six-ring hair curtain was replaced with a densely sampled monotone cubic
surface. Its upper rim starts inside the scalp, the cheek openings curve
backward, and the hem turns inward with small grouped tips. Subtle vertical
grooves break up the plain rear panel. The added hair uses a shaded palette
color closer to the source texture rather than the previous full-bright color.
The existing production Head attachment and caching remain in use.

Beauty, Bugsy, Clair, Daisy, Erika, Jasmine, Karen, Link Receptionist, Mom,
Receptionist, Rocket Girl and Sabrina were individually inspected from four
angles at idle and Run phase 0.25 (96 final views). Earlier versions folded at
the tip and had a conspicuously light panel; those versions were rejected.
The retained version has rounder ends and reduced color separation. The scalp
intersection remains visible, and repeated curtains do not yet express each
character's hairstyle. Bugsy's male source fringe remains brown beneath his
purple crown at this stage; the following male atlas pass corrects that mismatch.
Face identity, garment accuracy and moving hair remain unfinished.

The curtain regression checks finite positions, unit normals, outward main
surface winding, face clearance, connected rear coverage and closed edges.
It passes. All 1,021 enabled renderer library tests pass; twelve supplied-asset
tests remain ignored in the default suite. Temporary before/after captures,
contact sheets and verification logs were removed after inspection.

## Male forelock and sideburn color pass

The source male atlas has a central forelock through row 42 and temple locks
and sideburns continuing beside the face through row 61. These regions now
follow the selected hair color in 37 procedural male appearances. Bald and
sparse hairstyles keep their prior masks. Painted Chris, Super Nerd and
Youngster textures continue to bypass procedural recoloring.

All 37 affected appearances were inspected at idle from four angles (148
views). Bugsy, Falkner, Lance, Rival, Rocker, Surge, Will and Scientist were
also inspected at Run phase 0.25 from four angles (32 views). The broad brown
fringe now matches the crown, including Bugsy's purple bob and gray staff hair.
Small edge speckles, repeated forelock shapes, angular crowns, missing costume
details and generic stubble remain visible. The same adult face/stubble on
Bugsy, Bug Catcher, Gameboy Kid and Standing Youngster needs a dedicated face
pass. Captain and Fishing Guru still have oversized oval moustaches. This
color correction does not approve their full appearance.

The real-atlas regression checks the affected forelocks and sideburn samples,
unchanged alpha, and byte-for-byte preservation of the central facial features
and ear interiors. It passes. All 1,021 enabled renderer tests pass; thirteen
supplied-asset tests remain ignored by default. Temporary extracts, captures,
contact sheets and logs were removed after review.

## Child lower-face painting pass

Bugsy, Bug Catcher, Gameboy Kid and Standing Youngster no longer use the
shared adult stubble painting on their cheeks and chins. Production texture
creation blends the clean lower face from the already supplied human-female
atlas into the same UV region, preserving the male eyes, nose, colored lip
pixels, hair, clothing and alpha. Existing painted character textures bypass
this adjustment. No additional texture files or source copies are delivered.

A first version preserved a rectangle around the mouth and left a visible
moustache patch. It was rejected after close-up inspection; the retained mask
protects the nostrils and colored lip pixels instead. Each of the four boys
was inspected from four angles at idle and Run phase 0.25 (32 final views).
Their painted beard shadow is reduced, but the angular adult jaw, heavy chin
lighting, generic eyes and missing shorts/accessories remain visible. These
are still incomplete child models, requiring geometry and identity work.

The supplied-atlas regression checks clean chin samples, protected features,
unchanged alpha and clothing, and unchanged adult/painted identity inputs.
It passes. All 1,023 enabled renderer library tests pass; fourteen supplied-asset
tests remain ignored in the default suite. Temporary atlas extracts, rejected
and retained captures, close-ups and logs were removed after inspection.

## Child jaw geometry pass

Bugsy, Bug Catcher, Gameboy Kid and Standing Youngster now have tapered,
slightly shorter lower faces. Head triangles are refined twice before the
curved adjustment; jaw normals are accumulated across UV seams to reduce
hard lighting facets. Original texture coordinates and existing skin weights
remain intact, and new vertices interpolate the same skinning attributes.
The production body cache applies this adjustment once per appearance.

The first version shortened the chin too much and exposed an elongated neck;
it was rejected. The retained smaller adjustment was inspected for each child
from four angles at idle and Run phases 0.25 and 0.75 (48 final views). It
reduces the square adult jaw while keeping the existing hair and hat fits in
these samples. Repeated painted eyes, strong chin shading, angular scalp and
neck transitions, missing shorts and missing held accessories remain visible.
This is not approval of the four complete models.

The supplied-rig regression verifies finite geometry, unit normals, winding,
normalized skin weights, unchanged original UVs and skinning attributes,
unchanged non-head geometry, measurable jaw shortening, and unchanged adult
and trainer geometry. It passes. All 1,023 enabled renderer library tests
pass; fifteen supplied-asset tests remain ignored in the default suite.
Temporary rejected/retained captures and logs were removed after inspection.

## Shorts and bare lower legs pass

BugCatcher, Bugsy, GameboyKid, StandingYoungster, Youngster, Misty and Whitney
now render their assigned shorts. The production body preparation splits
trouser triangles at a mid-thigh hem derived from the source thigh and knee
joints. Lower legs reuse an interior sample of the existing arm skin island;
this also works with Youngster's supplied painting. No source painting is
rewritten. Shoes and upper clothing retain their original surfaces and UVs.
Cut vertices interpolate normals and normalized joint influences, and the
two hem surfaces share exactly the same positions and weights.

All seven were hand inspected in four GPU angles at idle and Run phases
0.25 and 0.75 (84 pose panels, focusing on lower-body continuity in the run
views). Shorts remain readable, with no visible hem or ankle holes in those
samples. The external-rig regression covers all seven, paired hem vertices,
finite normals, normalized weights, face winding, unchanged shoe triangles,
and unchanged adult appearances without shorts. All 1,023 enabled renderer
library tests pass; the new external shorts regression also passes.

This fixes the outfit boundary rather than completing character quality.
At this stage hems lacked physical fabric thickness, knees and calves retained angular
source silhouettes, the four procedural boys still have generic eyes and
heavy chin lighting, and several named accessories and sleeve lengths remain
unfinished. Temporary GPU captures and logs were removed after review.

## Curved legs and fabric hem pass

The same seven shorts appearances now have two rounds of curved subdivision
on their trouser/leg surfaces. Edge control points follow the source normals,
with curvature fading out near the shoes and upper garment. Original body
anchors, UVs and weights remain intact; clothing island seam normals are
averaged so duplicate vertices produce matching curved edges. The added
vertices retain normalized interpolated joint weights.

Each hem has a fitted 0.012-unit outward fabric lip fading into the garment
over 0.08 units, with an underside joining it to the unchanged skin surface.
An initial perimeter check found four tiny open edges caused by rejecting
small cut triangles. Retaining those slivers closes the fabric ring. The
regression now verifies that every welded edge in the hem plane is paired,
as well as finite normals, correct winding, normalized skinning and preserved
shoes and appearances without shorts.

Before/after lower-body comparisons were inspected for BugCatcher, Youngster
and Misty. The final seven were then hand inspected in all four lower-body
angles at idle and Run phases 0.25 and 0.75 (84 panels). Calf and knee
transitions are modestly smoother, hems read as separate fabric, and no hem
or ankle holes are visible in those samples. The source knee anchors still
make some bent silhouettes angular; this is a refinement, not a final model
approval. Faces, sleeve lengths, accessories, and map-wide readability remain
unfinished. All 1,023 enabled renderer tests and the supplied shorts test
pass. Own temporary captures, comparison sheets and logs were removed.

## Child face style pass

Bugsy, BugCatcher, GameboyKid and StandingYoungster now reuse the central
eyes, brows and expression from the already supplied Youngster painting.
The production texture cache applies this only to these four unpainted
appearances. Existing painted appearances bypass it, and the earlier clean
chin adjustment remains the fallback when Youngster's painting is absent.
Each child's hair, hat, ears, clothing and alpha stay intact; no source image
is rewritten or duplicated. Sampling follows normalized atlas coordinates
and interpolates across resolutions.

The first attempt made a visibly warmer central patch. It was rejected.
The retained version matches the borrowed painting to existing cheek colors
and widens the feathered transition around the face, while protecting the
hairline. Before/first/retained comparisons were inspected individually for
all four children. The retained faces were also inspected at all four angles
at idle and Run phases 0.25 and 0.75 (48 final face panels). Larger dark eyes
and readable expressions replace the blurry adult-style eyes in those views.
The four currently share the same borrowed expression; individual expressions,
dark jaw undersides, angular scalp/neck transitions and held accessories remain
unfinished. This is not approval of the complete character set.

The supplied-painting regression checks eye contrast and expression changes,
protected hair/ear/clothing samples, unchanged alpha and unchanged adult and
existing painted identity inputs. It passes. All 1,023 enabled renderer library
tests pass; seventeen supplied-asset regressions remain ignored by default.
Own temporary renders, comparison sheets and logs were removed after review.

## New Bark canopy pass

The layout-preserving imagegen reference in `map-concepts/new-bark-polish.png`
motivated a NewBarkTown-only scenery pass. New trees use rounded broadleaf
canopy groups, quieter tapered brown trunks and small folded leaf geometry.
Continuous interior groves use one larger tree per four source plots;
boundary plots keep individual smaller crowns within their blocked footprint.
Every source plot retains its original ground ownership and authored marker.
The original tree implementation remains in use on other maps.

An initial tall, narrow canopy version looked like a row of green columns in
the production render and was rejected. The retained shorter edge trees and
wider interior crowns were inspected with the local base pack at NewBarkTown
(8,8), zoom 0, orbit 0 and orbit 2. The central characters, lab and house
entrances and signs remain readable in those two views. This is a limited
scenery check, not verification of all camera positions or gameplay.

The retained silhouettes reduce thin-trunk noise but still have repetitive
lobes, sparse branch structure and a plain material finish. They do not yet
match the imagegen reference's foliage quality. Roof tiles, path transitions,
other towns and character readability across all maps remain unfinished.
The geometry regression covers finite unit normals, outward winding, stable
placement, ground anchors, boundary crown containment and interior density.
All 1,017 enabled renderer library tests pass. Temporary rendered captures
and logs were removed after review; the original concept images remain.

## Selected map target and leaf surface pass

The user selected `map-concepts/new-bark-polish.png` as the runtime target
("Aim for this"). A freshly built production renderer with the local base
pack was captured at NewBarkTown (8,8), zoom 0, orbit 0. The current game is
still visibly short of that reference: roof surfaces are flat grids, grass
edges remain abrupt, and canopy/trunk repetition is conspicuous.

Town canopy surfaces now use deeper green shadow values, twice as many leaf
placements, varied leaf sizes and domed eight-sided ovals with curved normals.
The first render had overly bright dot-like leaves and was rejected; the
retained lower contrast surface blends them into the crown. Boundary crown
cores reserve additional space for the wider leaves, keeping all foliage
inside the authored edge plots. The existing interior tree density and
world positions remain unchanged. Finite/unit normals, outward winding,
ground anchors, plot containment and stable placement regressions pass.
All 1,023 enabled renderer tests pass.

The retained runtime was inspected at orbit 0 and orbit 2 with the same
position and pack. Entrances, signs and the central player remain readable.
This is a modest surface refinement, not approval against the selected
reference. Rounded masses still look repetitive, leaf detail still lacks
natural clusters, trunks remain plain, and roofs/ground need substantial
work. Camera captures verify scenery appearance only, not gameplay.

The jaw investigation found normal skin colors at sampled lower-head UVs
and strongly downward source normals. A rounded ellipsoid jaw experiment
passed its geometry test but widened the dark band in the four children's
before/after views; it was rejected and reverted. Further jaw work remains.
Own temporary atlas inspection output, rejected/retained renders and logs
were removed after review.

## Curved ceramic roof pass

Selected New Bark and Route 29 images remain generated art targets, not screenshots
of this runtime. User specifically selected the forest lighting and floor:
dappled warm sun, dark canopy undersides, bright leaf tips, root undergrowth and
worn paths blending into grass.

Added `roof_tiles.rs` at the untextured authored town model import boundary.
Red and teal thin ceramic panel tops now have a smooth rolled crown and restrained
map-stable color variation instead of alternating high-contrast checker squares.
Original corners, closed lower faces, ridge/deck geometry, model placement bounds
and door anchors remain intact. Textured surfaces and other materials retain their
existing import path. An unpaired triangle is retained explicitly.

Reviewed actual New Bark GPU captures at (8,8), orbit 0 and orbit 2, zoom 0,
1280x960 requested viewport (2560x1920 readback), using the ignored known-good
browser pack. The first subtle sine crown did not read strongly enough; a wider
rolled crown was retained. The first dense grid exceeded connected-building
geometry budgets; the retained 4x2 subdivision reads similarly from the game
camera at a much lower cost. Roofs still need staggered overlapping courses and
more natural edge silhouettes to match the target. Foliage remains repetitive,
trunks too uniform, and floor/light detail far below the chosen forest reference.

Dedicated tests verify all three native house/lab ceramic palettes gain real
curvature while keeping original perimeter vertices, finite unit normals and
bounded growth. Unknown materials and lone ceramic triangles are preserved.
Full library verification after the final change: 1,027 passed, 17 ignored,
no failures. Temporary captures and logs are removed after evaluation.

## Route 29 floor and forest review

The previous turn made authoritative progress on ceramic roof geometry and
verified runtime captures. This pass follows the user's selected Route 29
forest reference, particularly the lighting and forest floor.

Fresh captures at Route29 (38,9), zoom 0, orbit 0 showed pale leaf cards and
thin striped trunks overwhelming the player. The initial broadleaf adaptation
removed that curtain but exposed rows of small crowns; a larger, overlapping
variant was visually evaluated, followed by pointed leaves. These candidates
were superseded by the shared worktree's newer `woodland_tree` implementation,
which now owns branching trunks, layered leaves, buttress roots and low shrubs.
That current implementation was preserved and checked in actual GPU captures
at orbit 0 and orbit 2. The player remains legible in both views. The canopies
remain too uniform and the repeated leaf silhouettes are still conspicuous;
this is progress toward the selected concept, not target-quality approval.

Retained this pass's Route 29 floor finish: a warmer grass palette applied after
existing verified path/lawn blending, plus map-anchored, multiscale variation
in canopy contact shading. Dense trunk contact remains dark; actual cast
shadows retain the production light path. No ground vertex displacement or
source occupancy edits were added. Ground shading gets finer subdivisions
near verified tree-shadow cells so the variation can appear without adding
fine geometry across the whole published map.

Regression coverage verifies route floor colors across a scrolling grid,
finite bounded colors, unchanged water/shore/stone materials, source-contained
trunks, mesh cutaway ownership, valid winding and repeatable route tree output.
The indexed shadow reference now carries the same route palette flag as the
production indexed evaluator. Six route checks pass. An earlier full run
passed 1,030 tests with 17 ignored; after concurrent scenery changes the latest
full run passed 1,029 and failed the Kanto shallow-bank test's exact geometry
comparison. Rechecking that specific test confirms it remains unresolved. No
clean full-suite claim is made for the current worktree.

The forest target still requires richer ground cover, fuller grass patches,
less regular crown silhouettes and more subtle sun/shade transitions. The
straight gray route strip and sharp terrace transitions also remain visible.
Task-owned GPU screenshots and logs were reviewed and then removed.

## Rear shirt atlas correction

Lass, Cooltrainer F, Nurse, Gramps and Super Nerd now use separate rear torso
UV vertices that sample plain fabric from their existing external paintings.
This removes repeated front buttons, zippers, collars and triangular shirt
openings from their backs. Front faces and every original position, normal,
color and skinning attribute are retained. No external texture is rewritten.
The same torso-width guard now protects Chris's arm UVs as well.

The first candidate painted fabric onto some bare rear forearms. Restricting
only the atlas region was insufficient because one forearm shares the shirt
island; those candidates were rejected. The retained version restricts both
the atlas and the T-pose torso width. All six affected appearances were
captured through the production GPU path at idle and Run phase 0.25, with
rear contact-sheet review for all twelve captures and detailed four-angle
moving inspection for all six. Bare arms and cuffs remain visible and the
repeated rear closures are removed in these sampled views. This is not a
complete animation or map review.

All 15 appearance tests pass with the local external rigs, including a new
whole-cast regression that checks the six intended corrections, unchanged
front and arm UVs, preserved geometry/skinning and unchanged unrelated
appearances. No full-suite result is claimed for this pass. Dark angular
jaws, generic outfits, incomplete hairstyle silhouettes, Nurse's waist
material transition and bunched moving skirt hems remain unfinished.
Temporary captures, atlas inspection images and task-owned logs were removed
following review. The full character goal remains active.

## Lower-face normal pass

The shared rig has chin normals with vertical components near -0.97, creating
an almost black lower-face band under the production southeast light. The
body upload now softens strongly downward normals within a smoothly blended
lower-head region. It retains a downward component and the original radial
direction. This is an intentional stylized shading adjustment; it does not
round the actual jaw silhouette or add indirect scene lighting. Every
position, triangle, UV, color, joint index and weight remains unchanged by
this pass. Eyes, forehead and non-head normals retain their previous values.
Geometry and materials still use the existing per-appearance GPU cache.

Captured the final production GPU skins for all 75 appearances at idle.
Front and three-quarter contact sheets were inspected for the whole cast,
including the shorter children. Twelve representative appearances were also
captured at Run phase 0.25 and inspected in front/three-quarter head views:
Chris, Kris, Mom, Gramps, Granny, Youngster, the four adapted boys, Chuck and
Outdoorsman. Chris, Mom and Gramps show a softer chin transition; painted
features and facial hair remain visible. Side lighting and actual angular
jaw geometry are still apparent. This sampling does not approve all frames
or the complete character art.

The current Rust map preview loaded the ignored known-good browser pack and
was inspected in PlayersHouse1F at (9,4), orbit 2, zoom 0. The player remains
readable at room scale. An earlier requested tile was rejected as unwalkable;
(7,4) overlapped an NPC, so that view was not retained as readability evidence.
The successful separated view is a limited art check, not gameplay proof.

All 16 appearance tests pass with the local external rigs. The new whole-cast
regression checks changed lower-head normals are finite, normalized, downward
and unflipped; geometry, skinning, upper-face and unrelated normals are
preserved. The supplied-rig bind/animated-envelope test and GPU instance
sharing/independent-animation test also pass. No full-suite result is claimed.
Temporary review captures and task-owned logs were removed after inspection.

The goal remains active: many NPCs still share generic faces and outfits,
several authored hairstyle silhouettes are absent, caps and hands remain
angular, and garment hems bunch during locomotion. These require further
individual modeling and map review.

## Sculpted scalp crowns

Black Belt, Blue, Brock, Cooltrainer M, Elm, Koga, Lance, Oak and Rival now have
modeled crown clumps; Surge has a flatter cropped top. These previously had
hairstyle names without corresponding geometry. The new Rust `hair_shapes`
pass refines and sculpts the supplied head skin, retaining its atlas and skin
weights. Roots remain part of that skin instead of using separate caps or
cones. Each appearance has a different crown shape; the larger styles have
individual clump layouts, height and sweep, while the smaller styles stay
close to the source head. Hats and every unrelated appearance bypass the pass.

Rounded first clumps looked like blobs and the first flat top was too tall;
those candidates were rejected. Narrower pointed clumps, a shorter flat top,
and separate clump layouts were retained after GPU review. Full head refinement
keeps crown boundaries from meeting an unrefined scalp edge. The jaw normal
pass runs before sculpting, preserving its original lower-face region rather
than expanding it with the taller hair. Per-appearance GPU bounds include the
maximum crown displacement in addition to the rig's animation envelope.

Inspected all ten final appearances from front, three-quarter, side and back
at idle and Run phase 0.25: 20 captures, 80 views. The roots are continuous
and the new peaks remain with the head in these sampled poses. Captured the
current production map viewer in ElmsLab at (4,4), orbit 0, zoom 0, using the
ignored known-good browser pack. Elm and the player remain readable; the
shorter Elm crown is subtle at room scale. This is limited art verification,
not gameplay proof or approval of every animation frame.

The external crown regression passes across the full 75-appearance catalog.
It checks ten distinct changed shapes, untouched unrelated meshes, preserved
original skin/face positions and UVs, finite unit normals, normalized valid
weights, no newly open scalp seams, nondegenerate triangles, a bounded vertex
budget and culling containment at eight sampled Run phases. All 16 existing
appearance tests also pass. No full-suite result is claimed for this pass.
Temporary renders, contact sheets and task-owned logs were removed after review.

These crown shapes are a first silhouette pass, not complete hairstyle approval.
Several still have generic front hairlines, weak short spikes and plain side
hair. Bill's curls and the side-part styles remain unmodeled. Distinct faces,
clothing, accessories and cleaner moving garment shapes remain required.
The full cast goal remains active.

## Reproduce the review

```sh
cargo build --locked -p crystal-voxel-view --features people-review --bin render_people_review
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  ./target/debug/render_people_review target/people-review
# Optional comma-separated appearances, then a normalized Run clip phase.
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  ./target/debug/render_people_review target/people-review-run beauty,mom,elm 0.25
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib costume -- --ignored
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib supplied_backpacks -- --ignored
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib supplied_glasses -- --ignored
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib supplied_tied_hair -- --ignored
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib supplied_female_fringe -- --ignored
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib supplied_male_hair -- --ignored
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib supplied_child_faces -- --ignored
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib supplied_child_face_style -- --ignored
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib supplied_child_jaws -- --ignored
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  cargo test --locked -p crystal-voxel-view --lib supplied_shorts -- --ignored
```

The reviewer freezes a requested run phase through the same `Person::update`
blend path used by gameplay and waits for actual GPU screenshot readback.
It continues processing when another window has focus. Review output is
scratch evidence under ignored `target/`, never shipped artwork.


### Connected grass and understory review — 2026-10-10

The selected forest concept remains the target, rather than evidence of the
current runtime. Actual Route 29 GPU readback at (38, 9), orbit 2 confirms that
adjacent authored grass cells now form continuous patches: blades fill the
shared margins and overlap the verified neighboring grass cells. Density doubles
inside connected patches, while outer edges remain contained beside paths.
Johto scenery, the generic outdoor grass path, and native ground bindings share
this geometry. Absolute source coordinates keep blade color, curvature, and
placement stable as the viewport scrolls. Collision and encounters are unchanged.

Understory now has taller fronds and connected stems around blocked tree plots.
The terminal grassy ledge cap and its front lip share the same slope through all
four source rows; the former full-height corner wedge is removed. Route 29 bank
faces use a warmer earth tint. These fixes improve joins and ground layering;
the uniform trunks and rounded canopy masses still fall short of the reference.

Shared-edge overlap, path containment at both detail levels, and scrolling tests
pass. Final verification: the example builds successfully; all 10 grass-filtered
tests pass; the full mesh suite passes 1036 tests with 19 ignored. An earlier
concurrent dungeon terrain failure was resolved in the checkout before the final
run. Runtime screenshots are temporary ignored review output, not shipped artwork
or gameplay verification.


### Route 29 canopy and bark review — 2026-10-10

The preceding goal turn made verified progress on connected grass, understory,
and ledge corner joins. This pass inspected current production geometry rather
than relying on the earlier woodland module mentioned above: that module is no
longer present; Route 29 uses the broadleaf implementation in `foliage.rs`.

Retained changes: source-anchored offset and bend in the trunk, a flared root
base, four tapered curved branches, and twelve staggered irregular canopy masses
instead of seven uniformly rounded masses. Longer leaf shapes rotate within the
canopy surface and use warmer sunlit greens over darker undersides. Using 160
leaves per route mass limits geometry growth while retaining visible leaf detail.
Town crown behavior remains on its existing profile.

Actual Route 29 GPU readback at (38,9), zoom 0, orbit 0 was compared before and
after; the retained version was also inspected at orbit 2. The former round spots now read as individual leaves, and the outlines
are less regular. The red player and the connected encounter grass remain clear.
The root and branch geometry stays within the blocked source plots; only crowns
overlap groves. No collision, encounter, pack, or character rig changes were made.

All six foliage checks pass, including winding, stable placement/color, cutaway
ownership, and the expanded bounds check covering branches as well as trunks.
The complete voxel-view suite passes 1036 tests with 19 ignored; the Bevy review
example builds successfully. Temporary review output is removed after inspection.

Remaining target gaps include repeated tree spacing, too little ground-cover
variation, hard rectangular terrace silhouettes, and the reference's much richer
dappled lighting. This review does not prove the broader every-character and
all-map quality goal complete.


### Whole garment cast waist fitting — 2026-10-10

The preceding goal turn improved and visually verified Route 29 crown, bark,
and leaf presentation. This pass returned to character geometry using fresh
production GPU captures of Chris, Kris, Mom, Nurse, Beauty and Receptionist.
Nurse and Beauty showed a skirt waist projecting away from the torso, with a
visible open-looking rim in side and rear views. Source rig inspection found
roughly equal Hips/Spine weights near that seam, whereas the garment's upper
ring had been forced back to Hips alone.

Added a renderer-only upper-garment fit against horizontal sections of the
supplied body triangles. The top ring sits over the body with real fabric
thickness, blends into the existing lower profile, and interpolates the body's
triangle skin weights. Upper seam transfer now continues through the top ring
instead of fading to a guessed pelvis attachment. Original body geometry,
textures, joints, clips and source content remain unchanged.

Inspected all 29 affected garments at idle from front, three-quarter, side and
rear views. Also inspected front/three-quarter views of every garment at Run
phase 0.25. Nurse and Beauty's projecting waist rim is reduced; the seam stays
close while the torso bends. Before/after Chris and Kris captures are pixel
identical. The actual PlayersHouse1F preview at (9,4), orbit 2 also keeps the
player and NPCs separated and readable. This is art evidence, not gameplay
validation.

Reviewed roster: Beauty, Blaine, Cooltrainer F, Daisy, Elder, Elm, Erika,
Falkner, Granny, Jasmine, Karen, Kimono Girl, Kurt, Kurt Outside, Lass,
Link Receptionist, Mom, Nurse, Oak, Old Link Receptionist, Pharmacist,
Pokefan F, Receptionist, Red's Mom, Rocket Girl, Sage, Scientist, Teacher,
and Twin.

The new supplied-rig regression measures each upper seam's distance from the
independently deformed body triangles at eight Run samples for all 29 garments;
maximum permitted bind-space distance is 0.035. It passes, as do the existing
bind/animated garment regression, all 12 ignored appearance checks, and GPU
asset-sharing/independent-animation checks. The full library suite passes
1036 tests with 21 ignored. Temporary task-owned output is removed.

Remaining defects observed: skirts bunch around the raised leg at Run 0.25;
several long robes expose dark angular shapes at the rear hem even at idle.
These lower-garment problems are not covered by the new upper-seam check and
remain unapproved. Generic faces and clothing, blocky hands, missing hairstyle
silhouettes, and uneven map quality keep the broader goal incomplete.


### Exterior material and terrace review — 2026-10-10

The active target is now outdoor rendering, measured against the selected
New Bark and Route 29 concepts. Actual production GPU readbacks at NewBarkTown
(8,8) and Route29 (38,9) were compared with those references. Concept images
remain targets rather than runtime evidence.

Retained changes: cooler, darker canopy undersides; warmer sunlit leaves with
individual normal shading; elongated town leaves instead of round studs; bark
grooves and a gentle trunk bend; darker grass roots with brighter tips; taller
understory; richer lawn variation and warmer earth paths. Authored foliage and
ceramic lighting now follow the production southeast sun. Roof courses have
more curvature samples and per-vertex lighting, making their rolled edges and
crowns visible in the palette shader.

Path/lawn transitions have centered, source-anchored edge variation instead of
an offset repeated field. Route 29 gains moss/soil variation and fine grain.
Verified level lawn/path terrace caps receive the same detail as the floor;
sloped support, original perimeter vertices and heights remain intact. Other
raised-map surfaces retain their previous geometry. Collision, source programs,
audio and packs are unchanged. Flattened town crown trials were rejected after
GPU inspection; the town retains its existing crown proportions.

Verification: the Bevy review example builds, the full voxel-view suite passes
1039 tests with 23 ignored, and all six foliage checks pass after rejecting the
flattened crown trial. The terrace regression checks cap boundaries and sloped
support. Actual Route 29 views were inspected at orbits 0 and 2; town views use
orbit 0. Temporary screenshots and logs are ignored verification output and are
removed after review. This does not establish gameplay correctness.

The rendering goal remains open. Tree placement and silhouettes are still too
regular, terraces retain angular silhouettes, some source-backed exterior
objects still render as atlas surfaces, and broad canopy shadows lack the rich
dappled light in the references. These are visible art gaps, not a claim of
perfect outdoor rendering.


### Closed garment drape and covered-body review — 2026-10-10

The previous goal turn made verified progress on fitted waist seams. This pass
started from fresh Elder, Erika, Nurse, Mom, Beauty and Sage captures at idle
and Run 0.25. Long robes had dark angular rear-hem patches; short skirts split
and bunched around the raised leg.

Closed garments now span the legs with a broader weight blend, retain a pelvis
contribution, and follow thighs without inheriting knee hinges. A continuous
waist-to-hem blend hangs the lower cloth from the source waist skin field.
An initial candidate still folded near the waist; the source-body weight
transition at each height was replaced by that common waist attachment. A
second candidate still folded in long lower panels; removing shin influences
resolved the 16-sample winding check without weakening its assertions.

Body surfaces wholly covered by closed skirts and robes are clipped once at
GPU upload against their shared hem height, preserving interpolated UVs and
skin attributes at the exposed-leg boundary. Original vertices and attributes
remain unchanged, and shoes, shirts, sleeves and hands retain their original
faces. Open coats and appearances without closed garments retain their body
surfaces and their prior lower-garment blend. No rig clips, source pack content,
controller behavior or per-frame mesh rebuilds were introduced.

Inspected the final 25 closed garments in side/rear idle sheets and front /
three-quarter sheets at Run phases 0.25 and 0.75. The former rear-hem angular
patches are absent in the retained captures; the fabric has a continuous lower
outline and gentler lift rather than trouser-like bunching. The raised shoe can
still meet the long robe opening abruptly, and garment detail remains generic;
this is not approval of perfect clothing or of every animation frame.

New supplied-rig coverage verifies that all 25 closed garments hide covered
leg faces while preserving exposed body, original skinning and shoes. Another
regression deforms both fabric layers at 16 Run samples for the whole closed
garment cast and requires every vertical panel to remain noncollapsed and
outward-facing. Horizontal rim caps share side normals and are excluded from
that normal-based panel assertion. The supplied waist-distance test also passes.
All 12 external appearance checks and the GPU sharing / independent joint
palette regression pass; the full library suite passes 1041 tests with 23
ignored. Task-owned captures and logs are removed after inspection.

Broader goal gaps remain: generic clothing identity and faces, angular hands,
remaining hair and hat details, richer cloth folds and hem/foot transitions,
and uneven map art / lighting. The goal stays active.


### Shallow ledge reconstruction review — 2026-10-10

The outdoor pass now prioritizes the ledges. Two separate defects were visible
in production: uniform rectangular earth faces, and whole-cell brown floor
stencils around the sides of the raised plots. Source-verified horizontal Johto
ledge border faces and caps now use the lawn finish at their original height. Upright shallow earth faces retain
their bank identity and gain a twelve-by-five relief surface, shaded soil layers,
short exposed roots, hanging grass and a grass fringe along the rim. The complete
face perimeter stays sealed at its original position and normal. Walking caps,
footing heights, jump programs and collision are unchanged.

A narrow soil-seam trial was rejected after GPU inspection because it looked
like a wavy line drawn on the ground. The retained version removes that stencil.
The refinement accepts only verified shallow Johto bank sources; tall cliffs,
Kanto banks, unknown reuse, water and other exterior objects retain their paths.
Temporary diagnostic tracing confirmed 84 upright Route 29 faces used the
refinement, then was removed from the code.

Verification: the production Bevy example builds and the full voxel-view suite
passes 1045 tests with 24 ignored. Regressions cover four-direction winding,
original support corners and normals, map-anchored colors under scrolling,
rejecting level floors as upright banks, and flat/raised side borders retaining
their footing while using their owned grassy verge rather than borrowed path
paint. Raised cap transitions now enter the blend field at their original tier;
a separate regression prevents a path/lawn blend from crossing a jump step. Final production review uses
Route29 (38,13), orbit 0/zoom 1, and (38,9), orbit 2/zoom 0, from the ignored local
pack. Screenshots and logs are temporary review output. These captures establish
visible rendering, rather than gameplay correctness. The broader reference art
goal remains open, including the regular terrain and canopy silhouettes.


## Shared hand surface review

Retained a gentle subdivision pass on source vertices weighted to the named
left/right hand subtrees. It rounds palm, finger and thumb silhouettes while
pinning the wrist transition. Geometric seams weld for shading; UV/color/skin
islands remain separate. Original non-hand geometry and attributes stay intact,
and the added vertices interpolate normalized skin weights. The pass runs once
inside the shared GPU mesh cache, with no animation or source rig changes.

Compared Chris, Kris, Nurse, Youngster, Elm and Outdoorsman before/after in front
and three-quarter idle views, then reviewed front/three-quarter hand crops for
all 75 appearances at idle and Run phase 0.25. The knuckle edges and curled-thumb
outlines are softer, without losing the grip cavity or showing a wrist split in
these views. Fingers still use the common stylized grip; this is improvement,
not approval of perfect hands or every animation frame. A fresh production GPU
capture in PlayersHouse1F also retains readable player and NPC silhouettes;
this screenshot is an art check, not gameplay validation.

The supplied hand regression passes on all four locally available source rigs,
including unchanged non-hand surfaces, pinned wrist boundaries, valid normals,
repeatability and finite skinning within the animation envelope at 61 samples
of each Idle, Run and Jump clip. All 12 supplied appearance checks, source bind /
animation-envelope coverage and GPU asset-sharing tests pass.

Updated older renderer checks to validate textured as well as solid surfaces,
actor ownership across imported hierarchies, and distinct remote materials on
bodies and accessories. The full suite now reports 1043 passed, 24 ignored and
one failure: tree LOD geometry is not reduced by over half. Inspection confirms
that the tracked external full and distant tree assets have identical geometry
(5450 vertices and 13788 indices each). The threshold remains intact; that
scenery issue is unresolved. Hand review captures/logs are temporary and removed
after inspection.

Broader gaps remain in individual clothing identity, faces, finer finger detail,
cloth/foot transitions and map lighting/materials. The goal remains active.


## Uniform caps and upper-lip hair review

Replaced the officer, captain and sailor's stacked ellipsoid hats with fitted
uniform shapes. The officer and captain have smooth crowns, contrasting bands,
curved attached dark peaks and shield-shaped gold insignia; the sailor has a
brimless white crown and narrow navy band. The source head attachment and shared
GPU cache remain unchanged. Crown surfaces use smooth normals and flat closing
faces; the peaks have upper, lower and edge surfaces rather than floating disks.

Inspected before/after front, three-quarter, side and rear captures for all
three caps, including Run phase 0.25. The crown/band/peak joins are continuous
in these views and do not obscure the eyes. The sailor remains a simple stylized
cap; this is not approval of complete staff uniforms. Nurse and Clerk were also
included as unchanged visual controls.

The captain comparison exposed the old moustache's detached oval appearance.
Replaced that shared shape with two upper-lip patches projected onto the actual
source face, keeping a small surface offset. An initial split candidate looked
like two separate ovals; the retained version has fuller roots, a narrow center
part and tapered outer ends. Inspected all seven moustached appearances (Captain,
Blaine, Fishing Guru, Gentleman, Gramps, Pharmacist and Pokefan M) from four angles
at idle and Run phase 0.25. The flat floating plate is absent; finer hair texture,
individual moustache styles and broader face quality remain open work.

Cap tests verify finite geometry, unit normals, winding and distinct styles.
The supplied-rig cap fit check validates head-local attachment, bounds and eye
clearance. The refined moustache check passes for finite geometry, valid winding,
unit normals, upper-lip placement and a small face-surface gap. All 12 supplied
appearance checks pass. The full library run reports 1045 passed, 26 ignored and
the same one failing tree LOD geometry-reduction check; its threshold remains
unchanged. These captures are art validation, not gameplay proof. Task-owned
captures and logs are removed after inspection.

The goal remains active. Individual outfit identity, collars, staff insignia,
face painting/scalp transitions, richer cloth details and map lighting/materials
still need work; these two accessory improvements do not establish perfection.

### Exterior daylight and leaf coverage review — 2026-10-10

The previous ledge turn made verified progress. The next current-state GPU
capture still showed a straight brown rim, pale canopies and sparse elongated
leaves on smooth cores. The user reaffirmed that the selected reference's
lighting and foliage are substantially better; the reference remains the target.

Shallow bank relief now has multiscale erosion, an uneven sod boundary and
continuous vertex lighting rather than per-quad shade jumps. Short, brighter
rim grass has irregular spacing. Original bank perimeters, corner normals,
walking caps, world placement and jump ownership are retained.

Route 29 crowns now use broader overlapping leaves around smaller internal
cores, with varied cluster heights and radii. Thicker branch forks replace the
previous thin branch sections; bark stays inside the original blocked plots.
New Bark receives more modest leaf broadening and core reduction, preserving
its existing crown proportions. Neither adds source foliage plots or changes
source texture pixels. The retained geometry has the same leaf count and
triangle count as before this foliage pass.

The canopy palette was adjusted for linear vertex color display, with darker
cool interiors and warmer leaf tips. Route29 and NewBarkTown materials select
warm direct daylight and deeper cool cast shade through the existing environment
uniform's Z channel. Cast visibility comes from production shadow geometry.
Other maps retain their previous shader response. The existing material cache
replacement test now crosses both exterior maps and Goldenrod after caves,
checking that daylight/reveal state resets and actor material state is retained.

Actual GPU views at Route29 (38,13), orbit 0/zoom 1, Route29 (38,9), orbit
0/zoom 0, and NewBarkTown (8,8), orbit 0/zoom 0 were reviewed. The first dark
palette exposed a polka-dot effect from sparse leaves; wider overlapping leaf
geometry was subsequently retained. This is still short of the reference:
canopy clusters remain too round, the grove retains regular source spacing,
trunks need richer bark and branching, and forest light still needs finer
leaf-shaped dappling. Render checks do not prove gameplay. Task review PNGs
and logs are temporary ignored output and are removed after verification.

Final current-state verification: seven foliage checks pass, including winding,
source-contained bark, stable generation and town edge plots. The full library
run passed 1,047 tests, ignored 27, and failed the separately edited Ice Path
`frozen_shelf_keeps_the_stair_opening_and_world_registered_faces` test. A fresh
isolated rerun confirms that stair-opening failure; it is not attributed to this
exterior pass, and no clean full-suite claim is made. Earlier in this turn the
library passed 1,047 tests with 26 ignored before that concurrent ice shelf test
was introduced. The retained town GPU view confirms leaf broadening; its
stacked crown silhouette remains an open art gap.
The retained ledge regressions pass (13 checks), as does the material cache
transition regression across caves, both daylight maps and Goldenrod. Final
diff whitespace checks are clean. All task-owned review PNGs/logs were removed;
source changes remain unstaged and the local content pack was not modified.


## Staff torso identity review

Added fitted, skinned uniform details for Officer, Captain and Sailor. Officer
has a blue collar, shirt placket, chest shield, pocket seam, buttons and belt;
Captain has a pale collar, gold rank strips and paired buttons; Sailor has a
navy V collar, red neckerchief and striped rear bib. Small panels are subdivided
and projected onto source torso triangles, borrowing barycentric skin weights.
Layered details have separate depths to avoid coplanar flickering. No source
body geometry, textures, rig clips or game content were replaced.

Initial captures exposed coplanar detail flicker, incorrect rear fits from a
shirt UV filter and abrupt ends of separate front/rear belts. Replaced the UV
filter with a bounded geometry intersection, and replaced the belts with a
continuous band fitted to horizontal source body sections. The existing section
sampler is now shared internally with the belt; its fitting calculation is
unchanged. The 29 dress/coat regression explicitly keeps its original garment
roster, with these three torso trims covered separately.

The animation check found rank strips and the outer sailor collar reaching the
source shoulder's sharp fold during Run. Kept those details within the stable
chest region; the signed clipping assertion remains unchanged. The retained
uniform regression passes for all three, checking every neutral face for valid
winding and every vertex's normalized skin attributes. It checks body distance
and signed clipping on every seventeenth trim vertex across 16 Run poses,
including both sides of the continuous belts. This sampled check is not proof
of every animation instant or of a perfect source shoulder.

Inspected front, three-quarter, side and rear captures at idle and Run phases
0.25 and 0.75. The separate belt ends and prior detail flicker are absent in
these captures. Collars, badges and bibs make the roles clearer; their forms
remain stylized and simpler than fully tailored clothing. All 12 supplied
appearance checks, GPU sharing / independent palette coverage and the unchanged
garment-waist distance check pass. The full library run reports 1047 passed, 27 ignored and the existing tree LOD
geometry-reduction failure. Captures are art checks, not gameplay proof.

Broader gaps remain in staff footwear, cuffs and fabric relief, individual
outfits elsewhere in the cast, source shoulders, faces/scalp transitions and
map lighting/materials. The goal remains active. Temporary review artifacts
are removed after verification.

### Leaf-shadow resolution review — 2026-10-10

The previous exterior turn changed authoritative geometry and materials. Fresh
production captures still showed horizontally banded New Bark leaves and broad,
blurred cast shadows despite the stronger daylight palette. The selected town
and Route 29 references remain the target, especially leaf-shaped dappling and
natural canopy silhouettes.

New Bark leaves now use stable varied rotations rather than all following the
same horizontal tangent. Route 29 internal crown cores shrink from 84% to 66%
of their outer leaf envelope, allowing gaps between the existing overlapping
leaves to transmit light. Leaf count, triangle count, source ownership, ground
support and cutaway ranges remain unchanged.

The native daylight maps now use three directional shadow cascades, with the
first ending at 256 world units and the last at 2,048. Other maps restore the
previous single 4,096-unit cascade. The browser path respects WebGL's single
cascade limit and narrows its range to 1,024; this turn did not execute a browser
render. The map-transition regression runs Route29, NewBarkTown, GoldenrodCity,
UnionCave1F and Route29 again, checking restored shadow settings. The original
2.5 normal bias is retained: an initial 0.8 trial produced diagonal shadow acne
on flat ground/windows and was rejected after actual GPU review.

Actual NewBarkTown (8,8), orbit 0/zoom 0 and Route29 (38,9), orbit 0/zoom 0
GPU views verify sharper leaf contours and a more broken canopy edge. The town
leaf banding is reduced. Route core reduction gives modest additional light
gaps, but tree silhouettes remain too regular, trunks remain too plain, and
reference-quality forest floor/dapple distribution is not yet achieved. These
are scenery checks, not proof of gameplay.

Current full library verification: 1,049 passed, 27 ignored, zero failures. The
previously reported Ice Path failures no longer reproduce in this current
worktree. The first new cascade test used exact float equality for a computed
far bound; it was corrected to a 0.001-world-unit tolerance. Temporary task GPU
captures/logs are removed after review; source remains unstaged.

### Footwear shape and cast review — 2026-10-10

The previous character turn changed the source shoe islands, extracted the
existing selective hand subdivision helper, and added role-based shoe colors.
This continuation completed the visual review and adjusted formal uppers from
near-black to charcoal: the first four-angle renders lost the rounded toe
highlight. Dark soles are retained. Colored sneaker uppers, brown outdoors
shoes, pale medical shoes and darker formal footwear now distinguish the cast
instead of the shared pale source shoes.

The production cached-body path applies one gentle subdivision pass only to
shoe vertices above the lowest sole and below the ankle rim. Welded geometry
across texture seams retains separate UV/skin attributes, with original ground
contact and ankle attachment positions pinned. Vertex tint preserves source
texture shading. Original rigs, animation clips and non-shoe islands remain
unchanged. This is a Rust mesh change, with no generated content pack or raster
asset added.

Review evidence: all 75 cast entries inspected from four angles at idle and at
Run phase 0.25; ten roles compared before/after and in motion; nine formal roles
recaptured at idle after the charcoal adjustment. No obvious detached ankle or
new shoe/leg intersection appears in these samples. This visual sample is not
an exhaustive animation or terrain-contact proof. The supplied footwear check
passes on human-male, human-female, skater-male and skater-female, preserving
original soles, ankle positions, UVs, skin attributes and non-shoe geometry;
it also checks finite winding/normals/weights and 61 running samples per rig.
The existing hand check, 12 supplied appearance checks and shared-GPU /
independent-palette check passed. The current full library run reports 1,048
passed, 28 ignored and one failure in
halo_foliage_lods_keep_exact_bounds_and_reduce_geometry_by_over_half. The tree
LOD assertion remains unchanged.

The shared shoe silhouette is still too generic for fully tailored formal,
hiking and dress footwear. Martial artists and swimmers still need suitable
feet/footwear and their full outfits reviewed. Faces, shoulders, scalp/hair
transitions, individual costumes and map materials remain unfinished. This
pass improves shape and readability; it does not establish that every model
is perfect. The goal stays active. Temporary review captures and logs are
removed after all owned processes finish.

### Jaw and cheek silhouette review — 2026-10-10

The preceding footwear turn changed authoritative mesh styling and completed
cast review. The next inspected issue was the long angular side/rear jaw edge
visible under staff hats and elderly hair. A new cached Rust head-surface pass
uses the existing welded selective subdivision to round cheeks and jaw edges.
The first draft pinned the whole front plane and left the chin too angular;
the retained version also rounds the chin below the mouth. Eye, nose and mouth
support vertices remain pinned, as do crown and neck attachment boundaries.
The original face UVs, colors and skin attributes stay intact. Child reshaping,
existing jaw-normal treatment and hair sculpting still follow this pass.

Source checks pass on all four supplied Kenney rigs, covering refinement,
limited displacement, exact original body and front-feature positions/normals,
unchanged original UV/color/skin attributes, valid winding and normalized
weights/normals. All 12 supplied appearance checks pass, and the real production
GPU sharing / independent joint-palette test passes when explicitly run with
its local-rig requirement. Two initial GPU test invocations selected no running
tests (one unmatched filter, one ignored test); neither is counted as proof.
The full suite run after the retained chin change reports 1,048 passing,
29 ignored and the unchanged tree LOD reduction failure. Other ongoing work
subsequently added tests; these counts describe that recorded run.

Visual evidence: eight four-angle baseline comparisons, all 75 final cast
heads examined at idle from four angles, and 11 final representative running
views (Officer, Captain, Chris, Kris, Beauty, Granny, Elm, Blaine, Bug Catcher,
Pharmacist and Gentleman). Side/rear jaw contours are softer; the improvement
is modest, and original front facial character remains. No obvious newly
detached glasses, hats, hair gathers or moustaches appears in these samples.
These are art views, not gameplay or exhaustive attachment-distance proof.

Many default faces still reuse blurry painted features, crown silhouettes
remain angular on several roles, bald scalp texture transitions remain jagged,
and common faces do not provide enough individual identity. Full outfit and
map polish also remain unfinished. The original goal stays active. All owned
review processes finished before removing task scratch images/logs.

### Rounded crown support review — 2026-10-10

The previous goal turn improved jaw contours and yielded concrete all-cast
render evidence. This turn addressed another visible source of blockiness:
linear crown refinement increased triangle density but retained the source
head's broad planar facets beneath styled hair. Before applying the existing
spikes, curls, flat top or swept part, the cached Rust sculpt now blends the
upper hair surface 80% toward its fitted ellipsoid. The blend fades to zero at
the existing scalp threshold; source skin pixels are excluded. Original face
positions and atlas/skin attributes remain unchanged by this crown pass.
Existing role-specific masses and culling-extension calculation are retained.

All 16 affected roles were compared before/after from four angles and reviewed
again at Run phase 0.25: Bill, Black Belt, Blue, Brock, Cooltrainer M, Elm,
Falkner, Koga, Lance, Morty, Oak, Rival, Super Nerd, Surge, Unused Guy and Will.
Pryce was also captured before/after as an unchanged control. The strongest
visible improvement is in the swept part and short-crown side/rear outlines;
pointed styles retain their distinct silhouettes. No obvious crown/face seam
opening or new glasses intersection appears in these views. The result remains
stylized and several points still look too regular to count as finished hair.

The supplied crown regression passes for the full source roster, with exactly
16 styled crowns and unchanged skipped models. It checks preserved source
face/body/atlas/skin attributes, distinct shapes, displacement and geometry
budgets, finite normalized attributes, closed welded upper scalp seams and
eight running samples per styled role within the expanded culling envelope.
The regression now also checks every triangle's winding against its shading
normal. All 12 supplied appearance checks pass. The recorded full library run
reports 1,049 passed, 29 ignored and the unchanged tree LOD reduction failure.
These renders are art checks, not gameplay proof.

Default face painting, individual facial identity, bald scalp transitions,
long hair, several costumes and map polish remain unfinished. The full goal
stays active. Owned review processes finished before removing this turn's
scratch captures/logs; no content packs or compiled artifacts were staged.

### Bald scalp atlas review — 2026-10-10

The cached Rust fallback texture pass now samples source forehead skin for bald
roles and projects a scalp/hair band through the supplied rig's head UV charts.
Nearest-chart gutter coverage prevents a dark center seam; softened source-color
coverage removes the remaining jagged painted forelock. Face features, ears,
body pixels and alpha remain protected. Externally painted textures bypass this
pass, including Gramps. No geometry, bones, UVs or gameplay paths changed.

Blaine, Elder, Pharmacist, Pokefan M and Sage were examined before/after at four
angles, then at Run phase 0.25; Gramps supplied an unchanged painted control.
The retained pass improves color consistency and removes the old forelock seam.
A pale forehead transition remains visible and the original bald cap still has
angular geometry; the hair band is too uniform to call these finished models.
No obvious newly detached glasses, moustaches or beards appeared in these views.

The supplied scalp regression passes for deterministic painting, changed scalp
pixels, protected face/ear/body pixels, preserved alpha and an unchanged Officer
control. All 12 supplied appearance checks pass. The latest full library run
reports 1,052 passed, 30 ignored and one tree LOD reduction failure. Two terrain
failures seen in an earlier run did not recur in this later shared-worktree run.
These are art and source checks, not gameplay proof. All owned scalp review
processes finished and scratch captures/logs were removed. The broader goal
remains active; the user's map imagegen request followed this review.

### Ground foliage and clarified UV repaint — 2026-10-10

Before the user clarified that the new map request meant character UV maps,
this turn addressed sparse native understory. Canopy-thinned interior tree
plots now retain their own ground vegetation and cutaway ranges; low broadleaf
rosettes fill between the existing ferns. Before/after Route29 and NewBarkTown
captures show fuller root-area ground coverage, while paths remain open. The
understory regression and all four foliage tests pass. A later full library
run reports 1,054 passed and 30 ignored with no failures; a transient missing
vector import in concurrently edited cave tests prevented an earlier build
but was resolved in the shared worktree before that successful run.

After clarification, work switched to the actual UV atlases, replacing Chris
and Super Nerd with built-in imagegen repaintings. See `character-uv-repaint.md`
for exact prompts, production-model review and validation. The goal remains
active: these two texture improvements do not finish all characters. All owned
review processes finished and temporary captures/logs were removed.

### Bob crowns and Red sleeve placement — 2026-10-10

The native crown pass now rounds the exposed bob crowns on Mom, Erika, Bugsy
and Receptionist, with modest role-specific swept volume. The source face,
body, UV charts and animation weights remain protected. Link Receptionist's
hat still skips crown sculpting. Mom's separate hair curtain now samples a
plain hair area of her installed painting, bringing its color closer to the
painted crown. Broad, restrained strand shading also applies to existing bob
and long curtains; curtain geometry is unchanged.

Red's painted short sleeves now use the white shirt island and his forearms
use the skin island. This duplicates selected complete source arm faces while
preserving their positions, normals, colors and skinning. The first trial used
mean-position face classification and produced a sawtooth hem; it was rejected
after rendering. The retained mapping ends at a complete source arm ring.
Red's sneakers also use a white upper tint so the installed white/red painting
is visible rather than receiving the generic brown vest-class shoe tint.

Mom, Erika, Bugsy, Receptionist, Link Receptionist and Red were compared from
four angles in Idle and Run phase 0.25. Red was captured again in both poses
after the hem correction. Beauty, Clair, Daisy, Jasmine, Karen, Rocket Girl and
Sabrina were also inspected in both poses because they share the curtain
shading. No new obvious hair detachment appears in those views. Long styles
still have generic faces, plain clothing and repetitive hair masses; these
characters need individual paintings and further silhouette work.

Validation passes: the full-roster supplied crown regression (20 styled
crowns), both painted torso regressions (11 painted roles plus an Officer
control, and Red's actual white/skin pixel sampling with preserved rig
attributes), and all 16 appearance regressions. Production review builds and
captures completed successfully. These are visual and source-attribute checks,
not gameplay proof. All owned processes finished before removing this pass's
temporary captures and logs. No new raster painting was generated in this
pass; the broader character goal remains active.

### Long-hair crown rounding — 2026-10-10

Extended the native source-scalp sculpt to the six exposed long-hair crowns:
Beauty, Clair, Daisy, Jasmine, Karen and Sabrina. It refines the source crown,
rounds its support surface and adds modest individual part/volume/sweep values.
The source face, body, atlas coordinates and live skinning stay protected.
The existing separate long curtains are unchanged. Rocket Girl's hat keeps her
outside this pass; before/after production captures have identical RGBA pixels
for that control.

Each of the six was compared before/after from four angles in Idle, then all
seven reviewed at Run phase .25. The crown outlines are smoother, especially
the top and rear. No obvious newly opened crown/face or crown/curtain gap
appears in these views. Lower hair still repeats a plain curtain shape, and
Beauty, Clair, Daisy and Sabrina retain blurry fallback faces and unfinished
clothing; this pass does not declare those models finished.

The supplied full-cast crown regression passes with 26 distinct styled crowns.
It verifies protected face/body geometry and atlas/skin attributes, closed
welded scalp seams, winding, finite normalized attributes, displacement and
geometry budgets, and eight running samples inside the culling envelope.
Production reviewer build and captures succeeded. No new raster assets or
gameplay changes were needed. The broader goal remains active.
All 16 supplied appearance regressions also pass. Owned build, render and test
processes finished before this pass's scratch captures and logs were removed.

### Painted hair on separate curtains — 2026-10-10

Bob/long hair curtains now read filtered vertex albedo from the installed UV
painting's left hair strip, outside the facial/ear islands. Previously the
entire curtain used an averaged hair color with a second dark palette shade;
this made it look plain and mismatched against the painted source crown.
The sampling follows the hanging hair with a small smooth bend. A first trial
traversed the horizontal atlas waves vertically and produced stacked bands;
that version was rejected after production GPU inspection.

This affects the five currently painted bob/long roles: Mom, Jasmine, Karen,
Beauty and Daisy. Unpainted roles retain the existing color path. Positions,
normals, indices, UVs, seams and head-bone attachment are unchanged. No raster
asset was edited or generated in this pass.

Reviewed all five final models at four angles in Idle and Run phase .25.
The retained version removes the stacked bands and brings the curtain color
closer to the crown, particularly on silver-haired Karen and golden-haired
Daisy. Clair's unpainted final Idle control remains pixel-identical to before.
All 17 appearance regressions pass, including a new sentinel-island check for
hair-only sampling, preserved exact curtain geometry and nonconstant finite
albedo. Final production reviewer build and captures succeeded. All owned
processes finished before removing scratch captures/logs.

This is a color continuity improvement. Curtain surfaces remain too plain,
their upper joins are still visible, and their repeated lock geometry needs
further individual modeling and texture placement. The full goal is unfinished.

### Hair curtain ends and lock volume — 2026-10-10

Reshaped the native closed bob/long curtain surface with broader flowing lock
relief, inward taper at the last rings and gently varied tip heights. The old
nearly horizontal, cylindrical end looked like a separate slab. The new end
has restrained curves rather than pointed spikes; bobs use a smaller variation
than long hair. Root profiles, face opening, closure topology, color sampling,
attachment and painted atlases remain unchanged.

Reviewed all ten affected roles individually at four angles in Idle and Run
phase .25: Mom, Erika, Bugsy, Receptionist, Beauty, Clair, Daisy, Jasmine, Karen
and Sabrina. Compared Mom and Clair directly to the captured prior geometry.
The long ends are less blunt, especially in rear views; bobs turn inward
without opening the closed shell. No new face occlusion or shoulder clipping
was observed in these captures. A numeric comparison confirms all ten Idle
images retain identical pixels below the hair/shoulder region. All 17
appearance tests passed, including closed edges, finite unit normals, surface
winding and painted hair sampling. Reviewer build and diff check passed.
All owned processes finished before removing scratch review output.

This improves the shared surface only. Crown joins, plain side shading and
repeated hairstyle silhouettes remain visible, and Erika, Bugsy and the
Receptionist still lack individual painted faces. They are not finished models.

### Erika's fitted obi — 2026-10-10

Added a native closed ivory-gold sash and a soft flattened rear bow to Erika's
production costume. Sash radius comes from horizontal body sections, smoothed
across neighboring directions while retaining clearance over the waist seam.
Inner/outer surfaces and top/bottom edges form a closed solid. The bow has
rounded flattened loops and a central knot, fitted against the sash.

Body triangle influences supply the sash weights. At the bottom of the sash,
the first version inherited leg weights from source waist triangles; the
binding regression caught this. Transferred those contributions to Hips while
retaining Spine/torso weights. Bow components share one rear torso binding to
avoid differential pulling between loops and knot. The initial render's sharp
source-polygon corners and overly plump bow prompted smoothing and flattening.
Painted UV assets, head, sleeves and underlying skirt remain unchanged.

Hand-reviewed final production captures from four angles in Idle and Run phases
.25 and .75. The band fits the waist and the flattened bow stays against its
rear surface in both raised-leg poses. The dedicated supplied-rig regression
passes closed-edge counts, winding, finite unit normals, normalized weights,
body clearance and absence of leg-joint contributions. The existing costume
regression passes all 29 profiles, preserving bind shape and checking eight
Run samples per profile. Final reviewer build and diff check passed. Owned
review captures and logs were removed after all processes finished.

The bow remains a simplified folded shape, and the sash has plain color rather
than authored fabric details. Erika still needs wide kimono sleeves, the hair
flower and better crown joins. This does not finish her model or the full cast.

### Erika's hair flower — 2026-10-10

Added five closed rounded ivory/blush petals, a gold center and two small green
leaves against the side hair, through the existing head attachment path. The
flower scales with the head and receives the same head bind transform as the
hair. Existing head geometry and painted UV assets remain unchanged.

Hand-reviewed the production model from four angles in Idle and Run phases
.25 and .75. The flower stays above the eyebrow, clear of the eyes and against
the hair in these captures. Mom's unaffected Idle control remains pixel-identical
to the prior capture. Two flower regressions pass closed edges, winding, finite
unit normals, side-hair placement, and exact preservation of original hair
positions, normals, colors, UVs and indices in head bind space. All 17 appearance
regressions also pass. Reviewer build and diff check passed; owned scratch
captures/logs were removed after all processes finished.

The flower is a simplified sculpt with plain petal color. Broader sleeves were
subsequently added below; crown joins remain unfinished.

### Erika's broader sleeves — 2026-10-10

Broadened the lower sleeve fabric with a smooth drape and one gentle selective
subdivision pass. Shoulder and wrist boundaries remain pinned; exposed hands,
original UVs and original skin weights are preserved. Only Erika receives the
shape change. Four-angle Idle and Run .25/.75 production captures show a fuller
lower sleeve without new visible hand or torso clipping in those poses. Mom's
control capture remains pixel-identical. The supplied sleeve regression and all
17 appearance regressions pass, including eight Run samples for all 29 costume
profiles. Reviewer build and diff check pass.

The sleeves still use the shared arm construction rather than fully authored
kimono folds. The crown join and simplified sash/bow remain visible limitations.
This does not finish Erika or the full cast.

### Relaxed visible hand poses — 2026-10-10

Close inspection of the production hand showed that much of the C-shaped hook
came from the supplied Idle/Run finger rotations: the middle index joint curls
nearly 90 degrees, with further curl at the tip. Kept the existing rounded hand
mesh and introduced a presentation-only pose adjustment. Optional named index
and thumb nodes are discovered in the source rig; wrists and end markers are
excluded. At spawn and after each visible Idle/Run sample, finger rotation keeps
40% of its sampled deviation from the source bind pose. Translations, scales,
wrists and every other joint remain untouched. Original animation programs,
source meshes, UV paintings and gameplay state are unchanged.

Hand-reviewed four-angle production wrist/hand contact rows for all 75 roles
in Idle and Run .25, and full baseline/after captures for Misty, Chris, Kris,
Gramps, Youngster and Officer. The large curled hook reads as a looser palm and
finger silhouette; no new wrist separation was observed in these sampled views.
Broad mitten fingers and simplified anatomy remain visible. Six upper-half
control crops are pixel-identical before/after, preserving head/shoulder art.

The first tight culling regression caught human-male Run phase 27 outside the
old envelope. Fixed the production rig bounds by unioning the original Idle,
Run and Jump envelopes with 61 samples of the relaxed Idle/Run poses. Retested
against the actual .02 renderer padding, using the smoothed hand mesh. The
regression passes all four supplied rigs, preserving every non-finger rotation
and all translations/scales, with finite unit rotations, reduced curl and
finite deformed vertices inside the updated bounds. Source rig/invalid-input
checks, hand-surface preservation checks and the production GPU instance test
pass (five targeted tests total). The instance test confirms the visible pose
on actual spawned finger entities and isolation from the other actor. Its first
exact translation assertion exposed only floating point interpolation rounding
and was changed to a 1e-6 bind-space distance tolerance. Reviewer build and diff
check pass. Owned scratch renders/logs were removed after all processes ended.

This is a shared hand-pose improvement, not finished hand anatomy. Separated
finger silhouettes, thumb shape, costume identity and the remaining individual
UV paintings still need work. The full character objective remains active.


### Janine UV and crown review — 2026-10-10

Janine now has an individual painted atlas with a focused face, plum hair, red
scarf and overlapping ninja tunic. Rust maps the front artwork separately from
plain rear cloth and rounds the exposed high-ponytail crown while protecting
the hairline, face and body. Ponytail color comes from the painting's hair-only
strip; its attachment and shape remain unchanged. Actual four-angle Idle and
Run .25 captures were hand-reviewed. UV geometry/weight preservation, closed
crown seams and 12 supplied appearance regressions pass. See
`character-uv-repaint.md` for source, hash and full generation prompt. Grey
heels/soles, simplified hand/neck anatomy and absent gloves remain unfinished.
The full cast is not complete.


### Koga texture and ninja heel review — 2026-10-10

Koga now has an individual mature face, layered charcoal hair, navy ninja
tunic and red scarf. Separate front/rear mapping removes the false rear white
waist band. Painted shoes retain their own upper palette. Koga and Janine's
upper heels now fade darker where source UVs sample the pale shoe gutter,
while preserving grey soles and front panels. Actual production four-angle
Idle and Run .25 captures of both roles were hand-reviewed. Three UV tests
pass; the existing four-rig footwear regression passes with its Officer
control. Reviewer build and diff check pass. Full imagegen prompt, selected
source and hash appear in `character-uv-repaint.md`. Missing wrist wraps,
simplified hand/neck anatomy and plain rear cloth remain unfinished. The
75-role objective is not complete.


### Morty texture and fitted headband — 2026-10-10

Morty has an individual grey-green-eyed youthful face, painted swept blond
hair, plum shirt/lavender scarf and khaki trousers. Separate front/rear UVs
remove the false white waist band, and a role-specific .595 front UV limit
keeps khaki from bleeding onto the shirt edge. Dark upper heel correction
preserves grey soles. Added a closed native cloth headband, fitted to the
production head surface by radial intersections, attached through the inverse
head bind above the eyes. The new supplied regression verifies closure,
consistent winding, finite normals, thickness, bind attachment, eight Run
bounds checks and a Koga no-op control. Three supplied UV regressions pass.
Hand-reviewed actual final four-angle Idle and Run .25 captures, reviewer
build and diff check pass. Source/hash/full imagegen prompt are recorded in
`character-uv-repaint.md`. Plain headband without ties, rear fabric and
simplified hand/neck anatomy remain unfinished; full cast remains active.


### Falkner UV and draped sleeves — 2026-10-11

Falkner has an individual youthful face, painted navy swept hair and blue
kimono lapels/ivory collar. Separate front/rear mapping protects the clothing
islands; lower garment samples plain painted fabric. Shoes preserve their
painted upper colors with darker heels. Native kimono sleeves now include
Falkner, with deeper hanging cloth and less depth than Erika's profile.
Both supplied rigs pass skin/UV/weight preservation and eight Run sample
checks; Mom remains a no-op control. Three UV regressions and the all-profile
costume animation regression also pass (five total). Actual final four-angle
Falkner Idle/Run .25/Run .75 and Erika Idle/Run .25 controls were hand-reviewed.
Reviewer build and diff check pass. Full prompt/source/hash are recorded in
`character-uv-repaint.md`. The following review adds the missing obi; plain
lower garment and simplified hand/neck anatomy remain unfinished. Full cast
remains active.


### Falkner fitted sash and knot — 2026-10-11

Falkner now wears a closed ivory obi with a compact rear knot and short ends.
Its .24 waist band fits the supplied male torso and transfers thigh influences
to the pelvis while retaining torso influences. Erika's existing broad bow
and .29 band remain unchanged; her before/after Idle captures are pixel
identical. Expanded supplied obi closure/fit regression covers both rigs; the
29-profile costume animation regression passes eight Run samples per garment.
Reviewed actual four-angle Falkner Idle/Run .25/Run .75 captures with fitted
waist and knot. Build/diff check pass. Qualified a Vec2 expression in the
shared opt-in renderer diagnostic to resolve its compile error. Plain lower
cloth, rear fabric and simplified hand/neck anatomy remain unfinished; full
cast remains active.


### Lance atlas and shorter rounded crown — 2026-10-11

Lance has an individual amber-eyed youthful face, painted crimson hair and
tailored jacket with dark/gold collar and small buttons. Initial generated
grey shoulder/shirt-edge strips were repaired with builtin imagegen before
installation. Separate front/rear UVs keep front details out of rear fabric.
Shoes retain painted uppers and darkened heels. The native hair rise is .42
instead of .66, with .50 cosine clump support and reduced rear sweep, rounding
the previously long conical peaks. Source hairline, face/body UVs, influences
and protected geometry remain intact. Actual final four-angle Idle/Run .25
production captures were hand-reviewed. Three UV and one all-cast crown
regression pass (29 sculpted crown profiles); reviewer build/diff check pass.
Prompt/source/hash/repair details appear in `character-uv-repaint.md`. The
missing cape is resolved in the following pass. Proper boot shafts, plain rear
fabric and simplified hand/neck anatomy remain unfinished. Full cast remains active.

### Lance fitted cape — 2026-10-11

A native closed cape now wraps Lance's rear shoulder line and falls to the
knees with shallow folds and a scalloped hem. Crimson outer cloth and a darker
lining use vertex colors; the painted atlas is unchanged. Chest/hip weights
keep the cape attached during running without leg or arm influences. Its
1,254 vertices include both surfaces and closed boundary caps.

The supplied-rig regression passes closure, winding, nondegenerate faces,
unit normals, normalized weights and the production costume bounds at 61
Idle and 61 Run samples. The shared GPU costume asset/independent pose
regression also passes using Erika as its control. Actual four-angle Lance
Idle, Run .25 and Run .75 captures were hand-reviewed: shoulder fit, folded
back, darker lining and raised-leg clearance look clean. Koga's Idle control
is pixel-identical to the baseline. Reviewer build and diff check pass.
Owned verification scratch was removed after its processes ended.

This resolves Lance's missing cape, without changing the 27 painted-atlas
count. The cape has torso skinning rather than cloth simulation. Proper boot
shafts and simplified hand/neck anatomy remain unfinished; the full cast
objective remains active.

### Rival atlas and rounded crown — 2026-10-11

Rival now has an individual youthful hazel-grey face, painted crimson hair
and a charcoal navy jacket with red piping and zipper. Separate front/rear
UV mapping removes the pale baseline lower-back stripe and avoids repeating
the opening behind him. Painted shoe uppers retain their intended color with
darker upper heels. Crown rise .56 to .40 and .50 cosine clumps reduce the
long conical peaks while preserving his swept outline and original hairline.

Actual four-angle production Idle and Run .25 captures were hand-reviewed.
Three supplied UV regressions and the all-cast crown regression pass; reviewer
build and diff check pass. Prompt/source/hash are in character-uv-repaint.md.
Owned verification scratch removed after all processes finished. This brings
the individual painted atlases to 28. Simplified hands/neck, plain rear fabric
and game-scale review remain unfinished. Full 75-role goal remains active.

### Blue atlas and rounded crown — 2026-10-11

Blue has an individual confident brown-eyed face, painted chestnut hair, a
purple woven shirt with gold pendant, navy denim and charcoal shoes. Separate
front/rear mapping removes the pale waist stripe and keeps the necklace on
the chest. Native crown rise .64 to .42 with .50 cosine support rounds the
previous tall peaks while retaining the forward sweep and original hairline.

Actual four-angle Idle and Run .25 production captures were hand-reviewed.
Three supplied UV regressions and the all-cast crown regression pass; reviewer
build and diff check pass. An existing cave helper needed crate::mesh visibility
for its sibling callers; no behavior was changed. Prompt/source/hash are in
character-uv-repaint.md. Owned scratch removed after processes finished.
There are now 29 individual painted atlases. Simplified hands/neck, plain rear
fabric and game-scale review remain unfinished; full 75-role goal active.

### Brock atlas, sleeves and rounded crown — 2026-10-11

Brock has a distinct narrow-eyed friendly face, textured brown hair, painted
olive vest/pockets over an orange shirt, brown canvas trousers and hiking
shoes. Imagegen repaired a thin generated grey shirt border before installing
the selected atlas. Separate front/rear UV mapping removes the pale waist
stripe and avoids rear pockets. Complete source arm-ring mapping gives orange
short sleeves and bare forearms, with a Brock-only warmer skin patch below
finger artwork and above the pants. Geometry, normals and influences remain
unchanged. Native crown rise .36 to .30 and .50 cosine clumps round the peaks
while retaining his short spiked outline. Existing hiking pack is retained.

Final actual four-angle Idle and Run .25 captures were hand-reviewed after
correcting the initially pale forearms. Three supplied UV regressions and the
all-cast crown regression pass; reviewer build and diff check pass. Prompt,
repair, source and hash are in character-uv-repaint.md. Owned scratch removed
after processes finished. There are now 30 individual painted atlases. Palms
remain lighter; simplified wrist/hand/neck anatomy, separately layered vest,
plain rear fabric and game-scale review remain unfinished. Full 75-role
objective remains active.

### Elm atlas and plain coat sleeves — 2026-10-11

Elm has a distinct clean-shaven adult face, warm brown eyes, textured tousled
hair and painted lab coat lapels, teal tie and chest pen. The generated grey
upper-lip patch was removed with builtin imagegen before installation. Native
glasses are retained. Separate front/rear UVs keep details on the chest; an
Elm-only sleeve mapping resolves the repeated upper-arm pen found in the first
render. Lower coat color samples the painting's plain white fabric. Crown
rise .30 to .25 and .50 cosine support round the short peaks.

Actual final four-angle Idle and Run .25 were hand-reviewed. The existing
coat waist join remains uneven and pulls away at the front in Run .25; this
is a geometry/weight issue still requiring repair. The color match alone
does not approve the coat. Four supplied UV tests, the all-cast crown test
and 29-profile costume animation test pass. Their bind/weights/bounds checks
do not establish cloth/body clearance. Reviewer build and scoped diff check
pass. Global diff check found unrelated EOF whitespace in sculpt_geometry.py.

Prompt/repair/source/hash are in character-uv-repaint.md. Owned verification
scratch removed after processes ended. There are now 31 individual painted
atlases. Coat join/overlap, simplified hands/neck, plain lower cloth and
game-scale review remain unfinished; full 75-role goal active.

### Fitted coat shading and garment rim winding — 2026-10-11

Elm's lower coat now rebuilds normals from its fitted geometry. The original
profile normals described the pre-fit waist slope and produced dark angular
patches in the running view. A new supplied-mesh regression also exposed
inconsistent winding on the profile shell's top, bottom and open-front caps.
Those cap triangles are reversed for the shared garment builder, leaving
positions, UVs, colors and weights unchanged. This affects all 29 authored
coat/skirt/robe profiles. The fitted-normal change applies only to Elm.

Extra clearance and a higher join were rendered and rejected; neither fixed
the waist transition. Their geometry changes were removed. The retained
changes pass Elm's finite/unit/outward normal and closed oriented-edge check,
and the shared profile test now verifies closed oriented edges after welding
the full-circle seam for both open and closed shells. The 29-profile bind/
weight/animation/bounds regression passes eight Run samples per garment.
Three targeted tests pass; reviewer build and scoped diff check pass.

Reviewed all 29 lower garments in actual four-angle Idle and Run .25 captures
(58 captures, 232 views), using labeled crop sheets. Also reviewed Elm's full
four-angle Idle/Run .25/Run .75. Rim surfaces remain intact and Elm's shading
is smoother. His front waist transition remains uneven in running and needs
a continuous join/overlap solution; this pass does not approve his coat.
Blaine, Oak and Scientist retain similar fitted-waist transitions, and the
cast's plain cloth and individual costume details remain unfinished.

A disk-full build failure interrupted verification; source integrity was
checked and existing test/build handles were confirmed terminal. Space became
available before retrying. All later checks finished successfully. Owned
review scratch/logs removed after owned processes ended. No atlas changed;
31 individual painted atlases remain installed. Full 75-role goal active.

## Oak UV repaint — 2026-10-11

Following the user's character-UV clarification, repainted Oak's actual
fallback atlas with builtin imagegen. A distinct older clean-shaven face,
silver hair/brows, cream lab-coat lapels over a brick-red shirt, chest pen,
tan canvas trousers and brown leather shoes replace his generic texture.
Full generation prompt/source/hash are in character-uv-repaint.md.
Torso front/rear and plain-fabric sleeve mapping keep front details off arms
and back while preserving positions, normals and skin influences exactly.
The lower coat's color samples the same painting. No shape changes.

Hand-reviewed actual four-angle Idle and Run .25. The texture reads clearly;
the existing coat waist gap in motion and simplified hand/neck anatomy are
still unfinished. Four supplied UV regressions, reviewer build and scoped
diff check pass. Owned scratch removed after all own processes completed.
32 individual atlases are installed. Full 75-role goal active.

## Scientist UV repaint — 2026-10-11

Repainted the actual fallback atlas with builtin imagegen: distinct older
clean-shaven face, short ash-grey hair, white lab-coat lapels over a pale blue
shirt, small chest pen, dark brown trousers and charcoal-brown leather shoes.
Rejected the initial blue coat-border/grey hem artifacts and used a targeted
builtin cleanup. Full prompts/source/hash are in character-uv-repaint.md.
Front/rear/plain sleeve mappings and lower-coat color matching reuse the
existing lab-coat path; no geometry or skin-weight changes. Separate glasses
remain native.

Hand-reviewed actual four-angle Idle and Run .25. The texture reads clearly
and front details stay off sleeves/back. Coat waist gap in Run, simplified
hands/neck and game-scale review remain unfinished. Four supplied UV tests,
reviewer build and scoped diff check pass. All owned processes finished before
scratch cleanup. 33 individual atlases installed; full 75-role goal active.

## Blaine UV repaint and four lab-coat collars — 2026-10-11

Builtin imagegen repainted the actual fallback atlas with a distinct older
face, bald scalp/sparse silver side hair, white coat lapels, ivory shirt and
burgundy tie/trousers. A targeted builtin repair removed initial rectangular
hair blocks and a grey shoulder stripe. Separate moustache/glasses remain
native. Full prompts/source/hash are in character-uv-repaint.md.

Hand-reviewed Blaine's actual four-angle Idle/Run .25. That exposed colored
side-collar tabs: source collar triangles cross the original shirt UV boundary
and missed the front mapping. A whole-face plain-fabric correction now covers
Elm, Oak, Scientist and Blaine without changing positions, normals or skin
influences. Reviewed all four in final Idle/Run .25, four angles each. The
colored tabs are gone; central shirt/tie details, plain sleeves and rear are
retained. Five supplied UV regressions pass, including a new plain collar/
unchanged skin/weight regression and Blue no-op control. Build and scoped
diff check pass. Owned scratch removed after all owned processes completed.

Blaine's angular crown, simplified hands/neck and all four coat waist gaps
remain unfinished; this pass does not approve those models. 34 individual
atlases installed; full 75-role goal active.

## Exposed bald crowns — 2026-10-11

Native upper-scalp refinement now applies to hatless bald Blaine, Elder,
Gramps, Pharmacist, PokefanM and Sage. `head_surface::bald_crown` selects only
head-weighted vertices above center.y + .35*half.y and runs two selective Loop
passes using the established .55 support. Lower face, eyes/nose/mouth, ears,
neck and accessory support remain fixed. Original UVs/colors/influences stay
exact; added vertices interpolate UVs and normalized skin influences. The
source renderer AABB is retained and verified, rather than expanded to hide
an out-of-bounds mesh. No painting or generated asset changed.

Hand-reviewed four-angle baseline/Idle/Run .25 head captures for all six.
Crown corners and top silhouettes are rounder, especially Blaine's exposed
scalp and Gramps's receding cap. Existing glasses, moustaches and Elder's
beard retain their fit. Scientist's full four-angle Idle render is pixel-
identical to the baseline no-op control. Two supplied head regressions pass.
The new regression covers all six and Scientist, fixed lower face/body
attributes, cap motion below .10, finite unit normals, winding and valid skin
influences, plus 61 Idle and 61 Run samples inside the production .02 AABB pad.
Reviewer build and scoped diff check pass. Owned scratch removed after all
owned processes completed.

The four unpainted bald roles still expose jagged/patched scalp hair and brow
painting; refinement makes that fallback defect more apparent, not repaired.
They need individual UV repainting. Crown transitions remain mildly angular,
lower head/neck/hand anatomy and clothing remain unfinished, and this does
not approve those models. Atlas count remains 34. Full 75-role goal active.

## Pharmacist UV repaint — 2026-10-11

Builtin imagegen repainted the actual fallback atlas with a distinct kind
older face, continuous bald scalp/sparse silver side hair, ivory stand collar,
pearl buttons, subtle cotton folds and dark leather shoes. A targeted builtin
repair removed rectangular hair/scalp patch outlines and grey garment borders.
Full prompts/source/hash are in character-uv-repaint.md. Existing torso/plain
sleeve/collar mappings include this white robe; original geometry and skin
influences remain exact. Lower robe color samples the same painting. Native
glasses/moustache and prior crown refinement remain.

Hand-reviewed four-angle Idle/Run .25 and retained opposite Run .75. The grey
hair band and generic beard are gone, while collar/buttons stay on the front.
Forehead shading still has patches. A geometric-normal blend was tried on all
six bald heads in Idle/Run .25; it did not remove those patches and was removed
with its test changes. The four-angle retained Run .75 was reviewed after the
rollback. Lower robe remains plain, its waist join/hem movement need work,
and simplified hands/neck and missing individual accessory details remain.
This does not approve the model or resolve the forehead shading.

Five UV regressions and two retained head regressions pass, plus final reviewer
build and scoped diff check. An unrelated missing/duplicate-qualified meadow
helper call in mesh/ledge_approach.rs blocked rebuilding; its import/call were
minimally repaired without changing helper behavior. All owned processes
completed before scratch cleanup. 35 individual painted atlases installed.
Elder, PokefanM and Sage still need individual bald texture repainting. Full
75-role goal active.


## Bald crown boundary follow-up

The selective crown subdivision left T junctions at its boundary. A new welded
edge regression failed on Blaine before repair: a crown edge had one incident
triangle instead of two with opposing winding. `head_surface::bald_crown` now
conforms existing midpoint vertices before subdivision and after each pass,
while retaining separate UV chart copies. New vertices interpolate UV/color
and normalized skin weights; original UVs and skin influences stay exact.
Area-weighted crown normals are rebuilt across welded positions. Lower face
and body positions/normals remain exact.

Both ignored supplied head regressions pass, including all six bald roles,
closed oriented crown edges, valid normals/weights, face preservation, and 61
Idle plus 61 Run animation bounds samples. Reviewer build and scoped diff check
pass. Hand-reviewed all six in four-angle Idle and Run .25 against the baseline.
Scientist Idle remains pixel-identical. Boundary continuity is repaired, but
forehead patches remain visibly present, particularly on the fallback atlases;
this is not a successful visual cleanup claim. Elder/PokefanM/Sage retain hard
hair bands and require individual paintings. No raster asset changed in this
follow-up; 35 individual atlases remain installed. Full cast goal stays active.

## Elder individual atlas

Builtin imagegen repainted the exported Elder fallback; targeted repair removed
rectangular scalp panel borders. Full prompts/source/hash in character-uv-repaint.md.
Native beard/crown retained; front/plain rear/sleeve/collar mappings preserve
geometry and skinning, lower robe matches painted ochre and shoes dark leather.


Elder visual validation: five supplied UV regressions pass, including exact
geometry/normals/skin-influence preservation and plain ochre sleeve/collar
sampling. Reviewer build and scoped diff check pass. Hand-reviewed actual
production four-angle Idle and Run .25 images against exported baseline.
Hard grey hair band is gone; distinct mature face, feathered silver side wisps,
front ivory collar and ochre robe read clearly. Forehead shading patches remain;
beard is still a flat grey geometric wedge, hands/neck simplified, cane missing,
lower robe plain with uneven waist join and stiff/irregular animated hem.
This approves only the texture improvement, not the finished character. All
owned processes ended before scratch cleanup. 36 individual atlases installed;
PokefanM/Sage still require individual bald paintings. Full 75-role goal active.

## PokefanM individual atlas

Builtin imagegen repainted the exported fallback atlas. Full prompt/source/hash
in character-uv-repaint.md. Distinct middle-aged face, feathered brown side hair,
chocolate polo with tan collar, trousers and dark leather shoes. Native glasses
and moustache retained. Plain back/collar sides and short sleeves retain source
geometry/skin influences, forearms sample warm skin below the hand details.


Five supplied UV regressions pass, including PokefanM exact source geometry,
normals/colors and skin influences, plain collar sides and short-sleeve/skin
sampling. Reviewer build and scoped diff check pass. Hand-reviewed actual
four-angle Idle and Run .25 against baseline: solid brown hair band and white
rear-waist strip are gone; face/collar/buttons/leather footwear are readable.
Forehead shading patches remain, source hands are simplified, wrist color seams
need attention, moustache is a flat surface, body is still thin in side view and
individual ball badge is missing. Sole/heel read grey. This retains the texture
improvement only, not final model approval. All owned processes completed before
scratch cleanup. 37 individual atlases installed. Sage remains the unpainted
hatless bald role. Full 75-role objective stays active.

## Sage individual atlas

Builtin imagegen repainted actual exported fallback. Targeted repair removes
straight scalp panel borders/notches and horizontal hem band. Full prompts,
source and hash in character-uv-repaint.md. Front sand robe/collar, plain
sleeves/back/side collar and matched lower robe preserve source geometry and
skinning. Distinct older face and feathered silver side wisps, cloth shoes;
previous crown retained.


Five supplied UV regressions pass, including Sage plain sand cloth sampling,
exact geometry/normals/colors and skin-influence preservation. Reviewer build
and scoped diff check pass. Initial test/build attempts failed on disk space;
after external space recovery both were retried successfully without deleting
shared caches. Actual four-angle Idle and Run .25 hand-reviewed against baseline:
hard grey hair band gone, face and overlapping robe collar are readable.
Forehead shading patches persist; hands/neck simplified, lower robe plain with
uneven waist join and animated hem, soles/heels grey and bead necklace missing
(the appearance declares beads but has no geometry implementation). Retain the
texture improvement only, not finished model approval. Owned processes ended
before scratch cleanup. 38 individual atlases installed; all six hatless bald
roles now have individual paintings. Full 75-role objective remains active.


## Bald forehead lighting follow-up

Added opt-in offline `CRYSTAL_PEOPLE_REVIEW_UNLIT=1` albedo-only review.
Production materials remain unchanged outside the reviewer. Actual four-angle
albedo-only renders of Blaine/Elder/Gramps/Pharmacist/PokefanM/Sage removed the
irregular forehead patches while preserving paintings, pointing to normals
rather than UV panels; actors already have NotShadowReceiver. The prior
area-weighted cap normals varied with selective-subdivision triangle areas.
Replaced those normals above the existing cap threshold with a smooth rounded
elliptical cap field derived from supplied source head bounds. Refined positions,
UVs/colors, joints/weights and lower-face/body normals stay exact; topology
repair is retained. This is distinct from the earlier rejected normal blend.

Hand-reviewed all six in four-angle normal-lit Idle and Run .25 against baseline,
plus albedo-only diagnostic: irregular forehead facet shading is visibly reduced
and cap lighting is smooth. Broader painting color transitions and source
contours remain, especially PokefanM's rear scalp and neck. Existing beard,
accessory, garment and hand defects are unchanged and still outstanding.
Both supplied head regressions pass, including an added bound on neighboring
cap normal variation, oriented closed crown edges, unit normals, pinned lower
face, UV/skin-influence preservation and 61 Idle plus 61 Run animation bounds
samples. Reviewer build and scoped diff check pass. No raster assets changed;
38 individual atlases retained. All owned processes ended before scratch cleanup.
Full 75-role objective remains active.


## Sage fitted prayer beads

Added original Rust geometry in beads.rs through the existing chest-attached
accessory slot. 36 rounded wooden beads (20 sides/12 latitude divisions, radius
.022) and a thin closed cord, gentle wood palette variation and smooth normals.
Medial source torso slice closes across the arm junctions with a convex outline;
144 source-fitted path samples resample by arc length to avoid shoulder spikes
and uneven front/back bead spacing. Each final center is fitted with .012 source
units of clearance. No body/texture/skin-influence changes and other roles do not
receive this necklace. The necklace is rigidly attached to Chest; no swing/cloth
simulation is claimed.

First fit regression failed on missing shoulder-root slices. A wider slice
candidate passed bounds/closure but visual review rejected projecting shoulder
beads and crowded rear beads. Revised medial torso outline and arc spacing
remove those defects in actual four-angle Idle, Run .25 and opposite Run .75
renders. Retained regression checks closed oriented bead/cord surfaces, outward
unit normals, round spheres, source outline clearance, bead spacing/extent,
non-Sage exclusion and body bounds through 61 samples over each complete Idle
and Run clip. Scoped regression, reviewer build and diff check pass. Initial
compiles failed from shared disk capacity and an unrelated terrain module error
subsequently corrected in current source; successful final runs disabled Cargo
incremental caching only for those commands. Removed only the specific failed
compiler working directory identified in this task's failure log after own
processes ended; other cache directories preserved. Review scratch/logs removed
once all owned runs completed.

Sage now has the previously missing bead accessory. Hands/neck contours,
robe waist/hem/limited lower cloth details, grey footwear heels and broader
cast defects remain. No raster asset changed; 38 individual atlases retained.
Full 75-role goal remains active.


## Pryce atlas and sparse-haired crowns

Builtin imagegen repainted Pryce's actual fallback atlas, followed by a targeted
repair of solid grey scalp panels and heavy knit/border treatment. Installed
pryce.png; full prompts, selected source and SHA256 are recorded in
character-uv-repaint.md. Distinct elderly grey-blue-eyed face, thinning silver
wisps over scalp, ice-blue cardigan/ivory insert/buttons, plum trousers and dark
blue leather shoe uppers. Plain rear/sleeve/collar-side UV sampling removes the
stray white rear waist band and avoids repeated front details. Original body
geometry and skin influences are preserved by UV routing.

The existing rounded, closed crown refinement and smooth elliptical cap normal
field now include hatless sparse-haired Pryce, Kurt and KurtOutside. Actual
four-angle Pryce Idle/Run .25 renders reviewed against baseline; Kurt and
KurtOutside reviewed separately in four-angle head/upper-body crops before/after
Idle and after Run .25. Crown contours/light improve, but Kurt's generic face,
dense grey cap and notched hairline remain fallback painting defects.

Five painted-torso and two head regressions pass, including all nine eligible
bald/sparse roles, Scientist exclusion, protected lower face/body, UV/color/
skinning preservation, oriented cap closure, outward/unit normals, neighboring
normal variation and 61 samples over each complete Idle and Run clip. Reviewer
build and scoped diff check pass. All own processes ended before scratch cleanup.

Pryce still has simplified hands, narrow neck/lower-head contours, scalp color
variation, visible knit grain and grey heel/sole regions. Declared cane/scarf
are absent. This is an improved UV atlas and crown pass, not a finished model.
39 individual painted atlases installed; full 75-role objective remains active.


## Kurt identity and robe UV repaint

Builtin imagegen repainted the actual exported Kurt fallback, with targeted
repair removing heavy outer/shoulder seams and a straight scalp transition.
Full prompts, rejected/selected sources and SHA256 in character-uv-repaint.md.
The same selected image installs as kurt.png and kurt_outside.png to preserve
one character's identity across the indoor/outdoor roles. This counts as one
new distinct painting applied to two roles: 40 distinct repaints, 41 role files.

Distinct weathered older artisan's face, brown eyes/silver brows, sparse silver
wisps over visible scalp replace generic young face/solid grey cap; crossed
indigo work-robe collar with narrow ivory insert and brown leather work shoes.
Wide front mapping and plain rear/sleeve/collar-side fabric sampling avoid
repeated neckline/details; lower robe color samples selected plain cloth rather
than the old swatch. UV routing preserves geometry/normals/colors/skin weights.
Actual four-angle Idle and Run .25 hand review separately for both roles: 16
views confirm these changes, with baseline comparison. Waist remains visibly
different/plain, and original join/hem limitations remain. Neck/rear lower-head
contours, hands/wrist colors, grey heels and absent apron/basket remain unfinished.

Five supplied UV regressions and 29-roster costume bind/animation regression
pass; reviewer build and scoped diff check pass. An unrelated terrain test's UV
tuple-to-four-corners call prevented the initial test compile; minimal correction
retained and that named regression passes, with no production terrain edit.
All own processes ended before scratch cleanup. Full 75-role objective stays active.


## Kurt fitted waist apron, pocket and belt

Added original Rust geometry in apron.rs through costume_mesh for Kurt and
KurtOutside only. Warm canvas waist apron with shallow broad folds, rounded
lower outline and raised work-pocket detail, plus a narrow belt around the
whole waist. Each panel is a closed thin shell on a 24-column/32-row grid;
the closed belt has 96 sides and two vertical sections. Total 3,876 added
vertices across apron/pocket/belt. This is colored native geometry; existing
40 distinct repaints / 41 role atlas files are unchanged. Back ties/knot and
KurtOutside's apricorn basket remain absent; other declared apron roles have
not received this targeted accessory implementation.

Front-ray projection fits apron to existing robe and pocket to apron. Radial
horizontal triangle sections fit belt to the full robe circumference. The
outermost supplied cloth triangle supplies barycentric joint weights, combined
and normalized to four skin influences. Thin layers are .008 source units;
base front gap .014, belt .018. The lowest apron region smoothly adds up to
.014 clearance, with the original waist/pocket fit retained. Original robe/body
positions and atlas are preserved. These accessories follow the authored rig;
no cloth simulation, free swinging, or usable pocket storage is claimed.

First compile caught omitted cutaway metadata in the test fixture; corrected
with defaults. Initial closure/unit-normal/interpolated-point bounds regression
passed, but a strengthened check against actual deformed source triangles found
insufficient normal clearance at the inner hem near raised-leg Run phase.
Complete-cycle diagnostic minimum .0025636577 at Run step 21 of 60, apron
vertex 1613, maximum separation .026063377. This remained outside the robe but
failed the retained .004 normal-clearance margin. Added only the local hem
clearance fade instead of lifting all cloth or relaxing the assertions.

Final apron regression passes for both Kurt roles: oriented edge closure,
nondegenerate faces, consistent winding/normals, unit normals/normalized skin
weights, exact appended apron geometry, non-Kurt exclusion, 61 samples over
complete Idle and Run clips, signed clearance above each ACTUALLY deformed
source triangle greater than .004 and separation below .045 for all three
parts, plus render bounds. Existing full 29-roster costume bind/animation
regression also passes (run from the newly built test executable). Reviewer
build and scoped diff check pass. One reviewer compile was stopped by an
unrelated terrain debug trace formatting an unsupported GridGeometry Debug;
the trace had already been corrected in current source when inspected, so
this pass did not edit terrain. Successful build then verified current source.

Hand-reviewed both roles separately in four-angle Idle, Run .25 and opposite
Run .75 production-mesh renders against the no-apron baseline: 24 final views.
Apron follows the curved robe in both leg poses, pocket remains attached,
belt wraps sides/back, and no obvious apron intersections/floating gaps appear
in those views. Waist/hem limitations of the underlying plain robe, hands,
neck/rear lower-head contours, grey footwear heels, back ties/knot and missing
outdoor basket remain unfinished. All owned processes ended before scratch
renders/logs were removed. Full 75-role objective remains active.

## Connected fingertip contour pass

Added native Rust `hand_digits::sculpt` after the existing hand smoothing.
The supplied rigs have one index branch and one thumb branch per hand, rather
than four independent finger chains. This pass shapes four rounded, connected
fingertip lobes into that existing index slab. It does not claim independent
finger articulation or finished hand anatomy.

Three conforming refinement passes (.19, .19, .35 hand-local selection cuts)
split shared geometric edges on neighboring faces as well as selected faces.
Attribute seams retain separate vertices; new UV/color attributes interpolate,
and new skin weights combine the strongest four normalized influences.
Original vertex UVs, colors, joint IDs and weights remain exact. The wrist,
thumb, palm base and noneligible body positions/normals remain unchanged.
Hand-local distal contour subtracts .120 times smooth activation and lateral
groove depth, with modest thickness shaping and welded area-normal rebuilding.
An initial candidate with extra outer-finger shortening failed the required
valley margin; that extra shortening was removed and the valley depth corrected.

The supplied-rig regression passes for human-male, human-female, skater-male
and skater-female: two detected hands each, more than 500 added vertices,
protected attributes/positions/normals, finite unit normals, normalized skin
weights, four sampled tip lobes with valleys at least .035 below both adjacent
tips, oriented welded edge closure in the distal region, and bounds across
61 samples each of relaxed Idle/Run and raw Jump. This is not an animated
self-intersection or face-inversion proof. Reviewer build and scoped diff check
pass. Two intermediate Cargo attempts stopped before compilation from shared
disk exhaustion; successful retries verified the retained source.

Compared Chris, Kris, Kurt and Nurse against old-hand Idle renders and new
Idle/Run .25 renders. Then inspected all 75 roles, four angles each in Idle and
Run .25: 600 final hand-focused views across 25 contact sheets. Wrist attachment
and thumb continuity remain visually intact; no obvious new hand/costume
collision appears in those reviewed views. Fingertips are more readable than
the former broad mitten slab, but still thin and strongly shaded in places;
palms, knuckles, nails, finger proportions and shared articulation remain
simplified. Existing robe/skirt/coat waist joins, plain lower clothing, angular
leg contours and footwear limitations remain visible and unfinished.

No bitmap atlas changed in this pass: 40 distinct repaints / 41 role atlas files
remain installed. Full character quality objective remains active. Owned render
processes completed before the temporary hand review images/logs were removed.

## Kurt lower robe drape and painted fabric

Added native `robe_drape` only for Kurt/KurtOutside when their supplied
painting is present. Both shell layers gain shallow outward longitudinal folds
(maximum .020 source units), faded below y=1.52 over .30 units. The fitted
waist remains exact. The existing plain left indigo atlas island supplies
averaged linear vertex colors (u=.075+.035*sin(angle), v=.62+.32*height);
crossed collar, center seam, hands, shoes and skin are excluded. This is subtle
vertex color variation at the existing mesh resolution, not a new full-resolution
cloth texture. Periodic coordinates and welded normals match duplicate columns.
No raster asset or atlas count changed (40 distinct repaints / 41 role files).

The new drape exposed a failure in the retained apron face/normal agreement
check. Large sloping faces could overpower thin rim faces with area weighting.
Apron, pocket and belt now share corner-angle-weighted normals, preserving
geometry/skin attributes while giving thin rims their proper contribution.
Assertions were retained; failure messages now identify the piece and face.

Validation passes: targeted closed-shell/finite-normal/safe-indigo-island test
with exact waist preservation, .020 displacement bound, periodic seam and
unchanged topology/UVs; supplied apron regression for both roles over 61 Idle
and Run samples, actual deformed triangle clearance and face/normal agreement;
complete 29-role costume bind/animation regression; reviewer build; scoped diff
check. Reviewed both roles separately in four-angle Idle, Run .25 and Run .75
renders: 24 final views, against old Idle baseline. Apron inherits shallow
folds and remains attached; pocket/waist show no obvious new gaps in those views.
Lower robe variation remains subtle, and neck/footwear, underlying garment
joins and hem behavior, back ties/knot and outdoor basket are still unfinished.
Full character objective remains active. All owned processes completed before
temporary drape renders/logs were removed.

## Kimono Girl atlas repaint and kimono integration

Built-in imagegen edited the actual reviewer-exported fallback atlas. First
output `exec-c048e202-3d35-435f-8afa-65e2b08b47d7.png` had beige strips along the
clothing's shoulder edge and a boxed panel border; rejected. Selected repair
`exec-8556ff9a-add3-46c7-8263-a67d427fd54a.png` installed unchanged at
`assets/open-models/people/npc-textures/kimono_girl.png`.
SHA-256: `cc693a502afdac4f3bb978e4d5c537f9556ca005a7c49a388490b4cff1665a88`.
This is the 41st distinct repaint / 42nd role atlas file; Kurt's two roles share
one painting. Clean young adult face, organized dark swept hair, crimson silk
crossed collar/ivory insert, small plum blossom motif and brown footwear.

Native integration uses the existing female front width .56, rear plain
patch centered u=.075/v=.62, plain sleeve and collar-side placement, and the
lower garment's color sampled from u=.075/v=.75. Rounded hanging sleeves and
a fitted cream/gold obi with rear bow now include Kimono Girl in their supported
roster. These are original native meshes; the face/hair/cloth painting remains
the unchanged selected image.

First production views exposed blossom fragments on the shoulders despite
clean shoulder edges in the atlas. Added Kimono-Girl-only whole-face shoulder
mapping for high cloth faces above y=2.30 with mean lateral distance above .18
or a rear-facing center. All three vertices must occupy the cloth island
(u<.625/v>=.475); skin islands and central front collar remain separate.
The first narrower mapping removed large fragments but left one rear seam
fragment; the final selection includes that inner rear shoulder seam.
This remaps UVs without moving vertices or changing normals, winding, colors
or skinning. The broad UV preservation regression's historical torso-only
bounds needed explicit Kimono Girl shoulder/island cases. Other role bounds
remain unchanged; exact geometry and skinning assertions remain in force.

### Initial prompt

Use case: style-transfer. Edit this actual renderer fallback into a clean production flat square UV albedo atlas for Kimono Girl, a young adult Japanese dancer in a crimson silk kimono. Preserve EXACT normalized atlas rectangles and original facial feature positions, scale and orientation. Upper-left head x0..0.625,y0..0.479; original eyes around x=.26 and .36,y=.22, brows/nose/lips/ears remain at their original positions. Paint a graceful expressive adult face, clear warm brown almond eyes, softly arched dark brows, small composed closed smile, smooth warm peach skin and restrained cheek color. Stylized hand-painted game albedo with broad soft planes, NOT a photographic face, no dense pores. Glossy very dark brown hair in organized sweeping locks; keep original swept hairline and entire upper band filled with hair through x=.625. Lower head strip to y=.479 is clean warm neck skin, remove muddy grey neck. Lower-left x0.. .625,y.479..1: elegant deep crimson silk kimono torso, crossed V collar centered u=.3125,v=.48 with narrow ivory inner collar, gentle flowing cloth folds. Small restrained pale pink plum blossom motif only on the outer front chest, around u=.42,v=.65. No painted sash, belt, buttons or lapels. Keep left plain crimson strip u=0.. .14,v=.53..1 for sleeve/rear/lower robe sampling, no motifs, collar or seam there. Lower-right region x.625..1,y.75..1 remains matching crimson cloth with modest folds; no trouser belt. Shoe islands x=.625.. .8125,y=.125.. .5 dark warm brown footwear with neat simple seam and muted sole; preserve exact island placement. Keep right arm/hand/skin islands intact, including tiny finger forms at original coordinates. Output ONLY flat square UV atlas, no model, portrait layout, scene, text, grid, border, crop, rotated islands or new islands. UV alignment and original feature placement are highest priority. Clean organized low contrast albedo, no dramatic lighting, hard highlights or heavy grain.

### Targeted repair prompt

Use case: precise-object-edit. This image is the edit target, a flat game UV atlas. Fix ONLY the lower-left clothing rectangle x0.. .625, y.475..1. Remove ALL beige/skin-colored horizontal bars and exposed beige gutters from the upper clothing edge, specifically the left x0.. .125,y.475.. .50 and right x.49.. .625,y.475.. .52. Fill those areas with matching deep crimson silk, continuous to the exact clothing boundary y=.475. The central V ivory inner collar should remain, but every shoulder and outer edge is crimson fabric. Keep a continuous plain crimson sampling strip at u0.. .14 across v.475..1. Remove the dark vertical boxed-panel border at u=.606; extend cloth smoothly to u=.625. Preserve the face, hair, entire head/neck rectangle above y=.475, right-side shoe/skin/hand islands, blossom motif, central crossed collar and all normalized atlas positions exactly. Keep lower right clothing unchanged. No new borders/frames, seams at patch boundaries, text, model, scene, cropped or rotated islands. Output ONLY flat square atlas.
