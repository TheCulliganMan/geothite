# Bemidji four-seam implementation contract

Status: **planned; connections and checkpoint scripts are not installed**. This
review leaves the frozen town, route sources, native city builder, and external
pack unchanged. It specifies a small additive connection layer, following the
native Boundary Basecamp connection pattern, with Bemidji's own story predicates.

## Exact inspected inputs

The inspected external pack is
`content-packs/minnesota-cities-review.crystalpack`, containing 498 maps.

- **Actual pack file SHA-256:**
  `6e3295367d6f2b4f6b07ef8f95f403968f2dcd3fe4790991b6e5f7380ef879af`.
- Its internal `identity.content_hash` is
  `10b50e16410e3a73b3ffe29228cc15abc4b6a021482d5e1a3419658e9afe1713`.
  This is an internal content identity, **not the file SHA or a newly published
  pack**. Root has not republished the corrected Gym7 global MapEvents mirror
  into this inspected file.
- Read-only geometry report SHA-256:
  `af5cedebf09de3da08cc5a1a7f5559e831064ec25dff49416ecd13e2d25dfa38`;
  [exact report](../renders/bemidji/canonical-inputs/seam-geometry-af5cedebf09de3da08cc5a1a7f5559e831064ec25dff49416ecd13e2d25dfa38.json).
- Static checkpoint proof SHA-256:
  `72cf669b68d5ee7bc0a0c428014ee520543308d49b9c43b71af667bc43cb5c77`;
  [exact proof](../renders/bemidji/canonical-inputs/seam-checkpoint-proof-72cf669b68d5ee7bc0a0c428014ee520543308d49b9c43b71af667bc43cb5c77.json).

The geometry report records exact hashes for the world, story, all four route
dossiers/layouts, and town layout. Key authority hashes at inspection:

| Source | SHA-256 |
| --- | --- |
| `world.json` | `4a24afa89a66c18c56bfbc00378ef00917b2d185aca1d20e88c5eb7b0ba8b476` |
| `story.json` | `27afee2095e41403efd8a20279d1baca956f9755902e248a6788727df7aabeaf` |
| `towns/MnBemidji.layout.json` | `bb51f6f46fcfd75d2dd3e2db482d6490d94889688f246f1072cc1626b236ae41` |

The current native city candidate still consumes the frozen Gym authority
`gyms-887afc43d19bc0573ca5ec361f4c141771347421040f70f100f02fea4997a189.json`
and matching immutable town dossier
`MnBemidji-c9cc0571747fa1b43b399ae3d23573ac8f66c791e58a7debb0c181cd5dfcb571.json`
in `renders/bemidji/canonical-inputs/`. The live town dossier and every previous
render/input remain retained. No gym revision is part of this connection review.

## Native connections and actual arrival tiles

Coordinates below are **local core walking tiles**, unless explicitly labeled
metatiles. Bemidji is 24×24 metatiles / 48×48 core tiles. Native connection
offsets use metatiles: target perpendicular core coordinate equals source
perpendicular coordinate minus twice the offset. Add both reciprocal records.

| Town side | Actual adjacent native chunk | Chunk metatiles / core extent | Town border pair → route border pair | Town direction / offset | Reciprocal direction / offset |
| --- | --- | --- | --- | --- | --- |
| South, R08 | `MnR08C00R00` | 32×24 / 64×48 | `(24,47),(25,47)` → `(40,0),(41,0)` | south / **−8** | north / **+8** |
| East, R09 | `MnR09C00R01` | 32×12 / 64×24 | `(47,24),(47,25)` → `(0,2),(0,3)` | east / **+11** | west / **−11** |
| West, R14 | `MnR14C01R00` | 16×24 / 32×48 | `(0,24),(0,25)` → `(31,26),(31,27)` | west / **−1** | east / **+1** |
| North, R17 | `MnR17C00R02` | 32×8 / 64×16 | `(24,0),(25,0)` → `(32,15),(33,15)` | north / **−4** | south / **+4** |

These are cropped native chunks. R14's east edge is x=31, not 63; R17's south
edge is y=15, not 47. R09's route-global gate metatile `(1,25)` is chunk-local
`(1,1)`, not core y=50. All sixteen border positions in this table are native
`FLOOR`, with no actor occupying them. Connection transitions arrive at these
border tiles, not at an interior warp arrival.

Current town connections are empty. Preserve the existing route internal links:

| Chunk | Existing links, all offset 0 |
| --- | --- |
| R08 `C00R00` | south → `MnR08C00R01`; east → `MnR08C01R00` |
| R09 `C00R01` | north → `MnR09C00R00`; east → `MnR09C01R01` |
| R14 `C01R00` | south → `MnR14C01R01`; west → `MnR14C00R00` |
| R17 `C00R02` | north → `MnR17C00R01` |

