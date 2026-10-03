# Rocket Base B1F potted plants

This change replaces the two remaining native plant cutouts with the existing
independently authored `interior/plant_tropic` mesh. Its radial, drooping fronds,
visible central trunk, recessed soil and rolled round pot match the source
plant's form. The generic branching-leaf plant and dense rounded jumbo crown
were compared and are less faithful silhouettes. No borrowed or newly generated
mesh is introduced; `interior_models::model(PlantTropic)` retains its existing
bounded cache. The editable source remains `ornamental_plant(..., 'tropic')` in
`tools/build-johto-interiors.py`.

## Exact source and ownership

- Map: `TeamRocketBaseB1F`; atlas: `underground`
- West block: `0x29` at map source-cell origin `(24,24)`; drawing begins at
  local `(2,0)`, hence map source-cell `(26,24)`
- East block: `0x2a` at map source-cell origin `(28,24)`; drawing begins at
  local `(0,0)`, hence map source-cell `(28,24)`
- Both drawings are `[[0x1e,0x1f],[0x2e,0x2f],[0x3e,0x3f]]`, each 16×24 pixels
- Each entire 4×4 native block is an identity/custom/crop guard. All ten cells
  outside each plant are floor tile `0x10`, including its underlay donor directly
  below the plant. Only the six plant cells acquire model ownership
- Underlay uses that block's live `0x10` texture at the original zero datum
- Mesh bounds are 15.28px wide, 12px deep, 23px tall, with the authored front
  foot line at source y=24. Width/depth remain inside the existing drawing
- All source support heights, collision, objects, scripts and events are read
  without alteration. Source drawing cells have native WALL collision

Unknown maps, atlases, metatiles, source phases, tile art, cropped guard cells,
custom profiles (including incomplete profiles with absent ground), and existing
ownership reject a whole plant before any mesh write. The other plant remains
independent. The existing flat/cutout path stays available for those rejected
drawings. No coverage label is assigned until the model is appended.

## Verification and remaining native gate

The standalone production-source harness was run against the ignored external
`core-modular.browser.crystalpack`. It passed: two plants, 12 owned cells, 32
guard cells, every identity/custom mutation, four-sided cropping, scrolling and
padding. It also verified the native source collision and no claimed overlap
with all 6 actors, 4 warps, 30 coordinate events and 9 background events. The
native map has no scripted block changes.

Run after integration:

```sh
python tools/check-rocket-plants-source.py --pack content-packs/core-modular.browser.crystalpack
cargo test -p crystal-voxel-view rocket_plants
```

The integrated Cargo regressions pass. They check cached
model output, finite geometry and bounds, exact underlay UVs, 12-cell ownership,
unchanged support, custom ownership and stale-source atomic append rejection.
The production audit confirms exactly 12 fewer unmodeled cutout cells and no other category change. All 388 base maps build without errors.

Native front/back/side and paired classic-view checks pass at the verified
FLOOR gameplay tile `(13,15)`. Both adjacent foliage/trunk/pot silhouettes are
complete, their native diamond underlays remain, and spacing is retained.
Ordinary held-Down input moved the player one tile to `(13,16)` and stopped
at the native wall `(13,17)`, with no extra terrain rebuild. Neither approach
tile overlaps an event or actor. Paired classic captures retain the source
plants. The high zoom deliberately frames the plants closely; it is not a
full-route camera or trainer/sensor acceptance test.

The reused blue-gray pot palette is the existing artistic 3D treatment, not a
claim of source-pixel color equality. This closes the two static cutouts only;
surrounding wall skins, equipment, floor treatment and unrelated trainer/sensor
scripts remain separate review work.
