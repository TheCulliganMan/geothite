# R10 cache terrace and native stair repair

Current integration: this phase is included in the selected outdoor terrain
cohort with the later west/carrier repairs. Resolve `world.json` and
`../outdoor-terrain-integration-v01/sources.json`; the parent/selection statements
below retain the historical cache-stage review state.

Native review pack: `content-packs/minnesota-current-r10-cache-cliff-v04.crystalpack`,
SHA256 `b42c0c43dcf2f62ccaad99aa60443edf088907dce1debde5c8edb6eb891020e7`.
The normal Rust CLI's v03 output is byte-identical. Neither is selected in
`world.json`; the active world remains the earlier Duluth cliff-stairs v03.

## Place and gameplay

Iron Range Descent remains48×20 metatiles, with32×20 and16×20 exterior chunks.
The western road connects Grand Rapids to the eastern Duluth approach through
the existing reciprocal chunk seam. Q06 still requires `BADGE_7` and
`EVENT_MN_DEPOT_CLEARED`; no gate or story flag is removed.

The upper MAX_REPEL cache remains at core(27,9). Its original grass threshold
and new northern forecourt jointly require at least four actual grass steps
from the regional arrival. The native graph includes directional cliff-rim
movement and clears other actor bodies so they cannot masquerade as permanent
barriers. Source/native validation rejects the preceding dry forecourt design.
The required eastern pine neck remains in the dossier, without a widened road.

Iva remains at(37,13), facing left. Her clear paved approach is(36,13), reached
from(36,14). The mine entrance remains(36,17), with the existing lower return
at(36,18) into `MnR10Cave1F`; both room/warp anchors are unchanged. All six
trainers, three residents, three pickups, six signs and the existing cave remain.
The other pickups are the HYPER_POTION at(17,27) and ULTRA_BALL at(85,19).

All coordinates below are32px metatiles unless explicitly called core tiles.

| Segment | Placement | Native result |
| --- | --- | --- |
| Continuous northern rim | (14..19,3) |70 |
| Actual northeast turn | (20,3) |6b |
| Grass continuing to the right face | (20,4..5) | private81 |
| Northern encounter threshold | (17..19,4..5) |03 |
| Clear mine forecourt | (19,6) |71 |
| Reclaimed grass shoulder | (16,5) |03 |
| Matching outdoor stair cap | (5,10) | private82 |

These18 cells change only `MnR10C00R00`. The original129 R10 metatiles remain
pixel-exact in all four palette banks. Private81 uses native03's left glyph
columns and native69's right columns; its collision is
TALL_GRASS/WALL/TALL_GRASS/WALL. Private82 uses native72's upper landing rows
and native stair80's unchanged lower gray treads, retaining cave12's
FLOOR/FLOOR/WALL/FLOOR collision. No glyphs, palettes, foreground masks or
source images are allocated or recolored. The stair's actual walking lane is
core(11,20)→(11,21)→(11,22), in both directions.

## Verified native build and evidence

The following normal Rust build and publication were executed successfully.
The output names already exist; use fresh names for another run.

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1 RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_r10_cache
target/runtime-fps/debug/revise_r10_cache \
  content-packs/minnesota-current-outdoor-north-stairs-v01.crystalpack \
  694e0700d4777b4bc1c06065e92a3ca58f8140ef94a33412120ddec05bf27ca0 \
  planning/minnesota/revisions/r10-cache-cliff-v02/before.layout.json \
  planning/minnesota/revisions/r10-cache-cliff-v02/after.layout.json \
  planning/minnesota/revisions/r10-cache-cliff-v02/dossier.json \
  content-packs/minnesota-current-r10-cache-art-v01.crystalpack \
  content-packs/minnesota-current-r10-cache-cliff-v03.crystalpack
