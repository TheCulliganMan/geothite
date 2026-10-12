# Map visual interest

The current priority is scenery across the game's maps. Pokemon sculpture
work is deferred. The Lake of Rage baseline was inspected in full: repeated
crowns, broad uniform lawn, bare right-angle banks and nearly empty water
make the scene read as a terrain test rather than a place.

The verified local pack contains 388 maps in 35 tileset families. Review
must cover their distinct outdoor, geological and furnished-interior
identities; one lake image cannot establish quality across that set.

The scenery pass should establish a clear foreground, playable middle
ground and background, with visual features appropriate to each location.
Woodland needs layered vegetation and openings; water needs planted margins
and readable depth; towns need deliberate gathering places and varied
materials; caves need geological silhouettes and distinct chambers;
interiors need furnishings and lighting that communicate their purpose.
Paths, entrances, signs and important characters must remain legible.

The first shared addition is sparse shoreline ecology in the Rust terrain
builder. Source-authenticated outdoor water beside lawn gains clustered
reeds, russet seed heads and, in named freshwater areas, notched lily pads.
Deterministic map coordinates produce groups and gaps. Every triangle stays
in its water cell. Paths, room pools, authored water features and unresolved
source art are excluded. The pass adds no collision or support height and
does not replace source art or create another content catalog.

The integrated Cargo shoreline regression passed both tests, and the fresh
native location renderer compiled successfully. A direct run of the resulting
production test executable passed eight live-water tests, the outdoor-water
geometry/UV preservation test, and both shoreline tests (11 total). Muted
native image-target captures of LakeOfRage, IlexForest, Route42 and
CherrygroveCity completed successfully at 2560 by 1600 and were inspected
in full. Lake and Route42 show the new grouped planted margins; town paths
and entrances remain readable. These are local views, not whole-map or
playability approval. Broader map review remains required.

Outdoor water also lacked the cave shader's authenticated current path.
The mask now admits source-proven animated outdoor water, while retaining
the cave and Dragon's Den scopes and rejecting indoor pools, unknown art,
missing animation families and nonwater cells. The ground finishing pass
keeps those faces' original geometry and atlas UVs instead of converting
them to flat solid pigment with static ripple bars. The shader gives outdoor
water a brighter teal palette and broken moving glints. Its phase still
comes from the original animation; these changes add no simulation clock.
Geometry, source-phase and scope checks passed as described above. Native
shader execution and temporal presentation have been verified below. Final
water appearance remains under review with the shared lighting owner.

The forest capture exposed unfinished purple atlas surfaces on native ledge
hulls. A source-scoped finish now resolves the real Forest ledge cell, rejects
authored claims and surfaces outside the existing height range, and applies
the shared forest floor/bank materials without modifying the hull, ramps,
footing or ownership. The follow-up native build succeeded; its fresh muted IlexForest capture
completed in 16.31 seconds at 2560 by 1600. Full-image inspection confirms
the purple ledge surfaces are replaced by ground/bank materials. The hull
remains blocky and needs further art refinement. The freshly compiled production renderer test executable passes the forest
regression, nine live-water tests, the original-geometry/UV water test and
both shoreline tests: 13 passed, zero failed. These were run from a fixed
copy of the executable. The Cargo union command subsequently completed
successfully. The first forest capture is not approved.

The broader muted capture batch also rendered UnionCave1F, IcePath1F and
PlayersHouse1F successfully; all three full images were inspected. Cave and
ice palettes/materials are distinct, though these views still have broad
quiet areas and do not establish completed art direction. The home has
modeled furniture and a clear entrance, with an unfinished source rug.
GoldenrodPokecenter1F failed before rendering: object synchronization rejects
GOLDENRODPOKECENTER1F_PCC_TRADE_CORNER_RECEPTIONIST at (16, 8)
outside the supplied 10 by 8 map. A Rust probe of the actual pack confirmed
that this position is already authored in the source object; a callback move
has not been established as the cause. No image or approval is claimed for it.
The batch stopped on this error. Separate NationalPark and
CherrygrovePokecenter1F captures both completed successfully and were
inspected in full. The park has a readable fountain and planted borders; the
clinic retains unfinished source rug/counter art. Nine distinct maps have
usable reviewed native views, with one failed clinic explicitly excluded. This callback blocker is preserved, not bypassed.

