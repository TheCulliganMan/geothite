# League Approach: current hand-authored implementation

The current route is **32×64 metatiles**, split into three clipped native maps
(32×24, 32×24, 32×16), plus the 10×8 Cedar Shelter. This replaces the rejected
96×144 reserve. Earlier eighteen-chunk captures and fixture coordinates are
historical and do not verify this revision.

The authoritative [layout](MN_R13.layout.json) contains explicit strokes;
the [dossier](MN_R13.json) supplies exact actors, teams, text, gates and room
rows. The [complete current render](../renders/league-approach/whole-route-f790eda0.png)
and [player views](../renders/league-approach/viewpoints.png) use unchanged
native Johto art. Generated images, saves and packs remain ignored.

## Traversal and useful branches

| Place | Actual play | Core coordinates |
| --- | --- | --- |
| Border Lake | Northern arrival, Ruth, shore lookout, Avrin and Hyper Potion | Gate `(30,2)`, Ruth `(31,9)`, potion `(13,27)` |
| Wind Tor | Native ramp, grassy crest, Nell and Ultra Ball; southbound drop saves sixteen walking steps | Ramp block `(23,12)` metatiles; upper `(39,23)`, hop face `(39,24)`, landing `(39,26)`, ball `(53,19)` |
| Grass Saddle | Six-step required habitat between fitted granite and pines | Grass core bounds `[18,42,22,48)` |
| Cedar Shelter | Optional native healing, revived partners, restored status/PP and Max Ether | Exterior `(14,53)`, safe return `(14,54)` |
| Long Lake | Reachable high-bank view with actual lake in the native camera and a blocking rail | View `(40,60)`, Max Repel `(37,61)` |
| Dry Pass | Cal's short road challenge or at least six grass steps | Cal `(47,85)`, grass `[42,84,46,90)` |
| Cedar Refuge | Cut seals Ivy and the Revive, including against Surf bypass | Tree `(18,103)`, approach `(18,102)`, Revive `(21,109)` |
| Ely approach | Tomas, scout and a narrowed southern road | Gate `(34,124)`, safe landing `(34,120)` |

One metatile is four 16px gameplay tiles. All actor coordinates above are core
tiles; the ramp's coordinate is explicitly metatiles. The Wind Tor ordinary
uphill return is nineteen steps through the real ramp. Reverse input is
blocked, and side rocks close false climbs. A misplaced sign that opened a
cliff quadrant and a boulder that blocked the road were corrected before
publication. A broad quiet junction was reduced with attached pine banks and
a fitted granite rib, then checked in the native camera.

Cut uses native Johto `63→3f`, opens 59 walking tiles and seals two targets,
Ivy's approach and the Revive before removal. The main passage needs no Cut or
Surf. Continue restores native temporary vegetation while keeping the pickup
flag and bag item; the controller test recuts from inside to prove an exit.
Every placed CUT_TREE quadrant is inventoried. Unconnected lake boundaries
are closed by actual terrain collision; there are no outer WATER quadrants.

Seven original trainers remain: Avrin, Reid, Nell, Cal, Fay, Ivy and Tomas.
Original parties/defeat flags are preserved. Owen, Norek and Varel retain unused
reservations rather than forcing three filler battles. Six residents, six signs
and four exterior rewards keep their stable identities. Exact source rows,
native text pages and legal parties are in the dossier.

## Room and regional contracts

Cedar Shelter is `MnR13Cave1F`, with native cave art/border block 1. Exterior
CAVE `(14,53)` in `MnR13C00R01` targets interior warp 1 at LADDER `(7,15)`;
clear interior floor is `(7,14)`. The room exit targets exterior passive warp 2
at `(14,54)`, avoiding immediate reentry. Harper stands `(13,5)`, approached
from `(13,6)`; native HealParty restores HP, fainted partners, status and PP.
Max Ether `(15,11)` retains `EVENT_MN_R13_SHELTER_ETHER`.

North Boundary Waters gate/landing metatiles are `(15,1)/(15,3)`; south Ely
is `(17,62)/(17,60)`. All eight badges and
`EVENT_MN_SURVEY_RESTORED` remain required. Q08 completion belongs in Boundary
Waters; this route never grants that flag. Real hub gates, recovery/Fly and the
earned connected journey remain integration work. The League building belongs
to Ely.

## Fast render loop and native publication

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_league_approach.py --serve
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_league_approach -- \
  content-packs/minnesota-northern-lakes.crystalpack \
  planning/minnesota/routes/MN_R13.layout.json \
  planning/minnesota/routes/MN_R13.json \
  content-packs/minnesota-league-approach.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_league_approach.py \
  --pack content-packs/minnesota-league-approach.crystalpack --verify-published
```

Source checks/rendering take about 0.3 seconds warm; publication is separate.
The current pack contains **405 maps**: the completed 401-map Northern Lakes
base plus three exteriors and one shelter. Exact comparison passes all 2,048
exterior blocks, seventeen outdoor objects, six signs, teams/text and room
rows/warps. The composition regression uses stable River Bluffs/Pine Country
bases, preserving existing maps, files, art, audio, HOME and recovery records.
It rejects duplicate publication, missing survey restoration and a room exit
on a wall. All 377 audio PCM hashes, frame counts and loop ranges match.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-assets --test league_approach
CRYSTAL_LEAGUE_APPROACH_TEST_PACK="$PWD/content-packs/minnesota-league-approach.crystalpack" \
CRYSTAL_LEAGUE_APPROACH_FIXTURE_SAVE="$PWD/.local/league-revised-review.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib league_approach_controller_ -- --ignored
CRYSTAL_CUT_ROUTES_TEST_PACK="$PWD/content-packs/minnesota-league-approach.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib cut_gated_routes:: -- --ignored
CRYSTAL_LEAGUE_APPROACH_TEST_PACK="$PWD/content-packs/minnesota-league-approach.crystalpack" \
CRYSTAL_LEAGUE_APPROACH_FIXTURE_SAVE="$PWD/.local/league-revised-review.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p geothite --test league_approach -- --ignored
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-league-approach.crystalpack --audio --desktop
```

All listed checks passed for the current pack identity. Three route controller
fixtures cover geometry/resources/room returns, damaged and fainted partner
healing, and Avrin's actual battle with selected PP spent, victory and persisted
party/defeat flag. Seven shared Cut/Surf fixtures pass, including this route's
new Cut case and the earlier three route gates/water boundaries. Empty-party
geometry and explicit level 60 Mew/Lapras/badges are fixture setup; they do not
prove earned progression, full populated traversal or route balance.

The stdio MCP check continues at `MnR13C00R00 (30,8)`, moves to y9, opens
production Start, closes it and requires byte-for-byte empty stderr. The actual
production text client also loaded the fresh save, opened Start and the Pack
with collected items, returned to walking and exited successfully:

```sh
target/runtime-fps/wasm-dev/geothite play \
  content-packs/minnesota-league-approach.crystalpack \
  --load .local/league-revised-review.crystalsave --compact
```

Headless native noon captures of [Wind Tor](../renders/league-approach/native-tor-revised.png),
[the lake view](../renders/league-approach/native-lake-revised.png),
[Dry Pass](../renders/league-approach/native-pass-revised.png),
[the quiet junction](../renders/league-approach/native-quiet-junction-revised.png)
and [the shelter](../renders/league-approach/native-shelter-revised.png) were
inspected by sight. No visible render window was opened. Final visual approval,
regional gates/progression and the full twenty-route objective remain open.
