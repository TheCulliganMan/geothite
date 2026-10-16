# Department store architectural courses and closed display islands

Current correction, 2026-10-14: the user identified the fifth-floor staffed
enclosure as counters on its north, west and east sides, with a glass wall
toward the screen (+Z). The previous sealed lower display/lid interpretation
below is superseded. The current editable kit has three counter runs and four
thin vertical glazing panels; the renderer separates those panels and their
reflections into a transparent material. Source identity, all56 owned cells,
the original staff-floor inset, collision, actor support and shop scripts remain
unchanged. Source-complete blue aisle boxes are separate long, low cases with
flat sides and ridges only above their lids. These revisions require fresh
production visual review; geometry/source checks alone do not accept them.
Current reviewed result: both stores'2F/3F blue cases and5F enclosure were
inspected against2D, including additional5F south-side views. The longer flat
blue cases and three-counter/front-glass interpretation are verified within
that scope. See `docs/handoff/kanto-interiors.md` for receipts and remaining
walking, counter-reveal, scale and room-composition work.
Counter reveal follow-up: authenticated adjacent bays now meet beneath their
caps and omit buried end faces. Both solid and source-painted bays use depth
coverage. Interior floor compaction remaps their reveal ranges and linked
label/register anchors together with the vertices. Both4F approach/retreat walks
verify player visibility and opaque restoration without layered gray triangles;
the pixel coverage remains visible during fade. These checks do not complete
room-scale, palette, interaction or all-stair acceptance.

This original kit refines the two stores' six shared floor layouts. It does not
change runtime maps, collision, events, warps, actor support, inventory or shop
logic. The eight cached prototypes use the existing bounded gzip-in-JSON model
storage and are appended into the existing terrain batch.

## Source ownership and honest coverage

The external `content-packs/core-modular.browser.crystalpack` was inspected in
memory. Source images remain temporary review material, outside shipped source.
The adapters embed only source identity hashes, sparse masks and placement
metadata, not a second content catalog.

| Floor, in both stores | New kit wall ownership per map |
| --- | ---: |
| 1F | 56 |
| 2F | 28 |
| 3F | 52 |
| 4F | 36 |
| 5F | 52 |
| 6F | 36 |

The wall kit owns 520 source cells across 12 maps. Older `WindowWall` signatures
recognized patterns for 416 of these cells, but required a missing floor sample;
the prior production audit recorded no store-map window ownership. The joined
kit uses the verified native floor and now consumes all 520 cells. The other 104 add
lift-side fluting/call-panel housings, live directory housings and complete
counter/wall junctions. Resolving this kit before filtering old interior
placements is essential to avoid duplicate windows or a reservation conflict.

The earlier 704-cell wall family estimate also contains 96 elevator
door/threshold cells and 88 stair cells. The stair follow-up now models all 22
complete flights across the two stores, using the existing editable interior
stair asset. These are separate stair claims, not additional wall geometry.
All twelve lift fronts now also claim their 96 door/threshold cells, using the
existing open sliding-door frame and an original Rust metal tread. The full
4×4 block guard preserves the adjacent call-panel housing and ordinary floor
row. The modeled aperture remains clear; actor support and lift warps are unchanged.
Native Goldenrod/Celadon 1F views verify the open fronts. A real northward
step from Goldenrod 1F `(2, 1)` enters `GoldenrodDeptStoreElevator` through the
production controller; lift-floor selection remains a separate route check.
Elevator cabins and the roof have their existing special-room kit; B1F remains
outside this architectural kit.

Each flight binds a complete 2×2 drawing at its native source origin, with exact
source identity and custom-profile rejection. The eight treads, stringers and
handrails rotate north without changing actor support. Descending flights have
their backing below the floor datum, leaving the treads exposed. Fitting before
rotation preserves bounds and normals even with unequal tile axes. Celadon's
6F shared roof-flight drawing has no roof warp; modeling it adds no connection.

