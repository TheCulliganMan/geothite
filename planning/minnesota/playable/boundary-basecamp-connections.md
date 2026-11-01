# Free camp walking seams and visible checkpoints

The ignored `content-packs/minnesota-boundary-basecamp-connected.crystalpack`
contains 471 native maps, hash
`9cb8936dbd1afeeae3abe69e237aeddf12d5c8bccb3a8eb9f8b7ed53bdb45841`.
Its full Rust CBOR write/read, 20 asset/core tests, all nine fresh production-controller
cases and 377 audio hash/frame-count/loop-range checks pass. Three reciprocal
walking links and six visible Officers connect the free camp to Gunflint,
League Approach and Voyageurs without route terrain edits.

`connect_boundary_basecamp(&CompiledGamePack) -> Result<CompiledGamePack>` in
`crates/crystal-assets/src/boundary_basecamp_connections.rs` installs the native
connections and checkpoints atomically. The publisher's `--connect-routes`
option uses this function and strictly reads back its written file. Public
care remains free; Safari capture expeditions remain closed.

The inspected actual input was `content-packs/minnesota-region-review.crystalpack` with 469 maps. R12/R13/R20 edge pixels/collision and reciprocal internal chunk connections are present there. The new free town/shelter is Root's separate 471-map publication. The current frozen town layout is `9e8ade95` with the approved shelter, junction and port positions. This leaf requires that actual town publication; it does not create a substitute test camp.

## Exact ordinary walking connections

Native `GameDataSet::resolve_connection_transition` calls the existing `connection_destination_tile_components`: the offset is in metatiles and its doubled value is **subtracted** from the transverse core coordinate. It is not a core-tile offset. The prepared validator calls the actual native boundary trigger and resolver for both tiles in both directions, twelve crossings total.

| From → to | Native direction/offset | Last source Floor → first destination Floor |
| --- | --- | --- |
| `MnBoundaryWaters` → `MnR12C00R01` | east,+8 | `(63,24)/(63,25)` → `(0,8)/(0,9)` |
| `MnR12C00R01` → camp | west,−8 | `(0,8)/(0,9)` → `(63,24)/(63,25)` |
| Camp → `MnR13C00R00` | south,+1 | `(32,47)/(33,47)` → `(30,0)/(31,0)` |
| `MnR13C00R00` → camp | north,−1 | `(30,0)/(31,0)` → `(32,47)/(33,47)` |
| Camp → `MnR20C01R00` | west,−6 | `(0,24)/(0,25)` → `(63,36)/(63,37)` |
| `MnR20C01R00` → camp | east,+6 | `(63,36)/(63,37)` → `(0,24)/(0,25)` |

The R12 local west opening is global route `(0,56)/(0,57)`, because it belongs to chunk row1. R20's east opening is global `(127,36)/(127,37)`, in column1. All those edge tiles are already actual ordinary FLOOR in both native469 and current source. **Required minimal source metatile placements: none.** Existing route identities, dimensions, blocks, native internal reciprocal connections, trainers, pickups, rooms and flags remain intact. Only one previously unused outward direction is added to each adjacent route chunk. No Floor warp replaces walking adjacency.

The native crossing trigger is one tile outside the source map: camp east `(64,24/25)`, south `(32/33,48)`, west `(−1,24/25)`; reverse triggers are R12 `(−1,8/9)`, R13 `(30/31,−1)`, R20 `(64,36/37)`. Both source and target ordinary Floor are checked, and an incorrect offset is independently rejected by the native resolver's actual destination.

## Verified visible checkpoint contract

Six additive `SPRITE_OFFICER` / `OBJECTTYPE_SCRIPT` bodies have static inward facing and `event_flag=-1`. The two League identities honor the existing `regional_gate_contract` reservation. The four other IDs are new explicit checkpoint contracts, not secretly inserted ordinary residents. Root owns synchronization of this table and its twelve coordinate planes with the town schema/preview. The six original town residents stay unchanged, yielding12 total map objects, within the native15-event/12-nonplayer-struct limits.

