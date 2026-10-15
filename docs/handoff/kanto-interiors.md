# Kanto interiors: checkpoint and restart

The user expanded the active goal on2026-10-14 to Johto and all interiors,
prioritizing the player's home. Current pack metadata selects311 non-TOWN/ROUTE
maps:191 Johto and the original120 Kanto. The original16 Kanto block combinations
and all unresolved findings remain in scope. The goal is not complete.
Authored-family counts and successful mesh audits do not establish visual completion.



## Store aisle walking exposes recovery defects, 2026-10-14

Previous turn made progress: seats/plant color corrected and actual family
views plus shelf near/away poses inspected. Current production-dispatcher
synthetic walk40992 completed0 fromGoldenrod4F(2,3),RRR: trace visits every tile
x2through13 atrow3.96frames, zero extra terrain builds, median16.93ms/p9557.46ms
(image-target diagnostic, not presented FPS). Final pair and all3 moving images
individually inspected. Source claims/support untouched, player/follower visible
along shelf bank. Walking reveals ugly triangular closed-face layers while
cabinets restore and final opaque service counter hides player lower body.
Therefore full walking/appearance acceptance remains rejected; earlier static
scoped visibility acceptance does not prove recovery quality.

Cabinet-only depth coverage hypothesis now implemented: exact4F shelf ranges
opt in via `TerrainMeshData.depth_coverage_textured_ranges`; prepared groups
retain private depth flag through weak cache and linked overlays. Masked fade
material writes depth while shader's existing stable coverage mask reveals the
actor. Default groups and existing foliage/wall reveal keep their current blend.
Geometry/palette/claims/support unchanged. Regression checks opt-in range,
weak-cache retention, masked material and unchanged ordinary/opaque modes;
placement checks exact two owned ranges. Test5210 logs
`store-depth-coverage-tests.log`, native69492 `store-depth-coverage-native.log`
live. Fresh exact same aisle walk and still/control views required before
acceptance. Source counter height10px (body9+cap1), unrelated to shelf change;
counter occlusion remains separately open. No exports/staging/commits/messages.
Full311 interiors/original16 variants active; no completion/block audit passed.
Test5210 passed1; same fresh binary fade suite57633 passed32 and placement50081
passed69. Native69492 completed0 in5m38s. Exact repeat Goldenrod4F(2,3),RRR
capture now live, handle in next update; `store-depth-aisle-walk.log`, output
`store-depth-aisle-walk/`, graphics wrapper. Source material specialization was
checked: nonzero fade.w enables coverage shader path for the opt-in mask.
All moving/final images must be inspected; still/control views required too.
Repeat depth-walk handle22318 logs `store-depth-aisle-walk.log`; still live at
next checkpoint. Separately lowered shared store service counters from10px
(9body+1cap) to8px (7body+1cap), retaining all footprint/top continuity and
native palette. Register drawer datum now8px, original dimensions/key slope
unchanged. Existing joined-top/register test updated to8px and11.2px upper
register bound. Test51812 logs `store-counter-waist-tests.log`; native44991
`store-counter-waist-native.log` live. Latest counter geometry needs paired
1F/3F/4F5F service views and exact aisle endpoint before acceptance. No gameplay
collision/support or source claim changes. Do not conflate counter revision
with depth-walk binary if its launch predates counter build completion.
Depth walk22318 completed0 before waist native44991 completed (thus counters
still10px). Moving01 individually inspected: cleaner recovery; moving02 still
exposes triangular gray panels, so depth-only hypothesis not accepted as full
fix. Moving00/final still need inspection. Existing closed-case builder includes
opposing interior side walls at each touching bank seam. Exact source-complete
adjacent13/15 cabinets now omit those coincident gray side triangles only;
outer bank sides/back/top/native fronts retained, claims/support unchanged.
Neighbor check validates entire drawing before culling; unrelated floors remain.
Updated regression requires no internal planes and retained outer sides. Test
81580 `store-shelf-joins-tests.log` live; waist native44991 predates join change,
still live. Wait terminal then build latest joins/waist/depth together before
repeat walk; no stale visual approval. Earlier source tests passed25 counters.

## Latest verified store milestone, 2026-10-14

Current source improves shared store furniture beyond station work: pink padded
seats retain7px height/12px depth with short supports and no unsupported center
button;4F electronics/stock cabinets independently reveal overlapping player
projections and restore opacity away;0b plants now have dense source-green
crown, coral-red body and pale pink rim using renderer-resolved bank-one art.
Both4F default poses, stock-near(3,3), away(12,2),3Fcontrol, both1F/both6F plants
actually inspected. Scoped seat silhouette/paint, reviewed cabinet readability
and plant pigment/density accepted. Original placement/collision/support intact.

Final native53188 completed0 in5m24s; targeted12981 passed1. Capture85624
completed0 through shared graphics lock; all four actual comparisons individually
inspected in `store-plant-bank-pot-review/`. States preserved:Goldenrod1F40/LCD44,
Celadon1F165/169,Goldenrod6F173/177,Celadon6F89/93. Four-frame offsets support
scenery comparison, not exact actor-pose/fidelity approval. Prior full placement
suite69 and cutaway3 passed; latest targeted plant passes cover final swatches.
`git diff --check` passes. No active task-owned processes, exports, staged files,
commits or cross-chat messages. Existing ignored receipts retained for ongoing
goal verification. Earlier failed paint/build entries below are historical.

Full311 interiors/original16 script variants remain in scope, not complete.
Next store work: walking along full aisle and seat interaction/readability,
rear service-counter NPC visibility, stairs/lift/entrance architecture and
whole-room scale/fidelity. Other Johto/Kanto room families and previously named
gameplay/Flash/ceremony gates remain. No completion or blocked audit passed.

## Store plant continuation and verification history, 2026-10-14

Current pack's own0b plant pixels inspected in memory, no exported catalog.
Complete2x4 source0b plants in reviewed store maps now reuse existing dense
facility-crown method with one pot and two extra smaller leaf clusters. Same
original bounds/height/claims/support; own green leaf shades and source red/pale
pot swatches replace generic brown-pot treatment. Exact all-cell tile/phase and
zero-base guard; unknown materials retain source fallback. Expanded existing
plant regression covers1F/6F of both stores and unrelated-map/raised exclusions.
Test61353 in `store-plant-tests.log`, native44815 in `store-plant-native.log`
live. Earlier shelf graphics request26370 was deliberately cancelled while
queued, never launched; combine fresh shelf/plant/control captures after current
build and shared browser-measurement all-clear. No full plant or shelf acceptance,
exports/staging/commits or cross-chat replies. Full311/original16 scope active.
Test61353 completed101: native textured plant subsequently reached generic solid
room-palette recolor, triggering its vertex-count invariant (0versus2862). Excluded
source-native store plant from that legacy solid recolor. Fixed67110 logs
`store-plant-tests-fixed.log`; native44815 predates fix, still live. Wait terminal,
then rebuild current source before any combined capture; do not render stale
plant binary or accept previous pending build. Shared browser all-clear pending.
Fixed67110 passed1; earlier native44815 completed0 in2m43s, superseded by recolor
exclusion. Latest native logs `store-plant-native-fixed.log` (handle in next
update); same fresh placement suite logs `store-plant-interior-tests.log`.
Shared browser all-clear received, no reply. Next combined capture after current
native completes must inspect both4F, both6F, a1Fplant pose and3Fcontrol, then
shelf-away opacity separately. No plant/shelf visual acceptance yet.
Final native78485 completed0 in3m14s; fresh full placement suite passed69.
Combined six-map capture10549 live through graphics wrapper in
`store-shelf-plant-capture.log`, output `store-shelf-plant-review/`; queued for
lease at this checkpoint. Current requested maps: Goldenrod/Celadon4F and6F,
Goldenrod1F and3Fcontrol. Poll this exact handle; no duplicate launch. Current
source and native include both shelf drawings, corrected plant path and seats.
Capture10549 completed0; all six actual comparisons individually inspected.
Both4F now reveal player legs through one cabinet, adjacent cabinets solid;
3Fcontrol retained original opaque shelves. Padded seats retained. Plant foliage
denser, but pot body incorrectly pink: rejected as final pigment. Actual2D pot
area contains141red versus66pink pixels in sampled24x18 area (background204);
native source85 is red,170pink. Changed body to own tilea7 pixel(7,7) [7,31],
trim to own170 [2,25]; foliage/geometry unchanged. Latest native71188 logs
`store-plant-red-pot-native.log`; narrow test handle in next update logs
`store-plant-red-pot-test.log`. Fresh plant color views required.
Shelf-away capture78373 live atGoldenrod4F(12,2), output `store-shelf-away/`,
receipt `store-shelf-away.log`; queued/running status must be polled. Shelf source
unchanged by pot correction. Paired states preserved in six-map capture:
Goldenrod4F55/LCD59,Celadon4F53/57,Goldenrod6F68/72,Celadon6F50/54,
Goldenrod1F82/86,Goldenrod3F56/60. Four-frame offsets: scenery-only acceptance.
Away capture78373 completed0; actual comparison inspected atGoldenrod4F(12,2):
foreground cabinet bank restored fully opaque, player unobstructed. State
preserved57/LCD61. Reviewed electronics-cabinet near/away visibility scoped
accepted; stock-shelf15 near pose still needs direct visual proof. No whole-room
acceptance. Red-pot native71188 and targeted26110 still live at checkpoint.
71188 completed0 in2m58s and26110 passed1. Four-map capture92961 completed0.
Actual Goldenrod/Celadon1F pairs inspected: body dark/black, so red-pot pass
rejected; remaining6F files not yet inspected, not accepted. Investigated current
renderer `resolve_tileset_tile_index`: mart PNG has192tiles, bank-one half96,
so sourcea7 resolves to PNG87, a8to88,89to69. Earlier raw PNGa7 inspection was
incorrect for these bank-one art sources. Resolved87 row1medium170 is pink rim,
row2shade85 is coral red interior; [7,31] hits its black bottom border.
Changed body to own-source [3,26], rim retained [2,25]; other swatches unchanged.
Latest native53188 `store-plant-bank-pot-native.log`, test12981
`store-plant-bank-pot-test.log` live. Fresh four-map capture required.
Stock15 near capture51542 live atGoldenrod4F(3,3), receipt
`store-stock-shelf-near.log`, output `store-stock-shelf-near/`. It may use previous
native plant paint, but this room has no0b plant and shelf code unchanged; scope
is stock-shelf reveal only. No overall visual acceptance or cross-chat messages.
Stock capture51542 completed0; actual pair at(3,3) inspected. Player legs visible;
projection overlaps two stock cabinets, which fade independently while other
bank cases stay opaque. State preserved69/LCD73. Together with electronics near,
away restore and unchanged3Fcontrol, scoped shelf readability accepted at these
poses. All-aisle walking/cameras, shop architecture and whole-room acceptance
remain. Test12981 passed1; native53188 still live at this checkpoint.

## Fourth-floor shelf player visibility in verification, 2026-10-14

Source-complete mart block13 two-cell-wide cabinet drawings in Goldenrod/Celadon
4F now each mark only their own new textured geometry for existing blocker fade.
Ground underlay and neighboring cabinets remain independent. Exact tile/phase,
zero-base, kind/dimensions and reviewed-map guard; other floors unchanged.
No geometry, palette, claims, collision or actor-support edits. New regression
checks two independent ranges, source mutation exclusions, raised-base/3F
exclusions and preserved support. First test91403 failed on an overly broad
claim assertion including fixture's surrounding floor; corrected to16claimed
source cells and explicit unclaimed border. Final35728 logs
`store-shelf-reveal-tests-final.log`; native35801 logs
`store-shelf-reveal-native.log`; live at checkpoint. Both4F fresh views and
3Fcontrol required before acceptance, through shared graphics wrapper.
No exports/staging/commits or cross-chat replies. Full311/original16 active.
Repaired35728 passed1; native35801 completed0 in3m17s, now superseded. Direct
current pack block inspection shows foreground bank combines13 electronics and
15 stock shelves. Expanded exact guard to both source drawings and regression
to both blocks/both stores, preserving individual ranges and unrelated floors.
Latest test15447 logs `store-shelf-reveal-both-tests.log`; updated native handle
in next update logs `store-shelf-reveal-both-native.log`. No visual acceptance
from earlier narrower source/build. Full aisle correction required.
Latest test15447 passed1 for both drawings, fresh full placement57881 passed69,
and mesh cutaway checks passed3. Native65474 completed0 in2m44s. Three-map
capture26370 is live through shared wrapper, queued on the graphics kernel
lease (`store-shelf-reveal-capture.log`, `store-shelf-reveal-review/`). Do not
restart this known request or infer failure from queue time. Both4F actual views,
3Fcontrol and away-opacity pose still required. No visual acceptance yet.
Incoming short browser-measurement request needs next GPU slot. Our wrapper
PID44768 was verified as this exact queued request, stopped momentarily to
prevent acquisition race, checked for active child (none), then terminated and
continued to deliver termination. No active capture or other process interrupted.
26370 intentionally cancelled; relaunch same three-map capture after measurement
all-clear, not because queue observation failed. No cross-chat reply. Checks and
native result remain valid; visual acceptance still pending.

## Store seat silhouette follow-up inspected, 2026-10-14

After actual paint review rejected table-like silhouette and unsupported pale
button, source-specific store seats now fit padding from2.5px to7px instead of
5px to7px. Dark supports stop at2px with2–2.5px frame. Same overall7px rise,
12px depth, source claims and actor support. Only central button triangles in
the fitted shared cushion are omitted; broad cushion/piping retained, remapped
to referenced vertices and refitted to7px. Inverse-transpose normals preserve
lighting after this small vertical correction. Other cushion variants unchanged.
Test92233 passed1 before final normal adjustment; current full placement test
handle in next update logs `store-seat-padding-interior-tests.log`. Native10325
live in `store-seat-padding-native.log`. Actual two-store views required.
Read shared `tools/with-graphics-lock.py`: kernel flock held by child command,
released on child exit; use it for next capture. Measurement all-clear received,
no cross-chat reply. No acceptance, exports, staging or commits; goal active.
Final placement20529 passed68 and native10325 completed0 in2m56s. Capture54480
completed0 through shared graphics lock. Both actual two-store comparisons
individually inspected: thick pink padding and short dark supports replace
table-like legs/slab; unsupported center button gone, player clearance retained.
Scoped seat silhouette/paint accepted at these scenery poses. State preserved:
Goldenrod6F68/LCD72, Celadon6F49/LCD53; four-frame offsets, no actor-pose acceptance.
Furniture walking/seating interaction, room architecture, plants and other full
scope remain. No live task-owned jobs. Previous turn classified as progress.

## Store seat source paint inspected, 2026-10-14

Previous turn made progress: inspected both6F vending images, station control
and Itemfinder tatami, then updated authoritative handoff. Current source keeps
store seats'7px total height,12px depth and original cushion/frame geometry,
replacing generic purple room material with own-source pink/pale/dark swatches.
Complete mart block07/20 seat drawings, phase, reviewed store and zero-base
guards restrict paint; unknown model materials use visible source fallback.
Support heights and source ownership remain unchanged. Fourth-floor foreground
shelf visibility remains open, as confirmed in actual existing Goldenrod4F pair.

Initial test17269/native51597 failed on unrelated shared Clip::ALL array length
five versus three values. Narrow count correction to three preserves the current
enum/list; no other rig edits or cross-chat messages. Repaired test37511 then
compiled but failed because its UV assertion included ground underlay; corrected
assertion checks raised furniture, adds unknown-map/raised-base exclusions.
Final test42188 logs `store-seat-tests-final.log`; native71894 logs
`store-seat-native-build-repaired.log`; both live at this checkpoint. Graphical
launches briefly held for incoming shared90-second performance request; none of
ours is active. No visual acceptance before fresh both6F views. No exports,
staging or commits. Full311/original16 scope remains active.
Final test42188 passed1; same fresh binary interior-placement filter passed68
(`store-seat-interior-tests.log`) and model suite passed38
(`store-seat-model-tests.log`). An initial commentary incorrectly reused167
from the previous source checkpoint; corrected to the actual68 result.
Native71894 completed0 in3m22s. Fresh graphical seat checks pending the shared
measurement window, then required before material acceptance.
Shared measured preview was observed live as PID38751, then absent at23:32:35
after the requested hold elapsed; no PID killed and no cross-chat reply.
Capture46900 completed0. Both actual comparison images individually inspected:
pink cushion and dark frame now follow source instead of generic purple weave.
State preserved: Goldenrod6F65/LCD69, Celadon6F53/LCD54; offsets4/1frames,
so these are scenery comparisons, not exact actor-pose evidence.
Paint correction retained, but furniture geometry remains visibly table-like,
and the authored pale center button has no source counterpart. Therefore seats
are not fully accepted. Next seat work must address silhouette/cushion edge and
that center detail, alongside4F shelving visibility, plants and room architecture.
`git diff --check` passes. No active task-owned processes at this checkpoint.

## Sixth-floor gray vending material inspected, 2026-10-14

Previous turn verified front-only5F reveal and away opacity. Current source
corrects teal vending material on Goldenrod/Celadon6F. New exact2x4 block21
guard validates both origins0/2, every drawing/phase, zero base and reviewed map.
Existing vending model's validated casing roles now sample its own gray source:
body170(1,1),slots85(4,10),dark0(0,8),metal255(2,1),trim170(2,8).
Full beverage geometry retained and painted in source gray, including drinks;
station ticket method still omits drinks and uses its existing blue source path.
Unknown material fails before mutation; unrelated maps retain old treatment.

New model regression compares positions/normals/indices/cache; placement test
checks both stores/both source origins,8claims/footing, changed art/phase, unknown
maps and raised base. Test80529 receipt `store-vending-tests.log` (filter vending),
native35657 receipt `store-vending-native-build.log`, live under target/kanto-resume/
receipts. Same fresh binary must also run native_beverage/interior-model tests;
fresh both6F plus station control captures required. No material visual acceptance
yet. Full311/original16 scope active; no messaging/exports/staging/commits.
Tests80529 passed2; same fresh binary interior_models38 and modeled_interiors167
passed (`store-vending-model-tests.log`, `store-vending-interior-tests.log`).
Native35657 remains live. Holding graphical launches briefly for received shared
tree-performance request; no preview of ours is live, and no other PID is killed
or messaged. Receiving coordination does not authorize a cross-chat reply.
Native35657 completed0 in3m16s. Shared measurement all-clear received; actual
both6F plus Goldenrod station control capture now live, handle in next update,
receipt `store-vending-capture.log`, output `store-vending-review/`. No reply sent.
All three actual comparison images individually inspected after an earlier
multi-image tool result was truncated. Gray casing now matches the two store
references; machine height, display detail, seating and plants remain open.
Station control retains blue ticket boxes with slots toward the chairs.
Paired state preserved: Goldenrod6F73/LCD77, Celadon6F47/LCD51,
station52/LCD52. Store offsets preclude exact actor-pose acceptance.
Capture log contains all three verified screenshots and completed render loops;
process handle29403 is no longer available, so its exit status is not reasserted.
`git diff --check` passes. No full-room or whole-interior acceptance.
Latest tatami correction was rechecked against the actual Itemfinder comparison:
green floor is flat woven tatami, while the two blue seat cushions stay separate.
The seven traditional-room source guards and floor treatment remain in place.

## Fifth-floor kiosk player reveal in verification, 2026-10-14

Latest native97193 passed5m17s; capture31847 completed0. All3 actual pairs in
`store-kiosk-front-review/` inspected. Both5F islands now reveal only lower front
display; rear rail and returns remain solid, player legs visible,3Fcontrol kept
source displays/floor. Scoped near-inset visibility improvement only. Allstates
preserved:Goldenrod5F56/LCD60,Celadon5F155/163,Goldenrod3F85/89; offsets4/8/4frames,
so no actor-pose/fidelity acceptance.25store/37model checks pass. Extra outside
pose capture now live to `store-kiosk-away/` (handle in next update), receipt
`store-kiosk-away.log`, to verify the front restores opacity away from player.
No globalfade/scale/collision/gameplay changes. Full311/original16 scope active.
Away capture45698 completed0; actual `store-kiosk-away/review-compare.png`
inspected at(3,5). Lower front restored to fully opaque blue glazing when player
is outside its projection. State preserved101/LCD105 (4frameoffset). This confirms
bounded blocker behavior at reviewed inset/away poses, not every camera/walking
pose. Static kiosk-player reveal is scoped accepted. Rear counter still hides
an unrelated NPC's lower body; NPC readability,4Fshelf occlusion,6Fvending and
other geometry/scale/pigment/fidelity work remain. No live jobs, exports/staging/
commits or cross-chat messages. `git diff --check` passes.

First native22313 passed4m23s, capture47304 passed. All3 actual pairs in
`store-kiosk-reveal-review/` inspected. Whole-island fade reveals player legs but
unnecessarily washes out rear rail/returns: REJECTED as final treatment.3Fcontrol
remains unchanged. Follow-up marks only the canonical model's contiguous lower
front display volume (all marked vertices in southern source half), leaving
rear/side returns and floor opaque. Geometry/palette/height unchanged. Updated
tests require front-only eligibility and exact original geometry/pigments.
Latest test95648 (`store-kiosk-front-tests.log`) and native77499
(`store-kiosk-front-native-build.log`) live. Fresh actual3map capture required.
Test95648 failed23pass/2fail: front range failed closed because canonical model
exports final west/east full-depth toes after front parts. Added stable partition
of closed_u_display's named lower-display/front primitives during model import;
no triangle/position/material changes, only storage order. This makes exact lower
volume contiguous for one fade group while returns/toes stay opaque. Test77004
(`store-kiosk-front-group-tests.log`) live. Native77499 predates importer change,
still live; wait terminal then rebuild latest before any capture/acceptance.
Group test77004 also failed23/2: geometric first-depth search still selected a
full-depth side toe. Replaced geometric inference with explicit front-component
bit in existing validated store material roles. Stable named-part partition puts
those bits in one suffix; paint masks the bit, preserving all native pigments.
`store_display_front_start` returns only a wholly tagged suffix, and only that
suffix participates in reveal. Test28273 (`store-kiosk-front-tag-tests.log`) live;
native5545 (`store-kiosk-front-group-native-build.log`) predates tags, still live.
Wait terminal then build tagged source; do not accept a stale native result.
Test28273 completed0:25 passed; same fresh binary's interior_models suite passed37.
Native5545 completed0 in4m31s, superseded by tags. Latest native build handle in
next update logs `store-kiosk-front-tag-native-build.log`; fresh capture needed.

Previous turn made verified source-display/floor-pigment progress across6Goldenrod
floors. Current source inspection finds the exact5F closed-U kiosk already fits
8px, with2x4-cell true-floor inset and closed lower display. It was deliberately
excluded from occluder fading, hiding much of a trainer in that inset. Enabled
the existing whole-object blocker fade for this source-complete island in both
stores, keeping ordinary service counters opaque. Geometry/collision/footing,
source hash/sparse ownership and floor underlay remain unchanged. This uses the
existing fade, not a new aperture shader; visual review must ensure it does not
wash out the whole fixture unnecessarily.

Updated atomic network regression expects island eligibility and checks its
marked textured range excludes underlay, remains within8px and links materials.
Test71162 (`store-kiosk-reveal-tests.log`) and native22313
(`store-kiosk-reveal-native-build.log`) live, target/kanto-resume/receipts. Latest
extra assertions were added while tests compiled, so inspect compile snapshot
before claiming those passed. Actual Goldenrod5F/Celadon5F/control captures are
required. No reveal acceptance yet. All remaining311/original16 scope active;
no cross-chat messages, exports, staging or commits.
Initial test71162 completed101:24passed, old native-palette island test still
required no reveal. Updated it to assert the exact model range is marked while
underlay/geometry/palette/footing remain unchanged. Corrected suite51514 live,
receipt `store-kiosk-reveal-tests-corrected.log`; native22313 still live.

## Goldenrod department-store source treatment in verification, 2026-10-14

Latest native19992 completed0 in12m26s; all7capture5186 completed0. Every actual
pair in `goldenrod-store-floor-review/` inspected. Corrected floor preserves
45/14/5 source pigment areas with visibly stronger orange/cream bands across all
six Goldenrod floors and Celadon2F control. Blue2F/3F displays retain native
source fronts instead of pastel books;1F/3F counter source colors retained.
Scoped static material/floor-pigment acceptance only. Floor remains flat source
geometry; full lighting/composition/scale acceptance is not established.

166 modeled tests pass, including all6 Goldenrod retail guards plus Celadon,
exact pigment-area/coverage/ownership/height checks. All7states preserved:
Goldenrod1F82/LCD91,2F165/170,3F83/87,4F160/164,5F213/217,6F136/140;
Celadon2F99/99 exact. Goldenrod offsets4–9frames: scenery proof only.
4F foreground shelf occlusion,5F kiosk geometry/actor hiding,6F teal vending,
plant/seat/source-fidelity/architecture/walking defects remain. No live jobs,
cross-chat messages, pack publication, staging or commits. Full311/original16
scope stays active. `git diff --check` passes.

First seven-map capture60820 completed0; all6 Goldenrod floor pairs plus Celadon2F
control actually inspected in `goldenrod-store-review/`. Blue source cases now
replace pastel books on2F/3F; source stock/electronics retained. Native counter
pigments applied on1F/3F. Scoped fixture-paint improvement only. Floor REJECTED:
old shared native floor algorithm reduced orange/cream bands to hairlines,
making the room overwhelmingly yellow. Source tile1 actually has45yellow,
14orange,5cream pixels. Current fix restores its exact45/14/5 area proportions
at16px slab scale (same source geometry pattern, no walking-height changes).
New area regression explicitly checks those three measures over fourcells.

Latest test15788 (`goldenrod-store-floor-tests.log`) and native19992
(`goldenrod-store-floor-native-build.log`) live; new actual all7 capture needed.
Earlier corrected modeled suite39576 passed165. Actual4F shelves still occlude
player body,5F kiosk hides actors,6F vending remains teal; plants/seats/architecture
also unfinished. Keep these defects open; do not claim full-floor acceptance.
Latest floor test15788 completed0:166 modeled regressions passed, including exact
45/14/5 pigment areas and256px² coverage/reservation/height check. Native19992
remains live; first floor capture is rejected pending fresh corrected images.

Previous turn yielded production-input theater walking evidence. Current actual
baseline2F comparison re-inspected: blue displays become pale pastel bookcases,
pixel-scale orange floor and large kiosk remain. Extended existing exact-source
reviewed retail family to Goldenrod1F–6F, existing source-bound8px counters to
store floors, and native three-pigment ceramic floor treatment to both stores.
Uses existing closed recessed display geometry/source fronts, not new assets.
Elevator excluded from native counter extension; unknown/custom/changed/clipped
display guards retained. Expanded existing retail regression across all6Goldenrod
floors plus Celadon control, checking claims/footing/frontUV/unknown/custom/art.

Live test24590 (`goldenrod-store-tests.log`, modeled_interiors) and native33660
(`goldenrod-store-native-build.log`), target/kanto-resume/receipts. Fresh actual
all6floor plus control comparisons required. Kiosk geometry/occlusion, vending,
plants and full room fidelity remain pending; no all-store acceptance. Full311/
original16 scope remains active. No cross-chat messages or pack publication.
Native33660 completed0 in4m20s. Initial test24590 passed164/failed1 older floor
domain-count assertion. Replaced that with intact excluded indices,256px² total
coverage and unchanged zero height; corrected suite39576 still live, receipt
`goldenrod-store-tests-corrected.log`. Seven-map capture now live (handle recorded
in next update) to `goldenrod-store-review/`, receipt `goldenrod-store-capture.log`.

## Theater production-input walking evidence, 2026-10-14

Previous turn made verified screen-material progress. Current turn used the
completed native screen build and actual browser pack; no production edits.
Physical-keyboard walk74025 paused because Mac is locked (confirmed CUA inventory),
then own process6236 deliberately terminated. No request to unlock needed: existing
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic injects accepted keys through production
input-owner dispatcher, with real focused=false retained in movement traces.

Completed four synthetic-input walks and read authoritative movement CSVs:
-52816 `theater-stage-walk-synthetic/`, start(1,6), UUUURRRRDDDD:
  ascends through(1,5),(1,4),(1,3),(1,2) to(1,1), eastward attempt blocked near
  dancer, then descends(1,3)->(1,5) and reaches(1,13). This is left ascent/drop
  proof, not cross-stage proof. Actual selected moving00/01/08/09 and final pair
  inspected. Final trainer at lower doorway is cropped; no entrance visibility
  acceptance.384frames,0extra terrain builds, median15.81ms/p9599.20ms.
-88993 `theater-right-approach/`, start(10,6), UD:
  authoritative(10,6)->5->4->3->2->3->5->6. Both moving frames/final pair inspected.
-87027 `theater-lateral-walk/`, start(1,3), RR:
  authoritative x1 through8 at y3; both moving frames/final pair inspected.
-55148 `theater-lateral-finish/`, start(8,3), R:
  authoritative x8->9->10->11 at y3; moving frame/final pair inspected.

Scoped evidence: both stage approaches and lateral row3 traversed using the
production input path; player/follower visible on reviewed stage frames, no
return of soil/seam or obvious sinking in those poses. These are diagnostic
location starts, not earned room entry/story progression. Final comparisons all
state_preserved=true but frame offsets129/63/48/27 respectively, so no matched
actor-pose/fidelity acceptance. Broader actor/support proof, entrance crop/pale
patches, source-faithful table/screen motifs, walking elsewhere and full311/
original16 requirements remain open. No live jobs, new builds, cross-chat
messages, pack changes, staging or commits. `git diff --check` passes.

## Native theater screen materials in verification, 2026-10-14

Scoped screen material result verified: native42548 completed0 in4m41s;
capture62516 completed0. Both actual pairs in `theater-screen-review/` inspected.
Screen now matches source blue/white/pale-blue palette instead of cream/green;
continuous timber stage, intact fascia, audience tatami and house control retained.
Theater45/LCD49 state preserved (4-frame offset, scenery only); house68/LCD68
exact. Six theater checks and13 model checks passed. Static material acceptance
only: screen motif fidelity/scale, actor support and stage walking still need
review. Full311/original16 scope remains active. No live jobs, cross-chat
messages, pack exports, staging or commits. `git diff --check` passes.

Previous turn resolved stage seam with actual production comparison. Current
source binds existing theater_backdrop material roles to its own source4e
swatches:85blue frame(0,0),255white panels(1,1),170pale-blue motif(4,5).
No geometry/placement/footing changes. Shared importer validates paper/paper_light,
oak/oak_cut/edge,ink/plum roles; unknown roles fail before emitting. New regression
preserves positions/normals/indices/cache and rejects each altered material.

Live test71850 receipt `theater-screen-tests.log` (filter theater); native42548
receipt `theater-screen-native-build.log`, target/kanto-resume/receipts. Fresh
actual default capture needed before screen acceptance. Stage walking/actor
support and full311/original16 scope stay open. No cross-chat messaging.
Test71850 completed0:6 theater checks passed; same fresh binary's dungeon-model
suite passed13 (`theater-screen-model-tests.log`). Native42548 still live.

## Theater board finish and cap replacement in verification, 2026-10-14

RESOLVED scoped seam/board defect: native99967 completed0 in5m12s; default
capture57700 completed0. Both actual `theater-terrace-review/` comparisons
inspected. Stage has continuous gold timber boards, no sawtooth or outdoor
soil/turf strip, and original access trim retained. Cause was outdoor terrace
finishing applying runtime jump edges through the already-authored stage,
introducing a second height field and soil drop. Exact successful stage ownership
now filters those edges; unknown/custom/clipped drawings get no override.

Five theater checks passed, including external pack plus jump-edge fixture.
Actual theater state preserved144/LCD156 (12-frame offset: scenery proof only);
house control112/LCD112 retained accepted mats/wood/furniture. No whole-actor pose
or stage walking approval. Backdrop still cream/green rather than source
blue/white, actor support/walking and broader311/original16 requirements remain.
Outdoor terrace regression suite now runs from the same fresh test binary,
receipt `theater-terrace-outdoor-regressions.log`. `git diff --check` passes.
Outdoor terrace suite1477 completed0:19 passed. No live jobs remain.
No packs staged/published, commits or cross-chat messages.

Domain isolation produced decisive evidence: native49222 passed4m33s; omit-base
capture55721 and omit-cap46877 completed0, actual pairs inspected in
`theater-without-base/` and `theater-without-cap/`. Sawtooth persists without base;
without cap, an outdoor soil/turf strip is exposed across the stage. Production
terraces::finish applies frame.jump_ledges, absent from prior synthetic fixtures.

Current source filters jump edges wholly owned by a successfully resolved
source-complete Theater placement before outdoor terrace finishing. No other
map/edge changes; claimed native8px stage retains support. External-pack test now
includes a south-facing jump edge through stage row8 and requires no geometry
above8px. Test61306 logs `theater-terrace-tests.log`; native99967 logs
`theater-terrace-native-build.log`, live. Fresh default capture required. The
stage is still unaccepted pending actual render. Domain diagnostics remain opt-in.
Actor support correction/walking still requires verification; full311 scope active.

Current continuation follows the rejected perimeter hypothesis with domain
isolation. Added opt-in CRYSTAL_THEATER_SURFACE_DIAGNOSTIC=omit_base/omit_cap
to stage append; default production geometry unchanged. After the single native
build, capture both variants sequentially to identify which surface owns the
sawtooth before another production edit. Test9554 receipt `theater-domain-tests.log`
and native49222 receipt `theater-domain-native-build.log` are live under
target/kanto-resume/receipts. Full311/original16 scope active; no messaging.

Perimeter hypothesis disproved: native72738 completed0 in5m01s, capture95362
completed0; both actual `theater-perimeter-review/` pairs inspected. Sawtooth
persists and relocating the foundation reduced visible access trim. Reverted
that production relocation; prior stage source restored. Kept actual external
pack regression covering complete source layout and upper stage bound, removed
the unsupported "no internal foundation face" assertion. Its full fixture now
allows legitimate below-cap decorative trim. The stage is still rejected.
No live jobs remain. Next diagnostic should isolate rendered domains/materials
at the seam instead of more foundation geometry speculation. Full311/original16
scope remains active; no cross-chat messaging. `git diff --check` passes.