Each 5F assembly is one 8×8-source-cell object at source anchor `(12, 8)`.
Its 56 owned cells comprise the entire closed lower 8×4 display (32 cells),
plus 24 connected counter cells above. Across both maps, the kit owns 112 cells:
all 64 lower-display residuals plus 48 existing counter-drawing cells.
The upper 4×2-cell inset is genuine FLOOR occupied by staff; it remains unclaimed
and free of authored geometry. The entire lower half is WALL and remains a
closed volume, including the dark central display lid. It must never acquire
an invented walk-through U opening. No geometry extends onto the approach carpet.

## Art and placement

Joined cream paper frames, a continuous cornice and ochre toe course surround
recessed blue glazing with diagonal highlights. Separate lift-side ribs and copper directory rims retain
their source-sized silhouettes. The directory's four source tiles and the
lift call-button tile are sampled from the current atlas onto outward faces;
lettering is not baked into the original authored assets.

The island has a connected sales rail, two returns, a sealed lower display lid,
four shallow drawer/glazing fronts, copper pulls and a continuous dark toe.
Its maximum height is the existing eight source pixels. Wall courses remain
16 pixels high and three pixels deep, entirely within the native blocked north
row. Models use exact source widths rather than stretching one pane across a
long run. The staff floor inset stays at its original support height.

Marked housings and their live plaques share one authored camera-fade group,
including when a player ray hits the housing but misses its small inscription.
Both material domains retain their original geometry, texture UVs and colors. Underlays sample the same-atlas native `mart` block `$04`, tile `$01`.
Custom live profiles take priority. A changed tileset, block, subtile phase,
tile identity, map, source origin or incomplete guard rejects the entire
corresponding network. The unchanged old renderer is then responsible for it.

## Rebuild and checks

```
python3 tools/build-department-store.py target/department-store --runtime-only
blender -b --threads 2 --python tools/build-department-store.py -- target/department-store
python3 tools/check-department-store.py --pack content-packs/core-modular.browser.crystalpack
cargo test -p crystal-voxel-view department_store
```

Runtime assets are deterministic products of the named editable parts in
`tools/build-department-store.py`. The compressed editable Blender source belongs
in the existing source manifest as `department-store.blend`.
The Python check verifies closed oriented parts with positive volume, valid
normals, unchanged source identities and collisions across all twelve maps,
exact warp exclusion, the staff-floor inset, three counter runs, thin front glass and the
editable-source hashes. Rust tests cover rejection at every source field,
native padded/scrolled frames, cropped guards, custom profiles, atomic append,
non-square scale/normal fitting, live inscription UVs and unchanged footing.

## Native review still required

Studio front/rear reviews verify the original model shape and materials only.
Before art signoff, run the Rust tests and production coverage audit, build the
native client, and inspect the integrated geometry in its actual lighting:

- Capture both stores' 1F and 5F; include 2F/4F shelves and 6F vending/stairs to
  check the shorter wall runs. Confirm no doubled panes, underlay patches or
  seams through fixture silhouettes
- On 1F, approach the receptionist from the native counter side and advance her
  text. On 5F, stand at gameplay `(8, 3)`, face south and interact across the
  native counter to the clerk at `(8, 5)`. Verify shop opening and closing,
  and that the staff remains grounded in the two-square floor inset
- Read each directory from `(14, 1)` facing north; read the lift call panel from
  `(3, 1)` facing north. Check the source lettering remains readable under both
  camera directions and cutaway
- Enter the lift through gameplay `(2, 0)`, travel and return. Use every existing
  stair warp in the inspected floors, including Goldenrod's 6F roof connection
- Orbit both sides and approach the enclosure. Confirm that its front is
  transparent glass while the other three sides remain counters, and that the
  existing staff-floor inset stays clear. Original collision and footing remain
  authoritative; a visual capture does not establish a new walkable floor

