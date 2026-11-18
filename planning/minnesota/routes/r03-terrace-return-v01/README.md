# River Valley: supply shelf and terrace return

A bounded outdoor revision of `MnR03C00R00`, built from the selected Crossings
pack. The combined release owner registers and selects it; this author does
not deploy or change the selected world pack. The cave, fishing house, both
bridges, regional ports and all other routes remain untouched.

## Why this stretch

The installed routes were reviewed as complete native-art maps and at 160×144
player scale, rather than rebuilding older canonical route dossiers:

- R02 has a recognisable farm drain, nursery bank, tool yard and westbound drop.
- R03's low terrace offers an item and a trainer but sends both journeys through
  the same east staircase. The supply ball at `(39,35)` cannot be seen from the
  south decision at `(39,43)`. The lower apron is wider than its purpose needs.
- R04 already offers a legible civic bridge, rose garden detour and picnic lawn.
- R05 has a distinct two-terrace silhouette, two-mouth rock cut and river pier.
  Its currently installed stairs differ from the historical ledge dossier;
  that dossier is not treated as proof of current travel.

R03 is the focused choice: give an existing discovery a visible promise and a
short return without adding a second reward, another battle or an HM gate.

## Reference comparison

Mainline Crystal Route 29's separated upper paths and south-facing drops make
outbound exploration and the trip home differ. Its native map was rendered
from the supplied pack and its original event script checked at
<https://github.com/pret/pokecrystal/blob/master/maps/Route29.asm>.
The NPC's explanation ties the ledges to the useful grass-free return, rather
than calling attention to an arbitrary obstacle.

The existing local Crystal Legacy clone at
`/Users/ryanculligan/Documents/Geothite-References/rom-hacks/Pokemon_Crystal_Legacy`
was inspected at `77e6b3859349698ad0a7a8a93cd27f78860d3e4a`:
`maps/Route29.blk`, `maps/Route29.asm`, and the Johto metatile/graphics sources.
Its 270 route metatiles are identical to the mainline layout; the original
source graphics match the pack after accounting for the 32 blank source tiles
between banks. Both maps were rendered and visually compared. This mod retains
the useful original topology instead of adding complexity for its own sake.
That is the design lesson used here, not a claim that Legacy invented the loop.
Polished Crystal's Route 29 event source was also read: it adds explicit Cut
objects and a northern supply ball. No new Cut prerequisite is appropriate
for this already-open Minnesota terrace.

## Exact design

Coordinates are 16px walking tiles unless called metatiles.

- Keep the east stair at `(41,41)` as the ordinary climb and two-way fallback.
- Replace metatile `(18,20)` with existing native `0x56`: the two upper
  quadrants `(36,40)` and `(37,40)` are `HOP_DOWN`; the row below is `WALL`.
  Both land on ordinary floor at y42. The adjacent cliff quadrants close both
  sides. No collision table or new graphic is introduced.
- Place a short native path across metatiles `(18..20,19)`. This joins the
  staircase, supply and return lip in a single player view. The first preview's
  longer north/south stripe was rejected and retained as review history.
- Move the original `MN_R03_ITEM_04` from `(39,35)` to `(39,39)`, preserving its
  item, quantity, script and collection flag. The ball is visible above the
  lip from `(39,43)`; collected saves do not gain another item.
- Extend the existing southern woods across metatiles `(16..19,22)`. This makes
  a two-tile east/west approach with a turn beside the existing sign, instead
  of a broad empty apron. Every encounter-grass cell stays exact.
- The existing sign at `(40,44)` names Minneapolis north and Mankato south,
  then explains the terrace supplies, east steps and southward return. Each
  page is authored explicitly with at most two 18-character lines.

`layout.json` owns these eight terrain edits, the item coordinate, and sign
pages. `northstar_river_terrace.rs` guards their native preimages and uses the
core movement/collision functions for static directed paths. It creates no
controller, game session, save or input stream. No gameplay was performed.

## Reproduction and integration

Use the Mac's shared `target` cache, coordinated with the release owner:

