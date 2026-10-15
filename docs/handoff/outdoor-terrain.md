# Outdoor terrain transfer checkpoint — 2026-10-12

## Buoy cleanup verified in production — 2026-10-14

Final native build completed successfully in8m17
(`target/outdoor-buoy-solid-bank-native-build.log`). The earlier textured-bank
cleanup removed blue cards but left solid brown bank seams. The final cleanup
also retires only the solid triangles owned by successfully converted raised
buoy cells. Unrelated solid indices and cutaway vertex ranges remain intact.
All6 buoy tests pass (`target/outdoor-buoy-solid-bank-tests.log`).

Fresh Route41 close `(22,13)`, wide `(23,22)`, and Cianwood southeast `(27,51)`
source/native pairs are personally inspected; all three receipts confirm
`state_preserved=true`. Blue source cards and thin legacy brown walls are gone
in these reviewed views; rounded low floats and the native whirlpool remain.
Receipts: `target/outdoor-buoy-solid-final-{close,wide}*` and
`target/outdoor-buoy-solid-final-cianwood*`. The source audit already confirms
all152 formerly raster buoy cells are modeled, with no remaining tile58
unmodeled cells (`target/outdoor-complete-buoy-route41-audit.json`).
This is scoped geometry acceptance. Ocean water is still too teal relative to
the source blue; Cianwood's coast/caps remain overly angular/broad. The two
distant Route46 ground gaps remain open. Its latest indexed shared-point
trace reports zero splits in all four surfaces, so the earlier unused-vertex
counts must not be treated as evidence of those holes. Route22 perimeter
coverage completed in `target/outdoor-route22-perimeter.log`, inspecting
a different ledge layout and new route edges. Whole-exterior acceptance remains
open; no packs or render artifacts are staged.

## Route22 perimeter and blank path samples — 2026-10-14

All eight new Route22 perimeter source/native pairs are personally inspected;
all eight receipts report preserved state (`target/outdoor-route22-perimeter/`).
They expose pale cell-shaped patches in the eastern path, flat source ink on
some shallow ledge caps/brick courses, and severe trainer occlusion at south
`(20,15)` / southeast `(37,15)`. Huge outer rock plateaus are visually dull;
neither source-preserved state nor the global jump audit accepts composition.

The pale path patch has a concrete classification mismatch: Kanto block01's
14 blank tile23 samples retain source paint while its two speckled tile39
samples receive native Path. The shared ground classifier now treats both
flat atlas samples as Path, alongside existing0d. The new focused regression
checks all16 source phases, removal of the residual raster faces, unchanged
ground corners/footing/claims, and rejection of unknown atlases / other art.
The focused path test passes (`target/outdoor-kanto-quiet-path-tests.log`);
fresh native proof at Route22 east is now inspected as recorded below.

The Route22 night baseline `(37,9)`, hour22/orbit2/zoom0 is also inspected
against2D, with preserved state (`target/outdoor-route22-path-before-night*`).
It confirms the same blank/speckled split, exaggerated into bright blue patches
at night. Foreground cliff occlusion is now traced to the native mineral shader:
negative-UV exterior rock never enters the cave-only body reveal, and exterior
material clones do not receive the supported player capsule. An experimental
capsule/aperture extension was tried and visually rejected as described below.
All7 focused Kanto ground tests pass
(`target/outdoor-route22-kanto-recheck.log`). The exterior reveal experiment
also passed its capsule regression, but visual review rejects it: its body
aperture exposes sky beneath the rock, and clipping at the projected foot line
clips the boots. A bounded lower margin reveals boots but retains an unnatural
sky patch. All experimental exterior reveal/capsule source changes are removed;
keep the existing cave behavior. Diagnostic receipts are
`target/outdoor-route22-aperture-{shader,foot,margin}-review*`. These are rejected
shader overrides, not production acceptance. Build36054 completed in5m25 but
contains the rejected candidate; do not use it for final visual claims.
Fresh normal production build completed in4m44
(`target/outdoor-route22-quiet-path-native-build.log`), including the path fix
and restored original cutaway behavior. Final Route22 east `(37,9)` day,
morning7 and night22 pairs are personally inspected against2D. All three
receipts preserve state (`target/outdoor-route22-quiet-path-final-*`). The
blank/speckled patchwork is gone in all three: the path is continuous, with
blended lawn edges and warm morning / cool night paint. This scoped ground
repair is verified. Night brick courses remain excessively royal blue, some
shallow caps retain flat ink, and foreground cliffs still hide the trainer
under the orbit2 view. South/southeast occlusion, two Route46 ground gaps and
overall exterior finish remain unresolved. No exterior reveal experiment is
included in the final source or binary.


## Sculpted cap/wall edge correction — 2026-10-14

A second focused repro proves an additional tear between source grid corners.
After a shared crest erodes by0.4, cap and wall copies of `(44,16,8)` move to
21.60 versus21.36. Receipt: `outdoor-mineral-mid-edge-repro-tests.log`.
The common legacy-floor mapping now covers the full authenticated tall-rock /
cardinal jump-bank contact, using a cached two- or four-cell source interval.
Both faces receive the same transformation; interior cap points and unrelated
rock contacts retain their existing rules. All15 terrace tests pass, including
both corner and mid-edge repros (`outdoor-mineral-contact-edge-field-tests.log`).
All21 mineral tests pass (one ignored), receipt
`outdoor-mineral-contact-edge-stone-tests.log`. Native build34529 completed successfully in13m58
(`outdoor-mineral-contact-edge-native-build.log`). Fresh Route46 east and
Blackthorn south pairs are inspected and state-preserved. The large cap/wall
slits are closed in those reviewed views. Route46 exact clear-color pixels
fall6433 to1609; two distant ground gaps remain, so whole-terrain closure is
still unproven. Blackthorn's Mart/Pokecenter roof faces remain black/missing,
which is a separate known outside-art defect. Receipts:
`outdoor-mineral-contact-edge-{route46,blackthorn-south}*`.
The new audit completed successfully:72 maps,2457 authored jumps, zero
mesher errors (`outdoor-mineral-contact-edge-jump-audit.{json,log}`).

## Measured grove underlay fix and ocean whirlpools — 2026-10-14

The actual Route46 production trace proves the bright forest strip's underlay
problem:961 grove plots all borrow Johto `$57` tile5 from `(68,1)`, whose bank
surface is24 world units high at32-unit source cells. Copying that sample's
height raises the lawn above the tree base and prevents its native paint pass.
The trace is `outdoor-mineral-contact-edge-route46-trace.json`; no interior
tree glyphs are unclaimed. Grove sampling now prefers level lawn from the
same atlas, and new grove underlays always use the plot's own base height.
The source/bank sample supplies paint only. The focused regression passes
(`outdoor-grove-ground-datum-tests.log`), including bank-only fallback and
foreign-atlas rejection; footing is unchanged. Water12, mixed-tree2 and
terrace15 rechecks pass under `outdoor-grove-ocean-*-recheck.log`.

New Route41 `(23,22)` source/native day views are inspected and state-preserved
(`outdoor-new-route41-swimmers-day*`). The female swimmer is mostly submerged.
Flat whirlpool rectangles and white placeholder buoys remain visible. The
exterior now reuses the existing guarded Den funnel/current producer for the
same complete Johto `$07` drawing, under `johto/whirlpool`; collision, source
cells and standing support are unchanged. All12 water tests pass, including
exterior phase refresh and malformed-art rejection, with original Den tests
retained (`outdoor-ocean-whirlpool-water-tests.log`).

The apparent male-sprite concern was an incorrect inference, corrected after
pack-owner review. Route41 has no callback; its male slots use
`SPRITE_OLIVINE_RIVAL`, initially `SPRITE_RIVAL`. OlivineCity's RivalSceneTop /
RivalSceneBottom set it to `SPRITE_SWIMMER_GUY` later. Fresh direct-map rivals
are faithful to that game state, not an export defect. No label or pack override
was added. A post-Olivine state is required for additional male Route41 visual
coverage; prior male Route21 and swimmer depth checks remain valid.

Combined native build31008 succeeded in10m23
(`outdoor-grove-ocean-native-build.log`). The fresh Route46 `(17,18)` pair is
inspected and state-preserved (`outdoor-grove-datum-route46*`): the bright
forest raster strip is gone, and all961 plots now select Johto `$02` tile5
at `(92,5)`, height0. The two distant ground gaps remain (1623 clear pixels).
Do not call this whole-terrain acceptance.

The Route41 ocean and closer `(22,13)` source/native pairs are inspected and
state-preserved. Native funnels/current paint replace the whirlpool quadrants,
but neighboring buoy glyphs remain raster. The source audit finds152 unmodeled
buoy cells:32 in Johto `$07`,120 in `$3a`. The legacy adapter covered only
`$30-$39`. `$3a` drawings are byte-identical in Johto and Modern; both verified
corner phases now qualify. `$07` is Johto-only, right-hand buoy quadrants.
The full family test passes6/6 (`outdoor-complete-buoy-family-tests.log`).
The floats now use a lower, broader shoulder with a thin darker submerged rim,
without increasing vertex count. Native build27988 completed in5m02 and the
close-up verifies `$07` float replacement. The still-visible `$3a` cards are
not accepted by that older binary. Build27506 finished successfully in11m17
(`outdoor-complete-buoy-native-build.log`), including `$3a` and the indexed
mineral diagnostic, but predating the old-bank cleanup.

The close-up also exposes thin legacy raster walls beneath converted `$07`
float beds. A source-scoped cleanup now retires those old raised-bank quads
only after an authenticated float replacement succeeds, preserving indices
and remapping unrelated cutaway ranges. The new regression includes a real
old shoreline face plus an unrelated cutaway marker. Cleanup tests pass6/6 in
`outdoor-buoy-bank-cleanup-tests.log` (9m18 compile,0.01s execution). This
includes both atlases' `$3a`, `$07`, source guards, closed consistent winding,
unchanged footing, removal of a real old shoreline face, and correct remapping
of an unrelated cutaway marker. Final cleanup native build exec9842 is running in
`outdoor-buoy-bank-cleanup-native-build.log` with one Cargo job. After terminal
success, inspect Route41 `(22,13)` orbit0/zoom2 and `(23,22)` orbit0/zoom1 against
2D, audit all152 formerly raster buoy cells, and check Cianwood southeast.
Do not accept the full course from the older partial binaries.

The existing close-up source/native pair is inspected and state-preserved:
`outdoor-ocean-whirlpool-close-native*`. Its trace reports12 native whirlpool
cells for three complete `$07` blocks in the halo; the fourth block is outside
that frame. The source art/clock guards are working, and no donor-animation
fallback or source predicate relaxation is needed. Visible leftovers are buoy
raster, including the known `$3a` family and old vertical bank faces.

The new optional mineral shared-point trace must filter indexed vertices;
its initial1074 count included retired/unreferenced geometry and is not a
hole count. The filter is now added. Future Route46 traces should inspect
`mineral.terrace_shared_points` with its map/geometry origin before altering
the two residual ground gaps. No gameplay height changes were made for that
read-only diagnostic. Whirlpool source trace confirms12 native cells for the
three complete whirlpools inside the close-up halo; the fourth source block
is outside that frame. No water-animation guard was weakened.

Canonical packs were newly published with159 authored scene constants and an
additive authored Surf closure; the publisher verified preserved old records,
all377 audio checks and actual gameplay gates. Only canonical packs are used,
never pending packs. The temporary source-edit hold was released. The human
also instructed all goals to stop messaging other chats; no more coordination
messages should be sent without a new explicit human request. Subsequent
builds use CARGO_BUILD_JOBS=1 to avoid overloading the shared host.

## Border background source identity candidate — 2026-10-14

The production border/halo scan retains actual metatile and subtile phases.
The ground apron was instead selecting from cells after legacy/live object
masking: masked tree artwork keeps its atlas pixels while becoming Flat with
TileMAX, so it could win the dominant-ground count. The scene now passes
original_cells/source_shapes to the background selector, and the selector
rejects TileMAX artwork. This does not alter tree placements, collision,
standing support or source images.

All five background tests pass, including a new case where1599 masked tree
cells must lose to one real lawn sample (and all-masked input yields no apron).
Receipt: `target/outdoor-background-source-identity-tests.log`. The native
build succeeded in24m02 (`outdoor-background-source-identity-native-build.log`).
Fresh Route46 and Route45 pairs are inspected; the Route46 strip remains. This
fix protects source identity but does not resolve the visible border defect.

The tree-card fallback now preserves cells already claimed by native scenery.
Its focused ownership regression passes and the native build succeeded in7m14
(`outdoor-tree-fallback-{ownership-tests,native-build}.log`). Fresh Route46
`outdoor-tree-fallback-route46` is inspected: the strip and rock gaps remain,
and its loading trace contains zero `tree.source_card` events. The fallback
was not invoked, so it is not the cause in that view. A bounded opt-in
`tree.source_ownership` trace will inspect actual production halo glyphs next.

## Shared mineral corner repro and new city coverage — 2026-10-14

A focused two-wall corner reproduces a real height tear: identical authored
world point `(48,8,8)` moves to9.2 on the old six-pixel bank side and14.0 on
its zero-floor neighbor. Receipt: `outdoor-mineral-shared-corner-repro-tests.log`.
The current candidate gives adjoining mineral corner vertices one legacy-floor
removal mapping, selected only at an authenticated tall-rock/cardinal jump-bank
contact. Authoritative standing height, collision and ledge direction remain
unchanged. All14 terrace tests pass, including the new corner regression.
The same test executable passes21 mineral tests (one ignored). Receipts:
`outdoor-mineral-shared-corner-{field-tests,stone-tests}.log`. Shared compile
blockers were corrected and coordinated with their owners. Native build22217
finished successfully in16m38 (`outdoor-mineral-shared-corner-native-build.log`).

Fresh Route46 east and Blackthorn south source/native pairs are inspected and
state-preserved (`outdoor-mineral-corner-route46*`,
`outdoor-mineral-corner-blackthorn-south*`). Route46 gaps persist. Identical
clear-color `(184,212,227)` pixel count drops7642 to6433 versus the prior same
pose, but remaining sky leaks are obvious; this is a partial corner repair,
not sealed-terrain acceptance. Blackthorn still has gaps and newly captured
black missing roof faces on Mart/Pokecenter; the outside-art owner received
that rejection fixture. The current global jump audit passes72 maps and2457
actual authored jumps with zero mesher errors:
`outdoor-mineral-shared-corner-jump-audit.{json,log}`.

Route46's production ownership trace contains114 unclaimed tree glyphs; its
first24 are clipped bottom-row tree art at local row0 on the outer halo.
These samples alone do not locate the visible eastern strip. The trace now
also groups by row and samples interior cells. Follow-up native build32871 completed successfully in10m58
(`outdoor-mixed-corner-tree-native-build.log`). The fresh inspected,
state-preserved `outdoor-tree-interior-route46` pair retains the bright strip.
Its trace now shows106 unclaimed glyphs only on outer rows0(76)/113(30), with
**zero interior unclaimed glyphs**. Do not blame missing interior tree cards.
The next bounded trace also records the selected native grove-ground samples,
source identities and heights, to inspect the remaining underlay directly.

All eight Blackthorn perimeter pairs are inspected and state-preserved under
`target/outdoor-new-blackthorn-perimeters/`. North has a large purple Gym roof
occluding the route; east hides the player behind a nearby cliff. South and
southwest retain sky gaps at cliff contacts and wrong roof colors. Rock masses
are broad, rounded slabs with sharp grass wedges rather than distinct source
pillars. These are new rejection fixtures, not visual acceptance.


## Route44 missing mirrored crowns — 2026-10-14

A new Route44 source audit finds2428/4320 modeled cells and868 native tree
cells, but one complete four-cell `$3d` crown remains raster. External canonical
Johto drawings confirm `$3c/$3d/$3e/$3f` are the four lawn-corner variants of
`1e/1f` over `3e/3f`. Added the missing `$3d` `(2,0)` and `$3f` `(2,2)` bindings,
retaining complete art/phase validation and half-height shared grove recipes.
The four-variant binding test passes (`outdoor-mixed-tree-corner-tests.log`).
The audit is `outdoor-route44-tree-source-audit.{json,log}`. The refreshed native source audit confirms2432/4320 modeled cells and872
native tree cells (+4), with zero unmodeled `$3d` crown glyphs. Production
Route44 `(14,12)` day/morning/night pairs are inspected and state-preserved:
`outdoor-mixed-corner-route44[-morning|-night]-compare.png` and their receipts.
The native crown replacement is verified; whole-scene acceptance remains open.
Night water remains brighter than source navy/purple, and the angular coast
and broad brown slab contours remain rejected. The
outside-art owner was notified; pine geometry is unchanged.

## Native small-tree strip and source paint verified — 2026-10-14

The user's remaining flat-tree strip is fixed in Azalea. Exact Johto Modern
`$2f` top-half and `$3b` bottom-half `1e/1f` over `3e/3f` drawings now enter the
complete mixed-tree/shared-grove pipeline. Four `$3b` crowns (16 source cells)
no longer remain as raster cards. Mixed small crowns use half-height grove
recipes within their unchanged plots; ground/standing support and source
collision remain unchanged. Altered art, wrong phase and cropped crowns refuse
replacement. The focused modern fixture passes. Native build succeeded in
11m57s. Final Azalea `(33,9)`, orbit2/zoom1 source/native pairs at hours12/7/22
are inspected and state-preserved; the former left-side pixel-tree strip is
visibly gone. Receipts use `target/outdoor-modern-small-tree-azalea-*` and
`outdoor-modern-small-tree-{tests,native-build}.log`. The current source audit
has1152 native tree cells versus1136 before, and **zero unmodeled `$3b` crown
cells** (1670/2880 modeled total).

The well now selects shared `SOURCE_PAINT_UV`: live paint already contains
night color, so a second moon tint made it cobalt. Corrected final native night
stone and stairs now match source purple; morning remains warm yellow. All
three latest Azalea pairs are inspected. Seven selected well/stair tests pass,
including palette invalidation, and all six terrain-tracking tests pass. A
stone/stair image-handle change rebuilds well paint; lawn animation retains
atlas-only refresh. Receipts: `outdoor-well-source-paint-tests.log`,
`outdoor-source-paint-tracking-tests.log`, native-build log and night pair.
Older well/night rejection notes below are historical; global lighting and
other material coverage remain open.

The broader new_bark suite finished66passed/3failed/2ignored. Ordinary tall-tree
crown margin, ground-detail datum and runtime building-pixel expectation failed;
exact terminal assertions were sent to the active outside-art owner. The tall
johto05 fixture does not take this mixed small-tree branch. Do not claim a full
suite pass or relax these failures. Receipt: `outdoor-small-tree-new-bark-tests.log`.

New geographic coverage: seven unique Cianwood perimeter pairs are inspected
and state-preserved, under `target/outdoor-new-cianwood-perimeters/` and its
batch log. The southwest/south views expose wrong roof hues and roof artifacts;
east/southeast expose unfinished buoy forms, angular coast edges and a residual
flat blue water patch. Gray coastal boulders differ from source warm stone.
These are rejection fixtures, not city acceptance.

The fresh Route46 `(17,18)` pair is inspected and state-preserved. Substantial
sky slits persist despite the world-aligned horizontal rings, and a pixel-tree
strip remains beneath the eastern forest (`outdoor-modern-small-tree-route46-east`).
The active-map source audit has608 native tree cells and no unmodeled crown
glyphs, so next inspect actual production halo/border identities and ownership;
do not infer a canonical source type from the active-map audit alone. Audit's
receipt is `outdoor-route46-tree-source-audit.{json,log}`. Rock continuity and
remaining border-tree cards stay open; the broad outdoor goal remains active.

## Slowpoke Well and mineral continuity checkpoint — 2026-10-14

**Slowpoke Well exterior artwork is now replaced by native Rust geometry.**
`mesh/slowpoke_well.rs` authenticates the entire Johto Modern
`$5b/$5c/$5d/$5e` drawing, including blank borders, and claims only its 36 art
cells. Source-sampled stone and stair paint supply the square masonry enclosure,
recessed course joints, pale south steps and open recessed shaft. Steps occupy
source rows4/5; rows2/3 remain the aperture. The pit bed overlaps shaft edges.
Collision, warp coordinates and existing standing support remain unchanged.
The scene binding is a small append call before the generic prop pass.

Final verification: **3 well tests pass**, including a visible-stone/backing
occlusion regression and source aperture check; the current compiled terrace
suite passes13 tests and mineral suite passes21 (one ignored). The final native
build succeeded in5m57s. Production source audit confirms all36 well art cells
are native. Matching-state Azalea `(33,9)`, orbit2/zoom1 pairs at hour12,7,22
are captured and inspected; all three have `state_preserved=true`. Receipts:
`target/outdoor-slowpoke-well-final-shaft-tests.log`,
`outdoor-slowpoke-well-final-shaft-native-build.log`,
`outdoor-slowpoke-well-final-source-audit.{json,log}` and
`outdoor-slowpoke-well-final-{day,morning,night}{,-2d}.png` with paired-state logs.
The geometry replacement and source footprint are verified. **Night material
acceptance remains open:** the source-painted well/stairs become excessively
cobalt under the existing night grading, versus the source purple/brown stone.
Morning warms the live stair palette correctly. Do not claim whole-scene
fidelity or finished time-of-day grading.

