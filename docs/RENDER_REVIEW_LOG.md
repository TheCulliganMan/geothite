# Render review log

Historical captures and findings. Session-state remarks describe the time of
each entry; inspect current processes before sending requests or restarting.
Temporary image paths are local evidence, not shipped artifacts. For commands
and profile authoring, see [Rendering toolkit](RENDER_TOOLKIT.md).

## Verified progress and remaining review

September 9, 2026: the compound-profile renderer built successfully. No tests
were run as part of the art loop. Subsequent profile edits used the same
executables, without compilation. Warm paired captures and comparisons took
1.20–1.50 seconds; cold startup is excluded. Multiple concurrent viewers can
slow startup considerably.

| Room | Evidence | Result / remaining work |
| --- | --- | --- |
| Elm's lab `(4,3)` | `/tmp/elm-north-2d.compare.png` | Upright cabinets, aligned windows, complete machine, rear workstation, compound central desk with preserved book and masked monitor; lab viewer now stopped |
| Player home 1F `(4,5)` | `/tmp/player-home-live-2d.compare.png` | Table includes its original stray leg pixels; default and angled views inspected; viewer stopped |
| Neighbor's house `(3,5)` | `/tmp/neighbor-live-2d.compare.png` | Table has separate legs and open underside; plants have separate pots/stems/fronds but foliage remains too fragmented; viewer stopped |
| Elm's house `(2,5)` | `/tmp/elm-home-live-2d.compare.png` | Table includes its leg pixels; PC floor strip cleared with a separate feet profile; viewer stopped |
| Player bedroom `(3,4)` | `/tmp/bedroom-live-2d.compare.png` | Raised table and attached feet; computer floor strip cleared; bed and stair landing still need final review; viewer stopped |
| Route 29 `(25,8)` | `/tmp/route29-live-2d.compare.png` | North gate upright; small-tree silhouettes repaired at the review location; broader route and ledge joins still need review; live |

Active requests use their corresponding prefixes, e.g.
`/tmp/neighbor-live-2d.request` and `/tmp/elm-home-live-2d.request`.
New Bark exteriors and all remaining room/route defects still belong to the
full goal; these captures do not establish its completion.

The player-house north wall matcher requires complete block identities.
Replacing PC cells inside its north course currently prevents the old wall
matcher from recognizing that course. The Elm-house PC therefore keeps its
existing upper fixture and overrides only the feet in block `$19`; the wall
remains intact. Its live correction took 1.26 seconds. Bedroom table `$07`
and computer feet `$06` are outside the north course and changed together
in 1.21 seconds. Both were profile edits with no build.

## Outdoor review

New Bark's town and building signs now use the complete 16-pixel drawing
with a one-pixel depth. `/tmp/new-bark-live-2d.compare.png` records the
current town view at `(9,8)`; that session is now stopped. A flat-fold tree
border trial was removed because it did not clearly improve the silhouette.

The Route 29 gate crosses the map connection: roof blocks `$08/$09` are in
Route 46's last row; facade `$1a/$11` is in Route 29's first row. The live
profile matches the whole 8×8-tile drawing using grass `$05` beneath it.
Its 32-pixel roof fold and 32-pixel facade produced an upright building in
1.74 seconds, without compiling. `/tmp/route29-gate-change.compare.png`
contains the flat/upright comparison. Both Route 29 and Route 46 now have matching map-scoped rules. The final
compound profile uses 64 pixels of depth so its rear meets the northern
approach; both sides were rendered after that change.

## Small trees and connected gate verification

Route 29's small trees use the complete `[[30,31],[62,63]]` source drawing.
Profiles are scoped to the actual map blocks, replacing disconnected canopy
fragments and oversized trunks with one ground-masked upright. Final height
is 18 pixels with 2 pixels of depth. A 5-pixel depth was visually too heavy
and was reduced. `/tmp/route29-tree-repair.compare.png` preserves the initial
repair comparison; `/tmp/route29-live-2d.compare.png` has the final default
view. Warm edits took 1.72–1.84 seconds, without builds or tests. Matching
New Bark variants are authored, but their southern placements still require
a closer review; the town-center capture does not show those placements.

`/tmp/route46-gate-live-2d.compare.png` shows the connected gate from Route 46
at `(8,31)`, zoom 0 and orbit 0.5. Its session is now stopped; when restarted it accepts
`/tmp/route46-gate-live-2d.request`. Gate depth was increased to 64 pixels
using a compound part, removing the ground gap between its rear and the
northern approach. Route 29 and Route 46 captures both reflect that final
depth. Tall tree borders, interior foliage and wider route traversal remain
unverified or unfinished; these changes do not complete the full goal.

## Tall trees and rendered movement

Tall `$05` border trees in New Bark and Route 29 now preserve each complete
16×32 drawing as a ground-masked upright with 2 pixels of depth. This removes
the dense cylindrical treatment while retaining the branch silhouette.
Default and 22.5-degree Route 29 views were inspected. New Bark's south view
at `(4,12)`, `/tmp/new-bark-south-2d.compare.png`, also includes its small
trees and the border beside the house; that viewer has been stopped.

A rendered walk used the existing executable, with ordinary directional
input and no build:

```sh
CRYSTAL_VOXEL_PROFILES=modpacks/voxel-view/profiles.json \
  target/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route29 --x 25 --y 8 --view 2.5d \
  --walk DDRRRRUUULLLL --screenshot /tmp/route29-walk.png
```

It captured 416 frames across 21 grid origins with zero additional terrain
builds; median frame time was 16.99ms and p95 was 28.34ms. A ledge jump
occurred, and some movement was blocked normally. The trace is
`/tmp/route29-walk.movement.csv`; moving screenshots use
`/tmp/route29-walk-moving-NN.png`. Frames 00, 06 and the final image were
visually inspected. This is a local walk, not proof of every route segment.
Terrain-edge treatment and remaining room foliage still need review.

After saving the final PNG, the one-shot process stalled in Bevy's
`RenderAppChannels` destructor during `World::clear_all`. A native sample
confirmed the wait (`/tmp/routewalk-exit.sample`); the finished capture
process was stopped. This was later corrected by keeping one-shot captures on the main render
thread; see the beveled-surface verification below. Live review remains
the preferred art loop.

## Route 29 western entrance review

Reviewed `(3,4)` at default orbit and orbit `0.5` using the existing live
executable. Artifacts: `/tmp/route29-west-2d.compare.png` and
`/tmp/route29-west-2.5d.png`. The sign already stands upright under the
existing renderer. An exact `$47` profile trial gave no visible improvement
and was removed; do not add a duplicate rule just to increase coverage.
The trial reloaded and captured in 2.21 seconds; the camera-only capture
completed in 1.45 seconds. No Cargo or tests were run.

The angled view exposes dense repeating border silhouettes and the connected
western map's water/bank edge. These still need visual refinement; the sign
review does not establish that the whole western approach is finished.
The western live viewer accepts `/tmp/route29-west-2d.request`.

