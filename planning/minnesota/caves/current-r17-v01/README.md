# Border Pines talus den

Keep R17's optional discovery short. Native bedrock now encloses the7×8room,
the upper stones join the overhang, and the attached lower toe turns the
approach west before the northward hook. The original Revive remains visible
in its small northern pocket. This is a ten-input discovery and short return,
with no new service, water, field-move requirement, stair or extra mouth.

- [Complete room before and after](../../renders/r17-current-den-v01/published-v01/whole-before-after.png)
- [Whole published room](../../renders/r17-current-den-v01/published-v01/MnR17Cave1F-whole.png)
- [Six actual native cameras](../../renders/r17-current-den-v01/native-v01/MnR17Cave1F-production-cameras.png)
- [Reached lower stone toe](../../renders/r17-current-den-v01/native-reached-v01/stone-toe.png)
- [Whole-pack preservation](../../renders/r17-current-den-v01/published-v01/receipt.json)
- [Two-process stdio replay](../../renders/r17-current-den-v01/mcp-v01/receipt.json)
- [Connected exterior atlas](../../renders/r17-current-den-v01/atlas-v01/full-state-native-current.png)

Most of the45changed metatiles finish the rock enclosure. They are not45new
activities. The free ordinary floor changes from27to26tiles, all connected with
the original item occupied. The original directional roof edges are retained;
native input can enter them from below and return. There are no open outer
edges, new Cut trees or isolated decorative walkable islands.

## Source and real native entrance

The [canonical source](../johto-quality-v01/MnR17Cave1F.json) is frozen in
`source-v01/`, SHA256
`0f714a8414119b95fc2d0a5f6355f1132918792d0e24e49566aa2059e22d2b1e`.
The exact preceding source has SHA256
`e7f4be4846edd4ee89f1176cd5b631b59e9ecd7c0ba47ef9d2791d6aa809fdf5`.
The prepared source before explicit current input probes remains in
`source-v00/`; its geometry is identical. All earlier candidate/native review
history remains in `renders/caves/` and is not relabeled as new-pack evidence.

The [native contract](native.contract.json) binds the exact parent, source
hashes, original reward and real paired entrance. Coordinates are16px gameplay
tiles unless explicitly labeled as metatiles.

| Feature | Exact contract |
| --- | --- |
| Inside ladder / clear arrival | (9,13) / (9,12), inside warp1 |
| Exterior | MnR17C00R00 |
| Active CAVE / true passive FLOOR | (10,43) / (10,44), exterior indices1/2 |
| Exit behavior | Inside warp1 targets exterior passive index2 directly |
| Fitted ordinary approach | (9,12) west to(5,12), north to(5,6), ten inputs |
| Original item / approach | REVIVE×1 at(5,5), approach(5,6), face up |
| Once-only flag | EVENT_MN_R17_ITEM_04 |
| Native roof cells | UP_WALL at(5,2),(6,2),(3,4); no collision substitutions |
| Palette / border | Forced NITE; native solid cave block2e |

Returning outside remains stable through presentation settling and a step
down to(10,45) and back to(10,44). Only deliberately entering CAVE(10,43)
reenters inside ladder(9,13). Native item text owns movement; after collection
the item body disappears, its tile becomes walkable, quantity stays1after
reentry and Save/Continue, and unrelated flags/money/party remain unchanged.

## Current-lineage child

The ignored pack is `content-packs/minnesota-r17-talus-den-v01.crystalpack`,
SHA256 `46074029adacca1aa0da411dbe878bb57b6e488d81eb5f2b444d0b40f9ffc329`,
identity `a0d9b83dce37dd6b7760ce970a9c2dc976b32fc602fee3184896c86400d3df41`.
Its exact parent is the accepted R13 spring-shelter pack, SHA256
`d9b4f2e85061e8e615cc668e68fb05b094bb38773e8dfc08d56af6dca4073816`.
All690 other complete maps and every unowned native program, event, reward,
species, encounter, art, audio, HOME, recovery and report remain exact.
Only this room's blocks, border01→2e, encoded block/attribute mirrors and
derived identity change. R03grass, R12/R13caves, compact warm coast, snowy pine
Ely, city/river work and all prior accepted data are retained.

After the fresh gameplay, native visual and preservation checks, `world.json`
selects `r17_talus_den_revision`. The exact prior selection and all previous
records remain recoverable. [Bound verification](../../renders/r17-current-den-v01/verified-v01/receipt.json)
and its adjacent selection receipt preserve the actual pack, source, executable,
execution-log and world-preimage hashes.

The original native forest/snow transition is preserved: the first two R17
chunks use `mn_winter_outdoor`, while the southern chunk uses `mn_border_pines`.
The older effect fixture incorrectly read every chunk through the forest bank.
Its local mouth proof now reads the actual installed bank and pins the
original warp indices, reciprocal targets, CAVE and passive FLOOR. All printed
item pages, input ownership, directional roof movement, resource and persistence
assertions remain intact; no production rule changes.

The complete-room source audit takes0.190s after pack loading. It reuses native
art and checks the actual whole pack, source arrays, enclosed perimeter and
explicit mouth. The frozen contract makes the same read-only reviewer usable
for later current cave publications. Native publication remains Rust.

The full map goal remains open. Three stone cave integrations, missing named
hubs, passenger service and broader regional quality remain. Local position,
empty-party and RNG fixtures do not earn Border Pines access or prove a
completed campaign. Native2D review does not establish final3D acceptance.
See [PUBLICATION.md](PUBLICATION.md) for the executed commands and honest checks.