Earlier circular, occluded-course and overlong-stair candidates were rejected
through native/source inspection and corrected. Interrupted or failed test
attempts are not passing evidence; use the final-shaft test receipt. Loading
trace `outdoor-slowpoke-well-square-day-loading-trace.json` identifies
prepare_render_meshes57.77s/build_source33.36s/finish_terrain29.99s as the largest
stages in an earlier122s capture; timings were shared with the performance goal.
Final day/morning/night captures improved to45s/42s/38s. The well's underground
floors remain with the existing expanded interior goal.

The previous rock shoulder build succeeded, but its fresh Route26 `(10,28)`
hour12/orbit2 pair still shows tiny sky-blue slits
(`outdoor-route26-shoulder-contact{,-2d}.png`). The780-test mesh result and
72-map/2,457-jump zero-error audit do not establish mineral continuity. Optional
`audit_authored_assets --mineral-seams x0,y0,x1,y1` reports bounded unmatched
geometric edges, including ground contacts/T junctions, so counts are not hole
verdicts. At source `(16,44)`, adjacent faces have different post-terrace height
rings, consistent with legacy floor6 versus floor0 remapping. Shared-corner
remapping remains an unverified hypothesis requiring a regression and native
inspection. World-aligned horizontal rings now match merged wall borders to
individual source-cell caps; the current21-pass mineral suite covers this, but
its native hole-fix proof remains pending.

All eight new Route46 perimeter pairs and seven successful Azalea perimeter
pairs are inspected and state-preserved. Route46 east/northeast/southeast/west
expose substantial pale sky openings at rock contacts; west also exposes raster
grass cards. South/southwest gate roofs remain flat. Azalea north resolves to
`(20,6)`; small rock stacks remain blocky, some trees poorly covered, service
roof palettes unfaithful, and the southwest gate flat. East/southeast exposed
the well drawing repaired above. Azalea west timed out after300s building
terrain; do not rerun the seven successful baseline pairs as new coverage.
These are rejection fixtures, not broad visual acceptance. Logs/images:
`target/outdoor-new-city-rock-perimeters/`,
`target/outdoor-new-route46-rock-perimeters/` and their batch logs.

## Directed ledge reconfirmation — 2026-10-14

The user's east/west clarification uses the same global invariant as south:
the permitted jump starts on higher ground and lands on lower ground; the
blocked reverse approach is the lower side. Direction comes from production
collision permissions, not the camera or a map-specific height guess. Terrain
and standing actor footing consume the same constrained elevation field.

Re-ran all 13 terrace regressions successfully and the compiled production
`audit_authored_assets --jump-maps` against the local browser pack: **72 maps,
2,457 actual production jumps, zero terrain/jump errors**. Verification receipts
are ignored `target/outdoor-directed-ledge-confirmation.{json,log}`. Reinspected
the existing Route38 eastward and Route4 westward native proofs. This confirms
the directed-drop rule; it does not accept remaining scenery, lighting, or
map-connection/script-variant visual coverage.

## Outside-art source correction and pine cutaway — 2026-10-14

Forest-gate facade live paint implemented; front production view verified: exact source masonry tile24 and window tile04 are sampled separately within the existing verified gate plot. Purple field/pale trim/yellow window values come from live pixels; no palette catalog extracted. Scoped world-model material pass repaints plaster/stone/timber trim/glass while preserving geometry, footing, claims, doorway, roof and insignia roles. Missing roof/wall/window coverage keeps exact2D rather than guessed paint. Expanded gate regression passes both variants, distinct synthetic palettes and missing-wall/window cases. Seven exterior-model and14 fence regressions also pass. Initial native36965 completed6m21s but its embedded shader snapshot failed pipeline compilation (`masked_cutaway` scope) and left scenery undrawn. Current shared shader already had the correction; no shader edit made here. Review-hook33333 rendered current shader successfully. Fresh default native61993 completed6m29s, and normal/no-hook front capture27302 completed without shader errors. Personally inspected `target/outside-art-gate-facade-final-front{,-2d}.png`, Ilex(1,43),noon,orbit0,zoom0; state_preserved=true,actors3. Purple field/pale trim/yellow glass confirmed. Source positions(6,44..48) are tree plots; use this valid west path. Normal clearing capture92861 completed through the graphics wrapper. Personally inspected `target/outside-art-gate-facade-final-reveal{,-2d}.png`, Ilex(1,37),noon,orbit2,zoom1; state_preserved=true,actors4, no shader errors. Gate still fully reveals the clearing. Other owners changed shared grove/shader behavior during this pass; do not claim all tree detail is identical to older captures. No own build/capture remains live. Failed/superseded diagnostic image scratch removed; final normal front/reveal pairs retained as verification evidence. Actual south building uses closed variant25; source front has three high windows and continuous masonry, while current generic model still has four windows, a central shutter and gold crests. Correct those authored geometry defects next, add source brick relief and refine roof material; facade paint alone is not acceptance. Source native/model geometry unchanged this turn; source brick relief and sign finish still need separate authoring. No whole-scene acceptance.

Forest-gate roof course refinement implemented and production-checked: `tools/build-world-exteriors.py::forest_gate_roof` replaces only old wide course slabs with closed, shallow-crowned, overlapping staggered shingles on both gate variants. Existing roof deck/ridge/eaves, exact bounds/doorway and all non-roof canonical primitives preserved. Hash-guarded native world source update replaced168 course objects with840 shingles, preserving6,499 unrelated object signatures; authoring scratch removed after publication. Canonical triangle counts11,148/open and10,872/closed. Initial runtime test caught generic roof re-tessellation multiplying the authored shingles; scoped gate roof import now preserves original geometry with mild normal-based pigment shading. Final seven exterior tests, live source-paint regression and scoped reveal regression all pass. Native23149 completed4m00s. Both graphics-wrapper captures completed and were personally inspected against matching2D: clear-view25353 `target/outside-art-gate-shingle-ilex-orbit0{,-2d}.png` and normal reveal2645 `target/outside-art-gate-shingle-ilex-reveal{,-2d}.png`, Ilex(1,37),noon,zoom1,orbits0/2. Both receipts state_preserved=true,actors4. Smaller staggered courses replace the broad slabs and reduce continuous black seams; normal reveal still clears the building fully. Roof remains visually flat/uniform, narrow dark seams remain, and brown fascia/facade/source detailing still need correction before building acceptance. No own build/capture remains live; whole outdoor goal remains active.

Ancient forest bole refinement verified: widened the main shaft while reducing buttress amplification, retaining source root centers, height, crown recipes, palette, source fit and all other tree families. Ancient/source-footing/cache tests4/4, streaming bounds1/1, ordinary pine2/2 and mixed-corner replacement1/1 pass. Native50473 completed successfully3m00s. Capture27473 completed through the shared graphics lock; personally inspected `target/outside-art-ancient-bole-ilex{,-2d}.png` at Ilex(1,37),noon,orbit2,zoom1 against the quiet-litter baseline and actual2D. Fuller trunks retain grounded contacts/readable clearing; paired receipt state_preserved=true, actors4. Canopy depth, uniform branch/leaf presentation, marble-like floor grain, speckled player-area tree reveal and gate facade/fascia/roof seams remain. This verifies the bounded trunk change, not whole-scene acceptance. No own build/preview remains running.

Forest litter contact refinement verified: main seeded leaves now scale with the existing tree-shadow canopy field, with muted cool litter, canopy-only twigs/ferns and dim sparse minerals. Removed the redundant generic warm-leaf/flower layer from this floor pass; separate source flower anchors remain. Five forest-floor regressions and both modeled-flower regressions pass; native build39367 completed. Personally inspected `target/outside-art-forest-litter-contact-ilex{,-2d}.png` at Ilex(1,37),noon,orbit2,zoom1; paired receipt state_preserved=true. Clearing is quieter and source flowers remain. Whole scene still needs mature trunk mass, surface/material depth and gate finish; no whole-scene acceptance.

Compact-toolbar native34912 finished successfully5m36s. Fresh pair `target/outside-art-compact-camera-toolbar-ilex{,-2d}.png` Ilex(1,37),noon,orbit2,zoom1 inspected against actual2D; receipt state_preserved=true. Large always-open camera overlay is replaced by one readable Camera button at the upper-right; source forest palette and full gate reveal retained. Expand/collapse and existing shared-camera action guards pass actual ECS tests. A separate interactive preview92570 was launched to inspect expanded layout, but CUA rejected the non-bundled executable and then reported the Mac locked; no pointer action occurred, no security bypass or unrelated app interaction attempted. Preview was stopped via its own PTY Ctrl-C. Expanded-layout pixel verification remains pending when native UI access is available; default compact layout is verified. Roof seams, excessive warm litter, facade/fascia paint, remaining tree coverage/material depth and separate planar-ledge regression are still open; no whole-scene acceptance.

Compact native camera toolbar implemented, visual verification pending: existing eight camera controls remain available behind one44px-high `Camera` toggle; panel is collapsed by default, preserving scenery rather than always occupying roughly a fifth of the frame. Original zoom/orbit/pitch/first-person/reset actions, scroll/pinch and world-visible guards retained. Replaced unsupported Unicode minus with ASCII hyphen. New ToolbarState/ToolbarPanel only affect presentation; toggle does not change camera settings or generate gameplay input. Test14356 terminal/pass2/2 in1m55s, including existing camera action guard and new real ECS expand/collapse/inactive-world test. Native34912 confirmed live, log `target/outside-art-compact-camera-toolbar-native-build.log`, -j1/voxel opt1/Bevy opt0. Poll same handle, then fresh Ilex(1,37),noon,orbit2,zoom1 pair to inspect collapsed layout, and expanded layout through actual UI if available. Keep game/source colors and gate behavior intact; no whole-scene acceptance yet.

Forest-gate full-reveal native62582 finished successfully4m47s. Fresh production pairs inspected against matching2D: `target/outside-art-gate-full-reveal-ilex{,-2d}.png` Ilex(1,37),noon,orbit2,zoom1; `...-ilex-orbit0{,-2d}.png` same position/hour,orbit0. Both receipts state state_preserved=true. Blocked view now reveals the blue/lavender clearing without the gate's former full-frame stipple; clear orbit0 shows the original gate opaque with purple roof, proving geometry remains. Existing tree coverage is still visible near the trainer, unchanged by this gate-only policy. Component behavior verified; whole forest not accepted. Coarse roof panels/large black seams, excessive warm litter, wrong facade/fascia paint and limited canopy/material depth remain. A separately added camera toolbar now covers roughly the top fifth of these production frames and its minus button shows a missing glyph; this independent UI is another readability defect, not caused or hidden by the gate change. The separate planar-ledge test failure remains open. No build/capture pending from these handles.

Forest-gate full reveal implemented, native verification pending: exact ForestGate/ForestGateClosed range opt-ins now supply residual opacity0 to the existing coverage-fade path. All other prepared objects retain default residual0.20. Per-object residual survives WeakPreparedGroup caching and linked overlays, reaches Fade at spawn, and uses the existing smooth easing/exit hold/restoration. No collision/rays, mesh geometry, source occupancy, default walls/trees or shadow base material changed. This removes settled gate stipple instead of leaving it across the clearing; geometry returns when player rays clear it.

Tests2871 terminal/pass full-reveal cycle1/1 in1m21s. Fresh source-range/cache isolation, existing wall fade/restore, timestep equivalence, real ECS camera/root/material stability and linked-source ownership regressions all pass. The unrelated planar-ledge relief assertion still fails on the current binary; preserved the separate planar soil change, no test weakening. Native62582 confirmed live, log `target/outside-art-gate-full-reveal-native-build.log`, -j1/voxel opt1/Bevy opt0. Poll same handle before fresh Ilex(1,37),noon,orbit2,zoom1 capture against `outside-art-source-forest-floor-ilex` pair; verify no settled gate stipple and unchanged source palette/game state. Also inspect a clear view of the actual gate from another camera/position to prove geometry is retained. No visual acceptance yet.

Live forest-floor native83189 finished successfully13m03s under concurrent host load. Fresh production pairs personally inspected: `target/outside-art-source-forest-floor-ilex{,-2d}.png` Ilex(1,37),noon,orbit2,zoom1; `...-ilex-night{,-2d}.png` same location at23; `...-route38-control{,-2d}.png` Route38(6,10),noon,orbit2,zoom1. All receipts state state_preserved=true. Ilex clearing is now source blue/lavender, preserving the source's fixed forest palette at both hours without another moon tint; original terrain/litter remains. Route38 ordinary path/pine/white-fence palettes retained. Independent grass work is visibly different from the prior control, so do not claim every unrelated mesh is unchanged from older screenshots. Forest floor hue correction is verified in production; whole forest remains unfinished: broad gate stipple field, wrong facade/fascia paint, coarse roof courses, overly conspicuous orange/yellow litter and limited canopy/material light depth. The separately introduced planar-ledge test failure recorded below remains open. No build/capture pending from these handles; no whole-scene acceptance asserted.

Live source forest floor implemented, native visual check pending: new original Rust `mesh/source_forest_floor.rs` samples complete opaque verified Forest ground tile05, converts actual live pixels to linear pigment, and recolors only existing role7 floor vertices while preserving the stable contact/weather luminance field. Map-space grain coordinates are retained with a separate76000 UV range; WESL recognizes sampled floors as live source paint, keeps neutral grain and avoids another colored night tint. Called from the authored production finish boundary in `build_scene.rs`; geometry, footing, claims, litter accents and other material roles unchanged. Missing/invalid samples leave existing diagnostic geometry intact; no extracted content or palette catalog added.

Tests90573 terminal/pass1/1 in6m15s. Fresh binary forest_floor filter passes4/4, including continuity/footing/litter/source-pigment and untouched roles. Native shader-review90340 terminal/success through the explicit `CRYSTAL_VOXEL_SHADER_REVIEW` hook with the prior executable, log `target/outside-art-source-floor-shader-review.log`; this proves shader compilation only, not new floor pigment. Native83189 is confirmed live, log `target/outside-art-source-forest-floor-native-build.log`, -j1/voxel opt1/Bevy opt0. Poll same handle; capture Ilex(1,37),noon,orbit2,zoom1 against `outside-art-ilex-gate-coverage` pair and inspect actual source palette; Route38 control should retain ordinary colors/white fences.

Concrete unrelated current regression: `forest_ledge_relief_and_litter_keep_native_cap_and_floor_contacts` fails its relief assertion on the fresh binary. It passes in prior `crystal_voxel_view-4b445daac5968931`. The test calls `polish_world_surfaces` directly and never invokes the new color finish. Current `ledge_finish.rs::append_soil` was separately changed to planar faces to avoid battle spike curtains; old displaced relief was removed. Preserved that independent work, did not weaken the test or restore it globally. This regression remains open and must be reconciled before broad outdoor acceptance.

Upper-crown native11507 finished successfully5m32s. Fresh production pairs inspected against matching2D: `target/outside-art-pine-upper-crown-route38-day{,-2d}.png` Route38(6,10),noon,orbit2,zoom1; `...-night{,-2d}.png` same state at23; `...-orbit0{,-2d}.png` noon,orbit0. All three receipts state state_preserved=true. Upper sprays now taper into covered leaders instead of the formerly bare masts; supported second view retains player readability, connected source white fences and distinct grass/path arrangement. Night stays navigable with dark cool foliage/lavender fences. Whole-scene foliage still has busy/fine-grain surfaces and limited light/material depth, sign placeholders and unfinished contact/detail work; this is not outdoor acceptance. No job/capture remains pending from these handles. Next concrete defect is Ilex's hardcoded brown/green floor pigment described below; resolve from live source samples while retaining ground/litter topology and map-space grain.

Pine upper-crown refinement implemented after the fresh Route38 critique: unequal5/4/3 upper boughs at staggered heights now carry main and paired lateral closed sprays, with source-seeded reach variation and a shorter leafy terminal leader. Lower limb recipes, source root/fit envelope, palette, understory, complete reveal ownership and distinct forest-family routing preserved. Upper geometry increases modestly; the per-bundle four-triangle cost is unchanged. Pine filter78660 terminal/pass2/2 in1m42s; streaming bounds91500 terminal/pass21.68s and family-cache isolation pass on fresh opt0 binary. New native11507 is confirmed live, log `target/outside-art-pine-upper-crown-native-build.log`, -j1/voxel opt1/Bevy opt0. Poll same handle, then capture Route38(6,10),noon/night23,orbit2,zoom1 against `outside-art-closed-pine-route38-*` pairs; critique leader/crown silhouette before acceptance.

Ilex wrong-ground pigment traced to actual code: `new_bark.rs::GroundFinish::color` explicitly replaces forest Lawn with hardcoded brown soil `[.31,.205,.105]` and green moss `[.13,.34,.075]`; this contradicts the actual paired source clearing's lavender/blue palette. `GroundFinish::new` receives no image samples. The scoped verified floor is already tagged role7 by `tag_floor`, preserving map coordinates and separating litter accents. `build_scene.rs` invokes `polish_world_surfaces_with_shapes` around2342 and has live images upstream; a source-sampled forest-only finish can use that boundary, keeping footing and original litter/terrain topology. Keep grain coords, avoid re-tinting already-night source paint, and preserve ordinary routes/Park. No ground pigment change made yet to keep the pine A/B isolated.

Native40820 finished successfully6m15s. Fresh normal production pairs personally inspected against matching2D: `target/outside-art-ilex-gate-coverage{,-2d}.png` Ilex(1,37),noon,orbit2,zoom1; `target/outside-art-closed-pine-route38-day{,-2d}.png` Route38(6,10),noon,orbit2,zoom1; `target/outside-art-closed-pine-route38-night{,-2d}.png` same Route38 position at23. All receipts state state_preserved=true. Ilex trainer reads more clearly through the gate, no repeated translucent wash; however its residual coverage creates a broad visible stipple field, and wall/floor source-palette/roof-course/sign defects remain. Route38 needles now retain more solid spray volume, white/day and lavender/night source fences retained, night vegetation cool/dark and paths readable. Pine upper leaders still look thin/bare, whole foliage silhouettes remain noisy below the approved layered-canopy standard; this is progress, not whole-scene acceptance. Next pine refinement should strengthen the upper bough/leader transition from this actual evidence, preserving source envelope, root/footing and regional families. No build or capture is pending from these handles.

Pine/gate verification advanced: session19499 finished successfully11m40s (default voxel opt2); both closed-needle/full-tree tests pass. Session29389 finished successfully4m34s; exact-range gate coverage/cache test passes. Refreshed opt0 binary `crystal_voxel_view-8dd8524c921c20b0` passes pine2/2, both-gate live source-paint/geometry/footing/fallback, coverage material preservation, source-family cache isolation, Route36 mixed-crown binding, and complete streaming bounds (session5599 terminal/pass18.72s). Native source identities verified via `tools/johto_art_sources.py --check`. One native build40820 is confirmed live, log `target/outside-art-closed-pine-gate-coverage-native-build.log`, -j1/voxel opt1/Bevy opt0. Poll exact handle before capture. Required captures: Ilex(1,37),noon,orbit2,zoom1 normal production to judge trainer readability and source purple roof, then Route38(6,10) noon/night23 same framing to critique new needle volume against prior directional pairs and approved art. No visual acceptance until inspected; gate roof-course/facade/floor/sign defects and whole-outdoor finish remain open.

Sign-content trace: ordinary Johto sign claims use canonical World RouteSign geometry; `VisualWorldFrame` currently carries map/tiles/actors but no source sign text. Generic blank/stroke boards remain unfinished. A later source-backed label treatment must consume existing external-pack sign content through presentation data rather than inventing text or committing a duplicate content catalog.

Scoped forest-gate cutaway implemented, tests/native inspection pending: successful ForestGate/ForestGateClosed appends record exact solid vertex ranges in `TerrainMeshData.coverage_fade_ranges`. Production `lib.rs` applies `occluder_fade::use_source_coverage_fade` after linked-group preparation and before adding grove groups. Only exact authored range matches opt in; linked captions inherit their owner's coverage sample, other walls keep blended reveal. Existing original surface collision/rays, fade timing/residual, shadow policy, geometry and footing are untouched. Added `source_coverage_opt_in_preserves_neighboring_walls_and_cached_ownership`; expanded both-gate source-paint regression verifies scope metadata and empty fallback metadata. Fresh test session29389 is queued behind19499 on our own cache, log `target/outside-art-gate-coverage-tests.log`, voxel opt0 to avoid another long optimized test compile. Do not duplicate either live command. Existing pine19499 still confirmed live (cargo99259/test rustc307), log `target/outside-art-closed-pine-tests.log`; it predates this gate edit and proves only the pine change. After both sessions finish, use the new opt0 binary to run the pine/forest-paint/streaming/family/source-crown regressions, then one native build and fresh normal Ilex+Route38 pairs. No visual acceptance yet.