## Cherrygrove continuation

The user moved the active visual pass to Cherrygrove City. The uncompleted
24px tall-tree height experiment was reverted before moving; canonical tall
trees remain 32px. Previous Route 29 and New Bark viewers were stopped.

Cherrygrove City `(25,8)` is captured at
`/tmp/cherrygrove-center-2d.compare.png`. This spawn lies on a house drawing,
so the translucent player over its roof is not evidence of ordinary walking
behavior. Use a street spawn for movement review. Existing New Bark/Route 29
tree treatment now has five exact Cherrygrove-scoped rules: two tall-tree
halves and three small-tree placements in block `$64`. Central tall trees
were inspected at default and 22.5-degree orbit; the `$64` northern placement
still needs a close view. Profile reload plus capture took 1.67 seconds.

Cherrygrove Mart `(3,5)` revealed flattened refrigerators and shelves. Five
exact rules now fold their 16x32 source drawings into 24px upright faces with
8px tops/depth: both halves of `$14` and `$15`, plus the right half of `$27`.
Floor tile is `$48`. Before/after capture showed the rear refrigerators and
front-left shelves standing upright while retaining the counter and aisle.
Reload plus capture took 1.27 seconds, without Cargo or tests.

Live requests: `/tmp/cherrygrove-center-2d.request` and
`/tmp/cherrygrove-mart-2d.request`. The city, coastline, northern exit, other
interiors, and ordinary door approaches are not yet a completed review.

## Cherrygrove Pokémon Center

The pack's exact map name is `CherrygrovePokecenter1F` (lowercase c in
Pokecenter). Live review at `(5,5)` uses
`/tmp/cherrygrove-pokecenter-2d.request`. The city exterior viewer was stopped;
the Mart and Pokémon Center viewers remain available.

Two exact profiles correct the center's fixtures. The PC is the complete
16x24 drawing in `$08`, origin `(2,1)`, folded with a 4px top and 6px depth.
Previously its upper screen was attached to the backdrop while the lower
panel stood separately. The healing machine is the complete 32x32 `$01`
drawing, with a 4px top and 10px depth; it now has a closed body. Ground is
`$11`. Front and angled PC captures and an angled before/after machine
capture were inspected. Reload/capture times were 1.32 and 1.35 seconds.
No build or tests were needed. Review artifacts use the
`/tmp/cherrygrove-pokecenter` prefix. Other fixtures and upstairs are not
covered by these two corrections.

## Cherrygrove houses

Pack inspection confirms `GuideGentsHouse`, `CherrygroveGymSpeechHouse`, and
`CherrygroveEvolutionSpeechHouse` share the same 4x4 block layout and house
atlas. Each now has exact map-scoped copies of the New Bark neighbor table
and two compound plant rules. The table removes the floor-colored apron,
raises its top, and keeps two distinct front legs. The plant rules separate
fronds, stem, and pot; their thin foliage remains a refinement opportunity.

All three houses were rendered at `(3,5)` and compared with faithful 2D.
The Guide Gent house also has an angled view showing both plants and the
open table underside. Its warm edit/capture took 1.29 seconds; angle capture
1.49 seconds. The other houses started in approximately 3 seconds, then
produced paired reviews in 2.58–2.69 seconds. No build or tests were run.
Artifacts use `/tmp/cherrygrove-guide-house`, `/tmp/cherrygrove-gym-house`,
and `/tmp/cherrygrove-evolution-house` prefixes. Guide and evolution viewers
remain live; the gym house, Mart, and Pokémon Center viewers are stopped.
These are static furniture reviews, not entrance or NPC movement validation.

## Waterfront and northern exit findings

Cherrygrove City `(13,9)` was rendered at default and 22.5-degree orbit;
artifacts use `/tmp/cherrygrove-waterfront`. `(17,3)` shows the northern exit
and connected Route 30 geometry under `/tmp/cherrygrove-north`. These two
viewers replace the house viewers and remain live.

The coastal `$0a` boulders repeat their drawing on tall cube faces. A live
trial folded their 32x32 source with a 20px top and 32px depth. Although lower,
it exposed pale horizontal side bands and was rejected. No boulder profile
is retained. A future correction must address the silhouette and side-surface
mapping together in the mesher; lowering a rectangular profile alone is not
adequate. The angled trial completed in 1.85 seconds without a build.

The northern view also exposes a mismatch between Cherrygrove trees and
connected Route 30 geometry, plus the raised ledge at that connection.
These findings remain open; neither static capture proves traversal or
completes the coastal terrain pass.

## Beveled surfaces

A compound part may specify `bevel_pixels` to map its complete drawing over
one raised surface with rounded sloping corners. Set `top_pixels` equal to
its source rectangle height, `height_pixels` to its rise, and `depth_pixels`
to its footprint depth. Bevel width must be positive and no greater than
half the smaller source dimension. This mode requires zero `base_pixels`
and `mask: "none"`; those restrictions keep its perimeter at ground level
and avoid open holes. Every source pixel maps once; no facade texture is
repeated down side walls. Height and bevel width remain live JSON controls.

This capability changes Rust and requires rebuilding the executable once.
The shoreline render and clean one-shot exit were verified after that build.
One-shot native captures now disable Bevy's
pipelined render plugin to avoid the observed `RenderAppChannels` teardown
wait. Live review and ordinary play retain their separate render thread.


The retained Cherrygrove `$0a` rock profile uses a full 32x32 drawing,
32px footprint depth, 8px height, and 10px bevel. The first 12px-height,
8px-bevel render established the new geometry; a live edit reduced its
height and softened the slope in 1.68 seconds. Default and 22.5-degree
views were inspected. This replaces the earlier rejected rectangular-fold
trial, preserving the source rock drawing without repeating it on cube
faces. Other coastal blocks retain their existing geometry.

One-shot paired capture `/tmp/cherrygrove-bevel-once` saved and validated
both images, printed `render loop finished in 8.19s`, and exited with code
0. This verifies the former teardown hang is resolved for paired capture.
The live viewer `/tmp/cherrygrove-bevel-live` uses the trial file
`/tmp/cherrygrove-bevel-profiles.json`; its retained profile has been copied
into canonical `modpacks/voxel-view/profiles.json`. Restart future reviews
with the canonical file. No tests were run. One Rust build installed both
capabilities; all subsequent visual adjustments used JSON reload.

A second shutdown check used ordinary rendered walking on Cherrygrove's
street: `(25,11)`, `--walk LLLLRRRR`, screenshot
`/tmp/cherrygrove-street-walk.png`. It completed 256 frames across 16 grid
origins, with two additional terrain builds, median 16.95ms and p95 25.31ms.
Some input met normal collision. Final PNG validation completed and the
process exited 0 in 15.03 seconds, confirming the walk path also tears down.
This run uses the non-pipelined one-shot configuration; its frame timings
are not directly comparable to earlier pipelined live measurements.

