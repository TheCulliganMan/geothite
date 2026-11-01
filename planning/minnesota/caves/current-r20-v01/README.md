# Voyageurs granite cleft: seep, two mouths and useful drops

The9×8metatile cleft keeps its two real entrances and opposite stair climbs.
A recessed west-wall seep now shapes the dry lower approach, the upper lip
gains one visible walking tile, and attached rock closes the eastern shoulder.
Native bedrock encloses the room. The original Hyper Potion remains on the
western supply shelf. No new admission requirement, Surf gate, NPC or extra
entrance is introduced.

- [Complete room before/after](../../renders/r20-current-cleft-v01/published-v02/whole-before-after.png)
- [Whole published room](../../renders/r20-current-cleft-v01/published-v02/MnR20Cave1F-whole.png)
- [Nine native camera views](../../renders/r20-current-cleft-v01/native-v02/MnR20Cave1F-production-cameras.png)
- Reached views: [upper lip](../../renders/r20-current-cleft-v01/native-reached-v02/upper-lip.png), [seep bank](../../renders/r20-current-cleft-v01/native-reached-v02/seep.png)
- [Whole-pack preservation](../../renders/r20-current-cleft-v01/published-v02/receipt.json)
- [Actual two-process stdio replay](../../renders/r20-current-cleft-v01/mcp-v02/receipt.json)
- [Connected exterior atlas](../../renders/r20-current-cleft-v01/atlas-v02/full-state-native-current.png)

Most of the48changed metatiles finish the rock enclosure. They do not describe
48new activities. The53free ordinary floor cells remain connected with the
item occupied. Eight new water quadrants form the recessed seep, clear of
every map boundary; both climbs and the lower mouth-to-mouth passage stay dry.

## Exact terrain and native mouth contract

The [canonical source](../johto-quality-v01/MnR20Cave1F.json) is frozen in
`source-v01/`, SHA256
`2121f1208195a058db12bdfc64ae90c9cd61329c19b39fd5a87bc4ba63f32514`.
The previous source SHA256 is
`72a17aece9bf2e50ef7789550053b594d7934b61f03d021ab1ea124839845688`.
`source-v00/` preserves the prepared geometry before the retained controller
probes were restored. That editorial addition does not change terrain.

The [native contract](native.contract.json) binds the current parent and both
mouths. All coordinates are16px gameplay tiles unless marked as metatiles.

| Feature | Native coordinates and behavior |
| --- | --- |
| West inside mouth / clear floor | (5,13) / (5,12), inside warp1 |
| West exterior | MnR20C00R00, active(60,33), true passive(60,34), indices1/2 |
| East inside mouth / clear floor | (13,13) / (13,12), inside warp2 |
| East exterior | MnR20C01R00, active(12,31), true passive(12,32), indices1/2 |
| West supply stair | metatile(2,3), x4,y5–8 |
| East return stair | metatile(5,5), x11,y9–12 |
| Upper lip | native05 at metatile(5,3), clear floor(10,7) |
| Left-drop approaches | (10,8) and(10,9) |
| Left-drop triggers / landings | (9,8/9) / (7,8/9), crossing WALL(8,8/9) |
| Closed side shoulders | (8/9,7) and(8/9,10) |
| Ordinary uphill returns |15inputs for the upper lane,13for the lower |
| Original cache / approach | HYPER_POTION×1 at(5,5), approach(4,5), face right |
| Original flag | EVENT_MN_R20_ITEM_01 |
| Palette / border | Forced native NITE; solid cave block2e |

Each inside exit returns directly to its own exterior passive index2. Walking
right and back over that Floor cannot reenter; deliberately moving up into
the active CAVE reenters the matching inside ladder. The original east passive
record itself still has target_warp_id1. It is an inactive Floor arrival, so
do not normalize that retained metadata to the east active entrance's2.

Both leftward drops work through native input. Rightward reversal is blocked,
both ordinary uphill returns remain, and all four side shoulders are solid.
The drops shorten the return from the upper eastern cleft; they are not fake
cliff art over ordinary two-way floor. Both native stairs work both ways and
the old eastern toe bypass stays closed at(12,11).

The original item prints its native found/pocket pages and owns movement until
acknowledged. Native Pack use restores200HP to the explicit injured partner
and consumes the only Hyper Potion. Reentry and Save/Continue retain the item
flag and consumed quantity, without changing unrelated flags, money or party
members. The test's source-legal Mew80party is an explicit local fixture, not
earned Voyageurs progression or a new regional encounter.

## Current parent and preserved work

Ignored output: `content-packs/minnesota-r20-seep-cleft-v02.crystalpack`, SHA256
`12efb9c264e46c1329e7dc9f52aa0fc95d8a75194a96a46eb8520491571720c3`, identity
`bdbef89c2ca7af31cc21a78b355431c5743b40a1154df54b37c2d91b32824942`.
The exact parent is accepted R04 grass, SHA256
`18d2114538e1c02dba7a05947b75c81cc603934d03ad215ff5a0df941ee71c07`.
Only this cave's blocks, border01→2e, native block/attribute mirrors and
derived identity change. All690other complete maps, every native program,
event, reward, roster, encounter, art, audio, HOME, recovery and report remain
exact. This includes R04's eleven grass cells, all earlier grass/cave work,
the compact warm coast, snowy pine Ely, cities and complete river stage.

The first fully reviewed R17-parent candidate remains
`minnesota-r20-seep-cleft-v01.crystalpack`, SHA256
`537d9e4eb704d4e825349b018c2fce9f81ea34cd0bda2dacf3dee64a4fd3d20e`.
Its contract, renders, checkpoints and test logs remain separate first epochs.
After R04's handoff the identical source was published onto that newer parent
and all four controller cases/native/MCP/audio checks were rerun on v02.
Earlier-parent evidence is not relabeled as a new-parent execution.

After final-parent preservation, gameplay, native visual and stdio checks,
`world.json` selects `r20_seep_cleft_revision`. All prior revision records
remain intact. [Bound verification](../../renders/r20-current-cleft-v01/verified-v02/receipt.json)
and its adjacent selection receipt retain exact source, binary, execution-log,
pack and world-preimage hashes. No user final visual acceptance is implied.

The final read-only whole-room/source audit takes0.173s after pack loading.
For source edits, use the existing `review_cave_quality.py` with this pack,
`--room MnR20Cave1F`, a fresh `--output` directory and no `--published` flag.
It previews the entire authored room and all source cameras without Cargo or
pack publication. The native publication and final acceptance remain separate.

The full map goal remains active. R01/R03 stone cave integration, missing named
hubs, passenger service, further grass/route/city quality and earned regional
progression remain open. Native2D review is not final3D acceptance.
See [PUBLICATION.md](PUBLICATION.md) for exact rerender/publication commands.
