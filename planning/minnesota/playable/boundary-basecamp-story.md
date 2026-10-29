# Free Boundary Waters basecamp: Q07 binding

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

The later [earned northern continuation](northstar-finale.md) now proves this
installed scene on the544-map shore pack using an actual Brainerd-to-Boundary
arrival save. It earns the chief win and separate restoration, demonstrates
the still-locked League checkpoint between those actions, and walks all of
R13. This supplements the earlier local Mew80/eight-badge fixtures below;
initial six badges/Mew45 remain setup in that upstream course. Two real stdio
replays pass. Ely and the League are still absent from that pack.

`crates/crystal-assets/src/boundary_basecamp_story.rs` binds the actual survey shelter's native chief battle and receiver scripts. It implements MS11/MS12 in [story.json](../story.json), using the [real shelter](../towns/MnBoundaryWaters.json) and native basecamp publisher. All five story component tests and all nine production-controller cases (four story, five regional-connection) pass against the actual471-map connected town. The [native runbook](boundary-basecamp.md) records publication, review saves, preservation, audio and input evidence. The connected input proof now includes the earned chief-to-receiver-to-League departure described below.

The mandatory shelter remains outside the paid preserve. The binding checks no Safari permit, fee, balls, steps or captured species, and adds no admission, budget, movement, recovery or hub-gate handler. The free medic's actual `HealParty`, assistant, furniture, map blocks and caller-supplied warps stay intact. The R12/R13/R20 public connections and closed/no-fee Safari check-in belong to the town publisher.

## Exact caller interface and room anchors

```rust
prepare_boundary_basecamp_story(
    base: &CompiledGamePack,
    shelter: &MapModule,
    anchors: BoundaryBasecampStoryAnchors,
) -> Result<PreparedBoundaryBasecampStory>
```

The result has `pack: CompiledGamePack` and `shelter: MapModule`. Both inputs are borrowed and remain unchanged on success or failure. The caller must publish the returned pack/catalog and real room atomically with the basecamp and actual native metadata/warps. The preparation step does not register maps, choose a HOME, add a recovery point or publish a pack.

| Contract | Exact current source |
| --- | --- |
| Shelter | `MnBoundaryWatersSurveyShelter`, native constant `MN_BOUNDARY_WATERS_SURVEY_SHELTER`,8×6 metatiles, original `ElmsLab` template/`lab` art |
| Chief | Supplied `OBJECTTYPE_SCRIPT` / `SPRITE_ROCKET`, object/script `MN_SURVEY_CHIEF`, core `(9,5)`, standing north, always present; adjacent Floor approaches `(8,5)`, `(10,5)`, `(9,6)` |
| Receiver | Supplied `BGEVENT_READ` script `MN_BOUNDARY_SURVEY_RECEIVER`, core `(12,3)`, original PC block `08` at meta `(6,1)`, real WALL collision; interaction from Floor `(12,4)` |
| Clear arrival | `(8,9)` Floor, connected to all chief/receiver approaches around actual NPC bodies |
| Caller-owned room warps | Passive arrival3 at `(8,9)`; both active carpets1/2 at `(8,11)`/`(9,11)` return to the basecamp's safe passive exterior record2. No warp is altered by the story binder. |
| Other residents | `MN_BOUNDARY_SURVEY_MEDIC` `(3,7)` with real native `HealParty`; `MN_BOUNDARY_SURVEY_ASSISTANT` `(3,3)` ordinary dialogue, both preserved |

`BoundaryBasecampStoryAnchors` fields are `chief_tile: [u16;2]`, `chief_approaches: [[u16;2];3]`, `receiver_tile: [u16;2]`, `receiver_approach: [u16;2]`, and `clear_arrival: [u16;2]`. Leech supplies the values above after building Northern's actual room. The binder verifies the real object/event identity, art/collision, three distinct chief approaches, connected clear Floor and lack of overlapping bodies/background receiver events. Only existing ordinary `jumptextfaceplayer MN_SURVEY_CHIEFText` and `jumptext MN_BOUNDARY_SURVEY_RECEIVERText` placeholders may be replaced. It does not construct a second shelter.

