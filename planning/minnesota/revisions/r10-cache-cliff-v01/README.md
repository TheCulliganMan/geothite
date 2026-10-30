# R10 cache and mine cliff review

**Rejected source candidate. Do not publish or select the current contour.**
The selected world remains `duluth_cliff_stairs_revision`. No R10 native pack
was produced by this review. All original, rejected and revised renders remain
under `renders/r10-western-cliff-v01/`.

The current native parent is the unselected outdoor-north-stairs v01 pack,
SHA256 `694e0700d4777b4bc1c06065e92a3ca58f8140ef94a33412120ddec05bf27ca0`.
`before.layout.json` matches both complete native R10 chunks, including the
already published cliff-hop closures and boulder setbacks. `after.layout.json`
contains the16 explicit placements in `plan.json`; dimensions, native art,
stairs, actors, cave mouth and regional seams are retained in the source.

## What was learned

The first nine-cell proposal put the right wall at x17 and stranded Iva's
approach. Its source check failed; the original plan remains in
`renders/r10-western-cliff-v01/rejected-plan-v85.json`.

The next contour extends the northern rim to the actual eastern wall at x20.
Its ordinary road, six trainers, three residents, three pickups, six signs,
cave approaches and required pine-neck habitat pass the source review.
Whole-route originals and enlarged views were inspected under `source-v86/`.
This is insufficient for acceptance: the new northern forecourt lets a player
approach the cache behind its declared encounter threshold.

`grass-contract-v89` and the widened-band v91 attempt both failed: the minimum
declared grass cost falls from4 to0. The independent graph includes native
UP_WALL lateral walking, blocks down-entry/up-departure, includes actor bodies,
and excludes warps and hops as ordinary steps. The pine-neck crossing remains
six grass steps. Widening the reward band's rectangle did not fix the bypass.
Retained evidence: `directional-grass-paths-v90.log` and `rejection-v91.json`.
Do not lower the minimum or treat the larger grass count as proof of gating.

## Publication contract repair

The actual attempted command was:

```sh
target/runtime-fps/wasm-dev/revise_authored_route \
  content-packs/minnesota-current-outdoor-north-stairs-v01.crystalpack \
  planning/minnesota/revisions/r10-cache-cliff-v01/before.layout.json \
  planning/minnesota/revisions/r10-cache-cliff-v01/after.layout.json \
  planning/minnesota/revisions/r10-cache-cliff-v01/dossier.json \
  content-packs/minnesota-current-r10-cache-cliff-v01.crystalpack
```

It failed before writing a pack in `RouteDesign::parse`: the historical R10
schema requires the carrier hop that the native outdoor repair already closed.
The updated Rust schema accepts its exact named closed face, preserves both
regional flags and the required habitat, verifies FLOOR/WALL retaining collision,
and still requires the ordinary stair return. Three fresh crate regressions
pass, including rejected hop restoration, an open face, unsafe approaches,
disconnected return, missing habitat/flags and altered closure bounds. Proof:
`closed-cliff-test-proof-v91.json` and its complete test log. This parser repair
does not certify the rejected new contour.

The idle v88 publication waiter was stopped only after the grass failure was
found, with no child/compiler or native publication started. Its withdrawal is
recorded in `withdrawal-v88.json`. Do not restart it on this rejected geometry.

## Next implementation

Recompose the cache barrier so every ordinary approach still crosses at least
four real grass tiles while Iva and the mine remain reachable. Audit the native
directional rim and eastern approach, not only a FLOOR-only graph. The current
generic route publisher does not type or enforce `optional_grass_rewards`;
close that publication-validation gap before accepting this source. Then rerun
native publication, `audit_route_terrain_revision.py`, all377 audio checks and
`check_r10_cache_mcp.py` against fresh identity-bound initial saves. The latter
checks the cache approach/return, east-face denials and Iva's production battle
menu; it has been authored and syntax-checked but not executed on an R10 child.

Fresh native screenshots and complete visual inspection remain required. The
western stacked boulders at x7..11 are a separate unresolved contour; no claim
is made that the entire western formation has been repaired.
