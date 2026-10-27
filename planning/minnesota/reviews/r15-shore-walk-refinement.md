# R15 eastern shore walk — source handoff

The proposed change replaces 17 eastern apron metatiles with attached ordinary
woodland shoulders and a narrow S-shaped road. The complete route remains
32×18 metatiles using unchanged native Kanto art. This is an unpublished source
candidate; active `MN_R15.json` and `MN_R15.layout.json` remain unchanged.

## Frozen inputs

| Input | SHA256 |
| --- | --- |
| Current native pack `content-packs/minnesota-northstar-lake-country.crystalpack` | `da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50` |
| Previous dossier `MN_R15.json` | `2d3d16ebd1e4488c378b893e84db134ba0e24054cd8d493c0ee329aaf294e9d8` |
| Previous layout `MN_R15.layout.json` | `5cda48486308bcebef1937ee5a6045af45a6240ca76a44f93fdc5bab2321dd85` |
| Candidate dossier `MN_R15.refinement.candidate.json` | `57bdf7662fadc71473d4a9ca610f62141326abaf69fb6923093796d0ab3c3773` |
| Candidate layout `MN_R15.refinement.layout.json` | `04c1a8ba6e19e0e267c7876596f53d9a0a50dfb65fbe17b20a4f5c05d4a5b335` |

Previous sources are preserved under
`renders/central-lakes/refinement-20261027/before-da27/`. The candidate dossier
retains every original field exactly and adds only editorial `review_contract`.
The layout retains every original field and its complete original layers prefix,
then appends explicit terrain placements. No event contract changes are needed.
The frozen handoff folder includes exact copies and hashes of the reviewer and
source report; it references retained render passes rather than discarding them.

## Native integration boundary

Use the existing Rust `revise_authored_route_terrain` API/CLI. Its inputs are
base pack, frozen previous layout, candidate layout and the unchanged original
dossier. The candidate dossier's review annotations need not participate in native
lowering. The source/grid preimage is exact on the reviewed538 baseline; when
composing after another overlay, independently certify that R15 and its native
art remain exact before calling the API.

Only `MnR15C00R00.blocks`, the existing uniquely owned
`MnR15C00R00_Blocks` encoded payload and the derived identity may change.
The native module's attributes, events, all typed/script tables, party records,
encounters, room, source/runtime art, programs, audio, report, HOME/recovery and
all other modules/data must remain exact. The native terrain API already enforces
same extent/art/preimage and original event anchors; no new API is proposed here.

The [exact 17-cell delta and check report](../renders/central-lakes/refinement-20261027/v03/review.json)
is the bounded expectation for the written-pack audit. Added blocks are only
`0f` solid ordinary broadleaf, `6c` upper canopy with Floor lower quadrants,
`6f` lower canopy with Floor upper quadrants and `31` ordinary road. No Cut
quadrant is present in the complete candidate route.

## Gameplay acceptance after publication

All coordinates below are 16px gameplay tiles. Setup fixtures must label their
initial position/party/badges/RNG; subsequent travel and mutations use only
`VisibleShellController::press`. Local setup does not prove earned regional progress.

| Feature | Required unchanged behavior / revised observation |
| --- | --- |
| Endpoints | East gate `(60,24)` / landing `(56,24)`; west gate `(2,24)` / landing `(4,24)`. Preserve actual native module/warp identities. |
| New road | Walk from lower shore `(47,23)` through the two-cell S bend `(49,22)` → `(51,20)`, cabin gate `(54,19)` and east road `(55,24)`. Test both directions with actor bodies present. |
| Cabin | Active facade `(53,11)` DOOR enters unchanged `MnR15LakeCabin`; its existing room exits return to distinct passive Floor `(53,12)`. Remain outside after settling; deliberate facade reentry still works. |
| Dock | Both gate rows `(42,14)` and `(42,15)` remain open; upper/lower rails, western endcap and existing deck are exact. Revive `(37,15)` remains reachable through the dock. |
| Original optional activities | Road walker `(57,13)`, optional Niko `(55,17)`, watcher `(45,17)` and sign reading steps `(49,26)` / `(57,20)` remain reachable. Preserve actual dialogue, parties, battle/PP effects and reward flags. |
| Ron/grass/hop | Ron `(17,24)` still controls westbound passage; eastbound grass alternative and original meadow ledge remain. Hop from upper `(22,17)` through face `(22,18)` to landing `(22,20)`, reverse blocked `(22,19)`, ordinary return through western grass. Retain native fixture's actual measured return, not an invented shorter route. |
| Water | All water/shore/buoy/rock blocks are exact; source closure checks remain valid. Prove old reachable Surf boundary cannot leave through an unconnected edge. No new access seam is claimed. |
| Persistence | Original pickup single-use/flags, party/money/bag behavior and semantic true event/engine flags survive Save/Continue. No new flag or item is added. |

Native captures should use the same day palette, south-facing CHRIS and actual
tile top-left as the source views at `(47,23)`, `(53,15)`, `(46,18)`, `(49,22)`,
`(51,20)`, `(54,19)`, `(55,24)`, `(49,26)`, plus the two weak views `(22,26)` and
`(45,14)`. Avoid centering assumptions at map edges. Keep native original buffers,
all conversion stages and exact pixel/foreground comparisons. The cabin's large
court and western weak apron should be visible in the review, not omitted.

## Fast source loop

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_r15_refinement.py \
  --output planning/minnesota/renders/central-lakes/refinement-20261027/NEW-UNUSED-REVISION \
  --self-test
```

Choose a fresh immutable output path each time. `v01` retains the rejected
detached toe; `v02` corrects it; `v03` adds native player foreground and computed
weak cameras without further terrain changes. The current actual result is
38 passing source checks, ten saved rejected mutations and zero shared geometry
errors. Whole route, ten views and authentic current-pack Johto references were
personally inspected. Parent native publication/visual/controller review and final
user acceptance remain pending.
