# Accepted Minnesota outdoor polish — 2026-10-28

Selected local review pack: `content-packs/minnesota-region-polish-final-v03.crystalpack`.
SHA256: `67fb7ee40c022f0de78b561bbb144767d1483fe97a7dba9a2e6ec111d8c3360e`.
Content identity: `cfbd112da24c3c2644aba93368f387d0701cb13d59a32d955ab09af55e72315d`.

The final composition starts from the accepted R09 mine descendant of R10,
then applies the ancient mire, distinct R06/R07 shores, 179-cell North Shore
capes, 64×46 Duluth recomposition, river bank cells, R01/R02 grass, native
Vergloom art, accepted R05 Rock Cut, optional Vergloom events and the bounded
completed river activity stage. The old selected lineage had only two river
trainers per route; bank-only cells could not restore the missing landward
paths. The final river-only publisher carries the accepted twelve-module stage
and all forty bank refinements forward, without importing fruit or winter work.

## Final evidence

- 691maps;97outdoor maps in one direct-seam component; zero overlaps and zero
  inconsistent connection cycles. Full native atlas inspected at
  `renders/region-polish-v01/final-v03/full-state-native-current.png`.
- All377audio PCM hashes, frame counts and loop ranges match.
- Final river step preserves679other complete maps, all unrelated visible
  actors, native art/audio/runtime files, river attributes and aggregate outer
  boundaries exactly. All Johto/Kanto maps remain unchanged.31Minnesota maps
  differ from the accepted R09parent;660complete maps remain exact.
- Current roster:1026trainer/wild/static/gift species references inspected in
  touched maps, zero retired or foreign species.
- R06:three current controller journeys pass, including the two-turn cape,
  trainers, resources, Cut and Continue. R07 refreshed final-v02 journey passes
  eight trainer victories with selected PP, four ordinary wild battles,
 54required grass steps, resources and Continue; final river step leaves R07 exact.
- R01/R02/R08:seven controller cases pass on final-v02. Two reached-save stdio
  replays pass with10replies and empty stderr. Final river step preserves those
  complete maps and unrelated programs exactly.
- North Shore:three-cape native input/water-view/grass/Continue trip passes;
  compact north gate and four legacy warm cases pass. Duluth's final controller
  passes38paired door returns, stair both ways, ledge reverse denial and retained
 23-step return (47.32seconds). Later composition preserves their maps exactly.
- Final-v03 river suite:3/3pass in177.98seconds, all44added activities,40trainers,
  native wild encounters, Surf, selected PP/HP/XP/money, supplies, healing,
  repeated interaction and Continue. Initial party/badges/resources are explicit
  test fixtures; this is not an earned campaign claim.
- Final-v03 Vergloom suite:2/2pass in41.38seconds; three acknowledged clues,
  reveal, refusal, flee/reentry, knockout/loss retry, actual ball consumption,
  party storage, permanent capture and Save/Continue. Native battle scaling
  applies to its authored base level40.
- Fresh production client:Winona New Game/guardian/starter/nickname/balls/Start
  MCP passes on final-v03. Optional encounter MCP passes RUN/refusal/reoffer,
  Master Ball capture, party/Pokedex, actor removal, Start and a fresh-process
  Continue; all processes exit cleanly with byte-empty stderr. The capture
  animation requires actual elapsed production frames; individual one-frame A
  inputs are not evidence of a failed capture.
- Headless native2D revealed Vergloom image inspected:
  `renders/ancient-swamp-v01/story-native-final-v03/revealed.png`.

Root logs are `.local/region-polish-final-{audio,atlas,winona-mcp}-v03.log`,
`.local/region-polish-final-river-native-v03b.log` and
`.local/region-polish-final-mire-native-v03.log`. Identity-bound raster,
roster and preservation receipts remain beside the final atlas. Earlier
candidates and render originals remain historical; do not select packs by mtime.

## Executed commands

```sh
target/northstar-story/wasm-dev/pack_northstar_ancient_swamp \
  content-packs/minnesota-region-polish-rock-cut-v01.crystalpack \
  planning/minnesota \
  content-packs/minnesota-region-polish-vergloom-events-v02.crystalpack

target/river-fun/wasm-dev/pack_mississippi_activities \
  content-packs/minnesota-region-polish-vergloom-events-v02.crystalpack \
  planning/minnesota/connections/mississippi-bank-refinement-v01/revision.shared-activities-v02.json \
  content-packs/minnesota-region-polish-final-v03.crystalpack

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-region-polish-final-v03.crystalpack --audio --desktop

.local/minnesota-review-env/bin/python planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-region-polish-final-v03.crystalpack \
  --output planning/minnesota/renders/region-polish-v01/final-v03 --scale 12
```

Publisher outputs and review epochs must be fresh; preserve existing originals.
The production client verified for this encounter mode is
`target/northstar-story/wasm-dev/geothite`. Rebuild older clients before using
the optional encounter. Native input/MCP must keep using VisibleShellController.

Scoped Rust formatting and Assets/Bevy all-target Clippy pass with documented
baseline allowances. Workspace formatting and standard Clippy retain unrelated
failures (including core multiplayer bad_bit_mask and missing radio oracle test
fixtures). The legacy52formerICE coastal test also fails on the unchanged
baseline. No blanket green workspace claim, full earned campaign acceptance or
3D acceptance is made. Packs, saves and renders remain ignored and uncommitted.