The stair source checker reads all twelve maps directly from the ignored pack
and verifies 88 STAIRCASE cells across 22 complete drawings. The production
audit adds exactly 88 stair claims. Rust checks include atomic append, negative
well depth, fitting, normals and unchanged support. A native 3F view confirms
the upward flight and recessed down flight; a real Goldenrod 1F-to-2F stair
transition passes. Further camera positions and routes remain to be checked.

Integrated voxel tests and the all-map audit pass. Ten native views cover both
stores on floors 1, 2, 4, 5 and 6; the ordinary 5F-to-4F stair route also passes.
Shop/lift interactions and further camera/movement routes above remain separate
checks.

### Source palette and service returns

The store floor uses a warm ceramic grid with visible grout and small tile
variations at a consistent 16px world phase. Window glass is blue, seats have
pink upholstery and raised frames, and cream counters have
colored front panels. These map-scoped finishes preserve the original playful
palette rather than applying a neutral material to every interior. Other Mart
and Pokécenter floors and non-store cushions retain their own finishes.

Eight complete 2×2 service-counter ends on both stores' 1F/3F replace their native
raised artwork: four plain returns and four mechanical cash registers. The
register has a cash drawer, sloped keys and receipt hood facing the clerk. Exact
source hashes, reservation checks and full drawing bounds preserve the clerk's
standing space, shop interactions, support, collision and adjacent wall courses.

Native views inspect Goldenrod 1F, Celadon 3F and both Game Corners. Store
seating now uses its source profile’s 12px top and 12px depth, with a supporting
frame and legs. The revised Goldenrod 1F native view verifies continuous tile spacing and
raised seating with visible supporting legs. All fifteen store regressions, four enhanced floor
regressions and slot orientation/seat regressions pass at this checkpoint.
The final third-floor plain-return binding adds eight cells across both stores;
its packed source/collision check passes and the production audit records
172 authored cells on both 3F layouts.

The destination-floor regression verifies complete floor finishing despite
redirected fixture samples, custom reservations, unchanged support and zero
object-coverage credit. It passes alongside the fifteen store regressions.

A final Celadon 3F native view also verifies its modeled left counter return
and the continuous warm ceramic floor. The source audit finds all eight
service ends across both stores’ 1F/3F, with no mesher errors.


### Second- and fourth-floor service runs

Both stores' 2F U-shaped service counters and 4F L-shaped returns use the same
cream countertop and coral cabinet kit as the other store counters. Repeated
2×2 sections preserve furniture proportions along each run; the customer
panels face outward on the north, west and south edges. Exact whole-drawing
source guards cover the full 8×8 or 8×4 layout, while sparse masks claim only
COUNTER cells: 80 on 2F and 40 on 4F across both stores. Clerk floors and the
2F right-side shelves remain outside the geometry. Reservation, cropped-view
and support checks apply to each run atomically.

Service counters remain opaque rather than joining the wall cutaway ranges.
This prevents standing inside the 2F staff bay from fading the entire U-shaped
fixture. The existing atomic append regression also checks this behavior.

The final production audit consumes all 120 new service cells across the four
2F/4F layouts: each 2F has 172 authored cells and each 4F has 200. Remaining
native samples in these base layouts classify as flat, with no mesher errors.
Native Goldenrod 2F and Celadon 4F views inspect joins, cabinet colors and staff
clearance with the counters opaque. All sixteen store regressions pass,
including the destination-floor test. This establishes static presentation and
ownership; it does not certify opening shops or every dynamic room state.


### Basement wall conversion — native verification pending

The 5F audit's eight remaining relief samples per store are the genuine staff
carpet inset, not omitted counter geometry; the native 5F comparison verifies
that inset stays visible and walkable.

Goldenrod's basement baseline consumes 144 crate cells and retains native wall
courses. Its new wall selector reuses the connected passage masonry kit with
modeled blue wainscot. Nineteen complete underground block fingerprints cover
all 492 native wall-course cells, including six basement-specific block types
missed by the initial passage vocabulary. The pack-side checker fails if any
wall block is absent or altered, rather than counting fallback rendering as
model consumption. Floors, crates and the lift drawing are excluded. The exact
production matcher also passes a standalone Rust check for its native block,
altered floor guard. The subsequent crop follow-up accepts all 1,900 native
rectangular crops across the nineteen variants while rejecting altered visible
drawings; valid crops no longer trigger native fallback.