A fresh same-camera native LakeOfRage temporal pair completed successfully
in 64.63 seconds at 2560 by 1600. Both full images were inspected: moving
water glints/current fields change while the planted banks and trees retain
their placement. This verifies visible temporal water presentation in the
image-target renderer. In a fixed 700 by 450 open-water region, 307,105
of 315,000 pixels changed; a 600 by 250 static canopy region was exactly
unchanged (150,000 pixels). Frozen preview/test executables were removed
after their completed runs; ignored screenshots remain for the ongoing art
review. Ordinary live-window, browser, whole-map and
playability checks remain separate.

The shoreline pass now also seeds cluster placement and individual reed
shapes from the location name. Different maps no longer repeat identical
planting at matching coordinates. The regular and Bug-Catching Contest
versions of National Park intentionally share one place seed. Existing
cell bounds, excluded surfaces, footings, claims and density limits remain;
the renderer's clock is not an input. Both expanded production geometry
regressions passed, covering three place seeds and all four bank directions,
stable repeated output, water-cell containment and protected paths/claims.
The fresh native build passed in 13 minutes 26 seconds, including its wait
for the test build. The first Lake capture mistakenly omitted
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and used the default window path.
It wrote an all-black image and exited with an explicit blank-frame error;
that result is a failure. The separately labeled explicit image-target Lake
check passed in 54.24 seconds (2560 by 1600); the full image was inspected.
Reed groups and notched pads are readable along the lake margins. Repeated
tree crowns and rectangular water glints still need refinement. This is
approval of the shoreline detail and image-target execution, not completed
Lake art or evidence that the window capture works. The fresh canonical-root
IlexForest image-target check also completed successfully in 54.66 seconds
at 2560 by 1600. Its RGB image shows apparent black leaf-card regions
and heavy translucent foliage overlap, so it is rejected as art. Forest
litter is visible, but the ledges remain largely occluded; their appearance
is still unapproved. These two completed captures do not expand the earlier
nine-map coverage count because both locations were already sampled.
The completed frozen preview executable was removed; review images and logs
remain ignored.

A subsequent Rust probe of the actual external pack located six Forest
ledge blocks inside the 30 by 54 tile map. The eastern group includes the
cap at (25, 22), so another muted, canonical-root image-target capture used
that verified position with the same previously built renderer snapshot.
It completed in 36.86 seconds at 2560 by 1600. Full-image inspection rejects
this view: foliage hides almost all of the target ground and the player,
with black leaf backgrounds and many overlapping translucent layers.
The ledge finish remains visually unapproved. Further camera-only trials
are deferred until the foliage presentation is corrected.

Read-only inspection of all five canonical tree variants confirms the same
leaf texture alpha range (0 through 255), transparent pixels and a 0.5
cutout threshold. Their source images do contain alpha data. The exact GPU
cause is not established by these checks; no tree geometry, tree dispatch,
texture asset or shared shader was edited for this diagnosis. The temporary
Rust probe and executable were created outside the repository and removed
when their verified-pack read completed.

The saved historical captures are RGB, not RGBA: the capture observer
previously converted readbacks to RGB before writing them. Converting those
files back to RGBA creates an artificial opaque channel and cannot diagnose
framebuffer transparency. Their black RGB pixel counts establish visible
black areas only, not their original GPU alpha values.

