# R05 and R08 terrain quality revisions

This preserved milestone is the base for the newer
[Iron Range/North Shore revision](geography-revisions.md). The current connected
atlas uses that newer471-map geography pack; the quality pack below remains
unchanged for replay.

The ignored `content-packs/minnesota-quality-review.crystalpack` contains 471
maps, content hash
`5fe3071c6242e932096912bb3195982ff5ce7e5bd4f821a94985e9d652e50424`.
It retains the connected basecamp and revises two selected visual weak spots:
R05's hand-authored cliff cleft changes 13 metatiles; R08's connected pine finger
changes 22. The original connected pack remains at
`content-packs/minnesota-boundary-basecamp-connected.crystalpack`, hash
`9cb8936dbd1afeeae3abe69e237aeddf12d5c8bccb3a8eb9f8b7ed53bdb45841`.

The native/source whole maps and walking-camera pairs were inspected. The cleft
puts a real bank recess beside R05's ascent instead of an uninterrupted rock
slab. R08's pine finger brings a coherent forest boundary into the main turn
and both shore junctions. No new prop, reward, actor or permission gate was added.
These are bounded improvements, not final approval of either entire route.

## Retained visual evidence

| Route | Before / current whole | Paired camera review |
| --- | --- | --- |
| R05 River Bluffs | [Actual baseline native whole](../renders/terrain-revisions/9cb8936dbd1a-to-5fe3071c6242/mn_r05-before-native-whole-original-rgba-a2d557662ec5.png) / [current whole `a988740b`](../renders/river-bluffs/whole-route-a988740b.png) | [Cleft before/after](../renders/river-bluffs/cleft-review/before-after-a988740b.png), [native/source `(24,21)` pair](../renders/terrain-revisions/9cb8936dbd1a-to-5fe3071c6242/mn_r05-current_walk-before-after-framed-original-5c6898e26064.png) |
| R08 Northern Lakes | [Actual baseline native whole](../renders/terrain-revisions/9cb8936dbd1a-to-5fe3071c6242/mn_r08-before-native-whole-original-rgba-35495a6c115a.png) / [current whole `ab83cf76`](../renders/northern-lakes/whole-route-ab83cf76.png) | [Pine finger before/after](../renders/northern-lakes/pine-peninsula-review/before-after-ab83cf76.png), [native/source main `(49,47)` pair](../renders/terrain-revisions/9cb8936dbd1a-to-5fe3071c6242/mn_r08-main-before-after-framed-original-6dc958f0615a.png) |

Five actual headless production 2D captures cover the R05 ascent and R08 main,
north, south and east-return positions. The [native camera panel](../renders/terrain-revisions/engine-review/native-camera-review.png)
contains their derived 160×144 frames and was inspected by Root and this author.
The full-resolution captures, raw source/native scenes, preconversion images,
RGB conversions, crops, previews and rejected passes remain archived for reuse.
The strict comparison retains 94 named render artifacts plus intermediate history.

## Executed verification

- Rust publication strictly reads back each complete written CBOR pack.
- The read-only [revision checker](../tools/review_authored_terrain_revision.py)
  verifies both actual file hashes/native identities and all 3,712 selected-route
  blocks against the previous/revised source. Only four existing native
  `maps[*].blocks` fields, their four matching uniquely owned base64 `map_blocks`
  entries and `identity.content_hash` change. Every other field and scalar type
  remains exact, including unrelated maps, scripts, attributes, connections,
  events, encounters, parties, art/files, audio, HOME/recovery and report/manifests.
- Nine adversarial mutations reject: stale old block, unrelated map, guard,
  party, HOME position, audio payload hash, encoded blocks, report and new source.
  Native/source glyph recipes and whole terrain pixels match on both sides.
- R05: **three production-controller cases pass**—three-map geometry, ramps,
  drops and both cave mouths; populated five-trainer journey with selected PP,
  rock-cut passage, medicine and Continue; native Surf rock boundaries and both seams.
- R08: **two production-controller cases pass**—the populated journey and
  actual Cut-gated reward/removal/reload case. This retains their existing
  coverage limits; it is not a claim that every regional battle was played.
