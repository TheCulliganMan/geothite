# Duluth stair joins

Source candidate: `after.layout.json`, SHA256 `5e897305035386ab59478b71556a6cd7df231e3c2f2df6b0d6aebeab795c1ed1`.
This layout is now included in the selected `outdoor_terrain_integrated_revision`.
See `../outdoor-terrain-integration-v01/README.md` and its source index for the
current composed pack, preserved town repairs and final verification. The default
Duluth preview resolves this exact layout for the selected pack; the measured
source render takes0.574s. Neither rejected uniform-30 pack was selected.

The stage-by-stage notes below retain the historical publication and review
states. Their earlier pending-selection statements describe those preimages.

## Native shape contract

The user clarified that30 was right for a particular reference tile, not every
staircase, and explicitly specified **36 for right corners**. Select by the
actual adjacent cliff faces and tread alignment:

- Native36 includes the vertical right retaining face. Use it at the garden's
  right corner(28,20), continuing the east wall straight down to the street.
- Native12 has a left front rock lip and right-aligned steps. Use it at(18,8),
  (35,7) and(49,7), beside their existing west retaining corners.
- Native30 has left-aligned steps and a right front rock lip. Retain it on the
  six other straight-front flights, where a separate cliff block forms any
  adjoining corner. It is not a replacement for native36 at a vertical corner.
- Private90 is the north-facing garden flight. Its upper rows reuse the native12
  gray treads and its lower rows reuse outdoor70; it is not a south-facing30.

Keep outdoor terrain in the top two glyph rows, not pale cave floor. Reuse
existing cliff69 above right-corner36, and cliff72 above straight12/30. Import
only the lower native stair rows with their per-glyph gray treads and brown rock. The rejected attempt
forced every glyph to palette5 and changed the step colors. Never fix a shape
mismatch by recoloring or apply one reference tile globally.

## Geometry and verification

The candidate retains the authored continuous garden wall, closed mid-height
spur, connected lower street, and moved front flights. Dimensions stay64×46.
All room/event/story/regional anchors are retained by the indexed publication
contract. The garden’s former three-metatile hop ledge at(22..24,20) is replaced by
continuous high cliff72, matching the rock face to its right. The six former
hop lanes are blocked in both directions. The two other instances at(12,7) and(42,7) are closed as well: no hops off
high cliffs remain in this candidate.
Eleven stairs use four row recipes; native12 shifts its walkable
lower quadrant one tile right, so the old uniform-30 gameplay evidence cannot
be claimed for this source. Updated native courses follow those real quadrants.

`review_duluth_stair_heights.py` passed for the corrected source: no flat bypass
around any of the eleven flights and no contradictory stair/ledge elevations.
Production traversal, strict pack preservation, audio and native captures must
pass for the fresh pack before canonical selection.

## Review loop

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_duluth_cliffs.py \
  --pack content-packs/minnesota-current-duluth-cliff-stairs-v03.crystalpack \
  --layout planning/minnesota/revisions/duluth-garden-terrace-v02/after.layout.json \
  --output planning/minnesota/renders/duluth-garden-terrace-v02/shape-correction-v18
