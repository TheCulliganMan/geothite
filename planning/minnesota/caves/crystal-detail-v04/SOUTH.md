# Limestone, spring banks, and granite

These six layouts extend the selected wall finish. They retain the quiet native
outer field and the exact event coordinates. Their layout references are
Union Cave's signposted crossings, Dark Cave's attached rock shoulders, Slowpoke
Well's deliberate landings, and the compact north–south climb of Diglett's Cave
in the [Crystal atlas](../crystal-reference.html).

| Room | Revision |
| --- | --- |
| Driftless Hollow | Both surface thresholds have visible yellow lips. The west guide turnout opens at a fitted wall corner. The lower crossing bends between staggered stones and reaches the east mouth without crossing the trainer's sightline. |
| Lamp Gallery | The golden relief now occupies the same rock as its Flash-only read command. The west switchback approaches the relief from the left. The cache is set behind the next bend, and the two-square downhill ledge still saves the long return. |
| River Pocket | A stepped west shoreline exposes the scratched waterline from the dry bank. Behind the Strength gap, a stair and thick rock shelf split the pillar walk from the long downstream pool and the Castorock bed. The front island cache still requires Surf. |
| Rock Cut | The southern healer's turnout sits under a narrow diagonal shelf. On the high side, the supply branch separates from the far mouth. Dark floor leads to both yellow threshold lips, which use a continuous native cliff with the original pair of carpet glyphs. |
| Cedar Shelter | A shallow stair offers access to the spring. Walking around the shore reaches Harper above the water and the Ether in a small, distinct lower recess. Both destinations remain accessible on foot. |
| Granite Den | A continuous cliff separates the crown and its cache from the lower spring. The lichen reader has a recess beside the bend, and the west ledge lands on a short dry shelf with a separate route back to the mouth. |

Run the earlier southern route contracts and the detail checks against the
compiled review candidate with:

```sh
uv run --with cbor2 --with pillow python \
  planning/minnesota/caves/crystal-detail-v04/check_south_detail.py \
  content-packs/minnesota-cave-wall-finish-v03.crystalpack \
  content-packs/minnesota-cave-detail-v04.crystalpack
```

The checker covers reciprocal arrival connections, collection states, ordinary
and downhill returns, the southern trainer bypass, the River Pocket Strength
and Surf restrictions, the Lamp relief and the two sets of marked thresholds.
Whole rooms, original-scale camera comparisons and the rejected iterations are
retained under the ignored `target/cave-crystal-detail/south/` directory.