Latest continuation adds actual external-pack scene regression21424, passed.
However its strict front bound also excluded the prototype foundation face at
79px. Current correction moves that exact south-facing foundation wall to the
80px outside perimeter, preserving cap/footing and other fascia. Expanded pack
regression now includes79px. Test47161 (`theater-perimeter-tests.log`, filter
theater) and native72738 (`theater-perimeter-native-build.log`) live under
target/kanto-resume/receipts. Fresh actual capture required; no seam acceptance.
Previous turn was diagnostic progress; full311/original16 scope remains active.
No cross-chat messages, pack export, staging or commits.
Initial perimeter test47161 failed on valid low decorative access relief at1px,
not a crossing foundation wall. Narrowed the new assertion to reject full-height
internal foundation wall at79px and geometry above8px while allowing trim.
Corrected test53995 completed0:5 theater/pack/full-scene checks pass, receipt
`theater-perimeter-tests-corrected.log`. Native72738 remains live. No visual claim.

Current continuation yielded diagnostic progress: full production builder test1907
passed, then expanded front-strip test50509 passed (covers interior z<31 instead
of z<16). Hardware-shadow-off capture78345 completed0 and actual pair inspected
in `theater-shadow-diagnostic/`; sawtooth persists, ruling out hardware shadows.
Full builder fixture uses source-complete theater, authored profiles and samples;
it finds no crossing faces after finishing. Latest diagnostic45083 adds an upper
8px bound to catch raised duplicate geometry (previous check allowed any high
face); receipt `theater-full-height-diagnostic.log`, live. Do not substitute
these geometry checks for visual acceptance: production stage remains rejected.
No production edits this continuation; new diagnostic regression and evidence.
No cross-chat messages. Full311/original16 scope remains active.
Height diagnostic45083 completed0: complete source-fixture scene has no extra
geometry above8px in the checked front strip either. Next compare actual pack
frame geometry/renderer state against this synthetic source-complete fixture;
do not infer the actual frame is identical merely from matching source patterns.
No live jobs remain; `git diff --check` passes. The stage is still visually rejected.

Latest native39477 completed0 in4m33s; capture59391 completed0. Both actual
`theater-board-seam-review/` pairs inspected. Sawtooth row persists, so removing
thin-cap sides did NOT solve it; stage remains rejected. House control preserved
55/LCD55, theater44/LCD48 state preserved scenery only. Test retry6137 passed3.
Next diagnostic test34495 (`theater-crossing-diagnostic.log`, filter
theater_atomic) asserts no crossing side faces inside the stage plot and prints
offending triangle positions if present. It is live; no new native build yet.
Do not claim seam acceptance or repeat the disproven thin-cap hypothesis.
Diagnostic34495 completed0: no crossing side faces exist in the isolated stage
append. This narrows the visible sawtooth to later production composition or
surface handling, rather than proving a cap-side cause. Inspected build_scene:
stage appended near887; world polish near2341 rejects unfamiliar topology;
interior surface finish follows near2360. Next inspect full built-frame mesh at
the rendered stage strip, including source shape/duplicate geometry and later
normal/shadow processing. No live jobs remain. Latest change is diagnostic only;
the current visual stage remains rejected, not complete.

Native37439 completed0 in6m09s; capture10773 completed0. Both actual pairs in
`theater-board-review/` inspected. Gold boards now read, house control remains
accepted57/LCD57. Theater45/LCD49 state preserved, scenery only. Stage is REJECTED:
retained thin-cap side triangles leave a sawtooth row across the top. Follow-up
also removes triangles wholly above the foundation seam7.79px before emitting
the new8px cap; lower foundation/sides/fascia remain. Latest test46591 receipt
`theater-board-seam-tests.log`; native39477 receipt
`theater-board-seam-native-build.log`. Both live; fresh actual capture required.
Test46591 failed compiling a transient unexpected delimiter in shared
mesh/outdoor_stone.rs:823. Current file parses with rustfmt; left it unchanged.
Retry test6137 logs `theater-board-seam-tests-retry.log`. Native39477 still live;
do not restart it because a different compile snapshot failed.

Previous turn made verified material progress but rejected the remaining plain
top/foundation ridge. Current append removes only the stage model's upward cap
triangles and emits a coherent horizontal timber finish over the same plot.
Source50 gold field,10board courses, staggered six-cell end joints and0.025px
recesses replace coarse foundation top exposure. Closed base/sides/access relief
and exact8px support/claims remain; backdrop and audience mats unchanged. Test
requires all emitted upward triangles stay within7.975..8px, exact8px vertices
exist, and original footing/atomic source guards remain.

Live theater test26508 receipt `theater-board-tests.log`; native37439 receipt
`theater-board-native-build.log`, target/kanto-resume/receipts. Fresh native
comparison required before acceptance. Full311/original16 scope stays active.
No cross-chat messages, pack publication, staging or commits.
Test26508 completed0:3 theater checks passed. Same fresh binary traditional
suite passed25 (`theater-board-traditional-tests.log`). Native37439 remains live;
no visual acceptance until its actual output is inspected.

## Dance Theater timber stage in verification, 2026-10-14

Latest native51932 completed0 in5m28s; capture83761 completed0. Actual theater
and house-control pairs in `theater-timber-cap-review/` inspected. Pale checker
and broad black top band removed; top now continuous source gold, vertical fascia
retains dark/light roles. House control preserved at exact54/LCD54. Theater state
preserved86/LCD93, scenery proof only. Stage still looks too plain and retains
a visible foundation ridge near its front: do NOT claim whole-stage acceptance.
Next correction should expose coherent timber board detail and remove that
unintended foundation lip while keeping the exact8px top and access columns.
Backdrop is also still cream/green rather than native blue/white. Original
stage walking/actors/full311 scope stays open. No live verification jobs remain.
Latest three cap/theater tests passed; earlier25 traditional/12 model checks pass.
`git diff --check` passes. No staged/committed pack or cross-chat messages.

First native4347 completed0 in9m; capture41370 completed0. Both actual theater
and Itemfinder control comparisons inspected in `theater-timber-review/`.
House retained accepted materials. Theater's checker is gone but first gold
stage is REJECTED: a visible foundation top produces a broad dark band and top
has insufficient timber finish. Follow-up maps every upward face to gold field
with subtle local grain; vertical fascia retains dark/light source roles.
Regression now requires all upward faces use gold, preserving geometry/cached
model. Latest test8156 receipt `theater-timber-cap-tests.log`; native51932 receipt
`theater-timber-cap-native-build.log`. Both live; fresh comparison required.
Test8156 completed0:3 passed, including every upward face using the gold field.
Native51932 remains live. The rejected first capture is not proof of this fix.

Previous turn was verified progress: house entrance gold floors. Current actual
DanceTheater comparison still shows an incorrect pale checker stage against
native gold timber. Existing stage geometry uses reed/reed_light materials.
Added validated theater-platform material roles to the existing dungeon model
importer, rendering that unchanged closed geometry with native tile50 swatches:
gold field(0,2), light trim(1,1), dark edge(0,0). Source-complete stage resolution,
native8px support/ownership and custom exclusions remain authoritative. Audience
tatami and backdrop geometry/materials are unchanged. Unknown material roles fail
before emitting geometry. New regression compares positions/normals/indices and
cached colors and rejects each changed role; existing atomic-stage test now
checks textured stage support as well as solid backdrop bounds.

Live test69216 receipt `theater-timber-tests.log` (filter theater), native16580
receipt `theater-timber-native-build.log`, under target/kanto-resume/receipts.
Fresh actual capture still required; no stage visual acceptance yet. Full311 and
original16 scope remains open. No cross-chat messaging or pack publication.

Initial test69216/native16580 failed compilation because the geometry-occlusion
constructor also creates Model. It now remaps the new stage roles consistently
with the existing semantic bindings. Corrected test69441 and native4347 are live,
receipts `theater-timber-tests-corrected.log` and
`theater-timber-native-build-corrected.log`. `git diff --check` passes.
Corrected theater test69441 completed0:3 passed. Same fresh binary traditional
suite76683 passed25 and dungeon-model suite13955 passed12. Native4347 still live;
actual stage comparison remains pending. No whole-stage visual claim from tests.

## Traditional entrance floor correction in verification, 2026-10-14

Completed scoped verification: native81545 completed0 in4m51s; capture90826
completed0. All8 actual `traditional-entrance-review/` comparisons inspected.
Four small houses and Kurt now show source-gold entrance floors instead of pale
cream; borders/red door mats and flat woven tatami retained. Mahogany/gift
controls retain gold flooring. Theater entrance is cropped in the standard pose,
so no visual acceptance for that unseen strip. Theater stage remains incorrect.

25 focused traditional tests and165 modeled-interior tests passed from the fresh
test build; ownership/footing preserved. All8 captured states preserved. Exact
frame/LCD Itemfinder146/146, Mahogany141/141, gift54/54; other5 have4–6 frame
offsets, so these establish scenery only. Whole-room scale/actor/walking,
furniture detail, theater stage and full311/original16 requirements remain open.
`git diff --check` passes. No pack publication/staging/commits or cross-chat messages.

Previous turn made verified progress (tatami craft gaps, timber seam/radio).
Current source targets the remaining pale entrance floor in the seven reviewed
traditional rooms. Only native tile1 cells of blocks5/6/7/2a qualify, with exact
phase masksfff0/33f0/ccf0/ffc0 and zero-height source guards. Door mat tiles4/14,
border16 and unknown maps are excluded. Reuses the verified gold floor material;
same-atlas fallback underlays now prefer that actual floor donor. Regression
checks all phases/maps, wrong art, preserved ownership/footing and output.

Test39768 completed0:25 focused tests pass (`traditional-entrance-tests.log`).
Native81545 remains live and logs
`traditional-entrance-native-build.log`, both under target/kanto-resume/receipts.
Fresh captures required after native success. No visual acceptance yet for this
latest change. Full311/original16 scope remains open; no cross-chat messaging.

## Tatami clarification and timber/radio correction in verification, 2026-10-14

Latest result: corrected native37836 completed0 in4m58s; capture73406 completed0.
All8 actual comparisons in `traditional-craft-floor-review/` inspected. Kurt's
craft-area pale rectangles are gone: source-authenticated tatami continues around
the tables/cushions. Six standard rooms retain the timber seam correction;
four small houses/Mahogany now have gold lower wall floor and separate radio.
Theater/gift controls retain prior appearance. Scoped static acceptance for these
mat/wood/radio changes only. Entrance pale floors, Kurt's source-drawn craft
furniture, theater stage, room scale/actors/walking and full311 scope remain open.

Final test90996 completed0:24 traditional regressions pass, including six new
mixed drawings, complete-source rejection and ownership checks. All8 comparison
states preserved. Exact frame/LCD: Itemfinder56/56, Kurt93/93, gift56/56; remaining
five differ4frames and provide scenery proof only. No whole-character acceptance.
`git diff --check` passes. No pack changes/staging/commits or cross-chat messages.

Update: first native build completed0 in10m52s; all8 comparisons in
`traditional-timber-radio-review/` actually inspected. Six standard rooms now
show timber at the corrected seam and the four small houses/Mahogany have a
separate gray radio. Itemfinder55/LCD55 state preserved. The lower wall strip
remains pale, so this is not full visual acceptance. Kurt craft-area pale tatami
gaps persist; theater/gift controls retained their previous appearance.

Follow-up source now paints the primary wall's lower half directly from its
authenticated floor50 gold field, rather than a generic underlay. Added exact
tatami drawings0x0c/14/23/33/35/36, preserving furniture cells and testing their
8/8/4/4/4/10 floor-cell masks, every damaged source cell and prop exclusions.
Latest verification: test2422 (`traditional-craft-floor-tests.log`, filter
`traditional`), native build30342 (`traditional-craft-floor-native-build.log`).
Native30342 failed on an invalid `interior_surfaces::` prefix (the file is
included in the parent module); the call is now direct. Corrected native build
37836 logs to `traditional-craft-floor-native-build-corrected.log`. Test2422
completed101:23 passed, one count mismatch. Retry60099 showed my first count
correction was assigned to the wrong block:0x0c correctly has8;0x36 has10.
Both exact counts are now corrected, rerun90996 receipt
`traditional-craft-floor-tests-final.log`. These edits postdate the
inspected timber images; fresh captures still required.
Do not restart live jobs. The native build30342 does NOT automatically capture.

Latest user clarification: these green floor surfaces are tatami mats. Preserve
the verified flat woven mats in all seven traditional Johto rooms; genuine blue
cushions remain separate objects. No new raised blocks or generic carpet.

New `mesh/traditional_wood_wall.rs` replaces the six standard rooms' fabricated
paper panels with source-colored timber at row2, the actual wall/floor seam.
The lower two rows stay horizontal gold floor; the existing radio model stands
separately there. Complete source-course hashes guard backing behind fixtures,
and original source/height/custom-profile guards constrain primary replacements.
DanceTheater is excluded from this wall correction. Footing is unchanged.

Verification is still running: test session19657 logs to
`target/kanto-resume/receipts/traditional-timber-radio-tests.log`. The native
build/capture wrapper PID77701 (cargo77703 when started) logs to
`traditional-timber-radio-native-build.log`, then captures all eight comparisons
in `target/kanto-resume/traditional-timber-radio-review/`. Its tool session ID
was truncated; inspect the existing process/logs rather than start a duplicate.
Initial test19657 completed101:163 passed, the backing test failed against its
old clock/cabinet fixture. Corrected fixtures now match the pack; corrected full
modeled suite8992 completed0:164 passed, receipt
`traditional-timber-radio-tests-corrected.log`. The same freshly built binary's
five `traditional_room` tests also passed, receipt
`traditional-timber-radio-tatami-tests.log`. All three production course
hashes independently recomputed from the current ignored pack and match. No actual
timber/radio image acceptance yet. `git diff --check` passes. All311 interiors,
original16 variants, walking/actor/scale/architecture defects remain unfinished.
No cross-chat messaging.

Additional actual Kurt comparison critique: the right craft area still has pale
rectangles where source tatami should continue. Its mixed craft blocks0x33,
0x35,0x36 (and left fixture0x0c) are absent from `tatami_drawing`; block0x23
contains radio plus tatami and also needs separate source treatment. The earlier
seven-room mat acceptance covers the already authenticated eleven drawings,
not every mixed craft footprint. Preserve the full source drawing and prop
ownership when extending this; do not recolor the craft tables into mats.

## All8 native clocks/shop floor pairs inspected, 2026-10-14

Native/all8 capture57391 completed0, build3m57s. Every actual pair in
`traditional-clock-floor-review/` inspected. Source-gold clock/cabinet/drawer
materials now replace pale cream in the six rooms containing these fixtures;
DanceTheater remains control. Mahogany's native gray stock,16px height and gold
floor/backing now agree with source; earlier pale green rectangles removed.
Continuous table cap/visible Itemfinder Pokédex,7green mats/blue cushions and gift
control retained. Scoped static material/backing acceptance only; rooms unfinished.

All8 state preserved. Itemfinder53/LCD53 and gift47/47 exact; other6 differ4frames
except Kurt71/LCD80 (9frames), so no exact actor-pose claims for these6. Stage
wood/paper palette, pale fabricated house walls/radio, entrance floor/stairs,
composition/walking/character fidelity remain defects. Source inspection confirms
traditional wall profiles own4rows but only upper2 are wall17; lower2 are floor50
or radio0a/0b/1a/1b. Current append defaults south_row=p.row+p.height (row4),
placing wall on lower floor strip. Next architectural correction must use real
wall/floor boundary(row2), retain horizontal gold floor and separate radio,
complete-source/custom guards and footprints/footing. Do not just repaint false
paper panels and declare room geometry accepted.

Latest cap test33400 completed0:2pass, including every upward timber face using
one gold UV plus geometry/trim/unknown-role preservation. Same fresh binary
modeled/tatami regressions now run receipts `traditional-clock-floor-interior-tests.log`
and `traditional-clock-floor-tatami-tests.log`; both completed0,162 modeled
and5 tatami passed. No new production edits this turn. Full311 rooms/original16 variants and all recorded
remaining requirements stay active. No cross-chat messaging.

## Clock/cabinet and gold shop-floor regressions passed, 2026-10-14

Latest combined test44626 completed0:162 modeled regressions passed1.29s,
including new seven-map/four-profile clock/cabinet guards and source-gold Mahogany
floor identity/height/footing test. External Vec2 import issue already corrected
in current source; no cross-chat messages. Focused latest upper timber-cap test
now compiles in `receipts/traditional-clock-floor-cap-tests.log` because the
broader upward-face UV fix postdates44626's compile snapshot. Native/all8
capture57391 remains live with1build job. Do not claim actual clock/floor visual
acceptance before its produced pairs are read. Existing continuous gold caps,
visible Itemfinder Pokédex and7tatami/blue cushions remain scoped verified;
Mahogany stock backing rejected pending correction. Full311 scope/original16
Kanto variants and every remaining room/furniture/actor/interaction defect stay
active. Current turn produced material/backing source fixes,162passing tests,
actual all8 critique and continued latest native verification.

## All8 cap/stock pairs inspected; clock and backing follow-up, 2026-10-14

Native/capture38673 completed0, build6m52s. All8 actual pairs in
`traditional-cap-shelf-review/` inspected. Four small-house caps are continuous
gold rather than alternating pale boards; Itemfinder Pokédex remains visible at
exact frame55/LCD55, state preserved. All7 mats/cushions and gift-shop control
retained. All8 state preserved; gift50/50 exact, other6 have4frame offset.
Mahogany stock is now source-authentic gray merchandise on lower16px closed
fixtures, but pale green floor backing beneath it is REJECTED. Do not accept
whole shelf/room from height and goods alone.

Implemented backing fix: Mahogany shelf uses actual gold floor tile1 donor;
only native floor identities5row>=1 and32..34 tile1 qualify for source-gold finish.
Unknown/annex maps, invalid tiles/blocks and door band remain excluded. New floor
regression preserves3.25footing/ownership and checks every valid source phase.
Also expanded source-gold clock/cabinet treatment to7 reviewed traditional maps,
using exact2x4 clock0x13,2x3 cabinet0x13/0x29 and2x4 drawer0x29 drawings, strict zero-base/
bounds/material guards. Reuses existing model geometry and native roles; new
regression covers7maps/4profiles, claims/footing/height, changed identities and
unknown/custom-base fallback. Upper timber cap/bevel faces now all use one gold
sample, retaining light vertical trim to avoid mixed-UV specks.

Initial clock test4293 failed external missingVec2 imports in modeled_exteriors/
new_bark. Current source already contains both imports; no edits/messages sent.
Latest combined test44626 lives in `receipts/traditional-clock-shop-floor-tests.log`.
New1job native+all8 capture57391 runs receipts `traditional-clock-floor-native-build.log`
and `traditional-clock-floor-capture.log`, fresh `traditional-clock-floor-review/`.
These postdate38673. No visual clock/backing approval yet. All311 interiors and
original16 Kanto variants remain active, with architecture/actors/interactions
and every recorded unresolved finding still open. No cross-chat messaging.

## Continuous tabletop cap regression passed, 2026-10-14

Retry96483 completed0:2 relevant tests passed (new immutable continuous-cap/
edge-trim/material rejection regression and seven-map table source/height/claims
regression). New native/all8 capture38673 remains live with1build job. Same fresh
binary modeled_interiors/tatami regressions now run in
`receipts/traditional-cap-shelf-interior-tests.log` and
`receipts/traditional-cap-shelf-tatami-tests.log`; consume terminal results.
Same fresh binary completed160 modeled and5 tatami regressions successfully.
All8 previous table/Pokédex comparisons inspected, new continuous cap and stock
still require their own actual images. Do not accept striped86412 tabletop.

Root pack owner reports newer scene-constant/Surf-preserving canonical packs
verified/published; current queued captures will consume canonical paths.
No pending packs/competing exports/audio changes by this goal. Root source-edit
hold on crystal-bevy/Cargo manifests released; this task did not edit them.
Latest relayed human instruction stops cross-chat messaging; send no further
coordination messages or acknowledgments. Preserve shared changes through narrow
source edits and read-only inspection.
Full311 interiors/original16 variants and outstanding quality/interaction work
remain open. Current turn made actual all8 critique, continuous-cap source repair,
passing relevant tests and resource-conscious build progress.

## Table/Pokédex actual all8 review; continuous cap follow-up, 2026-10-14

Combined native86412 completed build0 in14m31s and all8 captures0. All actual
pairs in `traditional-table-prop-review/` inspected. Itemfinder Pokédex is now
visible at upper-left tabletop at exact frame46/LCD46, state preserved, actors3;
source actor/footing unchanged. Scoped support/visibility accepted at this pose.
All7 tatami/cushion correction retained; gift-shop control retained. All8 state
preserved. Kurt41/41, Mahogany40/40, gift41/41 exact; other4 have4frame offset.
Pale wall/clock/cabinet/stage and prior stock defects remain open.

REJECT gold tabletop finish in the4 small houses: broad alternating light boards
replace source continuous gold wood. Added Model::append_native_traditional_table
using existing native low-table roles; every upward cap uses one source gold field
with subtle immutable model-local grain, lighter material stays on side trim.
Original positions/normals/indices/height remain. New cap regression covers
uniform upward top UVs, retained side trim, immutable cached source and unknown
material rejection before mutation. Initial test36458 failed from two invalid
float literal spellings and owned String parse argument; corrected in place.
Retry96483 uses1build job in `receipts/traditional-table-continuous-cap-tests-retry.log`.

Mahogany shelf test99360 completed0:160 modeled regressions passed2.31s for
new source stock/16px height, claims/footing/damaged art/unknown map preservation.
New combined continuous-cap+Mahogany native/capture38673 uses1build job;
receipts `traditional-cap-shelf-native-build.log` and
`traditional-cap-shelf-capture.log`, fresh output `traditional-cap-shelf-review/`.
This postdates86412; inspect all8 again before cap/shelf visual acceptance. Root
pack owner requested reduced new concurrency; respected. No crystal-bevy or
Cargo-manifest edits. Full311 interior/original16 variant objective remains open.

## Current verification handles revalidated, 2026-10-14

Combined native/all8 capture86412 and Mahogany shelf test99360 both re-polled
live. Current test compiler pid54007/parent54000 was confirmed running with
CPU activity; no terminal failure or observation-timeout restart. Native receipt
still has compile warnings, no completed build line/capture output yet. Preserve
same jobs and source snapshot distinctions:86412 checks gold tables/Pokédex support,
99360 includes newer Mahogany shelf extension. No production source changes during
this turn's verified waits. Full311 interiors/original16 variants and recorded
quality/interaction defects remain active; do not mark blocked while compilation
and independent visual work can advance.

## Mahogany stock fixtures source correction implemented, 2026-10-14

MahoganyMart1F shelves reuse the exact complete block02 drawing already guarded
and reviewed in MountMoonGiftShop. Extended native_gift_shop::source only for
Mahogany GiftShelf kind: source merchandise top/front retained on closed volume,
16px physical height replaces generic30px pastel bookcase. Other Mahogany kinds
and unknown/annex maps stay excluded. Existing regression now checks16claims,
1.25footing, source-textured output,16px height, unknown map and every changed
art cell for this second shop. Test99360 lives in
`receipts/mahogany-native-shelf-tests.log`; no visual shelf approval yet.

Combined table/Pokédex native/capture86412 remains live. It predates this shelf
extension, so inspect its all8 pairs for table/prop/tatami only; Mahogany stock
will still be a recorded defect until a subsequent source build/capture. Do not
restart this job from quiet output. Latest source support/custom ownership160,
tatami5 and actor38pass/4ignored remain separate regression evidence, not actual
rendered Pokédex proof. Preserve canonical ignored packs and all unrelated source
changes. Current turn makes concrete production stock/height progress; full311
scope/original16 variants and every unresolved finding remain active.

## Related actor and tatami regressions completed, 2026-10-14

Same fresh support-source binary: traditional_room5pass0fail; actor run56273
completed0 with38pass/4ignored in133.99s. Preserve those ignored cases as unproven;
these are related regression checks, not proof of final rendered Pokédex. Combined
160modeled+5tatami+38actor passing checks establish unchanged support/ownership
paths. Native/all8 capture86412 remains live in
`receipts/traditional-table-prop-native-build.log`; no final table/prop visuals
produced yet. Consume same job, inspect every produced pair, fix defects again
before scoped acceptance. Full311 interior/original16 variant objective stays
active and unfinished, with all recorded architecture/furniture/actor findings.

## Tabletop support/custom ownership tests pass, 2026-10-14

Latest test81296 completed0:160 modeled regressions passed4.31s, including every
inside/outside support cell,2xscale, custom table ownership exclusion, all source
identity changes, source-gold paint, unchanged4px/16claims/1.25footing. Same fresh
binary traditional_room and new_bark_actors regressions now executing with
receipts `traditional-table-prop-tatami-tests.log` and
`traditional-table-prop-actor-tests.log`. Read terminal results before claims.
Native/capture86412 remains live; no latest visual table/prop acceptance yet.
All prior scoped source-tatami/blue cushion acceptance and full311 room/original16
variant scope preserved. Do not mistake160passing tests for rendered Pokédex
proof. Shared new_bark_actors large existing diff belongs to other work; only
narrow source-table support/anchor branch added here, preserve all other edits.

## Latest combined table/prop verification launched, 2026-10-14

Intermediate gold-table native49447 completed0 in8m55s. No captures from it:
it predates the latest Pokédex support/custom-document guards. New combined
native build and automatic all8 capture86412 launched; receipts
`receipts/traditional-table-prop-native-build.log` and
`receipts/traditional-table-prop-capture.log`, output
`traditional-table-prop-review/`. The job launches captures only after actual
successful Cargo exit. Test81296 remains live in
`receipts/traditional-table-prop-support-tests.log`; helper tests cover every
inside/outside table cell, scaled height, custom ownership, every changed
source identity and unchanged16claims/1.25footing. No new visual acceptance yet.
Canonical ignored published pack consumed; no pending packs/exporter/audio
mutations. Previously verified tatami7+gift control remains scoped acceptance;
full311 rooms/original16 variants and all outstanding furniture/architecture/
actor findings remain open.

## Source-gold table tests pass; tabletop prop support implemented, 2026-10-14

Gold-table test16521 completed0:160 modeled regressions passed in0.93s. Existing
4px table height/16claims/1.25footing/source roles pass. Native49447 remains live
but predates the following support helper; do not use it for final prop approval.

Implemented narrow tabletop support for stationary Pokedex Object actors:
resolve a complete native1b/1c/18/19 table under the committed terrain profiles,
require selected low-table native materials, anchor flat decoration at source
center only with supported table, render at modeled table top. Actor/gameplay
state and walking footing remain unchanged; ground fallback outside valid tables.
New helper in mesh.rs/modeled_interiors; small new_bark_actors hook coordinated
with outdoor owner. Custom documents reserve their drawing in all7 traditional
rooms, and surface finish respects user-owned exclusions. Test extends gold-table
regression across every inside/outside cell,2xscale,custom ownership, every
changed source field; no source support over missing/changed/custom tables.
New retry now lives in `receipts/traditional-table-prop-support-tests.log`.

After native49447 completes, rebuild latest support source and capture all7
traditional rooms plus gift control to a fresh directory. Verify actual orange
Pokedex visibility and tabletop anchor in Itemfinder at exactphase, source-gold
table appearance, and retained tatami/cushions. Full311 interiors/original16
Kanto variants, architecture, other furniture and all outstanding findings remain
open. Current turn made production support/custom-ownership progress plus160test
verification; no blocked/completed status applies.

## All7 tatami/cushion rooms and gift control inspected, 2026-10-14

Capture19273 completed0; all8 actual final native73670 pair images inspected:
DanceTheater, KurtsHouse, EcruteakItemfinderHouse, EcruteakLugiaSpeechHouse,
MahoganyMart1F, VioletKylesHouse, VioletNicknameSpeechHouse, MountMoonGiftShop.
Scoped static acceptance for continuous zero-height green source tatami and
blue/white genuine cushions in all7 reviewed Johto rooms; gift-shop separate
native finish retained at inspected pose. Tests159+5 passed on final tatami
source after rock-call correction. No gameplay/whole-room acceptance.

All8 state preserved. Itemfinder frame41/LCD41 and gift42/LCD42 exact; other6
have4frame offset, so no exact actor-pose claim. Remaining pale wall/cabinet/
clock/stage surfaces, Mahogany shelf identity/stock, player visibility, broader
house architecture and missing tabletop pokedex support remain open.

Gold table source correction now tests16521/build49447 in
`receipts/traditional-gold-table-tests.log` and
`receipts/traditional-gold-table-native-build.log`; these postdate the accepted
scoped tatami executable. Table geometry4px/16sourceclaims retained; exact
1b/1c/18/19 phase/source guards and3native material roles. Inspect new table
views after these complete; no visual table approval yet. Full311 scope and
original16 Kanto variants remain active. Current turn made production table
progress plus actual all8 visual verification; no blocked/completed status.

## Final tatami views improve; source-gold low tables implemented, 2026-10-14

Native73670 completed0 in14m47s. Capture19273 now producing all8 final tatami
comparisons in `tatami-blue-cushion-review/`. Actual DanceTheater, KurtsHouse,
EcruteakItemfinderHouse, EcruteakLugiaSpeechHouse, MahoganyMart1F pairs inspected:
continuous native green weave around all mixed props and blue/white genuine
cushions confirmed; tan patches and purple substitutions removed. Remaining3
pairs pending. Theater/Kurt frames42/LCD46 and40/LCD44 retain animation offset;
no broad actor-pose approval. Walls/stage pale finishes, Mahogany pastel shelves,
traditional pale tables and Itemfinder missing pokedex actor remain defects.

Final source retry92141 completed0:159 modeled regressions passed3.35s after
external rock call correction. Same fresh binary traditional_room run also
completed0; read `receipts/tatami-final-source-traditional-tests.log` for5 tests.
These establish tatami/cushion preservation; not whole-room completion.

Next gold table correction implemented using existing native low-table roles:
complete4x4 1b/1c/18/19 metatile/subtile/art guards within the7 reviewed maps,
existing4px height/16claims preserved, source-dark/gold/light-gold swatches from
tiles33/15. In-memory source graphics verified15[1,0]=170,33[0,0]=0,
33[1,0]=255; no pack/catalog artifacts written. Unknown models/materials retain
source fallback. New regression covers7 maps,16claims/1.25footing/4pxheight,
native3UVs, unknown maps and all changed tile/block/subcolumn cases.
Test16521 is live in `receipts/traditional-gold-table-tests.log`; native49447 in
`receipts/traditional-gold-table-native-build.log`. These postdate tatami73670;
no table visual acceptance yet. Full311 scope/original16 variants remain open.

## External rock-test compile mismatch repaired; final retry, 2026-10-14

Test38605 terminal101: E0061 external mesh/kanto_boundary_rocks append tests
lacked new sixth map_origin argument. Exact original lines321/342 reported to
asset-variation and outdoor owners. Current source now has both calls corrected
with[0;2] at353/374; preserved their implementation. Final interior retry now
runs `receipts/tatami-final-source-rock-fix-tests.log`. Native73670/capture19273
still live on unchanged interior snapshot; no restart from timeout.

Read current actor source: Pokedex prop already exists in johto_actor_props,
mesh boundsY0..0.148, and new_bark_actors places props on ground footing. At64px
actor scale its9.472px height is below the4px source table's16-world-pixel top,
consistent with being hidden below the worktop. This is a source-backed support
hypothesis requiring actual transform/visual verification, not proof of a fix.
Prior people thread `Redo Outside` is idle under a transfer instruction and
will not repair it; do not assume that notification means implementation.
Next inspect render-only tabletop support/flat-prop anchor and coordinate shared
actor file before editing. Never alter actor/gameplay state or add duplicate book.

## Final custom-base cushion regression queued, 2026-10-14

Added explicit7px custom-base rejection assertion to the cushion regression;
production guard was already included in native73670 snapshot. Fresh final
modeled_interiors Cargo run38605 lives in `receipts/tatami-final-source-tests.log`,
covering the latest source rather than the earlier post-guard snapshot gap.
After success run traditional_room on this same fresh binary, then consume
native73670/capture19273 and inspect all8 actual comparisons. No further production
edits while this snapshot verifies. Source table and missing pokedex actor work
remain open; no broad completion/blocked claim. This turn adds a meaningful
custom-placement preservation check plus authoritative actor diagnosis.

## Itemfinder missing object identified as actor, 2026-10-14

Fresh native96947 EcruteakItemfinderHouse state receipt actually read: frame60/
LCD60, actors3, Object(1) source `pokedex`, center[-96,64]. The source-orange
book-like tabletop object is an authoritative object actor, not missing table
mesh detail. Sent actual receipt/comparison to existing people/object owner;
do not invent a duplicate static book on the table. Table finish remains an
independent issue. profiles.json currently has cushion10/1b/19 and clock/
cabinet/wall/radio bindings, no traditional table profile. Table resolver is
house_table_placements, complete4x4 tile art then generic LowTable at4px. Next
source-gold table guard should also verify actual metatile/subtile identities
across1b/1c/18/19 for the four small houses, preserve other layouts/custom art.

Native73670 and its dependent all8 capture queue19273 are confirmed live; keep
same handles. No source edits this turn while final tatami build advances.
This turn adds actionable authoritative actor diagnosis and verified waits;
full311 scope/original16 variants remain active, not blocked or complete.

## Full mixed tatami and genuine cushion regressions passed, 2026-10-14

Test64985 completed0:5 traditional-room regressions passed, including all11
mixed source drawings, damaged/claimed-floor rejection, valid prop ownership,
zero-height native mats and exact fullblock floor/footing streams. Direct fresh
test binary run31438 completed0:159 modeled_interiors regressions passed in5.39s,
including native blue/white cushion profiles10/1b/19 in all7 rooms with4claims,
3.25 footing and damaged/unknown-map rejection. Receipt
`receipts/tatami-blue-cushion-interior-tests.log`. The final added nonzero-base
fallback guard follows this compiled test snapshot; native build includes it.