### Reconcile the world source before installation

The four current `world.json` links are authored as `kind: "gatehouse_warp"`,
not native walking connections. This contract proposes eight reciprocal native
walking records for the already open roads. Root must explicitly reconcile
that source kind with the connection implementation; do not silently reinterpret
it. There are no real gatehouse facades at these floor roads, so do not install
door triggers or floor warps merely to imitate the schema label.

Keep these source gate and safe-arrival references when reconciling it. All
coordinates in this table are **local metatiles**; the paired safe anchors are
their two core lanes, not warp triggers.

| Route | Route gate / safe arrival | Town gate / safe arrival | Route core safe pair | Town core safe pair |
| --- | --- | --- | --- | --- |
| R08 | `(20,0)` / `(20,3)` | `(12,22)` / `(12,20)` | `(40,6),(41,6)` | `(24,40),(25,40)` |
| R09 | `(1,1)` / `(3,1)` | `(22,12)` / `(20,12)` | `(6,2),(6,3)` | `(40,24),(40,25)` |
| R14 | `(14,13)` / `(12,13)` | `(1,12)` / `(3,12)` | `(24,26),(24,27)` | `(6,24),(6,25)` |
| R17 | `(16,6)` / `(16,4)` | `(12,1)` / `(12,3)` | `(32,8),(33,8)` | `(24,6),(25,6)` |

## Story gates and free inbound care

Check each named native badge flag, not a badge count or an assumption that all
earlier badges exist. Source gate labels below identify the authored contract;
their presence in JSON does not establish an installed native script.

| Outward route | Exact source permission | Native predicate | Source gate label |
| --- | --- | --- | --- |
| R08 | `BADGE_6` | `checkflag ENGINE_MINERALBADGE` | `MnR08EntryGateScript` |
| R09 | `BADGE_7` AND `EVENT_MN_HEADWATERS_REPORT` | `checkflag ENGINE_GLACIERBADGE`; `checkevent EVENT_MN_HEADWATERS_REPORT` | `MnR09EntryGateScript` |
| R14 | `BADGE_6` | `checkflag ENGINE_MINERALBADGE` | `MnR14EntryGateScript` |
| R17 | `BADGE_7` | `checkflag ENGINE_GLACIERBADGE` | `MnR17EntryGateScript` |

Q04 reaches Bemidji over the R08 land route after Brainerd/Leech Lake. Surf,
Cut, lake side quests, and `EVENT_MN_BRAINERD_REPORT` are not additional R08
border requirements. R14 access requires Badge 6, even though its Q05 work
requires Badge 7. R17 does not require the headwaters or border report. The
optional International Falls desk earns `EVENT_MN_BORDER_REPORT`; neither
Bemidji nor the route hut should grant it. Do not copy Basecamp's all-eight-
badges, harbor-signal, or restoration predicates. No Bemidji rail stop or pass
is added.

Q05 has two distinct earned interactions: MS06 at the accessible dry-land
Lake Itasca marker requires Badge 7 and grants `EVENT_MN_HEADWATERS_READING`;
MS07 at Bemidji requires Badge 7 plus that reading, compares the officer's
copies, and grants `EVENT_MN_HEADWATERS_REPORT`. Reading alone never opens R09.
The optional Mankato notebook remains optional. The current fixed Bemidji
officer at `(21,19)`, approached from `(21,20)`, has ordinary dialogue only:
**MS07 is unbound, and native chapter implementation is also pending for MS06.**
Do not claim an earned playable R09 progression until those scenes are bound
and tested. Do not fabricate a replacement report, weaken the gate, or grant
permission during a boundary crossing.

Unqualified inbound travelers may enter town and reach care. Every outward
attempt rechecks the actual predicate. Admission, escort, and guard visibility
must never set a progression flag, charge money, or grant a reward.

## Minimal additive checkpoint geometry

No terrain widening, extent change, source port move, or new art is needed for
these four openings. Add two native `OFFICER` actors per opening at the body
pair below, using temporary native visibility (`event_flag = "-1"`). These are
eight new checkpoint actors; the existing Q05 officer remains fixed. Reserve
distinct actor/script labels after checking the installed catalogs. Do not use
a story event as a visibility bit or cached admission permit.

Each coordinate is a core walking tile. Both lanes require coordinate events.

| Passage | Inner check plane | Body pair | Outer inbound plane | Refusal destination, same lane | Inbound facing |
| --- | --- | --- | --- | --- | --- |
| R08 south | `(24,43),(25,43)` | `(24,44),(25,44)` | `(24,45),(25,45)` | `(24,42),(25,42)` | UP |
| R09 east | `(43,24),(43,25)` | `(44,24),(44,25)` | `(45,24),(45,25)` | `(42,24),(42,25)` | LEFT |
| R14 west | `(3,24),(3,25)` | `(2,24),(2,25)` | `(1,24),(1,25)` | `(4,24),(4,25)` | RIGHT |
| R17 north | `(24,3),(25,3)` | `(24,2),(25,2)` | `(24,1),(25,1)` | `(24,4),(25,4)` | DOWN |

