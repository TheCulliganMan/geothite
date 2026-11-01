# Iron Range rock and North Shore river revision

The retained geography milestone pack is
`content-packs/minnesota-geography-review.crystalpack`: **471 native maps**,
content identity
`f9b04e67f2bfa93d915dae15ffb005d732ccee2df58b21d2f6ca550206a21949`,
file SHA-256
`4874055e9328d875700fbd5008c5fd46c9b4525625511ab5a0fafc443fb08a56`.
The earlier `minnesota-quality-review.crystalpack` remains unchanged as the
reproducible base (`5fe3071c6242…`, actual file `88fa02bafc8e…`). Packs and renders
remain ignored local deliverables. The subsequent [prairie art pass](prairie-grass.md)
layers yellow R02/R16 grass on this exact base.

## What changed by sight

- **R09 Iron Range West:** stepped rock ribs and attached ore groups shape the
  quarry, mine approach and rail cutting. A narrow stone haul track and rubble
  aprons make the rock visible from the walking path. Earlier broad bare crowns
  were rejected and retained. [Whole route](../renders/iron-range/whole-route-0a507546.png).
- **R10 Iron Range Descent:** rock faces now enclose the former repetitive tan
  carrier bend; exposed native rock and gravel distinguish the haul road from
  its shoulders. The grass neck and harbor approach have joined rock boundaries.
  [Whole route](../renders/iron-range-descent/whole-route-2c234acd.png) and
  [same-position before/after views](../renders/iron-range-descent/rocky-899990976317/before-after-walking-sheet.png).
- **R11 North Shore:** two additional forest drainages reach Superior through
  actual falls, plunge pools, river mouths and required dry bridges. The northern
  headwater is an offset pine stream; the southern ravine has two falls separated
  by ordinary Water. The first matching rectangular rock basins were rejected.
  [Whole route](../renders/north-shore/whole-route-01d36794.png) and
  [same-position before/after views](../renders/north-shore/rivers-review/before-after-01d36794.png).

Root inspected all three complete maps, their weak walking cameras and four
fresh production captures at noon: [R09 upper bench](../renders/geography-review/native-r09-north.png),
[R10 carrier bend](../renders/geography-review/native-r10-carrier.png),
[northern fall](../renders/geography-review/native-r11-north-cascade.png), and
[southern cascades](../renders/geography-review/native-r11-south-cascade.png).
These captures use the actual new pack, headless native 2D, 640×576. Encoded
originals, original readback bytes/descriptors and every transformation remain
in the render archive. No final user visual acceptance is implied.

## Exact preservation

The revisions change 251/161/203 metatiles respectively. Only nine existing
`maps[*].blocks` arrays, their nine matching base64 `map_blocks` values and the
derived content identity differ. All other pack fields and scalar types remain
exact: the other462 maps, map attributes, connections, raw/compiled scripts,
actors, trainer parties, items, rooms, encounters, source art/files, audio,
HOME/recovery, manifests and report. No city, train service or fairground was
silently added by a terrain revision.

The read-only proof checks the exact serialized native identity tuple, actual
file hashes, all source/native blocks, original rail/Waterfall recipes and
source/native whole-map pixel equality. Nine meaningful corruptions reject
stale terrain, an unrelated map edit, altered guard/party/HOME/audio/report,
mismatched base64 and wrong source. See `.local/geography-exact-preservation-complete.log`
and the immutable comparison reports under
`renders/terrain-revisions/5fe3071c6242-to-f9b04e67f2bf/`.
Nine exact before/after/dossier source copies are protected in its `sources/`
directory, allowing replay after live authoring advances.

The source proof permits changed status prose and appended bridge annotations
only: old crossing records stay exact and each new annotation must identify a
newly placed native06 deck. Every other non-layer source field remains exact.
Native imported rail recipes retain their real source collision when no
override is authored. This does not relax the all-other-pack-fields comparison.

R10's first source syntax used an unsupported material alias. It was converted
to seven explicit native71 block rectangles before publication; all960 raster
blocks and full-resolution pixels remained identical. The rejected syntax,
normalization proof and original renders remain retained.

## Production input verified

