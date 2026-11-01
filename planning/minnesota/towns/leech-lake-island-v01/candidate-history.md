# Leech Lake pine island

Implemented native candidate, not yet selected. The current state remains bound to
`planning/minnesota/world.json`; no older field-use/gallery pack is a parent.

The large sheltered lake gains an optional pine island. From the original dry
launch `(31,29)`, Surf east from `(32,29)` to the west landing `(45,30)` and
step onto `(46,30)`. The northern pine finger bends the approach south through
four grass tiles `(50..51,32..33)` to an eastern clearing. No other shore permits
an island landing. The public through road, two cabins, free care, testimony,
closed pier and actual R07/R08 connections remain intact and require no Surf.

Ellis, an optional naturalist at `(53,30)`, offers a native trainer battle from
`(52,30)`. Shedstag35 and Leechurge36 represent the forest/lake relationship;
existing Minnesota scaling/evolutions and learned moves apply. Declining costs
nothing. Native victory pays the ordinary prize and sets only
`EVENT_MN_EXTRA_LEECH_ISLAND_NATURALIST`. Afterward Ellis explains the low ledge.
The ball at `(53,33)` contains one Mystic Water, protected by
`EVENT_MN_LEECH_ISLAND_WATER`. A full pocket must leave it for retry.

The return drop has HOP_DOWN lips `(50..53,34)`, wall courses at y35 and dry
landings at y36. Permanent pines/headbutt canopy close both sides. Walk west to
x48 and north to the landing for a quiet dry return; ordinary uphill access
still uses the grass. These are low grass ledges, with no cliff underneath.

The separate eastern current is native metatile07 at `(30,10)`, runtime tile
`(60,20)`. Native cap36 at `(30,9)` closes a north interaction bypass. The hidden
PP Up rock `(61,20)` has one approach, the cleared current. Native Whirlpool
needs its move and seventh badge, replaces07 with36 and preserves all other
quadrants. Itemfinder locates the cache; `EVENT_MN_LEECH_EDDY_CACHE` prevents
repeat claims. Temporary current clearing resets on map load; the item flag
persists. A loaded save is not evidence of a live cleared-current render.

`after.json` includes the complete native terrain, trainer, encounters and hint.
`contract-v02.json` binds the complete original town and actual publication
parent. All source iterations and every render are retained. Source
v02 fixed the wrong sideways ledge art, added the northern pine finger and
closed the west-side bypass; its whole-map render was visually inspected.
Source v03 corrected the native single-quadrant canopy orientation, keeping the
western dry return open while blocking entry to the hop lip. Source v04 shortened
one fisher direction to fit Crystal's 18-column text window. The warm source
loop took 0.111 seconds after pack load. The native v02 candidate publishes and strictly reads back. The complete-parent
preservation audit passes: all 733 other maps, old native programs, original
trainers, public services/world ports, art and audio bytes remain exact. The
regional audit covers the touched town and all 322 Minnesota trainers against
225 active species. All seven camera views were visually inspected: six land
frames and the coherent water/Whirlpool/Surf frame each match 368,640 pixels.
The final gameplay and scoped checks are pending; this candidate is not selected.

The first gameplay run rejected a missing Water encounter table. Source v05
supplies complete regional grass/water fallback tables at zero rate, so the
native Surf resolver has a valid table while ordinary roads and water stay
quiet. Only the island grass zone has a nonzero rate. Its corrected candidate
is `content-packs/minnesota-leech-island-v02.crystalpack`, SHA256
`6493a487ecfe3032c15a69e8c734838fd13bf0d221cc8c7fb831533529a6f6e8`.
Do not select v01; its first gameplay run exposed the missing fallback.