| Checkpoint | Actual body IDs/coordinates | Inner checked plane | Outer inward-care plane | Facing/native inward movement |
| --- | --- | --- | --- | --- |
| Gunflint | `MN_BOUNDARY_GUNFLINT_GUARD_NORTH` `(61,24)`, `..._SOUTH` `(61,25)` | `(60,24)/(60,25)` | `(62,24)/(62,25)` | LEFT / `step LEFT` |
| League | `MN_BOUNDARY_LEAGUE_GUARD_WEST` `(32,43)`, `..._EAST` `(33,43)` | `(32,42)/(33,42)` | `(32,44)/(33,44)` | UP / `step UP` |
| Voyageurs | `MN_BOUNDARY_VOYAGEURS_GUARD_NORTH` `(2,24)`, `..._SOUTH` `(2,25)` | `(3,24)/(3,25)` | `(1,24)/(1,25)` | RIGHT / `step RIGHT` |

Gunflint and Voyageurs departures require all eight actual Johto badge engine flags plus `EVENT_MN_LIGHTHOUSE_SIGNAL`, preserving both route dossiers. The League additionally requires `EVENT_MN_SURVEY_RESTORED`, preserving the stronger camp source contract and Q07. The separate chief/receiver binder still owns earning those Q07 events; checkpoints never set, clear or grant them. There are no new saved permission events, Safari fees, permits, balls, steps or capture conditions.

The three script roots are `MnBoundaryGunflintCheck`, `MnBoundaryLeagueCheck`, `MnBoundaryVoyageursCheck`. At an inner plane, each checks the eight badges, then signal, then restoration for the League. A qualified check uses native `disappear` on the exact pair and ends silently, permitting ordinary departure. Refusal displays its18-column native text, closes text and applies one native inward PLAYER step: Gunflint60→59, Voyageurs3→4, League42→41. Those destinations and all body/inner/outer cells are real unoccupied Floor. The visible Officers make the checked passage legible; the coordinate plane also protects it independently of transient NPC visibility.

The native source patterns were inspected directly: `VictoryRoadGate/_VictoryRoadGateBadgeCheckScript` uses a coordinate guard, badge check, text and `applymovement PLAYER` to refuse passage; `Script_ForcedMovement` uses `readvar VAR_FACING` plus source `UP/DOWN/LEFT/RIGHT` comparisons. The actual469 native constant table is `VAR_FACING=9`, `DOWN=0`, `UP=1`, `LEFT=2`, `RIGHT=3`. `apply_script_variable_command_now_in_session` derives the trainer's actual facing; no synthetic direction variable or frontend interpreter is introduced.

## Free inward care flow

An outer plane runs `<root>Inward` for inward **or transverse** arrivals. Only actual OUTWARD facing ends without an escort: Gunflint RIGHT, League DOWN, Voyageurs LEFT. The native sequence is read facing0 / `ifequal OUTWARD,End`1 / disappear first Officer2 / second3 / `applymovement PLAYER,<root>TwoSteps`4 / jump toReconcile5. Two steps cross exactly outer→cleared body→inner: Gunflint62→61→60, Voyageurs1→2→3, League44→43→42. Every cell is inside the camp; no escort steps off-map, crosses water, changes rooms or touches a fee.

`<root>Reconcile` repeats the actual requirements. Qualified pairs remain hidden. Unqualified travelers run `<root>Show` (`appear` first0 / second1 / end2), closing the visible pair **behind** them while leaving them inside the free camp. The source remains closed by default; temporary visibility and object-struct retention/reset on reentry and Continue follow the existing native implementation. Native `disappear`/`appear` on `event_flag=-1` must return no event flag and must not alter restoration. No story event is repurposed as a visibility bit.

This inward flow keeps access to the public dock, shelter and free medic available even to a locally placed traveler missing every prerequisite. It does not make unearned R12/R13/R20 travel or the eight-badge story complete. The passing controller cases verify native forced movement, coordinate ownership, all four arrival directions and Continue without stranding a player on a guard body.

## Collision, preservation and component proofs

The leaf uses Root's `authored_route_surface::reachable_map_with_surf(pack,&MapModule,start,occupied)`, reusing the existing numeric permission/Surf/ledge model. For each guard proof, all ordinary NPC bodies are cleared and only its inner coordinate plane is blocked. Neither member of that checkpoint's outer seam may then be reachable by walking or Surf. Closing all three planes must still reach every inward public landing `(56,24)`, `(32,40)`, `(6,24)`, the public launch `(13,14)` and shelter safe `(43,20)`. This is a conservative local static graph; it does not follow room warps or other map connections and does not execute guard scripts.

