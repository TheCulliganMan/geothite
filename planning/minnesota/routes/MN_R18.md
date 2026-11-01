> Current grass review: [R18 verification](../revisions/route-grass-v01/current-r18-v01/README.md).
> Five metatiles extend the western meadow across its former dry bypass: four grass
> steps each way, retaining the original six-step neck, docks and public shorewalk.
> Eight native cases, 15 production stdio calls, audio and actual 2D review pass.
> Current native Minnesota teams and encounter bindings supersede historical data below.

# Mille Lacs Shore — source authoring handoff

## Current executed native evidence — 2026-10-26

Root published `content-packs/minnesota-mille-lacs-shore.crystalpack` with **422
maps**. Exact comparison passed **960 exterior metatiles and current events** in
the two clipped shore maps. Five native asset/component suites pass; native SQ04
report/reward scripts and their allocated events are in the published pack.
The integration owner inspected the actual native marina camera. These facts
supersede earlier source-only/attachment-pending handoff notes below.

All **five production-controller tests PASS** after Root corrected native badge
reads and the STORM index4/MINERAL index5 identities. All twelve focused core
badge/flag tests also pass. Actual input now proves acknowledged observation
reports, all eight report subsets, sixth-badge denial even after all reports,
Good Rod hand-in/idempotence, existing-rod branches and Save/Continue, including
the partial-report save. No report or engine flag is injected after setup.
Initial badge/inventory/position setup remains explicit and does not prove
earned sixth-badge access.

Current audio evidence is the shared **443-map** composition: all **377** PCM
hashes, frame counts and loop ranges pass. Surf/islet loot, native fishing,
complete populated route/battle balance, regional gates and final visual
acceptance remain separate work. No duplicate audio run was performed here.


R18 is a **40×24 metatile** low-lake shore: narrow north-shore travel, a western marina with two distinct piers, three walking-access shore-watch stops, a meadow crossing and an optional offshore reward. It replaces the repeated 96×96 reservation. The broad lake is deliberate geography; the walking route has activities at its branches and four-core-tile bounds on routine shore sections. There is no copied cliff/ramp sequence, decorative Cut tree or fake boat art.

Authoritative source is [MN_R18.json](MN_R18.json) plus [MN_R18.layout.json](MN_R18.layout.json). The explicit native command arrays under `quest_contract.script_labels` are now bound to the published native SQ04 scripts and allocated events. Their current asset and production-controller evidence is recorded above.

## Review loop and visual evidence

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_mille_lacs_shore.py --self-test
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_mille_lacs_shore.py --watch --serve --port 8784
```

The page is `http://127.0.0.1:8784/`. Source edits rerender in roughly 140–185 ms warm; initial pack loading is separate. The tool reads the supplied ignored TUI crystalpack and writes only ignored `renders/mille-lacs-shore/` output. It never selects geography, exports a pack or writes a source catalog.

The reviewed whole map is [whole-route-8491f65e.png](../renders/mille-lacs-shore/whole-route-8491f65e.png), 1280×768 native source pixels. Also inspect [player frames](../renders/mille-lacs-shore/viewpoints.png), [Surf/islet frames](../renders/mille-lacs-shore/surf-boundaries.png) and [physical event anchors](../renders/mille-lacs-shore/anchors.png). Individual source frames are 160×144, enlarged nearest-neighbor. The widest routine camera, `viewport-widest_routine_shore.png`, remains an explicit review probe rather than showing only the most attractive pier.

The old entire blockout and native Route34/Route42 were rendered and viewed before authoring. Original source-pixel review caught an over-wide lawn shoulder in the first candidate, a 19-column sign line, and a real lower-pier corner where FLOOR touched WATER around its rail. The revised shore has attached woodland fingers on both sides of routine road, eighteen-column text, and a real native bank joint. The two similarly long first-pass piers were revised: registration remains the longer deck; fishing/watch uses a shorter deck and its own actors. Whole map, worst road camera, both pier cameras, beach, actual source palette and offshore frames have been viewed. Green collision checks alone do not establish visual acceptance.

## World and native authoring contract

| Native map | Metatiles | Role |
| --- | --- | --- |
| `MnR18C00R00` | 32×24 | Brainerd arrival, marina, meadow, launch and main lake |
| `MnR18C01R00` | 8×24 | Mille Lacs beach approach, eastern lake and islet portion |

