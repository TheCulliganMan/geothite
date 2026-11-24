# Crystal cave audit and replacement

Start with the [floor-by-floor audit](AUDIT.md) and the [original Crystal cave pictures](../crystal-reference.html).
This revision covers the 14 Minnesota caves and five rock mine floors that
were present in the selected v04 local pack. The layouts are authored in
`south.json`, `north.json`, `mines.json` and `ice.json`. The art files append
native metatiles, including quiet ice background and new glacier faces.
`sources.json` binds each room to its exact installed rows. All coordinates
in event moves and arrivals are walking tiles. Footprint dimensions count
32-pixel metatiles.

This pack has a new content identity and uses a separate local save slot.
The terrain, moved event positions, two ice footprints, trimmed Lamp Gallery,
and selected cave lighting are authored together. Outside entrances, warp
identifiers, progression flags, rewards and encounter tables retain their
parent contracts.

## Build and check locally

From the repository root, use the exact v04 parent pinned in
[build.json](build.json) and choose an unused ignored output path.

```sh
cargo build --locked -p crystal-assets --features pack-authoring \
  --bin pack_northstar_cave_detail --bin verify_crystalpack
target/debug/pack_northstar_cave_detail \
  content-packs/minnesota-cave-detail-v04.crystalpack \
  planning/minnesota/caves/crystal-audit-v05 \
  content-packs/minnesota-cave-audit-v05.crystalpack --audit
target/debug/verify_crystalpack \
  content-packs/minnesota-cave-audit-v05.crystalpack --audio --desktop
```

With Pillow and cbor2 installed, run the decoded terrain and mechanics audits
against the built pack. Each render command takes a new output directory under
`target/`.

```sh
PARENT=content-packs/minnesota-cave-detail-v04.crystalpack
CANDIDATE=content-packs/minnesota-cave-audit-v05.crystalpack
SOURCE=planning/minnesota/caves/crystal-audit-v05
uv run --with pillow --with cbor2 python "$SOURCE/review.py" \
  "$PARENT" "$CANDIDATE" --renders target/cave-audit-review/floorplans
uv run --with pillow --with cbor2 python "$SOURCE/check_south.py" "$CANDIDATE"
uv run --with pillow --with cbor2 python "$SOURCE/review_north_audit.py" \
  "$PARENT" "$CANDIDATE" target/cave-audit-review/north
uv run --with pillow --with cbor2 python "$SOURCE/review_mine_audit.py" "$PARENT" "$CANDIDATE"
uv run --with pillow --with cbor2 python planning/minnesota/tools/review_current_caves.py \
  --output target/cave-audit-review/selected
```

## Build the public editions

The production builder carries the reviewed cave modules and native tiles into
the exact published Pokédex-order release. It takes the raw and hosted parent
archives, and the exact v05 review pack above. The raw output is the server's
input. The hosted output is the expected browser and installer package after
applying the server overlays.

```sh
cargo build --locked -p crystal-assets --features pack-authoring \
  --example pack_northstar_cave_release --bin verify_crystalpack
target/debug/examples/pack_northstar_cave_release \
  content-packs/minnesota-pokedex-order-v01.crystalpack \
  "$HOSTED_DEX_PARENT" \
  content-packs/minnesota-cave-audit-v05.crystalpack \
  "$RAW_CAVE_RELEASE" "$HOSTED_CAVE_RELEASE"
target/debug/verify_crystalpack "$RAW_CAVE_RELEASE" --desktop --audio
target/debug/verify_crystalpack "$HOSTED_CAVE_RELEASE" --desktop --audio
uv run --with pillow --with cbor2 python \
  planning/minnesota/caves/crystal-audit-v05/verify_release.py \
  content-packs/minnesota-pokedex-order-v01.crystalpack "$HOSTED_DEX_PARENT" \
  content-packs/minnesota-cave-audit-v05.crystalpack \
  "$RAW_CAVE_RELEASE" "$HOSTED_CAVE_RELEASE"
```

The release policy retains the public Pokédex save history. A checkpoint
inside a revised cave resumes at a clear arrival in that same room. Active
scenes and movement must settle before upgrading.