## Cherrygrove / Route 30 tree continuity

Added the same two exact `$05` tall-tree rules to Route 30 after its southern
view exposed the older cylindrical treatment. Source drawing and dimensions
match the Cherrygrove profiles. Default before/after and 22.5-degree views
at `(7,49)` were inspected under `/tmp/cherrygrove-route30-seam`; warm profile
capture took 2.05 seconds and angle capture 1.32 seconds. The live viewer
uses canonical profiles and remains open; the older bevel trial viewer was
stopped.

An ordinary southward walk (`--walk DDDDDD`) crossed from Route 30 into
Cherrygrove, confirmed by the CSV map column and music transition. The final
image is `/tmp/cherrygrove-route30-crossing.png`, with trace alongside it.
It recorded 192 frames, 11 grid origins, and one additional terrain build
for the transition; median 16.94ms, p95 32.78ms. Later inputs met normal
collision. The one-shot process exited 0. This confirms that local map
crossing, not a full traversal or complete Route 30 visual pass. No build
or tests were needed for this continuation.

## Shared Pokémon Center upstairs

`Pokecenter2F` is the shared upstairs map, including Cherrygrove's center.
At `(5,5)`, the PC was split across `$31/$32` and the wall displays lay flat.
One exact cross-block rule now unifies the 16x24 PC (4px top, 6px depth), and
two 32x16 rules stand the `$2b/$0a` displays upright with 1px depth. These
changes therefore apply to the shared upstairs room wherever entered.
Default before/after captures took 1.31 and 1.34 seconds. Artifacts use
`/tmp/cherrygrove-pokecenter-upstairs`; the viewer is live with canonical
profiles. No rebuild or tests were used. Booth partitions, seating, and
stair traversal still require separate review.

The toolkit was shortened by moving historical review entries into this
log. Current process state must be checked rather than inferred from old
entries. Reusable commands, schema guidance, and capture-mode behavior stay
in `RENDER_TOOLKIT.md`.

## Pokémon Center seating

The shared upstairs `$2d/$29` seats were upright panels in the angled view.
Three exact 16x16 profiles now use 12 source rows for the horizontal cushion,
12px footprint depth and a 6px base height. The identical seat drawing in
Cherrygrove 1F `$2e/$2f` now has two corresponding rules, removing the gray
plinth around the old flattened cushions. Upstairs was inspected from front
and 22.5-degree orbit; downstairs before/after was inspected at the front.
Warm changes took 1.31 and 1.37 seconds without rebuilding or tests.

The two Pokémon Center viewers are live with canonical profiles, using
`/tmp/cherrygrove-pokecenter-upstairs` and `/tmp/cherrygrove-seat-downstairs`
prefixes. The Route 30 viewer was stopped. Stair traversal and the remaining
booth geometry are still open review items.

## Pokémon Center stair traversal and actor regression

From Cherrygrove 1F `(2,6)`, `--walk LL` was blocked by the stair-side NPC;
the walk driver correctly failed its no-movement assertion (exit 101).
Approaching from `(2,5)` with `--walk LD` reached `Pokecenter2F`, confirmed
at CSV frame 53. Extending to `LDRL` returned downstairs, but the final
capture caught actors hidden around the transition. `LDRLRR` provided a
longer landing sequence, rendered the player again, and exited 0. Trace and
image prefix: `/tmp/cherrygrove-stairs-settled`. The latter run recorded
192 frames and two additional terrain builds, median 16.78ms/p95 18.88ms.
No Cargo or tests were run; these were actual rendered movement runs.

**Open actor defect:** the nurse is present on the fresh downstairs render
but absent after the return, including the longer final image. Do not call
stair visual validation complete. Investigate actor restoration/occlusion
across the warp before accepting the round trip. The short screenshot's
nonblank validation did not detect missing actors. Both previously live
Pokémon Center viewers were stopped before these one-shot runs.

## Nurse disappearance diagnosis

A fresh paired capture at Cherrygrove 1F `(6,7)` reproduced the missing nurse
without any warp (`/tmp/cherrygrove-nurse-culling`). This narrows the earlier
"actor restoration" suspicion to visibility: source NPC sprites were culled
by `CLASSIC_SCROLL_HALO_TILES`, while the pitched terrain uses the larger
`VISUAL_WORLD_HALO_TILES`. The room remained visible beyond its actors.

The Rust source now uses the visual-world halo when retaining NPC source
sprites, including both endpoints of moving objects. Non-voxel margins stay
unchanged. Build and fresh-position/round-trip verification are pending for
this change; do not infer the fix is verified from diagnosis alone.

The first visibility-margin build did **not** restore the nurse. Further
inspection found an earlier filter: runtime snapshots contain only loaded
LCD-range object structs. The existing fullscreen presentation expansion
already supplies eligible unloaded NPCs with their last pose while respecting
explicit hidden/invisible flags. That helper is now shared with voxel-view
builds, in addition to the wider sprite retention margin. Gameplay object
allocation is unchanged. The revised build still requires render verification.

The revised build passed. Fresh `(6,7)` capture
`/tmp/cherrygrove-nurse-expanded-2.5d.png` shows the nurse present, unlike
`/tmp/cherrygrove-nurse-culling-2.5d.png`. The shared sprite publication also
makes her visible in this build's 2D capture; it does not change gameplay
object allocation. A full `LDRLRR` stair round trip from `(2,5)` retained the
nurse after returning (`/tmp/cherrygrove-stairs-fixed.png`), with two terrain
builds for floor changes, median 16.85ms/p95 19.84ms over 192 frames. Both
processes validated their PNGs and exited 0. The earlier nurse defect is
resolved for these reproduced positions and the stair round trip. Two Rust
builds were needed: the first exposed the remaining upstream snapshot filter;
the second shared the existing presentation expansion. No tests were run.

## NPC world orientation — superseded after user review

NPC (`VisualActorId::Object`) cards now use the authored default world pose
for both rotation and terrain-clearance displacement. Camera orbit no longer
turns or slides them. Gameplay facing and animation still supply the sprite.
Player, remote-player and effect presentation retain their existing behavior.
One native renderer build was necessary for this Rust transform change; no
tests were run. Inspected CherrygrovePokecenter1F at (6,7) front-on and at
45 degrees using the same live process. The nurse and room NPCs remain aligned
with the room; flat cards naturally foreshorten from the side. The warm paired
45-degree capture completed in 1.21 seconds without rebuilding. Evidence:
`/tmp/npc-world-anchor-front.png`, `/tmp/npc-world-anchor-orbit45.png`.
Removed the unaccepted Route 29 grass-tuft trial, which erased too much grass;
the canonical profile set returns to 89 objects.

## Upright directional NPC correction

