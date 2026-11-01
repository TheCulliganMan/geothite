# Iron Range mine on the current Minnesota lineage

The R09 mine is a complete12×11-metatile room with a fitted upper working
bench, three native stairways and a bent entrance shaft. Solid native rock
replaces the old black void/border. The foreman, survey cache, scout and mouth
retain their actual positions and current regional story. This is a terrain
overlay on the accepted R10/compact-coast/Ely/waterfront lineage, rather than a
replacement with the older standalone cave pack.

- [Whole room before and after](../../renders/r09-current-mine-v01/verified-v01/whole-before-after.png)
- [Complete published room](../../renders/r09-current-mine-v01/published-v01/MnR09Cave1F-whole.png)
- [Eight actual native cameras](../../renders/r09-current-mine-v01/native-v01/MnR09Cave1F-production-cameras.png)
- Actually reached native views: [turning shaft](../../renders/r09-current-mine-v01/native-reached-v01/shaft-turn.png), [fitted foreman bench](../../renders/r09-current-mine-v01/native-reached-v01/fitted-bench.png).
- [Preservation receipt](../../renders/r09-current-mine-v01/published-v01/receipt.json)
- [Bound verification receipt](../../renders/r09-current-mine-v01/verified-v01/receipt.json)
- [Production stdio replay](../../renders/r09-current-mine-v01/mcp-v02/receipt.json)
- [Actual connected exterior atlas](../../renders/r09-current-mine-v01/state-continuity-v01/full-state-native-current.png)

The whole room, native camera sheet and reached frames were visually inspected.
The lower stair emerges sideways into the entrance turn; the upper bench fits
the foreman's east face without a broad spare aisle. Two short downward drop
lanes provide return travel, with ordinary stair routes back uphill. The rock
infill accounts for most of the83 changed metatiles; that count is not83 new
gameplay features. Native art and the forced NITE palette remain unchanged.

## Exact publication and retained story

The ignored candidate is `content-packs/minnesota-r09-rock-mine-v01.crystalpack`,
SHA256 `b8178a3de63a7aab68280e6ed3646bbfb28e1c168315139aeeca5f2ff0ede14d`,
identity `ac459e36c3fdfc28c5bf736547ce0bb6ce993a770c6ad650eb218965cc6c5f81`.
Its exact parent is `content-packs/minnesota-r10-rock-gallery-v01.crystalpack`,
SHA256 `a69aa145e6ea870b459d79859958806170b56036af4f85e0c1e007629a2f67d6`.
The active world pointer remains R10 pending shared region composition.

The source is [the canonical room](../johto-quality-v01/MnR09Cave1F.json),
SHA256 `de255bc83a433a3c962a9e97d0b794146f3224bd33dfd809f65e4287431f9b64`.
The fixed previous-source preimage is
`7e0ead4bdd6aae39029618db5e9bfe504a32b127c9739a2c93233134bbe3f76c`.
The [current native contract](native.contract.json) has SHA256
`01d6f0f59ab69a73fce679cfcf6587426690469da64a91bd3cb8d5a44eb688f3`.

| Native gameplay contract | 16px core coordinate or condition |
| --- | --- |
| Mouth / clear indoor arrival | (13,19) / (13,18) |
| Exterior active entrance / passive return | MnR09C00R00(40,41) / (40,42) |
| Foreman / talk approach | (9,5) / (9,6), face north |
| Instrument cache / approach | (7,5) / (7,4), face south |
| Scout / clear approach | (19,7) / (19,8) |
| Three stair metatiles | (4,3), (8,5), (5,7) |
| Foreman prerequisites | Glacier Badge and EVENT_MN_HEADWATERS_REPORT |
| Exact current foreman party | BANDRAKE35, HELIKNOCK35, MESABLAST36 |
| Victory flag | EVENT_MN_DEPOT_FOREMAN_DEFEATED |
| Separate cache acknowledgment | EVENT_MN_MINE_INSTRUMENT_RECOVERED |
| Later handover, never granted by the mine | EVENT_MN_DEPOT_CLEARED |

The historical JSON retains GRAVELER/MACHOKE/MAGNETON and old dialogue as seed
provenance. They are not installed over the current faction dialogue, regional
party, moves, DVs, items or AI. The new current-only Rust publisher validates
their exact native preimages and leaves all programs and typed indexes intact.
It certifies a complete uncached reconstruction before and after publication.

Only the mine's blocks, border01→2e, their encoded/attribute mirrors and the
derived identity change. All690 other maps, species, encounters, fishing, art,
audio, HOME, recovery and reports equal the parent. The room has no water.
The outer perimeter is blocking rock, no new Cut gate is introduced, and the
approved warm North Shore/snowy pine Ely geography is preserved.

## Verification and next iteration

Three asset tests pass: full current-story/unowned-content preservation and
uncached reconstruction; invalid geometry, false source and stale-team
rejection; and the shared compact-geography positive/negative cases. The
current publisher's replay is byte-identical to the candidate. All377 audio
PCM/hash/frame/loop checks pass.

Four production-controller cases pass. One walks all eight declared cameras,
all three stairs in both directions, both native downward lips, reverse denial,
ordinary uphill returns, the mouth and Save/Continue. Three preserve the Q06
prerequisite refusals and locked cache, play actual victory with PP/experience
changes and every current opponent, test cache decline/acknowledgment/repeat/
Continue, and play a real losing battle without awarding evidence or victory.
The fixtures set initial position, prerequisites and party explicitly; these
are local gameplay proofs, not earned regional progression.

All eight complete640×576 native frames have zero differing pixels against
the source at the same position, facing and forced palette:2,949,120 pixels
compared. Reached screenshots load actual saved controller checkpoints without
position, facing or clock overrides. The whole-room/source-camera warm loop
took0.487 seconds after pack loading. Every render, rejected pass and save is
retained. Use fresh output directories rather than replacing them.

The production stdio client loads the latest reached shaft checkpoint, opens
Start/Pack, refuses three westward inputs into actual rock and walks
(9,18)→(10,18)→(9,18). It exits successfully with byte-empty stderr.

The current API `crystal_assets::validate_compact_superior_geography` guards
R11/R12 chunk sets and footprints, Grand Marais/Cedar dimensions, critical
reciprocal seams, a warm coast without ICE and consistent nonoverlapping
Minnesota outdoor placements. It allows legitimate terrain and gameplay
changes within those contracts; it does not establish their visual quality.
Original Johto/Kanto connections are outside this Minnesota placement check.

See [PUBLICATION.md](PUBLICATION.md) for executed commands and retained failed
attempts. Other caves, city density, coast/city continuity, Boundary Waters,
rail/fairgrounds and the broader map objective remain open. Neither full
campaign completion nor final3D acceptance is claimed.