## Native story behavior

MS11 requires **all eight Johto badges** and `EVENT_MN_LIGHTHOUSE_SIGNAL`. The native checks use `ENGINE_ZEPHYRBADGE`, `ENGINE_HIVEBADGE`, `ENGINE_PLAINBADGE`, `ENGINE_FOGBADGE`, `ENGINE_STORMBADGE`, `ENGINE_MINERALBADGE`, `ENGINE_GLACIERBADGE` and `ENGINE_RISINGBADGE`; no invented `BADGE_8` engine token is used. Missing any badge gives a refusal; all badges without the harbor signal gives a workshop reminder. Neither refusal starts a battle or grants a story flag. The room and medic remain physically accessible regardless of those checks.

The reserved trainer is exactly **`MN_LODE_CHIEF`: Magneton44, Weezing44, Crobat46**. The binder derives the opposing party's DVs/class behavior from the actual native `EXECUTIVEM` catalog and creates each member through the production species/move builder. Original trainer definitions remain unchanged. A new numeric chief identity follows the existing executive-class namespace; identifier collisions fail.

The chief root begins with `faceplayer`, checks the saved chief-defeat event for repeat dialogue, then checks the eight badges and signal. It uses the original game's supported boss pattern, inspected in `RadioTower5F/FakeDirectorScript`: authored dialogue, `winlosstext`, `loadtrainer`, `startbattle`, `reloadmapafterbattle`, then the victory event. No new `callasm`, second script interpreter or frontend dispatcher is introduced.

| Chief root command | Meaning |
| --- | --- |
| 0 | Native `faceplayer`, retaining the actual chief object for scripted battle presentation |
| 1–2 | Defeated-event repeat guard |
| 3–18 | Eight individual native badge checks/refusal branches |
| 19–20 | Upstream lighthouse-signal check/refusal |
| 21–25 | Challenge dialogue and native win/loss text |
| 26–27 | `loadtrainer EXECUTIVEM, MN_LODE_CHIEF`; native `startbattle` |
| 28 | Native `reloadmapafterbattle`; ordinary losing boss behavior whiteouts instead of continuing |
| 29–30 | Set only `EVENT_MN_SURVEY_CHIEF_DEFEATED`, then post-victory dialogue pointing to the east desk |

The source battle is ordinary `BATTLETYPE_TRAINER`, with no can-lose mode. Its scripted request has no automatic completion flag; the explicit post-reload source opcode owns the chief event, matching the inspected source boss flow. Chief victory does **not** set `EVENT_MN_SURVEY_RESTORED`. The executed controller cases below verify played loss/whiteout and actual story continuation; the successful connected case also walks through the released League seam.

MS12 belongs to the **separate receiver** interaction. An already restored link reports its saved active state. Before the chief event it refuses. After victory it asks `Restore the link?`; NO leaves the event false for retry. YES shows the alignment text, waits for acknowledgment, sets `EVENT_MN_SURVEY_RESTORED`, and reports the active link. Repeated use does not battle, grant items or reset resources. Receiver refusal/repeat roots use ordinary `jumptext`, avoiding a background sign trying to turn a stale last-talked NPC.

The receiver's exact native indices are check restored0 / repeat branch1, check chief2 / locked branch3, open/text4–5, `yesorno`6 / decline branch7, alignment text8 / wait9, restoration event10, completion text11 / wait12 / close13 / end14. Only this receiver has a restoration setter. No script here sets the upstream lighthouse event.

## Event metadata and preservation

The three required named events are `EVENT_MN_LIGHTHOUSE_SIGNAL`, `EVENT_MN_SURVEY_CHIEF_DEFEATED` and `EVENT_MN_SURVEY_RESTORED`. Missing identifiers are allocated after the current EVENT namespace; existing unique identifiers are preserved. A target identifier aliasing another existing EVENT number is rejected. This adds catalog references only: **no saved event is granted during preparation**. The upstream harbor scene remains separate implementation work; this binder cannot stand in for earning the signal.

