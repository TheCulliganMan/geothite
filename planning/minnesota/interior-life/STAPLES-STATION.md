# Staples signal room and parcel supplies

The current combined 730-map pack is
`content-packs/minnesota-interiors-selected-r12-v01.crystalpack`; see the
[current composition](selected-union-v01/README.md). All three Staples controller
cases pass there (69.03s). Its signal ledger is correctly bound to the wall at
`(4,10)`, read from `(4,11)`: four repeated upward inputs stay blocked, two
successive readings work, and input returns normally. The older table below
records the earlier ledger position. Passenger rides remain unfinished.

Publication lineage correction: the pack receipts below are retained history.
Current source refuses their obsolete coastal geometry; preserve the rooms and
[rebase onto the compact coast](COMPACT-REBASE.md) before further publication.

Staples now has a real native 2D station on a woodland spur off Central Lakes
West, between Detroit Lakes and Brainerd. The west side of its waiting hall is
a wooden signal room; the east side has native seats, a planter and the guarded
platform entrance. Five people give the building and its approach a purpose:

| Person | Purpose |
| --- | --- |
| Junction walker | Explains Staples' rail identity and gives directions to the existing regional road. |
| Signal keeper | Remembers his father's night shifts and still leaves a light for people walking home. |
| Parcel clerk | Sells REPEL, SUPER POTION, PARLYZ HEAL and FLOWER MAIL at their existing native prices. |
| Waiting student | Tells how a mailed seed packet became a family correspondence about a flower. |
| Conductor | Explains that passenger service is unfinished and physically closes the platform. |

The readable signal ledger says no freight is worth a life and to wait for
daylight. The first actual hall entry records only Staples' visited flag.
The exterior sign, outdoor arrival and walker cannot unlock a train destination.
This publication installs the public hall and shop; **passenger rides remain
unfinished** under the existing [five-stop contract](../transit/glacier-line.json).
The remaining station callbacks, all twenty guarded ride branches,
canonical coast/Cedar and separate Ely geography composition, and regional
native 2D room-quality acceptance remain required by the full interior objective.

## Source and physical binding

[MnStaplesStation.json](../transit/MnStaplesStation.json) is the editable source.
It adds `MnStaplesRailJunction` (16×10 metatiles) and `MnStaplesStation` (9×9)
over the exact 717-map Split Rock interior parent. The complete six-block
station facade comes from Goldenrod's native station. The train keeps its
original nose/tail assemblies and two native doors. The outer freight siding
uses the raised, fully blocking native track rather than walkable track art.
There are no incidental Cut gates or outdoor water decorations.

The new junction occupies `(10,-10)..(26,0)` in metatiles relative to
`MnR15C00R00`. It fits the actual gap south of Itasca without overlapping any
existing connected-map rectangle. Nine original forest cells form the spur:
four native boulder caps, one northern mouth and four path cells leading to
the existing encounter grass. Every original grass, water, cliff, gate, actor,
item, warp and program remains exact. The route's trainer and encounter species
are rebound separately to active Minnesota originals, as required by the current
repository rules. A dry-component comparison rejects any
new bypass between the old grass-separated areas.

| Binding | Native gameplay coordinates |
| --- | --- |
| R15 north, offset 10 | `(26,0)/(27,0)` to junction `(6,19)/(7,19)` |
| Junction south, offset -10 | The exact reciprocal lanes above |
| Real facade entry, exterior warp 1 | `(13,7)` to hall passive warp 3 at `(9,15)` |
| Safe exterior return, warp 2 | `(13,8)` |
| Both hall carpets, warps 1 and 2 | `(8,17)/(9,17)` to exterior warp 2 |
| Signal keeper and approach | `(3,11)` from `(3,12)` |
| Signal ledger and approach | `(4,11)` from `(4,12)` |
| Parcel clerk and approach | `(5,13)` from `(5,14)` |
| Student and approach | `(13,14)` from `(12,14)` |
| Conductor and approach | `(11,9)` from `(11,10)` |
| Protected future ride arrival | `(11,11)`, native FLOOR |

Both seam directions use native connection projection. The outpost's complete
outer boundary is WALL except those two southern lanes. With the conductor
present, neither train door nor the platform is publicly reachable; removing
only that body exposes the intended native boarding path. There is no active
train-door warp or coordinate boarding trigger. No recovery, HOME or Fly point
is added. Existing Brainerd and other regional gates are preserved, and the
outpost remains a single walking leaf.

The final v02 fixes legacy species found during the touched-route audit. The
camper now uses BRISTUB and CLINKLAW with legal native moves at the original
levels; the already regional fisher remains exact. Grass and water retain their
levels, rates, slots and time periods. Two private fishing groups, four private
time tables and one local swarm binding preserve all original bite/slot odds,
rod levels and daily swarm behavior. Every original shared Johto/Kanto fishing
table remains unchanged. There are no legacy creature names in the route's
visible text. This is a local R15 correction, not a claim that the entire older
Minnesota branch has passed the current roster audit. The superseded v01 pack
and all of its renders/tests remain retained.

The older connected component already has four cycle conflicts involving
Bemidji/R14 and Grand Marais/Cedar. This leaf adds no cycle and does not claim
to repair that wider geometry. The exact gap and baseline-conflict receipt is
retained with the native review.

## Executed publication