The user correctly rejected the first world-pose fix: it retained a 45-degree
lean and showed the same source face from behind. Replaced it with upright,
foot-anchored directional sprites. The publisher chooses among the source
front/back/left/right standing and action frames using gameplay facing plus
camera azimuth; it does not change gameplay facing. The card rotates only
around world Y and has no camera-directed position offset. Its world height
compensates for the fixed camera pitch to preserve source sprite proportions.
Directional animation mirroring reads the current movement phase instead of a
value retained at entity reconciliation. Four source directions mean discrete
view changes, not invented eight-direction artwork. Static props with only one
source image still have only that image.

Initial live render confirmed actual nurse back art at orbit 4 and side art at
orbit 2; it exposed compressed proportions, prompting the height correction.
Those initial images are `/tmp/npc-upright-0.png` and
`/tmp/npc-upright-180.png`; they are not final height evidence.

Final corrected executable built successfully. Inspected the same stationary
CherrygrovePokecenter1F (6,7) at zoom 0 and orbits 0,1,2,4 in one live process.
NPCs remain upright; the nurse shows front, side and back artwork as the camera
moves around her. The counter appropriately occludes her lower body from the
front. Final images: `/tmp/npc-directional-final-0.png`,
`/tmp/npc-directional-final-45.png`, `/tmp/npc-directional-final-90.png`,
`/tmp/npc-directional-final-180.png`. Warm paired captures took 1.16–1.20s;
no builds between angles and no tests. Player and remote-player presentation
are not part of this NPC correction and retain the prior camera-facing behavior.

## New Bark neighbor-house foliage depth

Reviewed PlayersNeighborsHouse (3,5) with the current NPC executable and live
profiles. Rejected unmasked foliage: it introduced a rectangular striped slab.
Rejected a uniform four-pixel extrusion: its top and sides read as a block.
Retained four masked four-row crown bands with depths 0.5,2,3,1.5 source pixels,
centered over the pot rather than aligned to its front edge. Centered the stem
at the same depth. Only the two PlayersNeighborsHouse profiles changed; the
Cherrygrove copies still require their own review. The outline remains visibly
pixelated and planar, but the crown has less slab-like depth at an angle.
Inspected at zoom 0, orbit 0 and 0.5. Evidence:
`/tmp/newbark-foliage-layered-front.png` and
`/tmp/newbark-foliage-layered-angle.png`. Final capture times 1.12–1.24 seconds.
No cargo, builds, or tests during this profile pass. Canonical count remains 89.

## New Bark bedroom and stair traversal

Reviewed PlayersHouse2F at walkable (3,3), including the bed, table, computer,
wall furniture and north-east stair opening. (5,4) is table collision and was
rejected by the location loader before rendering. Paired evidence:
`/tmp/newbark-bedroom-review-2d.compare.png`.

Rendered `RRRRUUU` from (3,3): CSV confirms PlayersHouse1F at frame 158 and
return to PlayersHouse2F at frame 196, because continued Up re-enters the stairs.
Inspected the downstairs moving frame and final upstairs landing. 224 frames,
two additional terrain builds, median 16.76ms, p95 20.00ms; exited successfully.
Evidence: `/tmp/newbark-bedroom-stairs.png`, its moving-05 PNG and movement CSV.

Rendered `RRRRUDDDD` to leave the lower landing: final CSV map PlayersHouse1F,
with the room and both NPCs present in `/tmp/newbark-downstairs-settled.png`.
The ordinary MeetMomScript starts and its dialogue is visible; this is not an
uninterrupted free-walk result. 288 frames, one additional terrain build,
median 16.91ms, p95 33.67ms; exited successfully. No cargo or tests in this pass.
This closes the initial bedroom/stair visibility check, not the broader
remaining terrain and presentation review.

## Route 29 source-shaped grass tufts

The earlier ground-color mask erased the blades because grass shares the
underlay palette. Added 16 exact metatile-$03/subtile placements for Route29,
each using 11 nonoverlapping source-pixel row spans from tile $04. These retain
the two blade tones without erecting the pale background. Thin 0.5-pixel depth,
seven-pixel silhouette height, and a common foot plane replace the shallow
legacy grass extrusion. All changes are JSON profiles using the Rust mesher.

Compared original and new geometry at identical zoom 0/orbit 0.5 in Route29
(47,9), restoring the original profiles for the comparison. Retained the new
upright tufts and inspected the front view as well. Evidence:
`/tmp/route29-thin-tufts-angle.png` and
`/tmp/route29-terrain-pass-2d.compare.png`. Warm reload captures took 1.83–1.85s.
No cargo or tests. Canonical profile count is now 105. The geometry is scoped to
Route29; other routes retain their existing grass. Broader terrain-edge and
movement review remains open.

## Route 29 grass movement follow-up

Rendered `DDDDLLLLRRRRUUUU` from Route29 (47,9) with the 105-object canonical
profiles. The run remained on Route29, crossed 17 published grid origins, and
finished successfully: 512 frames, two additional terrain builds, median
16.85ms, p95 29.73ms. Boundary bumps occurred, so this is not an uninterrupted
path traversal. Inspected `/tmp/route29-grass-walk-moving-01.png` and final
`/tmp/route29-grass-walk.png`; the moving player and nearby NPC remain visible,
with grass partially occluding the player's lower legs. CSV evidence is
`/tmp/route29-grass-walk.movement.csv`. No cargo or tests. Dense background
forest still reads as a tall textured wall and remains a visual priority.

## Route 29 forest height trial — rejected

At Route29 (47,9), compared tall-tree height 22/depth 3 against the retained
height 32/depth 2 through live reload at zoom 0/orbit 0.5. The shorter trial
made foreground trees visibly squat without resolving the repeated distant
forest. Reverted both profiles and confirmed revision 3 applied, restoring the
105-object canonical set. Rejected evidence:
`/tmp/route29-canopy-rejected-22.png`; matched comparison:
`/tmp/route29-canopy-2d.compare.png`. Reload/capture took 1.88–1.90 seconds.
No cargo or tests. Investigation confirms the expanded visual world includes
a 32-tile halo and the outdoor camera has a plain clear color, with no distance
fog configured. Distance treatment deserves review before further distorting
individual source trees; this is a next-step hypothesis, not a completed fix.

## Reloadable outdoor distance haze

Added map-scoped atmosphere settings using the existing Bevy fog pass in the
palette surface shader. One successful renderer build installed the capability;
subsequent tuning reused the same process. Object equality now controls terrain
revision changes, so atmosphere-only reloads do not rebuild geometry.

On Route29 (47,9), inspected strong haze (start 4/end 24/opacity .85), gentler
haze (8/32/.75), and haze disabled at identical zoom 0/orbit .5. Retained the
gentler setting: distant repeated forest rows recede while the nearest trees,
path and player remain crisp. Evidence `/tmp/route29-haze-retained.png` and
`/tmp/route29-haze-off.png`. Live captures took 1.37–1.48s, all at geometry
revision 0, with no additional mesh-applied messages. Invalid end distance 1
was rejected and a new capture retained the previous haze; restored the valid
canonical document afterward. No tests were run. Settings currently list
Route29, NewBarkTown and CherrygroveCity; the latter two still need render review
with haze, and an interior exclusion check remains to be captured.