Intermediate native28804 completed0 in10m52s but predates final overrides/
cushions; no new captures from it. Latest full native73670 now runs in
`receipts/tatami-blue-cushion-native-build.log`. Capture queue19273 waits
for confirmed Cargo pid42078 to exit and requires the successful build receipt
before launching all7 rooms plus MountMoonGiftShop control. Output is
`tatami-blue-cushion-review/`, receipt `receipts/tatami-blue-cushion-capture.log`.
Keep consuming73670/19273; actually inspect every produced pair,
then proceed to traditional table/wall/furniture work. No source completion
claim from passing tests. Current source diagnosis: traditional complete4x4
plain table uses4px low-table generic finish; native low-table material roles
already exist (rubber/timber_gold/timber_light). Source-gold top and Itemfinder
orange tabletop object need independent source-bound corrections. Full311 scope
and all original16 variants remain unfinished. This turn yielded verified waits,
passing required regressions and latest native build initiation.

## All27 gate source-floor comparisons reviewed, 2026-10-14

Capture38524 completed0. All27 actual paired images now inspected, including
final6 Route5/6UndergroundPathEntrance, Route7/8SaffronGate,
SafariZoneFuchsiaGateBeta and VictoryRoadGate. Source edge-centered diamond
floor, green8px joined counters, blue native walls/doors pass scoped static
material/height review; complete drawing and footing regression coverage was
158pass on final source. Source-frame offsets are retained in per-map receipts;
no full actor-pose/whole-room approval. Last6 state preserved, Route5Underground
frame37/LCD41; other5 exactframes. Source floor shape/palette stays consistent
through irregular VictoryRoad corridor. Phone cyan paint, plants/pots, terminal
fidelity, subway stair geometry and Route40 actor occlusion remain open.

All8 first tatami comparisons inspected; fullblock04 fix confirmed, mixed floor
and genuine cushion pigment follow-up remains under verification. Retry64985
is live in `receipts/tatami-cushion-floor-tests.log`; includes exact mixed-source
Flat override and11 mixed drawing guards. Native28804 predates latest guards/
cushion changes and must finish before latest native rebuild. No blocked or
complete goal status applies; full311 interiors/original16 variants remain open.

## Mixed tatami regression exposed generic raised-floor rule, 2026-10-14

Test89623 terminal101:4 pass/1 fail. Mixed mask expected12 floors but resolved0:
interior.rs generic traditional relief includes54/55 weave as raised fixtures.
Corrected with crate-visible mesh::traditional_tatami_floor_source and exact
map/source Flat override in profile::shape_for_source_on_map. Outdoor owner
notified; no outdoor/build_scene code changed. New source/test coverage includes
small-house table/seat blocks18/19/1b/1c as well as previous7 mixed drawings.

Genuine cushions now opt into existing native gift cushion rendering in the
seven Johto maps: complete2x2 02/03/12/13 drawing, exact origins in0e/10/19/1b,
strict material fallback, native blue/white top retained, original height/claims
and footing. New modeled regression covers existing10/1b/19 profiles, damaged
art/unknown-map rejection,4claims and3.25 footing. These source edits are newer
than native28804; do not accept its later images as final. Test retry running in
`receipts/tatami-cushion-floor-tests.log`. Need fresh native after28804 finishes.

Capture99055 completed0; all8 native96947 pairs actually inspected including
VioletNicknameSpeechHouse and MountMoonGiftShop control. Block04 green correction
confirmed, mixed floors and genuine cushion colors rejected pending follow-up.
Gift shop retained separate source finish/blue seat, no apparent regression at
this inspected pose. Fullinterior acceptance still absent. Gate batch38524 live.

## Additional current tatami comparisons inspected, 2026-10-14

MahoganyMart1F and VioletKylesHouse actual native96947 pairs inspected: continuous
fullblock green tatami and removal of fake purple floor pads confirmed. Tan mixed
floor patches remain pending newer native28804; Mahogany shelves remain pastel
book-like replacements, Kyle table/walls pale, real seats purple instead of blue.
Current capture99055 has6 actual reviewed pairs,2 controls/rooms pending. Preserve
separate acceptance: native96947 fullblock fix passed4+158 tests, newer mixed fix
still compiling in89623/28804. No whole-room acceptance or goal completion.

## Mixed tatami follow-up from actual images, 2026-10-14

Fresh native96947 pairs actually inspected: DanceTheater, KurtsHouse,
EcruteakItemfinderHouse, EcruteakLugiaSpeechHouse. Fullblock04 green mats now
remain floor and false pads are removed. REJECT full tatami acceptance: mixed
seat/table/stage-edge drawings still contain generic tan patches; genuine blue
seat art also becomes purple cushions, pale gold furniture/walls and missing
Itemfinder orange tabletop item remain defects.

Mixed tatami follow-up implemented in traditional_room_source/room and surface
finish: complete drawing guards for blocks0e/10/11/15/2e/2f/30, floor-cell-only
mask, preserve authored prop cells, reject any changed source and any excluded
floor cell, retain native green source UVs. Known genuine prop ownership does not
recolor surrounding floor. New seven-map regressions require12/12/10/10/8/8/8
floor cells and reject damage in every drawing cell. Test89623 lives in
`receipts/tatami-mixed-floor-tests.log`; native28804 in
`receipts/tatami-mixed-native-build.log`. These are newer than capture99055;
re-capture all8 after mixed native success before visual acceptance.

Six more actual Kanto gate pairs inspected: Route16Gate, Route17Route18Gate,
Route19FuchsiaGate, Route2Gate, Route5SaffronGate, Route6SaffronGate. Total21
reviewed,6 pending. All state preserved; Route2 frame42/LCD46, other5 exactframe.
Source diamond floor/low counter consistent; phone and plants remain wrong.
Original311 scope and16 variants stay unfinished. This turn made concrete
production progress driven by actual visual rejection and tests4+158 passed.

## Tatami native success and capture start, 2026-10-14

Native96947 completed0 in10m21s with the tatami correction. Capture queue99055
observed actual Cargo exit and successful receipt, and now runs all8 requested
comparisons (7 Johto mats plus gift-shop control); `tatami-source-review/` and
`receipts/tatami-source-capture.log`. Continue this same handle. No visual tatami
acceptance until actual images are inspected. Tests4+158 already passed.
Route43MahoganyGate actual final pair inspected, frame67/LCD67 state preserved:
all14 Johto gates now reviewed for scoped floor/counter changes. Route15FuchsiaGate
also actually inspected: repeated cyan phone/pale plants remain, source diamond
floor/native8px counter consistent. Remaining12 Kanto comparisons pending;
whole room, original variants, and full311 scope remain unfinished.

## Tatami capture queued behind confirmed native process, 2026-10-14

Native96947's actual Cargo pid32745/rustc32751 still confirmed live. A separate
capture queue99055 waits for that exact Cargo process and requires the build receipt's
successful Finished wasm-dev line before launching all7 tatami rooms plus
MountMoonGiftShop control. Output will be `tatami-source-review/`, receipt
`receipts/tatami-source-capture.log`; do not claim images until produced and read.
No competing export or pack mutation involved. Gate38524 continues separately.
Route42EcruteakGate and Route43Gate actual pairs inspected, state preserved,
frames83/LCD83 and43/LCD43 respectively: total13 reviewed,14 pending. Floors and
counter height pass scoped static review; phone/plant/readability work stays open.

## Exact tatami preservation verified; eleven gate pairs inspected, 2026-10-14

Test33299 completed0: all4 traditional-room tests pass, including exact native
floor position/normal/UV/index streams, nonzero3.25 footing and no invented
cushion placements across all7 Johto tatami maps. Fresh binary's combined
modeled_interiors run17155 also completed0:158 passed in9.05s, receipt
`receipts/tatami-combined-interior-tests.log`. Native96947 remains live; visual
acceptance of tatami still requires fresh all7 plus gift-shop control captures.

Seven additional final gate pairs actually inspected: Route34IlexForestGate,
Route35GoldenrodGate, Route35NationalParkGate, Route36NationalParkGate,
Route36RuinsOfAlphGate, Route38EcruteakGate, Route40BattleTowerGate. Total11
actually reviewed,16 pending. Diamond source phase and low joined counters
remain consistent. National Park pale PC and block-like fixtures remain wrong;
Route40 right NPC partly obscured at desk end, so whole-room visibility is not
accepted. Route34 Butterfree spread/scale finding sent to existing rig owner.
All7 state receipts preserved. Route36NationalPark frame42/LCD42 and
Route38Ecruteak frame41/LCD41 match; the other5 have offsets4 or5, preventing
exact actor-pose claims for those rooms. Capture38524 still live. Gate phone source drawing diagnosed in-memory:
block3f combines tiles0e/0f/20/21 on same claimed2x2 plot; original model exports
9 named primitive materials (teal,enamel_light,enamel,ink,screen,screen_light,
rubber,key,brass). No phone pigment fix implemented yet. Fullscope remains open.

## Tatami test result and four fresh gate reviews, 2026-10-14

Tatami test32568 completed0: all4 traditional-room regressions passed. Stronger
exact position/normal/UV/index preservation and nonzero3.25 footing assertions
were added after compilation; test33299 now runs those in
`receipts/tatami-exact-floor-tests.log`. Native96947 remains live; after success
capture all seven tatami rooms plus unaffected MountMoonGiftShop to a fresh
folder and inspect every pair before accepting the correction visually.

Actual final-source gate pairs inspected: IlexForestAzaleaGate,
Route29Route46Gate, Route31VioletGate, Route32RuinsOfAlphGate. Correct diamond
floor/green8px counter observed; state preserved, Route29 frame72/LCD76,
Route31 frame51/LCD55, Route32 frame37/LCD41, so these do not prove exact actor
animation acceptance. Pale cyan phones and sparse pale plants/pot paint remain
open. Remaining23 pairs are pending; capture38524 still active. This turn made
progress through stronger preservation assertions and actual image inspection;
full311 scope and original16 variants remain active and incomplete.

## Tatami native build and first fresh gate inspection, 2026-10-14

Native96947 now builds the tatami correction in `receipts/tatami-native-build.log`.
Gate capture38524 remains live; IlexForestAzaleaGate actual paired PNG inspected:
edge-centered cream/gray diamond floor and lower green counter show the guard,
while pale cyan phone and sparse pale plants/terracotta remain wrong. Do not
claim all27 gate acceptance from this first room. The batch starts with native76098
and subsequent children may load native96947 after publication; gate production
source is unchanged, source hashes recorded in ignored receipt
`receipts/tatami-build-gate-source.sha256`. Check every pair's state/frame receipt.
Test32568 remains live and predates stronger exact floor stream/3.25 footing
assertions added afterward; rerun tests after it completes. Full scope stays open.

## Tatami correction implemented; verification pending, 2026-10-14

User confirmed these are tatami mats. Exact block04 eligibility now covers all
seven reviewed Johto traditional rooms using that drawing. The shared cushion
placement guard therefore removes the counterfeit raised purple pads; genuine
cushion profiles remain intact. Complete floor drawings retain native green
weave/borders at zero height instead of the generic tan finish. Tests expanded
across all seven rooms for source changes, excluded cells, world phase, distorted
UVs/heights, unchanged footing/ownership and no cushion placements. Unknown maps,
barn/Wise Trio and gift-shop eligibility remain excluded. Test32568 is live in
`receipts/tatami-source-tests.log`; no visual acceptance yet. Kurt baseline actually inspected: mixed native green/tan flooring, purple seat substitutes, pale furniture and walls versus source gold; these remain visual recheck targets. Capture38524 is confirmed live with its Ilex child; preserve this batch before rebuilding the shared native executable.

Final gate test15628 completed successfully:158 passed. Native76098 completed0
in12m50s. All27 gate captures now running to `gate-final-source-review/` with
`receipts/gate-final-source-capture.log`; this executable predates tatami edits.
Continue actual pair inspection before gate floor acceptance. Full311-interior
scope and all outstanding architecture/furniture/interaction work remain open.

## Traditional small-house baseline review complete, 2026-10-14

Actual optimized comparisons for EcruteakLugiaSpeechHouse and
VioletNicknameSpeechHouse now inspected, alongside earlier Itemfinder/Kyle
reviews. Both repeat purple counterfeit tatami cushions, pale/cream walls and
furniture, striped pale table instead of source gold. All four small-house
baselines are actually reviewed; source eligibility fix remains pending.
No production source changed while final gate native76098/tests15628 are live.

## Final gate compile revalidation and tatami scope, 2026-10-14

Test60413 is terminal101 from external new_bark_models Vec2 import E0433 at
624/632/633/634. Outdoor owner corrected its import; new final interior retry15628
is live in `receipts/gate-final-source-tests-vec2-retry.log`. Native76098 remains
live in `receipts/gate-final-source-build.log`; no new gate source edits.

Read current ignored pack metadata/graphics in memory only. traditional_house
atlas has10 maps: DanceTheater, EcruteakItemfinderHouse, EcruteakLugiaSpeechHouse,
KurtsHouse, MahoganyMart1F, MountMoonGiftShop, Route39Barn, VioletKylesHouse,
VioletNicknameSpeechHouse, WiseTriosRoom. Block04 occurs in the theater/Kurt,
four Violet/Ecruteak houses, MahoganyMart1F and gift shop. Gift shop already
excludes fake cushions; barn/Wise Trio use distinct floor drawings. Next tatami
fix should scope complete block04 eligibility to the seven Johto rooms and
preserve gift-shop finish and distinct barn/Wise Trio art. No generated game
catalog/pack/audio artifacts were written. Do not treat map inventory as review.

## Current final gate verification queue, 2026-10-14

Species owner corrected the HashSet annotation in its rig source. Current
final-source interior Cargo test60413 is live in `receipts/gate-final-source-tests.log`.
Final-source native build76098 remains live in `receipts/gate-final-source-build.log`.
Gate source is unchanged after launch. Continue consuming these same handles;
then capture all27 explicit reviewed civic rooms to a fresh directory and
actually inspect each pair. Source phase is cream north/south, gray east/west,
joining edge-centered diamonds; footprint/height/color coverage tests require
both32-square-unit halves and positive winding. No final floor acceptance yet.

The previous goal turn made progress and this turn yielded source-diagnosis
and verified waits: traditional block04 tatami is incorrectly treated as cushions
outside two maps, and two concrete external compile errors were identified and
corrected by their owners. Preserve these handoff findings and the full311 scope;
no blocked/completed status applies while native/testing work remains live.

## Latest gate build/test handles, 2026-10-14

Intermediate native51715 completed0 in10m06s. It preceded the final source
triangle orientation. Final native build76098 is now live in
`receipts/gate-final-source-build.log`; no gate source edits after launch.
Final-source test13027 is terminal101: new unrelated species_rig.rs:1297 test
HashSet::default hasher inference E0283. Model owner notified with exact receipt
`receipts/gate-source-phase-tests-retry.log`; preserve its source and retry after
owned correction. No renderer goal blocker status is warranted: native final
build and source review can proceed. Next capture all27 civic maps and inspect
actual paired images; do not accept the superseded checker or intermediate phase.

## Traditional-floor root cause and gate retry, 2026-10-14

Purple tatami defect is traced to mesh.rs::traditional_house_cushion_placements:
block04's exact tatami drawing (house.rs::traditional_cushion_local) becomes2x2
Cushion objects unless traditional_room::floor_map excludes the map. floor_map
currently excludes only DanceTheater/KurtsHouse plus a separate gift-shop guard.
VioletKylesHouse/EcruteakItemfinderHouse/MahoganyMart1F therefore get invented
purple seat geometry over actual continuous green flooring. Next traditional
pass should correct source eligibility/complete drawing guards, not recolor fake
cushions. Preserve real cushion block19 and all floor/ownership invariants.

Final gate source-phase test32065 failed E0308 in another goal's shore_blend.rs
mixed3/4 color arrays; outdoor owner notified and fixed its owned source. New
test13027 is live in `receipts/gate-source-phase-tests-retry.log`. Native51715
still live; do not restart from observation timeouts. Its snapshot predates final
source triangle orientation; rebuild normally after it completes, then capture
all27 civic maps. No newer gate production edits are pending.

## Additional traditional-room review, 2026-10-14

EcruteakItemfinderHouse actual optimized comparison inspected: repeats the
VioletKylesHouse purple tatami/pale wall and furniture substitutions; source
orange tabletop object is absent. This adds concrete object-identity/readability
work to the traditional-room family, beyond palette changes. No production
source was changed during the final gate build/test waits.

## Gate check status and next house defect, 2026-10-14

Intermediate diamond test43197 is terminal0:158 modeled_interiors tests passed.
This predates the final source triangle orientation assertions. Final-source
Cargo test32065 is live in `receipts/gate-source-phase-tests.log`. Native51715
remains live; consume its terminal result, then run normal native Cargo on final
orientation source before all27 civic-room captures. Exact27 names are derived
from reviewed_gate_map and JOHTO_GATE_ROOMS; no extra maps guessed.

Actual VioletKylesHouse optimized comparison inspected: purple patches replace
part of the source green tatami, pale/cream walls and furniture replace source
gold, and table paint does not match its gold native drawing. This joins the
previous MahoganyMart traditional-room defects. Preserve stairs/collision/source
room layout while correcting that family in a later pass; no acceptance yet.

## Final source-phase correction pending, 2026-10-14

The bitmap inspection changed the next action: gate tile01 is an hourglass of
cream north/south triangles and gray east/west triangles, joining adjacent cells
into edge-centered diamonds. Corrected the first center-diamond approximation
to that exact triangle orientation and added orientation/index-count assertions.
This production edit happened while native51715 and test43197 were compiling.
Consume both terminal results, then run normal Cargo on latest source and capture
all27 civic rooms to a new folder before visual acceptance. No source changed
after this phase correction except the test orientation assertions.

## Gate source/phone diagnosis, 2026-10-14

Local pack graphic inspection in memory confirms gate tile01 has cream255 and
gray170 diagonal regions meeting around the cell middle. The corrected physical
diamond pattern is pending render alignment review; source phase must be checked,
not inferred from color coverage tests. Existing phone_console model has named
primitives/base_color but no material roles; generator tools/build-gate-counters.py
authors teal/enamel/screen/ink/key parts. Its existing cyan/cream finish differs
from the source green/black phone graphic. A future phone palette fix needs an
explicit authored role export and strict role checks, preserving its full handset,
keys/cord and footprint; do not blindly recolor or substitute another fixture.

## Gate diamond verification pending, 2026-10-14

All14 first Johto gate comparisons are actually inspected. Source green8px
counters/blue wall course improve guard visibility; oversized gray/cream checker
floor remains rejected, and phone/PC pigments and sparse plant/terracotta pot
fidelity remain open. Capture25634 is terminal0; preserve its rejection receipts.

Current corrected diamond source is frozen while native retry51715 and test43197
run. Both initial failed compilation handles are terminal101; Vec3 owned-array
conversion is fixed. The floor is one source-cell diamond with cream/gray half
areas and unchanged height, with winding/coverage tests. Existing counter/door/
wall/floor tests now positively cover all14 Johto gate names while preserving
unreviewed-map and custom ownership exclusions.

The shared gate floor helper affects27 exact civic-map rooms:14 Johto names
in JOHTO_GATE_ROOMS plus13 original Kanto names in reviewed_gate_map (including
two underground-path entrance rooms). After build/test completion, recapture all27
and actually inspect every pair to verify the floor change across original scope.
Do not accept from only14 new Johto maps. No room completion or goal completion
is established by this gate pass; all311/source variants/gameplay remain open.

## Johto gate implementation and rejected first floor, 2026-10-14

Previous turn made progress: all26 home/mart/clinic captures actually inspected,
156 interior tests passing, scoped static acceptance and remaining work recorded.
Current reviewed civic-map helper includes fourteen exact Johto gate-room names
already covered by authenticated counter NETWORKS. They now use native green
paint and8px counter height rather than generic timber/12px. Shared floor/wall/
door treatment is enabled for the same exact names. Counter masks/fingerprints,
phone plots, custom ownership and footing unchanged. All eleven gate_counters
tests passed; first native build passed5m19s.

First fourteen-gate capture25634 completed, `johto-gate-native-review/` and
`receipts/johto-gate-native-capture.log`. Actual first six pairs inspected so far:
IlexForestAzalea, Route29Route46, Route31Violet, Route32RuinsOfAlph,
Route34IlexForest, Route35Goldenrod. Green/lower counters and blue wall course
improve guard visibility; phones/plant palettes/details remain generic.
The newly exposed oversized gray/cream checker floor is explicitly rejected.
Finish inspecting remaining first images, preserving rejection receipts.

Broader interior run54833 passed153/156, failing three stale Johto exclusions
for floor/door/wall treatment. Tests now cover all14 positive rooms plus an
unreviewed-map fallback. Added all14 floor height/footing/source preservation
and source-cell diamond coverage/winding tests. New floor helper draws one
cream/gray diamond in each8px source cell (half area each), replacing invented
large squares. This helper is also used by original Kanto gates; final visual
verification must cover every affected gate-family room, not only Johto.

The first diamond compile52549 and test10925 failed E0631 from Vec3::to_array
as an owned-array mapping callback. Corrected to a closure before retry. Current
native retry51715 is live in `receipts/gate-source-diamond-build-retry.log`;
test43197 is live in `receipts/gate-source-diamond-tests.log`. No visual diamond
acceptance yet. No pack export/audio change/stage/commit. Full311 room/original
Kanto/variant/gameplay and remaining furniture requirements stay open.

## All eleven clinics: scoped static verification, 2026-10-14

Capture85715 is terminal0 for all three jobs: two player-home rooms, all13
ordinary marts, all11 Johto first-floor clinics. All26 actual comparisons have
now been inspected; this does not complete the full311-room goal.

Clinic comparisons in `all-johto-first-clinic-review/` show source pink floors,
low yellow lounge seats, gray PC, blue controls/counter, empty recessed healing
bay, and native gray wall course. Nurse visible in every captured idle view.
Accept those static fixture/material corrections only. All states preserved;
checksum/LCD equal for Cherrygrove41/41, Route3242/42, SilverCave41/41; other
clinics differ4 animation frames. Full NPC pose acceptance, active healing,
restoration, dialogue/walking, and room composition remain open. Receipt:
`receipts/all-johto-first-clinic-capture.log`. Earlier four-clinic-only status
is superseded by this actual all-eleven static review.

Source/code tests: all156 modeled_interiors tests passed in the latest area
retry, current native build passed7m52s. Home wall detail accepted narrowly
with exact paired frames; all13 mart material corrections accepted statically.
No unrelated files/caches/packs were removed, no exports or audio changes, no
stage/commit. Retain ignored receipts while this goal remains active.

Next priorities: Johto gate counter scale/source green and floor/wall fidelity;
remaining home stairs/detail/readability; ordinary Johto houses and Goldenrod
DeptStore1F-6F merchandise/kiosk/floors; active clinic proof; towers/ruins/Rocket
bases; every original16 Kanto variant and remaining original requirements.
Do not redefine completion around these26 rooms or static materials.

## Verified combined pass, 2026-10-14

Native build52369 passed in7m52s. Test39317 passed all156 modeled_interiors
module tests after including beveled faces in the home's projected-area check.
The failure in32482 was the older near-flat-face threshold, not a production
area defect. New clinic source guards, floor/footing and generic/custom ownership
checks pass. Receipt `receipts/johto-clinics-marts-home-area-retry.log`.

Capture85715 is live: first home pair then13 marts completed; eleven clinic
maps are still running. Both actual home comparisons inspected in
`player-home-plank-review/`: plank edges/short grain marks replace the plain gold
backing and join the lower source walls; stair opening retained. State preserved,
1F checksum/LCD41/41 with3 actors, 2F49/49 with1 actor. Accept this wall detail
correction only; home furniture detail, stairs, composition, readability and
walk-through remain open.

All13 actual mart comparisons inspected in `all-standard-mart-continuous-review/`:
continuous source green now replaces the rejected oversized checker squares;
source merchandise/counter identities and pigments remain. All receipts preserve
state, but checksum/LCD differ4 frames (Azalea5), so accept static material
correction only. NPC pose/visibility and shop interactions, room architectural
and merchandise detail remain open. Shared floor footprint/footing remains
covered by the passing source regressions. Retain old rejected checker captures.

Current pending clinic batch: `receipts/all-johto-first-clinic-capture.log`,
images `all-johto-first-clinic-review/`; consume the same85715 handle, inspect
all11 actual comparisons and paired-state receipts before scoped acceptance.
Then continue all original Kanto requirements and the full311-room scope.

## Latest combined home/clinic/mart verification, 2026-10-14

Floor test68702 passed22 Johto tests including the continuous-field regression.
Superseded native build16550 passed in11m50s; its source snapshot preceded the
latest darker field and clinic/home changes. Current native build52369 is live
in `receipts/johto-clinics-marts-home-build.log`, covering darker continuous mart
field, all11 Johto first-floor clinics, and player-home plank backing. Do not
restart from observation timeouts. Only subsequent edits were test assertions.

Current home backing correction: read the local pack's graphic donor pixels
in memory. players_room tile02 and players_house tile11 share identical board
art: west light edge170, body85, east joint0, grain170 at(3,2). The previously
flat single-swatch backing now has source-colored individual plank fronts,
shallow east V joints and small grain marks; existing closed body, cap, width,
height and stair opening remain. Source-course validation/claims/footing unchanged.
New regression checks pigment donors, front projected area, positive winding,
relief, footprint and untouched footing/ownership. Visual acceptance pending.

The first broader modeled_interiors run96631 passed154/155, failing an old
PlayersHouse2F untextured-floor expectation predating its native source finish.
That test now checks source home finish below unchanged3.5 actor datum plus the
legacy unreviewed-room fallback. New run32482 is live in
`receipts/johto-clinics-marts-home-tests.log`. The home projected-area test was
corrected to include beveled front faces (positiveZ), not only near-flat faces;
if this edit was after compiler parsing, rerun the test normally after terminal.
No production changes were made after native build52369 began.

After current tests/build, capture all13 ordinary marts, all11 Johto first-floor
clinics, PlayersHouse1F/2F to a fresh folder. Inspect every actual image; reject
and fix defects before scoped acceptance. Full clinic healing/nurse visibility,
full home detail/composition/walking and all311-room/original Kanto requirements
remain open. Preserve unrelated source/model/runtime work and unchanged packs.

## All Johto first-floor clinics implementation, 2026-10-14

Previous goal turn made progress: standard-mart fixture source correction,
actual all-six review/rejection of invented checker floor, source-based continuous
field fix, Research Center entry repair verification, and further room defects.
Current turn inspected Blackthorn, Cianwood, Mahogany, Route32, and SilverCave
clinic baselines (Cherrygrove/Violet and four earlier clinics already inspected).
All11 Johto first-floor clinics share the source pink floor/low yellow lounge
seats/native equipment pattern; the remaining seven still have generic gray floor,
blue backed chairs, and cream/pink equipment.

The reviewed floor/clinic selector now includes all11 exact names. Existing
complete source guards and native PC/healing enclosure/console/cabinet/counter/
wall/lounge paths are enabled, preserving claimed cells and footing. The clinic
fixture regression now covers all11; a new floor regression checks pink source
finish, footprint/height bounds and unchanged footing/ownership for every room.
Old generic/exclusion tests use UnreviewedPokecenter1F; actual first-floor rooms
are proved by the expanded positive tests, while Johto2F remains excluded.
These edits landed after live floor test68702 and native build16550 started.
Consume those results, then run latest-source modeled_interiors tests and normal
native Cargo build. Fresh visual cohort: all13 ordinary marts plus all11 Johto
first-floor clinics. No visual acceptance for the newly enabled seven yet.
Preserve active-healing/nurse/walk-through requirements for every clinic.

## Next verification checkpoint, 2026-10-14

All six first mart comparisons and Research Center retry32957 are terminal0
and actually inspected. Kanto alias retry92068 passed185 tests; its Johto
filter passed21. The newer continuous-field regression is not in that test
binary. Latest-source Johto Cargo test68702 is live in
`receipts/johto-mart-continuous-floor-tests.log`. Native build16550 is live
in `receipts/johto-mart-continuous-floor-build.log`; darker-field source changed
after launch, so consume its terminal result then run Cargo normally again
before capturing the final darker field. Do not restart from a poll timeout.
No acceptance of the rejected green checkerboard. Next final capture must cover
all13 ordinary marts, including the seven original Kanto rooms sharing this
helper. Inspect every actual pair and preserve collision/footing/custom ownership.
Then continue Johto clinics, homes, Goldenrod department store, gates, towers,
all311 rooms and every original Kanto requirement. No stage/commit was requested.

## Standard Johto mart continuation, 2026-10-14

Previous goal turn made progress: actual Flash-on cave evidence, corrected home
stool paint, retired table asset, passing tests, and verified station orientation.
Actual baselines for all six standard Johto marts inspected: Azalea, Blackthorn,
Cherrygrove, Ecruteak, Olivine, Violet. Each shares the native green floor/blue
stock-case drawing but the baseline uses gray floor and pastel book merchandise.
The reviewed retail/floor selectors now include all six. Existing strict source
cases replace booklike stock; native counter paint also follows the reviewed
selector. New regression covers each room, exact complete fixtures, per-cell
mutation rejection, eight claims, and unchanged footing; floor regression now
covers all six. Unknown map and Mahogany special layout remain excluded.
Both new standard-mart tests passed (6m07s build, 0.06s execution). Fresh
compiled test binary passed 21 Johto regressions; Kanto run passed184/185,
failing an old Cherrygrove generic-counter exclusion. That stale expectation
is corrected to assert native paint/claims/footing across all six standard marts
and preserve an unreviewed-map exclusion. Kanto retry4271 is terminal101 in `receipts/johto-mart-kanto-retry.log`: new
Spinarak/Ariados species registration requests missing .mesh.json compressed
inputs while canonical .glb and published .glb.gz/meta pairs exist. Model owner
notified of the include/alias mismatch; no species workaround/cache removal.
Retry after its owned correction, retaining the new all-six counter assertions. Native build3168 passed in 11m39s, `receipts/johto-standard-mart-build.log`.
Fresh six-mart batch then exact Research Center retry are live sequentially in
session32957: `receipts/johto-standard-mart-capture.log` and
`receipts/research-center-retry-capture.log`. The model owner now added explicit
spinarak/ariados species_mesh GLB arms; the broader Kanto alias retry is running
in `receipts/johto-mart-kanto-alias-retry.log`. No visual
acceptance yet: recapture all six after the build and inspect each actual pair.

Additional actual tower baselines inspected: SproutTower1F has a plain dark
flat-topped rectangular center pillar and pale rail framing; central pillar
silhouette/material and entrance guardians need correction. TinTower1F mixes
raised beige partitions with flat source rail/guardian strips and leaves pixel
floor cards; coherent authored architectural treatment remains open. Actor
models are owned by the other goals, not accepted from this static capture.

Pack owner reports the Research Center missing-scene-label fix in runtime
numeric constants plus exporter macro handling. Rebuild client after current
mart build completes, then recapture exact (4,3), noon, existing unchanged pack.
Its own two runtime tests are compiling in target/agent-pack-scene-fix; do not
claim their success yet. Preserve historical failure receipt.

IndigoPlateauPokecenter1F retry at (9,7), noon, now succeeds; both actual
images inspected in `indigo-retry-review/`. Receipt `receipts/indigo-retry-capture.log`.
State preserved, checksum41/LCD45, six actors: static-only evidence. Current
3D framing crops much of the south lobby; retain composition/actor-phase work.
This supersedes its historical crash as a current capture blocker. Thus all
original six capture failures now have successful current retries except the
Research Center, whose runtime fix is pending this goal's rebuild/recapture.

GoldenrodNameRater actual optimized pair inspected: source pink floor and blue
wall become pale timber/cream; source gold furniture becomes pale/pink table,
pastel books and generic electronic cabinets. Plants use washed-out sparse
leaves and terra-cotta instead of source green/gold. Route35GoldenrodGate pair
inspected: source green counters become tall pale timber, source diamond floor
becomes gray, blue lintel disappears. These houses/gates remain open defects;
do not accept from shared model registration or source-cell claims alone.

GoldenrodDeptStore1F and 3F actual optimized comparisons inspected: 1F retains
source orange/yellow floor as magnified pixel motif, sparse planters, generic
PC/counter bodies, and front-cropped cushioning. 3F blue merchandise cabinets
become pale pastel bookcases. Carry the reviewed retail/source-counter family
through Goldenrod after the six standard marts; physical floor treatment must
preserve its own orange/yellow pigments rather than generic gray.

GoldenrodDeptStore4F/5F/6F actual optimized comparisons also inspected. 4F
repeats pastel booklike substitutions for blue/gray merchandise. 5F kiosk
foreground/roof geometry hides most of the player and cashier; counter/kiosk
height, cutaway, and source ownership need correction. 6F source gray vending
cabinets become blue-green bodies. All retain the orange/yellow pixel-floor
motif, with generic planter paint on 6F. Full departmental-store acceptance
remains open across 1F-6F (2F was inspected previously).

Fresh standard-mart pairs inspected so far: Azalea, Blackthorn, Cherrygrove,
Ecruteak in `johto-standard-mart-review/`. Source stock-case identities and
counter paint improve, but large green checker squares are rejected. Floor
helper now uses continuous darker source green with lighter narrow seams,
retaining physical ceramic geometry. The actual 2D histogram has368774 pixels
RGB123/165/8 versus191051 RGB148/198/74, confirming darker-field dominance.
A new floor regression checks all four macro phases keep the same field UV
and over60 of64 square units while retaining narrow seams. Native build16550
is live in `receipts/johto-mart-continuous-floor-build.log`; the darker-field
choice was edited after launch, so run normal Cargo again after it finishes
before claiming the latest source is compiled. Then recapture all six to a new
folder and actually inspect them. Keep the rejected first checkerboard receipts.

First six-mart capture32957 completed successfully, all six actual pairs now
inspected in `johto-standard-mart-review/`. Cabinet merchandise identities and
counter paint improve; checkerboard floor is rejected as above. Kanto alias
retry92068 passed185 tests after owner added spider GLB aliases. Fresh compiled
Johto filter passed21 tests in `receipts/johto-mart-final-johto-tests.log`; it
does not contain the newly added continuous-field regression, so run Cargo
on latest source before claiming that check passes.

Research Center exact retry completed successfully after the native rebuild,
unchanged browser pack. Actual pair inspected in `research-center-retry-review/`,
receipt `receipts/research-center-retry-capture.log`: (4,3), noon, state preserved,
checksum/LCD47/47, three actors. Missing scene token is resolved; pack owner
notified with exact image paths. Room remains visually rejected: gray floor
replaces blue-striped source, gold bookcases become pale/pastel, native machinery
and workstation identities/materials need correction. All311 base maps now have
successful comparison captures across baseline/optimized/retry folders; this is
coverage, not complete visual review or acceptance.

