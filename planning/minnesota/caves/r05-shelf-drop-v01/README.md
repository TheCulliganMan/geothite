# Rock Cut: upper shelf drop and ordinary stair return

The complete 8×10 two-mouth River Bluffs cave now has a short native drop beside
its upper Repel shelf. Descending from shelf `(6,5)` takes two movement actions:
step onto native ledge `(6,6)`, then hop over the wall course to `(6,8)`. The
ordinary stair return takes nine walking steps. The adjacent lane at x7 has its
own clear landing and a seven-step ordinary return. Both reverse and side
bypasses are closed with the native wall course.

The cave keeps its size and original purpose: a fast passage through the bluff,
a sheltered free healer and the once-only Repel. The drop supplies a useful
shelf-to-passage return. It does not replace the uphill stairs or any exterior
connection. The whole through journey is only slightly shorter; the largest
saving is the return from the Repel shelf.

The verified local candidate is
`content-packs/minnesota-r05-shelf-drop-v01.crystalpack`, SHA256
`71a40f87927ed85404c631869295b091225ac0e26c76fef0d084adf32e5f8d67`.
Selection is recorded in `planning/minnesota/world.json`; the full map objective
remains active. Native rows and programs were checked against the selected
Cedar Shelter parent in `contract.json`. The preceding route dossier/cave
review remains historical; the composed current source index selects this
room's `current.json` after acceptance.

Three hand-placed cells change: native wall cap `2d` at metatile `(2,3)`, native
ledge `39` at `(3,3)`, and its two-lane floor landing `02` at `(3,4)`. The original
native stair `12` at `(4,3)` and forced night palette remain intact. No graphics,
recolor, item, dialogue, actor, encounter, map size or warp is added.

- [Complete before/after](../../renders/r05-shelf-drop-v763/whole-before-after-v01.png)
- [Eight native cameras](../../renders/r05-shelf-drop-v763/native-v768/MnR05RockCut-production-cameras.png)
- [Actual reached lip and landing](../../renders/r05-shelf-drop-v763/reached-native-v770/actual-reached-views.png)
- [Native gameplay cases](../../renders/r05-shelf-drop-v763/controller-v768/receipt.json)
- [Production stdio replay](../../renders/r05-shelf-drop-v763/mcp-v770/receipt.json)
- [Exact parent and connection audit](../../renders/r05-shelf-drop-v763/audit-v768.json)

## Retained connections and gameplay

| Feature | Native core coordinates and behavior |
| --- | --- |
| South mouth | Insidewarp1 `(5,17)`, clear `(5,16)`; exterior `MnR05C00R01` active `(16,29)` / passive `(16,30)`, global `(16,77)` / `(16,78)` |
| North mouth | Insidewarp2 `(11,5)`, clear `(11,4)`; exterior `MnR05C00R00` active `(30,27)` / passive `(30,28)` |
| Uphill return | Ordinary floor through `(7,8)`, `(8,8)`, `(9,8)`, native stair `(9,7)` / `(9,6)`, upper floor `(9,5)` |
| Healer | Original `MN_R05_CUT_WALKER` `(5,13)`, approach `(6,13)` facing left; original two pages and acknowledged `special HealParty` restore real HP/status/all PP |
| Repel | Original `MN_R05_ITEM_03` `(5,5)`, approach `(6,5)` facing left; event `EVENT_MN_R05_ITEM_03`, quantity1; native Pack USE consumes it for100 walking steps |

Three default-profile production-controller cases pass. They cover both mouths
and passive returns, stair travel in both directions, original healer pages and
resource effects, Repel collection/once/consumption/countdown, both drop lanes,
blocked reverses, the full ordinary uphill return, all declared camera positions,
and Continue. Fixtures provide only initial local position and an injured
source-learned Minnesota Cindore where required. They do not earn route access.

The fresh production stdio replay starts from actual controller checkpoints,
traverses both mouths and both drop lanes, rejects reverse movement, walks the
stair return, revisits the original healer and opens the production Start/Pack
menu. Its two scenarios make263 calls with empty stderr. One starts from the
real continued Repel effect. All subsequent actions use the production controller.

All eight native640×576 frames match368,640 source pixels each. The whole cave,
source/native views and two actual reached-save frames were visually inspected.
An initial camera change to `(9,9)` was wrong; it is a native wall. The original
`(8,9)` coordinate was restored and all affected review checks rerun. Earlier
failed frames and inputs remain retained. All732 other maps, original programs,
art/runtime files and audio remain exact. All377 audio programs pass PCM hash,
frame-count and loop-range checks. Scoped Bevy all-target Clippy and changed
Rust formatting pass; workspace formatting retains pre-existing differences.

## Executed publication and fast review

The existing Rust publisher was frozen as `.local/r05-shelf-publisher-v764` from
`target/verdigrim-scene/debug/revise_authored_cave`. Its identity is bound in the
publication receipt. It validates the complete source preimage, original actor
and service programs, native directional hop geometry and strict output readback.
A fresh replay with the final source reproduces the exact candidate bytes.
An unsafe side-course variant is rejected without creating a pack.

```sh
.local/r05-shelf-publisher-v764 \
  content-packs/minnesota-r13-cedar-basin-v01.crystalpack \
  planning/minnesota/caves/r05-shelf-drop-v01/before.json \
  planning/minnesota/caves/r05-shelf-drop-v01/after.json \
  content-packs/minnesota-r05-shelf-drop-v01.crystalpack

PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-r13-cedar-basin-v01.crystalpack \
  --source planning/minnesota/caves/r05-shelf-drop-v01/after.json \
  --output planning/minnesota/renders/r05-shelf-next-preview --watch
```

Publication requires a fresh ignored output; use a new path for another run.
The warm complete source rerender measured0.051 seconds after pack loading.
All original, converted, cropped, rejected and production renders are retained.

The fresh default-profile controller build used:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 RUST_MIN_STACK=67108864 \
  cargo test --locked -p crystal-bevy --lib --no-run --message-format=json
CRYSTAL_R05_ROCKCUT_CONTROLLER_PACK="$PWD/content-packs/minnesota-r05-shelf-drop-v01.crystalpack" \
CRYSTAL_R05_ROCKCUT_SOURCE="$PWD/planning/minnesota/caves/r05-shelf-drop-v01/after.json" \
RUST_MIN_STACK=67108864 \
  .local/r05-shelf-controller-v766 r05_rockcut:: --ignored --nocapture --test-threads=1
```

Use fresh review directories when retaining checkpoint saves through
`CRYSTAL_R05_SHELF_REVIEW_DIR` and `CRYSTAL_MAP_RETAIN_SAVES_DIR`.
