# Duluth — cliff stairs and connected streets

The [harbor revision](../revisions/duluth-harbor-platforms-v01/README.md) removes
the raised inner water edges and isolated buoy, keeping the outer buoy boundary.
Floating decks at metatiles `(26,36)` and `(34,39)` have native fishermen at
gameplay `(55,75)` and `(71,81)`, with open western Surf boarding faces.
Resolve `world.json.active_local_review_revision` and its authoring source index
before editing; the canonical layout below remains an older guarded preimage.

The current exterior is 64×46 metatiles, with 38 real doors and 76 paired exterior
warps. The cliff repair gives eleven native cave36 stair flights real upper and
lower landings, joins the western upper streets at one height, and closes the
garden's flat side-wall bypass. All ten ledge lanes have clear dry landings,
reverse denial and ordinary stair returns of 11–27 steps. Existing activities,
rooms, story, Gym8, services and regional connections are retained.

[Whole town](../renders/duluth-cliff-physics-v01/whole-town-v04.png)
· [Numbered stair plan](../renders/duluth-cliff-physics-v01/eleven-stair-flight-plan-v02.png)
· [Implementation and verification](../revisions/duluth-cliff-physics-v01/README.md)

The western native seam retains town tiles(0,44)/(0,45),
`MnR10C01R00` tiles(31,24)/(31,25), offset10. The northern seam retains
(64,0)/(65,0), `MnR11C00R03` tiles(46,47)/(47,47), offset9. Outdoor spawn513
remains(41,32). These are implemented walking connections, rather than the old
gatehouse reservations recorded below. Final selection follows
`world.json.active_local_review_revision`; the selected cliff repair preserves the State Fair building parent.

The [shoreline follow-up](../revisions/shore-rocks-open-routes-v01/README.md)
adds38 native boulders on grassy shore cells while retaining the open harbor,
Surf access and both floating fisherman decks. Its current source layout is
resolved through the selected `world.json` authoring index.

## Historical city and ice-course notes

The following sections retain older named publications and their evidence.
Their32×24/480-map geometry, planned seam reservations and old pack hashes do
not describe the current exterior. Use the linked cliff dossier and current
world selection for new work.

