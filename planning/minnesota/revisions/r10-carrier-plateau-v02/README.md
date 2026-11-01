# R10 continuous western ridge

This revision joins the western porch to the carrier ridge, removes the empty
northern rock finger and internal cliff faces, and opens a reachable grass cove
below the front wall. The southern public corner closes with a native turn;
its boulder sits one metatile inside the terrace. `plan.json` specifies all44
changed metatiles. `after.layout.json` is the complete48×20 route, rather than
a decorative crop. It supersedes the rejected carrier-v01 and southern-rim-v01.

The inward turn at(9,7), also used at(14,13), uses private85: native71 floor
in three quadrants, native69's upper-right quadrant in the fourth. Collision is
FLOOR/WALL/FLOOR/FLOOR. The rejected version put a north-rim stripe below the
wall. Native movement and pixel equality alone did not catch that composition
error; the retained corner comparison selected this clean floor ending.

The existing two stairs remain at(5,10) and(16,6), with matching brown outdoor
landings and unchanged native gray treads. The route retains its two regional
gates, six trainers, three residents, three pickups, six signs and mine entrance.
The cache still requires at least four grass steps and the eastern pine neck
requires six. All152 grass walking quadrants belong to authored habitat; the
new lower cove has an ordinary return through the western stair.

## Native implementation

Executed builder: `revise_r10_carrier_plateau PACK SHA256 BEFORE AFTER DOSSIER
ART_OUTPUT OUTPUT`, built with the pinned Rust toolchain in `target/runtime-fps`.
Use `before.layout.json`, `after.layout.json` and `dossier.json` from this folder.
Both output paths must be fresh. The parent is
`content-packs/minnesota-current-r10-west-terrace-v01.crystalpack`, SHA256
`e6c2126e57e1349ae73ba42c6139d8e1cc7f3b61298e2df892b0d122866da536`.

The written result is
`content-packs/minnesota-current-r10-carrier-plateau-v02.crystalpack`, SHA256
`2edf7312987a5d270d62e329e958c5ae577e95d2daf687b1eca67aa9150ee4cb`.
Its newer composed child also retains the accepted Grand Rapids, Grand Marais
and Rochester façade repairs. Resolve `world.json` for the active selection.

Evidence is retained under `renders/r10-western-cliff-v01/`:

- v141: native publication, exact art/root preservation,752 terrain checks,
  four rejected mutations and all377 audio hash/frame/loop checks.
- v143: two stair courses and ridge/grass/cache courses pass460 actual MCP
  calls, including reached-save Continue and the production Start menu.
- v144:54 further native calls check the southern public rim and Continue.
- v143 GPU: all11 captures have zero source/native pixel differences; all11
  were inspected in `eleven-view-contact-v146.png` and individual corner views.
- v142: affected assets all-targets Clippy passes. The parent cohort's Bevy
  all-targets Clippy passes in the v66 log. Existing unrelated warnings and the
  full-workspace formatting failures remain recorded; changed leaf Rust passes.
- v149: the production CLI rejects the old lower-rim recipe before either
  output can be written.

`corner-fidelity-v140/` preserves all three compared endings. Every earlier
render, failed receipt and rejected pack remains available. Initial party,
badge, position and Repel fixtures are local setup; these checks do not prove
earned access, reward collection, completed battles or statewide progression.

## Fast source review

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_r10_cache.py \
  --pack content-packs/minnesota-current-r10-carrier-plateau-v02.crystalpack \
  --layout planning/minnesota/revisions/r10-carrier-plateau-v02/after.layout.json \
  --dossier planning/minnesota/revisions/r10-carrier-plateau-v02/dossier.json \
  --output planning/minnesota/renders/r10-western-cliff-v01/next-source-review
```

The preview writes immutable whole-route images before inspection crops. Native
GPU captures use `review_connected_route_native.py` with this folder's
`views.json`; `public-boundary-course.json` and `stair-inventory.json` define
replayable production input checks.
