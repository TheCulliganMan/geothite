# Leech Lake mainland pines

Verified local native 2D revision. `planning/minnesota/world.json` is the active
selection authority. The reviewed pack is
`content-packs/minnesota-leech-mainland-v01.crystalpack`, SHA256
`bb416c46842b7a32bb213ed2a8e9c5eebdf384150c7dfab4ef8a341458833a87`.
Its parent is the newer water/cave habitat pack, preserving the lake's rate 5
Surf table, island grass and all other regional additions.

[Complete installed town](../../renders/leech-mainland-v871/whole-final-v893/whole-original.png)
· [Eight native player views](../../renders/leech-mainland-v871/native-v885/production-decisions.png)
· [Actionable plan](plan.json) · [Composed sources](sources.json)
· [Acceptance](../../renders/leech-mainland-v871/acceptance-v894.json)

## Walking and activities

Connected pine groves replace broad empty lawns around the original care and
fisher cabins. Their safe public-road connection remains54 steps. The winding
woodland link takes 34 steps, crossing 10 encounter-grass tiles: save 20 steps
through grass or take the longer safe road. The trail and side clearing need
no Surf or other field move. Its native rate 10 habitat includes Patchebee,
Skeeterex, Hatchrush, Velvawn, Shedstag and Mothosaic at levels 30–35. Existing
lake, island, fishing and all other habitat tables are retained exactly.

Finn ties fishing flies in the clearing at `(19,24)`, approached from `(18,24)`.
He offers an optional native battle with Hatchrush 34 and Mothosaic 35, using
existing Minnesota level/evolution scaling and legal learned moves. Declining
costs nothing. Victory pays the native prize and sets
`EVENT_MN_EXTRA_LEECH_MAINLAND_FLY_TIER`. The ball `(19,26)`, approached from
`(18,26)`, gives one Super Repel and sets `EVENT_MN_LEECH_MAINLAND_REPEL`.
A full pocket leaves it for retry. Native use consumes it and activates 200
Repel steps. The walker keeps the original public-shore testimony and appends
short directions to the cabin trail.

The public route still enters south at `(32/33,47)` from `MnR07C00R00`
`(20/21,0)`, offset+6. North `(32/33,0)` connects to `MnR08C00R01`
`(26/27,47)`, offset+3, retaining the original Mineral Badge wardens.
The care door/porch `(11,13)/(11,14)` and fisher door/porch `(9,35)/(9,36)`
retain both native room returns. Free full-party HP/PP/status care, PC/recovery,
catch testimony, the pier's passing lane `(35,22)`, closed end x41 and separate
Surf launch `(31,29)` → `(32,29)` stay intact. `plan.json` records all arrivals,
carpets, actors, dialogue, rewards, habitat, source hashes and exact paths.

The [inherited island](../leech-lake-island-v01/README.md), naturalist, Mystic
Water, one-way dry return and separate Whirlpool cache remain unchanged.
There are no new Cut gates, cliffs, stairs, rooms or unconnected water exits.
All 734 other complete maps and original programs/art/audio remain exact to
the water/cave parent, including its newer Moorhead fishing access.

## Fast authoring loop

`after.json`, identical to `after-v04.json`, is the complete native terrain and
additive activity source. Its final edit only replaces the editorial pending
status; a fresh Rust replay produces byte-identical tested pack output.
`before.json` preserves the original terrain; `contract-v02.json` binds the
complete native town and actual water/cave parent. `contract.json` is its
current alias, while `contract-v01.json` retains the earlier island parent.

Use a fresh audit output when repeating:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-water-caves-v05.crystalpack \
  --source planning/minnesota/towns/leech-lake-mainland-v01/after.json \
  --palette day --output planning/minnesota/renders/leech-mainland-next-source
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/towns/leech-lake-mainland-v01/source_audit.py \
  --pack content-packs/minnesota-water-caves-v05.crystalpack \
  --source planning/minnesota/towns/leech-lake-mainland-v01/after.json \
  --output planning/minnesota/renders/leech-mainland-next-source-collision.json
