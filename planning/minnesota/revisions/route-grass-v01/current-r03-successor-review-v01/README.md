# Route 3 current grass, low banks and terrace directions

Reviewed publication: `route_grass_r03_stair_guidance_revision`.
Current selected successor: `cave_reservations_current_revision`
(`content-packs/minnesota-r20-winding-cleft-v02.crystalpack`, SHA256
`bfd4e1091213f558bca8ffc4c86afbc93af8569d16a3145dd9da157c4b1164be`).
The current successor retains both complete R03 maps, teams, encounters, used
art and every R03 source binding exactly; see `post-selection-proof-v01.json`.
Pack: `content-packs/minnesota-current-r03-stair-guidance-v01.crystalpack`.
SHA256: `78ccaed227c86c2ab162c97eeb5bf3d793467bb0550588641ed96cf5a0ec81c8`.

The complete native exterior and authentic Crystal Route30 source were inspected.
Route 3 retains the short-bank pattern: six woodland lanes drop onto a dry trail,
with a grassy ordinary uphill return. The taller eastern terrace has ordinary
native stairs and a solid cliff. Two existing messages still promised the removed
terrace drop; the sign and terrace trainer now direct players to the east stairs.

All 224 grass walking quadrants remain intact. The main habitat crossing requires
six grass steps in either direction. Actual ordinary returns for hop lanes
10–15 paid 8,8,8,7,6,5 grass steps respectively; the collision model independently
requires at least 3,3,4,4,4,4 with actor bodies conservatively clear, after correcting current-tile jump and lateral ledge walking semantics. Both models
exclude Surf, Cut and indoor shortcuts for these returns.

Only two text programs and their native text caches change in the selected child.
All 732 maps retain identical geometry, actors, events, teams and encounters;
731 complete other maps remain exact. All art, audio and runtime files are exact.
The parent is the current accepted R10 exploration revision, so its cave work,
Route 1's guidance, Brainerd's roof and Bemidji's stairs remain preserved.

## Native verification

- Current scoped native checkpoint: terrace sign, stair-mediated walking between
  upper/lower closed cliff edges, four repeated denials from each edge and
  Save/Continue. This uses the generic external-contract test with zero roads;
  it is not represented as a whole-route or earned-progression test.
- Complete production course: 2,022 stdio calls, both chunks and bridges, both
  neighboring town connections, ten actual wild encounters/RUN, all six actual
  hops, reverse denials and grass returns, ordinary terrace stairs both ways,
  stair side wall, closed tall cliff from both sides, actual reached-save
  Continue and Items/Balls pockets. Source/input saves stay unchanged; stderr0.
- Five native GPU views were actually inspected and have zero source/native
  pixel mismatches: original woodland drop, extended drop, dry landing, terrace
  stair and fishing-house facade. The whole current source was inspected too.
  The captures used the R01 guidance pack; current source/traversal used the
  newer R10 parent. `native-visual-successor-inheritance-v01.json` independently
  proves both complete R03 maps, used tilesets and all existing runtime files
  exact between those parents. Captures are not relabeled as fresh R10 runs.
- The child repeats the scoped native checkpoint. Its 214-call production text
  review reads the new sign, wins the real terrace trainer battle through the
  FIGHT/SURF menus, receives the native 240 money reward, reads the new defeated
  trainer text, and repeats trainer/sign after Continue without another reward.
  Source/input saves remain unchanged; stderr0. Initial partner, two badges and
  position are explicit native QA setup; no later state is patched or retagged.
- Publisher build, affected-crate all-target Clippy and owned Rust formatting pass.
  Unrelated existing warnings/workspace formatting differences remain logged.

The historical native full-course waypoint at (30,28) is now inside the added
fishing house, and the historical geometry case expects the retired tall-cliff
hop. Neither unmodified historical course is claimed as passing the current
successor. The current full walking course is verified through production MCP.
An initial proposed sign line exceeded 18 characters and was shortened before
native source generation; the failed preflight and wrapper attempt are retained.

## Current source

`sources-v02.json` preserves every parent current-source binding and adds the
current R03 review. `current-native-chunks-v01.json` contains the exact 744
installed metatiles: a24×24 upper chunk and21×8 lower chunk. It records the
trimmed southeast gap instead of inventing filler geography. This is explicitly
an installed chunk-layout bundle; original full-route builder inputs remain
historical base-phase sources.

`current-exterior.contract-v01.json` records six active low-bank lanes, current
ordinary stairs, the closed high cliff and both superseded original ledge
contracts. `guidance-before-v01.json` and `guidance-after-v01.json` are the exact
text program preimages/replacements used by the Rust publisher.
`guidance-child-preservation-v02.json` corrects the inherited route-receipt
filename reference and records the explicit native-image inheritance proof.
The original receipt/index/acceptance versions remain retained.

Reproduce to a fresh output:

```sh
.local/route-grass-review/portable-r03-guidance-tools-v01/revise_r03_stair_guidance \
  content-packs/minnesota-r10-exploration-v03.crystalpack \
  acc5e63d532de34614cbcc0ac4bd414b190d8cdc9e5d319fdf71f3de3302e073 \
  planning/minnesota/revisions/route-grass-v01/current-r03-successor-review-v01/guidance-before-v01.json \
  planning/minnesota/revisions/route-grass-v01/current-r03-successor-review-v01/guidance-after-v01.json \
  content-packs/minnesota-current-r03-stair-guidance-reproduction.crystalpack
```

This closes the current Route 3 walking/grass/ledge/stair review. Other numbered
routes and regional links remain within the broader active objective. This is
not whole-campaign or earned regional-access completion. No commit, push or
external deployment was requested. Packs, captures, saves, build products and
generated verification material remain ignored.

Native movement-model correction: `current-grass-static-v02.json` and
`current-exterior.contract-v02.json` supersede the old static lane minima.
`main-crossing-native-model-v03.json` independently rechecks the six-step main
grass crossing in both directions. Existing production walks remain unchanged.