Retain route `MN_R18`, prefix `MnR18`, **BADGE_6**, quest **SQ04** and the existing BRAINERD/MILLE LACS town identities. No interior is reserved or needed. Suggested publication contract: group63 if free, `MINNESOTA_MILLE_LACS_SHORE`, exterior tileset **mn_headwaters**, original Route34 music. Root owns final group assignment, shared registration, world/story changes and all pack publication. Do not mutate original Johto blocks, original maps or audio.

| End | Gate metatile | Clear landing | Core gate / landing |
| --- | --- | --- | --- |
| BRAINERD west | (1,5) | (3,5) | (2,10) / (6,10) |
| MILLE LACS east | (38,5) | (36,5) | (76,10) / (72,10) |

Root must bind the existing Brainerd east and Mille Lacs west hub ports to these exact route gate/landing records. They are warp-port contracts, not invented outer walking seams. Within the route, register reciprocal east/west chunk connections at **offset0**, including road, open water and the islet walking seam. Root must retire the ten unused old chunks, preserving trainer/event identities07–10 as unused reservations. `MN_R18_SHORTCUT` remains unused; `shortcut:null`, `cut_gates:[]`, `one_way_ledges:[]` are deliberate. This flat lake shore does not need a Cut/ledge quota. Source inventory contains zero CUT_TREE quadrants.

All terrain is hand-authored schema2 orthogonal strokes with `reject_cliff_cuts:true`. It uses original Johto plus the already implemented three-block **mn_headwaters** extension: native Kanto timber recipes80/81/82. The JSON specifies the exact source recipes and collision provenance. The review tool compares these against actual published tiles when the supplied pack contains them; a local preview definition is not native publication. Do not add a second art exporter or fake dock texture.

## Traversal and activities

- The two-way road runs west(1,5) → (6,5) → (6,6) → (24,6) → (24,5) → east(38,5). West and east routine probes have **four actual 16px walking tiles** between native woodland; the white main lane is normally two.
- A western branch follows the bank to a long registration pier and a shorter fishing/watch pier. Paths join both actual decks. The ranger, reed volunteer and two optional Fisher battles justify the marina space. A Hyper Potion and lower-bank habitat reward looking around; no placeholder facade claims an inaccessible service room.
- The meadow volunteer sits on walking ground beside a real optional encounter meadow and an Ultra Ball. Source grass remains encounter terrain; no invented reed sprites or enlarged vegetation are used.
- The shore neck **core[36,10,42,18]** is all TALL_GRASS. No free dry route bypasses it; the shortest ordinary west/east trip pays **six grass steps**. Later Surf is a real field-move alternative through the large lake, with water encounters and required permission.
- East of the neck, attached north/shore groves frame the regular road. Dory/Finn give the local meadow and beach their own battles. The eastern volunteer and Max Repel are accessible on foot. At beach probe(75,18), the 160×144 frame contains **26 WATER cells**, making the promised lake visible.
- The public Surf launch is clear FLOOR **(49,19)** facing WATER **(49,20)**, with real native bollard pieces at meta(23,9)/(25,9). It is distinct from the closed piers.
- The low islet around meta(29..32,16..19) has a Revive at **(61,35)**. It has no ordinary land connection and no story actor. Surf gives a genuine optional destination; it never gates SQ04 or town travel.

## Marina structure and closed ends

The public registration pier uses meta deck x8..14 at row11, with the closed end at x15. Fisher Leon(17,23) and dock ranger(27,23) leave a passing route. The quieter lower pier uses deck x6..9 at row17, end x10; reed volunteer(15,33) and Fisher Hal(17,35) occupy different rows. Its shorter footprint and watch/fishing role distinguish it from registration.

Each has native82 north rail,71 dry deck,81 south rail and80 water below. The walking width including the inner rail quadrants is four core tiles. Native **42 northeast /46 east /4a southeast** caps close the far end in all quadrants. The lower bank joint uses native **48 at meta(6,18)**; without it the southwest inside corner can launch Surf around the rail. Both complete deck footprints are checked to have **zero FLOOR-to-WATER adjacency** through rail/corner boundaries. Source geometry proves the intended physical closure; actual field-move denial still needs controller input.

## Exact native actors and resources

Coordinates are 16px core tiles. Dossier dialogue and parties are authoritative; trainer lines are original and fit eighteen columns. All six active trainer IDs/defeat flags retain01–06 and their three-member reserved parties, eighteen opponents total.