Closed pine bundle refinement implemented, verification pending: ordinary Johto `foliage/pine.rs::spray` now uses one closed pointed four-face bundle per needle slot in place of two open folded pairs. Same four-triangle cost, half the needle vertices; source roots, seeded limb recipes, fit envelope, palette, understory, complete cutaway ownership and regional-family dispatch preserved. Expanded needle regression checks every edge closes twice and every face has outward finite volume across three seeds and horizontal/vertical/slanted directions, plus the existing full-tree index/footing/normal/silhouette regression. Test session19499 is confirmed live (cargoPID99259/rustc99349), log `target/outside-art-closed-pine-tests.log`; it uses default voxel opt2, so allow compilation to finish without restarting. Poll exact session, repair any concrete failure, then run `streaming_bounds_contain_every_source_leaf_and_branch`, `ancient_and_pine_templates_never_share_a_cached_family`, and the Route36 mixed-crown regression on the fresh binary. After passing, build native with -j1/voxel opt1/Bevy opt0 and inspect fresh same-state Route38 day/night pairs against prior directional captures and the approved reference. The new bundles are not visually accepted yet. Retired canonical/optional tree assets still absent. No other family geometry changed.

Forest gate roof topology diagnostic: both canonical world-exterior gate models have zero roof triangles disagreeing with authored normals (open1,152 rooftriangles; closed1,060). Roof seams therefore need course/contact refinement, not indiscriminate winding reversal. Existing source deck/course generator uses only7 across×6 rows; preserve shell/bounds/threshold while refining this scoped asset. Fade trace confirms ordinary gate walls/roof still use per-surface blended alpha, whereas grove coverage already shares a visibility sample. A scoped gate opt-in can avoid accumulating layers without changing all walls or weakening player protection; no global fade change made.

Forest-gate paint native build finished successfully in5m00s. Fresh production pair `target/outside-art-ilex-gate-source-roof{,-2d}.png` at Ilex(1,37),noon,orbit2,zoom1 inspected; state_preserved=true. Separate opaque diagnostic `target/outside-art-ilex-gate-source-roof-opaque-diagnostic{,-2d}.png` (same location/settings, zero-radius diagnostic only) also inspected/state-preserved: roof is now source purple, not teal, confirming live paint in production. Diagnostic exposes large coarse staggered roof panels with visible black seams; original brown fascia and wrong wall/floor paint remain. Normal production still layers translucent roof/walls over the trainer. Next work should repair forest-gate roof course/contact geometry and source facade/fascia palette, and resolve multilayer cutaway without accepting the opaque diagnostic as playable framing. Full Ilex and outdoor finish remain incomplete.

Forest-gate source roof paint implemented: actual world-exterior ForestGate/ForestGateClosed now use live roof swatches from the authenticated 2D source. Separated the open gate's 44 low doorway triangles into `forest_gate_door`, preserving geometry/normals/pigment/bounds/door anchors; canonical JSON, Rust routing, generator and hash-guarded native world-exteriors source agree. Native source publication preserved 6,666 unrelated object signatures. Missing live roof samples retain exact source tops instead of guessed teal paint. `forest_gate_roof_follows_live_source_without_changing_its_door_or_plot` and expanded `fitted_house_roof_palette_preserves_geometry_and_non_roof_materials` both pass, including both gate variants. Native build is confirmed live as PID97071 (rustc97156), log `target/outside-art-forest-gate-paint-native-build.log`; do not start a competing build. Fresh same-state Ilex capture remains required before accepting the paint. Authoring scratch was removed after publication.

Trunk refinement native99929 completed. Fresh `target/outside-art-ilex-ancient-trunks{,-2d}.png` inspected at Ilex(1,37),noon,orbit2,zoom1 with state_preserved=true. Unequal trunks and the distinct blue-teal broadleaf family are present; whole-scene quality remains unfinished. The foreground gate has overlapping transparent surfaces that obscure the trainer; wall/floor/source-palette and sign finish still require correction. Existing coverage-fade machinery may address layering, but do not broadly alter another owner's occluder implementation. Route36 mixed-crown regression rerun on the current test binary passes; matching actual source/native proof still shows no stray scenery raster tree.

Ilex family/palette production review: fresh build84424 succeeded9m01s. Pair `target/outside-art-ilex-ancient-family{,-2d}.png`, Ilex(1,37),noon,orbit2,zoom1 personally inspected; receipt state_preserved=true. The live forest now has closed broadleaf layers and cool teal/mauve identity, rather than generic Johto pines. Headless actual-pack audit75561 succeeded:4,560 cells claimed `forest/ancient_tree`, ZERO `johto/tree` claims, 4,906/6,480 total modeled cells. Remaining classifications:flat1,350/ledge26/raised62/ramp8/shore34/water94 (no unmodeled tree classification reported). Temporary audit JSON removed after inspection. Actual ordinary Route38 control pair `target/outside-art-ancient-family-route38-control{,-2d}.png`, (6,10),noon,orbit2,zoom1 inspected; state_preserved=true and ordinary pines/white fences retained. These prove family isolation, not polished whole-scene acceptance.

Critique/follow-up implemented: Ilex trunks looked overly smooth/uniform. Added source-seeded sinuous lean, restrained physical bark fluting, surface-derived normals including root flare and bend, longitudinal grain, and6/9/8 unequal bough recipes. Root center, plot fit, leaf palette and sparse-interior/footing policies unchanged. Forest filter69187 passed4/4 after this change. New native build99929 is CONFIRMED LIVE, log `target/outside-art-ancient-trunk-native-build.log`, -j1/voxel opt1/Bevy opt0. Poll same handle before new Ilex same-state capture; do not call trunk finish accepted until inspected.

IMPORTANT corrected gate diagnosis: actual audit identifies `world-exterior/forest_gate` and `forest_gate_closed` (144 cells each), NOT the Johto RouteGate model examined earlier. Use `tools/build-world-exteriors.py::forest_gate` and canonical world-exterior source. It uses roof_teal (also a low door leaf via shared lab-style front_entry), so roof-paint material separation still needs deliberate scope. Current `exterior_models::recolor_fitted_roof` paints all roof-prefixed groups, and modeled_exteriors only selects source paint for a narrow world-roof list; don't broadly recolour the door. Ilex still has a huge translucent, wrong-colour foreground gate and wrong-colour floor/unfinished contacts; the broadleaf family is a step toward the ancient-forest requirement, not Ilex acceptance. Pine needle improvement remains pending; do not reintroduce retired assets or route Ilex back to pines.

Regional forest mismatch found and correction implemented, visual verification pending: both source-verified Ilex tree paths previously reached `foliage::pine` through a familyless GroveRecipe/shared cache. Introduced `GroveFamily::{JohtoPine,AncientForest}`, selected from the authenticated source tileset; grove recipes and shared preparation now retain this family and the cache key includes it. Forest claims use `forest/ancient_tree`, with water-contact ownership recognizing that family. Ordinary Johto/pine, Park, Kanto, Fruit and Cut routing remains independent; no retired assets were restored.

New original Rust `foliage/ancient.rs` uses substantial flared trunks, unequal spreading forks, overlapping closed broadleaf surfaces and the existing rooted understory. It retains source plot bounds, deterministic seeds, source footing, sparse interior rules, complete reveal ownership and compact local indices. Forest leaf/bark colours were revised to cool blue/teal and muted mauve after inspecting the actual fixed-night Ilex2D palette; shared warm branch geometry is recoloured only in the new forest producer. Existing pine geometry is unchanged this turn; do not proceed with a generic pine edit until family dispatch/visuals are verified.

Tests: initial ancient geometry2/2 passed96486. Expanded source-recipe/complete-art/footing, family-cache isolation and ancient-geometry filter66353 passed4/4. Actual shared streaming-bounds regression16940 passed with BOTH families/scales/neighbour layouts. Existing pine regressions ran via fresh test binary in `target/outside-art-ancient-pine-preservation-tests.log`. Baseline production pair `target/outside-art-ilex-family-before{,-2d}.png`, Ilex(1,37),noon,orbit2,zoom1 was personally inspected; state_preserved=true. It confirms the pine-family mismatch, dense noisy foliage and a large translucent, wrong-colour gate; those remain whole-scene defects beyond tree-family work.

First native+audit build56961 succeeded7m21s, but the forest colour revision occurred around compilation, so it is not sufficient proof of the final palette. Freshness build84424 is CONFIRMED LIVE, log `target/outside-art-ancient-palette-native-build.log`, voxel opt1/Bevy opt0/-j1. Poll that exact handle; don't restart from quiet output. Once complete, capture Ilex(1,37) same state and ordinary Route38(6,10) day to verify distinct forest family and preserved pines, inspect actual2D/3D, run source-ownership coverage forIlex if needed, and critique/fix the new canopy before claiming visual acceptance. No Ilex art target or whole-scene acceptance has been asserted.

Outdoor-light selection review completed: native75824 succeeded7m04s. Fresh normal production pairs personally inspected against2D and the approved Route38 finish reference: `target/outside-art-directional-day-route36{,-2d}.png` Route36(30,8),noon,orbit2,zoom1; `target/outside-art-directional-day-route38{,-2d}.png` Route38(6,10),noon,orbit2,zoom1; `target/outside-art-directional-night-route38{,-2d}.png` same Route38 position at23; `target/outside-art-directional-day-cianwood{,-2d}.png` Cianwood(15,51),noon,orbit2,zoom0. Every receipt states state_preserved=true. Source white fences remain white/day and lavender/night; night vegetation retains dark teal identity; Cianwood cyan roofs remain intact with no dark-deck regression. Directional surface-light selection is functioning and source-paint handling preserved, but overall pine finish is still too sparse/noisy and the scene remains far below the references; do not call this whole-scene visual acceptance.

Disabling fades via existing `CRYSTAL_CUTAWAY_DIAGNOSTIC=zero-radius` produced diagnostic pair `target/outside-art-pine-opacity-diagnostic-route38{,-2d}.png` at the same Route38 noon position. It was inspected; receipt state_preserved=true. The sparse exposed tips and noisy needle mass remain even with opaque trees, so the earlier impression that heavy fading was the main cause is NOT supported. Do not alter occlusion or reduce player protection from that hypothesis. Next investigate pine spray geometry/material readability (closed volumetric bundles, spacing/layering) while preserving rooted source plots, distinct regional families, bounds/triangular budget, cutaway correctness and actor scale. A closed four-face pointed bundle replacing the present two open folded-V bundles could retain four triangles per slot and reduce vertices; this is a candidate, not implemented/accepted. Inspect exact source family dispatch first, especially Ilex, to avoid imposing Johto pines on its ancient-forest identity.

Sign investigation: current `tools/build-world-exteriors.py` route_sign explicitly generates three raised bars; `VisualWorldFrame` has no source caption metadata. Actual background scripts exist in the runtime map event data, but reading/inferencing captions must be readonly through the supported production boundary and preserve source text; do not invent route labels from map names or introduce a second script dispatcher. No sign-caption work has been implemented yet. No builds/captures from this lighting cycle remain live.

Outdoor surface-light selection generalized, visual review pending: `lib.rs::apply_built_terrain` now enables the existing directional/hemisphere exterior shader branch for the shared `cave_water::outdoor` map classification plus a recognized outdoor source tileset (johto/modern/kanto/forest/park/battle_tower_outside/port), replacing the old six-map list. Existing cave classification takes precedence; room/cave fixtures cannot inherit outdoor surface light merely from a route-like map name. No sun/ambient clock, regional Ilex gain, cascade budget, shader pigment, geometry, source-paint85000 handling, gameplay or pack changes were made. Extended the existing actual material/hierarchy replacement regression through20 map/tileset transitions, including Route36/38, Goldenrod/Blackthorn/Cherrygrove, Park/Ilex/Kanto/port and cave/room returns; actors retain their independent material environment. Test3471 passed1/1 in2m08s, log `target/outside-art-outdoor-lighting-tests.log`.

Native build75824 is confirmed live, log `target/outside-art-outdoor-lighting-native-build.log`, voxel opt1/Bevy opt0/-j1. Poll the same handle before capturing; don't restart from quiet output. Required fresh comparisons: Route36(30,8),noon,orbit2,zoom1 against prior `target/outside-art-pine-after-asset-removal-route36` pair; Route38(6,10),noon andnight23,orbit2,zoom1 to inspect pine volume and white source-fence paint; Cianwood(15,51),noon,orbit2,zoom0 to retain the verified rigid roofs. Inspect day/night colors against matching2D and the approved finish references. The selection logic is tested, but improved visual quality is not claimed until those captures are inspected. Roof repair remains verified; native source tree retirement remains intact.

Rigid-building roof repair VERIFIED: test46912 passed the new authored-Gym/doorway geometry and independent footing regression. The same fresh test binary passed all15 existing terrace regressions and the model-local roof coverage regression (17 applicable tests total, no failures). Native+audit build22007 succeeded6m59s. Actual final-geometry audit1762 on current canonical pack reported Cianwood0/28,788 and Blackthorn0/25,248 buried fine-tile samples, both zero inverted pitched faces; prior matching geometric audit had4,458/6,272 buried samples respectively. Result logs are `target/outside-art-rigid-roof-coverage.log`; temporary JSON was removed.

Fresh production pairs inspected: `target/outside-art-rigid-roofs-cianwood{,-2d}.png`, Cianwood(15,51),noon,orbit2,zoom0; and `target/outside-art-rigid-roofs-blackthorn{,-2d}.png`, Blackthorn(18,20),noon,orbit2,zoom1. Both receipts state_preserved=true. Large dark roof areas are gone, cyan source paint retained, cream gables and GYM lettering remain intact. Cianwood capture5.61s; Blackthorn11.57s on this host under its current load (not a portable frame-performance claim). Roof component now accepted against the defect, not entire town finish. Blank line-signs, coarse repetitive facade/roof finishing, grass edge ribbons, remaining ground/cliff-contact details and flat/busy foliage keep the full goal incomplete.

Next evidence-based lighting issue: `lib.rs` terrain `cave_reveal.z` only selects directional exterior surface light for NewBark/Route29/Route26/Park/Ilex; Route36/Route38 and most outdoor towns retain the older orientation-independent formula. Current shader already has modest normal-facing sun/hemisphere response in that branch; live source-paint85000 remains separately neutral to avoid night double-tint. Generalize proven outdoor lighting from authenticated outdoor map/tileset identity, retain cave/interior ownership and regional clock/woodland behavior, then inspect day/night pairs before acceptance. No lighting change has been made yet, to keep roof A/B evidence isolated. Current native binary contains the verified rigid-roof repair.

Rigid-building roof repair implemented, verification pending: corrected geometric audit `17101` completed11m16s; actual final-scene run `61721` succeeded and reported Cianwood4,458 / Blackthorn6,272 buried fine-tile samples under broad structural roof faces, with zero inverted pitched faces. Structural faces were explicitly distinguished by area and entry shelters below their eave were excluded; logs `target/outside-art-roof-geometric-diagnostic.log` retain evidence, temporary report JSON removed. Terrace processing previously applied the nonlinear ground field separately to every authored roof vertex, so broad decks could cross their fine tiles despite valid model-local coverage.

Added `mesh/rigid_buildings.rs` and per-building vertex-range/doorway metadata in the two existing successful building producers. Terrace finishing excludes those complete solid ranges from deformation/normal rebuilding and translates them uniformly by the doorway's source-field support. Ground grading, footing computation, bank/mineral contact processing, drop faces, gameplay and the pack are unchanged. Added `terrace_grade_preserves_authored_roof_and_doorway_geometry` using the actual Gym model across a finite ledge grade; it checks geometry/normals/colors/indices and every independent ground-footing sample. Initial test compilation needed explicit Vec2 imports (fixed). A following test caught a concurrently edited whirlpool-trace macro, already corrected by its owner in current source; native build caught a Hoppip JSON being removed mid-scan during another owner's canonical GLB conversion. Current Hoppip GLB exists; no obsolete JSON was restored. Failed handles are terminal, not live.

Retry test live session `46912`, log `target/outside-art-rigid-building-tests.log`, explicit voxel opt0 and -j1. Retry native+audit examples live/queued session `22007`, log `target/outside-art-rigid-building-native-build.log`, voxel opt1 / Bevy opt0 and -j1. Poll those exact handles. After test passes, run existing terrace tests from that same fresh test binary without triggering another broad compile. After native build passes, run corrected roof audit and fresh Cianwood/Blackthorn same-state captures; do not claim the roof repaired until the dark field is gone in production.

Latest pine removal verification: `target/outside-art-pine-after-asset-removal-route36{,-2d}.png`, current native binary/current canonical pack, Route36(30,8),noon,orbit2,zoom1 inspected; state_preserved=true. Raster tree remains gone after retiring optional-art meshes. Scene is still too busy/flat in the foliage, with blank line-signs and unfinished ground/building details; this is not whole-scene acceptance.

