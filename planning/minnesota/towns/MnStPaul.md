# St Paul — whole-town source review

Current candidate is **SOURCE only**: 32×24 metatiles (64×48 core tiles), native
Kanto exterior art, three complete local interiors, the canonical Gym reference
and the public entrance to Root's separately authored Union Depot. The canonical
gym is separately published in its review pack; this source preview does not
prove city services, facade binding, regional seams or a rail journey.
The fairgrounds street is visibly closed while its separate destination is
unauthored. The ordinary town and its services have no admission fee.

Current whole: [833baf26](../renders/st-paul/whole-town-833baf26.png).
[Player cameras](../renders/st-paul/viewpoints-833baf26.png) and
[current canonical gym](../renders/gyms/st-paul-og/St-Paul-review.png) accompanies it.
The prior room sheet remains an archived source review.
Layout SHA256 is `6c457277f3c0556fac29e191d849495994d6da9ee75627a86d0e670abb732c73`;
current gym hashes and anchors are pinned in the town JSON.
Source geometry is frozen for Root's native implementation review; this does not
claim final user visual acceptance or native integration.

The town is organized around the raised Capitol walk, its short one-way bank
and brick return, west clinic/Mart/fair street, eastern Steel Gym, stone-front
Union Depot and enclosed river park. All visible building doors have an
explicit room and reciprocal Floor return. The source references inspected by
sight are native Saffron, Vermilion, Celadon, Pewter, Goldenrod and Magnet Train
Station, retained under `renders/st-paul/references/`. They guide native facade
and street fitting; the town does not claim their original events.

The broad northeast lawn was corrected with one attached grove finger. The
[before/after pair](../renders/st-paul/ne-grove-before-after-833baf26.png) retains
both the same core 47,11 scenery context (now WALL, not a walking camera) and
actual Floor-centred 47,17 civic walk. Eleven former lawn 0a blocks become forest 0f;
all facade blocks and the original civic street remain exact. The inclusive
polygon's first rejected placement touched a roof/street; dfee44b5 remains
archived and is not current. Explicit facade and upper-street preservation checks
now reject that defect.

The [southern depot strip](../renders/st-paul/weakest-south-depot-lawn-e95b8f8a-160.png)
is unchanged, a short quiet lawn beyond the passenger seating. The unchanged
[east avenue](../renders/st-paul/weakest-east-avenue-e95b8f8a-160.png) is legible but
has little identity in isolation. The depot forecourt, building fronts and bluff
return carry the town's destinations; avoid filling quiet margins with unrelated
props. Final whole, paired grove contexts and all 13 current walking views were
inspected by sight against the native city references.

## Native lowering contract

All coordinates below are 16px core tiles. Exterior active/passive indices are
explicit proposals, not indices read back from a published native town.

| Destination | Actual exterior DOOR | Passive exterior Floor | Exterior entry / return | Interior passive arrival |
|---|---|---|---|---|
| Union Depot | 44,37 | 44,38 | 1 / 2 | 15,21, index 5 |
| Clinic | 7,15 | 7,16 | 3 / 4 | 3,5, index 3 |
| Mart | 23,15 | 23,16 | 5 / 6 | 3,5, index 3 |
| Capitol Hall | 34,9 | 34,10 | 7 / 8 | 3,5, index 3 |
| Gym4 | 54,15 | 54,16 | 9 / 10 | 2,17, index 3 |

The three local room records contain complete rows, native constants, actors,
text and active carpet records; the Gym resolves the exact canonical source.
Both carpets in each room target its matching exterior passive return. Clinic carpets are 3,7/4,7; Mart and Capitol carpets
are 2,7/3,7; canonical Gym carpets are 2,19/3,19. Their active indices are 1/2 and passive
arrival is 3. Root's depot source uses four public carpets 14..17,23, indices 1..4,
all targeting exterior 2, and passive 5 at 15,21. No exterior entrance targets a
train door. Root owns `transit/MnStPaulUnionDepot.json`, its network and native
implementation. `MN_CITY_COURIER` stays inside that depot at 20,16; there is no
exterior duplicate and Capitol Hall grants no Q03 completion.

The R04 west port remains gate meta 1,12 / landing meta 3,12, with real edge
Floor 0,24/0,25 and safe landing 6,24/6,25. The fairgrounds reservation is north
gate meta 8,1 / landing 8,3, edge 16,0/17,0. Source Officers at 16,2 and 17,2 seal both
lanes; the landing 16,6/17,6 remains publicly reachable. Removing only those two
bodies statically exposes a continuous future street. This is not native guard
dialogue, movement, connection or saved admission proof. Root owns the eventual
reciprocal fairgrounds connection; no fee or main-story prerequisite is planned.

