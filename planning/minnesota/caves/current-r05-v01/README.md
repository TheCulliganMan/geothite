# Rock Cut on the current Minnesota lineage

Rock Cut is the short two-mouth passage through the R05 river bluff. Its
8×10-metatile layout has a sheltered healer turnout, a bent rock neck, one
ordinary stair and an upper Repel shelf. Continuous native bedrock encloses
the passage and fills the former black ceiling/border. Both original mouths
remain paired to their respective hillside entrances.

- [Whole room before and after](../../renders/r05-current-rockcut-v01/published-v01/whole-before-after.png)
- [Complete published room](../../renders/r05-current-rockcut-v01/published-v01/MnR05RockCut-whole.png)
- [Six actual native cameras](../../renders/r05-current-rockcut-v01/native-v01/MnR05RockCut-production-cameras.png)
- Reached frames: [turning neck](../../renders/r05-current-rockcut-v01/native-reached-v01/bent-neck.png), [north mouth](../../renders/r05-current-rockcut-v01/native-reached-v01/north-mouth.png).
- [Complete preservation and mouth receipt](../../renders/r05-current-rockcut-v01/published-v01/receipt.json)
- [Six rejected unsafe publications](../../renders/r05-current-rockcut-v01/guard-cases-v01/receipt.json)
- [Production two-mouth stdio replay](../../renders/r05-current-rockcut-v01/mcp-v02/receipt.json)

The whole room, source and native camera sheets, and both reached frames were
visually inspected against the original Union Cave reference. The passage fits
its function: the healer occupies a reachable side turnout, a stone toe turns
the neck, and the upper shelf separates the north exit from the Repel. Most
of the56 changed metatiles fill the old void; they are not56 new activities.
The native stair and useful existing connections supply the traversal.

## Exact local candidate

The ignored pack is `content-packs/minnesota-r05-rock-cut-v01.crystalpack`,
SHA256 `133226dbc5b8c164e5fbf17a475093acd6a26836394e5fe73d2f9bf763749097`,
identity `63950c00d1524ced8c177f328d60c8888de9792a699c16037e133fde8509c537`.
Its exact parent is `content-packs/minnesota-r09-rock-mine-v01.crystalpack`,
SHA256 `b8178a3de63a7aab68280e6ed3646bbfb28e1c168315139aeeca5f2ff0ede14d`.
Thus it retains the reviewed R09 and R10 caves, compact warm coast, snowy pine
Ely, back route and waterfront fronts. Shared region composition is separate;
this review does not advance the active world pointer.

The [canonical source](../johto-quality-v01/MnR05RockCut.json) has SHA256
`69956475c9e0afc210db345c8a08eb43c8be8f6c87e5f267924a42d4d19d1af2`.
The [previous source](../johto-quality-v01/MnR05RockCut.previous.json) has SHA256
`b2ec8e138d0174f535264fbd3dc4e6892171a74a89912a147c405e00effb9ff1`.
Only terrain, border01→2e, their native mirrors and derived identity change.
All690 other maps and all events, healer/reward programs, split indexes,
species, encounters, art, audio, HOME, recovery and reports equal the parent.
The complete outer perimeter is solid rock. There is no water in this room.

## Gameplay and actual connections

Coordinates are16px core tiles; only the stair's source cell is a32px metatile.
The source's `exit_doors` values are global exterior coordinates. Resolve the
south door into chunk row1 before attempting local movement.

| Retained contract | Native coordinates |
| --- | --- |
| South inside mouth / clear arrival | (5,17) / (5,16) |
| South exterior active / passive | MnR05C00R01(16,29) / (16,30), global(16,77)/(16,78) |
| North inside mouth / clear arrival | (11,5) / (11,4) |
| North exterior active / passive | MnR05C00R00(30,27) / (30,28) |
| Native stair | metatile(4,3), ordinary floor lane x9, y5–8 |
| Healer / face-left approach | (5,13) / (6,13) |
| Repel×1 / face-left approach | (5,5) / (6,5) |
| Original once-only reward flag | EVENT_MN_R05_ITEM_03 |

The native interior warp indices remain1 for south and2 for north. Both exits
target exterior passive warp2. Each exterior's active CAVE record targets the
matching interior mouth; the passive Floor records retain their original
metadata and do not reenter the cave on arrival. Warps were not regenerated.

Harper's original two-page interaction owns input until the text is acknowledged.
It then restores actual party HP, PP and status, without an item or money grant.
The Repel remains a genuine100-step native item: pickup changes the real bag
and event flag once, Pack USE consumes it, walking decrements the effect, and
Continue retains the remaining steps and collected flag. No new story flag,
field move requirement or service is introduced.

## Verification and iteration

Three fresh production-controller cases pass: all six declared viewpoints,
the stair in both directions, both mouth exit/reentry pairings, Repel and
Continue; the original two printed healer pages/input lock and actual healing;
and the native Repel pickup, once-only return, Pack use and effect persistence.
The healer fixture deliberately starts with an injured/statused original
species; the geometry fixture has no party. These explicit local setups do not
earn route access or prove campaign progression.

Six complete640×576 native camera comparisons have zero differing pixels:
2,211,840 pixels total. Reached screenshots load actual controller saves without
position, time or facing overrides. All377 audio PCM/hash/frame/loop checks
pass. The new guarded Rust publisher reproduces the candidate byte for byte.

The freshly rebuilt production stdio client also exits and reenters both real
mouths, walks the18-input ordinary-floor/stair course between clear arrivals,
and opens Start/Pack. It exits successfully with byte-empty stderr. Reentry
lands on each original inside ladder, then an ordinary northward step clears
it; the clear tile is not substituted for the actual warp destination.

Six actual rejected publication probes retain the original candidate and write
no output: existing destination, unignored destination, retired geography,
walkable cave border, changed Repel quantity and an extra unpaired ladder.
The reusable terrain API is unchanged; its CLI now requires the current compact
geography before/after publication and strict readback, and a fresh ignored
untracked output. Historical cave branches remain readable and their existing
in-memory tests retain the original API.

The read-only [audit](audit.py) also renders the whole room and cameras after
loading the pack, measured at0.274 seconds. Choose a fresh output directory on
each review. All original, rejected, converted and native renders remain saved.
See [PUBLICATION.md](PUBLICATION.md) for executed commands and check limits.

Other caves, wider map quality, the shared region build and earned progression
remain ongoing. Final3D visual acceptance is not claimed.