| Trainer | Tile / facing | Class and party |
| --- | --- | --- |
| `MN_R18_T01` LEON | (17,23) east | FISHER: Wooper26, Skiploom26, Marill25 |
| `MN_R18_T02` BELA | (19,11) south | PICNICKER: Skiploom26, Bellsprout26, Pidgeotto25 |
| `MN_R18_T03` DORY | (53,11) east | LASS: Bellsprout27, Marill27, Girafarig26 |
| `MN_R18_T04` KEMP | (11,29) south | BUG_CATCHER: Marill27, Pidgeotto27, Yanma26 |
| `MN_R18_T05` FINN | (61,13) east | SAILOR: Pidgeotto28, Girafarig28, Wooper27 |
| `MN_R18_T06` HAL | (17,35) east | FISHER: Wooper28, Yanma28, Skiploom27 |

All have sight1 and an ordinary bypass; no initial trainer sight seals mandatory travel. Flags remain `EVENT_MN_R18_TRAINER_01`..`06`;07–10 are unused rather than deleted/reassigned.

| Quest actor | Tile | Role |
| --- | --- | --- |
| `MN_R18_WATCH_REED` | (15,33) south | Lower fishing/watch pier, report01 |
| `MN_R18_WATCH_MEADOW` | (31,9) west | North meadow, report02 |
| `MN_R18_WATCH_BEACH` | (75,17) south | Eastern beach, report03 |
| `MN_R18_DOCK_RANGER` | (27,23) west | Upper public registration/reward pier |

Pickup IDs and original flags remain `MN_R18_ITEM_01`/`EVENT_MN_R18_ITEM_01` through04:

| Suffix | Tile | Reward/access |
| --- | --- | --- |
|01| (11,31) | Hyper Potion×1, walking |
|02| (27,7) | Ultra Ball×1, optional meadow |
|03| (73,9) | Max Repel×1, walking beach branch |
|04| (61,35) | Revive×1, **Surf-only**, independent of SQ04 |

Signs: WEST(4,12), WATCH(10,20), MARINA(14,28), EAST(74,12), GRASS(34,12), SURF(52,16). All native sign quadrants are WALL with an ordinary same-chunk interaction approach.

## SQ04 production script handoff — pending

The current story already promises three walking volunteers and a rod upgrade. Existing pack item **GOOD_ROD** is the selected upgrade (source item constant59, KEY_ITEM, GoodRodEffect, nonconsumable), and existing **ENGINE_MINERALBADGE** constant31 corresponds to planning BADGE_6 / Johto index5. These were inspected in the supplied pack. No new rod, fishing engine or frontend dispatcher is needed.

Root must allocate three new event names **EVENT_MN_SHORE_WATCH_01**, **02**, **03**, and the already planned completion **EVENT_MN_SHORE_WATCH_REWARDED**. These names are absent from the supplied base global constants, so merely adding them to JSON is insufficient. Register them through the existing external-pack mutation path and compiled script constants.

The dossier contains exact `script_labels` arrays using existing commands and separate eighteen-column `text_labels`:

1. Each volunteer faces the player, opens its real dialogue, waits for acknowledgment, closes text, then sets only its own report event. Repeat talks are idempotent; they never grant a rod.
2. The ranger checks native `checkflag ENGINE_MINERALBADGE`; failure shows the sixth-badge requirement. Then `checkevent` completion; true goes to the completed dialogue.
3. Each report is checked independently. Any missing report goes to the unfinished-watch dialogue, with no reward or completion flag.
4. Check existing SUPER_ROD, then GOOD_ROD, with native `checkitem`. An already adequate rod is acknowledged and completes the watch without duplicating or downgrading it. OLD_ROD remains in the bag.
5. Otherwise open text, acknowledge the introduction, `giveitem GOOD_ROD,1`, branch on failure to the room/retry text. Only successful delivery sets completion. The successful path thanks the player and closes normally. Inventory failure never consumes the opportunity.

Root should patch the four resident labels in a focused native leaf module after generic actor creation, append the text labels, then rerun production runtime/object/field-pickup script parsing and validation before finalizing the map extension. A typed quest-authoring hook is another valid integration; unparsed JSON `quest_contract` metadata alone is not. The existing generic Resident service supports only HealParty, so it currently leaves these actors as ordinary dialogue.

Test badge denial, all eight report subsets, repeated volunteers, successful once-only delivery, existing GOOD_ROD/SUPER_ROD cases, storage failure/retry, save/reload completion and dialogue ownership through `VisibleShellController`. Test actual fishing with the granted rod from a legal water-facing tile after the quest. Do not replace an item grant with a text claim.