## Atmosphere town and interior follow-up

Inspected canonical haze in NewBarkTown (4,12) and CherrygroveCity (13,9).
Nearby buildings, paths, shoreline and beveled coastal rocks stay distinct;
the distant tree rows recede. Paired evidence:
`/tmp/newbark-haze-review-compare.png` and
`/tmp/cherrygrove-haze-review-compare.png`. Both one-shot captures exited 0.

In PlayersNeighborsHouse (3,5), captured canonical haze settings then changed
only global haze opacity to zero in the same live process. Decoded RGB images
were pixel-identical (difference bounding box None), confirming exclusion for
this interior. Inspected the comparison as well:
`/tmp/interior-haze-exclusion-2d.compare.png`. Warm capture 1.25s; geometry
revision stayed 0. Restored canonical opacity .75 afterward. No cargo or tests.

## Route 29 western approach and Cherrygrove crossing

Inspected the paired current render at Route29 (3,6): sign, path and bordering
trees remain distinct with the canonical grass/haze profiles. Evidence:
`/tmp/route29-west-current-compare.png`.
Rendered `LLL` from that location. CSV confirms CherrygroveCity at frame 34;
the final image shows the player beside the guide, with Pokémon Center,
shoreline and forest intact. A bump occurs at the end near the guide. Run:
96 frames, 11 grid origins, one additional terrain build, median 16.88ms,
p95 31.82ms, clean exit. Evidence `/tmp/route29-west-crossing.png` and
`/tmp/route29-west-crossing.movement.csv`. No cargo or tests. This verifies
the western connection, not every ledge in Route29's interior.

## Route 29 central ledge follow-up

Inspected Route29 (25,8) head-on and at zoom 0/orbit .5 with current profiles.
The horizontal raised face reads clearly, but a thin vertical seam is visible
at its right corner adjoining the perpendicular terrain. Evidence:
`/tmp/route29-central-review-2.5d.png`. This seam remains unresolved.
Rendered `DD`: the normal ledge-jump sound triggered and the final image places
the player below the ledge with terrain intact. 64 frames, five grid origins,
zero additional terrain builds, median 17.00ms, p95 31.69ms, successful exit.
Evidence `/tmp/route29-central-ledge-jump.png` and its movement CSV. This final
landing capture does not prove every in-flight jump frame is correct. No cargo
or tests. A transient process-creation failure was retried only after confirming
the repository and existing viewer were still present.

## Central ledge seam diagnosis

Close render at Route29 (27,8), zoom 3/orbit .5, resolves the previously reported
thin seam into an exposed textured end face. The raised cap uses
JUMP_LEDGE_HEIGHT=6 native pixels; adjacent flat ground leaves a height
termination that append_exposed_sides closes. This is not evidence of missing
triangles. Evidence `/tmp/route29-ledge-seam-close-2.5d.png`, capture 1.59s.
A smoother endpoint should reconcile the cap with the adjacent ground (for
example a local taper conditioned on the actual neighboring surface), rather
than place an overlay over the sidewall. Do not globally taper every metatile
$57 edge: continuous neighboring ledge blocks must retain their shared height.
No geometry edits, cargo or tests in this diagnosis pass.

## Route 29 ledge endpoint taper

Added a Rust shape-resolution pass for Route29's exact $57 grassy cap: only
its last source column, above the south-facing lip, tapers eastward when the
neighboring resolved surface is Flat. Adjacent raised ledge blocks retain
height 6, and the source lip stays intact. Uses existing RampEast geometry
and footing handling. One successful renderer build installed the change.

Inspected Route29 (27,8) at zoom 3/orbit .5 against the previous close-up.
The long textured end-face line is removed, replaced by a short grassy slope;
the player follows the lower footing at that edge. Also inspected zoom 0/orbit
0: the continuous south ledge face is intact. Evidence:
`/tmp/route29-ledge-taper-close.png` and
`/tmp/route29-ledge-taper-2d.compare.png`. Warm captures 1.35–1.40s, no builds
between angles, no tests. This addresses the observed long end-face line;
movement across the new slope remains to be inspected.

## Ledge taper movement check

Rendered `RLLR` from Route29 (26,8) using the corrected executable. Inspected
`/tmp/route29-taper-walk-moving-00.png` at the tapered edge and final
`/tmp/route29-taper-walk.png`: the player remains above the surface, the grassy
slope joins the flat side, and the continuous front lip remains visible.
The run hit nearby collision twice; it was not an unrestricted traversal.
128 frames, six grid origins, zero additional terrain builds, median 16.83ms,
p95 30.35ms, clean exit. No cargo or tests. This is sampled rendered movement
evidence, not a claim that every animation frame was inspected.

## Cherrygrove house plant consistency

Applied New Bark's retained layered crown and centered stem to the six exact
matching plant profiles in GuideGentsHouse, CherrygroveGymSpeechHouse and
CherrygroveEvolutionSpeechHouse. Source tile matrices were checked equal before
copying the part definitions. Inspected paired renders in all three rooms at
(3,5); foliage, pots and room furniture remain intact. The evolution-house spawn
overlaps a resident, so that image does not validate actor spacing. Evidence:
`/tmp/cherrygrove-guide-plants-compare.png`,
`/tmp/cherrygrove-gym-plants-compare.png`,
`/tmp/cherrygrove-evolution-plants-compare.png`.
All captures exited successfully using the existing executable; 2.5D capture
phases took 2.09–2.19s. No cargo or tests. Profile count remains 105.

## Cable Club booth partitions

At Pokecenter2F (4,4), the divider source drawings still read as floor strips
at 45 degrees. Added three complete two-block profile matches for $31/$32,
$0b/$28 and $0b/$0f, claiming only the right two source columns. Fold the upper
24 source pixels across a 48-pixel top and the remaining drawing into a
16-pixel upright front. This produces closed booth partitions while preserving
adjacent counters, PC and wall displays. Rejected the first 24-pixel-high trial
as too tall for this room; retained height 16.

Inspected final front and 45-degree views; the partitions separate the booths
and attendants remain visible. Evidence `/tmp/pokecenter-booths-raised-angle.png`
and `/tmp/pokecenter-booths-2d.compare.png`. Warm profile captures 1.24–1.27s;
front recapture 1.16s. No cargo or tests. Canonical profile count is now 108.
The dark sides use the source outline; further side-surface art refinement is
possible, but these no longer lie visually on the floor.

## Local-player directional presentation

Extended the upright, foot-anchored NPC transform to the local player. Retain
all four standing/action views alongside the classic player frames, refreshing
them in both the full scene and retained-player facing paths. The optional
publisher selects the view relative to world facing and camera orbit, preserving
walking stride mirroring. Remote players and transient effects remain separate.
One native build was required for this Rust presentation change (about six
minutes); no tests or further builds during capture review.