Inner-plane behavior: check the route predicate; on success hide both guards
and allow the road; on failure show the reason, retain dialogue input ownership,
and move one tile inward to the listed refusal tile. A blocked actor pair alone
is insufficient: retain both inner coordinate checks even while guards hide.

Outer-plane behavior: use native `VAR_FACING`. Skip inbound escort only when
facing outward. Inward or transverse arrival temporarily hides both bodies,
uses two native inward steps (outer → body → inner), then reconciles guard
visibility with current real permission. Inbound care remains possible even
without the outbound predicate. Test script continuation so arrival on the
inner event does not immediately deny or trap an escorted traveler.

Reuse the structure in
`crates/crystal-assets/src/boundary_basecamp_connections.rs` (`add_connection`,
`install_module`, `validate_geometry`, `add_guards`, `validate_connections`)
and its controller tests, extending its direction handling to north. Copy the
input ownership and facing behavior, not Basecamp's story requirements.

### Existing collisions and one route actor

Every listed corridor, body, checkpoint, and refusal tile is currently native
`FLOOR`; the town currently has no coordinate events at these points. The two
lanes are bounded by `WALL` at the town outer/body bands and at the route edges.
Floor widens farther inside town; keep that useful access rather than filling
it with obstacles. The full corridor and sidewall cells are recorded in the
geometry report.

The R14 route actor `MN_R14_TRAIL_GUIDE` at local core `(25,27)` occupies the
straight lower-lane path between edge `(31,27)` and safe anchor `(24,27)`.
Preserve the actor. There is an ordinary detour through y=26; a route-side
forced escort through `(25,27)` would block. The proposed escort is entirely
town-side and avoids this conflict. Prove both route arrival lanes and that
manual detour through production input before accepting the seam.

The static proof passed all four passages with these planned changes simulated:

- Blocking each inner plane separates both border lanes from the clinic, even
  when every actor is ignored and Surf water is considered traversable.
- With all eight closed guard bodies and all original NPCs blocking, both
  inside lanes still reach the six facade doors, clinic return `(31,10)`,
  officer approach `(21,20)`, and existing pier access/end/Surf launch.

That proof uses native collision cells and the actual downward ledge edge. It
does not execute coordinate scripts, connections, recovery, or controller
inputs and is not evidence of installed gameplay.

## Implementation and acceptance boundary

Implement one focused typed connection builder on the exact published city
candidate. Reject unexpected map extents, non-Floor checkpoints, occupied
actor/coordinate event cells, missing predicates, duplicate directions, wrong
neighbors, or offsets. Preserve original records and map geometry. Install
both `maps[..].attributes.connections` and `data.map_attributes`: the production
connection lookup reads the latter. Use `insert_compiled_map_split_payloads`
for global script/NPC mirrors, then remove cached modules in an isolated test
and reassemble from split data to prove the written payload, not just cache
coherence. The prior Gym7 global mirror defect makes that distinction material.

Before publication, require the following real regressions:

1. Resolve and drive **16 production crossings**: four seams × two lanes × both
   directions. Assert the exact border arrivals above, facing, stable input
   ownership, no bounce, both R14 manual paths, and retained internal chunk links.
2. Deny each missing predicate through the actual inner coordinate event and
   preserve all flags/items/money. Explicitly deny R09 with Reading but no
   Report; allow R14 with Badge 6 but no Badge 7; allow R17 with Badge 7 but no
   Report. Badge tests must distinguish the required named flag from a count.
3. Exercise both unqualified inbound lanes and transverse outer arrivals.
   Observe native two-step escort and body visibility, reach the clinic through
   controller input, heal HP/status/PP, use PC and both exit carpets, and Continue
   with unchanged permission. An escort that changes coordinates but leaves a
   pending script or denies care fails.
4. Reenter and save/reload both qualified and unqualified states. Reconcile
   temporary visibility from real flags; no hidden cached permit may bypass an
   inner plane. Retain the actor-free/Surf bypass and closed-body service-path
   proofs against the newly written map.
5. Compare every original map, source metatile/art/palette byte, facade warp,
   service/recovery row, trainer/party/reward, unrelated global table, and audio
   program. HOME and original recovery records stay exact; the two existing
   Bemidji add-on recovery rows stay exact. Allow only eight reciprocal
   connection records plus the explicitly additive guard/script/event mirrors.

Root owns the shared world reconciliation, story binding, registration,
compilation, pack publication, and controller execution. This document provides
an enactable geometry/story contract while those seams remain pending.
