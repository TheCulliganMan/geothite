# Brainerd clinic recovery — exact implementation plan

**Documentation only; not implemented or controller-verified.** The current
repaired498-map candidate, file SHA256
`37a09cd52d95b7425711237a4014ec387555a27cc3d4df61e246581b4d108cf6`,
passes the written-pack81 checks and preserves the source28 recovery rows plus
Bemidji256/257. Brainerd still has no retained recovery entry. Healing and PC
service code exists; this plan does not claim Brainerd blackout or Teleport works.

## Source trace and concrete gap

- `brainerd_city_publication::publish` currently gives every new map a temporary
  insertion spawn and immediately removes that exact row. Its town insertion
  anchor32,44 is a builder staging coordinate, **not the clinic recovery floor**.
- `brainerd_city_scripts::people` nurse flow runs faceplayer, opentext, writetext,
  waitbutton, special HealParty, closetext, end. It has no blackoutmod command.
- `game_data_encounters_and_loading.rs` records the exterior source constant on a
  real TOWN/ROUTE→pokecenter-tileset warp. Entering this clinic therefore remembers
  MN_BRAINERD even before nurse interaction. Exiting the clinic does not undo it.
- `game_data_scripts.rs` executes native blackoutmod by validating its map
  argument and updating `last_spawn_map_constant`. The lower-level script opcode
  alone does not register a recovery destination or perform travel.
- `game_data_mutations.rs` blackout resolves that remembered constant through the
  actual spawn catalog, falling back to the original HOME when no row exists;
  `game_data_items_and_battles.rs` Teleport instead reports a missing saved spawn
  when the remembered map has no catalog row. These are source-derived
  consequences of the current gap, not a newly executed Brainerd gameplay proof.

Reuse the existing Bemidji namespace and native parser/resolver behavior; do not
add a frontend-specific blackout/Teleport dispatcher or change core Fly limits.

## Exactly two legal persistent destinations

| Field | Town fallback before nurse | Clinic after nurse |
|---|---|---|
|map name|MnBrainerd|MnBrainerdPokecenter1F|
|map constant|MN_BRAINERD|MN_BRAINERD_POKECENTER_1F|
|group name / group ID|MINNESOTA_BRAINERD_CITY /69|MINNESOTA_BRAINERD_CITY /69|
|map ID|1|2|
|core tile|9,22|10,13|
|metatile|4,11|5,6|
|subtile|1,0|0,1|
|existing safety contract|Exterior passive warp2, FLOOR directly below active clinic DOOR9,21|Room passive warp3, FLOOR above carpets10,15/11,15|

Use the authored clinic door’s `return_tile` and room’s `passive_arrival.tile`,
not copied constants scattered through the builder. Validate actual module
FLOOR, bounds, actor occupancy, passive warp index and ordinary walking to public
care at both destinations. Waited frames at either arrival must not bounce through
an active door/carpet or resume an interrupted gym script. Neither the gym,
MART, ranger room nor outfitter gets a retained spawn.

## Allocation and publication

Use `FIRST_ADDON_RECOVERY_SPAWN` from `crystal_core::systems::special_routines`
(currently256). Source IDs0–27 remain unchanged;28–255 stays invalid. Use u16
checked arithmetic and decimal catalog keys, with each key exactly matching its
row’s identifier. No identifier, map binding, source row or new-city record may
be overwritten.

1. Validate the entire supplied spawn catalog first. Deny duplicate/mismatched
   numeric keys, duplicate map/group bindings, already-owned Brainerd recovery
   constants, namespace exhaustion and inconsistent map metadata. Compute
   `first = max(256, checked(max_existing_identifier + 1))`; if an allowed input
   has no rows, use256 explicitly rather than wrapping an absent maximum.
2. Reserve two free checked IDs for town and clinic. Confirm keys, identifiers,
   map constants and (group69,map1/2) are unused in the supplied recovery catalog.
   Keep all source/new-city entries exactly. On the current repaired498 input,
   Bemidji256/257 must survive, so the next pair would be258/259. A fresh rebuild
   starting at480 and publishing Brainerd first could legally assign256/257, then
   Bemidji must allocate later IDs. **These are examples, never hardcoded IDs.**
3. For a normal city rebuild, update the existing Brainerd publication path to
   retain only the town/clinic insertion rows and remove only temporary rows
   created for other rooms. Temporary staging IDs must also be free addon-range
   IDs; do not stage in the reserved28–255 gap or reuse a retained row. A small
   separately reserved temporary key can be inserted/removed for the remaining
   maps. Guard every insertion and remove only a row owned by that insertion.
4. If Root instead applies recovery to the already-installed498 candidate, use a
   narrowly scoped additive amendment after all cities are installed. Do not call
   `build_brainerd_city_modpack` again: its existing group/map collision guard
   correctly rejects reinstallation. The amendment must validate the expected
   current town/clinic, nurse script and absent recovery bindings, allocate after
   all current rows, then alter only the two new rows and nurse lowering/mirror.
   Do not silently choose this second API while implementing the first.
5. Work on an owned clone; failed validation returns without mutating input.
   Publish new identity/manifests through the existing Rust path and strictly
   reload the resulting external pack. Preserve HOME, start-new-game destination,
   all original28 rows, any previously installed city rows, Fly payloads, parties,
   flags, audio, existing doors and the canonical Gym6 module.