Executed build and publication commands (choose a fresh output when repeating):

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin build_leech_lake_island
target/verdigrim-scene/debug/build_leech_lake_island \
  content-packs/minnesota-landmark-habitats-v02.crystalpack \
  f6498564aec5f6cec506e47329f2eb72d248fe73d97677a6f0b4af0e6272fcf8 \
  planning/minnesota/towns/leech-lake-island-v01/after.json \
  planning/minnesota/towns/leech-lake-island-v01/contract-v02.json \
  /Users/ryanculligan/.local/share/geothite/export-workspace/vendor/pokecrystal/engine/events/forced_movement.asm \
  content-packs/minnesota-leech-island-v02.crystalpack
```

The output must be fresh and ignored. The final source intentionally does not
reuse the historical combined field-use installer, which would replace the
accepted 12x10 Driftless fossil gallery. The pinned external native movement
source is read directly, never copied into tracked content.

Evidence retained under `planning/minnesota/renders/leech-lake-island-v820/`:
`preservation-v835.json`, `regional-audit-v836.json`, `native-v829/`,
`native-whirlpool-comparison-v832/`, `native-capture-inheritance-v836.json`.
The generic land comparator's initial water-view discrepancy is retained too:
it painted water over the actor and lacked the animated current. The focused
`compare_current.py` compares every pixel with the original Surf sprite and
one coherent native frame per animation, with no masks or fitted translation.
The v834 run passes original road/care/cabin returns and the initial badge
refusal. Three remaining cases exposed test pacing/current-physics assumptions:
a Surf step can still be settling, and attempting an uncleared current invokes
native forced movement back onto open water west of the current. The next replay uses bounded native
movement settling, confirms that pushback, returns to `(59,20)` through real
Surf input and then tests the badge/move gate. Its results belong under
`gameplay-v842/`. These pending cases prevent selection.

The complete installed town, with actual native NPC facings, is retained at
`../../renders/leech-lake-island-v820/installed-whole-v840/whole-original.png`.
All377 audio programs in the corrected v02 candidate pass PCM hashes, frame
counts and loop ranges (`.local/leech-island-audio-v838.log`).

Render a fresh version without rebuilding the game:

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-landmark-habitats-v02.crystalpack \
  --source planning/minnesota/towns/leech-lake-island-v01/after.json \
  --palette day --output planning/minnesota/renders/leech-island-next-source
```


The shared selection advanced to the verified Rochester fossil lab during
this review. `contract-v03.json` binds that newer parent. A fresh v03 island
candidate is being rebased onto it; no active pointer changes before final
proofs. The older v02 checkpoints remain evidence of their exact pack only.
The Rochester lab and the newer habitats must remain intact in any selection.


The native gallery helper intentionally plans only dry FLOOR tiles, so it
cannot plan the island's required grass approach. The island replay now has
its own read-only live collision planner including TALL_GRASS; it sends only
ordinary buttons and replans around visible actors. Current-entry tests check
four fully settled attempts and require Surf on the west side, never release
control inside the cache approach, and return through real Surf input before
using Whirlpool. No native current program or terrain was changed to fit a
particular test timing.


Five current-parent native controller cases pass on v03 (`gameplay-v851/`):
Surf/required habitat/actual naturalist victory and PP/prize, held Mystic Water,
Continue, useful native drop and reverse/side denial, mainland return; native
Whirlpool teaching/use/PP Up/Itemfinder/Continue; seventh-badge denial; full
pocket/native TOSS retry; and original free full-party care plus all cabin
carpets/pier/public return. The naturalist uses actual native level scaling.
The separate production stdio trip passes360RPC calls with byte-empty stderr,
including a declined naturalist, reward/drop/quiet return, native Surf launches,
Start and a fresh-process Continue (`mcp-v849/`). These are explicitly local
party/badge/position fixtures, not earned statewide progression.

The publisher is gaining a guarded inherited-habitat path so the new regional
lake tables can compose with the island zone. It preserves an existing lake's
rates/tables/zones verbatim and appends only the four-tile island grass zone;
a fixture verifies the original lake remains identical after removing that new
zone. Existing or overlapping island zones are rejected. The current town
preimage, regional species/evolution policy and whole-parent audits still apply.
