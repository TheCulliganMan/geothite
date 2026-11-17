# Minnesota wild encounters

The region follows the campaign bands in [encounter progression](../encounter-progression-v01/progression.json).
Use the authored walking and water profiles in [habitats.json](habitats.json) for
the 20 main and optional routes, towns, caves, and Mississippi branches. A
habitat has recurring residents, a night lineup, and less common specialists.
Local overrides separate adjacent pine, marsh, basalt, and mining pockets.
Specific signatures add a reason to explore individual chunks. Existing levels
carry the badge curve, including early rods and late return Surf levels.

| Place | What distinguishes it |
| --- | --- |
| Driftless opening | Nibbin and milkweed in the woods, Tesslarv and Hushiss on the limestone floor |
| Prairie and river valley | Prairie chickens and skippers inland, caddisflies and toads along the floodplain |
| Twin Cities and bluffs | Urban greenway residents, bluff snakes, duck flocks, and the roadside rink |
| Pine and lake country | Pine-resin Pokémon and porcupines in woods, loons and crayfish near lakes |
| Iron Range and Superior | Mine carts and taconite in working caves, agates on basalt, lighthouse and smokehouse specialists |
| Gunflint, Boundary Waters, and Ely | Portage, granite, quiet water, and northern ridge wildlife |
| Western and border trips | Sugarbeets and prairie specialists, bur-oak Headbutt, cold-water and icefield catches |

The source covers 111 installed wild-table maps, 36 encounter zones, 50 fishing
headers with water, all 41 non-Safari maps with native Headbutt terrain,
and all seven Safari habitats. The build installs 43 previously missing species from 19 reviewed families,
then registers the exact 225-member active roster for Winona research. The final
research reward still requires 225 caught species. Every active species has an
authored acquisition or evolution path outside paid Safari.

Native encounter probabilities, zone bounds, level slots, rod group
indices, Old Rod catches, swarm flags, Safari costs and flee weights are
preserved. Castorock remains available through the fossil and one-time R03
cave discovery. Tugget and Wreckquiem retain their established waters.
Existing scripted discoveries and gift events are preserved, including four
legacy one-off battles and one legacy egg outside the current research roster.

## Build and verify

Run from the repository root with the selected canoe parent and the reviewed
native exports in `target/minnesota-pokemon/native`. Both outputs must be fresh
and ignored. The local selection is recorded in `planning/minnesota/world.json`.

```sh
base="$PWD/content-packs/minnesota-spawn-canoe-v02.crystalpack"
parent="$PWD/target/minnesota-region-encounters/current-roster-parent.crystalpack"
candidate="$PWD/target/minnesota-region-encounters/region-ecology-review.crystalpack"
mkdir -p "$(dirname "$candidate")"
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets \
  --bin pack_minnesota_pokemon --bin pack_minnesota_region_ecology --bin verify_crystalpack
families=MN_F013,MN_F035,MN_F080,MN_F088,MN_F132,MN_F146,MN_F147,MN_F148,MN_F149,MN_F150,MN_F151,MN_F152,MN_F153,MN_F154,MN_F155,MN_F156,MN_F157,MN_F158,MN_F159
RUST_MIN_STACK=67108864 target/debug/pack_minnesota_pokemon "$base" \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json "$parent" --add-families "$families"
RUST_MIN_STACK=67108864 target/debug/pack_minnesota_region_ecology "$parent" \
  planning/minnesota/pokemon/acquisition-v01/region-ecology-v01/habitats.json "$candidate"
uv run --with cbor2 --with pillow python planning/minnesota/tools/audit_region_encounters.py \
  --pack "$candidate" --compare "$parent"
target/debug/verify_crystalpack "$candidate" --audio --desktop
RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 \
  CRYSTAL_WINONA_TEST_PACK="$candidate" CRYSTAL_INTERIOR_PACK="$candidate" \
  cargo test --locked -p crystal-bevy --lib -- \
  region_ecology_ habitats_burrbud_current_r19_tree_capture_and_continue \
  safari_registered_rod_plays_native_cast_and_delivers_capture_without_combat \
  --ignored --test-threads=1
```

The audit checks collision geometry, installed sources, active roster membership,
and exact preservation of the unrelated pack. The controller tests use declared
local positions and parties, then exercise actual gameplay input and capture.
They sample the campaign regions and encounter methods without an uninterrupted
New Game-to-League replay.