Inspected Route29 (25,8) at orbit 0,1,2,4 with fixed zoom 0. Front, side and
back artwork now follow the world direction and the player stays upright.
Evidence: `/tmp/player-upright-front.png`, `-angle.png`, `-side.png`, `-back.png`.
Warm paired captures took 1.40–1.44s. The 90-degree view also exposes a remaining
scenery defect: thin tree profiles collapse into panels. This needs improvement;
these captures do not establish final scene quality.

Rendered RLU movement from the same location with the existing binary. Inspected
`/tmp/player-directional-walk-moving-00.png` and the final
`/tmp/player-directional-walk.png`: the player changes from a side view to a
back view when heading north and remains above the terrain. Final position is
immediately behind the resident, so this is not proof of actor spacing in all
views. 96 frames, seven grid origins, zero additional terrain builds, median
16.81ms, p95 31.82ms; clean exit. Bike/surf, remote players and every individual
walking frame were not visually verified in this pass. Toolkit criteria updated.

## Ecruteak first pass and Burned Tower investigation

User queued Cinnabar after Ecruteak; returned the live viewer to Ecruteak.
Added the exact johto $0a rounded rock profile for EcruteakCity and included the
city in gentle outdoor haze. Central plaza (18,22) front/22.5-degree renders
show the pond boulder rounded instead of a cube, with near actors legible.
Evidence `/tmp/ecruteak-before.png`, `/tmp/ecruteak-rock-angle.png`,
`/tmp/ecruteak-review-2.5d.png`; warm front capture 1.35s, no build or tests.

North at (7,7), the Burned Tower doorway is stretched onto the forecourt.
A complete $20/$21;$37/$3b live override with separate roof/wall/rock parts
produced duplicate roof surfaces and did not remove the stretched drawing.
Rejected and removed that override; evidence `/tmp/ecruteak-tower-rejected.png`.
Baseline evidence `/tmp/ecruteak-tower-before.png`. Do not treat the existing
placement-only Rust test as evidence of correct rendering. Next investigation
must trace the compiled building drawing/terrain sampling, including where
source pixels are shared or replaced. Canonical count is 109; no rejected tower
profile remains. Ecruteak is not complete, and Cinnabar remains queued.

## Haze removed everywhere at user request

Removed the entire canonical atmosphere setting, disabling haze on every map.
Live Ecruteak reloaded without a geometry rebuild (revision stayed 3); inspected
`/tmp/ecruteak-north-2.5d.png` with clear distant trees. Capture pair 1.38s.
This supersedes all earlier accepted haze treatments. Toolkit records the user's
no-haze preference. Also removed the rejected forecourt profile so the valid
configuration can reload. No cargo or tests. Existing compiled fallback still
reflects its last build; all current live sessions must use the canonical profile
path, and any subsequent packaged build must use this haze-free configuration.

## Voxel trees removed globally at user request

Removed all 41 tree profiles (68 remaining object profiles). The compiled mesher
now renders complete tree drawings flat using their source footprint and flattens
individual Tree/CutTree shapes. Other props sharing grouped-mesh utilities remain
separate. Rebuilt the native capture executable once; its embedded profiles now
also omit haze. No tests run.

Inspected `/tmp/ecruteak-flat-trees-2.5d.png`,
`/tmp/newbark-flat-trees-2.5d.png`, `/tmp/route29-flat-trees-2.5d.png`.
Tree drawings remain on the map without raised trunks/crowns; all three views
are haze-free. New Bark and Route29 captures omitted the live-profile environment
variable, verifying updated embedded defaults as well as Ecruteak's live path.
Both one-shots exited cleanly. This supersedes earlier voxel-tree treatments.
Burned Tower's separate forecourt/building defect remains unresolved.

## Corrected tree interpretation: upright 2.5D cutouts

User clarified that trees must remain 2.5D; flattening them was incorrect.
Restored the compiled upright path with rounded=false and zero card thickness.
Individual tree/cut-tree cards omit extrusion. Restored the 41 exact tree
profiles with every depth set to zero, including small variants not covered by
the compiled grouping. Canonical count 109; atmosphere remains absent.

Built once for the compiled path. Ecruteak plaza and Route29 front/45-degree
renders inspected. The first Route29 capture exposed remaining flat small-tree
variants; restored zero-depth profiles and verified the live angle capture at
`/tmp/route29-cutout-trees-2.5d.png`. Trees stand upright without voxel sidewalls.
Live profile review 1.68s; camera-only 1.31s. No tests. The latest live profile
restore postdates the build, so use the documented profile environment for review
until the next necessary packaged build. This supersedes the flat-tree entry.

## Burned Tower forecourt corrected

Confirmed the foreground slab was the separate compiled johto $74/$75 building
template, not the tower's own $20/$21;$37/$3b facade. Added an exact Ecruteak
$74/$75 profile that preserves the full source drawing on the ground. Its outer
fold uses top=0/depth=0; a full-top compound part supplies the actual flat surface
(the earlier failed trial incorrectly set the outer fold to the full height).
The tower doorway and wall are now fully exposed and upright. No renderer build.

Inspected front and 45-degree views at (7,7):
`/tmp/ecruteak-forecourt-front.png`, `/tmp/ecruteak-forecourt-2.5d.png`.
The ground lip remains in the source position; the false facade-covered slab is
gone. Warm angle pair 1.31s. Canonical count 110, haze absent, upright zero-depth
tree profiles retained. This resolves the visual diagnosis in earlier entries;
entrance traversal and Ecruteak's other landmarks/interiors still need review.
Angle inspection caught coplanar flicker between the replacement ground and the
full-top drawing. Set the drawing base to 0.05 source pixels and recaptured;
the ground lip is continuous in the final angle image. Warm profile pair 1.81s.

## Burned Tower entrance movement

Rendered UU from EcruteakCity (5,7) using the current live profiles and existing
binary. Crossed into BurnedTower1F and reached the normal Eusine introduction
(`BurnedTower1FMeetEusineScript`, first dialogue page). Inspected final interior
capture `/tmp/ecruteak-tower-entry.png`; this verifies the entrance transition,
not uninterrupted interior traversal or completion of the dialogue. 64 frames,
four grid origins, one additional terrain build, median 18.82ms, p95 52.44ms.
Clean exit. No cargo or tests.

## Dance Theater stage footing

At (5,10), paired render `/tmp/ecruteak-dance-compare.png` exposed dancers
embedded in the raised stage. The stage is map-scoped RaisedTop/Prop at eight
source pixels, while footing extraction only elevated banks and ledges. Added
an exact Dance Theater stage-shape branch to use its authored surface height.
One build completed in about 34s, also refreshing current embedded profiles.
Inspected `/tmp/ecruteak-dance-footing-2.5d.png`: dancers now stand above the
stage, and audience/player remain on the lower floor. Capture clean exit,
2.20s 2.5D phase. No tests; movement onto the stage not yet reviewed.