The mart floor helper is shared by all13 ordinary marts: six Johto plus
Cerulean/Fuchsia/Lavender/Pewter/Saffron/Vermilion/Viridian. After a fresh latest
source build, recapture and inspect all13 rather than accepting the shared
floor from only the six new Johto rooms. Preserve all other original scope.

CherrygrovePokecenter1F and VioletPokecenter1F actual optimized comparisons
inspected: gray floor instead of source pink, backed blue seats instead of low
yellow source seats, and generic pink medical equipment/counter. Apply the
verified Johto clinic family after current mart verification. Keep active
healing, nurse visibility, and walk-through acceptance open.

## Latest continuation: Flash and full capture, 2026-10-14

The full capture batch is terminal: 33 retained baseline successes plus 272
optimized successes produce 305 map comparisons. Six captures failed; this is
capture coverage, not full visual review. The optimized pinned executable was
removed by its runner. Three failures (WhirlIslandB1F, RockTunnelB1F,
SilverCaveRoom1) were dark no-Flash frames rejected by the image validator.
The user correctly identified the missing Flash setup. WhirlIslandB1F was
recaptured at (20,17), noon, with `--preview-flag STATUSFLAGS_FLASH` in a
fresh disposable visual fixture. Both actual views were inspected and the
image validator passed. Receipt: `receipts/whirl-flash-capture.log`; images:
`whirl-flash-review/`, under `target/kanto-resume/`. This confirms the lighting
setup, not complete cave geometry acceptance or a played HM interaction.
Remaining failures: IndigoPlateauPokecenter1F and TeamRocketBaseB2F panic on
vertex attribute access after RenderWorld extraction; RuinsOfAlphResearchCenter
fails its entry script with unknown numeric scene token
`SCENE_RUINSOFALPHRESEARCHCENTER_GET_UNOWN_DEX`. Preserve these concrete failures.

Both arcade corrected-pigment pairs are inspected in `arcade-pigment-review/`.
Cabinet bodies now use live source yellow/orange/red and dark controls; the
opt-in palette trace confirms the donors. The 185 Kanto tests pass in
`receipts/arcade-pigment-tests.log`. Accept cabinet pigment only; stools,
plants, wall details, and complete-room fidelity remain open.

The superseded shipped dining_table mesh has been retired after the baseline
runner ended. The checker and regression now reference domestic_table instead.
Its targeted regression passed both tests in `receipts/domestic-table-retirement-test.log` (7m02s build; 0.02s execution).
Fresh CherrygroveMart comparison was actually inspected in the optimized
review directory. Its source green floor becomes pale gray, blue stock cabinets
become pale cabinets with pastel booklike contents, and display bodies retain
flat source cards. Extend the verified Azalea mart family carefully; counter
height, stock identities, material fidelity, and full-room review remain open.

Additional actual optimized comparisons inspected: MahoganyMart1F uses pale
bookcases for gray merchandise shelves, pale timber/cream walls in place of
source gold, and invented purple squares across green tatami. Its floor and
furniture identities need correction. GoldenrodUnderground loses the green
entrance partition and blue lintel in favor of a large pale wall; gray floor
patches replace much of the source diamond field, and the central stair remains
a flat drawing. These are open defects, not accepted rooms.

TeamRocketBaseB2F fresh retry succeeded with the current built renderer;
`receipts/rocket-b2f-backtrace.log` records both image checks passing. Actual
`rocket-b2f-diagnostic/TeamRocketBaseB2F-compare.png` inspected: bulky pale
partitions replace source gold, blue floor is lost, and red machinery links
are absent. This supersedes its crash as a current capture blocker, while
preserving the historical crash receipt. Room fidelity remains rejected.

VioletGym actual optimized comparison inspected: north stone plaques become
beige crates; entrance statues/pedestals are absent. Statue owner notified.
Tall pale ledge side faces, blue floor palette, and entrance framing remain
interior defects requiring source comparison and recapture.

RockTunnelB1F and SilverCaveRoom1 Flash-on recaptures both passed and both
actual comparisons were inspected in `cave-flash-review/`. Positions remain
(15,18) and (10,18), noon, with STATUSFLAGS_FLASH disposable preview state.
Source-state/gameplay/complete geometry acceptance remains open.

User reiterated existing blue station cabinets and slots facing the chairs.
Verified selected existing vending_machine cabinet shell: source front is +Z;
metal ticket slot occupies z0.472..0.488. Existing station transform maps +Z
to west (-X), toward the chair bank. Actual station-ticket-review comparison
inspected; one ticket-bank geometry/footing regression passed in
`receipts/station-ticket-orientation-test.log`. Preserve this orientation.
Fresh home/station batch completed in `home-stool-station-review/`; both actual
comparisons inspected. Goldenrod existing cabinet fronts/slots face west toward
the chair bank. State preserved, checksum/LCD frame41/41, two actors.
Accept this orientation; full station and timed travel remain open.

Player-home stool paint is corrected by enabling the existing strict complete
players_house drawing path; source claims and footing remain unchanged.
All five player-home regressions passed in `receipts/player-home-stool-tests.log`
(7m22s build, 0.05s execution); native build passed in 7m57s in `receipts/player-home-stool-build.log`.
Fresh PlayersHouse1F comparison inspected in `home-stool-station-review/`: source
gold stool paint replaces pale upholstery. State preserved at checksum/LCD
frame42/42 with three actors. Accept stool paint only; full home walls, furniture
detail, composition, and walk-through remain open.

## All-interior expansion and player home, 2026-10-14

Authoritative pack inventory: Johto108 indoor/39 dungeon/32 cave/12 gate maps;
Kanto100 indoor/7 cave/13 gate maps. Include dungeon-classified towers, ruins,
underground rooms and Rocket bases. IlexForest is classified CAVE in the pack;
its outdoor geometry stays coordinated with the outdoor owner.

Historical capture setup (now terminal; see latest continuation above). Superseded unoptimized session47778 is
terminal (143); its33 validated successes and failure receipts are retained in
`target/kanto-resume/all-interiors-baseline-review/` and the original progress
log. After optimized captures proved a practical speed improvement, its owned
runner/capture processes were stopped and its pinned executable removed.
New session69376 consumes those successes and captures the remaining278 maps
(including the failed AzaleaGym) with a pinned optimized native renderer at noon.
Output `target/kanto-resume/all-interiors-optimized-review/`; progress/failures
`target/kanto-resume/receipts/all-interiors-optimized-batch.log`. Each subprocess
has a600s external deadline that terminates only its own capture group. The new
pinned executable is removed when the batch terminates. This was a deliberate
validated pipeline optimization, not a restart from an observation timeout.
Captures are review input, not acceptance. Inspect every actual paired image,
record defects, fix them, and recapture affected maps before full completion.

Fresh PlayersHouse1F/2F baseline batch completed; both actual comparisons and
paired-state receipts were inspected in `player-home-current-review/`.
Ground floor preserves state/frame141/three actors; bedroom frame88/one actor.
Observed source mismatches: gold wood becomes pale wood, gold tables become
pink, gray electronics become pale/blue clinical models, and the bedroom
bookcase invents pastel books. Kitchen fixtures and domestic wall materials
also retain generic finishes. Player-home source-fidelity correction is the
foreground priority. Full311-room review, source variants and route/interaction
checks remain open.


Player-home source correction is implemented but **not yet visually accepted**.
The reviewed residential selector now includes PlayersHouse1F/2F; live atlas
wood replaces generic pale floors, existing native 1F table/TV/PC/stool paths
are enabled, and the bedroom table validates all twelve source cells before
using native gold paint. Bedroom wallpaper uses its own native wall drawing;
the canonical bedroom TV and bookcase use source paint, including gray book
spines instead of imported pastel pigments. Furniture geometry, claims and
footing are retained. New `johto_player_home` regressions cover both tables,
both floors and the bedroom bookcase. Build/tests are still running after
shared dependency changes; do not report their result or visual acceptance yet.
Receipts: `player-home-material-tests.log`, `player-home-johto-tests.log`,
`player-home-native-build.log` in the ignored receipts directory.
Kitchen appliances, bedroom radio and the default bedroom bed now have a second source-paint
pass implemented, still unverified. Canonical named material roles bind four
source swatches on stove/sink/fridge/radio/default feathery bed, retaining geometry
and rejecting unknown roles. Exact complete drawings guard the append path;
decorated bed variants remain unchanged. Source bitmap sampling verifies bed
frame medium gray at(5,2), dark hardware at(0,0), white linen at(1,1).
`johto_player_home_appliances_and_bed_bind_only_complete_drawings` exercises
production resolution/append, wrong-art rejection and unchanged footing. The
shared geometry/material regression also covers all five canonical assets.
Initial native build93932 passed (exit0). The final incremental build and
first home capture queue80971 remains live; Johto regression21466 is still
compiling its updated shared dependencies. No current test pass is claimed.
These edits occurred after session80971's follow-up build started; its first
home capture is the earlier material pass and **cannot accept this kitchen/bed
pass**. Build again and recapture both home floors after consuming that queue.
Whole-room composition and visual acceptance of all home changes remain open.
The first all-interior capture (AzaleaGym) has not produced an image after more
than twelve minutes. A one-second native process sample locates work in
`new_bark_actors::sync` → `open_people_skin::PeopleGpu::spawn` / hashing during
character mesh preparation (`receipts/azalea-capture-stack.txt`), before the
capture watchdog can advance. The process remains live; do not describe this
as a completed baseline. My test concurrency was reduced to two jobs; initial
unbounded/queued test processes were interrupted, so their old receipts are
not passing evidence. Current Johto test session21466 and native build93932
are still running. Session80971 waits for the first native build, then runs
another incremental native build to include all final edits and captures
PlayersHouse2F/1F into `player-home-final-review/`. Its receipts are
`player-home-final-build.log` and `player-home-final-capture.log`. Do not restart
that live verification queue; consume its result and inspect both actual
comparisons before accepting the home pass. The rig owner was sent the
footwear/refinement sample evidence through the authorized coordination path. No other owner's builds or caches were interrupted.



## Fresh home evidence and Johto baseline findings, 2026-10-14

Previous turn made source changes (progress); current work also inspected new
actual comparisons. Queue80971 completed exit0. Both home paired states are
preserved: bedroom frame115/one actor; living room frame172/three actors.
Actual images in `player-home-final-review/` confirm gold floor/table paint and
gray PC improvement, but expose incomplete north walls, a too-pale bedroom
bookcase and unsupported white strips on the living-room table. These are
**not accepted rooms**. Current source repairs complete north wall backings
with exact full-course art guards, leaves both stair openings clear, binds the
bookcase frame to medium gray at source pixel(1,7) and paints the table from its uninterrupted gold
center. Stair/floor claims and footing are unchanged. The kitchen/bed/radio
pass still requires fresh visual proof.

The same actual review found sparse foliage and wrong terracotta pots in
SilphCo1F. Its exact facility13/14 plants now share native source paint with
TrainerHouseB1F and add two smaller rotated crowns from the selected closed
leaf geometry. Pot/soil are retained once, geometry stays inside its original
bounds, and the source drawing guard excludes unrelated facilities. New mesh
checks cover scaled bounds, original geometry/cache preservation and no extra
pot/soil. This reverses the earlier assumption that source paint alone finished
the basement foliage: both facility poses need new acceptance.

Baseline47778 remains live. AzaleaGym failed a real capture watchdog after
1713.61s; AzaleaMart, AzaleaPokecenter1F and three initial Battle Tower maps
completed. Actual AzaleaMart comparison was inspected: green floor became gray,
stock cabinets show generic pastel book-like contents. Current source enables
its native floor/counter and closed native cabinet paths; no unrelated Johto
mart is silently accepted. Actual Azalea clinic comparison shows generic gray
floor, blue chairs, pale three-ball healing equipment and pale PC: apply and
verify the already established source-native clinic family next. Actual
BattleTower1F has missing pink floor/gold fixtures and generic clinical furniture;
BattleTowerBattleRoom replaces blue/red tile art and metal fixtures with pale
faceted geometry and a large invented battle disc. Both tower rooms remain open. Three more actual baseline pairs were inspected:
BattleTowerElevator loses blue tile/pink-red striped walls to a small pale faceted
box (also review framing); BattleTowerHallway replaces repeated gold doorway
fixtures with generic pale PCs and loses its pink floor. BillsFamilysHouse has
wrong pale timber flooring instead of pink ceramic, pink table instead of gold,
pastel books, generic clinical electronics, cream walls instead of native blue,
pale stools and sparse leaves/terracotta pots instead of dense gold-pot plants.
None of these five Battle Tower/house findings is accepted or resolved.

Optimized renderer build2356 completed exit0 in9m08s. The source changes made
during build17015 superseded it; its Cargo child was interrupted before capture
(exit-2), and it is **not passing evidence**. Frozen correction queue68410 now
builds all latest source with voxel opt-level1, then captures PlayersHouse2F,
PlayersHouse1F,SilphCo1F,TrainerHouseB1F,AzaleaMart to
`home-facility-mart-final-review/`; receipts
`home-facility-mart-final-{build,capture}.log`. Poll this same live queue and
inspect all five actual pairs. Do not restart from an observation timeout.
Johto regression21466 still compiles updated shared Bevy dependencies; no tests
have yet passed on these changes. Source edits are frozen until those receipts
are consumed, except the bookcase donor was corrected from the black border
pixel(0,7) to medium gray(1,7) after build68410 started. Rebuild incrementally
before final acceptance if its compiled bedroom image still uses the border
pixel; do not infer it included a late source edit. Full311-map scope, original16 variants, unresolved Kanto work
and user station deferral remain intact.


## Latest production correction receipts, 2026-10-14

The previous turn made source changes and actual image review (progress).
Queue68410 completed: optimized native build exit0/9m04s and all five captures
exit0. Every actual pair was inspected in `home-facility-mart-final-review/`.
State preserved: PlayersHouse2F frame41/one actor; PlayersHouse1F frame41/three;
SilphCo1F frame54/three; TrainerHouseB1F frame76/three; AzaleaMart frame43/four
(LCD animation frame47, so do not use that pair to accept exact actor animation
phase). Source paint on kitchen/bed/radio, denser source-green crowns/gold pots
in both facilities, and green floor/native stock cabinets in Azalea are visible
improvements. Scoped plant/shop material work is visually accepted at these
poses, pending relevant tests. Whole-room geometry/composition/routes remain
open. Home wall backing did **not** appear, and the bookcase used the black
border donor from before the late pixel correction. Do not accept either home.

Concrete wall cause: production publishes a large padded VisualWorldFrame
(`render_mod.rs` grid origin/size), while the first helper required map-sized
geometry and origin zero. Current source matches the complete art course at
its true map coordinate inside the padded grid, preserving the staircase gap
and rejecting custom exclusions. A regression now covers padded36x32 geometry,
nonzero grid origin, complete art, the poster/stair variant, 64 mutations and
custom ownership. Queue76109 builds this correction and medium-gray bookcase
paint, then captures both home floors into `player-home-wall-final-review/`;
receipts `player-home-wall-final-{build,capture}.log`. It remains live.

Current cold-dependency test21466 remains live; queued suite29463 runs Kanto
then `johto_` tests and writes `interior-correction-kanto-tests.log` and
`interior-correction-johto_-tests.log`. These prefixes include the dense-crown
and new padded backing tests, which the earlier narrow home prefix omitted.
No current test pass is claimed. Do not restart live queues from observation
timeouts. The actual people/footwear task was sent the refinement-cost evidence;
the earlier Pokemon-rig task was not the people-file owner. Statue ownership
was coordinated with the active statue task; leave its geometry to that owner.

Additional actual baselines inspected: BillsHouse state preserved/frame136/two
actors still has pastel books, generic radio/stove, sparse foliage/terracotta
pots and the selected `tableCrossCloth` crossed-leg table shape. Inspect/switch
the shared native dining-table shape, not only its paint, to complete shared
furniture fidelity. BlackthornEmysHouse repeats the generic Johto house defects
recorded for BillsFamilysHouse. BlackthornGym1F has beige stair finishes and
blue boulder-like north guardians (sent to statue owner); lava/interface depth
and moving actors still need review. BlackthornGym2F has pale cream wall boxes
instead of source-gray barriers and obscured actor bodies; palette, wall scale
and cutaway visibility need correction. None is whole-room accepted. Baseline
47778 remains live and currently has attempted14/311 maps, including its real
AzaleaGym watchdog failure. Full311/16-variant scope remains active.


## Verified current home pass and repaired tests, 2026-10-14

Queue76109 completed exit0: optimized build4m49s, both fresh actual home images
inspected in `player-home-wall-final-review/`. State and animation phase are
preserved: bedroom frame41/one actor; ground floor frame46/three actors. Both
complete north backings now appear, staircase openings remain clear, and the
bookcase frame uses its correct medium-gray source swatch. Home floor, table,
PC/TV, kitchen, default bed/radio paint and backing topology are scoped visually
accepted. This is **not whole-home acceptance**: wall panel/grain continuity,
shared `tableCrossCloth` shape, pale living-room stools, broader composition and
walking/interaction checks remain. The earlier failed map-size guard is repaired
with exact offset-aware art matching inside the production padded world grid,
aligned to the authoritative map origin and yielding to custom exclusions.

Cold test21466 failed initially (two bad fixture ground samples); repair36538
left one kitchen fixture failure. Diagnostic40073 identified KitchenStove.
Its resolver needs both gold-wall11 and floor01 samples; the fixture now supplies
both. Radio/bookcase/TV fixtures use their actual ground02; mart fixture uses
native standard floor48 rather than department-store floor01. Two older Kanto
negative assertions now test unreviewed ElmsHouse instead of the explicitly
newly authorized player home. Required source claims, footing, wrong-art and
unreviewed-room guards remain. Final7702 PASSED18 Johto checks/0.53s;
final4667 PASSED184 Kanto checks/1.21s. Receipts:
`interior-correction-johto-final-tests.log`,
`interior-correction-kanto-final-tests.log`. No production behavior was weakened
to satisfy a test. Broader tests are not justified until another source change.

New baseline69376 successfully recaptured AzaleaGym in39.58s versus the old
1713.61s watchdog failure, then advances through all remaining maps. Its receipt
has checksum frame115/LCD animation134, so use it for static defect review,
not exact actor animation acceptance. The retained33 old successes and all278
new attempts preserve full311 coverage. Every actual image still needs review;
counts, captures and green unit suites never establish whole-goal completion.
Statue and people ownership coordination is recorded above. Full original Kanto
requirements/16 variants and expanded Johto scope remain active.


## Original four-leg table and bedroom footprint, 2026-10-14

The previous turn produced verified home/source fixes and tests (progress).
Current source replaces selected crossed-leg cloth geometry with the original
four-leg timber table at `assets/open-models/interiors/domestic_table.mesh.json`.
Editable104138-byte native source is `art/johto/source/domestic-table.blend`,
registered with `johto_art_sources.py`; regeneration is
`blender -b --python tools/build-domestic-table.py -- target/domestic-table`.
It reuses the existing original furniture vocabulary, exports only this table,
and contains no game pixels/content. Source parts remain individually editable.
The first compressed save could not be registered without zstandard; generator
now saves uncompressed Blender source before native gzip storage. All hashes
were registered successfully. Runtime selection and exact named material roles
support the new family. Browser bundle export discovers the new JSON; the game
port owner was told to include it in the next rebuild.

The old `dining_table.mesh.json` is temporarily retained **only because live
pinned baseline69376 still requests that file**. After that job finishes, retire
it, remove its checker reference and update/remove the legacy-file test include;
do not leave it in the final shipped artwork/bundle. Keep legacy parser support
only if justified by user-owned artwork compatibility. All original/source
artifacts are deliverable authoring inputs; ignored binaries/renders are not.

Bedroom table top uses two source rows; its former24px depth reached8px outside
that footprint toward the player. The canonical profile now uses16px depth at
unchanged10px height, retains all12 drawing claims/footing, and checks that the
physical table stays within the two-row approach boundary. The new mesh test
checks four corner supports, clear knee space, outward faces, source bounds at
three scales, unchanged cache and unknown-role rejection. First test/build
60441/20279 failed on my malformed quoted test string; it was fixed.
Repair26054 PASSED185 Kanto checks/1.58s. Build89664 PASSED3m47s; its six actual
pairs were inspected, including CeladonMansion2F as a surrounding-room regression.

Bill's table initially showed excessive alternating top colors. Authoring now
uses one top material while retaining plank seams and a separate frame.
Uniform capture56738 completed exit0; all five actual comparisons inspected in
`domestic-table-uniform-review/`: home2F frame41/one actor, home1F frame43/three,
BillsHouse frame41/two, BillsFamilysHouse frame40/three, all state/phase preserved;
Oak checksum35/LCD39/five actors (static fixture proof only, not actor animation
acceptance). Uniform185-check suite96956 PASSED0.72s; Johto22302 PASSED18/1.18s.
Receipts `domestic-table-uniform-{tests,capture}.log`,
`domestic-table-johto-tests.log`. Scoped table shape/depth/paint is accepted;
home wall finish, pale stools and broader room composition remain open.

Additional actual baselines inspected: Ecruteak clinic repeats Azalea's generic
blue seating/gray floor/three-ball machinery/pale PC; ElmsHouse has pale wood,
pink table, pastel books, cream wall and generic electronics; ElmsLab incorrectly
uses gold/brown flooring instead of source gray bands, pale lab bookcases instead
of native gold, pastel books, generic restoration machinery/PC and pale starter
balls instead of orange/gold source models. Next correction family is Johto
clinics and residential/lab source fidelity. Full311 scope/16 original variants
is active. Baseline69376 is live (latest133/278 new attempts plus33 retained
old successes); successful captures still require actual review and correction.


## Johto clinic family continuation, 2026-10-14

Previous turn made verified table/source progress. Current source enables exact
native clinic paths for AzaleaPokecenter1F/EcruteakPokecenter1F, then extends to
GoldenrodPokecenter1F/OlivinePokecenter1F after both additional actual baselines
were inspected. The shared art guards, wall headers, blue control bank/counter,
gray PC, empty recessed idle enclosure, source pink floor and low gold/yellow
seats are reused. Production claims and footing are unchanged. New Johto fixture
regression resolves/append-checks PC, machine and seats in all four rooms, rejects
every changed source cell and keeps unreviewed Cherrygrove excluded. Earlier
negative Goldenrod checks now target that unreviewed neighbor because Goldenrod
is explicitly authorized and reviewed. No unrelated map is silently accepted.

Test30594 initially failed because species_rig referenced Ledyba/Ledian before
their generated include inputs were available. Their raw GLBs were then present;
normal retry69519 PASSED19 Johto checks/1.64s. Model owner was told the concrete
missing OUT_DIR inputs; no species code/cache was modified. Native queue78972
PASSED build4m10s and both captures exit0. Actual Azalea/Ecruteak comparisons
were inspected in `johto-clinics-native-review/`: native fixtures/materials now
appear, removing generic blue chairs/pale PC/three-ball machinery. State is
preserved, but checksum/LCD phases are37/41 and42/46; accept static material/
fixture corrections only, not exact actor animation or whole-room visibility.

Current four-room queue79414 builds then captures all four clinics to
`johto-clinics-family-review/`, receipts `johto-clinics-family-{build,capture}.log`.
Current Kanto regression80246 writes `johto-clinics-family-kanto-tests.log`.
Four-room native queue79414 remains live. Initial Kanto80246 failed one
pre-expansion negative assertion in the service-apron test (Goldenrod was
expected to stay generic). That assertion now targets unreviewed Cherrygrove;
rendering behavior was not altered for the repair. Repair Kanto24521 and queued
Johto70135 are live, receipts `johto-clinics-family-kanto-repair-tests.log` and
`johto-clinics-family-johto-tests.log`. Poll the same three queues and inspect
all four actual images before acceptance; do not restart from observation timeouts. Whole healing interaction/nurse visibility
still requires its own production-state proof; old Kanto healing acceptance is
not a substitute for that Johto scope.

Further actual pairs inspected: GoldenrodGameCorner has generic pink slot banks
and clinical PC-like rear fixtures; its complete game_corner drawings can use
source-native arcade row treatment, currently gated to Celadon. DanceTheater has
wrong pale stage surface over source gold striped timber and generic cream/purple
patches among source green tatami/blue patterned cushions; stage support and
performer poses need separate review. OlivineLighthouse1F substitutes huge pale
wall boxes and plain gray floor for dark stone/worn stair art, crops the entry and
hides portions of the layout. GoldenrodDeptStore2F has pastel book-like stock in
its displays; native rack/counter source treatment remains needed. These rooms
are not accepted. The outdoor owner reported the Slowpoke Well exterior scoped
verified, with B1F/B2F still belonging to this interior scope; preserve that
ownership split. Baseline69376 remains live; all311 scope/original16 variants,
remaining Kanto findings and station deferral are preserved.


## Goldenrod arcade source guards and clinic queues, 2026-10-14

Previous turn changed clinic source and inspected actual images (progress).
Four-clinic Kanto repair24521 PASSED185 checks/2.70s. Johto70135 remains live.
Family native79414 PASSED build9m54s and its four-map capture is now running;
inspect every actual pair in `johto-clinics-family-review/` before accepting
Goldenrod/Olivine. Source edits after its build concern only the distinct arcade
map guard, so they do not alter the compiled clinic behavior being reviewed.

The actual GoldenrodGameCorner baseline had generic pink row cabinets. Its
complete game_corner rows now use the same exact source-palette path as Celadon.
`reviewed_arcade_room` is limited to those two maps. Full-art/profile checks and
partial-row fallback rejection apply identically in both; unknown arcade rooms
keep their existing behavior and custom profiles retain first ownership. The
existing arcade regression now exercises every source mutation/claim/footing in
both maps, with the negative control moved to an unreviewed arcade room.
Stool geometry and other casino fixtures are unchanged by this machine pass.

Arcade test84556 and native queue76506 are live. Native receipt
`goldenrod-arcade-build.log`, then `goldenrod-arcade-capture.log`; actual pair
output `goldenrod-arcade-review/` for Goldenrod and Celadon. No new machine visual
acceptance or test pass is claimed until those queues complete and images are
inspected. Full311 baseline69376 is live (latest198/278 new attempts plus33
retained successes); keep the old table file until it finishes, then retire it.
Every unresolved original Kanto requirement/16 variant and the expanded Johto
room findings remain active; successful captures are not completion proof.


## Four Johto clinics: scoped static acceptance, 2026-10-14

Family queue79414 completed exit0; all four actual comparisons in
`johto-clinics-family-review/` were inspected. Azalea checksum38/LCD42/five actors;
Ecruteak48/55/five; Goldenrod42/48/six; Olivine38/44/five. Every state receipt
preserves authoritative game state. Native pink floors, blue counters/control
banks, gray PCs, recessed empty idle healing bays, yellow low seating and gray
wall courses now appear consistently. Static fixture/material work is accepted
at these poses. LCD phase differences prevent exact actor-animation acceptance;
active healing, counter-side visibility, walking and whole-room review remain.
Kanto24521 PASSED185/2.70s; expanded Johto70135 PASSED19/1.23s. No wider test pass
or full-goal completion is inferred. Older Kanto scoped evidence remains intact.

Arcade source76506/84556 verification continues live as described above. Full
baseline69376 also remains live. Keep full311-map/original16-variant scope and
all other unresolved findings; retire the obsolete table only after that live
pinned baseline has finished using it.


## Arcade visual rejection and live palette diagnosis, 2026-10-14

Previous turn completed scoped four-clinic verification (progress). Arcade
queue76506 completed exit0: optimized build3m38s, Goldenrod/Celadon pairs both
actually inspected in `goldenrod-arcade-review/`. State preserved: Goldenrod
checksum41/LCD45/twelve actors; Celadon41/41/nine actors. Complete paired machine
rows now appear, but large pink cabinet surfaces remain inconsistent with the
source yellow/red/black drawings. **Do not accept arcade material fidelity** in
either region from the185 green source-identity/claim tests. Those tests do not
prove that selected pixel coordinates contain the intended pigment.

Current diagnosis adds `trace_arcade_palette` beside source-palette finishing
in build_scene, coordinated with the outdoor owner. It runs only when named
environment `CRYSTAL_ARCADE_PALETTE_TRACE` is set and only in the two reviewed
arcades. It reads the live eight-pixel tile image samples for nine material
roles and reports bounded unique source colors/coordinates for the first
complete pair. It mutates no game/render state and emits no unconditional
fullscreen output. No outdoor/audit geometry was edited. Diagnostic native
build62057 is live, receipt `arcade-palette-diagnostic-build.log`. After it
passes, capture Goldenrod with that trace enabled into a fresh diagnostic
folder/log, inspect actual role RGBA values, fix wrong donor selection, then
recapture both maps and run relevant tests. Do not restart the live build from
an observation timeout. No corrected palette result is claimed yet.

Full baseline69376 remains live (latest246/278 new attempts plus33 retained
successes). Keep its old table asset until the job is terminal, then perform the
recorded retirement/checker/test cleanup. All311 interiors, original16 variants,
other unresolved Kanto/Johto findings and station deferral remain active.


## Safari office furniture and floor, 2026-10-14

Current source corrects SafariZoneMainOffice's pink octagonal table, all six
small square seats, two source-painted potted plants, and the complete green
L-shaped counter. The user corrected its mistaken tall-divider interpretation;
both arms are now8px high, matching the established waist-height counters.
Its regression limits the top including the source green finish to8.002px.
Exact game-corner artwork guards each fixture; reserved
custom cells, altered art and neighboring maps retain their existing renderer.
Table/seat/divider geometry is closed. The plant retains its existing fitted
mesh and binds source green leaves, gold pot and soil through six strict roles;
unknown named finishes retain source fallback. Collision and footing are
unchanged. The rig owner was informed of the chairs'5px seat/8px back height.

Its native floor preserves the source's paired gray panel quadrants. Exact
floor-phase masks include mixed seat/partition blocks, and a narrow production
shape-dispatch case corrects the earlier false slab elevation. The finish
preserves the floor datum and claims. Tests measure quadrant areas, verify
source/map/custom ownership guards, check fixture winding/claims/footing, and
compare the repainted plant's positions, normals and indices with the original.

Verified paired progress: fresh baseline was inspected in
`safari-office-current-review/`. The first furniture pair completed25.18s;
its plain ceramic floor lost the panel motif and was corrected. Final panel
pair completed13.25s, actual image/receipt inspected, `state_preserved=true`,
frame49, one actor (`safari-office-panel-final-review/SafariZoneMainOffice-compare.png`).
This accepts table/seat/floor corrections at that pose; the later divider and
plant changes still require their final paired view.

Verification caught three integration issues, all repaired in current source:
missing imports in the new shared seating test; a shared cushion-height type
mismatch; and a generic palette post-pass trying to repaint the empty solid
surface after native textured Safari corner/plant geometry was appended.
The exact native Safari paths now skip that second paint. The failed run
passed167/failed1 (`safari-office-complete-kanto-tests.log`); subsequent paint
checks passed168 in2.50s (`safari-office-paint-rechecked-kanto-tests.log`). No
failure is counted as a pass, and no test was weakened. Shared owners were
notified; seating behavior and outdoor files were preserved.

Current-source tests passed169 in2.82s after2m46s compilation, including the
Underground follow-up below (`safari-underground-current-kanto-tests.log`).
Whitespace checks passed. Native dependency build75620 finished31m28s; corrected
build10029 then passed22m08s (`safari-office-paint-rechecked-native-build.log`,
earlier receipt `safari-office-complete-native-build.log`). Fresh noon paired
capture is now live for SafariZoneMainOffice, UndergroundPath and TrainerHouseB1F
in `safari-underground-basement-current-review/`, receipt
`safari-underground-basement-current-capture.log`. Inspect actual images and
paired-state receipts and fix any defects before visual acceptance.
The first full pair preserved state at frame126 (one actor), but exposed the
remaining flat north wall and gold trim covering the counter's green top.
Added closed source-painted north-wall courses and restored the green top;
the user subsequently rejected the20px counter height, which was corrected to8px
and classified as ReceptionCounter. Current checks/build receipts are
`safari-waist-counter-kanto-tests.log` and `safari-waist-counter-native-build.log`,
test60331 passed169 in2.18s; build49421 passed5m45s. Fresh low-counter pair
completed71.74s in `safari-waist-counter-final-review/`, receipt
`safari-waist-counter-final-capture.log`. Actual comparison and paired receipt
inspected: `state_preserved=true`, frame201, one actor. The8px green counter is
now below the table top and readable beside the player/seats; scoped corrected
counter height/top/plant/table/seat/floor view accepted. Three north-wall courses
are now closed, but the mixed block13 header at the counter junction remains
partially flat and still needs correction. This is not whole-room acceptance.
The original120-map/16-variant
objective remains active.

Junction follow-up: exact block13 upper4x1 and lower-right2x1 wall courses now
share the rear wall plane, with12px courses at base12/0. The counter remains8px.
The lower course exposed an old arcade row1 discard rule; matched Safari wall
courses now render exactly once. Initial test failed that rule and the first
native replay aborted at its assertion; neither is acceptance. Rechecked170
Kanto tests passed1.18s (`safari-junction-floor-rechecked-kanto-tests.log`), and
native build passed3m36s (`safari-junction-rechecked-native-build.log`). Final
pair completed68.69s, actual comparison/receipt inspected, state preserved,
frame169, one actor (`safari-junction-rechecked-review/SafariZoneMainOffice-compare.png`).
Wall junction, low green counter and preserved room layout are scoped accepted
at this pose. The preceding mixed-header pending note is superseded.

## Underground Path native floor follow-up, 2026-10-14

The historical paired view shows enlarged pixel stair-steps in the gray-and-white
corridor weave. Exact tile10 floor phases across blocks02/08/09/0a/0c/0d/0e/10/11/12
now use four triangles per cell, preserving opposed gray north/south wedges and
white east/west wedges. Colors sample the live tile; no generic paint is used.
Source phase masks exclude the ramp, wall cells and other maps. Floor height,
footing and authored claims are unchanged. The passing169-test run above covers
its equal32px² gray/white areas, winding, source/map guards and floor datum.

Production build and fresh paired visual acceptance remain pending. The
TrainerHouseB1F historical view was inspected too; its current generic beveled
finish still interpolates the raw facility floor atlas and needs a fresh view
before deciding its remaining material correction. The original full-scope
ledger below continues to apply.