**Thirteen targeted tests pass** on the new pack:

- R09 geometry/rooms/rails/ledge and its populated selected-PP battle, Surf,
  resource, rest and Continue journey; native Cut denial/opening/reward/reset.
- R10 geometry/cave/ledge, populated Calder victory with selected PP and Continue,
  actual Surf seam and visible outer-rock refusal.
- R11 exact native animation source, original bridges/rooms/events/seams,
  selected-PP Will victory and Continue, sea rock/Surf seams, and the original
  Waterfall summit dismount/relaunch/return.
- Both new bridges walked in both directions and both columns, with actual
  LADDER collision and accessible new bank/view paths.
- The new northern six-tile fall climbed through contextual A/confirmation from
  `(18,26)` to `(18,19)`; both southern stages climbed `(18,106)`→`(18,103)` and
  `(18,102)`→`(18,99)`. Native downward currents return to the original banks.

Logs: `.local/geography-iron-controller-corrected.log` (6 pass) and
`.local/geography-shore-controller-corrected.log` (7 pass).
These use explicit local position/party/badge/RNG fixtures; they do not earn
the prerequisites or prove connected regional progression. Additional new
upper-bank dismount/relaunch is source-checked, not asserted by the new climb test.
Native LADDER decks are not claimed to permit Surf underneath.

Two initial fixture defects were corrected rather than changing the runtime:
collision queries must read the currently loaded chunk, and contextual Waterfall
must face Up without an extra Up step entering the downward current. The new
helper asserts the actual base and Up facing before contextual A and retains
the real-action/ascent/return checks. R09's closed lower public seam check now
uses the public walking component: the new isolated crown at `(63,53)/(64,53)`
is native Floor scenery, unreachable even in the source's post-Cut/Surf graph.

Both new actual journey saves load through the production stdio MCP executable,
open native Start/PACK, cancel with B and move the authoritative position:
R09 local `(28,5)`→`(28,6)`, R11 `(42,13)`→`(42,14)`. Both have byte-empty stderr.
Evidence: `.local/geography-mcp.log` and full JSON observations. All377 audio
programs match PCM hashes, frame counts and loop ranges (`.local/geography-audio.log`).

Scoped rustfmt passes for both changed test leaves. Bevy all-target Clippy
completes with the three previously documented baseline lint allowances
(`never_loop`, `bad_bit_mask`, `approx_constant`); existing warnings remain,
with none referring to either changed test leaf (`.local/geography-bevy-clippy.log`).
Full-workspace formatting still reports pre-existing differences beginning in
`crates/offset-allocator/src/lib.rs:408`; the check does not modify those files
(`.local/geography-fmt-all.log`).

## Reproduce

Use the existing Rust `revise_authored_route` binary, beginning with the quality
pack, and apply R09, then R10, then R11. Supply the protected `*-before.layout.json`,
`*-after.layout.json` and `*.dossier.json` triples to each invocation. The verified
intermediate packs are retained under `.local/minnesota-geography-revision-stages/`.

```sh
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-geography-review.crystalpack --audio --desktop

export CRYSTAL_CUT_ROUTES_TEST_PACK="$PWD/content-packs/minnesota-geography-review.crystalpack"
export CRYSTAL_IRON_RANGE_TEST_PACK="$CRYSTAL_CUT_ROUTES_TEST_PACK"
export CRYSTAL_IRON_RANGE_DESCENT_TEST_PACK="$CRYSTAL_CUT_ROUTES_TEST_PACK"
export CRYSTAL_NORTH_SHORE_TEST_PACK="$CRYSTAL_CUT_ROUTES_TEST_PACK"
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib iron_range_ -- --ignored --test-threads=1
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib north_shore_ -- --ignored --test-threads=1
```

The [current connected atlas](../renders/atlas/minnesota-current-connected.png)
compares all twenty routes against this pack. It still has two native towns,
twenty-five built interiors and three solid guarded basecamp seams; all other
hub links remain planned. The [Glacier Line](../transit/README.md), three city
source assignments and [State Fairgrounds](../zones/state-fairgrounds.json)
are active authoring scope, not native content in this pack. The full visual and
earned regional journey goals remain open.