A read-only current-source check independently found all three blocked planes sealed their corresponding seams, while all five public-care/dock anchors remained reachable. All six bodies and refusal destinations are actualFloor. Opening the three north-east shoulder metatiles `(29,11)/(30,11)/(31,11)` in a negative copy creates a real east-plane bypass; the production numeric helper is required to reject that copy. No authoritative terrain was changed.

The six asset/component tests use prefix `boundary_basecamp_connections_`:

1. Preserve maps/count/audio/files/items/trainers/species/HOME/recovery/constants/blocks; only camp scripts/objects/coordinate events and four maps' connection attributes change; native split-payload rematerialization must reproduce each module, including actual object identifiers and movement tables. The first test also requires all12 raw coordinate scene tokens `-1`, all12 typed scenes empty, then writes the full pack with `write_preserving_storage` and strictly reloads it with `read_verified_compiled_game_pack`, comparing the complete pack. The real file-round-trip regression passes; duplicate integration rejects.
2. Certify all twelve native seam crossings, exact reciprocal offsets and original neighbors; reject a genuinely wrong native east offset.
3. Use native flag checks/branches for every missing badge, missing signal and League restoration; component prerequisites are explicitly setup, not earned story.
4. Test all four actual native facing values for each inward root; for all six lanes, apply exact native object mutations, require the returned object IDs and no event mutation, execute both native inward steps, then reappear the exact pair without money/story changes.
5. Trigger each actual always-active inner coordinate plane and execute its native one-step refusal, retaining Floor/camp location, guard bodies, money and flags.
6. Reject actual blocked seam geometry, a three-metatile walking/Surf bypass and original-actor overlap. These are physical/native failures, not schema-only guards.

Root's warm command after shared module/re-export registration uses the actual **bare**471-map input, before this integration manifest is installed:

```sh
CRYSTAL_BOUNDARY_BASECAMP_CONNECTIONS_BASE_PACK="$PWD/content-packs/minnesota-boundary-basecamp.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-assets --lib boundary_basecamp_connections_ -- --test-threads=1
```

The function rejects an already connected input rather than silently running twice. Root must select the retained bare-town pack explicitly if the canonical alias later points to the connected output. This author ran scoped formatting and read-only geometry/source checks without a competing Cargo build or export. The current publication, full-file regression and all controller cases pass; the exact results are recorded below.

## Verified production-controller coverage

Both lanes of all three ordinary walking seams pass in both directions through
`VisibleShellController::press`, with exact native destinations. The cases
exercise every badge/signal refusal, independent League restoration refusal,
visible bodies, pushback, qualified disappearance, zero-money unqualified care,
all four outer-plane facing directions, Surf boundaries, room reentry and Continue.

The four Q07 cases also pass on this connected pack. The victory case earns the
chief event through an actual PP-spending battle, separately earns restoration
at the receiver, then walks the unlocked South seam into `MnR13C00R00(30,0)`
and Continues. Its legal level 80 Mew, eight badges, upstream signal and starting
position are explicit setup; no after-setup restoration is injected. The full
earned regional journey, upstream workshop and Safari capture runtime remain open.

Every native/source render, full-resolution original, preconversion image, converted/cropped/preview variant and rejected pass remains retained immutably through the shared archive.

## Native movement and wire representation

The actual source exporter represents movement as `step` with argument
`LEFT`, `UP` or `RIGHT`, followed by `step_end`; the checkpoint scripts and
compiled movement tables use that native form. Raw always-active `coord_event`
commands use scene token `-1`, while typed scenes remain empty. The full-file
write/read regression verifies this distinction without relaxing deserialization.

## Passing connected controller cases

`crates/crystal-bevy/src/bevy_shell/tests/boundary_basecamp_connections.rs`
contains five ignored production-controller cases. All five and the four Q07
cases pass on the current connected pack: nine passed, zero failed, in 18.77 seconds,
recorded in `.local/boundary-basecamp-controllers-current.log`. Their input is
`CRYSTAL_BOUNDARY_BASECAMP_CONNECTED_TEST_PACK`; preflight requires all three
connections and all six checkpoint IDs. A bare town cannot satisfy this fixture.

