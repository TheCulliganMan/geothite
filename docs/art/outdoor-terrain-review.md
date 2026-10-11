# Outdoor terrain review — 2026-10-10

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