The native script parsers lower text, flags, controls, runtime/object commands, variable commands, scripted trainer descriptors and `reloadmapafterbattle`. The prepared pack identity is refreshed using the production identity function before returning to the publisher. Existing maps, source runtime files, art, items, marts, species, audio, HOME/recovery and all prior constants/trainers stay intact. The room's geometry, actors, events, attributes, medic and assistant are preserved; only the two designated story roots and additive supporting script/text labels change.

## Focused test coverage prepared for Root

All five inline tests use prefix `boundary_basecamp_q07_` in the focused leaf. Shared original pack/room construction and prepared component data use `OnceLock` to avoid repeated pack decoding. Tests read the actual town JSON's shelter rows; no invented test room replaces it.

1. **Preservation and reserved party:** compare the original catalogs/files/maps/audio/spawns and actual room blocks/objects/events/attributes; certify exact chief party, native load/start indices, sole chief/receiver setters, untouched upstream event and medic/assistant scripts, and dialogue within18 columns.
2. **All-badge and signal guards:** exercise each missing badge independently through production `check_script_flag` and `apply_script_control_command`; test the native signal refusal/continuation, zero-money access and unchanged bag/story flags. This is a command-component test, not a walking story claim.
3. **Receiver refusal, decline, retry and repeat:** use the native flag checks, actual native `yesorno` prompt plus `ResolvePendingYesNo`, conditional branches and restoration mutation. Chief victory is explicitly a component precondition setup in this console-only case; it is not presented as an earned win.
4. **Actual battle turns and terminal branches:** create an explicit legal Mew100/Surf component partner, start the native reserved three-member opposing party, resolve actual selected-move turns without HP rewriting, claim native faint rewards/advance opponents, and require actual party defeat and spent PP before the native victory completion/event mutation. Chief victory must still leave restoration false. A separate explicit losing-terminal state checks the original `reloadmapafterbattle` whiteout branch; it is not an actual played loss. These direct native asset APIs do not replace the required controller battle/continuation fixture.
5. **Physical/namespace negatives:** reject a chief on the PC, buried receiver, blocked receiver approach, missing receiver, duplicate chief, foreign story root and numeric event alias; preserve the caller's room on failures. These exercise actual anchors/collision/native identity contracts rather than schema-only rejection.