- All **377 audio programs** pass PCM hash, frame-count and loop-range checks.

Evidence logs are `.local/authored-terrain-revision-protected-final.log`,
`.local/quality-r05-controller-current.log`,
`.local/quality-r08-controller-current.log` and `.local/quality-review-audio.log`.
The strict immutable comparison is
`renders/terrain-revisions/9cb8936dbd1a-to-5fe3071c6242/comparison-6c2194d80e104e3fb906c265b19075117894805dfb8346e3aaa8e32341295001.json`.
The controller fixtures retain explicit legal party/badge/position and RNG
control; R05 starts its varied replay-divider stream at the first battle.
Gameplay uses the production controller. Their local results do not
establish an earned regional journey.
Both actual journey saves also load through stdio MCP, open production Start
and move one real tile with byte-empty stderr; see `.local/quality-review-mcp.log`.

## Reproduce the two Rust revision stages

Use the completed connected baseline and the existing Rust CLI. It rejects stale
previous terrain and preserves the installed route events instead of reinstalling
the route. Both outputs remain ignored; the baseline is not overwritten.

```sh
mkdir -p .local/minnesota-terrain-revision-stages
CARGO_TARGET_DIR=target/runtime-fps cargo build --profile wasm-dev --locked \
  -p crystal-assets --bin revise_authored_route

target/runtime-fps/wasm-dev/revise_authored_route \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  .local/r05-visual-before.layout.json \
  planning/minnesota/routes/MN_R05.layout.json \
  planning/minnesota/routes/MN_R05.json \
  .local/minnesota-terrain-revision-stages/r05-cleft.crystalpack

target/runtime-fps/wasm-dev/revise_authored_route \
  .local/minnesota-terrain-revision-stages/r05-cleft.crystalpack \
  .local/r08-visual-before.layout.json \
  planning/minnesota/routes/MN_R08.layout.json \
  planning/minnesota/routes/MN_R08.json \
  content-packs/minnesota-quality-review.crystalpack

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-quality-review.crystalpack --audio --desktop

.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_authored_terrain_revision.py \
  --before content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  --after content-packs/minnesota-quality-review.crystalpack \
  --revision .local/r05-visual-before.layout.json \
    planning/minnesota/routes/MN_R05.layout.json planning/minnesota/routes/MN_R05.json \
  --revision .local/r08-visual-before.layout.json \
    planning/minnesota/routes/MN_R08.layout.json planning/minnesota/routes/MN_R08.json \
  --self-test
```

Protected reusable authoring snapshots are retained under
`renders/terrain-revisions/9cb8936dbd1a-to-5fe3071c6242/sources/`:
`MN_R05-before.layout.json`, `MN_R05-after.layout.json`, `MN_R05.dossier.json`,
`MN_R08-before.layout.json`, `MN_R08-after.layout.json` and `MN_R08.dossier.json`.
These are the authored revision inputs, not an extracted game catalog. They
allow reproduction after the live source advances; substitute those paths for
the matching layout/dossier arguments.

## Final checks and remaining scope

All seven terrain-revision tests pass with both the explicit installed pack and
the default current-source fixture. They include full-pack preservation/strict
CBOR roundtrip, stale previous blocks, encoded disagreement/shared labels,
identity/extent/tileset/grid drift, stale event anchors, damaged required grass
and source-art recipe drift. The three changed Rust leafs pass scoped formatting.
All-target `crystal-assets` Clippy passes with the existing `never_loop`,
`bad_bit_mask` and `approx_constant` allowances and unrelated baseline warnings;
no warning references the new helper/CLI. Logs are
`.local/authored-terrain-revision-tests-current.log`,
`.local/authored-terrain-revision-default-tests-current.log` and
`.local/authored-terrain-revision-clippy-final.log`.
Whole-workspace formatting still reports pre-existing differences beginning at
`crates/offset-allocator/src/lib.rs:408`; no unrelated formatting was changed.

The original twenty-route visual objective remains open. Remaining weak spots
include R04, R06, R07, R10 and R11; source/native equality and green tests do not
settle aesthetics. Final user approval, the complete earned regional journey
and Safari capture expeditions also remain open.
