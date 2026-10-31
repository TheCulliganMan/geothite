# Leech Lake pine island

Verified local native 2D revision. `planning/minnesota/world.json` is the authority
for active selection. The reviewed output is
`content-packs/minnesota-leech-island-v05.crystalpack`, SHA256
`0ee27b841a47715f62082533cd324ec8d39c1823d524417874931449f5eeda5b`.
It extends the Rochester fossil lab parent and preserves all 734 other complete
maps, original services, story, train rides, art and audio.

[Whole town](../../renders/leech-lake-island-v820/whole-current-v863/whole-original.png)
· [Native player views](../../renders/leech-lake-island-v820/final-native-v860/production-decisions.png)
· [Acceptance](../../renders/leech-lake-island-v820/acceptance-v866.json)
· [Actionable plan](plan.json) · [Composed sources](sources.json)

## Play and connections

The public road connects south to `MnR07C00R00`, offset+6, and north to
`MnR08C00R01`, offset+3. Both use the existing two-tile ports; the northern
Mineral Badge wardens remain unchanged. The care cabin, free full-party
HP/PP/status healing, fisher house, both return carpets, catch testimony and
closed pier remain intact. The public through road needs no Surf.
`plan.json` records every port, door, porch, arrival and carpet coordinate.

From the original shore launch `(31,29)`, Surf from `(32,29)` to the island's
only landing, water `(45,30)` → land `(46,30)`. A pine finger bends the approach
south through the four grass tiles `(50..51,32..33)`. Ellis at `(53,30)` offers
an optional native battle with Shedstag 35 and Leechurge 36, using the existing
Minnesota level/evolution scaling. Declining costs nothing. Victory pays the
native prize and sets `EVENT_MN_EXTRA_LEECH_ISLAND_NATURALIST`.

The item ball `(53,33)` gives one Mystic Water and sets
`EVENT_MN_LEECH_ISLAND_WATER`; a full pocket leaves it available for retry.
The downward ledge lips `(50..53,34)` land at y36. Pines close reverse and side
bypasses; the permanent western canopy leaves the ordinary return at x48 open.
There is no cliff under this low ledge and no incidental Cut gate.
From reward approach `(52,33)` to landing `(46,30)`, the ordinary path takes 9
navigation actions and crosses 2 grass tiles. The dry drop takes 14 actions and
crosses no grass: it trades distance for avoiding encounters.

The separate eastern Whirlpool at `(60,20)` seals a hidden PP Up rock `(61,20)`.
Its cap closes the north approach. Native Surf, Whirlpool and the seventh badge
are required. Itemfinder finds the cache; `EVENT_MN_LEECH_EDDY_CACHE` prevents
repeat claims. Clearing changes native metatile 07 to 36 and keeps buoy collision.
The current resets on native map load; the item claim persists. The live
[cleared-current capture](../../renders/leech-lake-island-v820/final-native-current-cleared-live-v860.png)
comes from actual graphical keyboard input, rather than loading a cleared save.

Only the island grass zone has new encounters: rate 10 with regional Leechimp,
Leechurge, Yodeloon, Velvawn and Shedstag. Existing fishing stays unchanged.
When rebasing, the publisher preserves any existing town encounter tables,
rates and zones and appends only this island zone. The current parent has no
lake table, so complete zero-rate fallback tables support the native resolver.
A later lake habitat publisher must retain this island zone when adding Surf
encounters. All reachable unconnected water edges stay visibly closed.

## Authoring and fast review

`after.json` is the complete final native terrain, trainer, encounter and hint
source, identical to `after-v07.json`. `contract-v05.json` binds the fossil lab
parent and complete original native town module. `before.json` retains that
terrain preimage. `plan.json` records the story flags, actors, rewards and exact
collision contracts. `sources.json` preserves the inherited source index plus
landmark habitats and the Rochester lab overlays.

The source loop needs no Rust rebuild. Use a fresh render output each time:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-fossil-lab-v01.crystalpack \
  --source planning/minnesota/towns/leech-lake-island-v01/after.json \
  --palette day --output planning/minnesota/renders/leech-island-next-source
