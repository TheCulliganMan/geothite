# R10 western rock stair — bounded candidate and native handoff

Source candidate accepted visually by Root after personally viewing the whole route and all five paired decision frames. Native acceptance and the later authorized layout-only promotion are recorded below; no user final approval or earned regional progression is claimed. This worker ran no Cargo, native pack write or exporter.

Baseline is the completed 510-map `content-packs/minnesota-northstar-repel-review.crystalpack`, SHA256 `31258a1788a986b158d0679e4f3a26c4dd476964311f707a6e09c9640cad48c6`. Full preimage dossier and layout are saved inside the immutable [v01-right-lane candidate folder](../renders/iron-range-descent/rock-stairs-candidate/v01-right-lane/). These are the current 48×20 exterior and refined 18×14 cave source epoch, not the historical smaller gallery.

| Source | SHA256 |
|---|---|
| `baseline-MN_R10.json` | `0fd088086912e9a13ec64bae47750e856ac01d1faaedad34f80720b89a268ffa` |
| `baseline-MN_R10.layout.json` | `2db59b63a1184b5f0975c30f3ff8fec1bebbce2bf6cf18c81895410f3ff94e5a` |
| `MN_R10.layout.candidate.json` | `293c2a1d4ba6e7141074bc6f5f26c83d754d8b42d6c4de0014cbec1c2c7317f3` |

Inspect [whole candidate](../renders/iron-range-descent/rock-stairs-candidate/v01-right-lane/after/whole-route.png), [five paired decisions](../renders/iron-range-descent/rock-stairs-candidate/v01-right-lane/before-after-decisions.png), and [source proof](../renders/iron-range-descent/rock-stairs-candidate/v01-right-lane/source-proof.json). All original terrain/RGBA/RGB/crops/previews/overlays are retained. The rendered [v02-left-lane comparison](../renders/iron-range-descent/rock-stairs-candidate/v02-left-lane/after/viewport-western_ramp.png) is retained too. Its left lane needs an extra sideways turn; v01 fits the retained straight travel better.

The exact change is two metatiles inside [4,9,8,13): (5,10) changes johto74→new80, and (4,11) changes walkable scree71→attached blocking rock0a. The new stair is exact cave12 with native `[FLOOR,FLOOR,WALL,FLOOR]` collision. Right core lane (11,19)→(11,20)→(11,21)→(11,22) remains ordinary two-way movement. Core (10,21) is the native wrong-side Wall. The attached shoulder narrows the lower left edge without enlarging the rest turnout or creating a freestanding prop island.

The dedicated proposed tileset is `mn_r10_rockstairs`: clone native Johto blocks00–7f at the original addresses, then append only cave12 as80. Do not mutate shared Johto or reuse North Shore's81 address. Preserve exact native tile rows/pixels/palette indices; no palette override, row shuffle, warp or special stair movement is involved. The existing bounded `north_shore_art::append_native_cave_stairs` performs allocation into unused logical/physical glyph slots and preserves all referenced original pixels. Root added only the exact alias80/[12] guard and the corresponding route-parser alias allowance.

Expected native changes: both existing exterior modules (`MnR10C00R00`32×20 and `MnR10C01R00`16×20) use the new alias, with their cached attributes, split `map_attributes` and uniquely owned encoded block payloads kept coherent. The 18×14 `MnR10Cave1F` stays exact. Add one tileset, five alias runtime files, and exactly two manifests: `minnesota-r10-rockstairs-art` and `minnesota-r10-rockstairs-native-stair`. Keep all old runtime files, audio/storage manifests, other catalogs and field-move tables exact. The alias intentionally uses no behavior-table inheritance only after proving both old/new terrain have zero Cut Tree or Whirlpool collision; Surf uses the unchanged native Water permissions.

Keep the five installed source/native walking-seam deltas exact: metatile `(0,5):68→01`, `(47,10):02→05`, `(47,11):02→05`, `(47,12):02→01`, `(47,13):02→05`. The dedicated revision certifies them against the completed preimage and reapplies them after compiling candidate geometry; generic terrain replacement must not erase these city links.

All dossier fields remain byte-identical, including actors, parties, rewards, story flags, signs, rest turnout, six-step pine habitat, native short ledge/ordinary return, two-way rock climb anchors, protected Surf shore, cave, doors and mouths. The source review reuses `review_league_approach`/`SourceArt` rendering rather than implementing another renderer. Complete ordinary route and climb checks pass, including directional side-wall masks. Six negative source cases reject: changed retained field/extent, wrong stair orientation, transformed art, extra out-of-envelope stroke, missing stair, removed shoulder. These are source/collision checks, not actual production input.

Native leaf: `crates/crystal-assets/src/r10_stairs_revision.rs`, API `revise_r10_rock_stairs(base, previous_layout_json, revised_layout_json, dossier_json)`. It certifies the frozen source bytes, native art hashes, installed chunks/payloads and original event/party/door anchors; builds bounded art, validates route geometry, and exercises the core movement implementation in both stair directions plus the blocked side. The CLI `revise_r10_rock_stairs BASE BEFORE_LAYOUT AFTER_LAYOUT DOSSIER OUTPUT` requires complete strict written readback equality. Root owns shared registration, Cargo and publication.