## Water boundary and Surf-only validation

The lake spans both chunks. The entire **252-cell outer core perimeter** is checked: no WATER appears on an unconnected outer edge, and conservative combined Walk+Surf reaches **1,752 WATER cells** with zero outer WATER. Native visible rock recipes close south meta y23 across x0..39 (`38`/`39`/`3a`), east x39 at y10..22 (`32`/`36`), and west x0 at y20..22 (`30`/`34`). These are original blue-water rocks with BUOY collision, not brown land boulders.

Open water on internal core x63/64 has reciprocal native neighbors proposed at offset0. Source checks retain its exact WATER seam; actual connection metadata is inspected when those native maps exist. The islet also spans that same chunk edge and must retain real walking connection.

The generic authored-route anchor checker assumes all pickups have ordinary walking approaches. Root must add a **typed, explicit Surf-only access contract for `MN_R18_ITEM_04`**, preserving ordinary checks for every other actor/item. The local checker does not silently pretend all items are walking-access: it separately proves ITEM04 is unreachable on foot, reachable with Walk+Surf, on actual FLOOR, in the native item catalog and independent of all four walking quest actors. Source rendering still draws the actual item on the whole map. No bridge or fake walking tile may be added simply to satisfy an overly strict old validator.

## Current evidence and pending native acceptance

Current source review passes:31 block varieties,992 connected ordinary/grass core cells,140 accessible grass cells,6 trainers,4 quest residents,4 pickups,6 signs,2 closed piers. Thirteen negative fixtures reject three open outside water edges, missing rail/end/corner, Surf-only quest volunteer, free dry offshore item, removed required grass, decorative Cut, invented rod, unconnected water neighbor and an empty widened road shoulder. Python compiles; no native builder or original content was changed by this author.

Native registration, SQ04 event/script compilation, typed Surf-only anchor support, actual hub gates and all controller evidence remain pending. Root must publish through the Rust path, run `--verify-published` against the resulting ignored pack, inspect actual same-state 2D native captures, test earned BADGE_6 access, walking hub travel/volunteers, grass encounters, both piers and all corners, real Surf launch/dismount/islet pickup and buoy rejection, battle wins/losses and persistent trainer flags, once-only item grants and save/return behavior. Preserve unaffected original maps/content/audio. A source image, snapshot or coordinate change alone cannot mark R18 playable.

## Native SQ04 leaf handoff — 2026-10-26

`crates/crystal-assets/src/mille_lacs_shore_modpack.rs` and
`crates/crystal-assets/tests/mille_lacs_shore.rs` are saved, scoped-rustfmt checked,
and **not yet compiled or executed**. Root accepted group58 and owns shared
registration, the bounded Surf-only schema change, warm compilation and sole
publication. Source geometry remains frozen at whole-route-8491f65e.png.

The leaf exports `build_mille_lacs_shore_modpack`,
`SHORE_WATCH_REPORT_EVENTS` and `SHORE_WATCH_REWARDED_EVENT`. Its crate-private
`bind_shore_watch_quest(pack,modules,dossier_json)` must run after shared
`register_constants` and before native map lowering. It replaces only the four
existing generic resident roots, validates the exact reviewed command arrays
against the dossier, appends the seven native text bodies, allocates four new
collision-checked saved events, and reparses real native flag/control/text/item
and runtime/object tables. Unbound generic dialogue does not complete SQ04.

Explicit new event names are `EVENT_MN_SHORE_WATCH_01`, `_02`, `_03` and
`EVENT_MN_SHORE_WATCH_REWARDED`. All are absent from the supplied source pack;
`ENGINE_MINERALBADGE` is actually31, and all three rods are existing native,
nonconsumable KEY_ITEM definitions. Numeric event IDs allocate strictly above
the current maximum after ordinary R18 events; never hardcode source-only
numbers across composed packs or overwrite original aliases. All four names
and new numeric values are checked for collisions.

Accepted registration: group58, `MINNESOTA_MILLE_LACS_SHORE`, manifest
`minnesota-mille-lacs-shore`, landmark `LANDMARK_MN_MILLE_LACS_SHORE`, label
`MN_MILLE_LACS_SHORE`, name `MILLE LACS SHORE`, position `[88,88]`, tileset
`mn_headwaters`, music `Route35`; design tuple
`([40,24],2,1,6,4,0,"MnR18")`, four pickups/six signs/no rooms or doors,
`BADGE_6`, one required habitat/no ledge/no Cut. Add R18 to the existing native
headwaters-art fallback; no new art recipe is needed.

