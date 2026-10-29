# Gunflint shelter: fitted fork and spring bank

The16×12-metatile shelter now has an enclosed stone roof, a protected nook for
Otta, a fitted lower approach and an extended attached shoulder at the central
fork. The crossing bends below that shoulder before the separate shelter and
Max Ether climbs. Its three native stairs, two downward return lanes, spring
bank and original mouth remain playable.

- [Complete room before and after](../../renders/r12-current-shelter-v01/region-published-v02/whole-before-after.png)
- [Whole published room](../../renders/r12-current-shelter-v01/region-published-v02/MnR12Cave1F-whole.png)
- [Ten actual native cameras](../../renders/r12-current-shelter-v01/region-native-v01/MnR12Cave1F-production-cameras.png)
- Reached views: [central crossing](../../renders/r12-current-shelter-v01/region-native-reached-v01/shoulder-crossing.png), [spring bank](../../renders/r12-current-shelter-v01/region-native-reached-v01/spring-bank.png), [actual compact entrance](../../renders/r12-current-shelter-v01/region-native-reached-v01/compact-mouth.png).
- [Native preservation and mouth receipt](../../renders/r12-current-shelter-v01/region-published-v02/receipt.json)
- [Production stdio journey](../../renders/r12-current-shelter-v01/region-mcp-v01/receipt.json)
- [Current whole exterior atlas](../../renders/r12-current-shelter-v01/region-atlas-v01/full-state-native-current.png)

Whole room, source and production camera sheets, and reached frames were
visually inspected. The first current-lineage pass still left a broad central
crossing. The second extends native block19 from the existing attached rock
shoulder into metatile(7,6), blocking core(14/15,12) while retaining both
ordinary ledge approaches at(14/15,13). The reviewed camera moves onto actual
floor(15,13). The two drop lanes still return uphill in21 and19 ordinary inputs.
Most of the109 changed metatiles fill old black void with bedrock; the number
does not describe109 new activities. The spring retains all32 original WATER
cells and never reaches an outer edge. No Surf is required for either branch.

## Selected current-lineage pack

The ignored output is `content-packs/minnesota-region-polish-r12-v01.crystalpack`,
SHA256 `49b8121a4f1b0d00da34d7d93c61fffe784f097fbdfd682e15d9ecc9bb5993c2`,
identity `f3c41e401d512171141a41cf458a7c4782da32c4942fa1c97f46a157ae0a7905`.
Its exact parent is `content-packs/minnesota-region-polish-final-v03.crystalpack`,
SHA256 `67fb7ee40c022f0de78b561bbb144767d1483fe97a7dba9a2e6ec111d8c3360e`.
It retains all five accepted outdoor polish areas, the complete river activity
stage, Vergloom events/art and the verified R05/R09/R10 caves, compact warm coast
and snowy pine Ely. The initial R05-parent candidate and its complete evidence
remain separate historical epochs; their saves/images are not relabeled as
new-parent results.

`world.json` now selects `r12_shelter_revision` after the final-parent checks.
The [bound verification receipt](../../renders/r12-current-shelter-v01/region-verified-v01/receipt.json)
records the actual pack, source, binary and execution-log hashes.

The [canonical source](../johto-quality-v01/MnR12Cave1F.json) has SHA256
`52a29500418da67532498a74aa72742017fe09c21fc756bce03532a23494218f`.
The [previous source](../johto-quality-v01/MnR12Cave1F.previous.json) has SHA256
`bf39bc3d09073561fafef7053d66b09ce675b5227d84d5437a492916ec9357df`.
The exact first-pass source is retained in `source-v01/`; the final source is
also frozen in `source-v02/`. Both native packs and all earlier renders remain.

Only this room's blocks, border01→2e, corresponding native mirrors and derived
identity change. All690 other maps and every native event/program, dialogue,
reward, species, encounter, art, audio, HOME, recovery and report equal the
parent. Numeric map/warp identities are preserved. The original native07
mouth has an edge Floor at(16,23) beside its active ladder at(17,23); those
exact original quadrants remain, with solid rock everywhere else on the edge.

## Authoritative current connections and gameplay

See [native.contract.json](native.contract.json) for the explicit contract.
Coordinates below are16px core tiles, except the named stair metatiles.

| Contract | Current native coordinates or behavior |
| --- | --- |
| Inside mouth / clear floor | (17,23) / (17,22) |
| Exterior active / safe floor | MnR12C00R00(40,15) / (40,16) |
| Paired native warp indices | inside1 ↔ outside1 |
| Otta / talk approach | (5,5) / (6,5), face left |
| Max Ether×1 / approach | (27,5) / (26,5), face right |
| Once-only reward flag | EVENT_MN_R12_CAVE_ETHER |
| Shelter stair | metatile(5,3), floor lane x10,y5–8 |
| Cache stair | metatile(11,3), floor lane x23,y5–8 |
| Main stair | metatile(9,7), floor lane x19,y13–16 |
| Downward return lanes | approaches(14/15,13), lips(14/15,14), landings(14/15,16) |
| Blocked reverse | (14/15,15) are WALL; ordinary uphill returns remain |

The seed source's global exterior anchor66,23 belongs to the retired larger
Gunflint route. It remains provenance because the terrain publisher preserves
source fields; it is not the current entrance. Do not regenerate an outdoor
route from it. The actual pack, this contract and controller tests use the
current compact pairing above. The native cave exit targets the existing active
CAVE index1 and clears down to Floor40,16; no passive warp is invented.

Otta retains his original two-page shelter dialogue. He does not heal the
party or grant quest progress. The Max Ether is a real native item: pickup
changes bag/flag once, Pack use restores a selected legal move to its actual
maximum PP, and repeating or declining a full-PP use preserves resources.
Continue retains both the recovery and collected state. Separate catalog
checks also exercise Ether, Elixer and Max Elixer scopes/refusals through Pack.

## Verification and iteration

Three production-controller cases pass: every declared viewpoint, all three
stairs in both directions, both drops/reverse denial/ordinary returns, the
compact mouth, original item and Continue; Otta's printed pages/input ownership
and actual Max Ether PP restoration/once-only persistence; and the native PP
items' move-versus-party scope/full-PP refusals with exact resource preservation.

The renderer-neutral stdio client independently walks to Otta and the cache,
plays both drops, rejects reverse input, walks21/19-input uphill returns,
exits/reenters through the actual compact mouth and opens Start/Pack. It exits0
with byte-empty stderr. Both setups begin with explicit local position/party
fixtures; they do not earn Gunflint access or campaign progress.

All ten complete640×576 production camera comparisons have zero differing
pixels across3,686,400 pixels, including coherent native water animation.
Reached frames load actual saves without position, clock or facing overrides.
The read-only full-room/source-camera audit takes0.112 seconds after pack
loading on this final parent (initial candidate0.321s). The guarded Rust replay is byte-identical; all377 audio PCM/hash/
frame/loop checks pass. The [publication runbook](PUBLICATION.md) records the
executed commands, preserved first pass and verification limits.

The fresh actual exterior atlas retains97 maps in one seam component with no
footprint overlaps or inconsistent placement cycles. Its warp/seam graph is
not an earned regional playthrough. Named Detroit Lakes, Moorhead,
International Falls and Mille Lacs hubs remain absent; a Detroit Lakes approach
does not stand in for a completed town. Those broader requirements stay open.

Five remaining cave integrations, missing hubs and broader city/route quality
remain active. No full campaign completion or final3D acceptance is claimed.
