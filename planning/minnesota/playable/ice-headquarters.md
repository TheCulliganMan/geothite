# ICE headquarters: cross-floor revision

The two-floor v5 chapter was a winding, mostly linear switch sequence. It did
not meet the user's Gen 1 Silph Co. reference. This successor changes the
navigation graph, not just the number of steps or floors.

## Reference study

The original Red/Blue Silph source was read directly:
[3F](https://github.com/pret/pokered/blob/master/data/maps/objects/SilphCo3F.asm),
[5F](https://github.com/pret/pokered/blob/master/data/maps/objects/SilphCo5F.asm),
[7F](https://github.com/pret/pokered/blob/master/data/maps/objects/SilphCo7F.asm),
[11F](https://github.com/pret/pokered/blob/master/data/maps/objects/SilphCo11F.asm),
and the [5F persistent doors](https://github.com/pret/pokered/blob/master/scripts/SilphCo5F.asm).
The annotated original-game
[7F floor plan](https://livedoor.blogimg.jp/tono1985/imgs/f/0/f0c87431.jpg)
was also viewed. No disassembly, ROM or source graphics were added to the repo.

The useful precedent is disconnected sections within the same floor, reached
through other floors; a credential that changes previously explored routes;
optional recovery/rewards; and a rival confrontation on the approach to the
final office. This is not an eleven-floor reproduction of Silph Co.

## Implemented source

The authoritative [network](../lairs/ice-headquarters/network.json) pairs actual
active tiles and safe arrivals. Four explicit floor files define native art
and collision. Rust adds them to the frozen drilling parent before the final
Minnesota-only binding. Generated packs remain ignored.

There are six dungeon floors plus the existing public office:

| Floor | Distinct navigation/job |
| --- | --- |
| Cargo | Existing breaker, Uma and opened service stair/return |
| Operations | Public and secure halves; a separate return pocket; relay access |
| Records | Public stair hall, isolated Ada room, locked recovery room |
| Maintenance | Bent public corridor, locked workshop, isolated service side |
| Transfer | Public loading section, captive room, separate transfer annex |
| Relay | Existing Reed battle and independently operated holding circuit |

Eight **bidirectional panel pairs** and four stair connections join these
sections. Six individually opened card doors persist through Continue.

The pre-card route can follow public stairs, explore the C/D service loop,
and reach the card only by taking A into Transfer and B into Ada's isolated
Records room. Stairs alone cannot reach her. Ada kept a card because ICE made
her sort detainee names without charges. She offers it in a spoken conversation;
declining or having no bag space leaves the offer retryable.

H returns the player to a sealed pocket on the earlier Operations floor.
Using the card there opens a newly useful return. Other card doors unlock
Nora's optional recovery room, Tess's detention room, Ollie's supplies and
alternate approaches through Maintenance/Transfer to the secure Operations
section. The player can approach the relay through the Operations door or
the G panel. The card remains in the bag and does not automatically open
every door, defeat Reed, disable the circuit or release Ellis.

Four additional guards use native trainer sight and actual battles. The
existing parasite/slush/Cindore/Cartick identity, Wrigglet-line bosses and
Minnesota-only species policy are retained.

## Native implementation boundaries

All gameplay runs through `VisibleShellController`. Transfer tiles have real
`WARP_PANEL` collision in a local native-art variant; original facility
collision and every original runtime file remain unchanged. Door restoration
uses native entry/Continue callbacks. No custom frontend dispatcher is added.

The graph preview proves that the card is unreachable by stairs alone, the
relay is sealed before credentials, the three optional rooms open afterward,
and reachable panel compartments have returns. Those checks are source/data
proof, not native playability acceptance. Versioned complete-floor previews
remain in `renders/story-events-v2/ice-headquarters-*`.

## Acceptance

Publication, native traversal, strict preservation, stdio and scene review
are in progress. The earlier v5 test record remains attached to its original
pack. It does not establish this successor's acceptance.
