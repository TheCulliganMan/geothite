# Gym continuation: choices with a useful payoff

This source continuation develops two material improvements after reviewing all
eight Gyms and five caves in `minnesota-gym-cave-overhaul-v02.crystalpack`:

- St. Cloud becomes a smaller airworks with one shared, reversible fan relay
  and an optional battle that opens a central catwalk.
- Bemidji rewards Brindel's optional battle with a useful trail between the
  western clearing and the northern approach, without bypassing either Cut gate.

The reviewed base has file SHA256
`70202415e78301d7d8015daf4a3177864b54c651bdc015f138bb4eccd3e23324`
and content identity
`0a93aeca89fb76e46eb17cf3b1941aeea53d9c2ff6fcfe0b7ab8fe7866c9bf80`.
The compiled whole-room/player-view atlases are retained under
`target/northstar-gym-overhaul/compiled-review/`. The selected v04 pack uses the corrected routes below. Its compiled map views are under
`target/northstar-gym-overhaul/compiled-continuation-v04/`.

## Review of the actual rooms

The assessment used the compiled native atlases, player-sized cave views,
current sources/contracts, rejected-revision notes and the original Crystal Gym
atlas. Additional original Union Cave, Dark Cave and Ice Path rooms were rendered
read-only from the supplied pack under
`target/northstar-gym-continuation/mainline/`.

| Room | What works in v02 | Remaining weakness and decision |
| --- | --- | --- |
| Rochester | Compact introductory rail, one required encounter, short optional lane and a quiet bypass. | The court has spare floor, but the first Gym benefits from readable, low-pressure routing. Keep the introductory scale; a harder puzzle here would not address the weak late-game rooms. |
| Mankato | The rock spine, western ascent and three real drops make the return differ from the outward trip. | Upper terrace space is quiet and the trip is mostly prescribed. Preserve its useful ledges and avoid filling the quiet footing with arbitrary rocks. It already makes a stronger terrain lesson than another switch checklist. |
| Minneapolis | Each console can be related to a nearby bridge, and the middle gallery is revealed in stages. The service drop makes reversibility safe. | The solution is direct, but real spatial cause and effect is now legible. Keep the two-feeder introduction; do not revive the rejected guessed console order or all-FF native-art references. |
| St. Paul | Archive and workshop read as different places; either opponent qualifies Brannic. Reciprocal pads support returns. | Both branches remain short walks to a battle and a pad. The choice mainly concerns the opponent, not spatial problem-solving. Retain that pacing variety for now; adding a maze would obscure the civic fork without a stronger payoff. |
| St. Cloud | Arrows describe the real forced movement, and both-bank controls prevent stranded players. | The upper banks are oversized for their activity. Independent first-press toggles solve the route with little state reasoning, while Hallen offers no route benefit. Replace this with the smaller shared-relay/catwalk course below. |
| Brainerd | The small dock, visible buoy boundaries and explicit CENTER/LEFT choices have a clear physical identity. | The optional route must survive actual trainer approach positions. Lorla keeps her one-tile reach, and the continuation corrects Bravel's facing; do not copy the old south-facing Bravel ray back from v02. No geometry redesign here. |
| Bemidji | Pine-enclosed clearings, independent Cut branches and either-order required battles establish a coherent garden. | For Gym seven, the optional corner is a weak detour: its only benefit is the fight itself. Add a trail opened by Brindel while retaining both required keepers and both native Cut gates. |
| Duluth | Rough stops, progressive ice turns, required battles and a post-victory dry return form a coherent final course. | Difficulty still needs honest ordinary-party assessment, which is outside this static review. Keep the final combination and do not reopen the rejected early west bypass. |
| Driftless hollow | The early cave now has a fossil-ridge loop, a novice battle and a hidden reward while preserving the HM/story entrances. | Several eastern player views still show mostly empty floor around a broad rock spine. A future pass should give the two routes distinct costs or visible discoveries, not merely scatter more rocks. Lower priority than the two Gym fixes. |
| Route 10 gallery | The ore fork reconnects to the spring, giving a circuit rather than a single corridor. Trainers, visible items and hidden discoveries punctuate different branches. | The central/southeastern walking areas are wide, and several consecutive cameras contain little activity. Tightening a bottleneck around the existing ore or spring landmark would improve rhythm; another long spur with one hidden item would not. |
| Gunflint grotto | The healing alcove, upper circuit, voluntary angler and genuine Surf-only island serve different purposes. | The upper rim has a long stretch whose only distinction is walking around the pool. Preserve the healing bypass and water reward; a later improvement should change the return route rather than enlarge the lake. |
| Voyageurs 1F | The first floor introduces actual sliding, a dry central landing and separate item/ladder destinations. | It is a relatively forgiving navigation introduction. Keep that role; fixed rock stops alone do not reproduce Ice Path's cross-floor boulder puzzle, and the room should not be advertised as doing so. |
| Voyageurs B1F | Two basins, offset stops and separate reward pockets demand more alignment than 1F. | The route depends on reading similar rock stops; there is no cross-floor state change. A future major challenge should build on the two-basin connection and distinct rewards, not add untested movable boulders or unsupported holes. |