The fresh three-map capture completed; actual Underground and basement pairs
were inspected and preserve state (Underground frame110/one actor; basement
frame182/three actors). Underground still shows raw pixel stair-steps despite
the fixture pass: investigate the production finish path before acceptance.
The basement still has dominating blue/white floor dither and generic plants.
These observations change the next actions; neither floor is accepted yet.
Images are in `safari-underground-basement-current-review/`, receipt
`safari-underground-basement-current-capture.log`.

The full terrain-builder regression passes0.03s with four synthetic real floor
cells (48 indices, constant source swatches and unchanged footing), in
`underground-full-builder-test.log`. Production replay with opt-in
`CRYSTAL_INTERIOR_FLOOR_TRACE` reports grid116x114,408 eligible source16 cells,
but only256 finished surfaces (`underground-production-trace-capture.log`).
Actual pair was inspected, state preserved/frame85/one actor, and still shows
raw pixel edges. Thus the fixture pass is insufficient. A further opt-in input
group trace now separates irregular index groups, normals and cell dimensions;
native build passed5m10s (`underground-input-groups-native-build.log`). The
production input trace reports2998 irregular six-index groups near eligible
floors, first bounds1.92px square (`underground-input-groups-capture.log`). An
experimental earlier finish order passed170 tests in0.92s and native build1m,
but its production trace still reports256 finished cells and the same irregular
groups (`underground-finish-order-capture.log`). The ineffective ordering edit
was reverted; inspect the earlier geometry producers and triangle grouping
before further changes. No ordinary gameplay/TUI diagnostics were added.

Production ownership trace identified the actual path: all408 source16 cells
are Flat but claimed by `special:ceramic-paving`, with no live profile placements.
The dungeon extension's NativeCeramic constructor interpolated the complete
source atlas through its beveled mesh, bypassing the room finish entirely.
Diagnostic native build passed2m22s (`underground-ownership-native-build.log`);
paired receipt preserves state/frame57/one actor and actual image was inspected
(`underground-ownership-review/`). That owning branch now uses the existing
four source-swatch wedges, leaving NativeStone/NativeCarpet unchanged. Its
regression now requires constant swatches and outward winding; the full-builder
regression also exercises image-sample/instance context. Initial compilation
caught an enum equality missing PartialEq; corrected with a variant match.
Rechecked Kanto test75104 and native build75694 are live, receipts
`underground-owner-basement-rechecked-{kanto-tests,native-build}.log`.
Visual acceptance remains pending the new executable.

TrainerHouseB1F now uses the existing facility source-field ceramic finish
shared with SilphCo1F, preserving blue/cream source donors and zero footing
while removing stretched one-pixel dither. Its regression checks swatch samples,
height, claims and custom ownership. The first floor-only run passed171 tests
in0.83s (`trainer-basement-floor-kanto-tests.log`); the combined owning-path
build/tests above and fresh production basement pair remain required. Plants
and restoration machinery remain open.

Combined verification completed:171 Kanto checks pass in0.82s; native build
passed1m20s (`underground-owner-basement-rechecked-{kanto-tests,native-build}.log`).
Fresh paired capture completed38.88s in
`underground-owner-basement-final-review/`; both actual comparison images and
paired-state receipts were inspected. Underground preserved state at frame87,
one actor: diagonal gray/white floor boundaries are now clean, with the source
orientation/colors and stair/wall layout retained. Trainer basement preserved
state at frame67, three actors: blue/cream fields remain arranged as in the
source, but enlarged pixel dither is gone. These floor corrections are scoped
accepted at these poses. The basement plants, tall restoration machines and
other original room findings remain open, as does the full120-map/16-variant
objective. Whitespace checks pass; packs and captures remain ignored.

## Trainer basement plant and instrument follow-up, 2026-10-14

Latest paired view exposed the generic terracotta plant pots and a large blank
front band on the instrument cabinets. Exact facility blocks0d/0e plant art
now uses the existing fitted potted-plant geometry with six live source roles:
green leaves, gold pot/shadow and soil. Every source cell, origin and map is
guarded; altered artwork and other rooms keep their existing path. Its
regression verifies claims, unchanged footing, all six sampled roles, mutation
rejection and map exclusion.

The complete source-verified facility instrument match now routes TrainerHouseB1F
to a closed source-gray body at its existing24px height/footprint. The first
8px source row is fitted to the horizontal lid; the remaining24px of controls
fills the vertical front, eliminating the generic blank head band. Ground and
claim handling remain the existing production matcher. A regression checks
horizontal lid/front source regions, front height, eight owned cells, unchanged
footing and each source mutation. Other instrument maps retain their model.
Initial compilation caught a missing qualification for the shared closed-box
helper; corrected before retry. Test82026/native60772 are live, receipts
`basement-fixture-rechecked-{kanto-tests,native-build}.log`. Final production
visual acceptance remains pending. The full120-map/16-variant goal remains open.

Verification completed:173 Kanto checks passed0.73s, native build passed35.79s
(`basement-fixture-rechecked-{kanto-tests,native-build}.log`). Fresh pair
completed51.85s (`basement-fixture-final-review/TrainerHouseB1F-compare.png`);
actual image and receipt inspected, `state_preserved=true`, frame49, three
actors. All four cabinet faces now carry their complete control rows up to the
horizontal source lid; the blank generic head band is gone. Both plants keep
their fitted geometry and now read source green leaves/gold pots. These fixture
corrections are scoped accepted at this pose, superseding the preceding pending
plant/instrument note. Basement architecture/partition finishes and all other
original unresolved findings remain subject to the full-scope review. Footing,
source claims and the accepted floor remain unchanged. Whitespace checks pass.

## Vermilion port passage follow-up, 2026-10-14

Fresh production pair (`port-passage-current-review/`) was actually inspected:
state preserved/frame68/two actors. Both plants remained flat, the foreground
chamber wall hid most of the player, and the generic gray floor lost the source
weave. Exact underground blocks29/2a now own two complete2x3 potted plants;
map/source/origin/full-art guards preserve unknown art and other rooms. The
existing fitted plant receives six live source green/gold/soil swatches.

The owning dungeon floor path now uses the source-swatch gray/white weave for
exact Vermilion floor phases, including mixed stair/plant/door blocks. Walls,
stairs, plant pixels and unknown phases remain excluded. Complete source-owned
Vermilion wall segments register their closed volume for production occluder
fade while floor backing remains opaque. Other passage rooms retain their
existing behavior; collision and footing are unchanged. New regressions cover
plant mutations/claims/footing/source roles and wall-group/floor separation.
Test79952 and native80442 are live, receipts
`port-passage-{kanto-tests,native-build}.log`. Current test compilation began
before the two new regressions were added, so rerun current-source tests after
it completes if they are absent. Production visual acceptance remains pending.
The full120-map/16-variant scope remains open.

Verified follow-up: the initial173-test run passed2.22s but omitted the newly
added regressions. Current-source rerun passed175 in1.08s
(`port-passage-current-source-kanto-tests.log`), including source-complete plant
volume/palette guards and complete wall segment fade/floor separation. Native
build passed3m32s (`port-passage-native-build.log`). Fresh pair completed44.72s,
actual image/receipt inspected (`port-passage-final-review/VermilionPortPassage-compare.png`):
state preserved/frame76/two actors. Player is now visible through the faded
closed foreground wall, both green/gold plants stand in volume, and the native
gray/white diagonal floor weave is restored. These corrections are scoped
accepted at this pose, superseding the preceding pending note. Source wall/door
materials, stair treatment and route walking remain subject to broader review;
no whole-room or full-goal completion is implied. Whitespace checks pass.

## Vermilion and Saffron gym materials, 2026-10-14

Fresh baseline batch completed97.31s (`gyms-material-current-review/`), both
actual images and paired-state receipts inspected. Vermilion preserved state
at frame126/six actors: its north-panel lower sections were generic white,
although the source uses blue. Saffron preserved state at frame115/seven actors:
its floor weave retained enlarged pixel stair-steps. Teleport pads and walls
remain readable; neither observation establishes whole-room acceptance.

Vermilion's complete guarded8x4 source panel keeps the existing closed volume,
height and central passage, but now binds body blue, light trim, dark upper
panel and plinth from four exact live source swatches. The regression checks
both panels' swatches, geometry height, normals, complete32-cell ownership,
unchanged footing, source mutation and map exclusion. The first panel-only
run completed; see `gym-native-panel-kanto-tests.log` for its result.

Saffron's exact ordinary floor source guards now use four native source-swatch
triangles for a clean diagonal weave, preserving gray/white coverage, tile
size and floor datum. The existing closed-volume test now checks constant
source samples as well as surface area, winding, volume and every edge being
shared twice; it is named as a Kanto regression so the main filter includes it.
Teleport pads, carpet, planters, unknown phases and custom owners retain their
paths. Current test1915/native86569 are live, receipts
`gym-material-current-{kanto-tests,native-build}.log`. Fresh production visual
acceptance is still required. Vermilion floor motifs and other original gym
findings remain open; the full120-map/16-variant objective is unchanged.

The panel-only run passed175 checks in0.86s. Combined verification initially
failed on a shared occluder `PreparedGroup.coverage_fade` initializer/destructure
while another owner was editing it; current source already contains both fields.
No occlusion edits were made here and the outdoor owner was notified. Retry
runs75886/46579 are live in
`gym-material-rechecked-{kanto-tests,native-build}.log`. Inspect terminal results
and fresh production comparisons before acceptance; the initial failed logs
remain diagnostic evidence, not passing checks.

Retry source checks passed176 in1.80s; native build passed1m56s
(`gym-material-rechecked-{kanto-tests,native-build}.log`). Additional existing
Saffron regressions passed14 in0.03s (`gym-material-saffron-tests.log`). Fresh
final batch is live in `gyms-material-final-review/`, receipt
`gyms-material-final-capture.log`. Vermilion's actual image/receipt has already
been inspected: state preserved/frame79/six actors. Blue lower panel fields,
dark upper panels and light trim now match the source palette, with the same
central opening. This palette correction is scoped accepted at this pose;
Vermilion floor refinement remains open. Inspect Saffron's final image before
accepting its floor correction.

Final batch completed107.03s (`gyms-material-final-capture.log`). Saffron's
actual comparison and paired receipt were inspected: state preserved/frame84,
seven actors. The gray/white weave now has clean diagonal edges without enlarged
pixel stair-steps; tile size, source colors, teleport panels, walls and actor
layout are retained. This floor-edge/material correction is scoped accepted at
this pose. The preceding Saffron pending note is superseded. Whole-room contrast
and composition, route/teleport walking, Vermilion floor refinement and the other
original gym findings remain open under the full120-map/16-variant objective.
Whitespace checks pass; no pack/content/export changes were made.

## Gym entrance guardian verification, 2026-10-14

Fresh default-position batch (`gym-guardians-current-review/`) was inspected for
Pewter, Fuchsia and Viridian. Paired receipts preserve state (Pewter frame80/four
actors, Fuchsia frame83/seven actors, Viridian frame77/one actor). The default
3D frames crop the guide/entrance fixtures below the useful camera region;
these images cannot establish missing statues. An initial interpretation of
Pewter's lower rocks as replacement statues was incorrect and is withdrawn.
The pack's actual tower19/1a source art matches the existing eight-cell guardian
selectors; the entrance pose must be inspected before changing these matchers.

Unobstructed Pewter entrance pair at(5,11) completed53.66s, actual image/receipt
inspected (`gym-guardian-entrance-review/PewterGym-compare.png`), state preserved,
frame74/four actors. Both existing native gold-head/light-pedestal statues flank
the entry with the guide and player visible; the central walkable opening and
entry carpet are retained. This closes the historical boulder/guardian visibility
finding at this entrance pose; no statue code was changed. An initial CLI call
lacked the required single-map screenshot prefix and failed before capture;
its corrected receipt is `gym-guardian-pewter-entrance-rechecked-capture.log`.
Fuchsia entrance capture50854 is live at(5,15), receipt
`gym-guardian-fuchsia-entrance-capture.log`; inspect it and then capture Viridian
at the entrance before accepting those findings. Whole-room/route review and
the full120-map/16-variant scope remain open.

Fuchsia entrance pair completed34.22s at(5,15), actual image/receipt inspected,
state preserved/frame70/seven actors. It exposed actual false lab machines:
legacy interior signatures interpreted lab1a's blank eight floor cells and
lab1b's statue drawing as restoration machines. Both false signatures were
removed; real Elm/Ruins restoration selectors remain. An integration regression
uses both actual blocks through the complete terrain builder with canonical
profiles and image-sample context, requiring exactly eight guardian-owned cells
and unclaimed neighboring floor. Initial test compilation caught a private
canonical-profile helper path; the helper is now visible within the mesh module
and the test calls that exact shared document, with no duplicate catalog.

Viridian entrance pair completed28.73s at(5,13), actual image/receipt inspected,
state preserved/frame72/one actor. Both statues are present; generic pale material
loses source gold-head/gray-pedestal colors. The already selected canonical
Kanto guardians now share the named head/pedestal source-color binding used by
Pewter, preserving geometry, height, footprint and footing. An invalid-phase
bounds guard also prevents malformed Viridian source rows/columns from indexing
the art array; its regression tests both blocks at4/255. Current test session from `gym-guardian-rechecked-kanto-tests.log` and native17131
(`gym-guardian-native-build.log`) are live. Inspect terminal tests and rebuild
final current source if needed before fresh entrance acceptance. Fuchsia/Viridian
corrections and the additional Kanto guardian palette maps remain pending visual
review; default cropping findings alone are not absence evidence.

Source recheck passed178 Kanto checks in1.53s
(`gym-guardian-rechecked-kanto-tests.log`). Initial native build passed1m12s;
current-source native rebuild passed1m21s (`gym-guardian-current-native-build.log`).
The sequential six-map entrance capture8779 is live. Actual images/receipts for
Pewter, Fuchsia, Viridian, Cerulean and Celadon have been inspected in
`gym-guardian-final-review/`: all preserve state, with frames77/60/78/87/134 and
actors4/7/1/1/7 respectively. Fuchsia now has exactly two gray native guardians;
false lab machines are gone. Viridian, Cerulean and Celadon now retain source
gold heads and matching pedestal colors. Pewter's already correct layout stays
intact. These statue corrections are scoped accepted at these poses; inspect
Vermilion after its final child capture completes. Per-map receipts are
`gym-guardian-final-MAP.log`. These do not prove route/collision gameplay.

The user clarified that the Lego-like16th-gym objects are Viridian's green maze
walls. Their four raised alternating cap inlays per8px section were removed.
The existing continuous coping is now beveled only along exposed top boundaries,
with unchanged hidden join coordinates, top height, vertex/triangle ordinals,
source ownership, collision, footing and cutaway groups. Fitted face normals are
recomputed from the retained triangles. The regression exercises all16 exposed
masks at three scales, requiring unchanged shared joins/height/counts, outward
faces and untouched cached models. Test90612/native52017 are live in
`viridian-coping-{kanto-tests,native-build}.log`; inspect results and a fresh
Viridian pair before visual acceptance. The old cap-inlay acceptance is superseded
by the user's correction. The full120-map/16-variant goal remains open.

The six-map guardian batch completed, including Vermilion235.93s. Its actual
entrance pair/receipt was inspected: state preserved/frame128/six actors, with
two gray native guardians and the guide visible. All six selected Kanto guardian
palette views are now scoped accepted at their entrance poses. No route/gameplay
completion is inferred from them.

Viridian's first coping run passed178/failed1: the extra bevel folded a refined
triangle at exposed mask3. Bounded diagnostic reproduced the exact face
(`viridian-coping-winding-diagnostic.log`); no failed test is acceptance. Native
build passed6m18s, and the intermediate actual image showed the checkerboard
inlays gone (`viridian-coping-current-review/`, capture81.34s), but geometry is
not accepted. The cached wall prototypes already have authored bevels. Current
source preserves all their positions, normals and triangles, removes the added
stud boxes, and adds subtle continuous/desaturated green stone finish variation.
The regression now checks every exposed-mask prototype at three scales for
unchanged closed geometry/join ordinals, outward faces and untouched caches.
This supersedes the earlier extra-deformation plan and its normal recalculation.

The user also requested open Vermilion puzzle cans. The production source-owned
GymBin branch now emits a tapered hollow can with thin rim, inner wall and deep
bottom; no disk closes its mouth. Physical width fits12px within the same16px
source plot, height/depth remain12px, and claims, puzzle selectors, collision
and footing stay unchanged. The shell regression measures outward winding,
each geometric edge shared twice, rim-only top area and a deep interior bottom
at three scales. Other trash-can models remain unchanged. Combined test4437 and
native61462 are live in `open-cans-coping-{kanto-tests,native-build}.log`; fresh
Viridian wall and Vermilion can views remain required before acceptance.

Combined current-source tests passed180 in1.55s, including both corrected coping
and hollow-can regressions; native build passed2m03s
(`open-cans-coping-{kanto-tests,native-build}.log`). Fresh two-map capture85842
is live in `open-cans-coping-final-review/`, receipt
`open-cans-coping-final-capture.log`. Inspect both actual comparisons and paired
receipts before scoped acceptance. The failed extra-deformation run remains
rejected; current coping uses the original authored bevels with subtle stone
finish variation and exact original join geometry.

Final two-map batch completed106.50s (`open-cans-coping-final-capture.log`).
Both actual comparisons and paired-state receipts were inspected. Viridian
preserved state/frame111/one actor: raised checkerboard pieces are gone and
continuous muted olive coping retains the original maze silhouette, openings
and authored bevels. Vermilion preserved state/frame100/six actors: all fifteen
puzzle cans have visibly open rims, dark inner walls and low interior bottoms,
with the source puzzle positions retained. These requested wall-top and open-can
corrections are scoped accepted at these poses in `open-cans-coping-final-review/`.
The extra-deformation failure remains rejected and superseded by exact original
geometry. Full route walking, Vermilion floor refinement, other original room
findings and the full120-map/16-variant objective remain open. Whitespace checks
pass; packs and render output remain ignored.

## Mount Moon gift shop correction, 2026-10-14

Exact traditional-house art now guards native shop displays, the low counter,
gold wall/radio courses, clock/cabinet paint and the real blue flower cushion.
Block04 tatami quadrants no longer become eight raised lavender cushions. The
three displays retain their gray source merchandise instead of invented pastel
goods. Their ground donor is the actual gold tile01, replacing the inherited
tile50 wall band. These presentation changes preserve claims, collision and
footing; neighboring Johto rooms retain their renderer.

Native build passed1m33s (`moon-gift-ground-native-build.log`). The actual noon
pair in `moon-gift-gold-ground-review/MountMoonGiftShop-compare.png` was inspected;
its receipt reports `state_preserved=true`, frame54 and one actor. The gold
backing defect is corrected. That inspection exposed overly large green mat
checks; source was refined to narrow woven lines and its final build/tests and
paired inspection are pending. Initial combined tests passed161 and failed two:
the gift-counter area expectation incorrectly assumed an inset (fixed), and an
outdoor paving observation interval (owner notified). Do not count that run as
passing (`moon-gift-ground-kanto-tests.log`).

The later complete-block fixture run passed163 Kanto tests in0.93s
(`moon-gift-complete-block-kanto-tests.log`). The cushion test now supplies the
complete source block, including its green floor donor; an earlier partial
fixture could not resolve the canonical cushion. The final woven-floor build
passed5m41s (`moon-gift-woven-native-build.log`). Final pair completed58.81s;
actual comparison inspected, `state_preserved=true`, frame93, one actor
(`moon-gift-woven-final-review/MountMoonGiftShop-compare.png`, receipt
`moon-gift-woven-final-capture.log`). Narrow green fibers replace the false
checker blocks; the gold floor, gray displays and blue flower cushion remain
readable. Scoped fixture/floor corrections accepted at this verified pose.
The recheck after selecting the exact cushion10 fixture profile passed163 in1.82s
(`moon-gift-verified-kanto-tests.log`). Shared outdoor owner was notified.

Initial2D and native location captures omit the shop NPCs. Source time masks and loaded-object
eligibility were traced: the location fixture initializes map objects before its
requested hour is applied. Reentry at the requested time remains necessary to
verify actor composition; no actors were invented or visibility flags changed.
The original120-map/16-variant scope remains active.

The first `DDDU` route from shop(4,4) crossed the real exit to
MountMoonSquare(13,7), but continued held movement south and stopped outside at
(13,12). Its inspected pair preserves state but is not shop reentry evidence
(`moon-gift-reentry-capture.log`,68.73s). A corrected noon entry begins outside
at(13,8) and uses production Up input to the observed doorway.

Corrected noon entry completed43.80s (`moon-gift-noon-entry-capture.log`). Actual
comparison inspected, `state_preserved=true`, three authoritative actors (player
and the day clerk/girl). Movement receipt proves Square(13,8)→door entry→
GiftShop(3,7)→(3,6). Source NPCs are visible/readable in both views, including the
clerk beside the real blue cushion. No visibility/gameplay repair was needed.
Final image `moon-gift-noon-entry-review/MountMoonGiftShop-compare.png` accepts
the corrected fixture/floor palette and scale with the native daytime roster;
this resolves the earlier gift-shop palette finding. The broader full-scope
work remains incomplete. SafariZoneMainOffice baseline was re-inspected: table
and seats remain flat source drawings and are a next concrete furniture defect.

## Native healing effects integration, 2026-10-14

Added distinct presentation-only HealMachineBall/HealMachineLamp effect IDs and
extraction of the live pack-backed sprites. The optional renderer maps their
source centers onto the matched native enclosure's30px front, with fixed face
orientation, source-derived size/flip and recessed ball depth. Lamps sit on the
front header. Camera/viewport movement translates the whole assembly together;
no foot anchor or camera pull is used for these fixture effects. Original2D
sprites are hidden only after the native renderer reports consuming them, and
return in2D or unsupported/custom cases. Script timing, animation palette,
textures, input and gameplay are unchanged.

A full16-cell source guard and the existing Kanto clinic predicate are required.
The canonical model must be available. Altered named profiles and overlapping
custom live fixtures retain their original renderer; this guard uses the
established canonical/custom classification. No terrain/collision/footing or
RTC behavior changed. Shared-render owner was coordinated before API/bridge
edits; its paving work remains separate.

All-features cargo check passed5m11s (`native-healing-check.log`). Current Kanto
checks passed155 (`native-healing-final-kanto-tests.log`), including all six
ball centers/two lamps, fixed face orientation, camera translation, all16 source
mutations, Johto exclusion and custom ownership. Actual native nurse-input
MinimalPlugins test passed3.74s (`native-healing-nurse-input-test.log`): production
keyboard advances the source scene, publishes the live ball texture and both
lamps, and verifies conditional source overlay visibility. Its explicit
one-Pokemon fixture does not bypass script state.

Native build passed1m10s (`native-healing-final-build.log`); the rebuild after
an idle-path scan optimization passed1m58s (`native-healing-current-build.log`).
Corrected native visual replay/pair acceptance is recorded below; later follow-ups
supersede this earlier pending state. This is not
whole clinic or whole120-map/16-variant completion.

### Six-party animation and cleanup follow-up

First real-input Pewter pair completed92.06s; actual comparison inspected,
`state_preserved=true`, nine actors (base plus one ball/two lamps), zero extra
terrain builds across288 input frames. Ball is on the healing face and lamps
on its header; source matches2D (`native-healing-first-paired-review/`, receipt
`native-healing-first-paired-capture.log`). Paired route capture now accepts
`--view both`, using the existing settled-route and retained-state hold.

Added bounded render-only `CRYSTAL_RENDER_TEST_PARTY_SIZE=1..6` under the existing
fresh location-party fixture. It adds legal owned Pokémon only and does not arm
scripts. Six-party Indigo route completed69.18s, state preserved, zero extra
terrain builds across448 frames/two origins; final pair is after effect retirement
and cannot prove active six-ball correspondence. Actual moving frames09/10/11
were inspected and show filling/blinking on the correct machine. Top-row balls
were partly obscured by the bay lip; moved their source cards just inside the
opening (0.15px depth), retaining the recessed background and exact source centers.

The complete six-party nurse input regression initially exceeded its512-frame
allowance, then reached all four palette phases and exposed stale sprites after
animation retirement. Fixed only the dialog/world-only rendering shortcut to
force existing cleanup while stale healing markers remain. Final complete nurse
input test passed11.91s (`native-healing-six-cleanup-test.log`): six balls/two lamps,
live pack texture handles, all four phases, conditional overlay visibility and
no source/published healing effects after the authored sequence. Early failures
are retained as failure evidence, not passes.

Final native cleanup/readability build passed51.35s
(`native-healing-cleanup-final-build.log`). Six-ball paired Pewter replay is live
in `native-healing-six-final-review/`, receipt `native-healing-six-final-capture.log`.
An intermediate final Kanto rerun hit another owner's temporary E0603 in kanto_shore.rs:125
(private outdoor_stone re-import); owner notified, file untouched. Earlier155
checks passed; no final rerun pass claimed until that repair and recheck finish.

### Final full-party paired verification

Final source checks passed158 Kanto tests after the shore-owner namespace repair
(`native-healing-lip-final-kanto-tests.log`); shore file remains untouched by this
task. Cleanup/readability native build passed51.35s and the passive milestone
QA build passed1m33s (`native-healing-milestone-build.log`).

Ordinary route timing plus the standard post-route wait can miss the short
full-party blink. Added optional render-only
`CRYSTAL_RENDER_TEST_HOLD_HEALING_BALLS=1..6`: after the actual native publication
contains the requested ball and the source animation is active, release keys
and freeze through the existing paired-state hold. It does not arm any script,
change the party or advance simulation. It rejects a route that never reaches
the requested live effect count.

Pewter full-party milestone pair completed39.66s. Actual comparison and receipt
inspected, `state_preserved=true`, fourteen actors (base plus six balls/two lamps).
All six balls are readable inside the native bay, lamps on the header, source
layout/palette matches2D. Zero extra terrain builds across362 real-input frames.
Scoped active six-party placement/readability accepted. Image
`native-healing-six-milestone-review/PewterPokecenter1F-compare.png`, receipt
`native-healing-six-milestone-capture.log`. The same final full-party Indigo
milestone pair completed47.57s in `native-healing-six-indigo-milestone-review/`.
Actual comparison and receipt inspected, `state_preserved=true`, fourteen actors,
zero extra terrain builds across338 real-input frames/two grid origins. All six
balls and both lamps remain correctly placed/readable beside the nurse, matching
source2D. Receipt `native-healing-six-indigo-milestone-capture.log`. Scoped native
healing effect placement/readability accepted at the verified Pewter/Indigo poses;
complete source timing/palette/cleanup is covered by the six-party input test.
No whole-clinic/full-scope completion is inferred.

## Current full-scope acceptance work

Keep the original120-map scope (100 indoor,13 gate,7 cave) and16 declared block
combinations. The entries below identify remaining acceptance work; historical
captures/audits later in this document are not current whole-room approval.

- Clinics: final idle walls/floors family review is complete at the inspected poses; actual3D nurse healing now has source/input tests and paired Pewter placement
  proof; Pewter/Indigo active-party views are now accepted; restoration machinery and
  counter-side NPC visibility remain. HP/PP restoration through production input is
  already verified separately.
- League: Will/Lance terrace support is verified at inspected static/walking
  poses; other room walking, Koga foliage, native wall/floor materials and Karen
  seams remain. Earned battle exit/Hall continuation proof is separate from the
  sixteen diagnostic block combinations; retain both verification scopes.
- HallOfFame: scoped machine/platform/terminal accepted from the Lance-earned
  scene; whole ceremony, stair walking and remaining room surfaces are pending.
- Silph/Soul: accepted scoped tables/sofas/vases/source paint do not finish room
  architecture, flooring, vegetation, counter trim readability or actor fidelity.
- Houses/Mansion/Oak: preserve accepted furniture scale/native desk/workstation
  corrections; remaining corridors, books/plants, roof/entrance surfaces and
  broader source fidelity still need final views.
- Caves/gyms/ports: revalidate current shared-owner work at unobstructed poses,
  including Tohjo/MountMoon/Seafoam support, VictoryRoad small objects,
  RockTunnel Flash on/off, entrance statues and VermilionPortPassage visibility.
- Other remaining rooms: SafariMainOffice furniture/partitions, TrainerHouseB1F
  and Underground floors, arcade furniture/floors,
  and every unresolved base-room finding retain their original scope.
- Station: idle train/ticket fixtures are scoped verified; timed boarding/night
  power and actual trips remain. User requested work beyond the station, so it
  remains deferred while other interior families are corrected.

Do not infer completion from mesh counts, preview flags, original base-review
coverage or these scoped acceptances. Fix and inspect the remaining requirements
against the production2D view before whole-scope completion.

## Remaining clinic service aprons, 2026-10-13

Extended the guarded tile01 row2 service-floor finish to native north-wall strips:
block02 cols2/3,03 cols0..2,08 cols0/1,13 all,2a cols1..3. Existing blocks05/06/07
and38 treatment is preserved. First Pewter paired review preserved state and
showed the new north strips, but exposed the still-raw patch below the PC.
In-memory pack inspection identifies that patch as block0c row0 cols2/3 tile01.
Added that exact source phase; the superseded family batch was interrupted and
owned intermediate images removed. Final149 checks also pass
(`clinic-pc-aprons-kanto-tests.log`). The new phases require Kanto Pokecenter1F and
flat/zero-height source geometry. Counter tiles0f, upper drawing rows, rejected
quadrants, unknown blocks and Johto rooms remain outside the new treatment.
Source palette uses the existing ceramic field/joint donors (2,0)/(0,0); no
invented colors or changes to walking datum, source claims or collision.

All149 Kanto checks passed (`clinic-aprons-kanto-tests.log`). Added checks cover
all sixteen new source phases, palette-only UVs, bounded material surface
depth, unchanged footing/authored cells, reserved/custom ownership, altered art
and neighboring counter/quadrant exclusions. Initial native build passed57.24s (`clinic-aprons-native-build.log`). Final
native build passed3m08s (`clinic-pc-aprons-native-build.log`). Final family batch
completed in `clinic-complete-aprons-family-review/`; all eleven actual comparison
images and paired-state receipts inspected, each `state_preserved=true`.
Scoped final wall courses and service-floor finishes accepted in Pewter,
Cerulean, Celadon, Cinnabar, Fuchsia, Lavender, Route10, Saffron, Vermilion,
Viridian and wider Indigo. The PC patch is fully finished, wall courses meet,
and native counter/floor palettes remain separate. Receipt
`clinic-complete-aprons-family-capture.log`. Counter-terminal NPC occlusion
remains visible in Fuchsia/Lavender/Viridian; this is not whole-room acceptance. The complete
clinic animation, restoration machinery and actor visibility remain open;
all other unfinished families and16 variants retain their original scope.

### Healing presentation integration still required

Current source evidence: `overworld_rendering.rs::spawn_visible_heal_machine`
spawns authored ball/lamp images as Sprite + FieldCommandMarker +
HealMachineWorldSprite. `fullscreen_scaling.rs` assigns those entities to the
classic world root. `render_mod.rs::publish_visual_world_frame` extracts map,
player/object sprites and grass effects, but no healing sprites; the spatial
support predicate does not explicitly fall back for healing either. The render
API currently has no healing-machine field. Nurse HP/PP/controller acceptance
therefore does not prove an animated3D machine. Required next proof is a real
native nurse interaction during retained healing frames; implement the missing
3D presentation integration if the authored effects are absent. The production
render-walk harness accepts A/B/Start and adds an explicit party fixture, so
input can reach the scene without cursor/flag seeding.

### Actual native nurse scene diagnostic

Production native input run completed77.59s from Pewter(3,4), route UAAAAAAAA,
explicit normal render-party fixture. Authoritative movement reaches(3,3),
nurse dialogue advances through Welcome/WeCanHeal/NurseTakePokemonText and
MUSIC_HEAL starts. No script cursor or healing flag was armed. Receipt
`clinic-native-nurse-scene-capture.log`, per-frame CSV and moving frames in
`clinic-native-nurse-scene/`. Actual frames02/03/04/07/08 and final PNG inspected.
One grid origin, zero extra terrain builds across288 input frames.

Final PNG shows an actual defect: source ball/lamp sprites float above the blue
control cabinet rather than inside the recessed healing enclosure. This is
not animated3D acceptance. `battle_hud_tile_origin` adds the source4px half-tile
center, so the original first ball's local native center is(12,10), followed by
(20,10),(12,15),(20,15),(12,20),(20,20); source lamp centers are(14,4),(18,4).
The source4x4 machine drawing can map these local pixels onto its30px native
front. Required implementation: explicit presentation-only healing effect IDs
and extraction of the pack-backed ball/lamp sprites, matched-enclosure placement
and original2D-overlay suppression while3D consumes them. Keep authored animation
phase, textures, script timing and gameplay unchanged. Camera goal notification failed because that task is archived; the active
outdoor/shared-render owner was notified before upcoming API/render_mod/visibility
edits. No projection, terrain, footing or RTC changes are intended. Full paired animation proof still
needs the corrected production scene at identical retained state.

## Native clinic wall courses, 2026-10-13

Clinic WallClinical panels no longer use generic cream enamel. Guarded native
front courses retain their dark horizontal source bands, with closed24px bodies,
existing3px depth, full-width meeting seams and original footing. Exact source
fragments cover blocks02,03,08 and13. Their native counter-junction artwork is
retained rather than replacing it with a guessed wall palette.

First tests passed146 and native build passed47.61s. Actual Pewter and wider
Indigo paired views were inspected with preserved state (`clinic-walls-first-
review/`, receipt `clinic-walls-first-capture.log`). Pewter source paint and
nurse visibility were good. Indigo exposed two continuity defects: the row above
the PC retained its legacy16px top, and block2a's mirrored counter wall had no
complete matcher. Added a two-cell PC header from16px to24px, conditional on a
canonical complete PC and unclaimed original wall cells. Header tests passed147,
build passed1m08s, and actual Indigo pose(3,9) was inspected (`clinic-walls-header-
final-review/`, state preserved). The header is aligned; the block2a section
still required correction, so whole wall-course acceptance was withheld.