```sh
CARGO_INCREMENTAL=0 CRYSTAL_TERRACE_PARENT=content-packs/minnesota-crossings-v01.crystalpack \
  cargo test --locked -j 1 -p crystal-assets --lib \
  northstar_river_terrace::tests::terrace_return_collision_and_preservation \
  -- --ignored --exact --nocapture
CARGO_INCREMENTAL=0 cargo run --locked -j 1 -p crystal-assets --bin pack_river_terrace -- \
  content-packs/minnesota-crossings-v01.crystalpack \
  content-packs/minnesota-r03-terrace-return-v01.crystalpack
uv run --offline --with pillow --with cbor2 python \
  planning/minnesota/routes/r03-terrace-return-v01/review.py \
  --parent content-packs/minnesota-crossings-v01.crystalpack \
  --pack content-packs/minnesota-r03-terrace-return-v01.crystalpack \
  --output target/route-fork-20261118/compiled-review
```

Headless production captures use `review_connected_route_native.py`, the
existing `target/release/examples/render_at_location`, hour 12, native 2D,
and the three cameras `(39,43)`, `(41,42)`, `(40,39)`. Captures and original
source renders are retained under ignored `target/route-fork-20261118/`.
The review wrapper is unselected; the combined release owns identity and
existing-save migration. In particular, reconcile retained terrain snapshots
and actor coordinates when integrating the changed map into a compatible save.

## Review limits

The existing connected-route comparison tool successfully captures the real
headless client, but its raw source RGB comparator reports a full mismatch
even for the unchanged parent. Inspection found no exact palette-color
intersection: for example source `(222,255,222)` renders as `(209,240,209)`.
This is a presentation-color difference, not evidence of changed terrain. Retain that failed receipt; native screenshots are inspected directly,
and the source/preservation audit remains a separate data check. Do not label
that old pixel comparator a pass.

The first two shared Cargo invocations selected an older isolated Lakeside test
executable and ran zero matching terrace tests. Those are not validation. The
release owner rebuilt an immutable current-source test executable, verified its
named test list, and executed the exact terrace test: **1 passed**. The retained
logs are `target/field-review-20261118/terrace.log` (Crossings) and
`target/field-review-20261118/lakeside-terrace.log` (private Lakeside). Both
execute the same exact test and pass independently.

## Verified directed travel

The core-collision test executed without a controller or play session:

| Journey | Actions | Route constraint |
| --- | ---: | --- |
| `(37,42)` uphill to `(37,40)` | 10 | Goes through east stair `(41,41)` |
| `(37,40)` downhill to `(37,42)` | 1 | Native two-tile south hop |
| Lower path `(37,43)` to supply approach `(40,39)` | 9 | Ordinary east staircase |
| Supply approach back to lower path | 6 | Uses the short return ledge |

Both ledge lanes, closed side quadrants, the local north/south through route
in both directions, all native actor bodies, duplicate application rejection,
and unexpected terrain rejection passed. Restoring the owned map reproduces
the entire original data set exactly; runtime art and compiled audio also match.

## Compiled review and visual acceptance

`content-packs/minnesota-r03-terrace-return-v01.crystalpack` was built through
the registered Rust CLI and read back exactly. `review.py` passed on that
compiled pack: eight metatile changes, all 742 other maps identical, original
trainer teams in the active Minnesota roster, and all art/audio exact. Every
encounter-grass, water and Cut quadrant remains unchanged. The original ETHER
and collection flag remain intact at their revised coordinate. All six sign
lines match the compiled text body and split script table.

All three actual headless production views were inspected before and after.
The lower decision now shows the ball, east steps, short hop and return path
together. The complete route and unchanged north meadow were also inspected.
The accepted comparison is `target/route-fork-20261118/native-before-after.png`;
full-resolution originals, rejected preview v1, accepted preview v2, complete
route renders and capture receipts remain in that ignored review directory.
No game inputs, collection/battle playthrough, selected-pack change or public
deployment were performed. Existing-save integration belongs to the combined
release owner.