The bounded Surf proposal is a typed `surf_only_actor_ids` list with unique
existing pickup IDs only. R18 requires exactly `MN_R18_ITEM_04`; that item must
stand on actual FLOOR, have no ordinary walking interaction approach and have a
real walking+Surf approach. Every other actor retains normal walking validation.
Never exempt an arbitrary resident or silently drop an unreachable-anchor check.
Root owns the actual shared implementation.

Prepared tests use existing production `RuntimeMutationCommand`, script control,
flag mutation and save APIs, not a local script interpreter. They cover source
preservation, exact published native tables, badge/report denial, successful and
failed Good Rod delivery, native `iffalse` retry, existing Good/Super Rod
acknowledgment without changing inventory, original Old Rod retention, native
save/read of reports/completion and subsequent no-duplicate branch. These are
component fixtures, not dialogue/controller or earned-route acceptance.

The supplied catalog has only20 non-reward key items but25 key slots. Therefore
the full KEY_ITEM test explicitly adds five cloned-key aliases to a **test-only
cloned GameDataSet**, fills its actual native key pocket, then frees one slot and
retries. No fixture alias enters the published pack. Filling the ordinary ITEM
pocket would not test failure of a KEY_ITEM reward.

Remaining production-controller acceptance: all eight report subsets and repeated
volunteers; actual dialogue acknowledgment/input ownership before report flags;
walking access to all four quest actors; badgeless denial; successful reward,
existing rods, full key-pocket retry, Save/Continue; then actual native fishing
with the awarded Good Rod from a legal facing. Also verify the independent
Surf-only Revive, reciprocal lake/islet seam, and unchanged original recovery
and audio. Root will record executed Rust/controller results after wiring.

## Production-controller SQ04 handoff — 2026-10-26

Root has wired group58, the native SQ04 binder and the bounded typed Surf-only
pickup validator. Warm asset/component verification and external publication
are in root's queue. This updates the earlier unregistered/native-hook-pending
notes; it is **not** a report that the controller tests below have executed.

The new focused leaf is
`crates/crystal-bevy/src/bevy_shell/tests/mille_lacs_shore.rs`. It reads
`CRYSTAL_MILLE_LACS_SHORE_TEST_PACK` from the actual external
`content-packs/minnesota-mille-lacs-shore.crystalpack` once root publishes it.
It never synthesizes a disposable gameplay pack, patches native commands or
interprets scripts in a second dispatcher. Shared `RouteGrid` plans ordinary
steps, and every movement, turn, dialogue acknowledgment and reward continuation
uses `VisibleShellController::press` with native presentation waits.

The following five ignored production-controller tests have now passed Root's warm run; this section preserves their explicit setup and coverage:

- Ranger badge denial checks the real sixth-badge text and unchanged inventory,
  all three reports false, no completion and no rod. The authored badge guard
  belongs to the ranger. Volunteers retain their actual contract of setting
  their report after acknowledged dialogue; no new volunteer badge rule is
  invented.
- The full walking journey starts with Old Rod and an empty party. It visits
  reed `(15,33)`, meadow `(31,9)` and eastern beach `(75,17)`, proves dialogue
  owns movement and each report remains false before acknowledgment, then
  revisits volunteers without side effects. The ranger refuses after zero,
  one and two reports. The beach trip crosses to the real clipped eastern map;
  the return brings the player to ranger `(27,23)` by walking. The native
  introduction must precede actual Good Rod delivery and saved completion.
  Repeated reward visits and production Save/Continue preserve exactly one
  Good Rod, Old Rod, party, bag, position and all report/completion flags.
- Existing Good Rod and existing Super Rod are separate initial inventory
  fixtures. Each independently walks all three reports; the ranger's adequate
  rod acknowledgment completes SQ04 without any bag mutation, duplicate or
  downgrade, including a subsequent visit.
- All **eight** report subsets are separately constructed through actual
  volunteer walking/dialogue, without setting report flags in test setup.
  Only the complete subset can receive the rod/completion.
- A catalog/capacity guard records the full-key-pocket coverage limitation
  described below; it is explicitly not a full-bag gameplay test.

Every quest fixture retains an empty party and native Normal movement. The
optional offshore `EVENT_MN_R18_ITEM_04` stays false and Revive quantity stays
zero even after SQ04 completes. The quest therefore cannot accidentally acquire
a Surf/pickup prerequisite. Actual Surf launching and the Revive collection
remain separate controller work.