These judgments distinguish a deliberate simple room from an empty one. Not
every Gym needs the same puzzle length, and a hidden item does not by itself
justify a long, otherwise empty corridor.

## Mainline comparison

The supplied original maps show more than decorative variety. Violet's narrow
approach places two supporting encounters close to the route; Azalea uses a
compact loop; Cianwood makes boulders part of the floor plan; Mahogany gives
individual stopping squares consequences. Those are useful comparisons for
route density, readable obstructions and differing Gym lengths. Their native
event implementations remain the references:
[Violet](https://github.com/pret/pokecrystal/blob/master/maps/VioletGym.asm),
[Azalea](https://github.com/pret/pokecrystal/blob/master/maps/AzaleaGym.asm),
[Cianwood](https://github.com/pret/pokecrystal/blob/master/maps/CianwoodGym.asm),
[Mahogany](https://github.com/pret/pokecrystal/blob/master/maps/MahoganyGym.asm).

[Mauville's event source](https://github.com/pret/pokeemerald/blob/master/data/maps/MauvilleCity_Gym/scripts.inc)
demonstrates switches that change real barriers and restore their state on load.
St. Cloud applies the state relationship to a shared relay: crossing east and
returning west require different settings of the same system. Its optional
catwalk turns a battle into a useful route choice, rather than another mandatory
step. [Petalburg's doors](https://github.com/pret/pokeemerald/blob/master/data/maps/PetalburgCity_Gym/scripts.inc)
provide a separate reference for victories changing access and preserving that
change on return. Neither Minnesota candidate copies those layouts.

The original [Union Cave 1F](https://github.com/pret/pokecrystal/blob/master/maps/UnionCave1F.asm)
and [B1F](https://github.com/pret/pokecrystal/blob/master/maps/UnionCaveB1F.asm)
combine their through-routes with trainers, items and deeper/later-access
branches. The rendered mainline geometry uses water and narrow transitions to
separate those purposes. The current Minnesota caves have useful loops but
still contain stretches where a broad floor adds travel without changing the
decision. Ice Path's
[lower-floor boulders and rewards](https://github.com/pret/pokecrystal/blob/master/maps/IcePathB2FMahoganySide.asm)
are a more involved reference than fixed rock decorations; Voyageurs currently
uses a smaller, explicitly static sliding problem.

## St. Cloud: shared relay or battle catwalk

Apply [st-cloud.json](st-cloud.json) with
[st-cloud-contract.json](st-cloud-contract.json). The footprint changes from
9×11 to **9×9 metatiles**, removing two empty upper rows while retaining the
three indexed exterior warps and their destinations.

| Feature | New walking-tile coordinates |
| --- | --- |
| Safe arrival, facing down | `(4,15)` |
| Exterior exits | `(4,17)`, `(5,17)` |
| Saven / guide | `(3,3)` / `(7,14)` |
| Wyrel / Hallen | `(4,11)` / `(14,6)` |
| Lower west / lower east controls | `(6,13)` / `(13,13)` |
| Upper west / upper east controls | `(6,5)` / `(14,3)` |

Wyrel retains right-facing radius 3; Hallen retains left-facing radius 2.
Their native identities, parties, defeat receipts and optional flags remain.
All four controls use the explicit native EAST/WEST menu (`toggle:false`) and
share two new mutually exclusive state flags:

- `EVENT_MN_GYM_5_V2_RELAY_EAST`
- `EVENT_MN_GYM_5_V2_RELAY_WEST`

Both crossings start WEST. EAST changes both to `WALK_RIGHT`; WEST restores
both to `WALK_LEFT`. The player crosses the lower fan east, then reverses the
relay on the far bank to take the upper crossing west. The paired bank controls
also provide a reverse route. The old independent fan flags do not drive this
course. Integration builds from the exact published refinement parent and
preserves that parent's saved identifiers; it does not import the unpublished
five-cave review pack's saves into this revision. That review pack retains its
own identity and files. The fresh default is WEST even with both new mode flags
unset.

Hallen's optional victory opens a wooden catwalk across metatiles `(2,3)` through
`(5,3)`, changing native `tower:0b` WALL grilles to `tower:09` FLOOR. Four fresh
`EVENT_MN_GYM_5_V2_CATWALK_{2,3,4,5}_OPEN` receipts describe those blocks. This
connection also joins the west training deck to the leader bank, making the
return useful. It never opens just because the player changes the relay.

The leader predicate is Wyrel defeated **and** either relay WEST selected or
Hallen defeated. Hallen remains individually optional: the normal wind route
reaches Saven without entering his undefeated ray. Leader victory, rewards and
rematch scripts are retained. Reconstruct every gate from its declared state
on load; never clear a previously earned trainer win.

## Bemidji: optional battle earns a trail

Apply [bemidji.json](bemidji.json) with
[bemidji-contract.json](bemidji-contract.json). Dimensions, arrivals, exits,
actors, sightlines, both original Cut receipts and the post-leader return gate
remain unchanged.

The only geometry transition added is metatile **`(2,4)`**: the upper native
`forest:0e` bramble changes to `forest:01` grass. It opens only when both
`EVENT_MN_GYM_7_CUT_SOUTH_HEDGE_PRUNED` and
`EVENT_MN_GYM_7_TRAINER_3_DEFEATED` are earned, and uses new derived receipt
`EVENT_MN_GYM_7_V2_BRAMBLE_LINK_OPEN`.

This creates a west-to-east connection above the remaining lower shrub. From
Brindel's approach `(3,8)` to the north tree's approach `(7,7)`, static walking
distance falls from **21 steps to 5**, with the actual actor bodies retained.
Brindel's radius-1 encounter does not move him into the link. The player may
instead skip him and walk around through the entrance clearing.

The South Cut requirement matters for compatibility: a retained Brindel victory
alone must not open the link and let a player approach Moslen through an
unpruned South Hedge. Both Cut targets remain independently sealed; Moslen and
Elvren still qualify Orlis in either order. The new trail gives the optional
battle a spatial benefit without reducing the two required battles.

## Integration and static evidence

The 61 changed or added Gym dialogue labels live in the regional `story.json`
under `gym_overhaul.dialogue`. Overlapping `dialogue_overhaul.texts` entries
are synchronized. The four St. Cloud routing prompts are newly required because the
controls now offer a menu. Keep every unmentioned canonical line, including
the revised battle introductions for other trainers. All supplied lines are
ASCII and at most 18 columns.

The two room designs mirror their installed parties and actor art for
standalone authoring previews. Compilation takes the 26 parties from
`story-pokemon.json` and the textbox pages from `story.json`.

The ignored source review script is
`target/northstar-gym-continuation/check_and_render.py`; final inspected views
and its receipt are under `target/northstar-gym-continuation/v03/`. Whole rooms,
160×144 decision views and every prior render remain retained. The source graph
uses the supplied native collision tokens, actual blocking actors, forced wind
steps, explicit controls and abstract victories. It retains each trainer's
post-sight-approach endpoint instead of leaving the trainer at its original
position.

The fresh-state model covers 5,936 St. Cloud states and 2,120 Bemidji states;
every state in each full model can return to an exit. Separate forbidden-win
searches reach the qualified leader while leaving Hallen or Brindel undefeated.
Both Cut gates still seal their targets with the other tree already removed,
and an old Brindel win alone cannot bypass South Cut. These counts are source
topology evidence, not game inputs or combat results.

The production authoring graph includes prior save receipts and actual trainer
approach positions. Its reverse search requires a route from a fresh entrance
to the qualified leader and from each required victory to a qualified leader
without taking an optional fight. Bravel now watches his right-hand battle
square. His former south-facing ray intercepted the shared lower pier even on
the advertised LEFT bypass. The regression restores that facing and requires
the build to reject the room.

All 18 supporting trainers have individual win and loss reactions. Hallen
acknowledges the catwalk he promised, Tordel reacts to his partner's Encore,
and Jorel calls Mudnip back up from the dirt. The 36 text labels use the same
native battle result slots as before. The canonical dialogue also corrects the
St. Cloud denial and Bemidji HM gift so their directions match the actual
prerequisites. The St. Cloud arrival in the combined migration is (4,15).