Five prepared ignored actual-pack cases in `tests/r10_stairs_revision.rs` cover full remaining data/all508 unrelated modules/cave equality; five new alias files/two manifests and source physical-glyph fidelity; source recipe/stroke/metadata negatives and repeat rejection; missing city seam/native-art drift rejection; all510 truly uncached reconstructed modules and actual strict written roundtrip. Input paths are canonicalized. `CRYSTAL_R10_STAIRS_PACK` overrides the default pack but retains its31258 SHA assertion; `CRYSTAL_R10_STAIRS_SOURCE` overrides the frozen candidate folder.

Executed here: source review for both orientations and scoped Rust formatter/check. No Cargo results are claimed. Reproduce the fast source loop with:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_r10_rock_stairs.py \
  planning/minnesota/renders/iron-range-descent/rock-stairs-candidate/v01-right-lane
```

Root's next native commands, not executed by this worker:

```sh
cargo test --locked -p crystal-assets --test r10_stairs_revision -- --ignored
cargo run --locked -p crystal-assets --bin revise_r10_rock_stairs -- \
  content-packs/minnesota-northstar-repel-review.crystalpack \
  planning/minnesota/renders/iron-range-descent/rock-stairs-candidate/v01-right-lane/baseline-MN_R10.layout.json \
  planning/minnesota/renders/iron-range-descent/rock-stairs-candidate/v01-right-lane/MN_R10.layout.candidate.json \
  planning/minnesota/renders/iron-range-descent/rock-stairs-candidate/v01-right-lane/baseline-MN_R10.json \
  content-packs/minnesota-northstar-rockstairs-review.crystalpack
```

Native acceptance requires real headless frames at core (11,22), (7,11), and both stair approaches with matching time/facing/player pose, plus actual controller movement and preserved gallery/ledge/grass/seam regressions. The actual results below satisfy this bounded pass. Existing saves and terrain overrides need a separate compatibility decision; no migration is implemented.

Native publication/read-only review update: Root strictly published the composed road-bends baseline `95384a3a48908329b89ab2bbe43ce99a18d3a02d5dcca9f9034a6341e08e8ce7` into `content-packs/minnesota-northstar-rock-stairs-review.crystalpack`, SHA256 `6071912eeb2fde6982f9e857bed9157d8d86da0f7ae56dba38665c809ab100e1`. The independent read-only review passed 180 checks and five actual headless production captures, with all whole-route native pixels/source recipe pixels equal. All 508 unrelated modules and complete remaining normalized data remain exact; field moves, original source files, audio and the gallery are unchanged. The initial five-view audit compared 880 opaque player pixels and 111744 terrain pixels exactly. The subsequent complete LCD audit compares **all115200 pixels**, including every NPC/player/foreground pixel, with zero mismatches and no exclusion masks. All original images/raw GPU buffers/earlier reports remain retained. Final full-LCD report SHA256 `c30a43b8859dd9bd40beaf86533458c18fcef0219c90230ba60965675b7e4d25`. Whole actual route and every frame were personally viewed by this worker and Root. See [native review evidence](../renders/iron-range-descent/rock-stairs-native/README.md).

The first actual assets run passed three cases and exposed two test assumptions: manifests intentionally derive a new runtime pack ID, and the input world already has an unrelated Bemidji cached/split mismatch. The failed test source and complete log are retained under `rock-stairs-candidate/native-tests/first-failure-1793075594207979000/`. The corrected tests assert the exact two-manifest runtime ID suffix and compare unrelated uncached modules before/after, rather than changing unrelated data; owned R10 uncached attributes/blocks must still match their cached result. The complete cached-data preservation assertion remains intact. Root's corrected warm run passed **all five asset cases in8.21s**: `.local/r10-outdoor-stairs-assets-corrected-fixtures.log`.

Root's **four actual production controller cases passed in11.81s** on exact6071: native outdoor stair both directions/blocked side; original route geometry/events and refined gallery; actual Calder battle/Continue; Surf seam and outer blocking rocks. Log `.local/exterior-bends-final-iron_range_descent_controller-pocket.log`; receipt `.local/exterior-bends-final-controller-pocket-receipts.json`, runs[2], exit0, private binary SHA256 `250c34008027211e8e7f1b44c8f5620a57cf956a7f8280ba47168082faf5d9bb`. These are explicit position/legal-party fixtures, not earned regional access.

Root explicitly authorized exactv01 **layout-only promotion**, now applied atomically. Active `routes/MN_R10.layout.json` is exactly candidate293c; `routes/MN_R10.json` remains byte-identical0fd. Full live2db preimage hash guarded the write. An independent raw-byte reconstruction removes only the new alias value, two appended stroke objects and appended recipe field and recovers the entire2db preimage exactly; all original layer prefix bytes remain exact. Full before/proposed/after and dossier before/after, preflight and independent application receipt are retained at `renders/iron-range-descent/rock-stairs-candidate/promotion-1793076078297044000/`. Receipt SHA256 `c15ee9814a1318f2e45cf10efad8ab14d9d2bc32d02df363a2ddad16a9ff72f1`. The native6071 pack remains unchanged. No shared interiors, other route or event data were promoted.