```

The public-API publication v103 independently rejects the known dry cache bypass
with native minimum cost0 before writing any output. Its accepted current
source passes complete native geometry, actor/room anchors and the four-step
habitat guard. `normal-cli-equivalence-v111.json` binds both successful outputs.

Evidence under `renders/r10-western-cliff-v01/`:

- `grass-units-v93.log`: two numeric/native surface regressions pass, including
  lateral UP_WALL bypass, actor removal, warp exclusion and false grass cost.
- `native-regression-direct-v101.log` and `native-reward-regression-v93.log`:
  actual native baseline passes, rejected forecourt fails, revised whole route
  and both native art joins pass. External prerequisites are read at runtime.
- `art-preservation-v105.json`: both joins are exact across all four banks;
  original129 images, every map, actor, event, script, encounter, foreground and
  audio/root field remain exact. Actor/foreground mutation checks are rejected.
- `terrain-preservation-v105/`:752 exact checks and four negative mutations
  pass; only18 cells in one exterior and its encoded block mirror change.
- `audio-v105.log`: all377 PCM hashes, frame counts and loop ranges pass.
- `native-stdio-v106/`: actual cache approach/return, Iva's native trainer battle
  menu, reached-save Continue and production Start menu pass. Local initial
  party, position and Repel are declared fixtures; no state is repaired later.
- `stair82-stdio-v106/`: twelve production RPCs prove both stair directions and
  repeated side denials.
- `wall-stdio-v110/`:57 RPCs prove the east face and southeast corner with three
  directional presses per probe, plus actual reached-save Continue/Start. The
  earlier single-press wall checks only established turns and are superseded;
  retain their raw records and `wall-proof-strengthening-v111.json`.
- `regional-species-closure-v104.json`: all eighteen base species across six
  trainer teams, both wild tables and all rods/time groups, plus all34 reachable
  native evolutions, remain active original Minnesota species. Both gift lists
  are empty. The POUCHID/CINDORE fixtures are also active originals.
- `native-day-existing-v108/`: five unmodified production GPU captures pass
  zero pixel mismatches and were inspected. They show the cache, both grass
  approaches, Iva/mine forecourt and brown-capped stair using the existing
  production binary. Fresh-current-source captures remain a separate pending
  cohort under `native-fresh-v111/`.

The immutable whole route rendered directly from the written pack is under
`native-whole-v113/`. It uses the exact native reciprocal seam and original
starting actor poses; it is a native-source atlas, not a whole-map GPU capture.
Source before/after views and rejected attempts remain in `source-v85/`,
`source-v86/`, `source-v92/`, `review-loop-v94/` and `warm-loop-v98/`.

## Fast review loop

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_r10_cache.py \
  --pack content-packs/minnesota-current-outdoor-north-stairs-v01.crystalpack \
  --layout planning/minnesota/revisions/r10-cache-cliff-v02/after.layout.json \
  --dossier planning/minnesota/revisions/r10-cache-cliff-v02/dossier.json \
  --output planning/minnesota/renders/r10-cache-live-review --watch
```

The pack/art load happens once. The measured second warm render took0.670s;
`warm-loop-v98/timings.json` preserves both measured complete renders. Every
revision keeps its original terrain, complete route and enlarged decision
views. Invalid edits are reported without ending watch mode. This source loop
does not certify optional grass or gameplay; the native publisher now enforces
the habitat against actual numeric movement permissions.

## Remaining work

R10's older stair alias now inherits native Johto water/flower animations in
the renderer. The current-source five-alias animation regression passes in
`renders/outdoor-stair-landings-v01/five-alias-animation-v112.log`, including
R10 and the winter clock/icy-rustle comparison. All five fresh-source R10 GPU
views also pass with zero pixel mismatches in `native-fresh-v111/`; each actual
production image was inspected. These results apply to cache-cliff v04, not
the subsequent west-terrace candidate.
The winter stair alias also now follows the source winter clock lighting,
correcting the actual morning icy-rustle mismatch found in v97 and passed in
v107/v66. Do not treat those four-alias receipts as the fifth-pair proof.

Parent-cohort asset and Bevy all-targets Clippy now pass in the v66 logs.
The western
stacked boulders at x7..11 still need their own coherent contour; this local
repair does not certify the entire route, statewide progression, deployment,
balance or earned access. The cache was approached, not collected, and Iva's
challenge was shown without claiming a completed battle turn.
