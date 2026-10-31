# Border Pines talus shelter

This small dry shelter replaces the cramped single corridor with two connected
arms around a central granite fall. The northern supply pocket, southern reader
and bent entrance remain close enough to read together. It retains the native
7×8-metatile size, original Revive and lichen researcher. The room supports a
short discovery and walking circuit; regional story progression is unchanged.

[Whole room](../../renders/r17-talus-v473/published-whole-2x-v484.png) ·
[Native camera sheet](../../renders/r17-talus-v473/native-v477/MnR17Cave1F-production-cameras.png) ·
[Exact implementation plan](plan.json)

Coordinates below use 16px gameplay tiles. The entrance is `(9,13)`, with clear
floor at `(9,12)`. It returns to `MnR17C00R00`, passive warp2 at `(10,44)`;
the exterior's active warp1 at `(10,43)` enters interior warp1. Neither exterior
record changes. The researcher stands at `(6,10)`, approached from `(7,10)`.
The Revive remains at `(5,5)`, approached from `(5,6)`, with the original
`EVENT_MN_R17_ITEM_04` flag. Its original scripts and dialogue remain exact.

Walk the western arm through `(5,9) → (5,7) → (5,6)`, cross north through
`(6,5) → (8,5)`, then return by `(8,7) → (8,8) → (7,8) → (7,9)`.
The four central cells `(6,6)`, `(7,6)`, `(6,7)`, `(7,7)` are solid native
granite. Both paths remain connected with the item and researcher present.
There are 45 free ordinary floor cells. This room adds no water, field gate,
stairway, trainer or creature. Its native encounter data remains unchanged.

`before.json` freezes the previous terrain source. `after.json` freezes the
publication input. `contract.json` binds the complete installed native module,
including the later researcher overlay that the old terrain source omits.
`current.json` carries current review annotations; the canonical room source
and current source index must agree with it after selection.

## Fast whole-room review

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-r17-talus-loop-v01.crystalpack \
  --source planning/minnesota/caves/r17-talus-loop-v01/current.json \
  --output planning/minnesota/renders/r17-talus-live --watch
```

The renderer reuses native art and saves every source snapshot, original image
and 2× image in an immutable version folder. The measured rendering phase took
0.029 seconds after loading the pack and art. Source previews do not publish.

## Reproduce the native child

The Rust publisher is `crates/crystal-assets/src/bin/revise_r17_talus_loop.rs`.
Build it with the pinned toolchain, under the shared Cargo slot. The executed
publication command was:

```sh
RUST_MIN_STACK=67108864 target/verdigrim-scene/debug/revise_r17_talus_loop \
  content-packs/minnesota-ice-turning-chambers-v01.crystalpack \
  0f734df8621b02da80cc45cb905de3d793e28b4f987f98d24203232ddb4cd3b4 \
  planning/minnesota/caves/r17-talus-loop-v01/before.json \
  planning/minnesota/caves/r17-talus-loop-v01/after.json \
  planning/minnesota/caves/r17-talus-loop-v01/contract.json \
  content-packs/minnesota-r17-talus-loop-v01.crystalpack
```

Output must be fresh and ignored; choose another output name on a rerun.
Resolve `planning/minnesota/world.json` before subsequent authoring. This exact
parent is retained for reproduction, not permission to replace a newer parent.

The guarded publisher changes only this map's blocks, its encoded payload and
the derived identity. All 731 other complete maps, original programs, actors,
warps, native art/audio and every other table remain exact. Negative cases
reject a stale complete-module binding, an open border and a blocked researcher.
All five native 640×576 camera comparisons match with zero pixel differences;
the actual camera sheet and complete room were inspected.

The controller case is `r17_loop_controller_both_arms_reader_revive_and_continue`.
It requires `CRYSTAL_INTERIOR_PACK` and a fresh `CRYSTAL_R17_LOOP_REVIEW_DIR`.
`check_mcp.py` accepts `--pack`, `--client`, `--save`, `--output`; use the
controller's declared initial outside save and the same Python environment.
Its two real processes cover both arms, dialogue ownership, collection, blocked
granite, the native return/reentry and Continue. Input files stay unchanged,
stderr stays empty, and native read-only save verification checks the once-only
Revive quantity and flag. Local empty-party setup does not establish earned
regional access or wild-encounter balance. Native camera fixtures separately
use one original Minnesota Cindore.

Verification and selection receipts live under
`planning/minnesota/renders/r17-talus-v473/`. The first controller attempt's
failure is retained: its static path helper excluded the collected item cell.
The corrected case enters that cell with ordinary production input. The map
was not weakened to satisfy the helper. Earlier cave designs and renders remain
retained; this revision does not claim that every remaining cave is complete.