All mutations are confined to the single initial fixture constructor: native-created legal level 80 Mew with native Surf, injury/poison/zero PP when requested, zero money, an explicit badge array and upstream/Q07 events, local position and one divider stream. Qualified tests explicitly start with all prerequisites; they do not earn badges, chief victory or restoration. Isolated missing-badge/signal cases may retain other initially supplied events so each guard requirement is independently exercised. No state, clock, divider, visibility, position, HP/PP or story-event writes follow inputs. All movement, scripts, healing, Surf mounting/dismounting and saved changes use production `press`/ordinary frame waits.

| Passing case | Verified coverage |
| --- | --- |
| `...controller_twelve_walk_seams_reentry_and_continue` | Both lanes of all three passages physically open qualified guards, walk through their hidden bodies and enter outer planes facing outward without an inward escort. Each crosses the real seam out and back with exact native destinations, twelve transitions total; incoming movement executes the real escort; both guards remain hidden. Continue preserves exact current-map visibility and state. Real shelter entry/carpet return/reentry then revisits the checkpoint, which must open correctly after map reload. |
| `...controller_every_badge_signal_and_restoration_refusal` |28 separate local fixtures: each of eight missing badges and missing signal at every checkpoint, plus independently missing restoration at the League. Alternating lanes cover both physical checked planes. Exact native refusal text must appear; actual one-step pushback returns to the inward approach, actual Officers remain visible, and party/bag/badges/money/story flags remain unchanged. |
| `...controller_unqualified_inbound_care_dock_heal_and_continue` | All six route-edge fixtures begin with zero badges, no signal/restoration, zero money and an injured/poisoned/zero-PP partner. They cross the actual reverse seam and walk toward the outer plane; native facing/removal/two-step/reappearance must place the trainer exactly on the inner plane, never on a body. Each reaches the public dock, actual room and sole native medic; HP/status/Surf PP recover, both carpets are used across the two lanes, flags remain false, and Continue preserves results. Reattempting departure still gives the actual badge refusal. |
| `...controller_transverse_outer_arrivals_escort_to_free_care` | From explicit local placement on one outer lane, ordinary directional input walks sideways onto the other lane. These are achievable UP/DOWN inputs at the east/west planes and LEFT/RIGHT at the south plane. The native source escorts each transverse arrival exactly two inward steps, recloses the unqualified guard pair, and finishes facing inward. Continue preserves that real care-side state; outward departure must still refuse. Together with the first/third cases' outward/inward trajectories, this exercises all four input directions without fabricating a final transverse facing after the escort. |
| `...controller_public_surf_banks_cannot_open_signal_checkpoints` | With legal Surf/badges but no harbor signal, use the actual public dock `(13,14)` → water `(13,13)` through the native Surf prompt. Paddle ordinary WATER to the lake's closed forest banks nearest the three checkpoints; actual repeated movement against those WALL cells must remain Surf and blocked. Dismount naturally at the same dock and attempt all three walking departures: every signal guard remains enforced, actual Officers remain visible, no story/Safari resources appear, and Continue preserves the result. This is actual public-lake/shore input plus guard refusal, supplementing the asset helper's complete numeric walking/Surf bypass audit; it is not a claim to try every shoreline cell. |

The fixture-only walking planner uses actual **currently visible** object occupancy rather than blocking hidden map-event bodies. It plans ordinary Floor/Ladder/carpet steps only and avoids other inner checked planes as interaction hazards, retaining its current start and explicit target. The production controller still decides collisions, event ownership, hidden-object behavior and transitions. Guard and seam steps use direct bounded native input. A settled player on any Officer body's tile is rejected; intentional transit through a hidden body's Floor is allowed. Door/carpet input tolerates the native first direction press being only a turn, as confirmed by Root's actual bare-town MCP proof.

Read-only current source checks find296 ordinary WATER cells connected to the public launch. The chosen checkpoint-facing shoreline probes are water `(23,19)` facing the forest wall `(23,20)` for east/south, and water `(4,19)` facing wall `(3,19)` for west. No source terrain or capture was changed. The current actual public-Surf/controller case also passes; the read-only planner alone is not the execution proof.

Reproduce the five connection cases:

```sh
CRYSTAL_BOUNDARY_BASECAMP_CONNECTED_TEST_PACK="$PWD/content-packs/minnesota-boundary-basecamp-connected.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy boundary_basecamp_connections_controller_ -- --ignored --test-threads=1
```