### Full KEY_ITEM pocket: precise coverage limit

The unchanged source catalog has **22** native key-item definitions, including
Good Rod and Super Rod. Only **20** can fill a bag eligible for a new Good Rod,
but `KEY_ITEM_POCKET_CAPACITY` is **25**. Duplicate key stacks are invalid, and
an existing adequate rod intentionally takes the acknowledgment branch before
`giveitem`. Therefore a full eligible key pocket cannot be constructed from
this actual published catalog. Filling the ordinary ITEM pocket would test the
wrong storage. No fake aliases, duplicate slots, inventory rewrites after input
or new frontend hook are added to hide this limit.

The asset/component test separately uses five explicit cloned-key definitions
in a test-only `GameDataSet` to prove the native full-key refusal, unchanged
completion, released slot, actual retry and save behavior. That is useful
component evidence for future larger mod catalogs; it is not a controller proof
on the current pack. A future content catalog with at least25 eligible distinct
keys should replace the capacity guard with a real production menu/transaction
retry fixture. No missing script command prevents the ordinary quest: the
current coverage gap is the unconstructible real-pack bag state.

Root can register this Bevy test module and run its existing warm target after
publication:

```sh
CRYSTAL_MILLE_LACS_SHORE_TEST_PACK="$PWD/content-packs/minnesota-mille-lacs-shore.crystalpack" \
CRYSTAL_MILLE_LACS_SHORE_QUEST_SAVE="$PWD/.local/mille-lacs-shore-quest.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy mille_lacs_shore_controller_ -- --ignored --test-threads=1
```

The optional quest-save env exports the genuinely walked and rewarded SQ04
state for later MCP input. The regression's temporary Save/Continue file has a
unique process/time filename and is removed. No state, position, flag or bag
mutation occurs after explicit initial badge/inventory/position setup. These
fixtures do not earn the sixth badge or prove hub access, route battle balance,
native fishing, Surf-only loot or final visual acceptance. Root remains sole
publisher and will record executed results separately.

### Bounded controller coverage audit

A follow-up audit retained the five existing quest tests and added no duplicate
scenario. The existing badge-denial test now also earns all three reports by
walking/dialogue, revisits the ranger without the sixth badge, and requires the
same badge-denial branch with reports/inventory preserved and no Good Rod or
completion. The existing eight-subset test now saves and Continues its two-report
subset before visiting the ranger; exactly those reports, location, party and
bag must persist, with the third report/rod/completion still absent. Every
mutation after initial setup remains a production input or save/load operation.
Both bounded additions passed in Root's complete five-test controller rerun.

The audit also confirms real remaining gaps: the empty-party quest tour does not
prove Surf launch/islet reward/rail denial, native fishing, route battles/PP or
earned sixth-badge access. The native component full-key-pocket retry remains
explicit fixture evidence because the actual unchanged catalog has fewer than
25 non-adequate key items. No source geometry, quest implementation, item catalog
or shared controller code was changed by this audit.

### Optional Surf islet controller handoff — awaiting execution

The separate ignored test
`mille_lacs_shore_controller_surfs_to_optional_revive_continues_and_returns`
now covers the remaining optional Surf-only item contract. It is **prepared,
not yet runtime PASS**. Its initial fixture places a native-created Mew60 with
legal Surf and badge bits at the public launch `(49,19)`; it installs no items,
reports, reward flag, movement mode or subsequent position/divider override.

Only production input mounts `(49,20)`, follows the actual water legs through
`(49,31)` and `(60,31)`, and dismounts onto the island at `(60,32)`. The ordinary
item interaction at `(61,35)` must deliver exactly one Revive and
`EVENT_MN_R18_ITEM_04`. A real Save/Continue on the island must preserve party,
bag, location and item flag. Walking onto the old item tile then proves its
native disappearance; actual Surf input returns through the same lake to the
public shore `(49,19)`. All three SQ04 reports, completion and Good Rod remain
absent. Incidental wild battles retain the real encounter path and resolve
through native RUN menus and visible text. This is a local optional-resource
fixture, not earned badges, quest completion or fishing/battle-balance evidence.

Read-only source preflight found 23 unique WATER cells on these legs, FLOOR at
both landings and the item approaches, and no other actor blocking them. The
same native external-pack environment as the five existing quest tests applies.
Root owns compilation/execution; no new test registration or pack export is
required for this leaf addition.