Historical status: local city/gym publication and focused controller checks passed.
The then-current 480-map candidate binds the remade 7×10 ice room to its actual
facade. [Review the complete gym](../renders/gyms/duluth-og/Duluth-review.png)
and [current commands/evidence](../gyms/README.md#duluth-review-loop).
The room and city preserve the hand-authored 32×24 exterior, six ordinary
interiors and reciprocal door indices. Regional connections and clinic
blackout registration remain separate integration work. Older source/native
hashes and verification sections below describe their named historical tuples;
their 12×12 gym, borrowed sprites and quiz drafts are superseded.

The exterior uses original `johto_modern` with no derived metatiles or modified
pixels. Gym8 uses the exact native glyph allocation of the shared gym builder.

[Whole city at native source resolution](../renders/duluth/whole-town-02bbba861c.png)
· [Player cameras](../renders/duluth/viewpoints.png)
· [All seven rooms](../renders/duluth/rooms.png)
· [Latest source proof](../renders/duluth/source-review-862f0e632b4a6438f28ed267424f03ca536c82ff45d6d33254af76c9b969edb3.json)

The west civic bluff holds Norek's gym, a hillside home and a tended flower
strip. Its south-facing native ledge shortens the trip to the harbor street;
the eastern ramp is the ordinary uphill return. Below it, real houses and an
ore-history room face the harbor. A two-gameplay-tile-wide steel-railed bridge
crosses the channel to the receiver workshop on Canal Park. The public fishing
pier has a usable deck and closed tip. A single low working-harbor cove admits
Surf; rails, pines and rocks protect the remaining shores. The lake's outer
edges are native blocking water rocks. This candidate has no Duluth train stop.

## Unchanged reservations and regional interfaces

All positions below are gameplay/core tiles unless explicitly called metatiles.
A metatile is 32 source pixels and contains 2×2 gameplay tiles.

| Interface | Existing reservation retained | Source approach |
| --- | --- | --- |
| R10 / west | Gate metatile `(1,12)`, arrival `(3,12)` | Connected street at core `(2,24)` and safe `(6,24)` |
| R11 / north | Gate metatile `(16,1)`, arrival `(16,3)` | Two clear lanes at core `(32,2)/(33,2)` and safe `(32,6)` |
| Exterior | 32×24, `johto_modern` | Single native map; no chunk-size change |

These are still the existing **planned `gatehouse_warp` contracts** in
`world.json`; the white port lanes are not proof of installed walking seams.
R10's counterpart is `MnR10C01R00`, gate local metatile `(14,12)`, arrival
`(12,12)`. R11's counterpart is `MnR11C00R02`, gate `(17,22)`, arrival
`(17,20)`. Root owns allocation/lowering and must retain their existing story
requirements. No shared world, story, interior or actor file was edited here.

## Exact exterior and room pairing

Every visible native `DOOR` quadrant is bound. Each active door index has a
separate passive FLOOR return index; entering a room targets its passive FLOOR
warp 3, not an exit carpet. Both room exit carpets target the matching exterior
passive return. The source checker stops graph traversal at facade doors.

| Door | Exterior core | Safe return | Active/passive exterior indices | Room |
| --- | --- | --- | --- | --- |
| Hill home | `(9,11)` | `(9,12)` | 1 / 2 | `MnDuluthHillHouse` |
| Gym | `(16,11)` | `(16,12)` | 3 / 4 | `MnDuluthGym` |
| Clinic | `(37,11)` | `(37,12)` | 5 / 6 | `MnDuluthPokecenter1F` |
| Mart | `(48,19)` | `(48,20)` | 7 / 8 | `MnDuluthMart` |
| Harbor home | `(7,31)` | `(7,32)` | 9 / 10 | `MnDuluthHarborHouse` |
| Ore history | `(19,29)` | `(19,30)` | 11 / 12 | `MnDuluthHistoryHouse` |
| Receiver workshop | `(48,35)` | `(48,36)` | 13 / 14 | `MnDuluthHarborWorkshop` |

| Room | Metatiles | Passive FLOOR arrival, index 3 | Carpets, indices 1 and 2 |
| --- | --- | --- | --- |
| Clinic | 10×8 | `(10,13)` | `(10,15)`, `(11,15)` |
| Mart | 6×4 | `(6,5)` | `(6,7)`, `(7,7)` |
| Gym | 12×12 | `(12,21)` | `(12,23)`, `(13,23)` |
| Workshop | 8×6 | `(8,9)` | `(8,11)`, `(9,11)` |
| Hill home | 5×4 | `(4,5)` | `(3,7)`, `(4,7)` |
| Harbor home | 5×4 | `(3,5)` | `(2,7)`, `(3,7)` |
| Ore history | 6×5 | `(4,7)` | `(4,9)`, `(5,9)` |

The first four dimensions, arrivals and first exit triggers retain the shared
`interiors.json` reservations exactly. The three household rooms are explicit
new city-local source reservations; Root must register them in shared sources
when integrating this candidate. Their furniture/layouts differ. The history
room's single ore sample at `(8,4)` is a visible exhibit with an approach at
`(7,4)`, rather than a decorative rock wall around the room.

The nurse at `(7,3)` has a real counter approach `(7,5)` through `(7,4)`; the
PC is `(13,3)` with approach `(13,4)`. The clerk at `(1,3)` has approach
`(3,3)` through counter `(2,3)`. Authored stock is Ultra Ball, Hyper Potion,
Full Heal, Revive and Max Repel. Canonical healing, PC, blackout recovery,
item pricing, money deduction and inventory mutation remain native work.

## Terrain and interaction contract

- **Downhill shelf:** native `57` at metatiles `(6,10)`, `(7,10)`, `(8,10)`.
  All six HOP_DOWN quadrants have reachable FLOOR above, blocked reverse
  quadrants and reachable landings. Representative approach `(14,19)` hops to
  `(14,22)` through `(14,20)`. The ordinary return is 47 source-graph steps
  through the east ramp at metatile `(12,8)`, using native `74`; removing the
  core passage `[24,26) × [16,18)` disconnects that uphill return. No other
  incidental HOP or Cut quadrants occur.
- **Canal Park bridge:** metatiles `x=16..19`, north row `y=15` uses native
  steel-rail `41`, south row `y=16` uses `49`. The two clear walking rows are
  core `y=31` and `y=32`; the corresponding outer rows block movement/Surf.
  This is a fixed lowered city bridge. A lifting animation, ship schedule,
  vessel movement or custom bridge system is not installed or promised by the
  terrain preview.
- **Public pier:** source `44` on `(7,20)/(7,21)`, `46` on
  `(8,20)/(8,21)`, closed native rocks at `(7,22)/(8,22)`. Approach `(15,39)`
  and end `(15,43)` remain reachable with the fisher standing at `(15,42)`;
  the other deck lane lets the player pass.
- **Low launch:** representative bank `(31,37)`, water `(32,37)`. The complete
  allowed edge list is in `harbor.allowed_surf_edges`: six bank/water pairs
  within one cove, not unrestricted Surf along every lawn shore. Island,
  bridge, dock and lookout rails block all additional FLOOR-to-WATER edges.
- **Water limit:** native `34/36/38/39/3a/32` rocks close the west, east and
  south boundaries. No reachable WATER core touches any outer edge. The lake
  has no claimed regional Surf connection.
- **Superior lookout:** the signed dead-end has a protected shore, and the
  camera at `(50,26)` shows the actual lake. It is a view/read destination,
  not a hidden item or invented progression requirement.

All exterior ordinary walking cells belong to the public graph, allowing for
NPC occupancy and door stops; there are no isolated walkable lawn/facade nooks.
This static proof does not simulate native ice sliding or substitute for
production movement/controller proof.

## Norek and the receiver story

Norek's room is now a **reference** to `MnDuluthGym` in
[`planning/minnesota/gyms/interiors.json`](../gyms/interiors.json). That canonical
object owns every glyph recipe, row, gate, actor, trainer party and move, station
choice, prerequisite, badge and reward. No alternative functional Gym8 rows,
actors or parties remain in the city dossier. The reference is pinned by the
normalized individual Gym8 object SHA; the whole collection-file SHA is only
informational. Changing an unrelated gym does not invalidate this room.

`gym_art::install` assigns glyph blocks in sorted character order starting at
zero. This exact mapping, including duplicate recipes kept as distinct symbols,
is recorded in `source_reference.symbol_blocks`. The canonical source currently
has the leader at `(12,3)`, three trainers (LIVRA, TORDEL, ANVER), two bay stations,
and the native guide at `(10,20)`. Its badge is `ENGINE_RISINGBADGE`, its reward
is `TM_ICY_WIND`, and its explicit party/moves resolve directly from the shared
object. The old city-owned open ice arena, two alternative assistants, Pryce
sprite and `(12,5)` leader placement are historical candidates only; exact old
sources and renders remain preserved in `renders/duluth/canonical-gym-reconciliation/before-02bbba/`.

The canonical order is trainer 1, WEST_BAY release, trainer 2, EAST_BAY release,
trainer 3, then the leader court. Four full source phases and six real 160×144
player cameras are saved under
[`canonical-gym-reconciliation/4e5b9dcb448129eb/`](../renders/duluth/canonical-gym-reconciliation/4e5b9dcb448129eb/).
The static planner implements the native gym test's ICE advance rule: an input
continues until ordinary FLOOR or a blocking cell/actor. Camera positions must
be reachable **resting positions** in their declared phase, not arbitrary ice
cells or occupied actor anchors. Phase events and victories are explicitly
source fixtures; these pictures do not dispatch native inputs, execute battles,
award a badge/TM or prove earned progression.

Root must preserve the existing canonical native room/art/scripts/actors and
warp indices, and rebind its review-hall destinations on indices 1, 2 and 3 to
Duluth's passive exterior warp 4. The exterior gym door remains active 3 and
targets room passive 3. No canonical module is overwritten by this source review.

Existing canonical visual concerns are recorded without changing the shared
design: repeated gray `lab/01` horizontal floor strips and
`elite_four_room/14` wall plates dominate the bays; the leader's court is broad
and largely bare; released gates have little visual distinction beyond a
wall-to-floor opening. The closed/open comparison and legal player cameras show
these issues. Matching the canonical source is not a visual-quality acceptance.

The workshop preserves `MN_HARBOR_RECEIVER` at `(11,5)` and the existing Q07
contract. It requires all eight native badges **and**
`EVENT_MN_DEPOT_CLEARED`; activation sets only
`EVENT_MN_LIGHTHOUSE_SIGNAL` and repeats working-signal text afterward. The
receiver scene must not grant badges or conflate the Grand Rapids depot flag
with St. Paul's earlier courier. Root must lower this through native commands.

The north checkpoint contract records both outbound trigger cells
`(32,2)/(33,2)`, north direction, refusal return `(32,4)`, and free inbound
access. It requires the same eight badges and signal flag. The officer at
`(33,5)` alone cannot guard both lanes: Root must install the actual coordinate
events / guarded travel before describing the checkpoint as functioning.

## Review loop and saved evidence

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_duluth.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_duluth.py --serve --port 8776
```

The review reads the completed ignored `minnesota-quality-review.crystalpack`;
`--pack` may select another completed pack carrying the unchanged source art.
The earlier ordinary-room render measured 0.9–2.9 seconds after pack load;
the canonical reconciliation additionally checks ICE resting paths and saves
four puzzle phases. Its measured full pass is recorded in the source proof.
All whole terrain, RGBA/RGB images, player crops, enlargements, sprites/palette
stages, room images and rejected passes are retained through `render_archive`
and `render_stages`, including original files before conversion. Exact JSON
snapshots are under `renders/duluth/source-snapshots/` with full SHA names.
Nothing here exports a pack or duplicates the game's source catalog.

The author visually inspected native Olivine/Blackthorn and original tile
atlases, the complete city, all rooms, and twelve true 160×144 source player
cameras. The first draft had vacant garden shoulders and a rectangular crest;
connected pine screens, a contoured crest and a tended rear garden replaced
those weak areas. The first workshop incorrectly used a rock-display metatile
as an outer wall; the saved rejected render shows that error, and the current
room uses authentic wall/furniture grammar. The three house rooms were then
differentiated. Native metal railings replaced the initial wooden bridge rails.
A final shoreline pass closed all unintended launch points outside the low
cove. The final civic-crossing pass extends one attached headland canopy at metatiles `(17,11)/(17,12)`, defining the main crossing without changing its streets or filling it with props. The [same-position before/after camera](../renders/duluth/civic-crossing-review/before-after.png) records that correction. These are source-art critiques, not native-engine acceptance.

The source proof checks all original ports, seven native facade door
pairs, all room carpets/returns/services approaches, six native ledge quadrants,
ordinary ramp return, real rails and the entire exterior Surf perimeter. Twelve original city adversarial changes are rejected: false door, unsafe return, changed port,
outer Surf hole, bridge rail hole, pier rail hole, reverse ledge hole, blocked
room arrival, wrong carpet return, missing receiver prerequisites, isolated
floor island, and unplanned island Surf launch. Canonical reconciliation also
rejects stale Gym8 object hashes, alternative city-owned rows, wrong native
glyph allocation, occupied player cameras, and a changed Gym8 source object.
An unrelated gym change and an informational whole-file hash change are
explicitly accepted. None of these test counts establishes visual acceptance.

Before accepting native city play, Root must publish the exterior/six ordinary
rooms and rebind the existing canonical Gym8, preserving its module behavior
and all unrelated pack fields/art/audio. Compare same-position native captures,
then drive every door/carpet/return, both regional interfaces, the downhill hop
and denied reverse, ordinary uphill route, launch and closed shores, all three
canonical assistant battles, Norek loss/victory/repeat, receiver refusal/activation/repeat,
all north-guard refusal cases, healing/PC/shop/recovery, save/Continue and clean
MCP input through `VisibleShellController`. Explicit fixture party/badge setup
must remain labeled; it does not prove earned regional progression.

Frozen source handoff after canonical Gym8 reconciliation:

- Exterior layout SHA-256 (unchanged): `213a8b6dc5f38c07fb89ca990cb70dab130e6e3abeb09418ec4c5bf86d9ef06b`
- Dossier SHA-256 after first reconciliation (retained): `d0d04e74fc238300994e31016ab2b2501a7175b3772229dc210ea4383cec0fc0`
- Canonical Gym8 object SHA-256 after first reconciliation (retained): `4e5b9dcb448129eb511e1af023f0ecdf763ca1521e425d31d5cd25ec8d31e28d`
- Immutable source proof SHA-256: `862f0e632b4a6438f28ed267424f03ca536c82ff45d6d33254af76c9b969edb3`
- Source art pack identity: `5fe3071c6242e932096912bb3195982ff5ce7e5bd4f821a94985e9d652e50424`
- Earlier 02bbba dossier SHA-256 (preserved): `d6613dace7a8cc77279c61ee0f887c706fb2c6afffb33f3c5497241a257ef400`

The whole exterior remains pixel- and byte-identical as `whole-town-02bbba861c.png`.
The source proof also compares twelve encoded images for all six other rooms;
all non-Gym8 dossier fields and the exterior layout bytes match the protected
02bbba snapshots exactly. No shared canonical gym, Rust, Cargo, pack or shared
world/story/interior source was modified by this reconciliation.

## Native city builder handoff

`crates/crystal-assets/src/duluth_city_modpack.rs` now owns the typed native
builder, with focused design, event, collision, gym-validation and publication
leaves beside it. Root registers only the entry point in `lib.rs`:

```rust
mod duluth_city_modpack;
pub use duluth_city_modpack::build_duluth_city_modpack;
```

The integration call is:

```rust
let city = crystal_assets::build_duluth_city_modpack(
    &completed_pack_with_canonical_gyms,
    &std::fs::read_to_string("planning/minnesota/towns/MnDuluth.json")?,
    &std::fs::read_to_string("planning/minnesota/towns/MnDuluth.layout.json")?,
    &std::fs::read_to_string("planning/minnesota/gyms/interiors.json")?,
)?;
```

The function verifies its borrowed input, returns a new verified in-memory
`CompiledGamePack`, and never exports source or writes a pack. Group **68** and
`MAPGROUP_MINNESOTA_DULUTH` must be free; an existing map/constant/landmark or
manifest collision is rejected. The seven new maps are the complete exterior
and six ordinary rooms. Gym8 stays in its existing canonical group. Its entire
module remains identical except all three warp destinations and the matching
raw event arguments now target `MN_DULUTH` passive warp **4**. The builder
requires the canonical native room, exact individual Gym8 object digest, native
recipe pixels/collision, actors, parties, badge and TM to be present already.
It does not regenerate a missing gym or invalidate it when another gym changes.

The exterior uses the original `johto_modern` tileset directly; `mn_duluth` is
the source editor alias, not a duplicate native art catalog. Ordinary interiors
compile their authored room rows. All seven active exterior doors target room
passive **3**; fourteen carpet exits resolve to the paired exterior FLOOR
returns. Every existing recovery spawn, including HOME, remains unchanged.
Native collision validation checks connected ordinary floor/door approaches,
actual facade DOOR quadrants, safe returns, all six one-way ledge quadrants,
reachable room services and PC, the six intended cove launch edges, and the
closed outer walking/Surf boundary.

`MN_DULUTH_NURSE` lowers to production `HealParty`; the authored PC uses the
existing native PC collision and `PCScript`. `MN_DULUTH_MART_CLERK` opens a
standard native mart with Ultra Ball, Hyper Potion, Full Heal, Revive and Max
Repel. The workshop receiver at `(11,5)` checks the eight native Johto badge
engine flags and `EVENT_MN_DEPOT_CLEARED`, then sets only
`EVENT_MN_LIGHTHOUSE_SIGNAL`; later interactions use the working-signal text.
Ordinary actor and sign scripts use a native map-name prefix because split-pack
script labels are global. Source object identifiers remain unchanged, including
the two distinct city/clinic actors both named `MN_DULUTH_CLINIC_VISITOR`.
The receiver, nurse and clerk retain their named native entry points.
Installation allocates missing event identifiers without granting those events
or any badge to a player. The receiver retains the dossier's exact source
prerequisites, result and location.

The native work deliberately leaves these integration tasks explicit:

- Root owns reciprocal R10/R11 travel and the actual two-lane north checkpoint.
  A dialogue officer is not an outbound travel guard, and a facing-only tile
  event can be bypassed by stepping in and reversing direction. Bind and test
  the shared travel system before claiming that checkpoint works.
- The new clinic does not become a blackout/recovery destination. Root must
  install and test the shared recovery integration without replacing HOME.
- No lift bridge animation, ship timetable, Duluth train stop, or custom PC
  system was introduced. The bridge is the native static railed canal crossing.
- Assets and direct core fixtures do not prove an earned journey or production
  controller play. Root must publish the combined pack, run the existing
  `VisibleShellController`/MCP regressions, test the city interactions and
  save/Continue, and compare same-state native renders to the saved source art.

The focused tests are in `crates/crystal-assets/tests/duluth_city.rs`. Their
default fixture reads the completed external `content-packs/text-tui.crystalpack`
and builds the canonical gyms **in memory** from the same captured collection
JSON passed to the city builder. This fixture setup never writes a pack and is
separate from the city builder, which still requires an already installed exact
canonical Gym8. Set `CRYSTAL_DULUTH_TEST_PACK` to test a supplied completed gym
pack strictly, instead of constructing the default fixture. An older external
gym pack can correctly fail when the canonical source has evolved.
`CRYSTAL_DULUTH_GYMS_SOURCE` selects an exact captured canonical collection
JSON for reproducible tests while shared authoring continues. The city builder
still validates its selected Gym8 object digest against the authored reference.
Run from the repository root with the pinned toolchain and an agent-specific
Cargo target (the root integrator may instead reuse its own warm target):

```sh
CARGO_TARGET_DIR=target/agent-duluth-city cargo test --locked -p crystal-assets --test duluth_city
CARGO_TARGET_DIR=target/agent-duluth-city cargo clippy --locked -p crystal-assets --all-targets
cargo fmt --all -- --check
```

The seven tests assert additive content/art/audio/recovery preservation, exact
Gym8 module preservation apart from destinations, seven actual door and fourteen
carpet resolutions, a real native downhill hop with denied reverse and a full
ordinary uphill walk, native HP/status/PP healing, actual mart prices/inventory
and insufficient-money behavior, all receiver prerequisite/result commands,
and rejection of false doors, unsafe returns, missing prerequisites, stale or
alternative gym sources and an outer Surf hole. A guide-only canonical edit
with a newly valid source digest still rejects an old native program: validation
constructs and discards an expected complete module and trainer catalog fixture,
then compares them to the supplied installation. None of the fixture module,
trainers or events replaces production content in the returned pack. Warp tests construct explicit
core placement fixtures; healing uses an explicitly damaged legal partner;
these are not production input journeys. No fixture awards prerequisite
progression as evidence of an earned regional campaign.


## Canonical Gym8 source drift during native integration

The shared canonical authority changed while native city work was underway.
The city source now references object `c809cf8ffa6763fa78b65311e360ebf9c144c323770ddee02d4fb4fd533c1aa1`.
Its dimensions, anchor points, party, reward and puzzle order remain unchanged;
its art changes to the tower floor/wall vocabulary, adds `B` barrier terrain,
narrows the upper court and replaces two station sailors with `SPRITE_PAPER`
console objects. Native glyph allocation now follows sorted
`# . B I L R V X Y Z r s` at blocks `00` through `0b`.

The prior `4e5b9dcb448129eb` source object, all images and the earlier city
source are retained. The current reference and informational collection hash
are the only changed city data; every exterior and other-room pixel/field
remains identical. New four-phase images and six legal 160×144 resting cameras
are under
[`canonical-gym-reconciliation/c809cf8ffa6763fa/`](../renders/duluth/canonical-gym-reconciliation/c809cf8ffa6763fa/).
The author inspected its complete closed/open comparison and all six cameras.
The new barrier/wood grammar distinguishes stations and narrow court access;
large untextured dark regions and the repeated striped floor remain canonical
visual choices, not city-authored improvements or controller acceptance.

Current dossier SHA-256: `5088ccb7d4bed3ad87a631f582f2a1cea6cdf400c2334179f3d89873438797c8`.
Current immutable source proof SHA-256: `3ed67cc884c89293941a69fa767ca990be126abf42e02c2e55959025be4a0ff5`.
The full source review passed in 1.15 seconds after pack load, retaining 17
negative rejections, the accepted unrelated-gym/informational-file mutation,
all 15 byte-identical non-Gym8 images, and the unchanged exterior layout.


A second canonical update added authored leader/assistant dialogue and changed
Norek's sprite to `SPRITE_PRYCE` and assistants to female Cooltrainer, Sailor,
female Cooltrainer. The latest accepted Gym8 object is
`c809cf8ffa6763fa78b65311e360ebf9c144c323770ddee02d4fb4fd533c1aa1`;
latest city dossier is
`4bb2805abfedaa927182dee89f1646aa5b46bc709a0e86658c7c531cebf0d21b`;
latest source proof is
`b62d7dc6ad4dc0c63b0db956ccbffc6d364b2419663d3c19a979ca878fc46fa4`.
The author inspected the new complete phase comparison and all six resting
cameras under
[`canonical-gym-reconciliation/c809cf8ffa6763fa/`](../renders/duluth/canonical-gym-reconciliation/c809cf8ffa6763fa/).
The preceding `5de9ea3a34ecbce8` source/images and before-update city snapshot
are retained. Its source review passed in 9.244 seconds under concurrent builds,
with unchanged exterior and ordinary-room pixel/field preservation checks.
Final publication must capture a matching town source and canonical collection
rather than mixing revisions while shared authoring is active.


Root's common canonical collection capture for final city integration is
`.local/minnesota-city-source-freeze/gyms-b3f25b09e4ce114fd561ff28309464fb9e45500bb159dcb6d145ce2f06f7bbbb.json`.
It contains the selected Gym8 `c809cf8f…` object and is explicitly a frozen
review input, without replacing the shared canonical authority. Exact authored
city/layout copies are retained at
`renders/duluth/source-snapshots/MnDuluth-4bb2805abfedaa927182dee89f1646aa5b46bc709a0e86658c7c531cebf0d21b.json`
and
`renders/duluth/source-snapshots/MnDuluth.layout-213a8b6dc5f38c07fb89ca990cb70dab130e6e3abeb09418ec4c5bf86d9ef06b.json`.
Do not silently update these source references from a later live shared write.

Final Duluth render-retention audit verified **1,473 valid PNG files** and
**612 current image aliases**. Every current image has a byte-identical
full-SHA-256 history copy; every historical filename digest matches its actual
encoded bytes. Original modes, palette stages, conversions, crops, previews
and prior rejected renders remain available for reuse.

## Production controller test handoff

`crates/crystal-bevy/src/bevy_shell/tests/duluth_city.rs` adds six ignored
production-controller cases. Root owns its `mod duluth_city;` registration in
`tests.rs` and publication of the matching external city pack. Use
`CRYSTAL_DULUTH_CITY_CONTROLLER_PACK` or the default
`content-packs/minnesota-cities-review.crystalpack`, then run the ignored tests
with filter `duluth_city_controller` in the existing `crystal-bevy` harness.
These cases have been authored and scoped formatted; execution remains pending
the compiled shared native city candidate.

The cases drive seven paired doors and fourteen carpets, passive arrival
stability, the native downhill hop and blocked reverse followed by ordinary
uphill walking, nurse HP/status/PP restoration, PC boot and actual deposit /
withdrawal state changes, actual standard-mart quantity/confirmation checkout
and price/inventory mutation, all eight individual missing-badge receiver
refusals plus the missing-depot case, and successful signal-only activation,
repeat text and Save/Continue persistence. They enter the real PC/shop through
map interaction. They do not seed menu cursors, call service mutators, or
rewrite position/party/flags after the initial fixture. Explicit fixture
partners, injury, money, starting position and upstream badges/depot do not
represent earned campaign progression. Canonical Gym8 battles and puzzle
progression have their separate existing production harness.

## Native bundle freeze pending root verification

The latest executed asset test binary passed its stale-canonical-program
rejection and exposed the global duplicate visitor script label on the valid
build path. That label issue is corrected in the frozen source: ordinary
interactions are map-scoped and both original visitor objects remain. The
corrected asset suite and six controller cases still require Root's next shared
compile and matching published city candidate; no passing result is claimed for
that unexecuted snapshot. Root is integrating the separate shared recovery-table
extension before the next compile. The controller module is registered by Root.
Scoped rustfmt checks pass. The attempted workspace `cargo fmt --all -- --check`
fails on extensive pre-existing formatting outside these owned leaves; none of
those unrelated files was reformatted.

Frozen native/code inputs (SHA-256):

- `crates/crystal-assets/src/duluth_city_design.rs`: `7c0c1d7aa8edaaeb430ded13384e4b2ab6c5c97788c7e0b65ad0cce0246cbdf0`
- `crates/crystal-assets/src/duluth_city_events.rs`: `4b27d092f69145ee10e5cf7e66760a677ad7cea2fd647ff1d8631eda2443a53c`
- `crates/crystal-assets/src/duluth_city_geometry.rs`: `47fb022b717eedff5178e283a106369b319261410edec089f1bd09c2240ead4c`
- `crates/crystal-assets/src/duluth_city_gym.rs`: `56bc84fb76cfb73287abe75dcea4f652ad7d92fc69437ca4a2cd22e7f8e9bea2`
- `crates/crystal-assets/src/duluth_city_modpack.rs`: `33590c2bf6d5ebd3f4d62f4293bac29e1f0cb5fd9b9e5ec24aa56ee53a601c66`
- `crates/crystal-assets/src/duluth_city_publication.rs`: `8cdaf5777eb65e9bc38e1411457189904cc1caf8658a7476d5b51d9aac8fa223`
- `crates/crystal-assets/tests/duluth_city.rs`: `96ae652830a3395a23a0157d6c94ba7c65834f01c4c7a834cfe324eeed7cfeb6`
- `crates/crystal-bevy/src/bevy_shell/tests/duluth_city.rs`: `eac9e5d34484e6564e49b333326687173faf8511225b07a1e7caf1c7cd57b3fd`

## Frozen canonical epoch 887a / Gym8 c155

Root supplied a new exact canonical collection capture, without migrating or
stripping fields from the obsolete `b3f25b09…` input. The new Gym8 object is
`c1554f9d93d56117086ad978f6150c923bb877c336785e6e3ff30b07de9360ee`.
It retains 12×12 dimensions, the same art/rows/parties, arrival `(12,21)` and
carpets `(12,23)/(13,23)`. It has **no stations**: gates now open directly on
trainer1/2 victories, and the affected trainers' aftermath dialogue changed.
This is the supplied canonical authority, not a city-authored puzzle redesign.

A new isolated city candidate was derived from exact historical dossier `4bb`:

- Publisher input: `.local/minnesota-city-source-freeze/MnDuluth-932ac34a722a1600a92f90933734efe357bdf86732ad87f21759bf1360eacf6a.json`
- City SHA-256: `932ac34a722a1600a92f90933734efe357bdf86732ad87f21759bf1360eacf6a`
- Canonical input: `.local/minnesota-city-source-freeze/gyms-887afc43d19bc0573ca5ec361f4c141771347421040f70f100f02fea4997a189.json`
- Layout input: `planning/minnesota/renders/duluth/source-snapshots/MnDuluth.layout-213a8b6dc5f38c07fb89ca990cb70dab130e6e3abeb09418ec4c5bf86d9ef06b.json`

Only the Gym8 canonical object/file reference hashes and two camera phase
anchors differ from `4bb`: `east-bay` now follows `MN_GYM_8_TRAINER_1` and
`central-dock` follows `MN_GYM_8_TRAINER_2`. All actual camera coordinates,
sorted native glyph mappings, arrival/exit anchors, exterior and non-Gym8 fields
remain unchanged. A normalized comparison proved that narrow change set and
retained the new city bytes in `renders/duluth/source-snapshots/` as well.
Every prior `4bb`/`b3` snapshot and render remains. No live city JSON was replaced.
The earlier phase renders remain historical evidence and are not relabeled as
current c155 controller or visual proof.

All seven asset tests now read the same captured strings through these three
environment overrides: `CRYSTAL_DULUTH_CITY_SOURCE`,
`CRYSTAL_DULUTH_LAYOUT_SOURCE`, `CRYSTAL_DULUTH_GYMS_SOURCE`. Set them to the
three exact publisher inputs above. `CRYSTAL_DULUTH_TEST_PACK`, when supplied,
still requires an already installed matching canonical native gym; otherwise
the test fixture constructs that gym pack in memory from the captured collection.
Native city leaves were not changed for this handoff. Root owns Cargo/test
execution and pack publication; this new frozen snapshot is not claimed tested
until its actual seven-case run completes.

## Pending clinic recovery integration — exact implementation plan

**Documentation only, 2026-10-26. No recovery implementation was changed.**
Root reports all 17 frozen-887 city asset tests and the repaired written-pack 81
checks passing. Those results preserve the current contract; they do not add
Duluth recovery. The inspected 498-map pack now has file SHA256
`37a09cd52d95b7425711237a4014ec387555a27cc3d4df61e246581b4d108cf6`.
It contains Bemidji recovery records 256/257 but **no Duluth recovery records**.
`MN_DULUTH_NURSE` contains `HealParty` and no `blackoutmod`.

The live town dossier (`4bb2805a…`) and frozen publisher candidate (`932ac34a…`)
agree: clinic `recovery_contract.spawn_tile` is `(10,13)` and its status says
registration is currently uninstalled. The broad `source_behavior` promise
does not override that explicit implementation status. Existing local nurse,
PC and shop controller cases do not exercise recovery.

The missing path is traced through these native boundaries:

- `duluth_city_publication::publish` inserts an extension with a temporary
  spawn record, then deletes that record for **every** city/room. It preserves
  all input spawns, but leaves no persistent Duluth destination.
- `GameDataSet::resolve_warp_transition_with_state` automatically remembers
  the **exterior source constant** when a TOWN/ROUTE warp enters a pokecenter.
  Thus actual clinic entry remembers `MN_DULUTH`; it does not automatically
  remember the indoor clinic or register either map in the spawn catalog.
- `ResolveBlackoutToLastSpawn` falls back to the original HOME when that saved
  constant has no catalog record. Teleport instead reports that the saved map
  is absent from SpawnPoints. `blackoutmod` alone only sets the saved constant;
  it validates the map, not recovery membership. Both records and the nurse
  command are necessary.

Use the installed pattern in
[`bemidji_city_modpack.rs`](../../../crates/crystal-assets/src/bemidji_city_modpack.rs)
and its [asset tests](../../../crates/crystal-assets/tests/bemidji_city.rs).
The shared runtime extension already accepts addon recovery identifiers from
`FIRST_ADDON_RECOVERY_SPAWN = 256` through 65535; original source identifiers
remain 0–27, and 28–255 remain a reserved invalid gap. Do not change that policy
or enlarge Fly's source range for Duluth.

Implement only the following bounded change when Root approves the next code
snapshot:

1. Retain exactly **two** new `RuntimeSpawnPoint` records, with dynamic checked
   allocation `max(max_existing_identifier + 1, FIRST_ADDON_RECOVERY_SPAWN)`
   and the following identifier. Reject collisions, duplicate map-constant
   rows and overflow. Do not hardcode 256/257: Bemidji already owns them in
   this completed pack, and publication order can change. All temporary
   extension records for other rooms must still be removed. If temporary
   allocation is retained, use the same legal addon namespace and ensure its
   cleanup cannot delete either persistent row.

   | Field | Exterior fallback after clinic entry | Indoor recovery after healing |
   |---|---|---|
   | Map name / constant | `MnDuluth` / `MN_DULUTH` | `MnDuluthPokecenter1F` / `MN_DULUTH_POKECENTER_1F` |
   | Group name / ID, map ID | `MINNESOTA_DULUTH`, 68, 1 | `MINNESOTA_DULUTH`, 68, 2 |
   | Core tile | **`(37,12)`** | **`(10,13)`**, exact source recovery contract |
   | Metatile / subtile | `(18,6)` / `(1,0)` | `(5,6)` / `(0,1)` |
   | Existing passive arrival | City warp 6 | Clinic warp 3 |

   Both actual published positions are unoccupied ordinary FLOOR. The exterior
   point is directly below the clinic door `(37,11)`; the interior is two rows
   above carpets `(10,15)/(11,15)`. Their passive records do not themselves
   activate a door warp. The current temporary exterior extension anchor
   `(6,24)` is a west-route landing, **not** the recommended clinic recovery
   point; do not accidentally retain it as the town's final row.
2. In `duluth_city_events::install`, add native
   `blackoutmod MN_DULUTH_POKECENTER_1F` immediately **after** `special HealParty`
   and before `closetext` in `MN_DULUTH_NURSE`. Preserve dialogue, actor `(7,3)`,
   counter `(7,4)`, approach `(7,5)`, PC and all room rows/warps. Lower the
   changed script through the existing native finish/publication path once.
   Do not set the last-spawn constant on pack installation or arbitrary city
   entry. Actual clinic entry remembers the exterior; actual completed nurse
   service changes it to the indoor recovery constant.
3. Revise the focused preservation test's whole-table equality into equality
   for **every original row**, plus exactly two new checked rows. Keep HOME,
   title-new-game spawn/metadata and all preexisting addon rows byte-identical.
   Keep `fly_destinations` identical, including its currently empty table;
   adding recovery must not create a Fly destination/flag or a new-game start.
   Update builder API comments/status only after implementation and actual
   verification, preserving earlier results as history.

Required acceptance before marking this recovery contract installed:

- Asset proof must resolve each new row by identifier and map constant, check
  group/map metadata and coordinate decomposition, exact saved-reference
  validation, unoccupied Floor, their passive arrival records and both real clinic exits.
  Exercise real pending Teleport preparation/commit and blackout resolution to
  both destinations. Retain unknown-last-spawn HOME fallback, saved addon
  identity, original 28-row semantics and rejection of gap 28/255. Preserve Fly
  parser positives 0/27 and reject 28/255, either addon ID and 65535 as source Fly
  destinations. Run strict written-pack readback as well.
- Add production-controller recovery tests following
  [Bemidji's actual loss/Teleport cases](../../../crates/crystal-bevy/src/bevy_shell/tests/bemidji_city.rs).
  Start with explicit legal low-level partner/money/injury fixtures. Walk
  through Duluth clinic door `(37,11)` and assert exterior memory before any
  nurse conversation. Run two branches, without healing and after the actual
  nurse dialogue; assert respective saved constants. Exit via the real carpets,
  enter canonical Gym8, reach LIVRA at `(8,8)` through actual ice inputs and lose
  an actual battle using a legal Splash-only partner. Do not faint a party or
  rewrite last-spawn/position after setup. Assert real recovery to `(37,12)`
  exterior or `(10,13)` clinic, native money halving, restored HP/status/PP,
  unchanged bag/badges/story, no trainer 1 victory/gate flag, no badge 8, no
  interrupted victory-script continuation and no passive-warp bounce.
- In a separate fixture, walk an Abra with learned Teleport into the clinic,
  complete the nurse service, exit to the town, then select Teleport through
  the production Start→Pokémon→party action menus. Assert actual indoor arrival,
  preserved party/PP/bag/money and last-spawn memory. Save/Continue both recovery
  branches and Teleport, verify exact location and continued safe walking.
  These are explicit local fixtures, not earned regional or gym progression.

No Rust, Cargo, controller test or pack publication was changed for this plan.
The existing six Duluth controller cases still need Root's current compile/run;
the additional recovery cases remain **unauthored and unexecuted**.
