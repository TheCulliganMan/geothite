# Limestone hollows: fitted entrance and retained upper cycle

R01 keeps its two small hollows, two ordinary native climbs and useful upper
loop. Native28 rounds the attached rock spine into the entrance court while
retaining the adjacent floor used by the short return. Continuous native
bedrock replaces the surrounding black void. The room remains8×7metatiles;
the original guide and Antidote discovery are retained, with no new water,
Cut tree, admission condition, healer or extra exit.

- [Complete room before/after](../../renders/r01-current-hollows-v01/published-v01/whole-before-after.png)
- [Whole published room](../../renders/r01-current-hollows-v01/published-v01/MnR01Cave1F-whole.png)
- [Six native camera views](../../renders/r01-current-hollows-v01/native-v01/MnR01Cave1F-production-cameras.png)
- Reached views: [upper cycle](../../renders/r01-current-hollows-v01/native-reached-v01/upper-cycle.png), [spine toe](../../renders/r01-current-hollows-v01/native-reached-v01/spine-toe.png)
- [Whole-pack preservation](../../renders/r01-current-hollows-v01/published-v01/receipt.json)
- [Actual two-process stdio replay](../../renders/r01-current-hollows-v01/mcp-v01/receipt.json)
- [Connected exterior atlas](../../renders/r01-current-hollows-v01/atlas-v01/full-state-native-current.png)

Most of the29changed metatiles finish the rock enclosure; they are not29new
activities. The upper cycle was already present in the accepted parent. This
revision preserves that useful traversal and fits the entrance around it.
All53free ordinary floor tiles remain connected with both original actors
occupied. There are no open outer edges or unpaired ladders.

## Authoritative source and gameplay contract

The [canonical source](../johto-quality-v01/MnR01Cave1F.json) is frozen in
`source-v01/`, SHA256
`1468f2e8c70379eae0a9ea648f0ecbe7b067d3bca65f511998e5eef00d33d601`.
The exact preceding source SHA256 is
`aa84b08cfe43033f0b0b33b741a39f1e593fa6bf89ea662c9af708b2c8910d42`.
`source-v00/` retains the prepared geometry before the original controller
probes were restored. That editorial addition does not alter terrain.

See [native.contract.json](native.contract.json) and the complete adjacent-step
[input plan](input-plan.json). Coordinates below are16px gameplay tiles unless
explicitly labeled as metatiles.

| Feature | Exact native contract |
| --- | --- |
| Inside ladder / clear arrival | (9,11) / (9,10), inside warp1 |
| Exterior | MnR01C00R00 |
| Active entrance / true passive return | (18,17) / (18,18), exterior indices1/2 |
| West guide climb | native12 at metatile(2,4), x5,y7–10 |
| East supply climb | native30 at metatile(5,3), x10,y5–8 |
| Rounded spine toe | native28 at metatile(4,4); floor(9,8) remains clear |
| Complete upper cycle |26adjacent ordinary inputs from(9,10) back to(9,10) |
| East return from cache approach |6inputs from(10,5) to(9,10) |
| Guide-side return |18inputs from(10,5), across the upper seam and west stair |
| Original guide / approach | (5,5) / (4,5), face right; original three-page dialogue |
| Original cache / approach | ANTIDOTE×1 at(11,5), approach(10,5), face right |
| Once-only flag | EVENT_MN_R01_ITEM_03 |
| Palette / border | Forced native NITE; solid cave block2e |

The guide still names the two paths and the Antidote; he grants neither healing
nor quest progress. His dialogue and the pickup notice own movement until
acknowledged. The native Antidote actually cures the explicit poisoned
partner, consumes once and does not restore HP or unrelated PP. Reentry and
Save/Continue retain the consumed quantity, cured party and collected flag.
The original item tile becomes walkable after collection.

Both stair lanes work in both directions and retain their side walls. The
upper-seam save at(7,3) continues at the actual reached position. Outside,
the passive Floor remains stable after settling and walking down to(18,19)
and back. Only deliberately entering CAVE(18,17) returns to inside ladder(9,11).

## Current-lineage child and limits

Ignored output: `content-packs/minnesota-r01-limestone-hollows-v01.crystalpack`,
SHA256 `c7e3e0297533789a3c9e32f29db20cf69d73972b2a6f744192bc183fa23c74e1`,
identity `2cd511deac839c29e8ddd4729f6faa2aa255e7e4af6e48bff7f8026ec4a7a6b2`.
Its exact parent is accepted R20, SHA256
`12efb9c264e46c1329e7dc9f52aa0fc95d8a75194a96a46eb8520491571720c3`.
Only this room's blocks, border01→2e, encoded block/attribute mirrors and
derived identity change. Every other690complete map and all native programs,
events, rewards, species, encounters, art, audio, HOME, recovery and reports
remain exact, including accepted R04grass and R12/R13/R17/R20caves.

After the fresh native preservation, visual and gameplay checks, this child
was selected as `r01_limestone_hollows_revision`. All prior revision records remain
intact. The [bound verification](../../renders/r01-current-hollows-v01/verified-v01/receipt.json)
and adjacent selection receipt retain exact source, contract, input-plan,
binary, execution-log, pack and world-preimage hashes. No final user visual
acceptance is implied.

The later selected `route_grass_r05_revision` composes the completed League
text and33R05grass cells onto this accepted R01 parent. Its pack is
`content-packs/minnesota-current-grass-r05-v03.crystalpack`, SHA256
`79e14bbddb7723cfd688544f1c126668637c7583953b524a74008f332d7e61c3`, identity
`b04a7ba2bbfe1c930eef15d41662e0a7c7bbd1bc64923da4fb1b10d47305f37f`.
The independent [descendant readback](../../renders/r01-current-hollows-v01/r05-descendant-v01/receipt.json)
proves the complete R01 native module, payload/attribute mirrors, runtime art,
tilesets and audio remain exact. All57,344whole-room source pixels match.
Original controller/GPU logs remain bound to the accepted R01 pack above;
they are not relabeled as new-parent executions. R05/League acceptance has
its own [publication record](../../revisions/route-grass-v01/compact-rebase-v01/R05.md).
The [fresh selected-game atlas](../../renders/r01-current-hollows-v01/r05-atlas-v01/full-state-native-current.png)
includes that composition:97outdoor maps, one seam component, no overlapping
footprints or inconsistent placement cycles. Its source render was inspected.

Four fresh controller cases pass: the generic complete-room camera/stair/
mouth/item/Continue course; the original stair/passive-return case; original
three-page guide and actual Antidote effect; and the complete26/6/18upper-loop
course with an upper-seam Continue. Existing test assertions are not relaxed.
All six complete native frames match the source, and production stdio plays
the actual complete cycle, both returns, guide, pickup and passive mouth in
two processes. The explicit poisoned Totodile20fixture is source-legal local
setup, not an earned starter or a new regional encounter.

The existing source reviewer can preview the whole room and its six source
cameras without Cargo or pack publication: use `review_cave_quality.py`, this
pack, `--room MnR01Cave1F`, a fresh output directory and no `--published` flag.
Native acceptance remains separate. All source, rendered originals, crops,
conversions and prior review epochs are retained.
The measured full-room/source audit takes0.149s after pack loading.

The wider map goal remains active. R03stone cave integration, absent named
hubs, passenger service and further route/city/grass quality remain open.
These local fixtures do not earn regional/campaign completion, and native2D
review is not final3D acceptance. See [PUBLICATION.md](PUBLICATION.md).
