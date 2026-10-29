# Duluth hill-to-harbor tightening

The reviewed candidate contracts the native town from64×48 to64×46 metatiles,
removing two unused hillside shoulder rows. It retains all38 exterior doors,
their76 indexed active/passive warps, interiors, native programs, services,
trainers, rewards, checkpoint conditions and Minnesota identities.

The harbor workshop's existing facade and its two exterior warp anchors move
from door(80,59)/return(80,60) to door(72,41)/return(72,42). Its interior and
receiver program remain identical. The Mart and workshop now share a short
service street below the clinic. The old workshop site becomes an attached
pine margin beside the retained harbor promenade. A sixth native outdoor stair
at metatile(27,20) connects the gym shelf toward that promenade. The south ledge
keeps its original native blocks, reverse denial and a23-step ordinary return.

Ordinary FLOOR-only native walking distances, with actors blocking their real
tiles and no paths through doors:

| Trip | Before | After |
| --- | ---: | ---: |
| Gym return → workshop return |66|34|
| Clinic return → workshop return |55|21|
| Mart return → workshop return |58|10|

The western R10 seam moves from town lanes(0,48)/(0,49) to(0,44)/(0,45).
Its offset changes12→10 and the reciprocal R10 offset changes−12→−10 in the
same Rust transaction. Northern lanes(64,0)/(65,0) and offset9 remain unchanged.
The shared world metadata and selected pack belong to the integration owner.

## Source and publication

The immutable before/after layout, dossier and indexed native contract are in
`planning/minnesota/revisions/duluth-hill-harbor-v01/`. Publication uses the
existing Rust city recomposition implementation; no new exporter or art is used.

```sh
target/fair-maze/wasm-dev/recompose_authored_city \
  content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  planning/minnesota/revisions/duluth-hill-harbor-v01/MnDuluth.after.layout.json \
  planning/minnesota/revisions/duluth-hill-harbor-v01/MnDuluth.contract.json \
  content-packs/minnesota-duluth-hill-harbor-v04.crystalpack

.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_duluth_hill_harbor.py \
  --before content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  --after content-packs/minnesota-duluth-hill-harbor-v04.crystalpack \
  --output planning/minnesota/renders/duluth-tightening-20261028-v01/native-preservation-and-trips-v04.json
```

Both commands passed. The same publisher can compose this bounded change onto
another parent whose Duluth native module and reciprocal seam preimage match.
Do not replace a later integrated pack wholesale with this isolated candidate.

The exact native audit confirms every unrelated map is identical, except the
single R10 reciprocal connection. City executable programs, all runtime files,
the entire audio manifest and report are identical. Changed city split payloads
contain only the contracted geometry, coordinates and dimensions.

`verify_crystalpack PACK --audio --desktop` also passed:691maps,475species,
270moves and377audio programs; all PCM hashes, frame counts and loop ranges
match. Its original output is retained as `audio-verification-v04.log` alongside
the renders. The ignored v04 pack's file SHA256 is
`54eb23aa8d967907229916d178c7fefa91bb2b35382d63bdb5cc7aaf8ff68a6a`.

## Visual review

All original renders remain in
`planning/minnesota/renders/duluth-tightening-20261028-v01/`:

- `before.png` and `after-v04.png`: complete town at its actual proportions.
- `native-weak-street-before-v02.png` / `native-weak-street-after-v02.png`:
  native2D at(35,21), noon, same facing. Both stand on reachable FLOOR.
- `native-service-midpoint-v04.png`: native2D at(76,41), showing both the
  workshop and Mart with their actual entrances in one player camera.
- `native-harbor-promenade-before.png` / `native-harbor-promenade-after.png`:
  the same harbor landmark at(82,61) before / (82,57) after contraction.

The first weak-street after capture placed the fixture on the newly moved tree
row. It is retained as a rejected fixture and is not evidence of a reached player
position. The corrected immutable v02 pair replaces it for review. The first
stair proposal beside the ledge was also rejected because its return was too
short; v04 moves the stairs eight walking tiles east of the hop.

## Controller verification status

The focused ignored external-pack regression is
`duluth_hill_harbor_all_38_paired_returns_and_native_stair` in `duluth_city.rs`.
It enters every actual exterior door, walks both native interior exits to the
correct passive return, verifies no bounce, walks the new stairs both ways,
and tests the old ledge hop, blocked reverse and ordinary uphill return. Its
party setup uses active Minnesota Pouchid/Cindore. Explicit fixtures do not
prove earned regional progression.

The final fresh production-controller run passed on the combined
`region-polish-rock-cut-v01` parent in47.32seconds: all38actual doors and both
interior returns, passive arrivals, bidirectional staircase, native ledge hop,
blocked reverse and the retained23-step ordinary return. Original log:
`.local/region-polish-duluth-controller-v03.log`. The exact test is
`bevy_shell::tests::duluth_city::duluth_hill_harbor_all_38_paired_returns_and_native_stair`;
the serialized runner uses `CRYSTAL_DULUTH_CITY_CONTROLLER_PACK` for its pack.

Canonical `towns/MnDuluth.layout.json`, `towns/MnDuluth.json` and the new
`towns/MnDuluth.recomposition.contract.json` are promoted. The immutable prior
sources remain in the revision directory. Shared world metadata and final
selected-pack integration belong to root and are not changed by this promotion.
Direct formatting of the owned Rust file passes. Workspace-wide
`cargo fmt --all -- --check` reports extensive pre-existing/shared-file diffs;
those unrelated files were not reformatted.

The first fresh controller run stopped during fixture setup because the test
used authoring IDs `MN_001`/`MN_004` rather than runtime species keys. The fixture
and PC assertions now use `POUCHID`/`CINDORE`. No game pack was changed to fit
the test. The final fresh rerun recorded above passed.

The second fresh run completed every door/return and the staircase in both
directions, then exhausted the helper's eight-input budget on an eight-tile
rightward walk because its first input changed facing. The two eight-tile
return legs now explicitly turn through native input before walking. Their
exact target assertions, blocked-reverse checks and23-step route remain intact;
no terrain was altered in response to this fixture timing issue.
