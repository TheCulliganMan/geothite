# Headwaters stair boulder clearance

The complete snowline is verified in the
[Headwaters runbook](../../regions/headwaters-snowline-v01/README.md). Its native
stair camera exposed three older boulders touching the south terrace face or
crowding the lower stair shoulder. This proposal moves each one metatile away
from that face while retaining all ten northern-chunk boulders and the exact
native block inventory. Rust publication, controller/field-move/Continue and
native day/night visual review now pass. The frozen source's original proposal
status remains history; this runbook records the verified result.

Authoritative source: [`source-v01.json`](source-v01.json), SHA256
`e80f053d3035ad03e764cf96179a245e327bd3dfc2b87546b3840f5812531c18`.
Parent: `content-packs/minnesota-headwaters-snowline-v01.crystalpack`, SHA256
`2b8ea8076b18ee740e832beb11d1fd4ae3c5c1ef883945b8db78f579a6c2df48`.
Only `MnR08C00R00`, native alias `mn_winter_headwaters_stairs`, changes.
Every complete before/after row and the original module hash are bound in the
source. No source art, cliff, stair, tree gate, actor, reward or connection changes.

| Metatile | Before | After | Meaning |
| --- | --- | --- | --- |
| (13,12) | 0a | 03 | Restore upper grass beside the south face |
| (13,11) | 03 | 0a | Move western upper boulder inward one metatile |
| (20,12) | 0a | 71 | Restore upper terrace floor beside the face |
| (20,11) | 71 | 0a | Move eastern upper boulder inward one metatile |
| (16,14) | 0a | 03 | Clear the lower stair's left grass shoulder |
| (16,15) | 03 | 0a | Move lower boulder away from the face/landing |

The read-only whole-route preview and paired stair detail are retained in
`renders/headwaters-rock-source-v1202/` and were visually inspected. The source
review preserves all ten rocks, the exact block/collision inventory, all1275
reachable floor/grass cells by count and reachability of every unchanged cell.
Exactly twelve newly occupied cells replace twelve released cells. Both stair
ends and both public chunk mouths remain reachable, and no destination
metatile overlaps an actor's source coordinate. This is conservative static
source evidence, not native walking acceptance.

## Native candidate and preservation

Child: `content-packs/minnesota-headwaters-rock-clearance-v01.crystalpack`, SHA256
`d2b9d98f8204d216b9cdf7eb386441efd374a47424ee615c891071764f630bf9`, content
identity `e226d86fe5eaea295b5db8cde2e85c559f9cd2d1cdb6bb9c56c1372748e997bc`.
This unselected stage is followed by the verified [Itasca snowline](../../regions/itasca-snowline-v01/README.md); use that child for continuation. It retains the full
Leech thaw and every previous winter/grass-Cut/cliff/roof repair. The world
selector remains `north_shore_passing_width_revision`; no app/browser release,
staging, commit or push occurred.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_headwaters_rock_clearance.py \
  --output planning/minnesota/renders/headwaters-rock-source-FRESH
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_headwaters_rock_clearance
target/verdigrim-scene/debug/revise_headwaters_rock_clearance \
  content-packs/minnesota-headwaters-snowline-v01.crystalpack \
  2b8ea8076b18ee740e832beb11d1fd4ae3c5c1ef883945b8db78f579a6c2df48 \
  planning/minnesota/revisions/headwaters-rock-clearance-v01/source-v01.json \
  content-packs/minnesota-headwaters-rock-clearance-FRESH.crystalpack
```

Serialize Cargo with `.local/minnesota-cargo.slot.lock`. Choose fresh ignored
outputs. The reusable whole-route preview takes0.202 seconds after pack load;
`--installed PACK` also compares the published child. It audits source actor
bodies separately: all1270 remaining reachable cells by count and every unchanged
cell's reachability remain exact with those bodies excluded.

The Rust publisher guards the complete parent/module/rows, all six allowed
changes, exact native collision and complete block inventory, then requires a
full residual equality check. Strict readback passes. Wrong-parent and unlisted
row-edit probes refuse without writing a pack (`.local/headwaters-rock-guards-v1213/`).
The source/installed review in `renders/headwaters-rock-installed-v1206/`
proves all742 other complete maps, every runtime asset and602 audio entries
exact. The complete1,966,080-pixel route matches independent native source art
with zero differences; only the six declared metatiles differ from its parent.

## Controller and native visual review

`renders/headwaters-rock-route-v1207/` passes **18 sessions and4,467 production
stdio calls**, with byte-empty stderr. It walks all four chunks, the fitted
stair in both directions, actual Bemidji/Leech seams and return, Start and
Continue. It retains pier-tree Cut refusals, successful entry, inside-face
re-cutting after native regrowth, and cold/melt grass Cut/Continue against a
native green control. Reproduce it with `check_headwaters_snowline_mcp.py`
using the child above and the current client/Rust fixture writer from the
[Headwaters runbook](../../regions/headwaters-snowline-v01/README.md).

`renders/headwaters-rock-physics-v1211/` independently passes **six zero-badge,
no-Cut sessions and456 production calls**, including Continue. Ordinary inputs
walk the released upper floors and lower grass shoulder, hit all three moved
rocks, check the real west/south cliff blocking edges, and climb/descend the
unchanged native stair. The native south face's upper half is walkable; its
lower row blocks the drop. The earlier v1209 fixture incorrectly expected the
upper row to block and is retained as a rejected test assumption. The v1210
course encountered a normal wild battle after Continue; the completed course
uses native Run with its declared Smoke Ball and resumes the same walk.
No game state is rewritten after input.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/check_headwaters_rocks_mcp.py \
  --pack content-packs/minnesota-headwaters-rock-clearance-v01.crystalpack \
  --fixture-writer target/verdigrim-scene/debug/examples/inland_grass_cut_fixture \
  --client target/verdigrim-scene/debug/geothite \
  --output planning/minnesota/renders/headwaters-rock-physics-FRESH
```

All eight matched parent/child native GPU day/night views were inspected in
`renders/headwaters-rock-native-v1208/`. Original upper floor/grass separates
the upper boulders from the continuous face; the lower boulder clears the left
stair landing. The fitted stairs and native palettes remain unchanged. Every
original, pair and rejected pass remains retained. All-target crystal-assets
Clippy and owned Rust formatting pass; whole-workspace formatting has existing
unrelated differences. Source/actor setup remains explicit and these checks do
not establish earned campaign progression.

The Itasca thaw is now separately verified. The larger goal still includes latest story/Ely integration,
remaining cave and terrain review, combined field/controller/native/connected
atlas checks and final selection. Broader clearing-density review remains open.