```

The whole-town native-source render and unmodified terrain originals are saved.
Inspection crops and the original three native cave tiles are retained under
`renders/duluth-garden-terrace-v02/shape-review-v17/`. They are source previews,
not production screenshots. All earlier rejected source/renders/packs and their
positive or failed verification records remain immutable history. The previous
uniform-30 dossier is archived as `rejected-uniform30-README.md` there.

Publish only with the Rust `revise_duluth_terrace_stairs` CLI, against the latest
selected parent and exact before/module preimages. Use a fresh output filename,
then run `audit_duluth_terrace_stairs.py`, all377audio checks, native2D captures,
and the three `duluth_city::cliff_stairs::` production-controller cases. The
continuous stdio journey `check_duluth_stairs_mcp.py` covers all eleven flights,
all14 closed high-cliff lanes, real saves and Continue. Preserve stderr and receipts.

Serialize Cargo under `.local/minnesota-cargo.slot.lock`. Do not supersede another
owner's selected content; resolve `world.json` again immediately before publishing
and selecting. Source rendering remains independent of Cargo for fast iteration.


Latest source correction: the garden front continues west from(22,20) to its
actual corner(18,20). Native72 spans(19..21,20), native6c closes the front-left
corner, and native68 continues up(18,19) and(18,18) to the existing west wall.
This replaces the abrupt end against pines with one connected retaining contour.
The previous17 native images apply to the earlier six-cell preimage only;
fresh captures and current production input remain required for this extension.


The v15 source supersedes the earlier connected outline. The garden is now one
raised plateau: native70 closes its north edge, native69 closes the entire right
side, and the detached rear shelf is removed. New private90 uses the unchanged
native12 lower gray step rows at its north side and native70 lower landing rows
at its south side. Collision is WALL/FLOOR/FLOOR/FLOOR, the native northern-ramp
orientation. Place it at(23,13); its outside street is core(47,25), step(47,26),
and inside landing(47,27). The inside plateau is higher than the northern street.
The gray steps keep their original palettes and pixels. Native UP_WALL on the
north rim blocks crossing the rim; it permits standing along the inside lip.
Do not treat its horizontal movement as a south-flight side exit.

The rear-left Observation/civic plateau continues to the west map edge. Its
north boundary uses6a at(0,1),70 from x1..28 at y1, then6b at(29,1);69 runs down the right
face to the existing front. Removing the old internal left wall and right notch
makes this one shelf. The front reaches the west edge at y6 and turns with6e
at(3,6). Existing doors, actors, rooms and connections are retained.
The current source artifact folder is `renders/outdoor-stair-landings-v01/plateau-v16/`.
Older capture and movement receipts are retained preimages, not proof of this
latest source. Fresh native publication, full preservation and input remain
required before selection.


Native v04 now exists, unselected, with the v16 west boundary cap:6a at(0,1),
68 at x0/y2..5, and6c at(0,6). The native compiler rejects an open walking
boundary; keep these visible cliff pieces rather than flattening the west edge.
`renders/outdoor-stair-landings-v01/duluth-publication-v38.json` binds the parent
and native pack hashes. `duluth-preservation-v38.json` passed exact preservation:
101 terrain cells change, all141 original metatile images remain exact, the other
729 maps/story/audio remain exact, all38 passive room returns stay reachable,
and the11 flight graph has no flat bypass or contradictory heights.

Four actual offscreen native captures of the garden north stair/right face and
rear-left west/right faces have zero differing pixels at640x576 each. Evidence:
`renders/outdoor-stair-landings-v01/native-plateaus-v40/capture-progress.json`.
These are local visual fixtures; they do not prove walking or earned access.
The broader regional candidate remains unselected while preservation and native
input checks continue. The earlier flat west edge was rejected by native boundary
validation; retain that failure and all earlier renders as preimages.

### Rear-left plateau tile ledger

All coordinates below are32px metatiles in the64×46 source layout. The full
review is `renders/outdoor-stair-landings-v01/plateau-v16/rear-left-plateau-large-2x.png`.

| Segment | Coordinates | Native block |
| --- | --- | --- |
| Northwest turn | (0,1) |6a |
| North rim | (1..28,1) |70 |
| Northeast turn | (29,1) |6b |
| Western face | (0,2..5) |68 |
| Southwest turn | (0,6) |6c |
| Western front | (1..2,6) |72 |
| Join to existing projecting shelf | (3,6) |6e |
| Eastern face | (29,2..5) |69 |
| Removed internal western wall | (1..2,2..5) |71 |
| Filled eastern notch | (27..28,2..3) |71 |

These rows form one contour. Keep existing lower frontage and its three stairs,
building entrances and actor positions. Do not introduce a hop ledge on this
high wall. `audit_duluth_terrace_stairs.py` checks the native pack against the
source, verifies the rim and side pieces, and preserves every unrelated map.

`rear-left-plateau.contract.json` records all54 placements, including the removed
internal wall, filled notch and concave front join. The native preservation audit
now checks every contracted cell rather than only the outer rim. The fresh
`renders/outdoor-stair-landings-v01/rear-left-contract-v84.json` receipt passes
on native terrace v04. Complete town and enlarged shelf originals are preserved
under `renders/duluth-rear-left-plateau-v01/5e8973050353-1793338794902431000/`.
The enlarged source view was inspected alongside both unmodified native west-
and east-face captures from `native-plateaus-v40/`. This review does not change
the active world selection.

The continuous production MCP replay now passes on outdoor-stair-repair v02:
917 calls traverse all11 flights in both directions, exercise21 step-side
denials and all14 high-cliff lanes in both directions, save and Continue, then
open the production Start menu. Both phases exit successfully with zero stderr
bytes. Evidence is `renders/outdoor-stair-landings-v01/duluth-stdio-v49/receipt.json`
and `duluth-mcp-proof-v49.json`. Its initial local placement, legal partners and
money are explicit setup for this exact pack identity, not earned access.
The earlier historical save failed identity validation before gameplay and is
retained under `duluth-stdio-v45/`; no identity check was bypassed. Regional
controller/animation checks and Clippy remain pending before selection.

The back-left plateau review was refreshed against native terrace v04 without
changing its source or the selected world revision. The current complete-town
render and enlarged shelf are retained in
`renders/duluth-rear-left-plateau-v01/current-review/5e8973050353-1793348460330104000/`.
The fresh `renders/duluth-rear-left-plateau-v01/review-current-native-audit.json`
passes all54 plateau placements, the eleven stair height/side contracts, all38
passive returns and exact preservation of the other729 maps. Both existing
unmodified native corner captures were inspected again. This is a verified local
review candidate; it does not promote the pack to active selection.
