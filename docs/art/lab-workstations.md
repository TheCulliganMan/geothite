# Oak and Elm laboratories

Oak's and Elm's back-wall workstation drawing contains a compact CRT on the
left, a separate centre unit, and a tall right enclosure, with low front
controls on a gold workbench. The old shared computer mesh supplied a single
wide monitor, keyboard, and an unrelated tower underneath the desk.

The native Rust assembly in `native_lab_workstation_model.rs` reconstructs
these three desktop volumes. It fits the existing 32-pixel source width,
12-pixel physical depth and 22-pixel height. The worktop is 8.5 pixels high;
the left CRT reaches 19, the centre unit 16.5 and the right enclosure 22.
Small chamfers finish the closed housings. Keys, CRT strokes and enclosure
controls have their own shallow relief. Grey equipment, pale controls, dark
bezel and gold joinery sample the live source atlas; no copied image or new
compiled model asset is embedded.

Only the complete authenticated 4×3 drawing in `OaksLab` and `ElmsLab` uses
this assembly. Atlas, blocks, local phase, tile indices and selected material
contract are checked before replacement. All twelve source claims, existing
floor underlay, support heights and custom-profile priority remain. Other
computer placements, including mansion workstations, keep their selected
original mesh. The model cache is immutable.

Elm's three starter balls are authoritative object actors on the separately
modeled display table. They must not be suppressed by a room-name rule.
Removing that suppression restores the ordinary prop path and its existing
source visibility and real-table support. A removed object retires its model;
no fixed decorative balls are added to the table or game state.

Verification uses the ignored local native content pack with
`CRYSTAL_RENDER_TEST_PACK` set to its absolute path. The workstation, Elm and
item-ball test selections pass: ten distinct checks, eleven executions. A
separate supplied-coat test remains explicitly ignored for its external rig.
Tests cover closed outward surfaces at three scales, source mutations,
custom ownership, support, real-pack placement, all three balls at the actual
tabletop height, removal of one ball, and ordinary item-ball placement.

The terrain audit passes 388 maps and 288 scripted states with zero mesher
errors. Only Oak and Elm change: each loses 1,332 triangles and 3,996 vertices
when the generic computer is replaced. Their source coverage, family bounds,
floor and support fields are identical; the other 386 maps and all scripted
state records are unchanged. This audit covers terrain, not actor visibility.
Headless paired native/source captures separately verify that visibility.

Evidence is under `target/kanto-resume/lab-assembly-build` and
`target/kanto-resume/lab-followup-review`. Every final pair holds one actual
session across views. Different before/after captures have different idle NPC
phases, so whole-image before/after pixel identity is not claimed. Whole-room
actor and gameplay acceptance remains separate from these repairs.


## Wall, window and perimeter finish (2026-10-19)

The two labs now use their own complete wall and pane drawings instead of the
shared blue clinical panel and aqua domestic window. `native_lab_walls.rs`
authenticates the 4×2 wall drawings in blocks08/0d/17 and the two distinct
2×2 panes in blocks09/20, including the complete four-column window profile.
Other lab-tileset maps keep their existing treatments. Altered atlas/block/
phase/tile identities, crops, raised placements and custom owners decline the
native replacement.

A closed 32px wall retains one 16px source course near the ceiling and a plain
cream 16px course underneath. The checker band is not repeated over the lower
wall. Both source pane patterns remain at native width; their 14×9px frames
project 0.4px with finished edges. The complete source face is partitioned,
rather than overlaid on a second face. Floor underlay and support are unchanged.

The same source-painted course continues behind authenticated rear furniture
and around the perimeter. Freestanding bookcases receive no backing. A two-
column perimeter donor keeps the source checker pitch; partial door/corner
sections crop its continuous source interval instead of compressing the checks.
Tests measure the world-to-atlas pitch for 3px, 11px and 16px sections. Ordinary
room boundaries, warp apertures and camera cutaway behavior are preserved.

Sixteen distinct focused checks pass in seventeen executions. They include
source-field mutations, custom ownership, closed wall bounds/outward winding
at three scales, one complete front area, shallow frame relief, native checker
pitch, source-painted rear backing, previous workstation/ball controls, and
all 255 room perimeter donors. The separate supplied-coat test is explicitly
ignored for its external rig. A stale broad Vermilion assertion was corrected
to require its exact 46 authored header cells and 32 permanent checker floor
quads; all other wall vertices must still belong to cutaway sections. Its three
existing stepped-boundary regressions pass. No Vermilion production code changed.

Final headless native/source pairs for Oak, Elm and Elm's opposite side approach
are in `target/kanto-resume/lab-walls-review/final`. All six full originals were
individually inspected, and every paired receipt preserves its actual session.
The final terrain audit and unchanged-pack checksum are recorded in the latest
checkpoint. These are scoped wall/window, backing and trim-scale acceptances;
whole-room actor/service and the broader interior acceptance remain open.