An opt-in `CRYSTAL_RENDER_TEST_CAPTURE_CHANNELS=rgba` capture mode now keeps
the readback's alpha channel. The default remains RGB, and `rgb` is also an
explicit accepted value. This changes diagnostic export only, without
changing cameras, scene materials, geometry or shader behavior. Library and
test-source compilation checks passed; the native build and actual RGBA
readback verification are pending. Alpha measured after rendering must also
be distinguished from the source texture's alpha and from intermediate
prepass or color-pass values.

The first native RGBA attempt failed before readback, so it produced no
verified RGBA image. Its render worker panicked on an empty shoreline
candidate index. The source-scoped shoreline constructor deliberately
returns an empty index for excluded maps such as IlexForest, while its
color lookup indexed that array for a valid in-grid ground vertex.
The lookup now returns the original ground color when no candidate index
exists. Eligible shoreline palettes and geometry are unchanged. A regression
checks all in-grid cells for three excluded map types; the existing damp-bank
regressions all passed (four tests total). The fresh native build completed
in 15 minutes 13 seconds, including its wait for the test build.

The real-pack RGBA forest capture then completed successfully in 8.73
seconds, without the shoreline panic, at 2560 by 1600. The saved PNG is
actually RGBA and has alpha values from 0 through 255. Of 456,837 nearly
black pixels, 455,879 have alpha 0 and 958 have alpha 255. Thus almost all
of the apparent black leaf backgrounds are fully transparent regions in the
final framebuffer; calling them opaque GPU pixels was not established by
the earlier RGB captures. The exact cause of those regions in the scene
remains unproven. The complete image still has excessive foliage overlap
and remains rejected as finished forest art. The diagnostic export and
excluded-map crash fix are verified; leaf appearance and ordinary window
capture are not approved. The completed frozen diagnostic executable was
removed, while ignored images and logs remain for the ongoing review.

The capture environment did not enable `CRYSTAL_MODELED_OCCLUSION` or
`CRYSTAL_GPU_CULLING`. Source inspection also confirms the culling experiment
is scoped to New Bark and Route 29. These checks do not prove every camera
component's runtime state, but do not support attributing the forest failure
to those optional modes.

An opt-in `CRYSTAL_VOXEL_PIPELINE_TRACE` now reports scenery alpha modes and
the specialized voxel pipeline's key and fragment definitions. It adds no
pipeline mutation, shader change, material-mode override or geometry change;
normal runs emit no new trace lines. Production library compilation passed.
The native trace build completed successfully in 5 minutes 45 seconds.
The muted, canonical-root, image-target RGBA forest run completed with exit
code 0 in 25.24 seconds and wrote `forest-alpha-pipeline-trace.png` at
2560 by 1600. Actual scenery materials include `Mask(0.5)` and `Opaque`.
The masked forward pipeline and masked depth prepass both contain
`MAY_DISCARD` in their keys and fragment definitions. Other opaque and
blended pipelines also appear; their lack of that definition is expected
and does not establish a masked-pipeline failure. A missing discard
specialization is therefore not supported by this trace.

The new PNG retains alpha 0 through 255. Of 459,072 nearly black pixels,
455,879 have alpha 0 and 3,193 have alpha 255. Full-image review still rejects
the forest: large transparent regions between leaf silhouettes and dense
canopy overlap remain. This is execution and diagnostic evidence, not
finished art approval or proof of ordinary window rendering. The completed
frozen trace executable was removed. No tree geometry or shader appearance
was changed by this diagnostic pass.

The nine earlier map captures omitted the external canonical model root.
They establish image-target execution, but cannot approve the canonical tree
appearance. A subsequent IlexForest capture explicitly used
`CRYSTAL_OPEN_MODEL_ROOT=$PWD/assets/open-models` and muted audio. It exposed
black leaf-card backgrounds and excessive overlapping foliage; that image is
rejected. The shared outdoor owner controls tree geometry and the lighting
owner controls foliage alpha and water appearance. Further previews must use
the explicit canonical root.
Read-only inspection of the canonical tree asset confirms that its leaf
texture contains alpha values from 0 through 255 and a 0.5 cutout threshold.
This does not establish the cause of the black backgrounds. The completed
forest preview executable was removed; its image and log remain for review.

