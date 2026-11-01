# Winter renderer review plan

The integrated callsites were independently read after Root applied them.
`bitmap_font.rs` uses the winter source alias for common animation registration,
recognizes both North Shore IDs for waterfall20, separates winter rustle cache
keys, and loads actual icy glyph foreground metadata through the terrain palette
resolver. `menu_rendering.rs` selects the effect from the **actual block** and
uses that same style for both cache retrieval and error reporting. Thus original
green grass inside the winter alias retains its native cache. OAM layout,
zero/background handling and `(rustle.age / 4) % 2` cadence are unchanged.
Border Pines retains its current static animation behavior; this review does not
silently add forest animations to its derived alias.

`native-review-plan-v01.json` defines two baseline and nine winter views.
The exact3d1c baseline views have been bound successfully under
`renders/ice/native-render-plan-v01/baseline-bound/`. All nine winter camera cells
also pass actual published-pack alias, block, collision and occupancy checks on
`content-packs/minnesota-northern-winter-v01.crystalpack`, SHA256
`aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a`.
The exact bound views and receipt are under
`renders/ice/native-render-plan-v01/winter-bound-aa1aa251/`. The earlier
`source-camera-preflight.json` remains as source evidence. These are geometry
checks, **not GPU or native gameplay acceptance**. Run captures using the bound
`views.json` and this exact published pack:

- `bind_native_review_plan.py --pack PACK --mode winter --output NEW_BINDING`
  rejects missing maps, wrong aliases, wrong blocks/collision, occupied cells and
  off-map coordinates. Its output `views.json` is accepted by the reviewer.
- `review_winter_native.py --pack PACK --binary BINARY --views VIEWS --output NEW_REVIEW`
  runs the existing headless production capture path and connected-camera clamp.
  It modifies only its privately imported reviewer object in memory; no shared
  reviewer file or production image is changed. Every complete640×576 frame must
  match0pixels of difference. All GPU originals, raw readbacks, phase references,
  differences and failures remain saved.

The extension handles North Shore glyph20 as eight native downward scroll
stages, three pixels per stage, with cadence19/offset16/advance-on-offset. Water14
uses four native frames at cadence22. Visible flower03 uses both native frames
at cadence60. Every occurrence of a family shares one phase throughout the full
frame. A single global clock witness is reported for the matched phase tuple;
that witness is **not a timestamp observed from the production capture**.
Cadence and phase-offset correctness also require the actual renderer runtime
regressions. No image masks, fitted offsets or per-glyph phase selection are used.
The source checker verified that all64 flower/water/falls combinations have a
native-clock witness. The CLI help and Python compilation pass; new GPU captures
have not been executed by this worker.

Exact grass-rustle frame evidence should come from the actual production frame
loader regression listed in the JSON plan. Its two18×9 RGBA frames must retain
native opacity/OAM placement while using the actual icy glyph colors. The
test compares the actual in-memory images. Set
`CRYSTAL_WINTER_RUSTLE_CAPTURE_DIR` to a fresh nonexistent directory to retain
all 24 actual 18×9 RGBA frames: native baseline and both winter aliases, two
frames each across all four time banks. The optional export also preserves the
native source PNG, pure nearest-neighbor 16× previews, and a completed receipt
with pack, source, test, PNG and raw RGBA hashes. Existing output is rejected;
PNG round-trip equality and all original palette/opacity/cache assertions stay
required. These are production frame assets, not GPU/controller captures. The
general location tester's walk-capture
tick is not rustle age, and its movement CSV does not record rustle age. Do not
claim a tick12 screenshot proves rustle frame0 or1. If a later production capture
holds the observed effect passively, record its real age0–3/4–7 and keep the
ordinary step history; do not manufacture effect state.

The complete source board is
`renders/ice/whole-area-review-v01/board-display.png`, with full-resolution board,
all three original wholes and provenance alongside. It is explicitly a separate
panel review, not connected geography. Rainy's v03 differs from v02 only in its
native music ID; geometry and all other dossier fields were verified equal.

The subsequent actual-written-pack board is
`renders/ice/whole-area-native-v01/board-display.png`. Its full-resolution
original uses one native art pixel per board pixel and preserves separate Rainy,
R17 and R11 panels, including the installed R11 north cap. Every scene pixel and
every underlying terrain pixel matches its reviewed source whole exactly; the
three comparisons and actual module hashes are in its `proof.json`. The real
Rainy–R20 seam is a secondary crop from the actual native atlas originals.
This board is actual pack terrain with authored actors, not a whole-map GPU
capture. Its controller and GPU status remains pending until separate evidence
is available. The earlier source board remains unchanged.

By sight, R17 now reads as continuous snowy cedar woods and frozen marsh rather
than tiny overlays; the remaining lime Cut-tree square is an intentional native
gate that should not be recolored without replacement semantics. R11's cold upper
gorge remains visually subordinate to the large open lake. Its brown cliffs and
warm buildings remain useful contrasting landmarks. The stepped snowlines read
clearly in both wholes. No new mixed-edge defect requires another art family.

Rainy's open bend supports a useful17-tile lower glide,6-tile island-wall link
and3-tile cache approach, contrasted with the139-step ordinary snow return in
the frozen source courses. These give the broad ice a traversal purpose. They
do not establish Ice Path puzzle density, and the central expanse still has few
stopping decisions. The central stopping-tongue and weakest south-bank views
are explicitly included for native review. Do not add generic props or new
activity quotas to fill it; judge the actual slide/stop/return loop first.
