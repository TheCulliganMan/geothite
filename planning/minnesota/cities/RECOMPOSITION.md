# Tighten the installed cities

The current user direction is **“Smaller but tighter.”** The published64×48
exteriors remain an implementation baseline, not a density target or accepted
visual result. Minneapolis is testing a40×32 hand composition after a48×36
capacity study; St Paul is being assessed separately. The dimensions are subordinate to useful streets,
distinct districts and short trips. Do not scale the rendered image or crop
away rooms to simulate a smaller playable city.

Root inspected actual current Minneapolis player views against native Goldenrod
and Saffron. Six unoccupied lawn squares are8–10 walking tiles across; one
complete player view is green. Pure lawn occupies1,155 of3,072 metatiles.
These are specific defects to remove. Forty current facade footprints occupy
328 metatiles and fit a preliminary48×36 nonoverlapping packing without
changing native building art. Packing alone is not a finished map.

## Source candidate

Preserve every existing meaningful entry, all44 Minneapolis room modules,
original facade pixels, native inventories/prices, scripts, trainers/parties,
quest flags, signs and services. Hand-place buildings into inhabited market,
service, civic, residential and lake districts. Most ordinary streets need
only2–4 walking tiles; wider spaces need a visible activity or landmark.
Do not replace the lawns with repeated trees, props or a repeated district grid.

Record an explicit before/after address for every outdoor object, background
event, coordinate event, active door, passive return and city recovery spawn.
Keep native vector indices and object identities. Every original exterior warp
must still pair with its unchanged room's exact return index. Show the whole
town plus its weakest streets, actual player cameras, meaningful destination
trips and both regional walking lanes before native acceptance.

Candidate sources and every render live under
`renders/route-quality-next/minneapolis-assessment/` and
`renders/route-quality-next/stpaul-assessment/` until Root reviews them.
Minneapolis town/world/interior bindings now use the accepted native40×32
recomposition. The original64×48 creation recipe remains history. St Paul and
Duluth retain their earlier native baseline while their source studies continue.
The existing cached source renderer supports fast immutable revisions; native
publication is a separate step.

## Native implementation contract

The existing `expand_authored_city` deliberately accepts only32×24→64×48 and
adds new content. Do not replay it on the installed660-map pack, loosen its
old creation guards or recreate occupied room identities. Add a focused
installed-city recomposition path for a candidate that is worth publishing.
`recompose_authored_city` and its CLI are now implemented and under validation;
the first actual40×32 native attempt correctly refused a reachable pond at
the unconnected south perimeter, core(20,63). The corrected Falls branch is
now published separately; see [exact inputs and native verification](../playable/minneapolis-falls.md).
The following remains the acceptance contract:

1. Certify the exact installed city module hash, dimensions, compiled tileset
   and complete room/anchor inventory. Use the current parent pack, not a stale
   earlier story/city pack. Refuse unknown fields and stale before coordinates.
2. Clone the pack and city. Compile only explicit candidate terrain. Relocate
   the original indexed objects, background/coordinate events and warps through
   exact overrides. Preserve every noncoordinate field, program and ordering.
   Reject coordinate-bearing instructions that lack an explicit supported
   transformation; never leave old script coordinates hidden behind moved NPCs.
3. Revise only contracted city dimensions, block payload, actual anchor rows,
   reciprocal connection offsets and outdoor recovery coordinates. Refresh the
   existing event section and split native indexes through the normal publisher.
   Keep every room, catalogue, trainer party, unrelated map, audio, report and
   HOME record exact. No new room or flag is needed for compaction. The explicit
   Minneapolis Falls exception appends one native cave waterfall glyph in
   unused city art storage,54 exact neighbor river metatiles, and the fixed
   badge-controlled north buoy gate. Its six additive programs,12 new coordinate
   events and silent tile callback preserve every original program and event.
4. Reuse native directional movement/collision validation: reachable entries
   and passive returns, actor interaction neighbors, safe spawn, both exact
   seam lanes in both directions, no unintended walking/Surf edge escape, and
   actual ledge reverse/side/ordinary-return behavior where applicable.
5. Independently normalize only those declared fields and compare the entire
   resulting dataset and raw pack identity. Corruptions must fail for a stale
   anchor, blocked return, wrong reciprocal lane, altered room/program/party,
   opened water boundary and unrelated content drift.

All production input remains on `VisibleShellController::press`. Walk all
entries/returns and outdoor seams, rerun real priced transactions, activities,
battles/selected PP, resource consumption, checkpoints and Continue. Initial
local fixtures are not earned campaign progression. Do not rewrite position,
flags, inventory or RNG after setup to rescue a route. Then compare actual
headless native frames at the candidate's true camera clamps and review the
whole map by sight again. Retain all previous city renders and candidate packs.

## Separately owned fair seam

The fair author's later dense/grass candidate has664 maps and is based on the earlier
R04-only660-map stage, not the1186a243 four-route successor. It uses group80
and changes eight St Paul north street cells, meta x8,y0..7, to block55, with
an offset0 reciprocal walking link at core x16/17. Its guarded publisher
refuses a stale city preimage. Preserve this contract, but do not transplant
its old coordinates into a recomposed city or claim it is already integrated.
Rebase the actual fair arrival and both walking lanes onto the chosen St Paul
geometry and rerun its controller/preservation checks as a separate integration.