The current81-check proof expects precisely two additions because only Bemidji
currently contributes recovery. After this authorized feature, add a new
preservation expectation for exactly the two Brainerd rows relative to its actual
input; a fresh480→all-cities rebuild should have four total city rows. Do not
weaken that proof by allowing arbitrary recovery additions.

## Native nurse command and split payload

After successful native HealParty, before closetext, add:

```text
blackoutmod MN_BRAINERD_POKECENTER_1F
```

Do not replace that command with a direct session-state write, force a runtime
spawn byte, or require a new story flag/item. Actual entry alone still remembers
MN_BRAINERD; actual nurse completion selects the clinic constant. The existing
native map/command boundary resolves the constant to the retained addon ID.

Rerun the normal authoring/lowering path so parsed runtime/map commands match the
source script. Refresh the clinic’s global split script/NPC payload using the
existing `insert_compiled_map_split_payloads` helper. An additive amendment must
not leave cached module scripts and global map_scripts disagreeing. Force split
reassembly in its asset test by removing the cached clinic module from a private
clone, then verify the exact nurse sequence and raw/typed commands. No room rows,
objects, facade/carpet warps or art need modification.

## Required native asset checks

- Verify both catalog entries, namespace validity, decimal keys, exact map/group
  IDs, decomposed core coordinates, FLOOR and passive safe arrival. No other new
  persistent spawn exists. All input rows and HOME are byte/logically unchanged.
- Test composition after an existing Bemidji256/257 catalog and another occupied
  addon ID, reverse city build order, duplicate Brainerd bindings/keys and u16
  exhaustion. Fail atomically instead of replacing an existing recovery record.
- Exercise the existing core Teleport prepare/commit and blackout resolver with
  both declared constants as explicitly labeled unit fixtures; preserve the
  distinction between these resolver tests and real controller acceptance below.
- Keep `fly_destinations` exactly equal to input. Recovery IDs≥256 are **not Fly
  destinations**. Even if the source Fly table is empty, run real authored Fly
  parser positive controls for0 and27 and negative controls for28,255, each new
  addon ID and65535. Do not “prove” Fly behavior from two empty tables alone.
- Keep unknown remembered-map blackout fallback to the exact original HOME and
  the reserved gap invalid. Validate save references against the actual addon
  catalog with existing u16 APIs; never narrow IDs to a source byte.
- Compare the strictly reloaded written pack: old maps/scripts/constants/files
  unchanged except the explicitly new nurse opcode and its parsed/split mirror,
  plus exactly two additive recovery rows and updated identity/report. Audio
  source programs, compiled PCM metadata/hashes and source assets remain exact.

## Production controller acceptance — input after setup

Implement these as focused additions to Brainerd’s controller tests. Initial
placement, a legal weak/Splash partner or Abra, prior five badges where needed,
money and deterministic RNG may be labeled fixtures. Do not directly set
`last_spawn_map_constant`, faint a party member, teleport coordinates or grant
victory/permission during the acceptance journey.

**Actual Gym6 loss, two nurse branches.** Start at town9,22 with a legal level5
Magikarp/Splash at full HP. An initial BURN may be a labeled status fixture;
keep Splash usable so the losing battle remains a real legal input path. Enter the
real DOOR using Up and assert clinic10,13 plus remembered MN_BRAINERD. Run both:
(1) leave without speaking to the nurse; (2) walk to7,5, face the counter and
complete actual nurse dialogue/healing. Only branch2 changes remembered constant
to MN_BRAINERD_POKECENTER_1F. Exit a real carpet, walk to Gym door12,35, enter at
the exact frozen canonical arrival, and challenge NESSA5,13 through real input.
Use SPLASH through the production battle menus until the enemy actually defeats
the party. Do not resolve loss by a test mutation.

After native blackout, branch1 must be town9,22; branch2 must be clinic10,13.
Require real money halving, healed HP/status/PP, unchanged Bag and prior badges,
no NESSA victory flag, no Badge6/TM/report/Surf grant and no surviving battle/gym
continuation. Advance120 frames to prove safe passive arrival, then Save/Continue
and verify location, party, Bag, money and remembered spawn. Existing HOME and
Bemidji recovery destinations must remain selectable and unchanged.

**Actual Party-menu Teleport.** Start at town9,22 with a native-created Abra that
legally knows TELEPORT. Enter clinic, complete the nurse interaction, exit by a
real carpet and walk to an ordinary town FLOOR away from the clinic (for example
32,24). Open production Start→POKEMON→Abra actions→TELEPORT using press/cursors,
then let the real pending field-travel boundary complete. Assert clinic10,13,
unchanged party/PP/Bag/money, correct remembered clinic and no battle/reward flag.
Wait120 frames and Save/Continue. A companion no-nurse variant should return to
town9,22, proving the distinct exterior row rather than hardcoding indoor travel.
Respect the existing Teleport source-environment restriction; do not enable it
inside the clinic or replace Party actions with a direct field-move API call.

These are local service/battle fixtures, not earned regional progress or Fly
unlock proof. Final acceptance also needs matching actual native clinic/exterior
captures and the existing facade/HealParty/PC/shop/Gym/Surf tests on the same
strictly reloaded pack. Keep every render and prior failed/passing evidence.