```

The measured warm loop was 0.251 seconds after pack load. The complete final
1024×768 source preview matches all 786,432 installed raster pixels, including
NPC facing. Native Johto Route44 was visually inspected as the Surf-islet style
reference. Original native pixels and palettes remain authoritative.

## Publication and checks

The executed Rust CLI is `build_leech_lake_island`. Build through the shared
Cargo lock; keep the repository's two-job default:

```sh
python3 - <<'BUILD'
import fcntl, os, subprocess
with open('.local/minnesota-cargo.slot.lock', 'a') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX)
    env = dict(os.environ, CARGO_TARGET_DIR='target/verdigrim-scene',
               CARGO_BUILD_JOBS='2', RUST_MIN_STACK='67108864')
    subprocess.run(['cargo', 'build', '--locked', '-p', 'crystal-assets',
                    '--bin', 'build_leech_lake_island'], env=env, check=True)
BUILD
```

The final candidate was published with these arguments, using the frozen copy
of that Rust executable at `.local/leech-island-publisher-v852`:

```sh
.local/leech-island-publisher-v852 \
  content-packs/minnesota-fossil-lab-v01.crystalpack \
  4329e491655cd8d6c787a34266a5fa51e222b8cdbd596e56c3b046219c5df5db \
  planning/minnesota/towns/leech-lake-island-v01/after.json \
  planning/minnesota/towns/leech-lake-island-v01/contract-v05.json \
  /Users/ryanculligan/.local/share/geothite/export-workspace/vendor/pokecrystal/engine/events/forced_movement.asm \
  content-packs/minnesota-leech-island-v05.crystalpack
```

The output now exists; choose a fresh ignored name to repeat publication. Read
the latest world selection and rebase the contract before publishing onto a
newer parent. The publisher checks the parent hash before/after, native source
preimage, active roster, required grass, ledge barriers, cache cap and strict
pack readback. It imports only pinned external native forced-movement routines;
it does not invoke the historical whole field-use installer, which would
replace the accepted Driftless gallery.

Five native controller cases passed on the final pack. The exact completed
replay used the frozen default-profile test executable:

```sh
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-leech-island-v05.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-leech-island-v05.crystalpack" \
CRYSTAL_LEECH_ISLAND_REVIEW_DIR="$PWD/planning/minnesota/renders/leech-lake-island-v820/gameplay-v863" \
RUST_MIN_STACK=67108864 .local/leech-island-controller-v856 \
  leech_island_controller_ --ignored --nocapture --test-threads=1
```

Choose a fresh review directory when repeating; reached saves are immutable.
The five cases verify actual optional victory/PP/prize, encounters, held Mystic
Water and Continue, drop/reverse/side collision, native Whirlpool teaching and
cache/Itemfinder/Continue, badge denial, full-pocket/TOSS retry, and all original
care/cabin/public-road returns. The source snapshot is beside the controller log.

A separate production stdio trip passed 360 RPC calls, empty stderr, native Surf,
required grass, optional trainer decline, item collection, dry ledge return,
Start and a fresh-process Continue. Its executed command was:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/towns/leech-lake-island-v01/check_mcp.py \
  --pack content-packs/minnesota-leech-island-v05.crystalpack \
  --save planning/minnesota/renders/leech-lake-island-v820/gameplay-v863/journey-initial.crystalsave \
  --client .local/international-falls-client-v712 \
  --output planning/minnesota/renders/leech-lake-island-v820/mcp-current-v864
```

All replay directories already exist; use fresh names to repeat. Initial local
party/badge/position/capacity fixtures are declared setup. Subsequent actions use
production input; these results do not prove earned statewide progression.

The independent `audit.py` compares the full parent with the child. The final
receipt preserves 734 complete maps and all original assets/programs/services,
with 58 changed town metatiles. The regional audit checks the touched town and
all 322 Minnesota trainers against 225 active original species. All 377 audio
programs match PCM hashes, frame counts and loop ranges.

Seven native camera views each match 368,640 pixels. The final text-only edit
changes Ellis's “quick way back” to “dry way back”; the visual binding proves
all captured scene inputs equal to the final pack. The six land views, Surf
current view, live current clearing and complete town were visually inspected.
Both affected crates pass all-target Clippy checks, with unrelated existing
warnings. Owned Rust formatting passes. Whole-workspace formatting still has
unrelated differences, beginning `crates/offset-allocator/src/lib.rs:408`.

The final [acceptance receipt](../../renders/leech-lake-island-v820/acceptance-v866.json)
links the exact proofs. All original/rejected renders, crops, conversions, saves
and review iterations remain retained; [candidate history](candidate-history.md)
records earlier failures and superseded pack names. Packs/build products stay
ignored. No browser deployment or earned campaign completion is claimed. The
broader map and cave quality review remains active.