The native Kanto 07 bank is core 38/39,20 `HOP_DOWN`, with `WALL` immediately
south and Floor landing 39,22. Approach 39,19 has a 27-step ordinary walking
return from the landing via native 05 brick at meta 13. There are no Cut trees.
The river uses actual pure WATER 43 and native Kanto shore fitting, is fully
enclosed and has no outer water seam.

## Room/event contracts

- **Clinic:** 5×4, `pokecenter`, native nurse 3,1 facing south, interaction 3,3
  across real COUNTER 3,2. Bind actual `HealParty` HP/status/PP care. PC 9,1 is
  actual PC collision, approach 9,2. Native 2d seating replaces the former stair;
  no visible or unbound 2F stair remains. Root must register honest recovery
  and prove native care/PC/warp behavior.
- **Mart:** 6×4, `mart`, source CherrygroveMart rows. Clerk 1,3 faces east across
  COUNTER 2,3, player approach 3,3. Bind actual native source inventory/economy;
  payment and capacity must gate delivery. Source script reservation is
  `CherrygroveMartClerkScript`; no replacement shop interpreter or new item.
- **Capitol Hall:** 8×4, `facility`, source SilphCo1F rows and counter desk.
  Records clerk 4,2 has approach 4,4 across COUNTER 4,3. Officer 13,1 directs the
  player to the indoor depot courier. Ordinary information only; no vanilla
  TM gift, MS03 dispatch reward or completion script is claimed.
- **Gym4:** the single canonical 7×10 `MnStPaulGym` / `mn_gym_4` is
  authored in `planning/minnesota/gyms/interiors.json`. Four compact chambers
  use original Saffron gym flooring, walls and reciprocal native pads.
  Belva `(3,3)`, a generic Cooltrainer F, and Orven `(11,12)`, a generic Pokéfan M,
  can be battled in either order. The four pairs connect foyer/west room,
  west/east room, east/foyer and foyer/court. Trying the court early reaches a
  closed grille at metatile `(5,2)` and a safe return pad. Both trainer wins
  release the grille. There are no quiz panels, switches or loose book objects.
  Brannic is a generic Gentleman at `(11,2)` with the unchanged Magnemite23 /
  Pineco24 / Skarmory25 party and explicit legal moves. Badge 3 is required;
  victory earns the Capitol badge and one retryable Steel Wing TM.

Current file/room SHA256 values and the exact native symbol allocation live in
`source_reference` in the town JSON. The checker rejects stale hashes, competing
room geometry, altered allocation and changed carpet/passive anchors. It follows
the canonical walkthrough and verifies that the closed court denies approach.
Native controller tests separately exercise all four pad pairs in both
directions, saving on pads, early-court escape, both trainer orders, battles,
rewards, loss and badge refusal. These are standalone interior fixtures, not an
earned city or regional journey.

For native city integration, bind only records 1–3 to the safe exterior return
record 10. Preserve all eight internal warp records 4–11, their same-map pairings,
the gate callback, actors, parties and rewards. The city exterior, three local
service rooms and depot contracts remain separate source work. Prior gym/room
renders remain archived as historical candidates.

Nine exterior residents, eight signs and thirteen walking-camera anchors are
explicit in the JSON. Ordinary dialogue fits 18 columns. Native city event binding, shop/care services and city room returns still require
the published town/controller phase. The separate gym fixture proves its own
battles, pad travel, rewards, exits and Continue.

## Fast review and evidence

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_st_paul.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_st_paul.py --self-test
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_st_paul.py --watch
```

The reviewer reads `content-packs/minnesota-geography-review.crystalpack` as its
native art authority, not as evidence this new town already exists in that pack.
Static checks pass: doors/safe returns, actor/sign approaches, closed fair street
and future street, canonical Gym hash/geometry/anchors/glyph mapping and walkthrough, real room carpets/PC/counters,
depot source ordering, bank/ordinary return, 18-column text and no outer water.
Twenty-two named valid-schema negative cases reject specific defects; they do not
exercise a controller. A cached-art complete rerender measured 3.27 seconds,
including all room/camera conversions and archival writes on this machine;
current load-dependent canonical-phase complete render was 5.72 seconds.

All whole-town, terrain, RGBA originals, converted RGB, native 160 crops,
previews, camera/room sheets and rejected passes are preserved with exact-byte
SHA history through shared `save_render` / `render_stage`. First rejected whole
f11fd96a had incorrect stone-bordered water fill; 5b4e555c corrected that and
completed rooms. Neither is current. No PNG is deleted. Original skeleton
dossier remains at ignored `.local/st-paul-before.json`.