The follow-up forest finish adds sealed earthen relief to verified native
ledge walls and forest litter to their horizontal caps. Both integrated
forest regressions passed, preserving native corners, floor contacts and
claims. The canonical-root capture largely occludes the ledges, so visual
approval of this detail remains pending.

Following the user's no-substitute directive, four obsolete alternative
creature meshes and their dispatch paths were removed. Missing canonical
bodies retain original 2D source art. The current production library check
passed. A fresh production test binary passed two forest ledge tests, two
shoreline tests, four external-loader tests, the missing-forest-asset test and
the authored actor geometry test. Its body-routing suite passed three tests
and failed two: the earlier 251-body assertion included source-icon
substitutions. The cleanup also exposed three real starter rigs needing
direct canonical registration. They are now registered without substitutes;
the focused Cargo rebuild subsequently completed successfully in 12 minutes
48 seconds. All five body-routing tests passed, including bounded geometry
for the 225 dedicated-body catalog. The superseded full-host Cargo test build was stopped after
these measured failures; it is not a completed passing command.
The Python actor checker is
currently blocked by an existing refined Clefairy icon exceeding its old
25,000-triangle limit; no passing full actor audit is claimed.


A same-executable, same-location forest comparison used the existing
`CRYSTAL_CUTAWAY_DIAGNOSTIC=zero-radius` switch, preserving normal shader and
geometry source. The muted, canonical-root RGBA image-target run completed
with exit code 0 in 86.33 seconds. Full-image review shows the large gaps
remain with player-reveal fading disabled, and foreground foliage obscures
the player more heavily. The baseline has 455,879 fully transparent pixels;
the zero-radius comparison has 517,742. Thus the evidence does not support
removing player reveal as a fix for these gaps. Both images remain visually
rejected. This isolates one presentation variable; it does not establish
the precise material or depth failure. The completed comparison executable
was removed.

A subsequent experiment varied shoot and lily-pad counts. Its test build
remained live under heavy compiler load and was intentionally stopped
(exit 130) for the requested transfer checkpoint, not counted as a failure
or pass. The unverified count experiment was reverted using exact owned
edits. The final shoreline source retains the previously verified
location-seeded placement and reed variation, nine shoots and three pads.
No current native image or test pass is claimed for the reverted experiment.


## Transfer checkpoint

The Rust client must read the externally transferred, ignored content pack;
this pass used `content-packs/realtime-clock.browser.crystalpack`.
Canonical model loading requires
`CRYSTAL_OPEN_MODEL_ROOT=$PWD/assets/open-models`. Every owned native
preview used `CRYSTAL_PREVIEW_MUTE=1`. Image-target diagnostic verification
also explicitly used `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`; alpha
inspection requires `CRYSTAL_RENDER_TEST_CAPTURE_CHANNELS=rgba`.
A successful diagnostic image does not verify the default window path.

This checkpoint commits the standalone shoreline module, opt-in scenery
alpha logging and this review. The coordinating chat will capture the
remaining shared integrations in `mesh.rs`, `mesh/build_scene.rs`,
`mesh/new_bark.rs`, `lib.rs` and the Bevy `render_capture.rs`, together with
the shared `mesh/shore_blend.rs` file containing the verified excluded-map
empty-index guard. Those shared files contain other owners' changes and
must not be staged wholesale as solely this pass's work.

Outstanding verification and visual blockers: imported forest foliage gaps
and canopy overlap; black default-window capture; the authored out-of-bounds
Goldenrod clinic object; complete map-family/browser/live-window coverage;
and remaining canonical creature/shiny art coverage. Nine unique maps were
captured earlier, not all 388 pack maps. The current dedicated-body catalog
is 225, not proof that every species or shiny has finished 3D artwork.
Audio programs, generated PCM, packs and scratch render output are excluded
from this source checkpoint. No task-owned preview process remains active.