Rust 1.99.0, from this checkout:

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
cargo build --locked --profile wasm-dev -p crystal-assets --bin pack_staples_station \
  --config 'profile.wasm-dev.opt-level=1' \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0'
target/interior-life/wasm-dev/pack_staples_station \
  content-packs/minnesota-interiors-split-rock-v01.crystalpack \
  planning/minnesota/transit/MnStaplesStation.json \
  content-packs/minnesota-interiors-staples-v02.crystalpack
```

The publisher requires a fresh output path and strictly rereads the written
pack. A fresh CLI replay of final v02 is byte-identical; the temporary replay
pack was removed after hashing. Regeneration must choose another fresh path.
Packs remain ignored.

| Artifact | SHA-256 / identity |
| --- | --- |
| Parent file | `978ab6e3f674acf72967ca2f6dba799d590b47f34ef855ea4cfb8cfa7c9c5397` |
| Parent runtime content | `3430117b94738bed99ecc020e326d9277d240b7ebd96fe7802212c6c00b21a39` |
| Child file | `f8d09d84f1d16d34c4ba84373d4b35e3e3eb1e7844212a9f56062aee79db1ff0` |
| Child runtime content | `a52621bc526f49ea9fafe2c8acd2970a05e34535cbb565dbbf9ecc692ab7dacf` |

The child has 719 maps, 475 species, 270 moves, 377 unchanged audio programs
and 217 installed non-gym interiors. The registry now has 224 entries including
eight gyms; its 216 non-gym rooms are all installed. The additional Rainy River
warming hut brings the installed scope to 217. The developer gym-review fixture
is excluded. Red Wing's station has been withdrawn by user direction; counts do
not establish regional quality acceptance. Use a fresh save for this pack identity.

## Verification

The written child equals a deterministic Rust rebuild. The preservation test
checks every original map, all old native programs/NPC tables and unrelated
blocks, source art, audio, recovery/HOME, unrelated catalogs, encounters, trainers,
story and numeric constants. The old map changes only at the nine forest cells,
new northern connection and local fishing-group attribute. Separate assertions
cover the exact R15 roster corrections while preserving every other trainer and
encounter table and every shared source fishing record.
Thirteen unsafe-source variants and duplicate installation
are rejected. The first test exposed an isolated open outer row that the
reachable-only boundary check did not reject; the validator now checks the
entire outpost perimeter. A later test fixture targeted an absent optional JSON
field and was corrected to mutate the existing clerk service. Failed logs are
retained. The first roster test incorrectly expected two new timed-fish tables;
the source swarm also uses two different timed tables, so the check now derives
the exact expected set from both original groups. The final case passes in
11.61 seconds; results are retained in `staples-assets-v06.log`.

All three production-controller cases pass in 10.62 seconds. They walk from an
unchanged road tile through the original grass, cross both seam lanes in both
directions, enter the actual facade, read the sign and ledger, hear every
ordinary resident, prove the conductor blocks boarding, exercise both carpets
and Start, and preserve the visit through Continue. The shop case proves exact
money/item mutation, insufficient-funds denial and recovery of movement/Start.
The additional case produces a real regional wild battle from that original
grass and executes native RUN. Only the station's visit flag changes; no badge,
courier or other station event is granted. Initial position, empty party for
the room course, a VOYAGEAR partner for the encounter case, and shop money are explicit fixtures,
not proof of earned regional progression.

```sh
export RUST_MIN_STACK=67108864
export CRYSTAL_STAPLES_BASE="$PWD/content-packs/minnesota-interiors-split-rock-v01.crystalpack"
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-interiors-staples-v02.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-assets --lib staples_station \
  --config 'profile.wasm-dev.opt-level=1' \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  -- --ignored --test-threads=1
cargo test --locked --profile wasm-dev -p crystal-bevy --lib staples_station \
  --config 'profile.wasm-dev.opt-level=1' \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  -- --ignored --test-threads=1
```

All seven complete headless 640×576 native 2D frames match their independent
source references with zero differences across 2,580,480 pixels. No pixels are
excluded. The hall, signal area, kiosk, conductor, facade, seam and grass approach,
whole station/outpost and actual connected branch were personally inspected.
The existing production capture example is retained by hash in the receipt;
its private copy was removed and the pack remained unchanged. No voxel view or
visible window was used. All source/rejected/native frames, raw GPU originals,
references and review programs remain under ignored
`renders/interior-life/staples/`, with the accepted review in `native-2d-v02/`.
All 377 audio PCM hashes, frame counts and loop ranges pass.

Changed Rust leaves pass formatting. The required workspace format check still
fails first in unrelated `offset-allocator/src/lib.rs:408`. Normal all-target
Clippy for both affected crates stops at the pre-existing `bad_bit_mask` denial
in `crystal-core/src/multiplayer/mod.rs:1332`; the separately recorded pass with
only the existing `bad_bit_mask` and `never_loop` denials allowed is not an
unqualified workspace lint pass. That final all-target run completes in
59.08 seconds with no diagnostics in the Staples leaves. The whole planning
checker still stops at the existing Minneapolis mismatch in
`check_city_recomposition_plan.py:212` (`recomposition changed one of44 original
source rooms`). The focused Staples source/registry/actors/world/transit check
passes independently. Logs are `target/interior-life/staples-*`.
