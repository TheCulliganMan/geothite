# Winona field station

Pouchid’s capsule sits at the end of Tamarack’s table. From the front, the player walks around the table and up to Tamarack. From the aisle, the same four steps previously aimed at the computer. Choosing Pouchid there now takes two steps across clear floor. Both approaches finish west of Tamarack, with room for Reed to walk up from the door and face the player.

| Partner | Interaction tile | Walk to Tamarack |
| --- | --- | --- |
| Pouchid, side of table | (5, 3) | Left, up |
| Pouchid, front | (6, 4) | Left twice, up twice |
| Kindlit | (7, 4) | Left three times, up twice |
| Birchit | (8, 4) | Left four times, up twice |

The Field and Mothlight editions each descend from their Lake Observations parent. The station artwork, actor placements and capsule gift scripts supply the evidence for the four approaches. The checker walks the native collision tiles, follows the starter branches, and checks Tamarack, Reed and the specimen case throughout the scene up to the battle. It also checks losing access to a gift after accepting, retrying after declining and the aide’s five Poké Balls on either entrance lane. Supplying the parent additionally compares every map, script, team, artwork file and audio record in the archive. [Compiled opening audit](check-opening.py)

## Reproduce and check

Select a parent and output from the [Field receipt](field-build.json) or the [Mothlight receipt](mothlight-build.json). Use a fresh output path. The builder reads the written archive back and compares it with the in-memory result.

```sh
export RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0
cargo run --locked -j1 -p crystal-assets --example pack_station_return -- "$parent" "$child"
src=planning/minnesota/revisions/station-return-v01
uv run --with cbor2 --with pillow python "$src/check-opening.py" "$parent" --baseline
uv run --with cbor2 --with pillow python "$src/check-opening.py" "$child" --parent "$parent"
MN_STATION_RETURN_PARENT="$PWD/$parent" MN_STATION_RETURN_PACK="$PWD/$child" \
  cargo test --locked -j1 -p crystal-assets --lib station_return_ -- --ignored --test-threads=1
```