## Ecruteak Center furniture

Inspected EcruteakPokecenter1F (6,7), then added four exact map-scoped copies of
retained Cherrygrove Center profiles: complete PC, healing machine and two seat
variants. Source tiles/metatile requirements remain exact. Before:
`/tmp/ecruteak-center-2.5d.png`; after:
`/tmp/ecruteak-center-furniture-2.5d.png`. PC and machine now have closed depth;
seats remain intact and nurse visible behind the counter. Foreground player is
partly at the image edge, so this framing does not validate full player visibility.
Paired capture 2.15s 2.5D phase, clean exit. No cargo or tests. 114 profiles.

## Ecruteak Mart furniture

At (3,5), refrigerators remained flat and shelf drawings were thin upright
panels. Added five exact Ecruteak-scoped copies of the reviewed Cherrygrove Mart
profiles. Inspected before `/tmp/ecruteak-mart-2.5d.png` and after
`/tmp/ecruteak-mart-furniture-2.5d.png`: refrigerator faces stand upright, shelf
caps and sides are closed, counter and visible actors remain intact. Paired
capture 2.12s 2.5D phase, clean exit. No build or tests. Canonical count 119.
This is room-framing evidence; aisle movement remains to be reviewed.

## Ecruteak Mart movement evidence and harness defect

URRDDLL from (3,5) and RRLL from (5,6) exited 101 at render_walk.rs:99 because
the grid origin did not change. Do not record either as a successful harness run.
Inspection of `/tmp/ecruteak-mart-front-walk-moving-00.png` and `-moving-02.png`
shows the player changing position in the front aisle, clear of the shelving.
The room camera is fixed: grid-origin cardinality is not a valid proxy for player
movement indoors. The second run recorded 128 frames, zero additional terrain
builds, median 16.84ms, p95 18.28ms and collision bumps near a resident/boundary.
Next tooling fix should measure player position as well as map/grid transitions,
while still rejecting a route that only turns in place. No build or tests here.

## Fixed-camera movement check corrected

Updated render_walk to accept a full source-tile span of player translation
relative to map center, alongside existing grid/map transitions. This avoids
false failure in small rooms while requiring more than a facing/stride change.
Built the capture executable and reran identical RRLL from EcruteakMart (5,6).
It now exits 0 with one grid origin and a final render. Inspected
`/tmp/ecruteak-mart-walk-fixed.png`; player remains clear of the shelves.
128 frames, zero additional terrain builds, median 16.87ms, p95 18.17ms.
Collision bumps still occurred, so this is not an unrestricted path claim.
No test suite run. Toolkit criterion updated; embedded profiles now current119.

## Ecruteak Gym initial review

Rendered EcruteakGym (4,15), paired evidence `/tmp/ecruteak-gym-compare.png`.
The central hidden-path region remains visually uniform; no raised geometry
exposes the invisible route in this front view. Side walls and northern platform
are raised, trainers remain visible. Entrance statues are still flat and need
an authored upright treatment. Do not call this a completed gym review: angled
views, path/reset movement and statues remain outstanding. Clean exit, 2.20s
2.5D phase. No build or tests.

## Ecruteak Gym entrance statues

Added exact tower $12/$13 profiles for the two 16x32 statue drawings, ground
$02, top fold4/depth8 with source-ground masking. Inspected front and 45-degree
views at (4,15): statues now stand upright with closed sides, their floor copies
are removed, and the central hidden-path region stays visually uniform.
Evidence `/tmp/ecruteak-gym-statues-2.5d.png` (final angle) and paired compare.
Warm camera capture 0.89s. No cargo or tests. Canonical count121. Hidden-path
movement/reset remains unverified; this is geometry and framing evidence.

## Ecruteak Gym hidden-floor reset capture

Rendered UUR from (5,13) using existing executable/live profiles. Kinesis reset
sound occurred and the run reached the normal EcruteakGymClosed dialogue stating
Morty is absent. Final `/tmp/ecruteak-gym-reset.png` inspected at the entrance;
the hidden floor is unmarked; dialogue covers the statues, so this capture does
not revalidate them. 96 frames, three grid
origins, zero additional terrain builds, median16.68ms/p95 17.00ms, clean exit.
This validates the sampled reset/entrance presentation in this story state, not
a complete gym route or trainer battle. No cargo or tests.

## Wise Trio room and stair review

Inspected paired WiseTriosRoom (4,5), `/tmp/ecruteak-wise-compare.png`:
partitions stand above the floor, stair opening remains visible, residents render.
Then rendered L from (3,4), crossing the stair transition; inspected final
`/tmp/ecruteak-wise-stairs.png`. 32 frames, two grid origins, one additional terrain
build, median16.66ms/p95 17.34ms, clean exit. No geometry edits were needed in
this sampled route. No cargo or tests. This covers one stair direction, not every
scripted access condition or return trip.

## Ecruteak residential review

Inspected paired (3,5) renders for EcruteakItemfinderHouse and
EcruteakLugiaSpeechHouse. Both use the same complete traditional-house block
layout. North wall/furniture course, raised table, cushions and door framing
remain coherent; residents render. Evidence `/tmp/ecruteak-itemfinder-compare.png`
and `/tmp/ecruteak-lugia-house-compare.png`. No edits made from these front views;
angled table/book treatment and movement are not established by this review.
Both one-shots exited cleanly using the existing binary/live profiles. No cargo
or tests. Cinnabar remains queued after the Ecruteak pass.

## Ecruteak eastern approach

Paired render at (34,9), `/tmp/ecruteak-east-compare.png`, exposes a remaining
Tin Tower exterior issue: lower entrance art appears horizontal beneath the
raised storeys. This is distinct from the corrected Burned Tower forecourt.
Do not call the Ecruteak exterior complete until the lower tower template is
traced and corrected. Trees are upright cutouts and haze absent in this view.
Clean capture exit,0.97s 2.5D phase, no build/tests.

## Eastern tower entrance row included

The johto tower catalog template claimed three $24/$25 courses but omitted its
$28/$29 entrance row. Extended the complete template to include that row, so the
existing building mesher folds its doorway upright with the tower. Built native
capture executable; inspected same (34,9) paired view at
`/tmp/ecruteak-east-door-compare.png`. Door now joins the upright lower facade;
its horizontal duplicate is gone. Existing forecourt ground profile remains.
Capture exited0,1.04s 2.5D phase. No tests. Full-height/angled framing and exterior
entrance traversal remain separate checks; current view crops upper storeys.

## Eastern tower doorway movement

Rendered UU from EcruteakCity (37,9) with current executable/profiles. Entered
Tin Tower interior with normal door sound/music change; final
`/tmp/ecruteak-east-entry.png` inspected. 64 frames, five grid origins, one
additional terrain build, median16.70ms/p95 17.31ms, clean exit. This verifies
the corrected doorway transition, not the entire tower climb. No cargo/tests.