```

The measured warm render took 0.157 seconds after pack load. The final source
matches all 786,432 pixels of the installed town raster, including actor facing.
The complete town, seven before/after camera comparisons, eight actual native
captures and unmodified Johto Lake of Rage reference were visually inspected.
The review caught an inaccessible strip behind the care cabin; it is now
continuous native pines. Every floor/grass tile has its intended mainland or
Surf access. No incidental Cut quadrant or open water boundary was introduced.

## Native publisher and executed checks

The Rust CLI is `rebuild_leech_mainland`. Build through the shared Cargo lock:

```sh
python3 - <<'BUILD'
import fcntl, os, subprocess
with open('.local/minnesota-cargo.slot.lock', 'a') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX)
    env = dict(os.environ, CARGO_TARGET_DIR='target/verdigrim-scene',
               CARGO_BUILD_JOBS='2', RUST_MIN_STACK='67108864')
    subprocess.run(['cargo', 'build', '--locked', '-p', 'crystal-assets',
                    '--bin', 'rebuild_leech_mainland'], env=env, check=True)
BUILD
```

The completed source replay used the frozen Rust executable below. Its output
already exists; choose a new ignored name to repeat publication:

```sh
.local/leech-mainland-publisher-v883 \
  content-packs/minnesota-water-caves-v05.crystalpack \
  20fcdad6de43bb2e46a41faa49d7a8916cfe5bd2224680709f135166d3d2801b \
  planning/minnesota/towns/leech-lake-mainland-v01/after.json \
  planning/minnesota/towns/leech-lake-mainland-v01/contract-v02.json \
  content-packs/minnesota-leech-mainland-replay-v02.crystalpack
```

The publisher checks the parent SHA before/after, contract binding, complete
native preimage, protected original anchors, priced grass shortcut, reachable
floor/grass, regional wildlife and strict pack readback. It preserves inherited
lake rates/tables/zones and appends only `MN_LEECH_CABIN_PINES`. The ignored
current-parent unit case passes positive publication and rejection of dry
bypasses, stranded ground, shore edits, stale preimages and foreign new or
inherited wildlife. External fixtures are read at runtime.

Eight default-profile native controller cases pass on the reviewed pack:

```sh
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-leech-mainland-v01.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-leech-mainland-v01.crystalpack" \
CRYSTAL_LEECH_ISLAND_REVIEW_DIR="$PWD/planning/minnesota/renders/leech-mainland-v871/gameplay-v886" \
RUST_MIN_STACK=67108864 .local/leech-mainland-controller-v884 \
  minnesota_field_use::leech_island:: --ignored --nocapture --test-threads=1
```

Use a fresh review directory; every reached save is retained. The shared fixture
loader specifically requires `CRYSTAL_WINONA_TEST_PACK`. The cases prove no-Surf
trail travel and real wild encounters, declined/actual fly-tier victory with
PP/prize, item/Continue/native 200-step use, full-pocket/native TOSS retry, plus
all five inherited island/Whirlpool/ledge/reward/care/cabin cases. Only initial
party/badge/position/capacity setup is seeded; subsequent actions are production
input. These fixtures do not prove earned statewide progression.

Separate actual stdio journeys pass 818 mainland and 384 island RPC calls, both
with byte-empty stderr, real encounters, native RUN, rewards, Start and a fresh
process Continue. The corrected checker recognizes native `wildbattle` status.
The executed mainland command was:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/towns/leech-lake-mainland-v01/check_mcp.py \
  --pack content-packs/minnesota-leech-mainland-v01.crystalpack \
  --save planning/minnesota/renders/leech-mainland-v871/gameplay-v886/mainland-no-surf-initial.crystalsave \
  --client .local/international-falls-client-v712 \
  --output planning/minnesota/renders/leech-mainland-v871/mcp-v889
```

Choose a fresh output directory to repeat. The independent `audit.py` preserves
all 734 other complete maps and original catalogs/programs/services/assets,
with 104 mainland metatiles changed. The regional audit covers 347 Minnesota maps,
4,622 species references, 323 trainers and 225 active species. All 377 audio
programs pass PCM hash, frame-count and loop-range verification. Each of eight
GPU views matches 368,640 source pixels; captures are headless and retain raw
readback descriptors. `views.json` records their exact anchors.

Both affected crates pass all-target Clippy, including Bevy's `location-tester`
feature, with no introduced warnings. Owned Rust formatting passes. The full
workspace format check still reports unrelated differences beginning at
`crates/offset-allocator/src/lib.rs:408`.

The [acceptance receipt](../../renders/leech-mainland-v871/acceptance-v894.json)
binds the exact pack, final source/plan/index and proofs. All earlier source
iterations, rejected previews, failed checks, original renders, crops,
conversions and reached saves remain retained. Packs and verification output
stay ignored. Older lawn-position saves are not migrated; retain their matching
prior pack. No deployment or earned campaign completion is claimed. Broader
state map and cave quality work remains active.