Root's warm command after adding the leaf/re-export is:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-assets --lib boundary_basecamp_q07_ -- --test-threads=1
```

**Execution passes.** Scoped formatting and all42 authored dialogue lines at18 columns or fewer pass. The native publisher preserves all469 prior maps/files/audio and publishes the actual town/shelter atomically. Source party, prerequisites and result events are checked against the prepared native bindings so unsupported dossier changes cannot silently publish older story semantics.

## Production-controller acceptance contract

After native publication, use the actual free basecamp through `VisibleShellController::press`: zero-money walking from every public port, safe shelter entry/passive arrival/both carpet returns, free nurse HP/status/PP recovery, each prerequisite denial, actual chief battle with selected PP and a real loss/retry, actual win setting only the chief event, separate locked/declined/accepted/repeated console interactions, Continue after victory and after restoration, and the real R13 release check. Do not inject victory or restoration after inputs, pay a Safari fee, create a fake expedition or treat a map dump as gameplay proof.

The current slice does not implement the upstream Duluth workshop scene, earned eight-badge journey, complete regional gates/recovery/Fly, Safari admission/capture/session rules, or a cosmetic powered-device scene. Restoration is the native saved story flag and authored console text; no unsupported visual mutation is promised. Final native player-camera review and user approval remain separate.

## Executed production-controller fixtures

The focused leaf is `crates/crystal-bevy/src/bevy_shell/tests/boundary_basecamp_story.rs`. All four cases pass against the actual471-map publication specified by `CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK`; no substitute shelter or test script interpreter is used. Both native downward carpets require the onward downward input, and exterior re-entry uses an ordinary upward turn followed by the door step.

Every case labels its initial setup: actual source-created legal party, local placement, zero money, requested badge array, upstream signal and one deterministic divider stream. Successful cases use Mew80 with source-native `SURF`/`PSYCHIC_M`; free-care cases begin injured, poisoned and out of PP. Chief defeat and restoration always begin false. After setup, movement, text, choices, battle turns, healing, whiteout and story continuation use `VisibleShellController::press` and ordinary presentation waits. No later state, position, flag, clock or divider injection occurs. Saved Continue instances compare actual party, bag, position, badges, money and all three Q07 flags; optional `CRYSTAL_BOUNDARY_BASECAMP_SAVE_DIR` retains uniquely named review saves.

| Executed story controller | Concrete proof checked by its assertions |
| --- | --- |
| `boundary_basecamp_story_controller_free_ports_medic_both_carpets_and_continue` | Walk within the actual free town from all three public landings `(56,24)`, `(32,40)`, `(6,24)` to exterior safe `(43,20)`; enter the real door `(43,19)` and arrive on room passive3 `(8,9)`; actual sole nurse restores HP/status/all PP for zero money and no badges/signal; both carpet returns and passive no-bounce; Continue preserves resources and false story flags. These are local town landing walks, not implemented cross-route connection proof. |
| `boundary_basecamp_story_controller_each_badge_signal_and_receiver_denial` | Eight independent initial fixtures each lack exactly one actual Johto badge while holding the signal; a ninth holds all badges but lacks the signal. Each must show its exact native refusal, reject dialogue-time movement, preserve party/bag/money/badges and both success flags, and still reach the separately locked receiver. |
| `boundary_basecamp_story_controller_actual_victory_separate_receiver_retry_and_continue` | Locked receiver before combat; real three-opponent chief battle with selected native attacks, PP consumption and victory EXP; only chief-defeat after the battle; Continue before restoration; chief repeat gives dialogue without another battle/reward; receiver NO remains retryable, YES still waits for alignment acknowledgment then sets only restoration; Continue and both repeated interactions preserve party/bag/money and saved flags. |
| `boundary_basecamp_story_controller_played_loss_does_not_grant_victory_or_restoration` | Explicit legal Magikarp5 with1HP chooses native Splash in the actual chief battle, loses to the real first opponent and follows ordinary whiteout out of the shelter. Both success flags remain false through Continue. This is an executed played-loss proof, separate from the earlier component test's synthetic losing-terminal setup. No recovery destination is invented or rewritten. |

All cases require `ENGINE_SAFARI_ZONE` false and no `wSafariBalls`/`wSafariSteps` memory. The story must never fabricate an admitted expedition or require a fee. Victory money is the existing native trainer reward; only subsequent console/repeat interactions must preserve it. Source parties, battle scaling and original loss/recovery policy are left intact.

A read-only source/collision check confirms that the reused fixture planner's ordinary Floor/Ladder/carpet set, excluding actual town NPC bodies, connects the three landing setups to `(43,20)` in17,31 and49 steps respectively. It reads the supplied pack collision and current authored blocks; it is a planner precheck, not native movement proof. The shelter's revised `01` floor and `06`/`07` desk art preserve all story anchors.

Root's warm command after actual publication and module registration is:

```sh
CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK="$PWD/content-packs/minnesota-boundary-basecamp.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy boundary_basecamp_story_controller_ -- --ignored --test-threads=1
```

The connected case now proves the actual R13 release after earning chief defeat and receiver restoration through production inputs. The upstream workshop/lighthouse scene and an earned all-eight-badge journey remain separate acceptance work. This author has not run Cargo, exported a pack or produced a capture for these fixtures. Every future source/native capture must retain its immutable full-resolution original, pre-conversion image, converted/cropped previews and rejected passes; never delete a PNG or overwrite its sole copy. Optional latest aliases may point to retained immutable versions.


## Connected production result and earned League release

Root's warm run passes **all nine** basecamp production-controller cases against `content-packs/minnesota-boundary-basecamp-connected.crystalpack` (471 maps; connected review hash `e7257308`). The four story cases above pass alongside the five focused cases in `boundary_basecamp_connections.rs`: twelve reciprocal walking-seam crossings and re-entry/Continue; each missing badge/signal/restoration refusal; unqualified inbound access to dock and free medic/Continue; transverse outer-plane arrival escort to care; and public Surf-bank attempts that cannot open signal-locked checkpoints. Actual input, script continuation and saved visibility are checked separately from the source preview's static body flood.

The successful story case starts with a legal **Mew80**, all eight Johto badges and `EVENT_MN_LIGHTHOUSE_SIGNAL` as explicit upstream fixture setup. Chief defeat and restoration start false. It spends selected native move PP against Magneton, Weezing and Crobat, wins, saves/Continues, declines then accepts the separate receiver alignment, and acknowledges its text before restoration is awarded. With no subsequent state, position, party or flag writes, it leaves the shelter, reaches the southern inner plane `(32,42)`, proceeds through the released checkpoint and town seam, enters **`MnR13C00R00` at `(30,0)`**, then saves and Continues there with earned story flags and resources preserved. The connected publication contains that actual connection, so the test's connection-dependent branch executes in this run.

This proves the local earned Q07 release with supplied upstream prerequisites. It does not prove an earned eight-badge regional journey or upstream lighthouse/workshop completion. Safari capture/session admission remains closed and unimplemented; the successful fixtures retain no Safari balls/steps/session flag. Static qualified renders still label their requirements as assumptions rather than earned gameplay. Final user visual approval remains open.

## Northstar narrative pass — 2026-10-26

The chief now recognizes the original survey readings from the opening parcel,
admits changing the reports, and names Kettlurk as the intended pump captive.
His post-win dialogue yields the relay without repentance. The separate receiver
completion reports the true readings sent and harbor crews halting the fictional
pump test, then retains the Ely trail direction. These are authored offscreen
consequences of survey restoration; no new machine animation, legendary battle,
world simulation or additional saved flag is implemented.

`story.json` MS11/MS12 carry all twelve exact native text labels. The existing
source-preservation regression now compares the actual lowered native text
commands with those records, including the 18-column limit. All five Q07 asset
component tests pass after this change, including native guards, actual battle
turns, receiver choices, preservation and rejection cases. Earlier connected
pack hashes and controller results above describe the preceding dialogue revision;
they do not prove this text pass was published. Current publication evidence is
recorded in the regional status.

The narrative candidate was built without replacing the earlier review packs:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo build --profile wasm-dev --locked \
  -p crystal-assets --bin pack_boundary_basecamp --bin pack_grand_marais
target/runtime-fps/wasm-dev/pack_boundary_basecamp \
  content-packs/minnesota-region-review.crystalpack \
  planning/minnesota/towns/MnBoundaryWaters.json \
  planning/minnesota/towns/MnBoundaryWaters.layout.json \
  content-packs/minnesota-northstar-finale.crystalpack --connect-routes
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-finale.crystalpack --audio --desktop
CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK="$PWD/content-packs/minnesota-northstar-finale.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --features location-tester --lib \
  boundary_basecamp_story_controller_ -- --ignored --test-threads=1
```

All four controller cases pass on this new candidate. The separate Grand Marais
foreshadowing candidate was also published and its full town integration passed:

```sh
target/runtime-fps/wasm-dev/pack_grand_marais \
  content-packs/text-tui.crystalpack \
  planning/minnesota/playable/grand-marais.json \
  content-packs/minnesota-northstar-grand-marais.crystalpack
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-grand-marais.crystalpack --audio --desktop
CRYSTAL_MINNESOTA_TEST_PACK="$PWD/content-packs/minnesota-northstar-grand-marais.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --features location-tester --test grand_marais
```

Both retain all 377 audio PCM hashes, frame counts and loop ranges. The finale
candidate uses the preceding regional source, while the Grand Marais candidate
is an isolated new-game town; neither is a newly earned complete regional story.