The mirrored block2a matcher now verifies all16 native cells, protects any custom
ownership within that drawing, and models only its eight upper wall cells. It
leaves the lower counter/floor untouched. Mirrored checks passed148 and native build passed1m01s, receipts
`clinic-walls-mirrored-kanto-tests.log` and `clinic-walls-mirrored-native-build.log`.
Final refinement gives the PC header the same12px course as the upper half of
its24px neighbors (base12, top24), so stripe cadence also joins. Rebuilt final
native QA passed46.87s and all148 Kanto checks passed, receipts
`clinic-walls-course-native-build.log` and `clinic-walls-course-kanto-tests.log`.
Superseded mirrored-only capture was interrupted before review; owned images
removed. Final Pewter/wide Indigo pairs completed in
`clinic-walls-course-final-review/`, receipt `clinic-walls-course-final-capture.log`.
Both actual comparisons and receipts inspected, `state_preserved=true`. The
native dark courses now meet continuously across PC headers and Indigo's
mirrored wall. Nurse and equipment remain readable in Pewter; Indigo's wider
view verifies the full wall span and junction. Scoped wall/course/source-paint
acceptance at these two poses is complete. This does not accept the whole rooms.

Next clinic correction: remaining red service aprons under north walls retain
raw flat pixel diamonds while existing south-counter aprons use physical native
ceramic. In-memory pack inspection confirms tile01 at row2 of blocks02 (cols2/3),
03 (cols0..2),08 (cols0/1),13 (all),2a (cols1..3). These need guarded material
finishes, preserving counter top tiles0f and floor tile11, then one fresh family
review can verify both final walls and floors across the remaining clinic poses. Full-family wall views, red aprons,
restoration machinery, animation and actor visibility remain open.

## Blue clinic control cabinet correction, 2026-10-13

The exact block02 upper-left 2x2 drawing (04/05,14/15) was incorrectly a generic
WallClinical panel. A guarded native cabinet branch now retains all four source
courses, gives the blue upper section16px height above the13px lower control
console, and aligns both fronts at the block's south edge. Depth is6px for the
upper cabinet. Claims remain four independent source cells; custom ownership,
collision and footing remain unchanged. Other wall fragments and Johto clinics
retain their existing treatment.

The new alignment regression caught the inherited24px wall height (total37px).
That first pass was rejected; its capture batch was interrupted and owned
intermediate images removed. Final total height is29px, with exact-art guards,
source UV retention, Johto exclusion and unchanged footing checked. Final
145 Kanto tests passed (`clinic-cabinet-height-kanto-tests.log`) and native
build passed1m10s (`clinic-cabinet-height-native-build.log`). Fresh ten-room
paired review completed in `clinic-cabinets-final-family-review/`. Actual
comparisons and paired-state receipts were inspected for Pewter, Cerulean,
Celadon, Cinnabar, Fuchsia, Lavender, Route10, Saffron, Vermilion and Viridian;
all preserved state. The complete blue/gray bank, native PC and idle healer
remain readable at these poses; scoped cabinet/source-paint alignment accepted.
Receipt `clinic-cabinets-final-family-capture.log`. Indigo's closer nurse-side
pose(3,9) also completed in `clinic-cabinet-indigo-final-review/`; actual comparison
and receipt inspected, `state_preserved=true`. Full bank, PC, healer and nurse
are visible. Receipt `clinic-cabinet-indigo-final-capture.log`. Scoped idle
fixture/source-paint review is now accepted at the inspected poses in all eleven
Kanto1F clinics; this does not establish whole rooms or healing animation.

This earlier fixture acceptance preceded the wall correction recorded above.
Floor red aprons still need their remaining material finish, and some counter-side
NPCs need unobstructed pose checks. Full healing animation and restoration
machinery remain open, as do every other unfinished family and16variants.

## Current clinic terminal pass, 2026-10-13

Kanto lobby PCs now use their complete native 2x3 drawing (block08, origin2,1)
on a closed beveled enclosure. The nurse-side lower console independently uses
its exact block02 lower-left 2x2 drawing. Both retain existing physical height,
depth, claims, ground backing and footing. At this earlier checkpoint the upper blue/white cabinet remained separate
pending review; the correction and final evidence now appear above.
The healing enclosure keeps its prior side donor and recessed bay unchanged.

Focused PC regression passed: all six source courses, outward winding, six
claims, unchanged footing, exact-art guard and Johto exclusion. Receipt
`target/kanto-resume/receipts/clinic-pc-focused-test.log`.
Broad Kanto checks passed144 (`clinic-terminals-kanto-tests.log`). Native build
passed (6m15s, shared Bevy dependency rebuild), receipt
`clinic-terminals-native-build.log`. Fresh Pewter paired capture passed and the
actual comparison was inspected, `state_preserved=true`; receipt
`clinic-terminals-capture.log`, image
`target/kanto-resume/clinic-terminals-review/PewterPokecenter1F-compare.png`.
Scoped PC front/scale and lower gray console source paint are accepted at this
pose. At this earlier checkpoint the upper blue cabinet was absent behind the lower
console in 3D (subsequently corrected above), so whole clinic acceptance remains
open. No combined 2x4 fixture was guessed or claimed. Earlier nurse HP/PP verification is
complete (see the later verification section); stale running notes are historical.

## Whole-scope review continuation, 2026-10-13

### Latest healing enclosure and controller verification

Idle enclosure views are accepted in Pewter and the closer Indigo pose: both
actual2D/3D comparisons were inspected and preserved state. The green display,
empty recessed bay, native lower cross and nurse visibility remain readable.
Mesh/source guard checks passed143 Kanto tests and native QA passed36.70s.
Receipts `healing-enclosure-143-tests.log`, `healing-enclosure-native-build.log`,
`healing-enclosure-first-two-captures.log`, `healing-indigo-close-pair.log`.

Actual nurse-input regression82466 PASSED in both rooms (21.73s compile,1.47s
execution): native counter interaction, visible text, production A advances,
restored party HP and move PP, then production Start menu after dialogue ends.
Receipt `kanto-nurse-hp-pp-controller-pass.log`. The earlier apparent failure
was the test's wrong generic `window_open` predicate; diagnostic source text
was actually visible with `text_window_open=true`. No production controller
repair, script cursor arming, healing/victory flag seeding or direct HealParty
call was used. Party damage/PP fixture inputs are explicit; the nurse does the
actual restoration.

Animated3D healing presentation and the other clinic idle poses remain open.
The adjacent2x2 console and clinic PC now have guarded native fronts; their
paired visual review is recorded above. Restoration machinery remains open.
This verified idle/healing-state work does not establish whole clinical-family
or whole-room acceptance. All120 maps and16 declared variants remain in scope.

### Current native healing enclosure pass

`mesh/native_clinic.rs` now builds the idle Kanto healing machine as a closed
beveled enclosure with a recessed empty central bay and the complete original
4x4 front drawing. Its green top display, gray frame and lower red cross come
from live source UVs, with no invented static capsules or top emblem/readout.
The source guard requires a declared Kanto1F Pokecenter, HealingMachine role,
4x4 dimensions, exact block01 phases and all16 native tile identities. Existing
resolution/custom-profile protections,16 claims, physical plot/depth/30-pixel
height, original floor backing and native footing are preserved. Other clinics
retain their existing treatment. Controller input, healing state and scripts
are unchanged by this geometry pass.

Focused53724 is running (`/tmp/geothite-healing-enclosure-tests.log`) for native
claims/footing, full front area/source courses, empty recessed bay, outward
winding, all16 art mutations and unrelated clinic scope. Native QA9108 is
running (`/tmp/geothite-healing-enclosure-native.log`). Actual idle and healing
scene presentation still need fresh verification; no visual acceptance is
claimed yet. Restoration machines and all other open families remain required.

Lance carpet ascent5381 completed. Actual moving frames00/01 and final view in
`lance-terrace-carpet-walk/` were inspected. Its CSV proves authoritative movement
(4,7)->(4,6)->(4,5)->(4,4)->(4,3)->(4,2), crossing the native stair rows and
ending on the raised landing. Player and follower feet stay on the upper level;
no terrain rebuilds across4 origins. This is ordinary production input through
the render-location fixture; no trainer approach scene or dialogue is shown,
so it is not additional gameplay/script proof. Receipt `lance-carpet-ascent.log`.
A full UUDD route at the same carpet pose is now running to verify descent,
log `/tmp/geothite-lance-both-directions-walk.log`. The scoped static pair and
ascent do not substitute for its final descent check or whole-room acceptance.

Lance current paired capture95209 completed in `lance-terrace-final-review/`.
Actual comparison and receipt inspected: `state_preserved=true`, upper player
and Lance stand on the raised landing and the three courses retain native
carpet/gray paint. Scope accepts static source/layout/elevation correspondence,
not full room materials or walking yet. Native side-lane walk26199 completed
but CSV proves it stopped at(6,6), blocked before the stairs, then descended
only through the lower corridor. It is NOT stair ascent proof. Actual moving
frames00/01/02 were inspected and confirm that limitation. Receipt
`lance-side-lane-blocked-walk.log`. A carpet approach walk from(4,7), routeUU,
is now running (`/tmp/geothite-lance-carpet-walk.log`) to exercise the native
trainer approach without bypassing its input ownership or collision.

Final Lance snapshot40785 PASSED139 Kanto tests, including corrected side
winding, both north exit drawings, three native tread levels, fade ownership,
240 source mutations and custom fallback. Receipt
`lance-terrace-final-139-tests.log`. Current native81081 PASSED1m46s
(`lance-terrace-final-native-build.log`). Fresh paired upper-platform view is
running in `lance-terrace-final-review/`, log
`/tmp/geothite-lance-terrace-final-capture.log`. Native ascent/descent remains
required before accepting this scoped terrace correction.

Lance initial snapshot96691 finished: guard/customization test passed, geometry
test failed the side-face winding as expected from the source snapshot before
the correction. Receipt `lance-terrace-first-winding-failure.log`. Initial
native2533 passed1m16s (`lance-terrace-first-native-build.log`) but also predates
that correction. Final current-source broad tests and native QA rebuild are
running in `/tmp/geothite-lance-terrace-final-tests.log` and
`/tmp/geothite-lance-terrace-final-native.log`. Do not reuse the failed geometry
snapshot as visual acceptance.

### Latest Oak workstation acceptance

Corrected pair55809 completed in `oak-worktop-final-review/`. Actual comparison
and receipt inspected, `state_preserved=true`; the gold worktop remains separate
from the gray keyboard/tower/console and dark display. Scoped workstation paint
is accepted; fitted geometry, footprint and native footing remain unchanged.
Receipt `oak-worktop-final-capture.log`. Current region check64547 PASSED,
requiring gold vertices to stay in the published fitted worktop band and
retaining exact triangle/normal, uniform-face UV, source/customization guards.
Receipt `oak-worktop-region-test.log`. The broader current production-rule
snapshot passed142 Kanto checks (`oak-worktop-142-tests.log`), and native QA
passed37.35s (`oak-worktop-native-build.log`). The only later change was the
focused assertion, not production geometry or paint.

The actual factory `tools/build-johto-interiors.py` identifies healing machine
capsules, sockets, header emblem and readout bands separately from enclosure/
tray/vents. Its idle source drawing has an empty dark bay, green top display
and red cross at the lower panel; the existing model's raised capsules and top
cross/readout are not source-faithful. This is the next distinct clinical
geometry/palette correction, with healing behavior kept authoritative. No
healing/restoration-machine or whole clinical-family acceptance is inferred
from the Oak color pass. Lab book contents, walls, entry-floor art and the other
120-map/16-variant unresolved work remain required.

Tighter Oak worktop snapshot65036 PASSED142 Kanto checks; native8622 PASSED37.35s.
Receipts `oak-worktop-142-tests.log`, `oak-worktop-native-build.log`. Added a
focused assertion that all gold vertices stay in the fitted worktop band,
excluding tower and keyboard; this test-only addition is now running in
`/tmp/geothite-oak-worktop-region-test.log`. Fresh corrected pair is running in
`oak-worktop-final-review/`, log `/tmp/geothite-oak-worktop-final-capture.log`.
No corrected-palette visual acceptance is claimed before that image is viewed.

Oak first workstation pair16175 completed in `oak-workstation-final-review/`;
actual comparison and receipt inspected, `state_preserved=true`. Gold desk and
dark screen improve fidelity, but the broad low-height color rule also gilded
the keyboard/tower. That complete palette pass is REJECTED pending separation.
Using the actual factory in `tools/build-johto-interiors.py`, the rule now selects
only the enamel worktop's published y=0.73..0.85 band, while keyboard/tower/monitor
retain their verified equipment swatches. Whole-face source sampling still
preserves every triangle and normal. Final tests/native rebuild are running in
`/tmp/geothite-oak-worktop-final-tests.log` and
`/tmp/geothite-oak-worktop-final-native.log`.
Earlier focused10164 passed its geometry/guards test, and the exact compiled
suite passed142 Kanto checks; receipts `oak-workstation-focused-test.log` and
`oak-workstation-first-142-tests.log`. First native13728 passed41.08s
(`oak-workstation-first-native-build.log`). Those snapshots predate the tighter
worktop rule. First capture receipt `oak-workstation-first-palette-rejected.log`.
Healing/restoration machine geometry and all other open families remain required.

### Current clinical equipment pass

Oak's complete native4x3 workstation now maps its exact lab08/0c drawing to
live gray equipment/dark display pixels and its gold desk-base donor. The
existing canonical volume is retained. Whole-face painting separates the
published model's low desk parts (y<=0.95) from the monitor without UV blends
across different atlas cells; no triangle position or normal changes. The
source helper is OaksLab-only, strict on all12 tile identities/phases and the
selected workstation's eight named material roles; other lab treatments remain
unchanged and custom profiles retain their normal ownership.

Initial current source suite47696 PASSED141 Kanto checks (before the new focused
regression). Focused10164 is running (`/tmp/geothite-oak-workstation-focused-tests.log`)
for12 claims, native source gold/gray/ink samples, exact fitted triangle/normal
correspondence, uniform face UVs, all12 source mutations, custom profile fallback
and other-map baseline. Native QA13728 is running
(`/tmp/geothite-oak-workstation-native-build.log`). No new workstation visual
acceptance is claimed until the actual fresh pair is inspected.

Healing/restoration machines and other clinical fixtures remain open. Their
idle modeled balls/buttons and generic paint need distinct source fidelity
work; an Oak workstation color fix does not establish the clinical family's
completion or any whole-room acceptance.

### Latest Soul House offering acceptance

Current final suite78129 PASSED141 Kanto checks, including complete opaque
projected floor coverage, unchanged footing, retained memorial face, native
swatches, one complete vase fade owner and source/customization guards. Receipt
`soul-offerings-floor-final-141-tests.log`. Native floor build23380 passed1m40s
(`soul-offerings-floor-native-build.log`). Corrected pair24144 completed in
`soul-offerings-floor-final-review/`; actual comparison image and receipt were
inspected, `state_preserved=true`. The noisy under-vase patches are gone;
compact blue vases/pale blooms sit on the same ceramic floor finish as their
neighbors and remain separate from the gold memorials. Scoped offering
geometry/palette/backing is accepted. Receipt
`soul-offerings-floor-final-capture.log`. Whole-room architecture, remaining
entry-floor art and actor fidelity still require their own acceptance.

The earlier first pair retained raw noisy backing and was rejected, then fixed.
Its test's assumed triangle count was replaced with full opaque projected-area
coverage when the canonical bevel tessellation proved more detailed; no
production geometry was weakened for that correction. The original source
programs/content pack remain external/ignored, and no new model export or
compiled deliverable was added. Full120-map/16-variant scope remains open.

Soul backing native23380 PASSED1m40s, receipt
`soul-offerings-floor-native-build.log`; corrected pair24144 is running in
`soul-offerings-floor-final-review/`. Broad4333 passed140 and failed one test's
assumed backing triangle count: the canonical ceramic helper tessellates its
bevels more finely. Replaced that storage-count assumption with complete opaque
projected floor-area coverage (16x64 pixels), while retaining fade ownership,
memorial-face and native footing checks. Current final suite is running in
`/tmp/geothite-soul-offerings-floor-final-tests.log`; failure receipt
`soul-floor-tessellation-count-failure.log`. Production floor geometry was not
altered for this test correction.

Soul first pair47029 completed in `soul-offerings-final-review/`. Actual image
and receipt inspected, `state_preserved=true`; compact blue vases and pale
flowers read correctly and the memorials/aisle remain intact. Raw noisy checker
patches remain under each vase, so complete offering visual acceptance is
REJECTED pending floor backing. The pass now calls the existing native house
ceramic floor helper at the offering's actual world phase, using a verified
floor donor from its source guard. The helper was exposed within the mesh module
only; its implementation is unchanged. Floor backing remains opaque and
cached native footing unchanged. Final tests/native rebuild are running in
`/tmp/geothite-soul-offerings-floor-tests.log` and
`/tmp/geothite-soul-offerings-floor-native.log`.
Earlier compiled suite passed141 Kanto checks, receipt
`soul-offerings-first-141-tests.log`; first paired capture receipt
`soul-offerings-first-floor-rejected-capture.log`. Both predate this backing fix.

### Current Soul House offering pass

New `mesh/soul_offerings.rs` gives each native tile4f vase its own compact volume
below block28 or2a memorials. Whole4x4 guards verify the complete block, exact
source phases/art and all-floor margins, but claim only the one offering cell.
The existing strict FlowerStand model is fitted to a6x6 plot and8-pixel height,
with all material roles sampling the offering's own live blue/pale/dark/ink
pixels. No green/coral generic paint is retained. Only original floor-facing
vase triangles are removed; adjacent upright memorial faces remain intact.
The vase fades as one object while its independent floor backing stays opaque.
Native collision, cached footing, memorials and aisle state are unchanged.
Canonical bundled memorial profiles are accepted; matching user modifications
or renamed/new profiles protect the whole source guard.

Focused84368 PASSED two checks (38.85s compile,0.09s tests) covering both variants,
one claim/one fade owner, native source UVs, unchanged footing, retained memorial
face, all16 guard mutations, matching custom fallback and unrelated-map scope.
Receipt `soul-offerings-two-tests.log`. Native QA64176 PASSED46.90s
(`soul-offerings-native-qa-build.log`). Fresh SoulHouse pair is running in
`soul-offerings-final-review/`, log `/tmp/geothite-soul-offerings-capture.log`.
Current compiled Kanto suite is also running; no offering visual acceptance is
claimed until the actual room image and receipt are inspected.

### Latest Lance terrace acceptance

Current-source139 Kanto tests and native QA build passed (receipts
`lance-terrace-final-139-tests.log`, `lance-terrace-final-native-build.log`).
The paired upper view in `lance-terrace-final-review/` was inspected and
preserved state. Native carpet walk25645, routeUUDD from(4,7), completed:
CSV proves ascent through(4,6),(4,5),(4,4),(4,3) to(4,2), then descent through
(4,3),(4,4),(4,5),(4,6),(4,7) to(4,8). Actual moving frames02/03 were inspected;
player/follower stand at the upper and lower levels, with no terrain rebuilds
across5 origins. Receipt `lance-carpet-both-directions.log`, captures/trace in
`lance-terrace-both-directions/`. This accepts the scoped physical terrace/stair
and actor support correction in the render-location fixture; it does not claim
new script reachability, whole room materials or whole actor fidelity.

Rig owner was asked to check player facing correspondence in the actual static
pair (source appears front-facing while native shows the backpack). Rig/model
source was not modified here. Lance's gray statue paint, flat upper wall/pillar
art and remaining material/architectural finishes remain open. Other League
rooms, SoulHouse offerings, clinical fixtures, cave/gate issues and all other
unresolved120-map/16-variant work remain in scope.

### Current Lance terrace implementation

New `mesh/lance_terraces.rs` implements Lance's own three broad courses under
its16-pixel upper platform, using champions_room and10x24 core map dimensions.
It requires all240 native upper source cells (20x12), including the closed2e or
open0b north exit, exact subtile phases/art and no touching live customization.
Native source carpet and stair UVs remain intact; three full-width stair rows
rise through0,16/3,32/3 to16 pixels.48 stair cells receive authored support;
existing upper scenery rises with the platform, lower carpet remains at0.
Surface rebuilding preserves each fade owner and group order/cross-domain links
while allowing separate vertices at level seams. Core collision/scripts/warps
and positions remain unchanged.

Focused96691 is running (`/tmp/geothite-lance-terrace-tests.log`) for both north
exit drawings, three tread heights through the public footing resolver,48
claims, complete fade owner, outward winding,240 source mutations, matching
customization and wrong-map-size fallback. Native QA2533 is running
(`/tmp/geothite-lance-terrace-native-build.log`) with the same CLI package opt0
settings as the prior Hall capture. A side-face winding correction was made
after those build requests; verify their terminal snapshots and rebuild current
source if needed before accepting native side closures. No Lance terrace
visual acceptance is claimed yet.

### Current verification update

Final feature retry70077 PASSED137 Kanto tests, including exact front area and
spatial source detail. Receipt `hall-feature-final-137-tests.log`. Native QA
build46236 PASSED1m20s (`hall-feature-native-qa-build.log`) using CLI-only
opt-level0 for the renderer/frontend packages; it is not a release performance
measurement. Actual earned Hall pair69459 completed in
`hall-feature-final-review/`. Both actual2D/3D images and receipt were inspected,
`state_preserved=true`. The recording machine now has readable native screen,
frame and controls on its physical body, the platform has real stair treads,
and the separate terminal retains source blue/ink paint. Player/Lance position
and visible dialogue are preserved. Scoped machine/platform/terminal geometry
and paint are accepted in this actual scene. Receipt
`hall-feature-final-capture.log`. Hall floor/wall refinement, native stair
walking and the complete Hall-of-Fame ceremony still require verification.

Five diagnostic exit-only captures35023 completed in
`league-exit-only-review/`. All five actual comparison images and receipts were
inspected, each `state_preserved=true`; the north exit gap matches2D in
Will/Koga/Bruno/Karen/Lance. Receipt `league-exit-only-five-captures.log`.
These are explicit preview-flag combinations, not earned gameplay states.
Scope accepts only source/layout correspondence for this mask. Whole-room
acceptance remains open: Koga trees are flat, upper walls have generic cream
paint, Karen has an unresolved side landing/wall seam, and Lance's visibly
stepped central platform is still flat. Lance uses champions_room, native5x12
metatiles (10x24 core tiles), with front platform blocks3a/3e/3d/3f/3b. It needs
its own multi-level platform/stair assembly and source guard; it must not
inherit the other four rooms' layout. Its actual battle/Hall continuation is
separately passing. All120 maps and16 declared combinations remain in scope.

### Current Hall-of-Fame checkpoint

The guarded machine/platform pass is implemented in `mesh/hall_machine.rs`.
It requires the complete native12x8 drawing at source(4,0), six native blocks
30/31/32/34/35/36, exact atlas/phases,10x14 core map dimensions and no touching
live customization. It owns46 source cells; floor/wall guard cells and the
player dialogue aisle remain independent. A4-pixel platform with side wings
and four1-pixel stair treads supports a24-pixel recording body. Actual tread
sampling is enabled only when the complete resolved stair field matches.
Authoritative coordinates, collision, scripts, warps and dialogue are unchanged.

The terminal retains its existing geometry and now samples live source pixels
through a strict named-material mapping, including occlusion-remap ownership.
The first earned-scene pair in `hall-machine-first-review/` was inspected:
`state_preserved=true`. Scoped terminal blue/ink paint is accepted, but the
initial machine had uniform paint and lost its screen/control details; its
whole visual was rejected. Current source maps the verified front courses2/3,
top course1, shelf course4 and cropped stair course6 onto the physical faces.
Small face separations prevent coplanar flicker. A new assertion checks the
complete32x16 front area and spatial source UVs.

Evidence: `hall-machine-kanto-137-tests.log` and final
`hall-feature-final-137-tests.log` passed137 Kanto checks, including96 source
mutations, matching customization fallback, wrong map size, each tread,46 claims,
independent floor, winding, exact native-detail front area and unchanged
terminal geometry. First native80551 passed4m04s; screen-separation96870 passed
6m24s; final feature QA46236 passed1m20s. Their distinct receipts retain source
snapshot provenance. Actual final earned-scene pair69459 was inspected and
preserved state, accepting the scoped machine/platform/terminal correction.
Earlier uniform machine paint was rejected, then fixed before this acceptance.

Full scope remains120 Kanto maps and16 declared variants. Next work includes
Hall ceremony/stair walking, remaining League terrace/material work and remaining
room materials/geometry, SoulHouse offerings, clinical equipment, caves/gates
and the other unresolved families recorded below. Successful scoped checks do
not complete whole-room or whole-scope visual acceptance.

### Earlier verification history

The following checkpoints retain the evidence and defects found at each earlier
snapshot. References to running jobs or flat Hall machinery in this history
are superseded by the current checkpoint above and later terminal receipts.

Actual Lance-earned HallOfFame paired capture99758 completed with the fresh
production saved-session renderer. Both actual2D/3D images and the receipt in
`lance-victory-review/` were inspected, `state_preserved=true`. Player and Lance
stand at the same native continuation position and the visible Lance dialogue
is retained. The large northern HallOfFame machine/platform is still flat source
art, and its separate terminal remains generic cream/blue despite the dark blue
source palette. This is evidence of earned scene/layout correspondence, not
whole-room visual acceptance. Receipt `lance-hall-final-capture.log`. Those
machine/platform volumes and terminal source paint are the next concrete
HallOfFame correction; all120-map/16-variant scope remains open.

Current native build18837 PASSED in6m33s, receipt
`kanto-cap-final-native-build.log`. Fresh paired Silph/Will capture35335 completed
in `cap-counter-final-review/`; both actual comparison images and state receipts
were inspected, each `state_preserved=true`. Will's formerly floating north cap
now has a continuous source-painted front. The scoped closure is accepted;
generic cream walls and remaining flat floor/water fields are still open.
Silph retains clerk visibility, gold table and complete foreground sofa fade;
counter courses are present, but the black bottom stripe is subtle at this
camera scale and does not establish whole-counter paint acceptance. Actual
Lance-earned HallOfFame capture99758 completed as detailed above.

Current mesh checks completed: Kanto51586 PASSED all134 tests, including terrace
front closure area/treads/winding, counter fascia source samples and floor fade
owners. Reused its exact newly compiled test binary for the unknown-neighbor
invariant, which also PASSED; all quad attributes and both indexed triangles
survive compaction. Receipts `kanto-cap-final-134-tests.log` and
`unknown-neighbor-final-pass.log`. Outdoor owner notified.

An additional dataset `overworld_session` filter passed all three self-contained
bounds/traversal/override tests, but two older Route29 fixtures failed loading
`apps/web/assets/data/content-packs/index.json` from a sibling repository path.
Receipt `overworld-session-legacy-fixture-failure.log`; these failures are absent
source fixtures, not successful gameplay evidence. Actual external-pack Lance
regression remains separately passing. No legacy content was restored/copied.
Native18837 remains live; final graphical verification is still required.

Saved-layout negative controls17593 PASSED (7m30s compile,1 test): earned open
block accepted; closed edit, out-of-bounds override and undeclared metatile all
rejected. Receipt `receipts/saved-block-occupancy-negative-controls-pass.log`.
This supports the actual Lance pass without loosening save validity. Kanto51586
and native18837 remain live; collect their exact terminal outcomes next.

Latest source detail: northern cap closure has an exact full-face area assertion
(160x16 source pixels across all four rooms), alongside outward triangle winding
and existing actor tread checks. Current native build18837 and Kanto run51586
include the new closure; dataset controls17593 remain live. A process inventory
confirmed active compiler CPU work, not a stale lock, so no job was restarted
for elapsed time. Subsequent native captures should include both final counter
ink stripe and closed northern terrace front, then actual HallOfFame save and
remaining diagnostic exit-only variants. SoulHouse's baseline review confirms
its blue offering fixtures are still flat; that family remains open beyond the
current League/Silph corrections.

Current continuation: previous turn made source/runtime and gameplay progress.
Collected terminal failures96660/42791: concurrent grove-stream compile errors,
receipts `unknown-neighbor-shared-grove-compile-failure.log` and
`state-aware-shared-grove-native-failure.log`. Current source now has the module
and repaired Bevy APIs. Broad Kanto retry51586 is queued, log
`/tmp/geothite-kanto-state-aware-final-tests.log`; dataset negative controls17593
remain live. Current native retry includes final counter stripe and a new
northern terrace front closure using both native front courses (32 to16 pixel
levels), eliminating the unsupported floating lid without changing actor
support, core collision or map state. Native rebuild log
`/tmp/geothite-kanto-cap-final-native.log`; visual acceptance remains pending.

Live verification checkpoint: last polls confirm17593 (dataset negative
controls) is compiling crystal-core/audio dependencies;96660 (unknown-neighbor)
and42791 (native rebuild) remain live handles with recorded grove-stream compile
errors. Do not restart either merely from these observations; poll the same
handles to collect terminal outcomes. Final source diff whitespace check passed
for the owned runtime/dataset/furniture/test/handoff files. No files staged or
committed, and no content pack/export work was performed.

Latest actual gameplay result: state-aware Lance run58728 PASSED (1m28s build,
6.49s test). It earns BEAT_CHAMPION_LANCE, spends move PP, preserves the authored
exit override, leaves EXIT_OPEN unset as native, and executes the authored warp
into HallOfFame. Actual continuation save is in `lance-victory-saves/`.
Receipt `receipts/lance-state-aware-controller-pass.log`. Full HallOfFame scene
and current-renderer paired view remain pending; this is not a normal-level
League journey. Saved-block negative controls17593 and unknown-neighbor96660
remain running/queued.

Current native rebuild42791 encountered incomplete concurrent grove-stream
source (missing module file plus Bevy Projection/light field errors), log
`/tmp/geothite-state-aware-final-native.log`. Outdoor owner notified with exact
errors; preserve this failure and retry after its source settles. This failed
build is not a native verification of the state-aware repair or final counter
ink stripe. Existing compiled renderer remains usable for earlier scoped
geometry, but must not be represented as including these new changes.

Current-state update: trim native build88925 passed. Fresh paired capture72037
in `silph-counter-trim-review/` was inspected, `state_preserved=true`; the fascia
and light rim are visible and the sofa still fades correctly. The source tile77
also has a black bottom stripe; added its live ink sample as the lowest1-pixel
side band after this capture. That final stripe needs a rebuilt native view.

Lance retry76723 executed the real victory and post-battle movements through
script55, then failed snapshot validation at earned-open exit tile(4,1). The
validator rebuilt immutable baseline collision before saved block overrides.
Implemented `overworld_session_for_traversal_with_state`: validates block edits
before indexing/applying them and then checks occupancy. Runtime saved-state
validation, save restoration and warpcheck rebuild now use this state-aware
path. Closed, out-of-bounds and undeclared block edits remain rejected. Added
an explicit negative-control regression; actual Lance retry58728 and dataset
regression17593 are running. This is a gameplay defect, not visual acceptance.

Other-room terrace walk61939 failed in Koga at(4,2) with the same baseline
walkability error; Bruno/Karen were not attempted after that failure. Keep
all-room walk proof pending until the repaired controller is verified.
Unknown-neighbor run32704 compile failed in another owner's encounter helper;
that owner has repaired its Bevy attribute ID lookup. Warm retry96660 is queued
behind the current controller build; do not claim its pass yet.

Current continuation: the last turn made concrete progress through native terrace
ascent/descent and the fresh Silph sofa fade capture. Silph's L counter now adds
its live tile77 dark lower fascia and tile65 light upper rim, while preserving
its34 claims,8-pixel height, clerk aisle, native footprint and core footing.
Focused current-source run28901 passed all three Silph furniture tests (29.35s
compile,0.11s tests). Native build88925 is running, log
`/tmp/geothite-silph-counter-trim-native.log`; a fresh paired capture is required
before accepting the trim.

Added a separate Lance regression for actual battle/victory, spent PP, native
exit edit and authored continuation into HallOfFame without seeded victory
flags. First compile98969 failed on an incorrect UI field, repaired to the
production `window_open` field; retry76723 is running, log
`/tmp/geothite-lance-continuation-retry.log`. No continuation pass is claimed.

Outdoor full mesh verification found a vertex-storage-order assumption in the
unknown-neighbor gym-floor test after owner-preserving compaction. The test now
locates the complete untouched quad and verifies all positions, normals, UVs,
colors and both original indexed triangles, rather than requiring it to be the
last stored quad. Production geometry was unchanged for this correction;
focused run32704 is running, log `/tmp/geothite-unknown-neighbor-test.log`.
The outdoor owner was notified. Its strict shared-grove geometry invariant is
reported passing with the corrected exact source recipe; this does not accept
any uninspected exterior views.


Latest user correction: the Elite Four layout in the reviewed captures is a
multi-terrace with connecting stairs. Treat distinct terrace levels, landings,
risers and actor stair support as required visual work. The existing
`elite_four_room::shape` deliberately leaves block14 at zero height, and the
central stair drawing is currently flat source art; the earlier assumption that
this is only decorative apron art is superseded by the user's correction.
Derive complete source-scoped terrace/stair geometry and support; preserve
authoritative 2D coordinates, collisions, scripts and warps. Do not keep actors
at the lower plane when the visual floor gains elevation. Coordinate shared
terrain/footing files with the outdoor owner. Initial `mesh/league_terraces.rs`
is implemented for Will/Koga/Bruno/Karen: lower battle datum0, upper landing16,
blocked northern cap32 source pixels, eight2-pixel risers over the native32-pixel
stair run, plus stair sides and native front drop faces. Whole source guards
cover the complete16-cell stair and every visible upper course, native atlas/
phase, closed22/open16 callback drawing, map dimensions10x18 core tiles and
custom profiles. Final rendered actor footing samples each actual tread and
requires all16 resolved stair heights to match the authored field before using
the correction. Core coordinates/collision/scripts/warps are unchanged.
Initial test91687 failed missing math imports, repaired; retry20994 passed two
tests for all four maps, tread support, winding, altered source, custom floor
and map isolation (`receipts/league-terrace-two-tests.log`). Later dimension
guard and room-floor fade retention passed all134 Kanto checks in run64049
(`receipts/league-terrace-compaction-kanto-134.log`). Initial native build58302
passed2m38s (`receipts/league-terrace-initial-native-build.log`). The first paired captures completed for all four rooms in
`league-terrace-first-review/`; all four comparison images and state receipts
were inspected, each `state_preserved=true`. Receipt
`receipts/league-terrace-first-captures.log`. The initial argument failure is
retained in `league-terrace-capture-argument-failure.log` (`--maps` needs one CSV
value); the temporary pinned renderer was removed. Current native build7270
passed in5m00s (`receipts/league-terrace-final-native-build.log`).

