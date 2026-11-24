# Minnesota cave routes and landmarks

The [original Crystal cave atlas](../crystal-reference.html) is the visual
reference. Union Cave and Slowpoke Well use shelves to distinguish walking
routes from water approaches. Dark Cave and Rock Tunnel use rock masses and
ladders to give different trips through the same dungeon. Ice Path gives its
sliding lanes readable places to launch, stop and turn.

The 19 Minnesota interiors use those principles in their own layouts.
Recessed boulders and ledges give the caves bends that matter. Ladders,
switches, fossils, supplies and water landings have visible locations in the
native camera. The quiet outer terrain from the wall finish remains the
backdrop.

- [Southern rooms](SOUTH.md) restores the mouth colors in Driftless Hollow and
  Rock Cut, puts the Lamp Gallery relief at its actual read point, and
  shapes the River Pocket, Cedar Shelter and Granite Den shores around their
  approach routes.
- The [northern caves](NORTH.md) give Gunflint a camp bench and a lower bank, Winding
  Cleft a lookout that sees the search rock, Ore Line separate fissure and
  pool approaches, and Inner Basin a distinct sanctuary behind the spillway.
- The [Iron Range](IRON-RANGE.md) has aligned sidings, a readable service route alongside
  the flooded Sump, and cues at the two pressure controls, feed isolation
  and Drill Core. The foreman shaft and three machine states use their
  original scripts and destinations.
- Voyageurs Ice 1F has a frost lip at the first launch, a dry stop at the
  bend and a short raised shelf near the far ladder. On B1F, the Revive is
  reached by turning off the first chute. The NeverMeltIce lies across the
  lower sheet. The return graph was checked with each combination of
  collected items so none of the pickups are needed as stopping points.

## Rebuild

The [build recipe](build.json) binds the wall-finish parent and every terrain
and art source. The Rust builder appends only designated metatiles and
original glyphs in unused addresses. It mirrors the collision and palette
tables and refuses edits that alter existing entrance or event collision.
The local pack has its own save identity.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets --features pack-authoring \
  --bin pack_northstar_cave_detail --bin verify_crystalpack
target/debug/pack_northstar_cave_detail \
  content-packs/minnesota-cave-wall-finish-v03.crystalpack \
  planning/minnesota/caves/crystal-detail-v04 \
  content-packs/minnesota-cave-detail-v04.crystalpack
target/debug/verify_crystalpack \
  content-packs/minnesota-cave-detail-v04.crystalpack --audio --desktop
uv run --with pillow --with cbor2 python planning/minnesota/caves/crystal-detail-v04/review.py \
  content-packs/minnesota-fair-presentation-v01.crystalpack \
  content-packs/minnesota-cave-wall-finish-v03.crystalpack \
  content-packs/minnesota-cave-detail-v04.crystalpack
```

The static review reads the installed archive and checks its exact terrain and
native tile art, prior event identities, flood and drain states, mine gate
capabilities, and ice turns, pickups and return paths. The authoring review
also compares full-floor layouts and same-position native camera images with
the wall-finish pack.

Scoped headless input checks use the native renderer's location fixture.
The first eastward slide stops at the bend at (18, 11), including when Up is
pressed during the glide. The far ladder lands at (5, 4) on B1F and returns to
(35, 6) upstairs. The west mouth returns to the passive exterior landing at
(6, 34). These fixtures have their own starting locations and observe the
engine's committed map and tile in its movement trace.

The same fixture verifies the lower Driftless Hollow bend from (19, 19) to
(26, 19) by walking around the staggered rock shoulder. The compiled graph
keeps both full routes through the bend at 58 steps.