Latest verification and cleanup: native build `44643` completed successfully in 11m51s. Fresh production Cianwood pair `target/outside-art-gym-gables-cianwood{,-2d}.png`, `(15,51)`, noon, orbit2, zoom0 was visually inspected; receipt reports state_preserved=true. Gym gables are now cream/plaster and GYM lettering is readable; source cyan roof identity is preserved, but the broad dark roof defect persists. Corrected geometric audit build `17101` remains live (same handle; don't restart), now compiling against current sources.

Found and removed six retired tree mesh copies still tracked under the shared optional-art root: `assets/open-models/new_bark/tree.mesh.json`, `assets/open-models/johto/tree_lod.mesh.json`, and `assets/open-models/johto/tree_variants/tree_{2,3,4,5}.mesh.json`. Both canonical and optional model-directory scans now contain none of these retired tree or stone-post files. Current open-model bundle validation explicitly rejects the retired tree paths; the Rust bundle tool consumes interiors, not those trees. This cleanup leaves the shared Rust grove producer and source-specific Fruit/Cut/Park/Kanto/Ilex families intact. No pack, generated model bundle or compiled output was staged.

Roof diagnostic follow-up: audit build `68180` completed successfully in 12m53s; its first `--roof-coverage` run completed on Cianwood/Blackthorn, but the brightness cutoff incorrectly classifies many linear-color ceramic faces as dark decking. Do not treat those buried-sample counts as verified roof defects or causal evidence. Corrected the helper to distinguish broad structural pitched faces (area >=200 world units) from fine tile faces (area <100), reject lower entry roofs below a deck's eave, and report inverted pitched-face counts. Corrected audit build is queued/live session `17101`, log `target/outside-art-roof-geometric-audit-build.log`, with `-j 1`, voxel opt-level 1 / Bevy opt-level 0, matching native build `44643`. Poll these exact handles. Both were confirmed live; do not restart from a timeout. Temporary report JSON from the preliminary audit was removed. New second-angle native pair `target/outside-art-cianwood-roof-orbit4{,-2d}.png` was inspected: at Cianwood `(15,51)`, noon, orbit4, zoom0, the large dark Gym deck remains at the same physical roof area as orbit2. Pair receipt reports state_preserved=true. This capture uses the prior completed native executable and is diagnostic evidence of the roof issue, not visual acceptance of the subsequently published gable/nameboard assets.

Published localized traditional-house/Gym finish: canonical `traditional_house.mesh.json` remains 2,410 triangles; `violet_gym.mesh.json` drops from 3,138 to 2,886. Both retain exact old bounds/door anchors and pass raw winding validation with four outward plaster gable triangles each. The Gym now has original closed-prism GYM lettering instead of the old gold blocks and wing crests. In the hash-verified editable `connected-johto.blend`, replaced only 16 corresponding objects with 21 authored parts, preserving all 1,396 unrelated object signatures and studio placements. Publication checked the current native source SHA before storing. Authoring scratch directories were removed afterward. Native visual verification is still required: queued build session `44643`, log `target/outside-art-gym-gable-native-build.log`, uses explicit voxel opt-level 1 / Bevy opt-level 0 and waits behind audit build `68180`. Poll those exact live handles before any restart; large dark roof fields are not claimed fixed.

Roof investigation continuation: the final `refined_roof_panels_stay_above_the_structural_deck` regression passed when run directly from the freshly compiled test binary (1 test, 0 failures). It proves model-local roof coverage, while the inspected Cianwood production image still shows large dark areas. Added opt-in `--roof-coverage` to the existing `audit_authored_assets` example: it checks final live-painted pitched triangles for dark surfaces above tile centers and prints actual terrace edges. This standalone diagnostic does not change gameplay output or renderer behavior. Its native build is live in session `68180`, log `target/outside-art-roof-scene-audit-build.log`; poll the same session before running the updated tool on CianwoodCity and BlackthornCity. The older Cianwood coverage audit confirmed two actual jump-edge samples, but does not yet establish that terrace deformation causes the roof defect. Temporary audit JSON was removed. The Gym/traditional gable/lettering candidates were subsequently published as described below; they do not establish acceptance of the large roof surfaces.

Route 36 stray 2D tree verified fixed: the terrain owner's exact Johto mixed-crown bindings for blocks `3c..3f` cover the `3d` crown present in the external pack. Production capture `target/outside-art-route36-tree-fixed.png` at `(30,8)`, noon, orbit 2, zoom 1 shows the bright pixel card gone and native pine foliage in its place. Its paired `-2d.png` was inspected and `.paired-state.txt` reports `state_preserved=true`; Sudowoodo remains present as its original actor. The proof used the completed `target/agent-outdoor/debug/examples/new_bark_3d`; our own native build also completed afterward. Strengthened `mixed_small_tree_corner_variants_replace_complete_art_only` to exercise Route 36 and verify native foliage, unchanged footing, and lawn-only source UVs beneath every accepted crown. Focused regression passed (1 test, 0 failures), recorded in `target/outside-art-route36-tree-test.log`. The opt-in ownership diagnostic also reported no unclaimed interior tree cells on Route 36; its temporary trace JSON was removed after inspection.

### Native lettering/paint proof; roof defect and next asset candidate

Pipeline60459 completed successfully. Native build4m27s and inspected
state-preserved Cianwood `(15,51)` noon/orbit2/zoom1 plus Cherrygrove `(29,4)`
noon/orbit2/zoom1 pairs prove cyan compact-house roofs and readable MART/POKé
lettering. A further Cianwood zoom0 pair81932 is inspected/state-preserved;
it exposes a large black region/jagged blue patches across the gym roof.
These are component proofs and a roof rejection, not finished city acceptance.
Receipts use `outside-art-common-source-cianwood-*` and
`outside-art-source-service-signs-cherrygrove*`.

The connected-kit candidate under `target/outside-art-source-gym-gables`
finished50694: GYM lettering replaces gold bars/wing crests; traditional/gym
inner and outer end caps use plaster_light with outward normals. Audit proves
traditional2410/Gym2886 triangles, unchanged exact bounds/door anchors,
identical material/triangle geometry outside the cap/sign correction regions,
all face/normal winding positive and four outward wall caps per model. This
candidate is NOT published yet. Native source replacement must preserve
unrelated objects/layout, then store the existing source and only these two
runtime JSONs. Remove its task scratch after verified publication.

The independent roof/deck test initially failed compile on missing Vec2;
current source has the import. Its first geometry failure counted a chimney
as deck. It now compares only broad pitched dark deck faces and excludes
entry shelters below the deck eave, leaving vents/ridge caps legitimate.
Session73622 is the older retry; final source test is queued under41202.
Read both terminal results before deciding whether the black roof is a model
surface problem or later scene/rendering problem. Do not accept the roof from
unrelated model tests.

Route36 identity pair86925 is inspected/state-preserved with the raster tree
still present. Receipt lists actor0 as species:SUDOWOODO:sudowoodo, but that
alone does not attribute the card. Diagnostic39352 using preview flag
EVENT_FOUGHT_SUDOWOODO also preserves state but still lists actor0; therefore
it did not remove Sudowoodo and cannot distinguish actor/terrain ownership.
Do not treat it as successful isolation or relabel the card as a Cut tree.
Terrain scope is independently adding Johto3d/3f mixed-small-tree bindings and
ownership/fallback traces; preserve those edits. No actor-binding change made.

### Source paint fixture repair and raster-tree identity caution

Session8882 stopped on the new fixture's descriptor unwrap before native
build. The traditional fixture incorrectly started at source row0 and the
wide fixture used the wrong metatile pattern. Fixtures now use the catalog's
two-row background skip, exact2c/2a/2d over26/27/2f wide drawing, and correct
roof bands; roof images attach at the owned row offset. An additional assertion
requires actual source cyan on each model/town, not merely changed colors.
The sequential replacement pipeline is live under session60459.

That repaired source-paint regression now passes (six common models on three
towns, actual cyan selection plus geometry/footing/claims/fallback checks).
All12 model tests also pass with the published lettering and expanded roof
role coverage. Roof/native/capture stages remain pending under60459.
Paired capture receipts now record each compared actor's exact source ID,
center/size and facing as JSON, only when an explicitly requested paired
capture completes. This adds no gameplay output or dispatcher behavior and
will help distinguish an actor card from terrain rather than guess its family.

Do not call Route36's visible raster tree a Cut tree without runtime identity
proof. A read of the actual pack with the existing audit executable found no
Cut-block sources there, but found two Johto3d small-tree drawings and possible
actor rendering remains to check. This was a base-map/source read only, using
an older audit executable; it does not prove current production geometry or
the specific visible card's identity. The temporary source audit JSON is
removed. Inspect the live card's source/actor identity before choosing its
replacement. Leave the terrain owner's pending owned-card fallback guard,
trace span and original-background source selection edits alone.

### Published source lettering; shared-town paint verification

The sign export finished successfully. Native replacement changes only11 old
sign parts to33 new parts, compensating for existing studio layout offsets.
All1,379 other objects retain exact geometry/transforms/material roles and
both model bounds remain exact. The first native attempts saved nothing:
Blender mutates the supplied collection-name list, the old source lacked an
unused ink material, and unlinked imported collections had unresolved world
transforms. The final procedure links the temporary collections before
measuring them, adds only the new ink role and proves all preserved signatures.
The existing native source and Center/Mart JSONs are published; native source
identities validate. Finished authoring scratch is removed. Runtime visual
lettering acceptance is still pending.

Native pipeline23993 completed: build17m23s, Route38 `(6,10)`, Route36 `(30,8)`
and Cherrygrove `(29,8)` noon/orbit2 pairs are inspected, all state-preserved.
The split needles and shallow roof panels are visible, and compact-house
cream gable caps are confirmed. Full appearance is rejected: foliage remains
visually busy/flat, Route36 retains a raster Cut-tree card, Cherrygrove has a
large ghosted foreground roof, and signs/lighting/ground need further finish.
Receipts use `outside-art-pine-needle-tuft-*`. Corrected scatter tests77058
completed with both tests passing; roof Cargo check38056 passed allfour.

`modeled_exteriors` now paints common House/PlayerHouse/TraditionalHouse/
Center/Mart/Gym roofs from live source on the generic path, removing the old
town-name exceptions. Service brick/mortar paint reuses the owning sampler.
Missing source coverage preserves the source drawing. Gym roof courses now
use the same shallow refinement. A new six-model/three-town regression checks
source selection, unchanged geometry/footing/claims and missing-paint fallback;
the model-role test covers allsix families. This does not cover teal Lab/Gate
roles (their existing material also paints doors), special Ecruteak producers,
or multi-tier landmarks; those remain source-review work. Sequential tests,
native build and Cianwood-south/Cherrygrove-center captures are live under
session8882, logs `outside-art-common-johto-source-paint-*` and
`outside-art-source-service-signs-*`. No visual acceptance yet. The shared
gym's invented wing crests/three placeholder bars still need source lettering.

### Source storefront lettering candidate and Cianwood rejection

The inspected Cherrygrove2D pair proves storefront labels `MART` and `POKé`.
The connected-kit recipe now uses original closed-prism lettering for those
words on pale boards, replacing the generic M and invented healing emblem.
No font dependency or raster copy is introduced. Candidate export is live
under session54967 in `target/outside-art-source-service-signs`; nothing from
that candidate is published yet. Both runtime candidates now pass an exact
audit: Center5810/Mart6390 triangles (existing6500 budget), identical bounds
and door anchors, identical material/triangle geometry outside the sign area,
and positive face/normal winding on every source triangle. Before publication,
verify editable native
collections; retain unrelated collections. Remove this task scratch after
verified publication. The unused old medallion recipe is removed.

The fresh Cianwood south pair in `target/outdoor-new-cianwood-perimeters/`
is inspected. Source compact/Gym/Center roofs are cyan; native compact roofs
are red and Gym purple with white invented wing crests. Generic
`modeled_exteriors::append_building_with_samples` currently samples only
TraditionalHouse and two named map exceptions. Next source-palette work should
cover all exact-bound common Johto roof models, reuse live service wall/mortar
sampling, and preserve source drawings if paint is unavailable. The named
roof recolor helper also lacks teal roles. The gym authoring recipe adds bird
crests to every reused gym model, while its actual source says `GYM`; replace
that invented detail with consistent source lettering. None of these
Cianwood defects is fixed or accepted yet. Leave the terrain owner's small
background original-cells/source-shapes call and apron sampling work alone.

### Smaller shallow roof courses under verification

Current concept/native Cherrygrove comparison still shows broad pillowy roof
panels. `roof_tiles.rs` now splits each authored panel into two narrower
ceramic courses, lowers relief from0.055 to0.014 model units, and retains all
outside edges on the original closed course box. Two4x4 grids use the same
64 triangles as the former8x4 grid; no raw model volume or door changes.
Blue Mart ceramic now uses this shared treatment as well. Source paint roles
remain separate and recolor from the live atlas. A geometry regression checks
both shallow crowns, the center valley, outer edges, winding and exact budget;
the existing roof perimeter/normal/material test also covers Center/Mart now.
Roof tests are queued under session38056 (same task-owned Cargo cache,
confirmed waiting for the live build lock); fresh native appearance remains
unverified. Session23993 is
still live: ground-height and live-house source/fallback tests passed; scatter
checks are compiling before final native build/captures.

Roof evidence update: the compiled current test binary contains the new split
course test; running allfour `roof_tiles::tests` directly passes. Source
perimeters, two shallow crowns, center valley, winding and64-triangle budget
are verified. The earlier scatter Cargo filter was incorrectly missing the
`new_bark` module and ran zero tests; it is not passing scatter evidence.
The actual two scatter tests failed because shortened flower stems retained
their old leaf offset, placing leaves below ground. Leaf attachment is now
raised with the stems; unchanged source-cell/height/world-coordinate tests
are queued under session77058 with the correct module name. Native pipeline
23993 and full Cargo roof check38056 remain live; do not restart quiet handles.

### Civic masonry native proof and pine needle study

Exec22314 completed successfully. The native build took7m55s; inspected
Cherrygrove `(29,4)` noon/orbit2 and hour22/orbit2 pairs preserve state.
Receipts use `outside-art-civic-masonry-cherrygrove{,-night}{,-2d}.png` and
paired-state files. The source pink brick field and pale mortar now have
actual shallow relief and visible courses. The result is component progress;
façade depth/finish remains modest, gables are plain, signage is schematic and
night windows remain overly blue compared with2D. Ground surface also changed
in the shared worktree and remains visually noisy; no source-ground acceptance
is implied by these masonry shots. Both finished source scratch directories
(civic masonry and compact gable) are removed after verified publication.
The compact end-cap regression passed in the12-test compiled model run; fresh
south-view native confirmation is still needed because that edit followed the
previous binary's start.

The current pine sprays' wide lance blades read as fern fronds in those actual
frames. A production Rust change now emits two narrow V-folded needle bundles
per prior blade, retaining the old four-triangle budget per slot. Stagger,
root/base coordinates, source plots, thinning and cutaway ownership remain.
A focused geometry regression checks slenderness, actual folded faces, fixed
triangle count and each complete tree's compact16-bit index range. Existing
foliage winding/normal/root/crown-envelope tests remain unchanged. Initial
foliage compile exec13061 failed on a shared actor test calling private
`skin_point`; current source already exposes that crate-local method. The
rerun is sequential under exec67217. All **thirteen foliage checks** now pass,
including the new slender/folded/triangle-budget/index-range regression and
unchanged root/winding/normal/envelope checks. Exec67217 then failed the
plot/footing regression: it measured every elevated solid vertex, including
ground flowers outside the tree plot (flower tops are exactly 8 * 0.098 =
0.784 high). The unchanged crown limits now inspect the pine's explicit
cutaway ranges, including its understory, and require nonempty ranges in both
variants. The corrected regression completed successfully under terminal
session38075 (one test passed, unchanged envelope limits and footing checks).
Native build session78277 succeeded in9m36s. Two broader suite failures were
then identified: flowers exceeded the existing low-decoration height limit,
and the old runtime-house fixture assumed roof replacement without paint.
Flower stems are now shorter (the existing <0.6 assertion is unchanged), and
the house fixture explicitly verifies both live-paint replacement and the
32-quad original-drawing fallback with missing paint. Targeted ground/house/
scatter checks, final native rebuild and three paired captures run sequentially
under session23993. Route38/Route36/Cherrygrove-south captures remain pending. Log
`target/outside-art-pine-needle-tuft-tests.log`. New native captures are required
before any visual acceptance; no old tree asset is restored or retained as an
alternate implementation. The full outdoor goal is still incomplete.


### Compact source roofs verified; physical civic masonry underway

Civic verification update: exec2641 stopped on the new relief regression's
selector, not on authored geometry. Mortar's continuous front quad has only
corner vertices at heights0.285/2.415, outside the test's brick-course range.
Raw candidate inspection proves brick-frontZ1.395023 versus mortar1.372795.
The corrected selector retains the same0.01..0.05 relief requirement and
samples the full dedicated mortar role. Current pipeline exec22314 passes all
**eleven model tests** and is compiling the source-palette regression, then
native/day/night captures. No visual masonry acceptance yet.

The inspected compact-house south view proves that its remaining black end
triangles are roof-shell caps, as in the civic models. Two existing cap
triangles now share `plaster_light` (cream), preserving source façade identity,
positions, bounds, doorway and triangle count. Its authoring call selects only
this end-cap role; roof source swatches remain separate. A regression checks
both visible end faces' outward normals and wall ownership when roof paint
changes. The native source uses material-combined objects rather than the
recipe's original names, so the first named-object edit found no object and
saved nothing. The corrected edit identifies the exact roof-dark end faces
within `house__roof_red_dark` and splits them without moving vertices or other
collections. Source store exec22267 completed successfully. The compiled current test
binary now passes all **twelve model checks**, including the new compact-house
end-cap regression (`outside-art-civic-masonry-all-current-model-tests.log`).
The source wall/mortar palette regression also passes. Native build/captures
remain live under exec22314. Test/capture verification
must use the next compiled source; it was added after the first masonry model
compile. Leave terrain_tracking.rs and slowpoke_well.rs to the terrain goal's
active well palette-refresh work; no simultaneous edit is needed here.


Exec39483 completed: the compact-house source-roof regression and allten New
Bark model tests pass. Native build3m24s produced two inspected preserved-state
pairs at Cherrygrove `(29,4)` and `(29,8)`, noon/orbit2. The compact roofs are now
source cyan. The south view clearly exposes compact houses' remaining black
roof-end caps and heavy ghosted foreground cutaway; neither is accepted. Their
facades use the source's separate warm palette, so the civic pink-wall mapping
must not be applied globally. Receipts use `outside-art-compact-house-roof-*`.

The civic wall candidate replaces flat pink paint with six modeled running-bond
courses across allfour existing walls. Shallow closed brick faces have restrained
clipped arrises, staggered half bricks and a separate recessed mortar backing.
Existing trim/glazing/doors retain their envelopes and occlude the wall detail.
Brick and mortar paint are sampled separately from the same live upper-wall
source bands (saturated clay field / pale mortar); both use the verified
source-paint shader marker. Gable ends retain source pink.

Authoring validation proves all source triangles align with their normals,
original bounds `[-2,0,-2]..[2,3.448,2]` and doorway `[0,0,2]` remain exact.
Counts5866/6398 remain below the existing6500 authoring budget, which was not
relaxed. The candidate replaces only the existing Center/Mart runtime meshes
and editable native collections, retaining allother collections. Current source
publication, model/relief/source-palette tests, production build and day/night
pair verification are sequential under exec2641. Logs use
`target/outside-art-civic-masonry-*`. Do not accept appearance before inspecting
those new images; remove the task-owned civic masonry source scratch after
needed source receipts finish. No game pack, audio data, scratch or compiled
artifact is staged. Whole outdoor finish remains incomplete.


### Verified source colors and exposed gable defect

Latest cap verification: exec76047 completed after the interior quoting repair.
The native build succeeded in4m00s; its Cherrygrove `(29,4)` noon/orbit2 pair is
inspected and state-preserved. The black visible end triangles are now source
pink, with retained roof envelope and door/ground placement. Receipt stem is
`outside-art-service-gable-cap-cherrygrove`; the old inner-gable-only picture
remains rejected historical evidence. Gable authoring scratch is removed after
verified source publication and the inspected native result.

Compact houses' old red roof remains a confirmed source mismatch in this pair.
The exact existing `ModelKind::House` drawing/door guard now also samples live
source roof swatches, retaining its separate existing facade/trim and every
position/normal/index/claim/footing. Missing source paint retains2D rather than
inventing red. A focused regression tests distinct cyan/ochre source swatches
and missing coverage. Current test/model/native/two-view verification runs in
sequence under exec39483; logs use `outside-art-compact-house-roof-*`
and `outside-art-gable-and-roof-model-tests.log`. These pictures must be
inspected before accepting the roof correction. No whole-scene masonry,
roof-course, foliage, signage, ground or lighting acceptance is implied.


Gable update: exec86374 passed the focused wall-normal/role regression, built
native successfully in2m13s and produced an inspected, preserved-state pair
(`outside-art-service-gable-cherrygrove`). The image **still showed black end
triangles**. The inward inner gable was real, but the larger closed roof shell's
end caps sit outside it and inherit outline-black `roof_*_dark` paint.
The actual visible end caps are now moved into `plaster_service_wall` with
unchanged positions/envelope/counts, retaining the dark deck/underside only.
The authoring roof helper accepts an optional end-cap material; only these
service calls select source masonry. The existing native source separates
those same two shells into deck and end-cap parts without moving vertices.
The regression now also inspects original roof-dark end faces, so an unpainted
visible cap cannot pass just because hidden inner gables are correct.

Cap publication and the strengthened gable regression passed under exec20063.
Its native build failed on a newly introduced interior-test syntax error:
`source.replace(""timber_gold"",""unknown_top"")` in interior_models.rs.
The quotes are escaped without changing test intent. Native rebuild/capture
now resumes under exec76047.
Receipts use `target/outside-art-service-gable-cap-*`. Do not claim the black
triangles fixed from the inner-gable regression or the previous rejected image.
The task-owned gable source directory remains temporarily for these source
receipts and should be removed once publication/verification finishes.


Exec81649 completed successfully: six source-paint/refresh tests, nine New Bark
model tests, fourteen fence tests, native build5m14s and three preserved-state
pairs. The latest compiled exterior-model binary also passes allseven tests,
including the new source-roof UV/untouched-material assertion. Daylight
Cherrygrove now has both source cyan roofs and pink walls. Its initial night
pair and Route38 night pair still failed color despite the UV marker.

The actual cause was the shader's older map-specific exterior gate:
`cave_reveal.z` selected only NewBark/Route29/Route26/Park and excluded both
Cherrygrove and Route38. The source-paint rule now executes independently of
that gate, with a small float interpolation tolerance around the explicit UV
marker. Unmarked materials and geometry remain unchanged. Exec75947 completed
its native build and Route38 night / Cherrygrove night / Cherrygrove day pairs.
Allthree are inspected and `state_preserved=true`; their receipts use
`target/outside-art-source-paint-map-independent-*`. The electric blue fences
are now source-like lavender at night, and Cherrygrove roof/wall hues follow
the actual2D palette. This verifies the scoped source-paint correction, not
whole-scene night lighting, material craft or finished outdoor quality.

The daylight image also exposes black triangular openings at both buildings'
right gables. Raw mesh inspection proves inward-X normals on their right end
triangles. The exact existing positions are retained; those triangles now face
outward, both gable ends share `plaster_service_wall`, and unused old gable
vertices are removed from the original plaster group. The authoring recipe
corrects open-triangle normals explicitly after Blender's ambiguous consistency
operator. The existing editable connected source is updated at the same four
gables with unchanged positions/transforms; all other source objects remain.

Gable source publication completed under exec62619. Its initial regression
selected timber fascia as well as wall faces and failed the material-role
assertion. The selector now restricts itself to the original/new plaster wall
roles, retaining outward-normal and both-end closure assertions. Current
verification is sequential under exec86374: verified
native source store, focused gable role/outward-normal regression, production
native build and fresh Cherrygrove `(29,4)` noon/orbit2 pair. Logs use
`target/outside-art-service-gable-*`. Do not claim the opening visually fixed
until that new image is inspected. Remove the task-owned gable source scratch
directory after needed receipts finish. Compact Cherrygrove houses still have
an incorrect red roof in3D versus cyan in2D; service masonry is flat paint,
signage is schematic, needle blades are too broad/fern-like, and ground/light/
regional coverage still fail the full art bar. Those remain part of the goal.


### Source-paint and retired-tree continuation

Exec14120 completed after33m40s with **two service-source tests passing**.
Current-source verification/capture is now sequential under exec81649: source-paint/refresh tests,
all New Bark model tests, fence tests, production native build, then Cherrygrove
noon/night and Route38 night pairs. Logs use `target/outside-art-source-paint-*`,
`outside-art-service-model-current-tests.log` and
`outside-art-source-fence-paint-current-tests.log`. Fresh native art remains
unverified until the pictures are inspected. The cache/renderer now uses the
worktree's Bevy0.20 dependencies; the long first compile was confirmed live.

The current source tree meshes/catalog hooks were removed by the shared tree
cleanup. A remaining old contour utility/hash test and unreachable authoring
calls still targeted those removed assets; they are now deleted. All changed
Python authoring files parse. Native-source inspection then found six retired
Tree objects in `new-bark-kit.blend` and30 in `foliage-lod.blend`, including the
old LOD, reference copy and crown construction collection. Those collections
are removed from the existing verified sources. In-memory signatures prove all
68/18 unrelated retained objects' transforms, geometry and materials unchanged.
The cleaned sources are saved through the native hash-verifying source tool;
no historical asset was restored and no new duplicate catalog was added.

A source-palette refresh regression now verifies that changed fence/roof image
handles invalidate the retained geometry cache at a time-of-day boundary.
This is needed because sampled paint carries the current2D palette and must
not remain daytime paint after the source changes to night. Current-source
shader/metadata verification is still pending in the new pipeline. The first
latest-source filter completed with **six passed** including the palette-handle
refresh regression and service-paint role/UV checks. The exterior roof painter
also now tags only its source-recolored roof vertices; its role regression
asserts that other paint/UVs remain unchanged. Run `exterior_models::tests`
against the next completed compiled test binary as well. The current pipeline's
remaining model/fence/native/capture stages are live under **81649**. All
**nine New Bark model tests** have now passed, including service/cabin source
paint, untouched trim, authored geometry/normal validity and doorway/plot fit.
The fence repeat is compiling; native captures have not started yet. The
exterior-roof metadata check must still run against the newest test binary.

The task-owned fence, service and retired-tree source scratch directories
have been removed after verified source publication; their
code/signature receipts are in the tool transcript. Runtime assets, native art
sources and the existing source identity manifest remain in the repository.
No pack or compiled/scratch artifact was staged.


The user clarified that **2D is the actual reference** and that the Johto
"stone posts" are **white fences**. The previous paintover/material explanation
was wrong. `$4a` is already documented by `append_johto_vertical_fences` as the
north/south stripe, including source elbows. The modeled prop pass incorrectly
claimed it first as `StonePost`, suppressing the rails. That claim, enum/catalog
entry, runtime mesh and editable source collection are removed. The same fitted
rail model now serves horizontal and vertical bays, with live source palette
sampling; no source opening is closed. The source-corner lower stripe can
complete its original horizontal rail bay. `renders/ART-STANDARD.txt` and
`GOAL.txt` record the user's source authority over the paintovers. Do not use
those images' stone posts or brown wood to override the 2D fence identity.

Fence verification is in progress under exec **45952**: source export/store,
focused `fence` tests, exterior model tests, production native build, then a
same-state Route38 `(6,10)` hour12/orbit2 pair. Logs use
`target/outside-art-source-fence-*`. The initial model test attempt failed
because a collected raw model was removed while its build script ran; that
attempt is not a passing result. Final source generation precedes the current
verification pipeline. Focused fence verification completed with **13 passed,
zero failed**, including the new white-palette/shared-rail ownership regression.
A final source-corner pairing edit followed that first compile; re-run the fence
filter against the next compiled test binary before claiming final coverage.
All **seven exterior-model tests** pass. The native build succeeded in5m01s.
The default capture target produced a uniform frame and was rejected; a fresh
explicit-image-target run (exec91364) completed successfully in34s. Its
`outside-art-source-fence-route38{,-2d}.png` pair is inspected and
`state_preserved=true`: continuous white rails replace the former disconnected
stone rows and brown pickets. All13 fence tests were re-run against the latest
compiled test binary and pass (`outside-art-source-fence-current-tests.log`).
Remaining source-card seams near the foreground corner, overly flat materials,
foliage/ground/sign finish and broader regional coverage are not accepted.
No pack is modified or staged.

Additional fence source/native pairs at Route38 `(6,10)` noon/orbit4 and
hour22/orbit2 completed under exec91845 and are inspected/state-preserved.
The opposite view confirms connected rails, corner runs and retained openings.
The night view is **rejected for color**: white source fences become strongly
blue in3D, beyond the actual2D lavender shade. The live sampled color already
contains the map time-of-day palette; the outdoor shader then adds a second
colored light. A scoped source-paint UV marker (`85000,0`) now preserves sampled
paint's color/energy under face shading, shadow visibility and contact AO,
without repainting untagged foliage/actors or changing any geometry. It applies
only to source-recolored fence/roof/wall vertices. Updated material-role tests
assert this marker while preserving other UVs. This shader correction still
requires current-source tests, native compilation and new night/day pairs.
Do not treat the rejected blue fence image as final acceptance.

Cherrygrove service palette work is now under exec14120. The source-authoring
wall role separates masonry from pale door/window trim. Its candidate preserves
bounds and door anchors exactly; all original triangles agree when rounded to
1e-5 model units, with only 18/27 triangles differing by1e-6 exporter rounding
in the Center/Mart. Warm base framing no longer shares roof paint. The production
building binder samples the complete roof and upper two masonry bands from the
live pack, with exact source-drawing/door guards. Missing paint coverage retains
the original source drawing rather than guessing colors. Source and model
regressions are added; test/build/capture acceptance remains pending. Exec14120
is confirmed live (Cargo27762) after a shared dependency/profile update rebuilt
Bevy0.20/WGPU30; no compiler failure has been reported. Continue polling this
handle, never restart it because the output is quiet. The source-paint UV/shader
change followed this compile's start, so run a fresh focused test invocation
before the next native build to ensure those latest metadata assertions ran.
Then build the production example and inspect Cherrygrove day/night and Route38
night pairs. All artwork scratch stays ignored; remove the task-owned
`target/outside-art-{fence,service}-source` directories after their needed source
receipts are complete. No claim of finished masonry craft or whole-scene
quality is made from palette work.

The palette-pine coverage fix passes all **24 occluder tests** in
`target/outside-art-pine-coverage-tests.log`. Screen-stable cutaway coverage
now discards the same pixels across needle layers, avoiding accumulated alpha
that previously hid the trainer. The weak prepared-cache roundtrip preserves
this policy; original opaque materials and shadow geometry remain unchanged.
Production build and paired Route36 `(30,8)` captures at noon/orbit2,
noon/orbit4 and hour22/orbit2 are completed and inspected. All paired-state
receipts preserve state. The trainer is readable beneath foreground crowns
from both angles and at night. These are component evidence, not whole-scene
acceptance: flat green needle planes, source-sign/flower/cut-tree cards, dull
light and empty ground remain visibly unfinished.

Physical walk exec18818 was confirmed live but paused by the real-window-focus
guard. It was intentionally terminated and replaced with the supported QA
synthetic input mode, which still uses the production input dispatcher.
Replacement exec58874 completed successfully. Its Route36 movement CSV proves
`(30,8)` to `(32,8)`, three grid origins and zero additional terrain builds;
the paired source/native image is inspected and state-preserved. Median145ms,
p95 1056ms in this concurrently loaded debug run do **not** establish acceptable
performance. Native walk, orbit and night receipts use
`target/outside-art-pine-coverage-route36-*`.

Final pre-fence full mesh tests were **777 passed, one failed, six ignored**;
the failure was the terrain owner's mineral-wall/jump-terrace join test.
Two shared-grove cache tests passed. No claim of a fully green mesh suite or
completed outdoor art is warranted. Cherrygrove still needs source cyan roofs
and pink facades; signs, foliage craft, ground/water/light and regional coverage
remain part of the full goal.

## macOS continuation — 2026-10-13

### New route coverage, junction proof and pine coordination — 2026-10-14

This update supersedes the pending process notes in the older pine/junction
sections below. The earlier ledge continuation was verified progress.

Native build `18357` finished successfully (26m22s). Its four hour12 pairs
at Route38 `(6,10)`, Cherrygrove `(29,4)`, Route13 `(54,6)` and Route24 `(2,9)`
are validated, inspected and all report `state_preserved=true`. Receipts use
`outdoor-pine-{route38,cherrygrove}` and `outdoor-junction-{route13,route24}`.
The Route13 before/after comparison proves the `$78/$31` checker junction
strip is gone; the graded brick cap remains free of its former white patch.
Route24's native shoreline is continuous, but a separate `$2f`/`$37,$34`
checker patch and pale shallow-water rim remain. Do not call every strip fixed.

Three additional source/native diagnostic pairs are inspected at Route36
`(30,8)`, Route37 `(9,9)` and Azalea `(20,9)`, all hour12 and state-preserved.
They are under `target/outdoor-new-route-pine-baselines/`; batch session
46968 finished with exit0. Route36 hides the trainer beneath crowns and still
shows a flat Cut-tree drawing; Route37 retains flat flowers; Azalea's roof and
facade materials/scale do not match source. These are new coverage, not visual
acceptance. Fresh native Route36 fit review finished under session68754,
with `target/outdoor-route36-pine-fit*` receipts and preserved state. It is
inspected: the trainer remains mostly occluded/ghosted beneath foreground
crowns. The outside-art goal received this exact rejection fixture.

The widened pine source, upper shoots and staggered needle planes now pass
all12 foliage tests (`outdoor-pine-final-proportion-tests.log`) and the existing
tree plot/source-variant/footing regression. The old broadleaf vertex-count
slice was replaced by the actual 13×13 trunk; the original crown envelope and
footing assertions remain. A nonlinear envelope failed folded-needle winding
and was removed. Current affine fitting preserves the rooted center/base,
fits the unchanged plot margin, and limits interior height to a mature crown
proportion; normals use the corresponding inverse transpose. Native pine
appearance is still unaccepted. The active outside-art goal
`01a13a1d-d43e-7cc1-92cc-fc0770811ccd` received the files, test evidence,
occlusion findings and cache-ownership request and owns further pine finishing.

Rock continuation in `terraces.rs` now tests a sculpted wall shoulder whose
geometric normal is almost horizontal. The original source UVs have finite
relief thickness, so the first exact-collinearity guard failed that regression.
The current guard retains the dominant source-axis evidence without rejecting
the face by its tilted Y normal. The tilted fixture also extends beyond the
ledge's tangent end; its expected contact now uses the actual graded field,
not a constant six-pixel rise. Latest session6914 completed with **13 tests
passing** (`target/outdoor-rock-shoulder-field-tests.log`). Full mesh session
62553 completed: **780 passed, zero failures, six ignored** (145.83s,
`outdoor-rock-shoulder-full-mesh-tests.log`). The final native preview/audit
build is session27708
(`outdoor-rock-shoulder-final-native-build.log`). Fresh Route26 visual proof
remains pending. Build67698 succeeded (3m09s), but predates the last guard
refinement and is not final rock proof. After build completion, inspect
Route26 `(10,28)` at the same source state and other rock routes, then rerun
the relevant mesh/audit checks. Tiny Route26 slits remain unproven fixed.

### Johto pine implementation — 2026-10-14, visual acceptance pending

Outside-art goal continuation: the shared terrain owner finished pine source
handoff. Current proportional XZ/height fit about the planted stem preserves
the unchanged readable margin and uses inverse-transpose normals. Its final
focused log `target/outdoor-pine-final-proportion-tests.log` passes all **12
foliage tests**. The earlier radial/normal/envelope and clearance failures below
are diagnostic history, not the current passing focused result.

This goal now owns `target/agent-outside-art`; it uses a copy-on-write seed from
the completed cache, and does not run Cargo against `target/agent-outdoor`.
Full latest-source mesh/cache verification is live under exec **85667**, logs
`target/outside-art-pine-final-{mesh,shared-cache}-tests.log`. Latest native
build/capture is live under **54517**, log `target/outside-art-pine-final-native-build.log`.
The build uses voxel-view opt1 and crystal-bevy opt0 for local visual iteration.
Its Route38 `(6,10)` / Cherrygrove `(29,4)` paired captures use
`outside-art-pine-final-{route38,cherrygrove}` stems, hour12/orbit2/zoom1.
Do not call these accepted until their actual output is inspected.

The intermediate `outdoor-pine-{route38,cherrygrove}` and
`outdoor-pine-current-{route38,cherrygrove}` frames are inspected. Their source
receipts preserve state; fuller crowns improve on the initial poles, but flat
foliage planes, exposed leaders, actor occlusion and material/light finish still
need review. Cherrygrove's current source view has cyan roofs and pink walls
for both services; the native Center has a red roof and pale facade, an actual
source-palette defect to fix after pine verification. Small posts/fences,
signs and ground remain well below the approved reference.

The Route13 `(54,6)` and Route24 `(2,9)` `outdoor-junction-*` pairs are also
validated/state-preserved and inspected. Route24 still contains a raster-looking
gold/white junction strip; **do not claim complete junction acceptance**.
The actual pack audit `outdoor-junction-pack-audit.{json,log}` succeeds and
records 69/16 native shore cells respectively, with no unmodeled Route24 `$31`
cells. Thus a claimed cell alone does not establish removal of every visible
source face; investigate the exact strip/owner rather than broadening bank
matching blindly. No pack, scratch image or audit report was staged.

Current-source verification update: foliage job **52412** finished with
**13 passed / one failed**. The failure is the new mature canopy breadth
guard, not finite normals/winding or Park foliage. Current primary-limb reach
is consequently wider (`1.04 - 0.105 * level` plot spacings), keeping the
`0.60 * canopy height` breadth requirement. Breadth rerun PID **95192** is
confirmed live, logging to `target/outdoor-pine-breadth-tests.log`.

Full mesh job **7128** finished with **774 passed / one failed / six ignored**.
Its failure was `tree_variants_preserve_horizontal_plot_and_ground_anchor`,
which sliced the old broadleaf trunk-and-limb vertex count. The current test
checks the actual 13-by-13 pine trunk, a readable crown envelope, source-stable
variants and unchanged footing. Both shared-grove cache tests passed against
the compiled candidate in `target/outdoor-pine-shared-cache-tests.log` (16.91s).
Latest-source mesh/cache repeats wait on breadth PID 95192 under exec **95224**;
outputs will be `outdoor-pine-breadth-{mesh,shared-cache}-tests.log`. Poll this
existing job rather than compiling another test copy.

Native/capture **18357** is still live (Cargo 90952, Rust 94857/95057). Its
compile began before the latest primary-limb/test refinements. Inspect its
resulting pictures as an intermediate candidate, then rebuild current source
after tests pass before claiming final visual acceptance. Keep any shoreline
capture receipts separate from pine acceptance.

The initial native build's Route38 `(6,10)`, hour12/orbit2 source/native pair
is now captured, validated and inspected (98.94s/99.40s). Receipt reports
`state_preserved=true`: `target/outdoor-pine-route38-initial-build{,-2d}.png`,
with `.log` and `.paired-state.txt`. **Reject this appearance:** trees read as
tall bare poles with sparse combs; their crown-to-height ratio is much too
narrow for the saved reference. Later indexing/orientation work alone cannot
correct that structural defect. This image is an initial-build diagnostic,
not acceptance of the current source.

The current pine source consequently widens primary limbs, secondary sprays
and folded needle blades, varies warm lit tips against a blue-green interior,
and adds a minimum mature canopy breadth regression. Trunks still occupy
their blocked source plots, while elevated limbs and crowns can overlap like
the previous broadleaf crowns. The earlier all-wood-inside-plot clamp was an
implementation choice that made the canopy skeletal; it is removed. Collision,
ground ownership, understory and cutaway paths remain unchanged. The existing
queued tests/capture job below will consume this corrected source; inspect
their actual outcome instead of claiming the initial render proves the fix.

This continuation makes implementation progress beyond the completed bridge
edge review. `foliage/pine.rs` now generates ordinary Johto/Johto-modern groves
through the existing `append_grove_tree` boundary. Both streamed recipes and
the modeled-exterior path use it. The superseded procedural broadleaf grove,
woodland shell/leaf generator and its obsolete silhouette tests are removed.
There is no alternate old Johto crown option or duplicated replacement asset.
The external tree catalog remains actively required by the separate Ilex/
Battle Tower forest path; Kanto, Park, fruit and Cut families remain separate.

Pines have source-seeded tapering/fluted warm trunks, unequal staggered limbs,
supporting twigs and folded indexed needle sprays. Trunks are bounded to
blocked source plots; elevated limbs and needles overlap grove crowns. The same
source coordinates, thinning mask, understory, shared preparation, ownership
and complete cutaway ranges are retained. Park's helper now has only its
original garden-leaf branch; its numeric leaf generation is preserved.

The expanded shoreline test job **64010 completed with exit 0**, three tests
passing. Pine verification is **pending**: foliage tests under exec **52412**,
full mesh tests under **7128**, using `target/agent-outdoor` and test-package
optimization zero. Logs are `target/outdoor-pine-tests.log` and
`target/outdoor-pine-full-mesh-tests.log`. The initial native build **32918**
completed with exit 0 in 21m21s. Later needle/index/twig refinements require the final build;
do not use that initial binary as proof of the latest pine source.

The next continuation also removes the unused woodland-shell color branch,
leaving Park's existing leaf colors unchanged. At this checkpoint Cargo PID
88506 owns the cache and is compiling the native preview/audit tools; log
`target/outdoor-shore-junction-native-build-isolated.log`. Its build-script
phase completed and Rust compilation is active. Pine tests **52412**, mesh
tests **7128**, and final build/capture **18357** remain live and queued behind
that owner. Do not restart them from a quiet log or a missing screenshot.

Final build plus four muted matching-state production pairs is queued under
exec **18357**. Build log: `target/outdoor-pine-native-build.log`. Captures:
Route38 `(6,10)` and Cherrygrove `(29,4)` use `outdoor-pine-{route38,cherrygrove}`;
Route13 `(54,6)` and Route24 `(2,9)` use `outdoor-junction-{route13,route24}`.
All use `core-modular.browser.crystalpack`, hour12, orbit2, zoom1, 640x480,
and paired source views. Each pair has its own `.log`/`.paired-state.txt`.
Poll these existing jobs, fix any geometry failures, inspect actual resulting
images, then review other orbits/movement and morning/night. Pine appearance,
performance and whole-scene quality are not yet accepted. No pack or build
artifact is a deliverable, and nothing was staged.

### Bridge perimeter review and junction-bank continuation — 2026-10-14

The preceding goal turn made verified brick-grade progress. The pending
Route13/Route24 capture process (PID 83591) is now terminal and all **15 distinct
hour12 perimeter pairs** have been inspected. Every receipt reports
`state_preserved=true`. Route13: NW `(2,2)`, N `(30,2)`, NE `(51,2)`, E/SE
`(51,7)`, S `(30,15)`, SW `(2,15)`, W `(2,9)`. Route24: NW `(2,2)`, N `(11,2)`,
NE `(17,2)`, E `(13,9)`, SE `(13,15)`, S `(11,15)`, SW `(2,15)`, W `(2,9)`.
Artifacts and final log remain in `target/outdoor-native-brick-new-perimeters*`.
The previous exec handle 48149 is unavailable; do not restart this completed run.

These pairs confirm clean brick caps without the rejected mortar breakthrough.
They expose distinct raster bank strips under junctions (especially Route13
NE and Route24 W), large pale boundary stones, unfinished post rows, repeated
pale broadleaf crowns, and overly plain ground. Whole-scene acceptance fails.
All eight saved concept images and their art standard have now been inspected;
ordinary Johto pine replacement, regional exceptions, finished small props,
materials and lighting remain requirements. No tree replacement or old-tree
asset retention decision was made in this continuation.

The production shoreline matcher now includes exact Kanto `$78`: top-row `$31`
bank ink with three `$14` water rows. Its own authenticated four-color ink
joins the existing mineral bank finish; bridge bricks remain a separate owner.
The guards now test this layout as well as the original pond for changed art,
phase, ownership, height, borrowed UVs, alpha, water shape and cropped inputs.
Day/night pigment tests now cover both pond and junction bank geometry.

Verification of the expanded guards and production visuals is **pending**.
The inherited optimized shore-test compilation (PID 84323 / rustc 84339)
completed successfully: three shore tests pass. It was confirmed live before
completion and was not restarted on the unavailable observation handle.
The current-source shore-test command is compiling under exec session
**64010** (PID 86597 / rustc 87218), and the native
location-preview build is queued under **32918** (PID 86635), both using
`target/agent-outdoor`.
Logs: `target/outdoor-shore-junction-tests.log` and
`target/outdoor-shore-junction-native-build.log`. Poll these handles and inspect
the results. After build completion, capture fresh matching-state Route13
`(54,6)` and Route24 `(2,9)` pairs, then review morning/night junction banks.
Do not treat the older perimeter renders as proof of the newly extended bank.

### Brick-deck continuation: grade artifact verified fixed

The prior shoreline turn was verified progress. The current paving extension
adds exact Kanto `$05`/all-`$4b` decks to `mesh/kanto_paving.rs`, using the live
two-color brick art, staggered beveled bricks and original shallow water-side
faces. Original LADDER permissions, water level and footing arrays remain
unchanged. Half-bricks clip across source-cell edges without an artificial
seam or changed pigment. Altered art, phase, claims and unknown geometry retain
source. The bridge family has its own `outdoor/kanto-brick-bridge` receipt.

The first bridge pass compiles and passes all six paving tests plus the then
current **769-test** mesh suite, zero failures/six ignored. Its native build
passes in 4m38s. The new audit verifies 77 maps/2,003 descending jumps, zero
errors, and 1,360 bridge cells across Celadon16, Cerulean32, Route12 768,
Route13 160, Route20 64, Route21 64, Route24 144 and Route25 112
(`target/outdoor-native-brick-audit.json`). Four native/source hour12 pairs at
Route24 `(8,10)`, Route13 `(54,6)`, Route12 `(4,34)` and Cerulean `(20,1)` are
validated and state-preserved. These are checked native LADDER positions.
Production Route24 `D` records `(8,10)->(8,11)->(8,12)->(8,13)->(8,14)`;
the final pair preserves state. Route13 hour7/22 pairs also preserve state.
Files use `outdoor-*-native-brick*` and `outdoor-route13-brick-{morning,night}*`.

**The first visual pass was rejected.** Route13 contained white mortar
triangles leaking through the brick caps on graded ground. `terrace_levels`
interpolates separately across each source cell's NW-SE diagonal; an unsplit
brick triangle crossing that crease can sink below its recessed bedding.
The current source now splits clipped brick faces into those same two planar
domains. A new regression applies a strong folded source-cell grade and
requires every brick triangle to remain above its bedding. Exact source-cell
edge position/pigment/normal equality still has its own passing regression.

The grade-fix test process **3518 completed successfully**: all seven paving
tests pass (`target/outdoor-brick-grade-tests.log`). The corrected full mesh
suite also passes **769 tests, zero failures/six ignored**
(`target/outdoor-brick-grade-full-mesh-tests.log`). Native rebuild **14940**
completed successfully in 12m13s after rebuilding shared Bevy graphics
dependencies (`target/outdoor-brick-grade-native-build.log`). Fresh Route13
`(54,6)` hour12 native/source pair is validated, state-preserved and inspected
(`target/outdoor-route13-brick-grade-fixed{,-2d}.png`): the previously visible
white mortar breakthrough is gone. The rejected crop was compared directly
with this corrected view. This establishes the graded brick component fix.
Preserve all concurrent seating, Safari-office and gift-shop edits.

Route13/Route24 edge capture **48149 completed**. All **15 distinct paired
views** in `target/outdoor-native-brick-new-perimeters/` are validated,
state-preserved and inspected at hour12:

- Route13: NW `(2,2)`, N `(30,2)`, NE `(51,2)`, E/SE `(51,7)`, S `(30,15)`,
  SW `(2,15)`, W `(2,9)`. East/southeast collapse to one nearest walkable tile.
- Route24: NW `(2,2)`, N `(11,2)`, NE `(17,2)`, E `(13,9)`, SE `(13,15)`,
  S `(11,15)`, SW `(2,15)`, W `(2,9)`.

The remaining raster strip under distant bridge junctions is a separate
shoreline variant, Kanto `$78` with top-row `$31` bank ink and three `$14`
water rows. The current `kanto_shore` source adds this exact layout/art to
the guarded mineral finish. All three shoreline tests now cover ten layouts
and pass; the current rebuilt test binary's full mesh suite passes **774 tests,
zero failures/six ignored** (`outdoor-shore-junction-full-mesh-tests.log`).
The original focused command **23252 exited successfully**. Another invocation
used the same target/log while it was running and truncated the log; the final
test report remained visible after filtering NULs. Treat the command's confirmed
exit0 and the separate full-suite receipt as authoritative. Do not reuse that
contended log for another run.

Native junction rebuild is now queued/live under exec session **89054**, with
its distinct log `target/outdoor-shore-junction-native-build-isolated.log`.
Poll this handle instead of restarting on timeout. After success, capture the
Route13 junction again and verify `$31` no longer retains raster art, then
update this checkpoint. Preserve all concurrent source/cache owners; a second
test at PID86597 was observed using the same outdoor build directory, and the
interior owner confirmed it is not theirs. No process/cache was deleted or
stopped. Whole exterior acceptance remains open: repeated/faceted stones and
gray shafts, source-brown post materials, pale trees and global night grading
still need work. The source post asset's named `cool stone shaft` and `shaft
facet` primitives confirm that the observed gray/brown mismatch is authored
material, not merely a lighting impression.

### Native Kanto pond and coast bands — 2026-10-14

The preceding turn made verified paving/edge progress. This continuation
replaces Kanto's retained `$33/$54` gold shoreline checker artwork with native
mineral caps and the existing shallow water-drop faces. `mesh/kanto_shore.rs`
accepts complete `$1d/$1e/$1f/$2d/$2e/$64/$65/$67/$6a` drawings, coherent source
phase, authenticated opaque bank art, original zero-height caps and exact
water-neighbor faces. It preserves unknown/cropped/changed/claimed/elevated
drawings, unrelated rock owners, water surfaces and the original footing array.
No collision, actor, RTC or gameplay dispatch changes occur. `append_coast`
reuses the existing mineral wall relief and shared height rings with live bank
pigment and a wetter lower face; it adds no meadow moss. Existing canyon/stone
pigment algorithms remain unchanged.

All three new shoreline regressions pass: nine layouts, all four cardinal
water drops, outward winding, exact footing preservation, other-owner retention,
changed-source rejection and live palette changes. The full renderer mesh suite
passes **764 tests, zero failures, six ignored** (`outdoor-kanto-shore-full-mesh-tests.log`).
Native build passes in 2m48s (`outdoor-kanto-shore-native-build.log`). The fresh
audit verifies **77 maps / 2,003 descending production jumps, zero errors** and
**1,500 native shore cells in 22 maps** (`outdoor-kanto-native-shore-audit.json`).
The native finish is in the shared production mesh path, not a preview override.

Five hour12 native/source pairs are validated, state-preserved and inspected:
FuchsiaCity `(30,18)`, Route12 `(7,34)`, Route17 `(8,16)`, ViridianCity `(11,23)`
and VermilionCity `(26,14)` (`outdoor-{fuchsia,route12,route17}-native-shore*`,
`outdoor-{viridian,vermilion}-native-pond*`). These positions were checked against
the source collision table and are ordinary FLOOR; they are scenery poses,
not new movement-mechanics acceptance tests. The initial Fuchsia/Route12
post-cell poses were discarded and their scratch captures removed. Valid
pre-change pairs at `(30,18)/(7,34)` clearly show the old raster banks
(`outdoor-fuchsia-pond-before-valid*`, `outdoor-route12-shore-before-valid*`).
The final bank caps/water faces have continuous warm mineral paint, no retained
checker stripe and seated water contact.

All **15 distinct new perimeter pairs** are validated, state-preserved and
inspected at hour12 in `target/outdoor-native-shore-new-perimeters/`:

- Fuchsia: NW `(2,2)`, N `(20,4)`, NE `(33,2)`, E `(37,18)`, SE `(36,33)`,
  S `(20,32)`, SW `(2,32)`, W `(2,19)`.
- Route12: NW `(2,2)`, N `(9,1)`, NE `(17,0)`, E `(15,32)`, SE/S `(6,51)`,
  SW `(2,51)`, W `(2,26)`. South and southeast choose the same nearest walkable
  tile; the production capture tool deliberately deduplicates that position.

Fuchsia `(30,18)` hour7/22 pairs are also validated, state-preserved and
inspected (`outdoor-fuchsia-native-shore-{morning,night}{,-2d}.png`). The live
bank pigment changes with time and the shallow geometry remains stable.
Morning is warm/yellow. Whole nighttime fidelity still fails: ground is too
cobalt-blue, native water is much brighter/bluer than the 2D navy, and NPCs
lose readability. The already documented global lighting investigation remains
open; no archived task was reopened.

New edge evidence exposes extensive retained gold brick bridge/forecourt
surfaces (especially Route12), overly large/boxy mineral plateaus, repeated
gray/faceted posts versus source brown shafts, building scale/occlusion and pale
repeated trees. Flower visibility is inconsistent: some fresh Fuchsia day views
show native flowers, but other views/times retain source roses with no native
flowers. Do not attribute shared/phase-dependent flower changes to this shore
adapter. Trace the exact art/animation provenance before claiming flower closure.
The full exterior objective remains active; this is shoreline component progress.

### Native Kanto walkable paving — 2026-10-14

The previous goal turn verified the global ledge direction rule. This
continuation replaces the remaining flat road/curb drawings with native
beveled flags and rounded junction inlays in `mesh/kanto_paving.rs`. The nine
complete drawings `$22/$23/$26/$27/$44/$45/$48/$49/$5d` all have ordinary
FLOOR collision. They gain no barriers or positive walking rise: the original
footing array is unchanged, cap tops stay at zero, and joints recess below it.
All sixteen cells must agree on native block, phase, art, original zero-height
floor geometry and unclaimed ownership. Unknown/cropped/changed drawings,
borrowed floors, transparent art and custom elevated surfaces retain source.
Paint comes from authenticated live source RGBA, including time-of-day colors.

The initial Route11 pair exposed excessive grid frequency and dark square
backings around gray junction centers. That view was rejected. The refined
finish uses larger, quieter flags and pale junction centers with gray rims.
Its chamfers bake the existing exterior kit's directional shade because the
production terrain material is unlit; normals alone did not show that volume.
The final Route11 `(10,7)` hour12 pair is validated, state-preserved and
inspected (`target/outdoor-route11-paving-refined{,-2d}.png`). The raster curb
stripe is replaced and the dark corner backing is gone. Dense grass and pale
repeated trees remain outside whole-route acceptance.

All three new paving regressions pass, including nine native drawing variants,
altered-source rejection, outward floor winding, unchanged footing and live
palette changes. The refined full mesh suite passes **761 tests, zero failures,
six ignored** (`outdoor-kanto-paving-refined-full-mesh-tests.log`); its native
build passes in 2m59s (`outdoor-kanto-paving-refined-native-build.log`). The
outdoor audit covers **77 maps / 2,003 descending production jumps, zero errors**
(`outdoor-kanto-paving-audit.json`). Its source-guarded paving coverage is
4,592 cells: Cerulean32, Route11 928, Route17 1,584, Route5 128, Route8 480,
Saffron1,376 and Vermilion64. That audit precedes the purely decorative flag/
pigment refinement; the refined regressions preserve identical footing.

Final Saffron `(18,7)` and Route17 `(8,16)` pairs are validated, inspected and
state-preserved (`outdoor-{saffron,route17}-paving-refined*`). Route11 `(11,7)`
production `R` records `(11,7)->(12,7)->(13,7)->(14,7)->(15,7)`, crossing the
curb into grass; the final native/source pair is also state-preserved and
inspected (`outdoor-route11-paving-walk*`). No collision or input changes were
needed. Route17 is a scenery pose, not a cycling-mechanics acceptance test.

All **16 new perimeter pairs** are validated, state-preserved and inspected in
`target/outdoor-native-paving-new-perimeters/` at hour12:

- Route11: NW `(4,2)`, N `(20,2)`, NE `(34,2)`, E `(37,8)`, SE `(33,15)`,
  S `(20,15)`, SW `(4,15)`, W `(2,9)`.
- Saffron: NW `(2,2)`, N `(20,4)`, NE `(37,4)`, E `(37,18)`, SE `(37,33)`,
  S `(19,33)`, SW `(1,32)`, W `(2,18)`.

The Route11 hour7/22 pairs are also validated, state-preserved and inspected
(`outdoor-route11-paving-{morning,night}{,-2d}.png`). Morning paving follows
the source yellow temperature. Night is still strongly cobalt/royal blue
versus the source lavender, with low trainer/NPC readability. The lighting
goal is now archived: its evidence notice failed and it was not reopened.
Next lighting investigation should inspect `day_cycle.rs`'s night tint
`[0.40,0.55,2.0]` and `voxel_surface.wesl`'s multiplication of the already
time-colored source surfaces by that normalized tint. This is a diagnosis
to investigate, not a verified global grading correction.

Outstanding visual defects remain: oversized/occluding Saffron buildings,
long yellow brick strips retained as flat surfaces alongside east/west
buildings (trace roof/ground ownership before replacing), repeated gray posts,
flat checker plazas, pale repeated trees and Route17's gold shoreline ink.
Route11's western mineral plateau remains boxy. These views establish paving
component progress, not whole-map acceptance. Keep the full exterior goal active.

### Ledge direction clarification and verification

The human explicitly extended the rule to east/west: the side from which a
permitted jump begins is higher; the landing side, which blocks the reverse
jump, is lower. Direction comes from production HOP permissions, including
corners, rather than a fixed south-only visual assumption. Terrain and actor
support share the same directed terrace field.

The rerun `target/outdoor-ledges-direction-confirmation.json` verifies all
**2,457 production jumps across 72 maps**, with zero mesher or non-descending
jump errors. Route6 south and Route38 east source/native pairs were inspected
again. This confirms the height rule for static pack maps; map connections and
script-mutated variants retain the outstanding coverage noted below.
The freshly compiled directed-terrace suite passes **13 tests, zero failures**
(`target/outdoor-ledges-direction-confirmation.log`), including all four
directions, repeated south drops, opposing eastward courses and shared corners.

### Latest continuation: Kanto stone volume and new perimeter coverage

The prior goal turn is verified progress: the shared ledge rule passes all
2,457 production jumps in 72 maps and 2,003 jumps in 77 outdoors. The full
exterior objective remains active; those height receipts do not establish
all scenery fidelity.

The three exact `kanto_boundary_land/shore/path` model names now receive
cached, position-welded stone fairing and area-weighted smooth normals in Rust.
No triangles are added. The seated underside/perimeter, fit footprint, original
source drawing guards, source underlays and footing are preserved. The six
existing boundary tests pass, including closed outward volume and the
under-500-triangle budget. A new regression requires actual silhouette change,
unchanged seated positions and identical normals across material seams.
Initial native build passed (2m34s, `outdoor-rounded-boundary-native-build.log`).
This does not change Route26 mineral/grove finishing or other model families.

All eight PewterCity and all eight Route15 perimeter pairs are validated and
inspected at hour12 in `target/outdoor-rounded-stone-new-perimeters/`. All
16 paired-state receipts preserve source/native state. Actual locations:

- Pewter: NW `(2,2)`, N `(20,0)`, NE `(34,4)`, E `(37,18)`, SE `(37,33)`,
  S `(19,33)`, SW `(2,33)`, W `(2,18)`.
- Route15: NW `(2,1)`, N `(20,2)`, NE `(35,2)`, E `(37,9)`, SE `(37,15)`,
  S `(20,15)`, SW `(2,15)`, W `(4,9)`.

The first close stone views still exposed triangular pigment patches at the
caps. Original primitive color bands had separate colors at coincident
vertices despite smooth normals. Those pigments now blend by triangle area
at each welded point, preserving each stone family's palette. The strengthened
seam regression requires both matching normals and matching colors. All
24 selected boundary checks pass; current full mesh suite is **757 passed,
zero failures, six ignored** (`outdoor-rounded-pigment-full-mesh-tests.log`).
The fresh pigment build passes (4m42s, `outdoor-rounded-pigment-native-build.log`).
Pewter `(2,18)` hour12 close native/source pair is validated and inspected
(`outdoor-pewter-stone-pigment-fixed{,-2d}.png`): triangular cap patches are
gone, with rounded volume and seated contact. Route15 `(20,2)` and Route19 `(10,11)` hour12 close pairs are also validated
and inspected (`outdoor-route15-stone-pigment-fixed*`,
`outdoor-route19-shore-stone-pigment-fixed*`). All three receipts preserve state.
The land, paving and shoreline variants now show rounded volume with smooth
cap shading and seated contact. The Route19 explicit cliff-cell fixture is
scenery evidence, not proof of normal walkable footing. Its flat cliff ink,
wrong generic green/sand edging, unmodeled floor surfaces and rigid scenery
placement remain defects. The stones still repeat identical silhouettes and
separate capped posts remain angular. Do not generalize the verified stone
component to whole-map visual acceptance.

New edge evidence also exposes flat mint/checker plaza paving, retained brick
road surfaces, sparse flower beds, pale repetitive ordinary trees, roof/body
scale mismatches and northern building occlusion. Separate Kanto capped posts
still have angular caps; they are not one of the three smoothed stone families.
The tree goal is archived; a coordination notice returned archived and it was
not reopened. Preserve its existing work. Terrain deformation currently
recalculates normals per indexed triangle on nonuniformly displaced surfaces;
this may undo shared normals or deform rigid scenery on graded terrain and
needs a targeted final-geometry check if the new material view remains faceted.

### Current continuation: directed terraces, Route8 and city edges

The previous goal turn and this continuation made concrete code, test and
paired-render progress. The full exterior goal remains active. **The human's
latest rule applies across the app:** permitted jumps start on higher ground
and land lower; reverse blocked approaches are on the lower side. Keep the
approved Route26 mineral/grove finish. Do not return to isolated raised ledge
strips or an uphill toe before each south-facing jump.

Production publishes full active-map/connection `VisualWorldFrame.jump_ledges`
from the existing collision catalog, independently of camera crops. The shared
constraint field keeps successive drops distinct, shares one upper level at
corners, and supplies terrain, actor support, camera height and grove seats.
Cache keys/reuse include this metadata. Runtime collision and input ownership
are unchanged. Mineral walls retain their authored cap height while their
feet join the adjusted neighboring-bank ground; this closes the upper/lower
seams found during Route4 review.

The remaining triangular corner gap was caused by face endpoint queries
sampling past a perpendicular adjoining course into its lower landing. The
face now samples from inside its own unit span at both endpoints. The new
actual-face regression requires both joined faces to meet their upper cap.
All **six** terrace tests pass. The latest full renderer mesh suite passes
**728 tests, zero failures, six ignored**
(`target/outdoor-route8-entrance-mesh-tests.log`), including the former transfer
rock-winding failure. The native build passes (1m08s,
`target/outdoor-route8-entrance-build.log`). Final current-source outdoor audit
passes **77/77 maps, no errors**
(`target/outdoor-corner-and-route8-all-maps.{json,log}`, ignored verification).
Earlier support/cache checks passed 12 camera tests and four cache tests.

All named captures below are in ignored `target/`.
Native/source pairs have been validated and inspected at the same position,
time, facing and game state:

- Route6 `(10,2)`, hour12: northern grass, middle strip and southern path
  descend in order (`outdoor-route6-terraces{,-2d}.png`). Production key QA
  `DD` crossed `(10,2)->(10,4)` then walked to `(10,9)`; the landing pair is
  inspected (`outdoor-route6-terraces-walk-south*`).
- Route4 `(20,2)`, hour12: mineral top/foot joins are closed
  (`outdoor-route4-terraces-cliff-contact-fixed{,-2d}.png`). The final legal
  westward QA starts at `(21,2)` and uses `LL`; its lower landing pair
  (`outdoor-route4-corner-sealed{,-2d}.png`) confirms the corner sky gap is
  closed. A prior valid `L` trace recorded `(21,2)->(19,2)->(17,2)->(16,2)`.
  The newer fast 32-frame `L` attempt recorded authoritative movement but
  failed the visual-motion assertion before jump presentation had settled;
  `LL` passes. No QA assertion was weakened. This timing finding was sent to
  the interiors/capture owner. Starting directly on the hop tile `(20,2)`
  was also an invalid movement fixture.
- Route8 `(10,6)`, hour12: alternate underground entrance
  `[0c,0e;10,66]` now binds the existing Kanto house, using the source doorway,
  live cyan-blue roof swatches and single-house scale
  (`outdoor-route8-entrance-native{,-2d}.png`). Exact complete drawing/door
  regression covers both this roof and Route7 `[38,39;10,66]`; changed facade
  is refused. Roof/body detail still needs broad fidelity review.
- Route8's original center daytime/morning7/night22 pairs are inspected
  (`outdoor-route8-terraces*`, `outdoor-route8-hour{7,22}{,-2d}.png`). Morning
  warms yellow; night becomes darker/bluer. Remaining night issues: royal-blue
  retained borders versus lavender source, muted gray-green native grass versus
  source blue-cyan, poor trainer readability. Lighting owner has the evidence.
- Route26 fresh daytime preservation pair is inspected
  (`outdoor-route26-terraces-preservation*`): approved canyon, rounded edges,
  grove composition and ground material survive. Keep those finishes. Its
  remaining cabin roof is brown against blue source.

Route8's long gray outlined strips were initially called fences. Source
collision shows the involved complete Kanto road blocks are **all FLOOR**
(`26/27/44/45/48/49/5d`, also horizontal road family `22/23`). Their native
replacement must preserve the walkable paving/low-border behavior, not add a
blocking wall. Source tiles `10` and `20` are vertical/horizontal shaded border
strips, `21` a round junction cap, `23` plain paving. No paving implementation
has been added yet. Use source-complete phase/art guards and native low edging;
keep source footprint and footing, and inspect actual walk-through afterward.
The round barrier rocks remain overly angular and ordinary Kanto trees pale.

CeruleanCity's eight-position perimeter is now validated and all eight pairs
inspected (`target/outdoor-cerulean-perimeter-native-first/`). Actual positions:
NW `(2,2)`, N `(20,2)`, NE `(39,4)`, E `(37,18)`, SE `(37,33)`, S `(20,33)`,
SW `(2,33)`, W `(2,18)`, hour12. All eight state receipts say preserved, and
all source/native images passed nonuniform-frame validation. Defects exposed:
Nugget Bridge still uses flat source brick paving; ordinary grove trees are
pale/repetitive against saturated source greens; round boundary posts are
angular boulders; flower/garden strips are sparse; town paths retain flat
mint/checker source treatments; Mart roof is blue against gray source.
The west Surf view also confirms the missing rider, reported to the actors
owner with the precise comparison path. These are defects, not whole-city
acceptance.

The first source-first perimeter attempt was terminal with one blank 2D image
(saved0.49s), despite a valid state receipt, while native rendered later.
It supplies no visual acceptance. `render_at_location` now warms/captures
native first for `--view both`, then takes the frozen-state source view. Logical
file names and comparison panels stay 2D reference left/native right; existing
state and pixel guards remain. Native-first build passed (3m21s), and the
successful eight-pair batch proves this path. The exact cause of the uniform
startup source frame was not established; do not claim a GPU diagnosis.

Current next verification: KantoFlatHouse and KantoMart now join the existing
live-source roof repaint. Their flat roof art (`4c..4e`, `5a..5f`) is included
alongside the pitched roof courses; all named roof materials retain weather
and facet shading while geometry and non-roof paint remain fixed. The first
build began before that art-family addition and was deliberately stopped
(exit130) so it cannot serve as a current receipt. Expanded roof-model tests
and a fresh build/native Cerulean Mart + Route6 gray-roof proof are required.
The first test compile hit the lighting owner's in-progress Dark enum addition
(`sky.rs` missing exhaustive arm). Its owner fixed that match. The current
expanded roof-model test now passes (17m03s receipt,
`target/outdoor-flat-mart-roof-palette-tests.log`), covering KantoHouse,
KantoFlatHouse, KantoMart and KantoBlank with unchanged geometry/non-roof paint.
The fresh current native build passed (7m22s,
`target/outdoor-flat-mart-roof-current-build.log`). Cerulean Mart `(25,30)`
and Route6 `(10,2)`, hour12 native/source pairs are validated and inspected:
`target/outdoor-cerulean-mart-gray-roof{,-2d}.png` and
`target/outdoor-route6-gray-roofs-native{,-2d}.png`. Both formerly blue roof
families are now neutral gray matching source, with unchanged source plots,
non-roof materials and descending ledge geometry. This establishes the roof
hue correction, not full architectural/area fidelity. The Cerulean close view
also exposes a retained flat NPC sprite behind the buildings and large,
angular boundary stones; keep these as concrete remaining defects.

The shared-grove regression is repaired without weakening its exact triangle,
normal, UV or color checks. The former four-variant/density cache discarded
source coordinates and directional neighbors. `shared_groves` now keeps a
bounded 64-entry cache keyed by exact placement, base/tile size, full source
coordinates and all four neighbors. Shared CPU/BVH/upload storage and independent
fades remain. Empty-ledge frames again run the completed detached-grove bank
finishing pass before preparation. The strict legacy grove regression and the
new exact-source/cache-hit regression pass. This restores source geometry;
initial preparation cost is greater than the former four-template shortcut.

Mineral wall ownership now comes from original mineral UV coordinates and
geometric triangle orientation, rather than sculpted positions or smoothed
shading normals. Those can cross the source-cell boundary or point upward.
The regression explicitly exercises both cases. All six directed-terrace tests
pass, including east/west, successive drops, corners, support and mineral-wall
contacts (`target/outdoor-mineral-geometric-owner-tests.log`). The current full
mesh run is **735 passed, zero failed, six ignored**
(`target/outdoor-geometric-mineral-full-mesh-tests.log`). The prior cave aperture
and gym neighbor-preservation failures have been repaired by their owners.

ViridianCity now has eight further validated and inspected perimeter pairs in
`target/outdoor-viridian-standable-perimeter/`: NW `(2,2)`, N `(19,2)`,
NE `(35,2)`, E `(35,18)`, SE `(37,33)`, S `(20,33)`, SW `(2,33)`, W `(2,18)`.
All eight state receipts preserve the same source/native game state. The
capture picker now rejects blocked terrain, warp and jump cells using actual
collision permissions; merely constructing a session can admit blocked cells
through a connection exception. Production movement is unchanged. Earlier
perimeter captures remain landscape evidence, but blocked starting positions
must not be treated as proof of normal player occlusion. Viridian still has
flat mint/checker paving, missing flower detail, pale repeated tree crowns,
angular stone posts and fence posts distorted along the bank. These remain
visual defects; city-wide acceptance is pending.

The current native build passes (3m53s,
`target/outdoor-geometric-mineral-native-rebuild.log`). The fresh Route26
`(10,28)` pair is validated and inspected
(`outdoor-route26-geometric-mineral-owner{,-2d}.png`); its receipt preserves
state. Most mineral contact holes are closed, but tiny upper-right white slits
remain. Do not claim full closure from passing structural tests. Shared camera
changes give a wider composition than the preceding pair; this cliff fixture
also does not establish preservation of the approved road/grove composition.

The new whole-map ledge audit now uses production `attempt_ledge_jump` and
`resolved_footing_height` at the exact sprite-foot point. The initial adjacent
ink-cell check was inappropriate for folded front courses and is superseded.
The first production-position run checked 2,003 jumps across 77 maps and found
10 non-descending samples in Route38, RuinsOfAlphOutside and SilverCaveOutside.
The refined sprite-foot diagnostic retained nine failures in those same maps
(`target/outdoor-jump-source-diagnostics.{json,log}`). These exposed a real
solver ordering problem, rather than just a drawing interpretation issue.

`terrace_levels` now joins equal-side course segments before normal rays,
reserves the nearby takeoff/landing ground through the sprite-foot ending
boundary, then protects tangent-end ground before remote flat continuation.
Previously remote courses could consume another jump's local footing or lift
its landing through corner interpolation. The broader `--jump-maps` audit
reached 72 maps / 2,457 production jumps, including forest and dungeon maps;
its pre-final-boundary snapshot found further TinTower3F/5F cases. Direct
execution of the repaired Levels source now descends at every known failing
point, including Route45 and both tower layouts. Focused regressions cover
opposing Route38 courses, Silver Cave local takeoffs, a minimized Route45
landing-boundary case and a minimized short eastward course ending at the
sprite foot. Forest `$1a` caps now remove their old local rise before applying
the shared field, preventing a doubled ledge height.

Johto `$4c`–`$4f` east/west drawings and `$50`–`$53` corners now fold proven
side ink onto west/east faces instead of retaining it flat. Exact cell art
identity guards unknown drawings. The side/corner verge retains lawn paint
when the cap uses path paint, and `$4c/$4d` now participate in the existing
native shallow-bank finish. The strict grassy-verge preservation test initially
caught borrowed path paint, then passed after the material correction; no
assertion was weakened. The source-guard test also passes.

The standing-contact renderer mesh suite is **751 passed, zero failures, six
ignored** (`target/outdoor-standing-contact-final-mesh-tests.log`). Its fresh
native build passes (1m36s,
`target/outdoor-standing-contact-native-build.log`). Final production-footing
verification is green:

- All **2,457 production jumps in 72 maps** descend, zero errors, including
  forest, cave and tower maps (`outdoor-standing-contact-all-jump-maps.{json,log}`).
- All **2,003 production jumps in 77 outdoor maps** descend, zero errors
  (`outdoor-standing-contact-outdoor-maps.{json,log}`).

The two-pixel standing footprint is anchored to its course level. Its floor
vertex uses that anchor on the connected side of authored cuts; blocked
neighboring pixels are not forced into the same affine level. A minimized
right/right/down chain regression preserves intermediate contacts where one
jump's landing is the next jump's takeoff. This fixes the remaining tower
cases without changing gameplay permissions or authoritative input/state.
These checks cover static pack maps; continuity across map connections and
script-mutated map variants still needs dedicated movement/visual coverage.

Route38 `(6,10)` production `R` records `(6,10)->(8,10)->(9,10)`. Paired native/
source takeoff/landing images are validated and inspected
(`outdoor-route38-east-{takeoff,landing}*`): the path is higher than the grassy
landing and the trainer stays visible. The fresh side-profile/material pair
(`outdoor-route38-east-side-face-fixed{,-2d}.png`) removes the flat yellow/black
side stripe while keeping its grassy rim. Dense grass, overlarge trees and
angular posts remain below whole-route fidelity acceptance.

Fresh TinTower5F `(8,12)` production `R` records `(8,12)->(10,12)` and the pair
is state-preserved, validated and inspected
(`outdoor-tin-tower5-jump-contact-fixed{,-2d}.png`). The trainer is visible on
the resulting contact. The room remains visually unaccepted: heavily retained
pixel floor art, distorted floors/walls and generic grassy drop supports.
The interiors owner received exact evidence and global-height receipts.
Proven tower bamboo-face source identities now select a warm timber support
instead of outdoor turf. Its combined mesh suite passes **753 tests, zero
failures, six ignored** (`outdoor-timber-drop-final-mesh-tests.log`), including
no-turf color and unchanged six-to-zero drop geometry. Native build passes (1m42s, `outdoor-timber-drop-native-build.log`); the
final timber material pair is validated and inspected
(`outdoor-tin-tower5-timber-drop-fixed{,-2d}.png`, state preserved): outdoor
grass is gone from the wooden supports. Floor/wall distortion and retained
pixel floor art remain unaccepted.
Do not claim whole-tower fidelity from the passing height audit.

A fresh pre-order-repair SilverCaveOutside `(24,26)` `D` native/2D capture is
validated and inspected (`outdoor-silver-cave-production-jump*`), and its log
queues the production jump sound. The native player is occluded by the
oversized pale mountain wall, so this view is not accepted as visible jump
motion evidence. Flat shoreline source pixels, pale grove crowns and the white
mountain mass are further concrete defects from this new location. Shared
in-progress compile failures in `grove_stream`, Model metadata and
TerrainBatchCoverage were resolved by their owners; preserve those changes.

Two additional walkable source/native route pairs are validated and inspected
in `target/outdoor-new-walkable-route-coverage/`: Route26 `(10,54)` and Route11
`(20,9)`, hour12. Both state receipts preserve the game state. The Route26 road
view retains mineral/grove materials and keeps the trainer visible, although
its cabin roof remains red-brown versus cyan source and prop/tree scale still
needs review. Route11 is new coverage: tall dense grass, pale repeated ordinary
trees and flat white/gray road borders remain. The border source is FLOOR
paving, so native replacements must remain walkable. These comparisons provide
new route variety; neither establishes whole-map visual acceptance.

A new tree/art goal (`01a137c8-6899-7361-92fe-5ad8ca10ff94`) received the
Cerulean/Route8 evidence, source-family distinction and Route26 preservation
requirement. Coordinate broad foliage/build_scene/occluder changes with it.
Own-file diff checks pass; whole-worktree diff check also saw an unrelated
webmcp.rs blank EOF line, which was preserved.

Other concrete outstanding work: Route6 pond shores and ledge-gap underlays
retain flat ink; the roof hue mismatch is now fixed as verified above. Its eight-position
perimeter was inspected (`target/outdoor-route6-perimeter/`), including a
north-east building that fully hides the trainer and a southwest Surf view
missing the rider. Rider ownership stays with Pokémon/actors. SafariZoneBeta
south entrance also has flat source border strips reported by the interiors
owner; inspect their actual source/collision before choosing geometry. Continue
new city/route perimeters, Ecruteak timber/roof fidelity, Park/forest materials,
shorelines, all ground contacts and remaining flat objects. Do not claim
whole-exterior acceptance from the tests or these local fixes.

### Expanded city/route and palette review

The human explicitly confirmed **all** cities and routes, including ledges and
remaining flat scenery. Preserve the approved Route26 finish. Fresh paired
native/source views now include Ecruteak, Olivine, Blackthorn and Routes33,38,42,
in addition to the earlier route/forest/park/lake reviews. Their captures are
ignored `target/outdoor-new-*.png`; they establish defects, not acceptance.

Latest human steering: **swimmers mostly under water**. Route21's native
pair confirms standing swimmer NPCs. `new_bark_actors::swimmer_depth` now lowers
only Object SwimmerGuy/Girl rigs on authenticated water by 68% of actual
rendered stature, retaining the map foot and facing. Land swimmers and mounted
players keep existing support. A pure regression covers both genders, land,
non-swimmers and player identity. The first actual Route21 pair at78%
(`target/outdoor-swimmers-submerged-route21*.png`) was inspected and found too
much submersion (hair/head only), so depth was reduced to68% for readable
head/shoulders. The revised swimmer depth test passes (1/1, `target/outdoor-swimmer-refined-tests.log`).
Native rebuild completed in3m55s (`target/outdoor-swimmer-refined-native-build.log`).
The revised female pair (`target/outdoor-swimmers-refined-route21*.png`) and
male pair at `(5,30)` (`target/outdoor-swimmer-male-route21*.png`) were both
validated and inspected against source2D. Heads/shoulders now clear water
while most of each body is submerged; this scoped immersion repair is verified.
The original78% pair remains an earlier rejected depth candidate.
The native rider was also absent while the mount rendered; the Pokémon goal
was sent the exact pair. The swimmer helper never affects Player/SurfMount.
The Pokémon/model owner was notified of this small shared-file change.

The human additionally requires **city perimeters**, not only centers: corners,
exits, water boundaries and route connections. `render_at_location --maps ...
--perimeter --view both --output-dir ...` now selects eight perimeter targets,
resolves each to the nearest position accepted by the production runtime and
names pairs by edge and actual coordinate. Duplicate resolved positions are
skipped. This is a coverage tool, not proof that the edges are accepted.
The eight-target perimeter batch completed five Ecruteak pairs (NW,north,NE,
east,SE), all now inspected. South `(20,33)` was rejected during GPU readback:
unchanged frame12 checksum1330585277 and LCD14, actor list7->7 but unequal.
The batch stopped before SW/west/Cianwood; preserve strict paired-state
checks and do not count the failed view as covered. The interior goal owns
new failure-branch actor diagnostics and settling investigation; it was sent
this reproduction (`target/outdoor-city-perimeters.log`).

Ecruteak edges exposed two wrong landmark bindings: Burned Tower is a bright
ornate house, and the small shrine gate reuses the full tall TinTower model,
filling the north view with a wall. `ecruteak_houses.rs` now authors a charred
compact ruin and a separate timber gate with its source east-side passage,
narrow slits and cedar roof. The full pagoda remains separate and still needs
material/silhouette review. The east/SE groves also hide the player; not accepted.
These new models have footprint/winding checks. First native/source ruin and
north-house pairs were inspected (`target/outdoor-ecruteak-{ruin,gate}-fixed*.png`).
The charred roof/body now replaces the bright ornate model. The north view
now shows a compact wood building instead of a wall filling the frame.
That inspection plus exact source/warp data exposed a binding mistake:
north `[2c2d;26,2f]` is Wise Trios' side-entry house; the wider shrine
`[2c2a2d;26,27,2f]` has a front door. Current source now gives them separate
functions/openings, and the roof-footprint test also covers the wider temple.
Fresh pairs at north `(20,2)` and shrine `(18,12)` were validated and inspected
(`target/outdoor-{wise-trios,tin-shrine}-entry-corrected*.png`). The
north house's opening now covers its two east warp cells; the wider shrine
uses the source front-door offset. Its added temple geometry is included in
the expanded footprint/winding test, now passing in the current mesh suite.
Both buildings now use compact timber/plaster forms; canopy occlusion and
the wider shrine's plain facade still need refinement before accepting the city.
The latest source comparison showed missing lower timber paneling; both
facades now have upright wood boards and a narrow cap rail, with the shrine
doorway span excluded. Geometry tests pass in the latest 713-test mesh suite.
The shrine's latest pair (`target/outdoor-ecruteak-timber-base-fixed*.png`)
is now validated and inspected against 2D: its lower wood paneling is present
and the front opening is clear. The facade/door still look taller relative to
the trainer than in source; compact vertical proportions and warmer source
roof paint need another targeted pass. The north side-entry pair
(`target/outdoor-wise-trios-timber-fixed*.png`) is validated and inspected;
the entrance is clear, but foreground canopies still obscure its lower facade.
Do not mark all Ecruteak houses accepted.
The visible tower forecourt still uses a wooden deck/rail model; exact source
74/75 drawing needs a material/topology review against its gray steps.
Latest full mesh result: **713 passed, zero failed, six opt-in tests ignored**
(`target/outdoor-current-mesh-tests.log`). Kanto canyon hue/footing,
shallow-bank underlay, buoy geometry and all three shrine/ruin models pass.
The cave owner reconciled the former ice-hatch assertion with its reviewed
opening geometry. Current water suite passes 7/7 and swimmer regression 1/1
(`target/outdoor-current-{water,swimmer}-tests.log`).

Cianwood's complete eight-position perimeter sweep is validated and inspected
(`target/outdoor-cianwood-perimeter/`). The mineral cliffs stay continuous at
these edges, but buoy bodies read blue rather than source pale, rectangular
foam surrounds them, some shoreline source strips remain, and house roof
palettes differ from 2D. Buoy body faces now use the source foreground highlight;
the water distance field treats only authenticated Johto/Modern course floats
as wet rather than tiny islands. Geometry and water-distance tests pass.
Fresh native/source `(27,2)` pair (`target/outdoor-cianwood-buoy-fixed*.png`)
is validated and inspected: floats are pale, and rectangular foam bands are
gone. Course placement and the ordinary shore remain intact. This confirms
the scoped buoy repair; the eight perimeter captures do not accept the city.

Route4 is a new complete eight-position perimeter review, hour12
(`target/outdoor-route4-perimeter/`). Every pair is validated and inspected.
Its mountain interior remains pixel art between native mineral edges: exact
Kanto block `$2c` contains sixteen `$11` fill tiles but lacked the shared
plateau rule. The new narrow classifier gives that fill the same datum16
as adjacent edge blocks, and the existing terrain regression now includes
both corner and interior fill with Route4. Current mesh tests pass 713/713
(six opt-in tests skipped), and Kanto profile tests pass 12/12. Receipts:
`target/outdoor-route4-{timber-mesh,profile}-tests.log`. Native rebuild passes
(`target/outdoor-route4-native-build.log`, 3m46s). Fresh `(2,2)` native/source
pair (`target/outdoor-route4-fill-fixed*.png`) is validated and inspected:
the large interior pixel-art patch is now a continuous mineral plateau at
the adjoining edges' height. Pixel-painted bases around separate round
barrier rocks remain; those are a different source family and need their
own underlay repair. This scoped fill repair is verified.
Other observed defects remain: side-facing shallow-ledges retain source ink
strips, low faces are pale, trees are overfaded, and the southeast cliff hides
part of the player. This new route review establishes defects, not acceptance.

Mahogany's north `(10,2)` actual production/source pair was inspected
(`target/outdoor-mahogany-north-edge-*.png`). It confirms the flat house and
foreground roof occlusion. Its facade is `16/1e` on Mahogany's first row;
the connected Route43 final row supplies `08/09` roof tiles. Added this exact
complete signature and PlayerHouse selection, with a broken-pattern rejection
regression. Its roof now uses the existing live-source roof-paint path so the
Mahogany ochre palette replaces the PlayerHouse model's red roof paint.
Fresh orbit1 native/source comparison (`target/outdoor-mahogany-boundary-fixed*.png`)
was inspected: the boundary house now renders in 3D with the correct roof hue.
The foreground building still needs better occlusion, and low pale bank seams
remain; this confirms the house repair, not whole-town visual acceptance.

Celadon's east `(37,18)` actual pair was inspected
(`target/outdoor-celadon-east-edge-compare.png`). The adjacent blue-roof drawing
is still flat at the far right edge; source dark, narrow trees read as pale
rounded crowns with excessive transparency, and source brick paths remain
flat textured patches. The flat entrance is Route7's complete Kanto drawing
`38/39` roof over `10/66` facade, absent from the former residential
`38/39` over `3c/3d` detector. Added the exact doorway variant with a
complete-pattern/broken-pattern regression; native reinspection is pending.
Fresh Celadon `(37,18)` orbit2 pair (`target/outdoor-celadon-entrance-fixed*.png`)
was validated and inspected, but this camera omits the adjacent Route7 entrance.
It therefore does not verify that entrance repair. It confirms overpale grove
color and continued raw brick floors; capture on Route7 itself is still required.
The subsequent direct Route7 pair (`target/outdoor-route7-entrance-fixed*.png`)
is validated and inspected: the formerly flat entrance is now a complete
native house with a visible doorway. Its red roof still differs from the
source blue roof, and the stock house is taller than the source entrance.
The model-binding repair is verified; palette/proportions and pale low-face
seams remain pending before accepting the entrance.
This perimeter is not accepted. Its original capture
position lies within source trees, reinforcing the need to inspect the actual
pair rather than assume nearest runtime-valid positions provide clear views.

Six fresh Kanto route pairs were captured and inspected: Route1, Route3,
Route9, Route12, Route17 and Route21, hour12. Files are in
`target/outdoor-kanto-route-variety/`, receipt `target/outdoor-kanto-route-variety.log`.
Route1 exposes native shallow-bank underlay ink; Route3/9 have pale faceted
cliffs instead of source ochre rock. The outdoor Kanto deep-cliff branch now
uses the shared canyon finish, and exact complete low-bank front underlays
now receive their cap ground instead of retaining `$37/$34` pixel ink.
Fresh Route9 pair (`target/outdoor-route9-mineral-fixed*.png`) is validated
and inspected: the pale cliffs now use continuous ochre mineral surfaces.
Source fill patches remain at the bases (the `$2c` repair is newer than this
build), along with low gray faces. The source's rounded mound silhouette still
differs from the broad native rectangular forms; material repair is verified,
but this route is not fully accepted.
The existing actual-mesher bank regression was updated for that underlay.
Route12/17/21 expose flat dock/paving and shoreline borders, plus pale tree
crowns. They are not accepted; the new fixes still require paired reinspection.
This batch provides route variety, not complete perimeter coverage.

Focused portal regression passed (1/1). Exterior building tests passed (19/19),
including both boundary patterns and incomplete-pattern rejection. Receipts:
`target/outdoor-perimeter-portal-tests.log`, `target/outdoor-boundary-building-tests.log`.
The incremental native repair build completed successfully in 6m49s
(`target/outdoor-boundary-repair-build.log`); a Mahogany boundary native/source
recheck is running with orbit1 so the foreground roof does not dominate the view.
Latest Kanto finish/underlay changes require a fresh mesh suite after it; do
not infer passing new coverage from earlier tests.

Eight additional center pairs were inspected: Goldenrod, Mahogany, Azalea,
Lavender, Pewter, Fuchsia, Celadon and Saffron. Defects remain: raw flat scenery
at Mahogany's north side, pale bank seams, flat patterned floors, Celadon's
shore strip and buildings hiding the character in close views. Perimeter
inspection and fixes remain required. Latest Route38 22:00 native/source pair
(`target/outdoor-final-night*.png`) has the final 0.50 energy blue temperature
(0.40, 0.55, 2.0 before luminance normalization); it was inspected and is darker
and bluer while retaining readable scenery.
The separate active day/night-cycle goal now owns subsequent atmosphere and
clock refinements; it was sent these actual paired findings. Preserve its
shared-source changes rather than restoring these earlier constants blindly.
The cave-exit-lighting goal was also notified of exterior portal ownership.

Natural Route33/42 portal pairs exposed a black cap seam from the recessed
back wall's closed cuboid. It is now a vertical arch-following dark face.
The fresh Route33 native/source pair (`target/outdoor-arch-seam-fixed-route33*.png`)
was inspected and confirms the exposed black cap is gone. Footing remains
unchanged, but the fixture places the player on the face of the source bank;
this is not a gameplay/collision acceptance claim. The Route42 pair was also
inspected (`target/outdoor-arch-seam-fixed-route42*.png`): the major stripe is
gone, but a small back-wall edge still protrudes above the deformed intrados.
The dark wall now has 0.30 source-tile head clearance within the recess.
Another rebuild/reinspection and focused regression remain pending; neither
portal is yet wholly accepted.

Concrete new findings and repairs in progress:

- Ecruteak's small houses reused the pale plaster/large blue window kit.
  `mesh/ecruteak_houses.rs` now renders dark horizontal cedar roof courses,
  timber wainscot, narrow lattice windows and a wooden lattice entrance inside
  the original plot and doorway. The first corrected native pair was inspected
  (`target/outdoor-fixed-ecruteak*.png`); a lower roof and clearer timber lintels
  are being verified next. The gym is intentionally a separate source drawing.
- Blackthorn's complete mountain mound was replaced by a pale fitted mesa.
  Production building selection now leaves it to connected native canyon
  geometry; samples-only legacy diagnostics retain their existing contract.
  The first corrected pair shows the ochre mound instead of the pale pedestal.
  Dark mineral strips and exposed low path seams still require follow-up.
  Native rock-source samples now share the canyon pigment across maps.
- Route33's mountains were flat because the modern Johto atlas lacked their
  profiles. Its mineral cap/face glyphs now resolve separately from corner
  grass and cave-mouth art. Complete four-cell mountain entrances get recessed
  mouths through `mesh/mountain_portals.rs`, without writing footing or warps.
  Route42's previously covered cave mouth must be inspected after this repair.
- A Route38 22:00 pair proved the source 2D palette changed while 3D remained
  daylight. `VisualWorldLighting` now publishes the effective palette period
  of the successfully rendered map; it contains no advancing game clock.
  The initial dimmed night pass was visually rejected. **Latest human steering:
  temperature, not brightness—morning yellow, night lighter blue like the game.**
  A bright temperature-only night pass was inspected, but its path still read
  gray; the blue temperature was strengthened. **The subsequent latest human
  instruction is darker and bluer like the 2D view.** The current night uses
  0.50 of daytime light energy and a stronger luminance-normalized blue
  temperature; morning retains warm yellow temperature at normal energy. The shader now
  consumes ambient RGB instead of reducing it to brightness alone. Cave
  lighting retains its existing presentation. Fresh three-period native pairs
  are required before this revision is accepted.

The headless source-contained audit now accepts `--outdoors`. It successfully
meshed **77/77** `TOWN`/`ROUTE` maps with no mesher errors (receipt:
`target/outdoor-all-map-audit.log`). Its temporary report is outside the repo
at `/tmp/geothite-outdoor-coverage-20261013.json` and must be removed when no
longer needed. Native terrain without an authored-object claim is not thereby
flat; its report is source/ownership evidence only. This audit does not replace
visual coverage across all maps, positions and periods. Forest/other outdoor
environments remain separately in scope.

Latest completed mesh suite: 698 passed, zero failed, six opt-in checks ignored
(`target/outdoor-temperature-mesh-tests.log`). The earlier sole station-floor
fixture failure was corrected by its owner. Subsequent shared modern hop-down
profiles, modern bank-material coverage and the final darker/bluer temperature
still need the final tests/native review. Canonical external packs remain
unchanged; no generated reports, packs or build products are deliverables.

The goal is active again. The current source already contains the edge-anchored
bevel and shared-position repair; the old failing topology account below is
historical and must not be treated as the current test result. Fresh verification
is in progress in `target/agent-outdoor`, independently of the interior goal's
cache. The pack-build goal retains sole export/publication ownership. The
completed local `content-packs/core-modular.browser.crystalpack` is available;
the old `realtime-clock.browser.crystalpack` filename is absent on this host.
Use the actual completed pack explicitly, without inventing a replacement path.

Five model-asset storage tests pass on this checkout. New verification exercises
49 mineral-coordinate phases through the combined production rock finish and
adds an opt-in external-layout cap diagnostic for New Bark, Route29,
Cherrygrove, Route30, Route31, Violet, Route26, Route27, Route43, Route45,
Route46 and Lake of Rage. The latter checks native-scale cap winding and
unchanged footing/claims using the actual external source layouts; it is not
whole-scene visual acceptance or a gameplay test. The 49-phase combined-finish
test passes. The external-layout diagnostic passes all twelve maps: 753,408
native rock cap triangles retain upward winding with unchanged footing/claims
(434.69s in the unoptimized test build). Receipt:
`target/outdoor-resume-external-rock-tests.log`.

The first test compilation stopped at missing generated Wooper storage: the
Pokémon goal added the canonical GLB/registry entry after this cache collected
its model inputs. The asset is now present and subsequent compilation succeeds.
That failure was not a terrain assertion result. The current shoulder candidate
projects samples onto one diagonal mineral plane instead of radially scaling
them by an octagonal distance; radial scaling bowed height rings within each
octant. The previous edge recession, support constraints and shared-coordinate
evaluation remain. Its planar-ring and existing stone/ledge/shore checks pass.
All 63 ground/exterior tests pass after lowering the ordinary decorative lawn
blades to their established low-height bounds; source floors stay fixed. The
native mineral drawing now has a regression requiring identical source pigment
across routes instead of obsolete map-name recoloring. Three legacy unmodeled
tree tests are being updated to require exact source tiles rather than silent
pixel-hull substitution. The final mesh suite passes all 690 tests (six opt-in
checks ignored; 57.25s), including those source-tree checks, all 49 mineral
phases and the distant-mass index/full-scan comparison. Receipt:
`target/outdoor-resume-final-mesh-tests.log`.

The full-map diagnostic exposed the cost of scanning every rock edge at every
vertex. A local edge index now conservatively includes four source tiles around
each joined edge. It retains edge order and discards only remote zero-influence
fields. Full-scan comparisons across the 49 phases and a separate distant-mass
fixture pass. The external-layout diagnostic is being rerun after this indexing
change. Fresh muted side-view captures remain
pending. Do not call the shoulder candidate visually accepted yet.

Route variety is a first-class acceptance requirement. Fresh paired reviews
must cover the early-game chain, coastal Route27, the Route45/46 mountain
family, Route43/Lake, Ilex and both National Park plazas, alongside Route26's
45/90-degree rock views. Passing one rock fixture or revisiting only Lake and
Route26 does not complete this goal. Review natural green variation, tree
family and silhouette, readable routes and clearings, banks/floor seams,
water contacts and source fidelity in each scene. Any scene repairs must be
inspected again against matching-state 2D.

Latest human steering: stop concentrating captures on the same Route26 scene.
The user explicitly clarified that the area is good. Preserve its current
finish while reviewing and improving other regions, with their own scenery
and character. The optimized Route26 45/90-degree pairs completed and were
inspected at 45 degrees; their successful captures do not stand in for the
rest of the exterior world. Route45, New Bark, Park and Ilex reviews are now
in progress. Do not resume repeated Route26 tuning ahead of that broader work.

The user further expanded the acceptance gate: inspect and clean/smooth ledges
throughout the routes, fix any visible environment tiles still retained in 2D,
and review morning, day and night. They explicitly require new cities and routes,
not only previously worked areas. The next new-city set is Ecruteak, Olivine
and Blackthorn; the next route set is Route33, Route38 and Route42, with broader
route coverage still required afterward. `new_bark_3d` now accepts validated
`--hour 0..23` through the existing authoritative preview clock boundary,
preserving the previous default of 16. Its updated build/time captures are
pending. A source-only tile diagnostic or one attractive city does not satisfy
the new whole-route/lighting gate.

The Park reference companion now records the later rejection of an inferred
closed Poké Ball sculpture. Retain the source fountain geometry; further Park
work concerns foliage, benches, the two grass families and material finish.
The original external forest reference remains unavailable locally. Neither
fresh GPU acceptance nor complete all-map floor-seam coverage is established
yet. Missing Gyarados coverage and the ignored lake-removal variant also remain
open until checked against the current pack.

The first macOS native build completed (16m16s), with renderer optimization
disabled to shorten initial compilation. Its muted Route26 (10,28), orbit2,
paired capture **failed** at the 300-second deadline: `building authored terrain`.
There is no image and no visual acceptance from that attempt. A live process
sample reached terrain occluder preparation. A new native build uses
`--config profile.dev.package.crystal-voxel-view.opt-level=1` while keeping the
same pack, model root, geometry and capture coordinates. Receipt:
`target/outdoor-resume-native-optimized-build.log` (passed, 5m38s); failed attempt:
`target/outdoor-resume-route26-45.log`.

The existing Rust `pack_outdoor_terrain` builder successfully produced ignored
`content-packs/outdoor-terrain.browser.crystalpack` from the current verified
browser pack. The canonical packs are unchanged. The new opt-in
`crates/crystal-assets/tests/outdoor_terrain_pack.rs` checks the actual output
against that base for exactly four changed blocks and unchanged audio bytes,
PCM metadata, runtime assets, events and all unrelated data. Matching-state
Cherrygrove/Route30 captures remain pending. The actual-file
preservation test passed (2.21s): `target/outdoor-resume-variant-test.log`.

## Current acceptance status

The missing pack inputs were rebuilt through the Rust pipeline. Production
source/native paired captures now run on this Windows host with `WGPU_BACKEND=dx12`,
muted audio and the canonical model root. The shared shader's invalid cutaway
field access has been corrected; terrain renders again.

The cap/wall classifier and shared deformed positions now preserve the combined
rock finish: all 19 stone and six base-contact regressions pass. Edge-anchored
bevel variation replaces height-dependent variation. A narrower mineral bevel
also passes those checks. Wall-only fractures now retain separate normals;
only actual cap/wall joins receive unconditional shoulder smoothing. Fresh
45/90-degree comparisons show closed visible contacts, but the broad caps
remain too soft for the requested mineral silhouette; visual acceptance is open.
The retained bed relief is shallower, with stronger cap pigment and wider low
lawn tufts. Source-footing checks pass. Fresh 45/90-degree native captures retain
closed visible contacts; the broader silhouette remains visually unaccepted.

Cherrygrove path/water contacts use bare soil without adding lawn plants or
moving source floor corners. Its datum regression, shoreline and ledge checks
pass. Fresh Cherrygrove and Lake captures show soil edges and tapered water
glints. The separate ignored lake-removal variant restores exactly four blocks;
its audio manifest, runtime assets and other game data match the verified base.

The full outdoor goal remains incomplete. Route26 silhouette quality, complete
floor-seam coverage, Lake canopy readability, Gyarados coverage, Ilex foliage
quality and Park's central fountain/reference finish remain acceptance work.
The current Lake 45-degree capture confirms a 2D Gyarados marker: the source
binding exists, but Gyarados is absent from the exact-species model registry.
The historical transfer failures below describe the old checkpoint, rather
than the current passing rock regression.
Preserve the actual source fountain silhouette. The user's latest review
rejects replacing it with an inferred closed Poké Ball sculpture; source
fidelity takes precedence over the concept note's wording.
The restored fountain passes its three source-ownership regressions and a fresh
matching-state 2D/native comparison. No fountain-shape change remains.

## Goal and stopping point

Original goal: **“Make outside rendering perfect up to the standards I set.”** This goal is incomplete. The human requested all work committed for transfer and explicitly instructed this thread to listen to the coordinating agent. Work stops after this checkpoint; resume improvements on the new host after shared integration is committed.

Source checkpoint: `4d8d5824bd36109655493f28506c9623687783ab`. The commit adding this note can be found with `git log -1 -- docs/handoff/outdoor-terrain.md`. Neither this scope commit alone nor an old screenshot establishes a complete transferable build: the coordinator must capture the shared integration paths below.

The target is the supplied warm, nuanced forest remake: natural green variation, dappled light, alive grass, convincing trunks and mineral forms, and blended ground contacts. The supplied forest attachment is external to Git and is not available locally for this handoff. Saved National Park art is tracked at `docs/art/map-concepts/national-park-remake-v1.png` with its companion Markdown. Preserve its real image bytes through Git LFS transfer where applicable. The full review history and area requirements are in `docs/art/outdoor-terrain-review.md`.

## Current implementation

The latest change composes mineral-rim sculpting and rock-base contact rounding before one fracture-aware normal calculation. Previously the base pass erased cap fracture shading. Outdoor caps reuse the existing bedrock texture at the cave top-projection scale; no duplicate rock asset was created. Larger source-coordinate mineral beds replace short-period height ripples. Source floor support, closed corners and tree seats remain requirements.

Lake cabin roofs use their own source colors and the existing closed curved roof courses. Corner trees use exact source-phase bindings. Ordinary lawn/water contacts reuse shared soil relief and a narrow map-anchored damp color band without moving native floor corners. Buoy courses follow exact source ownership. Required tree loading has no alternate 3D fallback; original 2D is permitted to reveal missing coverage. Ordinary grove, Park and intended forest families must stay distinct. Shared route signs reuse one resized authored model.

Latest action: inspected muted Route26 45-degree and Lake clearing 135-degree captures, then completed the extended production winding diagnostic. **The diagnostic failed.** No geometry fix was attempted after the transfer instruction. Its failing regression remains in the checkpoint so the defect is reproducible.

## Checks and unresolved defects

The following bullets preserve the old-host transfer state. Their old winding
FAIL is superseded by the passing current macOS production rock regression
and mesh receipt above; historical captures still do not establish current
whole-scene visual acceptance.

- Current combined native build passed (`target/outdoor-final-rock-pipeline-native-build.log`, 12m16s). This was the full dirty worktree, including shared integrations, not the scope commit in isolation.
- Before the winding extension: 19 stone tests and six base-contact tests passed. Seven ledge-approach tests passed. Five Python asset-storage tests passed at transfer.
- Earlier scoped checks passed: nine cabin, three roof-course, three shore-blend, five ground-blend, two soil-contact, four required-loader and one missing-forest tests. These were not all rerun after the final diagnostic extension.
- **FAIL:** `production_rock_finish_keeps_fractures_after_rounding_the_contact_base`, including corrected native per-tile tessellation. `target/outdoor-final-rock-pipeline-native-topology-test.log` reports an inverted cap triangle at (9.220304,4,14.89865), (8.822326,5.3333335,13.378186), (8.905096,4,13.626882). Do not claim earlier independent winding checks cover the combined sculpt/base pass.
- Route26 (10,28), orbit2/45 degrees: latest inspected image completed in 202.16s. Fracture shading survives, but cliffs are still swollen/lumpy. Side-view bank caps rise; requested saw-blade presentation and every floor-Y seam remain unaccepted.
- LakeOfRage (23,30), orbit6/135 degrees: latest inspected image completed in 228.83s. Quiet cyan curved roof and damp soil contact are visible. Shore geometry stays straight, water retains rectangular glints, trees repeat. This angle leaves the clearing visible but does not resolve canopy overlap in earlier 45-degree views. Earlier Lake view exposed a permitted 2D creature marker; missing Gyarados coverage remains to verify.
- Whole-area review remains: New Bark through Cherrygrove and other routes/cities, Lake/Route43, stone/canyon surfaces, all source floor contacts. National Park still needs exact outer tree families, benches, two grass types and centerpiece shape. Ilex appearance is unaccepted; do not repair it by changing every tree globally.
- Cherrygrove's unwanted lake was overridden in ignored local content: Cherrygrove blocks 17/18 row0 and Route30 x9 rows25/26 water changed to trees. Transfer the actual modified pack and recheck source/native appearance; do not commit packs or restore them from Git history.

Investigate cap deformation/winding first on the new host. Per-vertex height-dependent bevel noise may need an edge-anchored radius; that is an unimplemented investigation, not an accepted repair. Then compare fresh 2D/native pairs at 45 and 90 degrees, correct shape and floor seams, and return to Lake cabins/shore/water. Keep source collision, events, occupancy, claims and floor datums authoritative.

## Ownership and shared integration

The source checkpoint owns `mesh/outdoor_stone.rs`, `mesh/rock_bases.rs`, `mesh/shore_blend.rs`, `mesh/buoy_course.rs`, `mesh/ledge_finish.rs`, `new_bark_models.rs`, `roof_tiles.rs`, `open_models.rs`, `exterior_models.rs`, `foliage.rs`, `foliage/park_tree.rs`, route-sign JSON, three obsolete tree JSON removals, the foliage/world authoring tools, storage test and outdoor review document. Rust paths are under `crates/crystal-voxel-view/src/`; model paths under its `models/`.

The coordinating agent must preserve these mixed-owner paths in its shared integration commit:

- `crates/crystal-voxel-view/src/mesh.rs`: shore/buoy module wiring.
- `crates/crystal-voxel-view/src/mesh/build_scene.rs`: production `finish_rock_geometry`, source binding/roof integration.
- `crates/crystal-voxel-view/src/mesh/new_bark.rs`: source lawn palette, shoreline finish/tessellation, corner crowns, tree foundations, source floor contacts and common grove integration.
- `crates/crystal-voxel-view/src/mesh/modeled_exteriors.rs`: cabin roof palette extraction, canonical forest loading and exact source-2D missing coverage; preserve other owner's underplanting.
- `crates/crystal-voxel-view/src/mesh/ground_blend.rs`: shared source path/lawn palette integration.
- `crates/crystal-voxel-view/src/voxel_surface.wesl`: outdoor negative-tag rock top projection at 144 units; preserve lighting/cave owners' changes.
- `tools/build-new-bark-models.py`: obsolete tree prototype removal mixed with another owner's NPC work.
- Required-loader adapters, including `interior_models.rs`, `johto_actor_props.rs` and their callers, plus remaining obsolete actor/interior fallback deletions, belong in the coordinator's shared checkpoint. They were deliberately not swept into this scope commit.

No whole-worktree staging was used. No pack, native executable, scratch render, ROM, exported source catalog or generated PCM is a deliverable here.

## Restart on the new host

Use the coordinator's `docs/cold-start-3d.md` and `scripts/cold-start-3d.sh`; do not create a competing startup path. Transfer Git history, separate real Git LFS objects/assets and ignored `content-packs/` contents. Required pack: `content-packs/realtime-clock.browser.crystalpack`. Required canonical model root: `$PWD/assets/open-models`. Git LFS pointer text is not an asset. Follow pinned Rust toolchain and platform/GPU dependencies in the cold-start documentation. Captures need a working native GPU backend even when using image readback.

Run inside the transferred checkout after shared integrations:

```sh
./scripts/cold-start-3d.sh check
python3 tools/test_model_asset_storage.py
cargo test --locked -p crystal-voxel-view --lib \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  production_rock_finish_keeps_fractures
```

The last command currently fails and is the first defect to fix. Relevant focused filters afterward are `mesh::outdoor_stone::`, `mesh::rock_bases::`, `mesh::ledge_approach::`, `mesh::shore_blend::`, `cabin_` and `roof_tiles`.

For a dedicated native build:

```sh
CRYSTAL_PREVIEW_MUTE=1 CARGO_TARGET_DIR="$PWD/target-outdoor-verify" \
CARGO_INCREMENTAL=0 CARGO_PROFILE_WASM_DEV_DEBUG=0 \
cargo build --locked --profile wasm-dev \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 --jobs 2 \
  -p geothite -p crystal-bevy -p crystal-voxel-view -p crystal-render-api \
  --all-features --example new_bark_3d
```

Capture fresh source/native comparison without loud audio:

```sh
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
./target-outdoor-verify/wasm-dev/examples/new_bark_3d \
  content-packs/realtime-clock.browser.crystalpack \
  --map Route26 --x 10 --y 28 --orbit 2 --zoom 1 --size 640x480 \
  --screenshot target/outdoor-route26-new-host-45.png \
  --second target/outdoor-route26-new-host-2d.png
```

Repeat with orbit4 for 90 degrees. For Lake use `--map LakeOfRage --x 23 --y 30 --orbit 6`; also review 45 degrees to expose canopy issues. Keep every preview muted with `CRYSTAL_PREVIEW_MUTE=1`, never by changing system volume. Ignored old-host screenshots/logs are diagnostic history and must be regenerated for current acceptance. A pack dump or render does not prove playable controller/MCP input, and this scope makes no audio or gameplay validation claim.