Optional `CRYSTAL_BOUNDARY_BASECAMP_CONNECTIONS_SAVE_DIR` retains uniquely named actual saves after qualified/inward care/transverse/Surf checkpoints. Temporary `.crystalsave` files are removed after comparisons; no image is deleted. Every full-resolution capture, conversion, crop, preview and rejected render remains subject to immutable archival.


## Current milestone result

`.local/boundary-basecamp-guarded-assets.log` records 20 passing asset/core cases
in 7.51 seconds, including the actual full-CBOR write/read regression. The publisher's
read-back and all 377 audio checks pass on the current connected file. Earlier
rejected wire/outer-plane passes remain retained as history and are superseded
by this verified artifact.

The native outer-plane rule is `ifequal OUTWARD,End`: RIGHT for Gunflint,
DOWN for League and LEFT for Voyageurs. Inward and transverse arrivals use the
two-step escort, then reconcile guard visibility. The care fixture's ordinary
planner avoids sibling inner coordinate planes that would correctly trigger
another refusal; direct guard cases still exercise every plane. These bounded
source/fixture corrections use the existing production controller and native
script machinery.

The strict read-only native comparison passes all source blocks/art/actors,
six extra guard bodies, twelve raw/typed planes, reciprocal offsets and actual
compiled prerequisites/movement/control tables. It labels the connected
render as initial unqualified bodies, retains 141 artifacts and rejects twelve
adversarial skip-direction mutations against the actual new file. Whole-town
and all three handheld checkpoint views were inspected. Qualified visibility
and earned restoration are established by the controller cases above.

Bare-town MCP/TUI checks remain valid. Connected earned-save alias/MCP/TUI
checks are pending. All original, preconversion, converted, cropped and
rejected renders remain archived. Final user visual approval and a full earned
regional playthrough remain open.

## Source/world contract

The three camp links in `world.json` use `guarded_connection`, actual reciprocal
metatile offsets/core edge lanes and the three source checkpoint scripts.
Requirements apply only to camp departure: eight badges plus lighthouse signal,
with restoration additionally required for League. Inward/transverse care has
no fee or prerequisite. Town ports reference the same native metadata. Other
route endpoints remain separate work; this milestone does not make every
planned regional connection playable.

The shared `check_plan.py` calls the focused source validator below. Its
symbolic progression checks do not replace native inputs or establish an
earned eight-badge journey.

## Focused source walking-link validator

`tools/check_boundary_walking_links.py` now exposes `validate(world, root) -> list[str]`, where `root` is the Minnesota planning directory. It reads only the actual town JSON and three route dossiers; it does not change metadata, load a pack, build a movement graph or replace native/controller validation. Root's updated current world records pass. The shared `check_plan.py` now calls it; unrelated internal zero-offset connection validation remains separate.

The checker requires exactly the three stable route/chunk/camp identities, their frozen reciprocal directions/metatile offsets and paired core-tile edge arrays. It checks the native transverse arithmetic in both directions, the actual source chunk identity/dimensions and route endpoint's selected chunk, and world/source town port metadata. The existing wayfinding gate/landing markers remain separate from actual seam trigger coordinates. It rejects additional basecamp adjacency, duplicate records, obsolete warp-only fields and unknown/reverted kinds on these three links.

Each world link must name its actual source checkpoint/script and exact unique outbound prerequisites; League includes lighthouse signal independently of restoration. The source checkpoint must retain both visible native Officer identities, body positions, checked inner/outer planes, actual normalized inward movement and outward-facing constant, one-step refusal, two-step care escort and always-active scene. All three require zero fee, flag-free inbound care, camp-departure-only requirements and the approved inward-or-transverse escort policy. This checks the authored contract, not execution of those commands.

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/check_boundary_walking_links.py --self-test
```

This command passes against current source/world and rejects16 adversarial in-memory cases: wrong kind,999999 outbound offset, incorrect reciprocal offset, displaced lane, noninteger lane coordinate, nonexistent script, missing badge, duplicate badge requirement, paid care, boolean fee, inbound prerequisite, old transverse-stranding policy, duplicate link, source checkpoint plane drift, source League signal omission and stale world port metadata. A separate positive case confirms unrelated internal zero-offset links remain untouched. Syntax/import/API checks and malformed-input rejection also pass. No source metadata, Rust, shared verifier, pack or render was changed for this checker; no symbolic badge graph is presented as an earned journey.