Production native walk61873 completed with ordinary input ownership using the
native QA synthetic key snapshot, route `UUUUDDDD`, Will start(4,6). The actual
movement CSV records ascent through(4,5),(4,4) to landing(4,3), then descent
through(4,4),(4,5),(4,6) and further down the arena. Actual moving frames00,01,04
were inspected: player feet follow the physical stair and upper landing;
geometry remains stable across11 viewport origins and no terrain rebuilds.
Receipt `receipts/league-terrace-walk.log`, views and trace in
`league-terrace-walk/`. This establishes the scoped stair/support behavior in
Will, not whole architecture acceptance or all-room walking. The generic cream
walls, unmodeled source floor fields and floating-looking northern cap still
need refinement.

Current Silph native build57414 passed in6m39s, receipt
`receipts/silph-final-native-build.log`. Fresh noon capture45071 completed and
the actual comparison/state receipt in `silph-final-review/` was inspected:
`state_preserved=true`, selected table is gold with physical supports, three
blue sofas have volume, and the lower L counter retains a visible clerk head.
The scoped table paint is accepted. Sofa light backs are too broad and the
foreground sofa hides most of the player; counter dark lower trim is missing.
Whole furniture/room acceptance remains open. Receipt
`receipts/silph-final-first-capture.log`. Sofas now have blue backs with only a
0.3-pixel light top cap, and their complete bodies/feet are registered as one
fade object with opaque native floor retained. Focused current-source run26970
passed all three tests, including one complete fade group and separate floor
(`receipts/silph-sofa-visibility-three-tests.log`). Native refinement build13874
passed3m59s (`receipts/silph-sofa-refinement-build.log`). Refined paired capture
13715 was inspected in `silph-sofa-refinement-review/`, `state_preserved=true`;
blue back paint improves, but the foreground sofa remains opaque and still
hides the player. Scoped visibility remains rejected. Cause found:
`interior_surfaces::compact_surface` rebuilt the textured mesh without its fade
ranges. It now packs opaque vertices and each complete owner separately,
preserves range order/cross-domain links and all triangle attributes. Regression
84238 passed exact geometry signature with interleaved indices, two owners and
independent opaque floor (`receipts/room-floor-fade-compaction-test.log`). The
outdoor owner was notified. Fresh build7270 paired Silph capture4449 completed in
`silph-fade-final-review/`; the actual comparison and receipt were inspected,
`state_preserved=true`. The foreground sofa now fades as one complete object,
revealing the player's body while the checker floor stays opaque. This accepts
the scoped fade fix; counter trim and whole-room paint remain pending. Receipt
`receipts/silph-fade-final-capture.log`. The single-map default center attempt
failed on a blocked tile; retry used the batch renderer's nearest walkable pose. The earlier131-test
pass and native capture predate this sofa refinement.

Post-battle state verification is now implemented for Will/Koga/Bruno/Karen:
`kanto_elite_four_controller_wins_battles_and_opens_native_exits`. The fixture
uses one owned level100 Mew with Ice Beam and legal maximum PP Ups to avoid
level grinding. It never seeds door/victory flags or inserts block edits.
It executes the production entrance scene, walks through native collision to
the trainer, and presses A through real dialogue/command/move surfaces until
the battle script earns EXIT_OPEN. It requires trainer-battle phase, spent PP,
completed battle, earned exit flag, matching override and authoritative exit
metatile. Optional actual post-victory saves go to the ignored review directory.
Run36765 finished and failed in Will's real battle: enemy AI damage evaluation
returns `MissingTypeCategory { move_type: "CURSE_TYPE" }`. Receipt
`target/kanto-resume/receipts/elite-victory-curse-type-failure.log` preserves the
failure. The battle owner was notified with the exact controller path and error;
no battle flags, source moves or victory states were bypassed. Rerun after the
battle fix. The battle owner's latest status reports zero-power moves now
return zero damage before type-category lookup; its state/RNG regression
passed, including a control that a damaging move with the same unknown
type still fails correctly. Actual controller rerun is now running with
`--config 'profile.test.package.crystal-bevy.opt-level=0'` to reduce compile
cost while preserving the same controller, pack, combat and assertions.
Run84757 passed all four rooms in10.51s after a2m35s test compile. Each actual
trainer battle spends PP, completes, earns its native EXIT_OPEN flag and applies
the declared authoritative exit edit without seeded victory flags. Receipt
`target/kanto-resume/receipts/elite-victory-controller-pass.log`; scratch log
removed. Actual post-victory saves are in ignored
`target/kanto-resume/elite-victory-saves/`. Will/Koga/Bruno/Karen paired post-victory
captures completed in `elite-victory-review/`; all eight actual images and four
state receipts were inspected, each `state_preserved=true`. The earned northern
exit gap matches the 2D changed drawing in each room. This accepts only the
post-battle block/layout correspondence, not room architecture/paint/floors.
Receipts `elite-victory-will-capture.log` and `elite-victory-other-captures.log`.
Koga trees remain flat; generic cream wall paint and raw green/blue/red fields
remain unfinished. The existing saved-session production binary successfully
loaded the actual current saves; these views predate the latest global renderer
build and do not establish whole-room acceptance. Lance is a distinct continuation:
the pack's `LancesRoomLanceScript` sets BEAT_CHAMPION_LANCE and ENTRANCE_CLOSED,
not EXIT_OPEN; do not impose the other four rooms' flag contract on that script.
Its native exit-block change and HallOfFame continuation still need execution
proof. This fixture is script-state verification, not a full normal-level trip
through the League.

Whole-pack typed flag-writer inspection now confirms each first-four-room
ENTRANCE_CLOSED setter belongs only to its entry scene, and each EXIT_OPEN setter
only to its battle continuation. The only clears anywhere in map metadata are
the paired Indigo callback commands6..15. Lance has no authored EXIT_OPEN setter;
its battle continuation changes the exit block at command17 and sets the
ENTRANCE_CLOSED flag again at20. Therefore exit-only audit masks remain
diagnostic combinations with no identified earned visible path, rather than
gameplay proof; a transient paired-clear step in Indigo is not a visible League
room state. Keep all16 declared combinations in the visual/audit scope, and
execute actual post-battle states separately. No controller reachability pass
is inferred from this metadata inspection or from seeded story-test flags.

The user requested moving beyond the station. Current next correction is
SilphCo1F's meeting table: its existing complete8x8 guard owns only24 central
table cells, leaving four chairs and floor separate. The selected shared
DiningTable now uses live gold from facility tile51 pixel(4,4), verified in the
external pack, for both strict material roles. The native48x32 footprint and
9.135-pixel rise, claims and walking datum remain unchanged. Missing selected
models/materials retain original source rendering. Focused test81137 and native
location build7541 finished with missing `VisualWorldLighting::Dark` references
during the other goal's lighting migration. The lighting owner was notified;
the current API has the variant restored. These are failed builds, no pass or
visual acceptance. Fresh furniture test59816 and native build85382 are running,
logs `/tmp/geothite-kanto-silph-furniture-test.log` and
`/tmp/geothite-kanto-silph-furniture-build.log`. Counter/clerk occlusion still
requires correction, followed by whole-room review.
Current pack inspection identifies each Silph sofa as the complete upper4x2
of facility block2d: `[23,24,24,25;57,58,58,59]`. The lower two rows are native
checker floor and must not become sofa claims or support. Three occurrences
are visible in the reviewed pair. The new Silph-only complete4x4 guard owns
only the eight sofa cells. Original Rust geometry adds closed chamfered frame,
back, arms, two cushions and four feet within the30x14 physical footprint,
11-pixel rise. It samples tile58 pixel(4,4) blue, tile24 pixel(3,3) light, and
tile57 pixel(0,0) ink from the live source. Lower checker rows are guards only;
custom profiles win over the whole assembly and footing remains unchanged.
Tests cover all guard cells, map isolation, winding, claims, bounds and source
floor customization. Production visual acceptance is pending. The facility
sofa seat cushions now use the native blue field rather than the light back
sample. This final color adjustment happened while test59816/build85382 were
compiling, so rerun both after those exact handles finish before capturing.
Current processes were re-polled and confirmed live; do not restart them from
an observation timeout. Counter source inspection confirms native blocks34
and3f form the horizontal course and corner, block1e the upper vertical cap;
the receptionist is native movement tile(4,2). Existing generic facility wall
network does not support Silph and must not be broadened by atlas alone.
Use a complete Silph-scoped assembly with custom ownership protection for any
counter correction, preserving the L enclosure and native clerk aisle.
That assembly is now implemented: fixed native anchor(0,0), complete12x8 guard
across blocks35/35/1e and34/34/3f, only34 counter cells owned. Two joined closed
Rust courses keep the native L footprint and sampled tile41 gold, with8-pixel
presentation rise and unchanged footing. The receptionist's source foot(9,5),
chairs, upper wall and aisle remain unclaimed. Custom geometry anywhere in the
guard rejects the whole counter. New test covers every guard cell, anchor,
custom aisle, winding and8-pixel bounds. Full checker14205 is running with the
new34-cell expectation, log `/tmp/geothite-kanto-silph-counter-pack-check.log`.
Test59816/build85382 predate this counter addition and final cushion-color edit;
poll them to completion, then rerun the Silph suite/native build on current
source before the required fresh visual comparison. No counter visual acceptance
is claimed. Source tests alone do not establish clerk visibility.
Full checker14205 passed all five maps/nine states, including34 Silph counter
cells, all native actor/warp/custom/source guards, existing stored geometry and
editable source. Receipt `target/kanto-resume/receipts/silph-counter-full-check.log`;
its scratch log was removed. Furniture test59816 meanwhile
reported four new loading-trace compile errors serializing `Arc<str>` in
`mesh/build_scene.rs` and `lib.rs`. Trace-only map values now use `.as_ref()`;
the outdoor owner was notified. No serde dependency or trace meaning changes.
Test59816 finished with exit101; receipt
`target/kanto-resume/receipts/silph-loading-trace-test-failure.log`. All four trace
map fields are confirmed using `.as_ref()` now (two were concurrently repaired
by their owner). Final current-source test72882 passed all three Silph tests,
covering table source pigment/complete footprint, sofa winding/floor separation,
and counter native L/clerk aisle/custom guards. Receipt
`target/kanto-resume/receipts/silph-final-three-tests.log`; terminal scratch
test logs were removed. The same compiled current-production test binary also
passed all80 `kanto_native` checks; receipt
`target/kanto-resume/receipts/silph-kanto-broad-tests.log`. That binary predates
only the cfg(test) shared-grove helper correction. The broader `kanto` filter
also passed all131 cases (`receipts/silph-kanto-131-tests.log`), including the
three new furniture checks. These source/mesh tests do not establish visual
acceptance or script-state completion. Native build85382 finished successfully
in28m50s (`receipts/silph-initial-native-build.log`), but predates final counter/
cushion changes. Final current-source native rebuild57414 is running, log
`/tmp/geothite-kanto-silph-final-native-build.log`; follow with a fresh noon
Silph pair and inspect furniture paint/scale, clerk visibility and state receipt.
No current-source native-render success or visual acceptance is claimed yet.

Shared verification follow-up from the outdoor goal: its full mesh run passed730
and failed the grove geometry helper plus a cave-hatch assertion (cave owner
notified separately). `geometry_signature` attempted to read the placeholder
`leaf.mesh` after canonical geometry moved to `leaf.shared`. The cfg(test) helper
now reads the shared unuploaded CPU mesh and applies the instance transform to
positions/normals. Every triangle/normal/UV/color bit remains part of the
comparison; missing CPU geometry remains an explicit failure. Production meshes,
BVH, Time<Real> fade and source invariants are unchanged. The outdoor owner was
notified. Fresh helper regression75956 finished: the missing-attribute panic is
gone, but strict geometry equality fails at triangle524. Counts match; corner
positions, normals and colors differ substantially. The invariant is retained,
and the outdoor owner was notified to investigate the canonical shared-grove
recipe/placement versus the reference. Receipt
`receipts/grove-shared-geometry-equality-failure.log`; no grove acceptance is
claimed. Do not replace the equality check with counts or weaken its geometry
contract. Both terminal scratch logs were removed after receipt preservation.
The facility
pack verifier now includes24 sofa cells across the three occurrences. Its full
command initially failed an existing runtime/generator normal-precision equality
check before pack guards; its investigation and later full pass follow below.
Separate pack-guard run40103 passed all five maps/nine base-script states,
including Silph's24 sofa cells. The equality failure was investigated: only
36 normal components in two old assets differed at <=2.23e-16, and all float32
normal values are identical; meeting-table/chair assets are exactly equal.
The checker now compares only normals at actual renderer float32 precision,
while every other model field remains exact. Full checker14711 passed: four
deterministic stored models/96 closed positive-volume parts/2784 triangles,
all five maps/nine states and the existing editable source archive. This
geometry count describes the four stored assets; the new Rust sofa geometry
has its separate focused winding/bounds check still pending. Receipts `silph-facility-full-check.log`
and `silph-pack-guards.log` are under `target/kanto-resume/receipts/`.
The earlier precision and lighting failures are retained there as
`silph-original-normal-precision-failure.log`, `silph-lighting-test-failure.log`
and `silph-lighting-build-failure.log`; task scratch logs were removed.

Mansion1F lounge table follow-up: current pack block10 is the complete4x4
plain-gold drawing `[36,37,37,38;46,47,47,48;46,47,47,48;56,57,57,58]`.
Tile47 pixel(4,4) was inspected in memory (medium source swatch, live palette5).
It now resolves atomically only in CeladonMansion1F, selecting DiningTable with
both existing material roles sampled from that gold field and native tile11
floor backing. Selected-model/missing-role source fallback is inherited from
the existing dining-table path. No new geometry, source pack, collision or
footing changes. Test67269 passed (`receipts/lounge-table-focused.log`);
native location rebuild63738 passed in3m10s (`receipts/lounge-table-native-build.log`).
The test checks all16 claims, source point UVs, height,
unchanged footing, every altered source cell, map isolation and custom ownership.
Broader Kanto rerun26804 passed all128 cases (`receipts/lounge-table-kanto-128.log`);
its handle is terminal. Fresh Mansion1F capture76495 passed, and its actual
noon pair in `lounge-table-review/` was inspected: the flat gold drawing now
has the selected table body/supports, with gold paint and preserved reference
plot. Its receipt records `state_preserved=true`. The scoped table correction
is visually accepted; gold corridor motifs, book/plant paint, architecture
and whole-room actor fidelity remain unaccepted. Receipt
`receipts/lounge-table-capture.log`; capture handle is terminal.
Further script-state metadata inspection found IndigoPlateauPokecenter1F's
`IndigoPlateauPokecenter1FPrepareElite4Callback` clears each room's entrance/exit
flag pair in consecutive commands6..15. The exit-only mask's visible gameplay
reachability remains unproven; do not promote the synthetic mask to a playable
state without following these callback and battle paths.

Mansion checker continuation: current pack tile11 was inspected in memory:
native pixel(0,0) is the white/light swatch and(1,0) is the medium swatch that
the live room palette renders pink. Celadon Mansion tile11 floors now use the
shared physical ceramic checker courses with those exact pixels, replacing
the magnified bitmap checker. The separate tile01 gold ornament retains its
full drawing. The source-specific floor pass keeps claims and walking datum.
New regression `kanto_native_mansion_checker_keeps_live_colors_claims_and_walking_datum`
covers1F/2F/3F/RoofHouse, live pixel bounds, existing footing/claims and separate
gold ornament preservation. Test59871 passed (`receipts/mansion-checker-focused.log`).
Broader Kanto rerun34343 passed all126 cases (`receipts/mansion-checker-kanto-126.log`).
Production location/audit rebuild9011 passed in6m29s
(`receipts/mansion-checker-native-build.log`). Fresh1F/2F/3F/RoofHouse capture
and current whole-scope audit were launched; exact handles/logs follow below.
The earlier office pairs predate this change. Capture40289 passed, and all four
fresh pairs in `mansion-checker-review/` were inspected: the quiet pink/light
ceramic floor is readable with live source colors, and all four record
`state_preserved=true`. This scoped checker-floor correction is accepted.
Receipt `receipts/mansion-checker-captures.log`. The separate gold corridor motif
remains magnified source artwork. Mansion1F's then-flat gold table is corrected
in the current lounge-table follow-up above; roof
house furniture, planter paint and remaining architecture need their own review.
Whole Mansion room acceptance remains incomplete. Audit53010 passed120 maps and16
declared combinations with no mesher errors (`receipts/mansion-checker-audit.log`);
its temporary JSON was inspected and removed. This native build includes the Mansion floor change
and predates the separately pending League fade registration.

Foreground League barrier correction: existing complete slab/LeagueWall geometry
for the four elite_four_room maps was never registered as an occluder fade group.
`Detail::LeagueBarrier` now marks only its appended solid range, so the existing
whole-object fade can reveal the trainer while keeping geometry/source guards,
floor backing, neighboring scenery, collision and footing intact. The outdoor
owner was notified about the shared-file branch. New targeted regression11572
passed (`receipts/league-barrier-fade-focused.log`); it verifies complete
source ownership, one prepared fade object, unmarked floor/prior scenery and
unchanged footing, plus rejection of every altered source cell. Native renderer
rebuild48408 passed in3m44s (`receipts/league-fade-native-build.log`).
Controller-save regeneration55083 passed all five rooms with the external pack
(`receipts/league-fade-controller-saves.log`), writing fresh actual lock states to
`elite-entry-saves/`. Broader Kanto rerun11334 passed all127 cases
(`receipts/league-fade-kanto-127.log`). Correction capture19847 passed for
Will/Koga/Bruno/Karen; all four state receipts record `state_preserved=true`.
Output `elite-entry-fade-review/`, receipt `receipts/league-fade-first-captures.log`.
Fresh Will/Koga3D images were inspected and still show opaque barriers hiding
the player. This correction is not visually accepted. Code inspection confirms
the cause: fade easing reads virtual Time, frozen by PairedCaptureHold.
Only occluder easing now reads `Time<Real>`, allowing presentation opacity to
settle without moving actors or advancing gameplay. The outdoor owner was
notified. Existing occluder suite94804 passed all18 cases, including its real
ECS camera/root/asset-growth regression (`receipts/fade-real-clock-tests-18.log`);
native renderer rebuild65874 passed in2m16s (`receipts/fade-real-clock-native-build.log`).
Fresh correction capture96523 passed for the same four controller saves in
`elite-entry-real-fade-review/`. All four actual2D/3D pairs and receipts were
inspected, each `state_preserved=true`: the center source barrier fades as one
complete object and the trainer's full body is readable; adjacent source objects
and opaque floor backing remain intact. This scoped visibility correction is
visually accepted. Generic wall paint, floors/side scenery and post-battle variants
remain unfinished; no whole-room acceptance is claimed. Receipt
`receipts/elite-entry-real-fade-captures.log`. The earlier127-test pass predates
the clock change, whose18-case ECS suite and fresh production views are verified.
The existing real ECS occluder regression was strengthened to pause
`Time<Virtual>` before its120-update fade assertion and require world elapsed
time to remain exactly unchanged while opacity reaches the residual target.
This checks the actual paused-capture condition with real camera/root/material
state. Rerun19731 passed all18 cases (`receipts/fade-paused-world-tests-18.log`).
Production clock code is unchanged from rebuild65874. Those build/test/capture
handles are terminal. Temporary entry saves/backups were removed after review.
The controller entry regression now additionally opens/closes Start, approaches
each closed door and requires two Down presses to be rejected. Saves are exported
before that movement probe so reference positions remain stable. This strengthened
all-five-room regression40006 compiled and passed the four shared-atlas rooms,
then failed its test assumption in Lance: the replacement block's first row22
is traversable and the second row23 is the barrier. The fixture now derives
the actual blocked row from the authoritative replacement's collision permission
for walking Down, preserving both atlas-specific layouts. No gameplay change
was made. Final collision-derived rerun36014 passed all five rooms, including
production Start release and actual blocked Down movement at each native door.
Receipt `receipts/closed-door-native-collision-pass.log`; its handle is terminal.
This confirms the visual fade preserves physical entrance closure. It does not
prove post-battle exits or a full trip through previous rooms.

Editable train source is now reconstructed: Blender was installed through
Homebrew and generated the current tube/open-pose scene in ignored scratch.
Independent Blender inspection matched all242 meshes, float32 vertices, faces,
materials and colors to the generator. Repeated segment labels and Blender's
numeric name suffix rules invalidated the first name-based verifier assumptions;
the final complete geometry multiset verification passed. Generated JSON differs
from the canonical runtime only in float64 normal roundoff (max2.23e-16), with
every float32 normal identical and all other fields exact. The verified scene
was stored with the existing native gzip helper in `art/johto/source/train-station.blend`,
updating only its manifest entry/runtime hash. The complete train checker passes,
and the Blender preview in `train-editable-rebuild/preview0001.png` was inspected.
Receipts retain installation/build/geometry/full-check output. The temporary
uncompressed scene and generated runtime JSON were removed. Timed door motion
and production boarding/travel acceptance remain pending.

Oak's Lab desk follow-up: the complete six-by-three blank lab-table drawing
now selects DiningTable for OaksLab, removing the embedded display model's
three invented specimens. Both required table materials sample the original
gold worktop; the existing source claim and profile footing are preserved.
Johto retains LabDisplayTable. The original four-cell Red desk offset is
restricted to that width so it cannot shift Oak's wider desk. The focused
ownership/geometry/footing/custom-override regression passed, and the final
source rerun passed all125 Kanto tests (`receipts/oak-desk-kanto-125.log`), with
the focused result in `receipts/oak-desk-focused.log`. Initial production build
71527 passed in4m11s; final rebuild40462 passed in2m35s after the four-cell offset
restriction (`receipts/oak-desk-final-build.log`). Capture46484 passed and its
fresh noon pair in `oak-desk-review/OaksLab-compare.png` was inspected: a gold
table with no invented specimens now matches the blank source desk. Its receipt
records `state_preserved=true`. This scoped desk correction is visually accepted.
The lab equipment remains clinical/white, book contents retain generic colored
paint and the floor still magnifies horizontal bitmap bands. Whole-lab acceptance
remains pending. All jobs above are terminal.

Script-state acceptance planning: the16 synthetic audit combinations belong to
BrunosRoom, KarensRoom, KogasRoom, LancesRoom and WillsRoom (three masks each),
plus CeladonDeptStore6F (one mask). Current pack metadata was inspected in memory:
the department-store edit is `(12,0)->$03`, sourced from
`CeladonDeptStore6FHideRooftopStairsCallback`. Will's two distinct edits are
`(4,2)->$16` (door callback/battle script) and `(4,14)->$2a` (door-lock script/callback).
Lance's are `(4,0)->$0b` (door callback/Lance script) and `(4,22)->$34`
(door-lock script/callback). Duplicate command sites are collapsed by the audit;
its bit masks alone do not prove which combinations are reachable. Next verify
department-store callback output through the production controller, then each
Elite Four entry/lock/battle exit state and its resulting native paired view.
No script reachability or new variant visual acceptance is claimed here.

Department-store callback follow-up: added ignored external-pack regression
`kanto_department_store_sixth_floor_controller_applies_rooftop_stair_callback`
in `tests/kanto_interior_states.rs`. It initializes the production location
path, wraps its shell in VisibleShellController, settles, compares the callback
override and authoritative collision/layout metatile against the pack declaration,
requires a change from the original block, and opens the production Start menu.
Its actual command is running with handle3105:
`CARGO_TARGET_DIR=target/kanto-resume CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/core-modular.browser.crystalpack" cargo test --locked -p crystal-bevy --lib kanto_department_store_sixth_floor_controller -- --ignored`.
Initial handle3105 failed to compile because the regression used the
location-tester-only `render_test_hour` field with default features. The field
was removed; final rerun3498 passed (one test, zero failures, no ignored/skipped
cases in the actual run), recorded in `receipts/dept-sixth-callback-test.log`.
It proves the real callback replaces the original block in both the override
state and authoritative map, then the controller exposes Start. All handles
above are terminal. Capture77175 is terminal/passed: the noon pair in
`dept-sixth-callback-review/CeladonDeptStore6F-compare.png` was inspected and has
`state_preserved=true`. Together with the regression, this verifies the sole
department-store callback variant's production entry and paired presentation.
It does not prove the five Elite Four rooms'15 combinations. Cabinet paint, plants and seating remain
visually unfinished, so the entire room is not accepted.

The current pack's Elite Four callback flag metadata was also inspected in
memory. Each room uses `EVENT_<ROOM>_ENTRANCE_CLOSED` and
`EVENT_<ROOM>_EXIT_OPEN`, where ROOM is WILLS_ROOM, KOGAS_ROOM, BRUNOS_ROOM,
KARENS_ROOM or LANCES_ROOM. Their first entry scenes are LOCK_DOOR; Will/Koga/
Bruno/Karen have no coordinate events for that lock, while Lance's approach
uses `(4,5)` and `(5,5)` in APPROACH_LANCE. Therefore coordinate walking alone
cannot prove the door combinations. Verify the actual scene/controller path,
callback flag branches and post-battle exit mutation. Fixture-seeded callback
states must be labeled separately from flags earned through battle gameplay.

Next entry-scene regression is now implemented for all five Elite Four rooms:
`kanto_elite_four_controller_entry_scenes_lock_native_doors`. It starts at each
native southern warp, obtains the lock flag and block edit from that room's
typed script metadata, and uses VisibleShellController Up input to execute the
entry scene. It requires the actual flag, override and authoritative metatile
mutation. It seeds no flags or block edits. Initial run1176 compiled but failed
in WillsRoom: runtime-tile preview initialization deliberately skips arrival
scene dispatch. Production walking warps call `settle_visible_overworld_arrival`,
which arms and consumes the native scene (`battle_results.rs`, with the shared
scene arm function in `credits.rs`). The corrected fixture invokes that exact
production arrival transaction, verifies the entrance flag starts unset, and
then settles the controller. This is a test-arrival correction, not a claimed
gameplay fix or proof of a full trip through previous rooms.
Final run63877 is live, default features and `-- --ignored`, with the external
browser pack and `KANTO_INTERIOR_STATE_REVIEW_DIR` set to the ignored
`target/kanto-resume/elite-entry-saves`. Its log is
`/tmp/geothite-kanto-elite-arrival-test.log`; poll that exact handle. Only passing
rooms write controller saves after the real lock mutation. No pass, post-battle
state or Elite Four variant visual acceptance is claimed yet.
Production saved-session renderer build4090 passed in4m14s, using
wasm-dev/all-features and example `render_saved_session`. Preserve its owner's source.
Arrival rerun63877 compiled but failed in the real WillsRoom entry script:
`reanchormap $86` produced a saved map event whose byte operand was rejected by
identifier-only map_setup validation. This is an executed runtime blocker,
distinct from the earlier preview-arrival fixture error. The current runtime
already treats reanchormap as refresh-only; no named setup callbacks run.
The fix in core `state.rs` preserves bounded hexadecimal byte operands verbatim
only for reanchormap map events and pending refresh requests. Other command/token
contracts remain unchanged. Added core round-trip and malformed/overflow/padded
byte rejection regression in `systems/script_warps.rs`; notified the battle owner
about these shared-file edits. Core test17109 passed its byte/serde regression;
the complete core script-warp suite also passed all24 tests. Receipts are
`receipts/reanchor-core-focused.log` and `receipts/reanchor-core-suite-24.log`.
External-pack entry rerun78123 passed: the production arrival scenes lock the
native doors in all five rooms and update their authoritative maps. The test
seeds no entrance flags or blocks and saved each settled controller state in
`elite-entry-saves/`; its actual run skipped no cases. Receipt:
`receipts/elite-entry-reanchor-pass.log`. This proves entry locks, not post-battle
exit mutations or full traversal through previous rooms.
Updated renderer rebuild43373 passed in10m35s, recorded in
`receipts/elite-saved-render-reanchor-build.log`. Paired capture97548 passed,
using `render_saved_session PACK SAVE OUTPUT --pair` on all five actual
controller saves. Receipt `receipts/elite-entry-captures.log`, output
`elite-entry-review/`. All five2D/3D pairs and receipts were inspected, each
`state_preserved=true`. These entry states remain visually unaccepted:
Will/Koga/Bruno/Karen share a generic cream foreground barrier that hides most
of the player; Koga's trees remain flat drawings. Their floors/side areas retain
magnified bitmap motifs, including Will's water field and Bruno's red terrain.
Karen preserves the void boundaries but has generic wall paint and raw brick
flooring. Lance's red runner and player are readable, while its cream statue
paint differs from source gray and its locked entrance is outside the useful3D
framing in this pose; do not infer missing geometry from that view. It needs
an unobstructed entrance probe. Temporary saves/backups were removed after review;
rerun the passing controller regression with the review-dir variable to regenerate
them for future correction captures. Post-battle exits and remaining combinations
are still unverified. All capture/build handles in this paragraph are terminal.

Large-table/display follow-up: the eight-by-eight table shared by Fan Club,
WardensHome and TrainerHouse1F now resolves atomically across six metatiles.
It reuses selected DiningTable geometry with live gold paint and keeps the
unknown centerpiece's six original drawings on the tabletop. Small house
display drawings now use a selected LowTable6px stand and FlowerStand8px floral
arrangement above it, with required three/twelve material roles and independent
live yellow/green/wood source pixels. Unknown material bindings keep original2D;
Johto behavior is unchanged. Fresh pairs in `large-table-display-review/`
for Fan Club, WardensHome, TrainerHouse1F and Fuji were inspected: solid gold
tables and compact yellow/green floral stands are readable; all four preserve
game state. The scoped table body/display composition is accepted. The
unidentified centerpiece remains source artwork rather than a guessed3D asset.
The initial compile stopped on one extra parenthesis, now fixed. Native build
30149 passed in2m10s. The first broad Kanto run passed123/failed1: the unrelated
shallow-bank position equality at `new_bark.rs:3804` failed and was reported to
the outdoor owner. All three new table/display regressions passed. The latest
test-only optional-pack fixture correction passed a fresh focused run of73
native-source tests (`receipts/large-table-native-tests-73.log`). Capture25278
and focused run30841 are terminal. The outdoor owner corrected its obsolete
bank equality assertion after deliberate side relief; full Kanto rerun30598
passed all124 cases (`receipts/large-table-kanto-tests-124.log`). Outdoor work is
adding read-only jump-ledges metadata; preserve that shared API work.
SaffronMart, SaffronPokecenter1F/2FBeta and SeafoamGym were inspected next
(92 base pairs inspected). The retail/center pairs predate accepted paint/floor
fixes. Seafoam's older portal arch overlaps the player in projection; the cave
owner was notified to verify current placement/footing at a second position.
SilphCo1F and SoulHouse were inspected next (94 base pairs inspected). Silph's
sofas remain mostly flat, its meeting-table paint is generic cream and its
counter obscures the clerk. Soul House's gold markers are readable; small
offerings remain source drawings. Neither review establishes whole-room acceptance.
The remaining18 base pairs through WillsRoom were inspected, along with the
previously unreviewed LavenderSpeechHouse (all113 base comparisons now reviewed).
LavenderMart's valid retry was also inspected. Combined with the other six
retry/alternate maps, all120 distinct maps have review evidence; this is not
visual acceptance and RockTunnelB1F is still only Flash-active here.
Findings: TohjoFalls shows suspended player footing (reported to the cave owner);
VictoryRoad has partially embedded small actors/objects near raised terrain
(owner notified); VermilionPortPassage's front wall hides the player and its
plants remain flat. VermilionGym has generic white divider paint and busy floor
panels; its bins are readable. ViridianGym's source entrance statues are not
visible in that saved view and need an unobstructed probe before inferring they
are missing. WillsRoom's water/floor source textures and wall finishes remain
unrefined. The other houses retain readable proportions, while saved retail
and clinic pairs predate their accepted material fixes. The final table audit
build passed, and the external-pack audit verified all120 maps with zero mesher
errors and16 declared script-block combinations. Receipts are retained as
`receipts/large-table-audit-build.log` and `receipts/large-table-audit.log`;
the temporary JSON was removed. This is structural coverage, not proof of
reachable script states or whole-room visual acceptance.

Gate follow-up: fresh paired probes at Route5UndergroundPathEntrance(4,6)
and Route6UndergroundPathEntrance(4,6) in `gate-stair-probe/` prove the original
stair drawings are preserved flat, with both `state_preserved=true`.
The previous(4,4) view hid it under the follower; do not invent missing
geometry from that occluded view. The follower owner was notified about
scale/readability. Native Kanto connected counters now fit the authenticated
one-course8px front instead of12px, improving guard visibility while keeping
source claims, phone plots and cached footing unchanged. Legacy/Johto12px
fits and the cafe's independent height are retained. All121 Kanto tests passed
(`receipts/gate-height-tests-121.log`), and the production build passed in2m49s
(`receipts/gate-height-build.log`). Fresh Route15FuchsiaGate, Route19FuchsiaGate
and VictoryRoadGate pairs in `gate-height-review/` were inspected: guard torsos
are now visible above the counters, with all `state_preserved=true` receipts.
The scoped native counter-height correction is accepted for these positions
and states. Jobs91720/49551/29959 are terminal. The stair probe receipt is
`receipts/gate-stair-probe.log` (Route5), `receipts/route6-stair-probe.log`
and the capture receipt is `gate-height-capture.log`.
Route7SaffronGate and Route8SaffronGate were inspected (82 base pairs inspected),
both showing the same older high-counter occlusion. This is not visual acceptance.
SafariZoneBeta, SafariZoneFuchsiaGateBeta, SafariZoneMainOffice and
SafariZoneWardensHome were inspected next (86 base pairs inspected). Beta's
entrance fences remain flat source art; the outdoor owner was notified. The
full native image was inspected: southern fountain/paving is outside this
framing, so its absence cannot be inferred. MainOffice has flat round-table,
plant and partition drawings; WardensHome repeats the flat large table and
oversized small display pots seen in Fan Club/Fuji. These rooms remain open.
SaffronGym and SaffronMagnetTrainStation were inspected next (88 base pairs
inspected). Gym walls and teleport drawings are readable but floor diamonds
still dominate. The saved station pair predates the verified column-seat paint
and later floor finish; it is historical evidence, not current acceptance.