## Cinnabar signs stand upright

Captured CinnabarIsland (10,11) with the existing executable and canonical
profiles: `/tmp/cinnabar-baseline-compare.png`. The two visible signs were
foreshortened against the terrain. Added exact Kanto $08/$79 sign profiles,
source tiles $46/$47/$56/$57, one-pixel depth and ground masking. Inspected
`/tmp/cinnabar-signs-2.5d.png`: both signs now stand upright with readable faces
and feet on the terrain; the Center facade remains coherent. Paired capture
completed in 1.76 seconds, clean exit, 123 profiles applied. No cargo or tests.
This front view does not verify every Cinnabar angle or its interior.

## Cinnabar Center furniture

CinnabarPokecenter1F uses the same source PC, healing machine and seat metatiles
as the reviewed Ecruteak Center. Added four map-scoped profiles using those
exact source patterns. Inspected before/after at (5,6):
`/tmp/cinnabar-center-before-compare.png` and
`/tmp/cinnabar-center-furniture-2.5d.png`. The PC and machine now have complete
solid bodies; seats have closed sides. Nurse remains visible above the counter,
player is framed, and room floor remains readable. The lower seat row is
partially cropped. Capture exited cleanly in 1.76 seconds with 127 profiles.
No rebuild or tests. This verifies the front view, not room traversal.

## Cinnabar Center movement

Rendered UULL from CinnabarPokecenter1F (5,6), inspecting the first moving
capture and final `/tmp/cinnabar-center-walk.png`. Player moves north through
the open aisle and west along the counter, shows back/side facing appropriately,
and remains upright. Final position is at the left room boundary; bump sounds
occurred during held input. Furniture remains stable throughout the sampled
views. 128 frames, one fixed grid origin, zero additional terrain builds,
median 16.67 ms/p95 17.41 ms; clean exit. No cargo/tests. This does not cover
walking between both seat rows or a healing interaction.

## Cinnabar northern rocks: live geometry pass

At CinnabarIsland (11,5), orbit 1 exposed shoreline rocks as thin upright cards.
Added 17 exact source-pattern placements across the island's Kanto rock
metatiles. Rejected the first straight extrusion because it produced boxy pale
sides; retained a full 16px footprint with 8px height and 4px bevel instead.
Inspected `/tmp/cinnabar-north-2.5d.png` after revision 4 (144 profiles): rocks
now retain volume at 45 degrees. Pale lower edges still merit refinement;
distant map-border drawings remain flat. Trees were not edited. Each warm
capture took approximately 0.92 seconds in the same process, no cargo/tests.
An intermediate non-atomic JSON write was rejected safely; subsequent writes
used atomic replacement and revisions applied successfully.

## Cinnabar rock footprint refinement

Reduced the bevel source rectangle to [1,1,14,12], depth12/height7/bevel4,
removing the broad pale bottom padding from the raised shape. Revision5 at
(11,5), orbit1 inspected in `/tmp/cinnabar-north-2.5d.png`: separated rock
silhouettes now meet the grass with substantially smaller pale edges. An
intermediate ground-mask trial was rejected by profile validation (masked
bevels are unsupported); it was replaced with this valid rectangle. Warm
capture0.92s, same live process, no rebuild/tests. Distant border art remains
flat and the southern shoreline has not yet been reviewed with this revision.

## Cinnabar south framing and bevel documentation

Captured CinnabarIsland (10,15) with 144 current profiles, paired output
`/tmp/cinnabar-south-rocks-compare.png`, clean exit in1.79s. This location starts
surfing. Center facade/signs and west bank rocks remain coherent, but the south
rock row is clipped by the bottom frame; this capture cannot establish its full
silhouette quality. Surf art remains visible on the water; no surf movement or
camera rotation was evaluated. Added actual bevel validation constraints and
compound-mask independence to RENDER_TOOLKIT.md to avoid repeating the rejected
masked-bevel trial. No cargo/tests.

## Southern rock row: wider angle exposes ground issue

Position changes at the map edge do not widen the clamped camera. Used live
CinnabarIsland (10,15), zoom0/orbit1 instead. Full southern row has beveled
volume, but its replacement floor forms a green strip. Trial ground20 on
$14/$6b instead of44 caused the row to render flat again; rejected and restored
44. This is unresolved, not an accepted water-ground fix. The matching and
water/terrain handling need inspection before another profile trial. Both
captures completed in approximately0.9s without builds/tests. Current live
output may still show the rejected capture until another request.

## Water-backed live profiles resolved

The ground lookup in mesh/live.rs accepted only CellShape::Flat, causing all
profiles selecting water20 to be skipped. It now accepts Flat or Water. Set
Cinnabar $14/$18/$6b rock profiles to water20, built the native executable once,
and inspected (10,15), zoom0/orbit1 in
`/tmp/cinnabar-water-fixed-2.5d.png`. Southern and distant western rows retain
beveled volume with blue ground replacing the erroneous green strip. Pale
source-art rims remain visible; water replacement currently samples the tile
through the profile texture path. Warm capture0.89s. No tests.

## New Bark current-renderer review and loop cleanup

Inspected `/tmp/newbark-current-review-compare.png`, NewBarkTown (8,8), current
144 profiles and water-ground renderer fix. Visible trees remain upright,
facades and signs retain their raised presentation; this framing crops the
southern buildings and is not a full-town audit. Initial paired run3.18s.
Found three task-owned completed Cinnabar live capture processes still active;
terminated their exact PIDs25089/25312/25418 and reran the same New Bark view
as `/tmp/newbark-single-renderer-compare.png`. Added a one-live-renderer rule
to the toolkit to avoid repeated background rendering overhead. No build/tests.

## New Bark southern full-building angles

Current executable/profiles, NewBarkTown (9,13), zoom0. Inspected 45-degree
`/tmp/newbark-south-45.png` and opposite orbit4 in
`/tmp/newbark-south-angle-2.5d.png`. Southern roofs, closed side/rear walls and
front doorways are fully visible in the angled frame. Visible actors remain
upright and opposite view shows player back art. The chosen spawn overlaps a
sign, so the sign occludes part of the player; this is not evidence of a valid
walking position. Use a clear road tile for subsequent movement review.
Both warm angle captures0.90s, no rebuild/tests. One live renderer remains.

## New Bark southern road movement

Closed the prior live review process and rendered DDRRLLUU from (7,10).
Inspected `/tmp/newbark-road-walk-moving-02.png` and final
`/tmp/newbark-road-walk.png`: player traverses the southern road upright,
returns toward the central sign, and stops in front of it instead of spawning
inside it. Sampled views show coherent buildings and trees while the camera
scrolls. Held input produced bumps; this is not a claim that every segment
completed unobstructed. 256 frames,18 grid origins,zero additional terrain
builds,median16.68ms/p95 17.19ms,clean exit. No cargo/tests. No live review
process intentionally retained after this one-shot.