The full renderer regression and native build encountered shared migration
compile failures: render API imports mismatched the resolved Bevy version and
the old patched bevy_winit source was removed during the migration. The new basement geometry is not yet certified by native
views or the production coverage audit. Require 492 consumed wall cells, inspect
joins and blue trim, and review the lift separately when that build completes.


### Basement lift and warehouse stair — native verification pending

The basement lift opening is a separate complete 4×2 source drawing across
underground blocks 25/26, with original lift warps at runtime tiles (9,4) and
(10,4). It now uses the shared open door-frame model above its live pink
threshold artwork. The gray 2×2 drawing on block 03 at source tile (34,4) is the
warehouse stair, whose original warp at runtime tile (17,2) leads to the
underground warehouse. It now uses a recessed descending stair flight with its
backing at -16px, so a floor plane cannot hide the treads. Both fixtures retain
unchanged support and original events. They add eight and four model cells,
respectively; together with the walls and existing crates, require 648 consumed
cells in the basement production audit. The pack checker verifies both source
drawings against their separate warp destinations. Full Rust regressions,
native geometry consumption and room presentation remain pending a fresh
Bevy 0.20 build. No fallback result certifies these fixtures.

The crop checker now lives in `tools/check-department-store.py`. It compiles the
actual Rust matcher against native identities read from the ignored pack and
removes its temporary fixtures afterward. Missing cells outside the viewport
use the known quadrant identities only to reconstruct the full fingerprint;
all visible cells contribute their actual identities. Geometry ownership stays
limited to visible wall cells. This verifies source acceptance, not production
mesh consumption. The renderer regressions currently still fail in migration
APIs before these tests can run. The coordinated disk cleanup removed old build
caches; verification now uses the shared `target-bevy20-verify` cache. Native
verification remains pending that build.

The current window-course finish uses explicit blue glass, sky reflections,
glints and blue shadow materials. Closed diagonal facets replace the former
rectangular highlight. The Rust store pass no longer infers glass from RGB
channels. Frame sizes and non-window geometry/paint are retained. The updated
canonical JSON matches the generator and passes the complete pack/geometry
checker (178 closed parts, 4,184 triangles). All nineteen department-store Rust regressions pass, including production
paint preservation for all five window-course sizes. All twelve muted native
comparisons in `target/store-blue-glass-review/` were inspected; their 24 visible
readbacks passed with no shader errors. The production audit retains all base
object totals and 520 wall cells, finite footing and zero mesher errors. The
window finish is verified; fifth-floor display transparency still needs work. The archived editable Blender scene still contains
the previous finish and needs a resave from the current generator; no local
Blender executable has been found in the usual installation/build locations.

The fifth-floor display now has a dedicated named countertop enamel finish,
copper edging and blue glazing with closed diagonal reflection facets. Its
closed bodies, source-floor inset, 64x64x8 bounds and source matching remain
unchanged. The generator and canonical JSON pass the complete checker (192
closed parts, 4,416 triangles across eight models). The runtime finish is
limited to this model's exact countertop material. All twenty-one department-store Rust regressions pass. Native review of
this refinement remains pending; the editable Blender archive is still the old
finish and must not be treated as the current generated scene.

Fresh native review of both 5F displays and two controls is complete in
`target/store-display-glass-review/`: all four comparisons were inspected and
eight visible readbacks passed without shader errors. The new blue glazing,
diagonal reflection relief, copper beads and opaque warmer enamel are visible.
Both 5F islands retain 56 source-owned cells and all reviewed store floors
retain 140 modeled object cells, with finite footing and no mesher errors.
The finish increment is verified; the broad display still has limited
merchandise detail, and the archived Blender scene still needs its resave.