Clinic/bookcase follow-up: the explicit Mansion bookcase signature now includes
block18's complete2x3 drawing, reusing the selected canonical bookcase and its
native material guards. Kanto Pokecenter counter strips now use their live blue
fronts/cream caps with the existing five required canonical material roles.
Native Poké Ball cap markings remain live source decals; the block07 end owns
only its two counter columns, leaving the eastern doorway cells untouched.
Tile11 lobby/link-floor pink and tile01 service-apron darker pink now use quiet
ceramic seams from their own live palette pixels. No guessed RGB is used.
The final Kanto run passed120 tests (`receipts/clinic-final-tests-120.log`).
The first run exposed two fixture/assertion issues: block38 was incorrectly
tested as a lobby counter, and an older service-floor assertion expected raw
bitmap UVs. The corrected tests retain proper upstairs-map classification,
exact live field/joint pixels, complete source claims and unchanged footing.
Production build passed in6m00s (`receipts/clinic-build.log`). Fresh paired
Mansion3F, CinnabarPokecenter1F/2FBeta and IndigoPlateauPokecenter1F captures in
`clinic-bookcase-review/` were inspected with all `state_preserved=true`.
The scoped bookcase, pink ceramic floor and blue counter-paint/marking changes
are accepted for these positions/states. Small red patterned areas behind
the healer/PC and southwest source landing art remain; whole-clinic acceptance
is still open. The headless audit passed all120 maps /16 declared combinations
with zero mesher errors (`receipts/clinic-audit.log`); its temporary JSON was
inspected and removed. This still does not establish runtime script reachability.
All jobs from this follow-up are terminal.
Route2NuggetHouse, Route5CleanseTagHouse, Route5SaffronGate and
Route5UndergroundPathEntrance were inspected next (78 base comparisons).
The houses retain readable furniture. The gate repeats guard occlusion. The
underground stair appeared missing in that view; the later unobstructed probes
above corrected this finding. Its original drawing remains flat and visible.
The model owner was also asked to verify generic bird NPC species/scale in
Copycat2F, Fuji and Route28 from authoritative scripts rather than guessed icons.
Route6SaffronGate and Route6UndergroundPathEntrance were inspected too
(80 base pairs inspected); they repeat the older guard occlusion. The apparent
stair absence was later disproven by the unobstructed probes above.

Office/seat follow-up: Mansion2F/3F desks now resolve as complete4x4 native
desk/chair assemblies, including the split18/19/1b/1a occurrence. Canonical
LabWorkstation and LinkSeat geometry
use strict eight/four material roles and live source swatches. Unknown selected
materials retain flat2D. Kanto upstairs Pokecenter seats use their yellow art;
Johto is unchanged. New tests cover geometry/cache preservation, missing roles,
each altered source cell/phase, complete claims and unchanged footing.
The first116-test run and native/guard builds passed. Initial paired review in
`office-first-review/` preserves state and confirms gold desks/yellow center
seats, but exposed remaining flat desk-leg artwork. The final source match owns
the entire assembly, including those lower legs and two source floor cells;
the desk's geometry/anchor and actor footing stay unchanged. Desk and chair
selected-material checks are atomic. Final source tests passed all116 Kanto
cases (`receipts/office-complete-tests-116.log`), and native build92272 passed
in3m48s (`receipts/office-complete-build.log`). Fresh pairs for Mansion2F/3F,
CinnabarPokecenter2FBeta and Route10Pokecenter2FBeta in `office-complete-review/`
were inspected: gold desks/chairs, no lower-leg art remnants, and yellow center
seats. All four have `state_preserved=true`; receipt `office-complete-capture.log`.
The scoped desk/chair and upstairs seat-paint corrections are accepted for
these positions/states. Their rooms still have other open defects. All jobs
from this follow-up are terminal and the temporary pinned preview was removed.
The first source build failed on a concurrent species-rig test module path;
that owner corrected it before the successful rerun.
Mansion3F's center bookcase was subsequently corrected and verified in the
clinic/bookcase follow-up above. The room's floor/material refinements remain
open; desk, chair and bookcase acceptance does not establish whole-room acceptance.

RedsHouse2F, RockTunnel1F and Route10Pokecenter1F/2FBeta were inspected next
(64 base comparisons inspected). Red2F keeps the corrected furniture scale.
RockTunnel1F repeats the no-Flash dark2D/lit3D defect; the day/night owner was
notified to verify both floors against current code. Route10 repeats the known
Pokecenter floor/counter/seat defects. Its upstairs seat paint is included in
the new implementation and needs a fresh native review.
Route12SuperRodHouse, Route15FuchsiaGate, Route16FuchsiaSpeechHouse and
Route16Gate were inspected too (68 base pairs inspected). The houses keep
readable furniture scale. Both gates retain source colors and entry gaps, but
the guard's body is nearly hidden behind the green booth; refine counter
height/readability against the full source sprite before acceptance.
Route17Route18Gate and Route19FuchsiaGate were inspected too (70 base pairs
inspected); both repeat the guard occlusion issue, including the side booth.
Route26HealHouse and Route27SandstormHouse were inspected next (72 base pairs
inspected). Their furniture retains the same readable proportions and native
gold/blue/pink colors as the other ordinary houses.
Route28SteelWingHouse and Route2Gate were inspected too (74 base pairs
inspected). The house repeats the readable furniture; its bird interpretation
belongs to model work. Route2 has clear source entry gaps and no guard behind
its side booth in this state; other guarded gates still need the occlusion fix.

Current follow-up: the blue Goldenrod fixtures were re-inspected in the paired
ticket-box capture: four upright cabinets face the waiting room. The train
doors remain an authored open pose, with timed movement still pending.

Native Celadon arcade paint is implemented using nine strict material roles
and complete source drawings; canonical geometry and Johto paint are preserved.
Direct pack inspection confirmed separate palettes for trim tileA0 (palette12)
and center tileA1 (palette9). Cabinet bodies sample A1 pixel(3,4), relative
drawing(11,4); trim samples A0. The final production pair in
`arcade-pink-review/` was inspected with `state_preserved=true`: pink bodies,
orange trim and black screens now follow the native drawing. This accepts the
scoped cabinet-paint correction, not the whole arcade room. Earlier orange and
cream captures are rejected and retained as diagnostics.

All113 Kanto tests passed, including the arcade guards and final department-store
fallback (`receipts/kanto-source-tests-113.log`). After the last palette-pixel
correction, both focused arcade tests passed again, the final production build
passed in2m53s and its paired capture passed. Receipts are `arcade-pink-tests.log`,
`arcade-pink-build.log` and `arcade-pink-capture.log`. The focused capture-stability
regression passed too (`capture-stability-test.log`). All jobs listed in this
follow-up are terminal; no duplicate build or capture is needed.

Twelve additional base comparisons have now been inspected: CinnabarPokecenter2FBeta,
CopycatsHouse1F/2F, DayOfWeekSiblingsHouse, DiglettsCave, FightingDojo,
FuchsiaGym/Mart/Pokecenter1F/Pokecenter2FBeta, HallOfFame and KarensRoom.
This brings inspected base pairs to40, without blanket acceptance. The upstairs
Pokecenter gray seats and patterned floor defects recur. FuchsiaGym's entrance
statues are missing in3D. FightingDojo's statues exist but the enlarged floor
grain needs refinement. HallOfFame still retains the throne as flat source art
and its terminal has generic clinical paint. KarensRoom needs material and floor
refinement. Copycat furniture proportions are readable; the large Pokémon
actors are owned by the concurrent model/follower work. Mansion2F/3F were also
re-inspected: office desks remain flat and are still an open requirement.

Six more base pairs were inspected afterward: KogasRoom, LancesRoom,
LavRadioTower1F, LavenderNameRater and LavenderPokecenter1F/2FBeta (46 inspected
base pairs in total). Koga's trees still lie flat, and Lance's side architecture
and end mural remain flat source art. Lavender centers repeat the floor/seat
defects above. The settled recaptures for CeruleanMart,
IndigoPlateauPokecenter1F, TrainerHouse1F and ViridianMart were inspected too:
Mart counter paint is corrected; Indigo retains generic clinical counter
paint; Trainer House's large table and centerpiece remain flat. All are still
review findings, not acceptance. The runtime train geometry checker was
rerun successfully in `receipts/train-runtime-recheck.log`.

MountMoon and MountMoonGiftShop were inspected next (48 base pairs inspected).
MountMoon's saved capture shows suspended player footing and exit-opening
terrain differences; the cave-exit owner was notified to verify current code.
The gift shop uses generic cream/brown furniture and lavender floor sections
where the source is gold/green; its source palette still needs correction.
MrFujisHouse and MrPsychicsHouse were also inspected (50 base pairs inspected).
Their compact stools, TV and dining-table proportions are readable. Fuji's
small display plants use the generic large pot/leaf shape and need a separate
source-family refinement; the room's Pokémon scale belongs to the model owner.
OaksLab and PewterGym were inspected too (52 base pairs inspected). Oak's gold
desk is replaced by a white clinical table with invented visible specimens;
its workstation also keeps generic clinical paint. These need source-fidelity
correction. Pewter's entrance statue positions currently become boulders;
require a whole-statue drawing match before the rock adapter, preserving the
walkable entry. Its gold stone colors and rock layout are otherwise readable.
PewterMart and PewterNidoranSpeechHouse were inspected (54 base pairs inspected).
The saved Mart pair predates the counter-paint fix; recapture it before visual
acceptance. The speech house has the same readable furniture proportions as
other ordinary houses; its NPC Pokémon interpretation belongs to model work.
PewterPokecenter1F/2FBeta, PewterSnoozeSpeechHouse, PokemonFanClub,
PowerPlant and RedsHouse1F were then inspected (60 base pairs inspected).
The centers repeat the existing defects. Fan Club's table/centerpiece remain
flat and its small display plants are oversized generic pots. Power Plant's
office computers retain generic clinical paint, machinery banks remain partly
flat and the enlarged source floor texture needs refinement. Red1F's furniture
proportions remain readable. None of these reviews establishes full acceptance.

The previous turn made verified progress (train shell, doors, ticket boxes,
source tests and paired station inspection). This continuation keeps the
entire 120-map / 16-variant scope active.

- Fresh source audit: all120 maps and16 declared combinations, zero mesher
  errors (`whole-scope-audit.log`). It still does not prove script reachability.
- Initial whole-scope paired run produced113 valid base comparisons in
  `full-native-review/`. Initial capture failures were CeruleanMart,
  CinnabarPokecenter1F, IndigoPlateauPokecenter1F, LavenderMart,
  RockTunnelB1F, TrainerHouse1F and ViridianMart. The first batch stopped at
  CeruleanMart; a verified per-map continuation completed the rest, retaining
  every failure rather than accepting mismatched state.
- Inspected the first28 alphabetical base pairs through
  CeruleanTradeSpeechHouse. These are reviews, not blanket acceptance.
  Retail counter fronts had generic pink paint; Mansion2F/3F workstations
  remained flat; arcade cabinet bodies had generic pink paint; Pokecenter
  floors retain enlarged noisy diamonds, upstairs chairs have generic gray
  paint, and the southwest source-art remnant remains. Many later captures
  still need human-equivalent visual inspection.
- Saffron block02 column chairs now use the same native orange/yellow paint
  as block01 row chairs. Both complete two-by-two occurrences keep their
  footprints and footing; altered art and the floor half are rejected.
- Kanto Mart and Celadon department-store counters retain canonical full/body
  geometry, fitted normals and indices while using five required named
  material roles with live cream/gold/ink swatches. Unknown roles retain2D.
  Ordinary Mart strips reject every altered source cell; service loops retain
  their full fingerprints, clerk aisles, custom overrides and footing. Johto
  paint is unchanged. Fresh109-test source run passed; fresh paired1F/2F/4F
  store and Saffron captures in `counter-refinement-review/` were inspected,
  with all `state_preserved=true` receipts. Pink counter fronts are corrected.
- With the exterior owner's clearance, CeladonMansionRoof's existing access
  hut now uses seven required material roles: gold roof, blue body/trim and
  gray door/handle. Its geometry is identical to the canonical hut. Roof
  paving uses its own live source cells over a closed substrate below the
  unchanged actor datum. Fresh111 Kanto tests passed. The paired capture in
  `mansion-roof-review/` was inspected and preserves state; gold/blue/gray/cream
  finishes are accepted for this position and state.
- Capture diagnostics identified the transient actor failure exactly:
  CeruleanMart Object(2) center(160,128) reconciles to(157.84944,128), and
  Indigo's Object(1) center(160,-224) to(160,-227.17487), with core/LCD,
  texture, size, flip, priority and facing unchanged. RuntimeTickTimer is
  already frozen; enabling read-only actor extraction can reconcile newly
  spawned NPC transforms one update later at its frozen subframe. The new
  capture baseline waits for three equal source snapshots. Strict checks
  throughout both GPU readbacks are unchanged. New native build passed; all four failed-map recaptures (CeruleanMart,
  IndigoPlateauPokecenter1F, TrainerHouse1F and ViridianMart) passed with
  `state_preserved=true` in `settled-capture-review/`. The focused Bevy
  regression passed; the four settled pairs were also visually inspected.
- RockTunnelB1F without Flash has legitimate black2D terrain but incorrectly
  lit3D terrain. Do not approve that state or relax the blank-image heuristic
  to hide it. A disposable `--preview-flag STATUSFLAGS_FLASH` pair at(15,18)
  in `rock-tunnel-flash-review/` was inspected: purple stone and terrain match
  the lit source. This is a visual fixture, not executed Flash gameplay proof.
  The day/night and cave-exit goals were notified about dark-state fidelity.

The combined capture corpus now has valid paired receipts for all120 distinct
maps. RockTunnelB1F is represented by its explicit Flash-active fixture; its
no-Flash rendering remains unaccepted. Captures are scattered across the base
113-map directory and the refinement/retry directories named above. Capture
coverage is not visual acceptance:113 base pairs, the four settled recaptures,
the CeruleanMart retry, counter/seat refinements, mansion-roof refinement and
Flash-active cave pair have been inspected. Review findings are listed above.
The first capture review is complete. Observed defects still need fixes and
affected maps need fresh production captures before visual acceptance.

The native department-store missing-material path was also tightened: failed
material binding restores every owned sparse source cell as an original flat2D
drawing, clears authored coverage and prevents the legacy3D counter extrusion
from acting as an alternate fallback. Known-material geometry is unchanged.
The fresh source rerun passed all113 Kanto tests, including the final fallback
path and new arcade guards (`receipts/kanto-source-tests-113.log`). The focused
capture regression passed too (`receipts/capture-stability-test.log`). The
initial whole-corpus capture/audit jobs are terminal; current arcade verification
handles are listed at the top. The pinned full capture executable was removed
after the batch ended. The refreshed compiled
source audit again passed120 maps /16 combinations with zero errors, recorded
in `whole-scope-refined-audit.log`; its temporary JSON was removed.

Remaining work includes review/correction of the remaining map captures,
Mansion floors, other arcade furniture/floors, remaining Pokecenter fixtures/floor art,
other observed family defects, actual script-variant reachability and native
variant review and timed train-door animation. Editable Blender reconstruction
is now verified in the current continuation above.
Do not mark the goal complete from these scoped fixes or capture counts.

## Destination-host continuation, 2026-10-13

The destination checkout started clean, with the shared renderer integrations
present in source control. The historical transfer receipts below describe the
old host; their target files are absent here and are not current verification.
The old `realtime-clock.browser.crystalpack` is also absent. The pack-build task
owns export and publication; this task consumes its ignored
`content-packs/core-modular.browser.crystalpack` and does not run competing
exporters. Selected furniture is present in `assets/open-models/interiors`.

This task uses `CARGO_TARGET_DIR="$PWD/target/kanto-resume"`. Interior files
are owned here; exterior scene/shader changes require coordination with the
outdoor task. Preserve the concurrent battle-model changes.

Canonical `DrawerCabinet` and `GameConsole` now retain geometry while sampling their
own complete players-house drawings in Kanto rooms. All four named material
roles are required; unknown roles retain 2D. The cupboard uses drawer art from
tiles0e/3a rather than its first, floor-only row. Console controls/shell/highlights
use tile4a's black/dark/white pigments, removing the generic red button paint.
Source pixels were inspected directly from the current ignored pack in memory.
Both new regressions passed: unchanged geometry and cached paint, missing-role
rejection, complete drawing ownership, altered drawing rejection, unchanged
footing and Johto isolation. Native Red1F/Red2F review is recorded below.

Current source verification receipts remain ignored under
`target/kanto-resume/receipts/`:

- `kanto-source.log`: all90 scoped `--lib kanto_` tests passed with the actual
  pack and selected-asset root, including the34-cell café counter regression.
- `saffron-wall.log`: the corrected Saffron wall regression passed. Its fixture
  now includes the complete block09 and rejects edits across all16 cells,
  retaining closed-volume, live UV, footing and single-cornice assertions.
- `dungeon-source.log`: all41 focused dungeon tests passed, including both
  corrected cave-bank regressions and unchanged cave/shelf/ladder checks.
- `renderer-suite.log`: the full suite completed with1,432 passed,22 failed and
 74 ignored. It is not green. Exterior/battle failures were reported to their
  owners; this task owns the three dungeon failures.

The focused dungeon rerun passed. The enclosing-bank test now supplies
native ceiling-rock block2e/tile26 instead of walking-floor tile16, retaining
relief, cached footing, floor substrate and six-pixel shelf checks. The tiered
terrain test counts the rock domain separately (two caps and two exposed east
faces), so floor tessellation changes cannot hide a real extra rock face; it
also requires the lower-floor substrate explicitly. Both corrected cave
assertions passed without changing production geometry or source ownership.

The initial Kanto command aborted before assertions because the concurrent
Wooper migration added a GLB reference after model metadata generation. The
normal queued Cargo build regenerated that metadata; no fallback or model
rollback was used. The later results above supersede that aborted attempt.

The native `render_at_location`/`audit_authored_assets` build completed. The actual
pack audit covered all120 maps and16 script variants with zero mesher errors;
this establishes ownership/topology, not visual acceptance. Receipts are
`native-build.log` and `native-audit.log`; the temporary audit JSON was removed.

The first production paired captures are in ignored
`target/kanto-resume/first-native-review/`. CeladonCafe, RedsHouse1F and
RedsHouse2F each report `state_preserved=true`, including actor state, position,
clock and authoritative checksum. The café counter cap/trim and gray bin read
correctly. Remaining defects include the table's unclaimed northern art strip,
four front stools, a wardrobe-shaped drawer cupboard, pink bedroom bedding,
and a mostly flat bedroom desk. Most visibly, the console and seats fill too
much of their original source plots relative to the TV.

User requested physical scale correction explicitly. Current changes separate
source claims from prop bounds: console6px wide/deep and3px high, seats8px
wide/deep, stools7px high, chairs11px high. Two-row drawings remain centered;
footing, collision and complete source ownership stay unchanged. A geometry
regression measures the actual selected models, including the bedroom's
cabinet-free CRT, at three grid scales. Seats are narrower and have smaller
overall volume than the TV; the console is smaller on every axis than the
standalone CRT. The fresh `--lib kanto_` run passed all93 tests, including this
regression, cupboard/console material guards, bed geometry/material guard,
café front stools and complete table ownership. `scale-tests.log` records it.

The production rebuild passed (`scale-build.log`). Fresh paired captures in
ignored `target/kanto-resume/scale-review/` covered CeladonCafe, RedsHouse1F,
RedsHouse2F, PlayersHouse1F, PlayersHouse2F, SaffronMagnetTrainStation,
Route5SaffronGate and ViridianPokecenter1F at12:00. All eight paired receipts
report `state_preserved=true`. Inspected the actual2560x1600 renders: Red's
console is now clearly smaller than the TV, house stools are compact beside
their tables/TVs, station chairs are proportionate to the trainer, and center
seats retain their own small footprints. Red's cupboard now reads as drawers;
bed linen retains the source cream/gray palette rather than pink. Café tables
have no dangling northern art strip and all four counter stools are modeled.
Capture logs are `scale-capture.log` and `chair-capture.log`.

The bedroom desk now claims its complete four-by-three source drawing across
blocks20/12, including feet, and reuses the selected dining-table geometry as a
32x16px top at10px height. Its own gold source pigment paints both table material
roles. Full-source/footing/negative controls passed in the94-test desk run;
fresh native visual acceptance remains pending.

User then requested a larger player and called out major station quality issues.
The presentation scale is now1.25 only for `VisualActorId::Player`, preserving
the floor-centered authoritative foot and feeding the same scale to the gait.
NPCs sharing the trainer sprite are not resized. All18 active actor regressions
passed, with4 existing supplied-rig tests ignored (`player-tests.log`). The new
test exercises actual spawned roots, unchanged feet and the source frame.

The station's old planter matcher covered only block0c, missing its actual20,
2f and33 variants. Complete c/e/20/2f/33 drawings now reuse the existing long
planter model with seven named material roles and live native foliage/pot/soil
swatches; missing roles retain 2D. Raised gray/blue rear walls and physical
railings replace their complete known source drawings. Saffron tile3d now reads
as blue ceramic with source-colored grout, without its repeated enlarged glyph.
All97 scoped Kanto tests passed (`station-tests.log`), including whole drawings,
altered-art rejection, canonical plant geometry/material guards, wall/rail
claims and unchanged footing. A production rebuild and station recapture are
still pending; do not describe these changes as visually accepted yet.

The first station review (`station-first-review/`) exposed a reversed front
wall, false floor behind that wall, blue foliage instead of flowers, and the
unmodeled half-rail in block08. These were repaired. The wall regression now
requires its complete positive-Z front area and no floor behind the wall; both
full and half rails retain all source claims and footing. Green foliage comes
from tile51; blue blossoms reuse only the named petal triangles of the existing
flower-stand model, with its stand/pot excluded. Missing plant or blossom roles
retain 2D. Fresh paired geometry captures of both stations in
`station-geometry-review/` report `state_preserved=true` and were inspected:
closed rear walls, complete railings, rectangular containers and distinct blue
blooms now read correctly. The desk is also visually complete in
`station-first-review/RedsHouse2F-2.5d.png`; player scale is visibly larger.

The train and chairs had incorrect generic paint in those geometry captures.
The current train uses named casing/roof/steel/ink/glass/accent roles over its
unchanged canonical geometry, retaining the live door, rail and warp cells.
Source pigments were checked directly in the ignored pack: tile0a cream,
tile1a gray/dark glass, tile27 ink, tile28 orange at native pixel(80,22).
Required rounded chairs retain the selected geometry with orange vertical
surfaces and yellow upward caps from tile40. Missing roles retain 2D.
`current-source-tests.log` passes all100 Kanto checks including train placement
and palette guards. Native final paint review remains pending.

One current-source build/test attempt failed on the battle task's transient
missing `location_details` implementation; that function is now present and the
scoped retry passed. Preserve the other task's battle files. Pack SHA256 remains
`e3e9998e7002a4511222a9b362fc5b8e6a9479572cc9f9977616ad42faf55678`.

Additional visible review defects include
busy center/station flooring, flat station planters, and a source-art remnant at
the center's southwest edge. These eight reviewed maps do not close the120-map
goal or prove script reachability.

No destination-host full visual acceptance is established yet. Outstanding
scope remains café paint/floor review, house finishes, retail rooms, gates,
gyms, caves, all120 maps and script variants. Native comparisons must use the
same position, time, facing and game state; source tests and family counts do
not establish visual completion.

## Runtime inputs

Keep the user-supplied Crystal content pack external and ignored. This checkout
uses `content-packs/core-modular.browser.crystalpack` for interior verification.
Transfer that file separately; do not add it to Git or rebuild it from an
incompatible export workspace. Selected furnishing assets must be present under
`assets/open-models`; missing selected assets retain their source 2D drawing.
Native verification sets `CRYSTAL_OPEN_MODEL_ROOT` to the absolute path of that
directory. Embedded authored counter and bin meshes are canonical assets, not
alternate fallbacks.

## Shared geometry

`src/shared_model_geometry.rs` in crystal-voxel-view shares identical positions,
normals, and indices through an Arc cache. Hashes only locate candidates; exact
geometry equality authorizes reuse. UVs, paint, cutaway ranges, and source
material ownership remain separate. Regressions verify shared geometry with
independent palettes, including the café/gym bin.

## Current visual work

The café now has complete shared display cabinets, a connected source-gold wall,
and a compact ribbed bin using the canonical gym bin geometry with native gray
swatches. The connected café service counter reuses the existing counter modules
at its original eight-pixel height and preserves the service opening. Its source
palette distinguishes the green upward cap from gold/brown vertical trim.

The floor remains visually busy. House cabinet/console finishes, retail rooms,
gates, gyms, and caves still need review and improvement across the full scope.

## Shared-file commit boundary

The source worktree was tested as a whole. Multiple tasks have edits in the same
renderer files; this task cannot safely commit those files wholesale on their
behalf. The coordinating commit must retain the verified current changes in:

- `crates/crystal-voxel-view/src/interior_models.rs`
- `crates/crystal-voxel-view/src/dungeon_models.rs`
- `crates/crystal-voxel-view/src/mesh/modeled_interiors.rs`
- `crates/crystal-voxel-view/src/mesh/interior_surfaces.rs`
- `crates/crystal-voxel-view/src/mesh/gate_counters.rs`
- `crates/crystal-voxel-view/src/mesh/gate_counter_bindings.rs`
- Shared renderer integration and the no-alternate-fallback asset deletions
- The appended review receipts in `docs/3d-remaining-families.md`

Do not transfer target directories as required runtime assets. Build logs,
verification PNGs, temporary test executables, and source probes are not shipped
or committed. Use the coordinating task's cold-start script rather than creating
another launcher.

## Original goal and checkpoint

Original goal: **kanto kanto interiors need work**. Preserve the entire actual
KANTO non-TOWN/ROUTE scope: 120 maps (100 INDOOR, 13 GATE, 7 CAVE), including
script block variants. Do not declare completion from this café checkpoint.

Last old-host action: corrected the café counter worktop material selection after native
review found green reaching vertical trim. The selection derives the upward cap
height from named cap faces rather than the slightly higher grain primitive.
Stop further visual edits for the requested host transfer.

## Verified checks and remaining verification

- Source regressions passed in `target/kanto-cafe-counter-cap-source-test-receipt.log`:
  11 connected-network tests, 43 Kanto tests, 23 interior-model tests, 31 placement
  tests, and the existing shared-geometry, missing-asset, water and lighting checks.
  These source tests used freshly compiled Rust code with cached dependency rlibs.
- All 16 counter modules preserve geometry and keep green off vertical trim.
- The actual external pack café regression claims exactly 34 counter cells,
  preserves the service opening, rejects edited source art, and leaves footing alone.
- Production build20381 passed (5m45s); native capture44107 passed and was inspected
  at original2560×1600 resolution. It exposed incorrect green trim, subsequently fixed.
- An initial cap-height test failed because grain stood0.0004 above the cap.
  Corrected source rerun12815 passed. Do not describe the initial capture as proof
  of the final paint.
- Final production build41346 passed in16m11s. The final native paint review
  was not run before the requested stop and remains pending on the new host.
- Previous whole-scope production audits covered all120 maps and16 declared script
  variants with zero mesher errors. They establish geometry integrity, not visual
  approval. Old-host ignored logs/captures are evidence, not new-host dependencies.

## Restart on the new host

Use `scripts/cold-start-3d.sh` and `docs/cold-start-3d.md` from the coordinating task
for Git, LFS and content-pack setup. Before resuming, ensure the final coordinated
commit includes all shared integration files listed above; this task's isolated
checkpoint alone is not a complete rendering implementation.

From the Geothite repository root, with the actual ignored pack present:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/core-modular.browser.crystalpack" \
  cargo test --locked -p crystal-voxel-view kanto_native
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/core-modular.browser.crystalpack" \
  cargo test --locked -p crystal-voxel-view mesh::gate_counters::tests
cargo test --locked -p crystal-voxel-view shared_assets
cargo test --locked -p crystal-voxel-view missing_selected_interior
cargo test --locked -p crystal-voxel-view interior_models::tests
CARGO_TARGET_DIR=target/kanto-resume cargo build --locked -p crystal-bevy --profile wasm-dev --all-features \
  --example render_at_location --example audit_authored_assets -j 2
CRYSTAL_PREVIEW_MUTE=1 \
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=240 \
  target/kanto-resume/wasm-dev/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --maps CeladonCafe --view both --output-dir target/kanto-cafe-resume-review
```

These normal Cargo commands are the portable checks. The old-host cache-specific
rustc workaround and `target-bevy20-verify` rlib hashes must not be copied into a
new-host workflow. If source tests skip because the external pack is missing,
restore and verify the real pack before claiming that packed-room checks passed.
Never use a placeholder path or a ROM/disassembly export as a substitute.

Inspect the original native PNG and2D comparison before accepting paint. Next
improvements: café floor texture; Red's house cupboard/console palettes; Kanto
retail floors/counters; gate finishes; gym furnishings/walls and missing plants;
cave readability. Reuse canonical assets with independent local palettes and
physical sizing. Missing or unknown roles retain2D; do not introduce alternate3D
fallbacks. Keep collision, authored script ownership, and actor datum unchanged.

No audio or controller changes are part of this checkpoint. Keep all game content
packs, renders, compiler output, source probes and audit JSON ignored and external
where required. Temporary test binaries and native hardlink snapshots were removed
by their verification wrappers.

To repeat the complete geometry audit after visual work, with the same external
pack and selected-asset root:

```sh
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  target/kanto-resume/wasm-dev/examples/audit_authored_assets \
  content-packs/core-modular.browser.crystalpack \
  target/kanto-resume-audit.json --kanto-interiors --script-block-variants
```

Inspect the report, then remove the temporary JSON. Do not commit or transfer an
audit manifest as game content; it is only a transient verification artifact.

## Transfer checkpoint verification status

The final native build completed successfully after the initial checkpoint:
old-host session41346,16m11s, receipt
`target/kanto-cafe-counter-cap-production-build.log`. This was a locked,
all-features `crystal-bevy` build of `render_at_location` and
`audit_authored_assets` using profile `wasm-dev` and two build jobs. Warnings were
reported; the command exited successfully. The corrected final native capture
was not run. Resume with the portable build and capture commands above and
inspect the image. Source material regressions passed; visual verification of
the final paint remains pending.

Source checkpoint commit: `0bde6a14ee8d506079cd08ec0bbb0b1816f6444c`.
It contains exactly the shared-geometry module, its isolated lib registration,
the two connected-counter files, and the initial handoff note. The handoff
completion commit changes only this note. The cold-start script and transfer
runbook were committed by the coordinating task in
`aea28005f0f014311642311e3150d18c11c244e9`.

At handoff completion, these shared integrations still had uncommitted changes:
`interior_models.rs`, `dungeon_models.rs`, `lib.rs`,
`mesh/modeled_interiors.rs`, `mesh/interior_surfaces.rs`, and
`docs/3d-remaining-families.md`. The coordinating task must commit these and the
remaining asset integration before calling the transfer consistent. Do not
interpret the isolated source checkpoint as a standalone cold-start guarantee.

All source probes and temporary verification executables from this task were
removed. No further visual edits or capture processes were started after the
requested stop. The goal was paused and unfinished at transfer. It is now
active; the destination-host continuation above records current work.

The checkpoint parent was `aea28005f0f014311642311e3150d18c11c244e9`. The checkpoint above includes the shared-geometry module and its isolated lib registration,
the owned connected-counter files, and this note. Remaining mixed-owner renderer
integrations and review receipts are deliberately left for the coordinating
commit. The isolated checkpoint alone is not a standalone cold-start guarantee.

The successful fresh-source regression runner used these exact test filters
against the compiled crystal-voxel-view test binary; its temporary binaries were
removed after completion:

```text
mesh::gate_counters::tests
kanto_native
cave_water::tests
floor_volume_stays_below_unchanged_actor_datum
cavern_lights_dim_real_energy_and_restore_when_leaving
waterfall_uses_each_source_row_once_on_one_bounded_cave_slope
shared_assets
missing_selected_interior
open_models::tests
interior_models::tests
native_bookcase_tests
mesh::modeled_interiors::tests
```

Each can be rerun with `cargo test --locked -p crystal-voxel-view FILTER`, with
`CRYSTAL_RENDER_TEST_PACK` and `CRYSTAL_OPEN_MODEL_ROOT` set to the actual pack
and selected-asset directory as described above. These checks do not replace
native visual review across the entire120-map scope.


The latest station review rejected the train silhouette: all-height door cutouts
made three disconnected blocks. The canonical train authoring source now uses
a 32-sided tubular cross-section, tapered cab ends, a continuous crown above
two clear boarding portals, and four open plug-door leaves parked sideways on
slim tracks. Source claims, live door cells, warps and footing remain unchanged.
The Python geometry checker passes (242 closed parts, 4,544 triangles); fresh
production paired captures are now inspected in both stations at noon, with
`state_preserved=true` receipts. The first tube capture exposed floating flat
door leaves; the corrected leaves follow the shell curvature and no longer
stand away from it. The fresh final Kanto-native run passed 56 tests, and the same rebuilt
test executable passed all five train adapter tests, including the continuous
roof/clear-portals check. Earlier unrelated battle-camera and cave-palette
test compilation errors were fixed by their owning goals. This is
an authored open-door pose, not a timed door animation.


The user identified Goldenrod's blue east-wall fixtures as ticket boxes. Their
complete 2x8 source bank spans blocks 2c/2d. The new whole-drawing matcher
requires both blocks and every tile/phase; four upright cabinets face west
into the waiting room, with live blue casing, screens, controls and ticket
hatches. They reuse the canonical cabinet geometry while excluding beverage
parts. Unknown materials retain the original 2D drawing. All 16 source cells
retain their existing footing and collision. The fresh Kanto-native run passed 56 tests, including the four-ticket-box
placement/material/fallback checks. Native paired captures in `target/kanto-resume/station-ticket-review/` are
now inspected for both stations at noon, with `state_preserved=true`. Goldenrod
shows four upright blue boxes with visible controls and ticket hatches; Saffron
does not acquire that bank. The final native build passed (18m32s). The broader
120-map visual review, Saffron seat-color parity and timed door animation were
open at this historical checkpoint. The current continuation records the
completed source-scene reconstruction and later seat/review verification.
