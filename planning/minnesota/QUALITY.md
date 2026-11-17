# Minnesota complete-game implementation

This tracks the twenty recommendations accepted for implementation on November
16, 2026. Builds and tests run after implementation, at the user's request.
An authored change is not installed content or acceptance evidence.

User correction: no playing to verify, including focused controller gameplay
tests. Finish the twenty implementation areas directly in source, then use
compilation and static/content checks only. Earlier gameplay checks below are
historical partial evidence, not instructions to resume or repeat them.

The original integration started from `world.json`'s `glacier_rear_rail_revision`.
Existing saves and the selected/hosted game stay intact while the candidate is
assembled. Generated packs, captures and run receipts remain ignored. The final
acceptance must identify the exact candidate and distinguish earned play from
chapter fixtures. Keep the original numbered reports in `TASKS.md` intact.

| Recommendation | Required implementation and acceptance | State |
| --- | --- | --- |
| 1. Integrated game | One reproducible current-world candidate with reviewed gyms, roster, scenes and current pages; verify installed branches and preservation. | Implemented in the integrated candidate recipe |
| 2. Character canon | Reed, courier, foreman and Hale knowingly remain antagonists; player performs shutdown and rescue. | Canonical villain and player-action revisions installed |
| 3. Earned balance | Review the three starter curves, recruitment, resources and legal counterplay; check authored levels, moves and resource tables. | Early fishing levels and legal boss counterplay revised; no whole-game balance claim |
| 4. Opening | Clear partner/delivery/badge sequence, optional discovery branches, level-appropriate early fishing. | Opening, short reminders, delivery guidance and early fishing integrated |
| 5. Reliable play | Resolve open menu, field-target, battle, recovery, arrival and move-learning reports through shared runtime paths. | Service/arrival fixes integrated; shared runtime fixes retained; static checks only |
| 6. Dialogue rhythm | Short, varied ordinary challenges; distinctive major voices and coherent repeat replies. | Trainer challenges, major voices and 24 additional town/aftermath conversations revised |
| 7. Playable investigation | Each chapter has a visible observation or meaningful action; custody and knowledge remain causal. | Evidence handoffs, independent observations and Mina refusal staged |
| 8. Villain escalation | Distinct ICE, shipment, falsification, mine and intake stakes; consistent responsibility. | Courier, foreman, Reed and Hale responsibility clarified across chapters |
| 9. Pokemon involvement | Recurring habitat consequences and grounded legendary anticipation; no mandatory legendary capture. | Habitat consequences and captive release retained; retired encounters rebound or removed |
| 10. League presence | Establish Casey and leaders before their final challenges without inventing offscreen player achievements. | Leader voices and Casey references established before the League |
| 11. Public travel | Integrate open roads and visited-station travel; keep action/site/League prerequisites and safe returns. | Open roads and visited-station travel integrated; action/League gates retained |
| 12. Spatial quality | Review twenty routes in both directions, cave geography, visibility, lighting and shortcuts. | Lit east-west Driftless passage, aligned cave exit and current route geography integrated |
| 13. Gym places | Integrate eight reviewed rooms, preserve external geography, verify every puzzle state, loss and return. | Eight reviewed rooms, private native art, teams and puzzle flags integrated |
| 14. Discovery | Active roster across every acquisition and opponent path; useful staged recruitment and in-game hints. | 225-species roster/Dex alignment and regional acquisition closure confirmed statically |
| 15. Battle identities | Deliberate boss/rival roles, legal moves, accessible counterplay and continuity of recurring teams. | Explicit faction teams, recurring partners and legal move checks installed |
| 16. Economy | Affordable recruitment/recovery, useful timely rewards, once-only receipt and full-pocket retries. | Claim-before-receipt ordering, full-pocket retries and once-only rewards implemented |
| 17. Town experiences | Distinct existing activities and ambient lives; reduce repetitive errands and directions. | Clerk greetings, Battle Club returns and concise local conversations wired |
| 18. Canoe revisits | Meaningful early-water discoveries, readable portages, fishing, return transport and afloat Continue. | Selected canoe world preserved with free canoe hints, transport and fishing implementation |
| 19. Presentation | Native sprite/scene review and contextual listening; verify graphical, browser and native terminal paths separately. | Reviewed native gym art integrated; native-2D policy and existing audio preserved |
| 20. Ending/postgame | Separate battle/controls/release, personal aftermath, League celebration and clear optional continuation. | Player shutdown/release, personal aftermath, League and optional continuation integrated |

## Final verification limits

After implementation, run scoped formatting and required Clippy, compile the
client and a fresh ignored pack, and statically compare audio content.
Compare the final installed text and active roster against authored sources.
Inspect refusal, loss, full-pocket, reentry and Save/Continue branches in source.
Do not launch gameplay tests, controller journeys or playthroughs. Report the
resulting verification limits plainly; compilation does not prove game balance.

## Implemented candidate recipe

The notes below retain the implementation history. References to future
controller runs are superseded by the no-gameplay instruction and final
source/content candidate results at the end of this document.

`pack_northstar_integrated` now composes the bounded story delta on the exact
Glacier parent, installs the eight reviewed gym rooms, applies the current
opening/clinic scene fixes and public-road changes, and finally binds the active
roster and canonical dialogue. It also includes corrected Mart/house arrivals,
ordinary trainer challenges, the lit Driftless passage and native Centers.
Center resizing follows scene installation to relocate Reed's clinic approaches
into the final room. That order matters: old scene authors still name
retired species, so the roster pass must follow scene installation. Audio must
remain byte-for-byte unchanged. The complete recipe now builds and reads back
identically. Installed gameplay, visual review and the earned campaigns remain
open; the candidate is not selected or published.

The source command for reproducing the final pack is (choose a fresh output):

```sh
target/debug/pack_northstar_integrated \
  content-packs/minnesota-glacier-rear-rail-v01.crystalpack \
  content-packs/minnesota-reviewed-maps-v11.crystalpack \
  content-packs/minnesota-eight-gyms-active-roster-review-v01.crystalpack \
  content-packs/minnesota-minneapolis-citywide-cycling-v02.crystalpack \
  content-packs/minnesota-dialogue-overhaul-v27.crystalpack \
  planning/minnesota/revisions/story-consolidation-v01/writing.json \
  planning/minnesota/story.json \
  content-packs/minnesota-integrated-campaign-v03.crystalpack
```

The executable rejects an existing output and refuses publication when canonical
dialogue remains deferred. Reachability still needs controller verification.
No command here establishes
that a candidate exists, is selected, or is published.

The current team source now replaces retired GOOSNIDE with VANECHICK/SILOHAWK,
keeps Reed's Crookit, bird and Wrigglet families through successive chapters,
and reserves Cartick's strongest coverage for later guards. The candidate applies
explicit faction teams after ordinary roster repair; the installer checks legal
level/TM moves. The nursery egg policy and retired-species bindings now use the
active roster. These are authored choices, not measured balance results.

Reward installation now records successful claims before showing receipts,
including the shared fossil success branches and bicycle handoff. Full-pocket
branches remain before the claim. A controller regression saves at the first
visible boundary after a research award, then Continues and talks again; it has
been written but not run. Existing full-pocket/retry coverage also remains part
of the final phase.

The existing unboosted New Game through Hall of Fame course now selects
POUCHID, KINDLIT or BIRCHIT with `CRYSTAL_NORTHSTAR_STARTER`. It rejects chapter
save overrides and restores the chosen starter family after temporary party
switches. Gym courses read the integrated rooms' individual source plans;
Mart visits use the installed passive arrival. These changes prepare the final
three-starter runs, but no run or balance claim is recorded yet.

Investigation dialogue now connects the copied ring stamp, Mina's dated
witness account, the altered notebook and the independent headwaters reading.
Refusing Mina leaves her waiting without acknowledging her testimony. Clinic
dialogue also describes the displaced Loobit's nest and recovery. The final
controller pass must confirm these observations and refusals in their actual
story order, including early visits and return conversations.

The town writing pass removes 22 duplicate old label aliases and five obsolete
pre-scene placeholders, and binds six remaining Duluth pages to their installed
map-qualified labels. The North Shore sign and guard no longer advertise the
removed badge gate. Four Battle Club trainers keep their immediate battle
responses and gain distinct voluntary return conversations. These changes are
source-only until the final installed-dialogue and controller checks run.

## Final verification in progress

- `cargo fmt --all -- --check` passed after formatting the changed campaign files.
- `CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets --bin
  pack_northstar_integrated` passed. Seven existing dead-code warnings remain in
  verification and sprite-sheet helpers outside this change.
- Source inspection of all 2,832 canonical labels across 340 maps found no
  invalid page counts, non-ASCII text or overlength lines. The native compiler
  test and installed page review remain required.
- The first actual candidate composition rejected overlapping changes to
  `MnStPaulGymGuideText`: reviewed pad instructions versus the current canoe
  hint. The specific conflict is now resolved through canonical pages containing
  both. Other merge conflicts still fail. The corrected composition must run
  successfully before a pack or installed behavior can be claimed.
- The successful fresh candidate is
  `content-packs/minnesota-integrated-campaign-v01.crystalpack`, content hash
  `6b7d526fd5b9081fe8ea1a8f8da1f2b8a3c9f7558440c503efff858c7b209ea5`,
  SHA-256 `87609e4fad4b9d34f47e47896157e752ce468d6d115d7dedad6b5617514990b6`.
  Runtime verification, zero deferred canonical labels, identical readback and
  exact parent audio manifest/program/compression preservation passed inside
  the builder. This does not establish conversation reachability or playability.
- Integration repairs also cover the three St. Paul leader reward pages,
  coordinated publication of shared fossil text and actual clerk-to-greeting
  bindings. Two obsolete Pine Badge denial pages were removed because their
  investigation gates are intentionally absent. The canonical source now has
  2,830 labels. `canonical_dialogue_fits_native_pages` passed through the native
  compiler (one test, zero failures), and formatting passed again.
- A static traversal from local object, background, coordinate, scene and
  callback roots found 129 canonical labels without a local reference path.
  This is a triage list, not proof of runtime unreachability: it includes retired
  road denials, replaced nurse pages and potentially indirect calls. Remaining
  shop/interior voices and investigation labels require owner-by-owner review
  before the dialogue coverage requirement can be accepted.
- `current_campaign_combines_scenes_gyms_services_and_roster` passed (one test,
  zero failures). It rebuilds from the five exact parents, requires no deferred
  dialogue, compares all eight gym blocks/warps against the reviewed donor,
  checks non-Minnesota maps against the selected parent and verifies the opening
  services, relocated clinic trigger, lit Driftless passage, canoe and road
  marker. The test root is canonicalized to satisfy the pack reader's path
  guard; the guard itself remains unchanged. This is structural integration
  evidence, not an earned playthrough.
- The first KINDLIT New Game campaign attempt reached Tamarack, then failed the
  brief-repeat check: `MnWChooseText` restarted an introduction about Mom's
  message. The source now restores the short choice reminder. Four additional
  Mart greetings found without clerk references are now wired through the same
  guarded shop installer (Brainerd, Bemidji, Grand Rapids and Duluth).
- The earned delivery driver now reads the installed Rochester Center arrival
  and walks to Ivy, reflecting the restored room's real entrance. It does not
  teleport or grant progression. The fresh `minnesota-integrated-campaign-v02`
  candidate builds and reads back successfully: content hash
  `33795e823dab71d8672f8776693079e47ddd7f8b9025f49d4bfa5d283c3d7e1a`,
  SHA-256 `4a32899d9893505500a332d0c2828c066847aa7877b7b4e77cc92d50be3d0008`.
- The v02 starter run exposed a driver timing error: YesNoBox exists while the
  starter description still has unread pages. The helper now waits for the
  production prompt-readiness predicate using the complete printer stream,
  then sends the requested cursor input. A trace confirmed that comparing the
  controller's single-page projection instead incorrectly missed the choice.
- The regional acquisition audit passes roster/Dex alignment: 225 active
  species in 99 families, 177 potential direct acquisitions and 225 after
  evolution closure, no missing family, and a completion threshold of 225.
  The encounter audit reports no inactive species in 355 regional maps and no
  active species without known/free acquisition. These are table/program upper
  bounds; availability, capture and evolution still need earned-play evidence.
  Both reports remain ignored under `target/northstar-integrated-review/`.

The latest direct content pass rewrites 20 long ambient conversations plus four
home/League aftermath replies. It preserves local character and useful travel
information while shortening ordinary interruptions. No gameplay verification
is authorized after the user correction above. The final candidate is a source
and content integration, not a claim of an exhaustively playtested release.

## Final source/content candidate

`minnesota-integrated-campaign-v03.crystalpack` builds successfully, passes the
builder's content checks with zero deferred canonical dialogue and reads back
identically. Content identity:
`cd81b377a9d08285dddba8568547e573827860f1ea41630e2301be4750296313`.
File SHA-256:
`6d4cca148a5e277442a714fd1474d84bd315a1c63aea0ff80dd2a1003437e3a7`.
The build requires exact parent audio preservation. All 2,830 canonical labels
pass the static native-page limits. Formatting and
`cargo clippy --locked -p crystal-assets -p crystal-bevy --all-targets` pass;
repository warnings remain. `CARGO_INCREMENTAL=0 cargo build --locked -p
crystal-bevy --bin crystal-bevy` also passed, with one existing dead-code
warning for `browser_save_path_for_identity`. No client was launched.

The candidate was selected and published as `integrated-20261116` at the user's
explicit request. Deployment results and verification limits are recorded in
`docs/deployment.md`. Existing saves remain
untouched. No gameplay verification is being performed after the user's
explicit instruction to stop it.

## Adventure improvement goal (2026-11-16)

The next accepted goal builds directly on the selected integrated v03 campaign,
not on older review worlds. It covers battle balance, town activities, Reed,
route discoveries and canoe returns, gym clarity, postgame, return dialogue,
native presentation, progression safeguards and build speed. Hidden items are
an explicit part of this goal. The no-gameplay verification instruction remains
in force, including for final release checks.

The `pack_northstar_adventure` recipe combines the following authored changes:

- Forty native hidden items: one on each of the twenty routes, thirteen in
  towns/lake areas and seven in caves. Eight occupy water detours. Each location
  has an explicit tile, approach, entry and surrounding collision contract in
  `discoveries/hidden-items-v01.json`. Native `hiddenitem` events supply
  Itemfinder discovery, full-pocket handling and persistent collection flags.
- Twelve late-campaign party-slot corrections preserve levels, prizes and
  progression while replacing overleveled juvenile partners and ineffective
  move selections with legal active-roster choices.
- Eight optional Champion gym rematches, each with five distinct partners at
  the advertised levels. Pending badge TMs retain priority; each first-win
  Rare Candy has a separate receipt and a full-pocket retry before another
  battle can be offered.
- Sixty-two existing dialogue labels across twenty-four maps are revised,
  including Reed's deliberate choices, real museum activities, gym victory
  handoffs, restoration returns and discovery hints. Brainerd's obsolete
  trainer/menu directions now identify the actual blue winch. The rematches
  add fifty-six canonical text labels.
- Production dependencies can omit pack-authoring modules while pack tools
  retain their existing default feature. Shared gameplay methods and data
  remain available in the runtime library.
- All eight leaders receive their existing accepted native portraits and
  repaired field poses through private trainer classes. This restores the
  supported leader battle music as well. The unrelated river teacher REED is
  renamed ARNE; the rival's name and identity remain intact.

These are implementation records, not a claim of complete campaign balance or
visual/gameplay verification. Existing numbered playtest reports remain open
unless their own acceptance evidence has separately been recorded. Final build,
content identity, selection and publication results are recorded below.

The first adventure candidate builds and reads back identically at
`content-packs/minnesota-adventure-v01.crystalpack`. Content identity:
`d4ce7998673f393e69771a36c3575878060f121102f6db840ddf197c583979dd`.
File SHA-256:
`36da6928a3aa0b0d75c9ad9a7abe353a9d79a7d3d1327eed2dc29d4fc4adb181`.
Independent static comparison confirms all forty new hidden items (forty-two
total), eight rematch teams, forty-eight added native art files, exact original
art/audio/event constants, identical terrain and unchanged non-Minnesota maps.
The Rust verifier passes all 602 audio PCM hashes, frame counts and loop ranges.
The canonical native-page compilation/protection test passes. Formatting and
the initial all-target asset Clippy pass succeeded; existing warnings remain.
The adventure revision is selected in `world.json` and published as
`adventure-20261116` from implementation commit
`c44d317df809d7d41e2980d1550eb6a330f89b86`. Fresh production client/server and
pack builds, all-target asset Clippy and all-target WASM worker Clippy passed;
existing warnings remain. Public checks matched 148 assets and both gzip HTML
pages, verified the service routes and matching WebSocket identity, and checked
both native downloads, the launcher and versioned pack against their manifest.
macOS and Linux each passed two silent `--no-play` installs with zero output and
preserved scratch save markers. No game client was launched.

The [release record](../../docs/deployment.md#minnesota-adventure-release-2026-11-16)
distinguishes the raw pack above from the hosted identity and checksum after the
standard server wrappers. An initial candidate permission issue was fixed before
rollout; no production rollback was needed. Existing saves were preserved without
migration. No gameplay, controller journeys or visual review was performed. These
build and static/content results do not establish complete campaign balance or
playability.

## Refinement goal (2026-11-17)

The accepted ten improvements extend the selected adventure v01 release. The
`pack_northstar_refinement` recipe preserves old map geometry, original actor
slots, event constants, audio and non-Minnesota maps. Save migration is limited
to the exact integrated/adventure raw and hosted releases, with full target
state validation and an independent destination slot.

Implementation covers:

1. Checked prior-release save migration and backup recovery that preserve the
   original primary, backup and earlier release slots.
2. Grand Marais late-campaign supplies and early-route healing/Repel access.
3. Distinct tactical AI profiles for each Gym Leader.
4. A shared move reminder costing 500 only after successful learning, plus
   renewable TMs and held battle tools in Minneapolis.
5. A campaign-wide static assessment of all three starters, experience,
   resources and counterplay, with targeted legal team and supply corrections.
6. Six location-specific hidden-item clues and six upgraded detour rewards,
   retaining the original forty collection receipts.
7. An eight-leader rematch circuit with missing-leader feedback, differentiated
   first-win prizes, and a one-time completion reward with full-Pack retries.
8. An optional Champion-level Reed battle that preserves his hostile character.
9. Distinct interactive Minneapolis model and St. Paul river-chart activities,
   preserving the museums' existing clue rewards and collection flags.
10. Visible shore restoration, a player-controlled Sera radio handoff, and
    progress-aware return dialogue without granting rescue progress by talking.

The selected raw candidate is
`content-packs/minnesota-refinement-v02.crystalpack`, content identity
`a9f23e3bc1dadad74c377d02c23df61dcf9ed70c1b1d5e3918805593f37a9527`,
file SHA-256
`5b173130deed7dc9c2af97b4bc2509e4f54f848c207361ff5278e21abce513ad`.
Build/readback, independent static content comparison and all 602 Rust audio
PCM/frame/loop checks pass. Both native object-entry and Continue sprite
callbacks reconcile the appended shore actors for old saves. The hosted pack's
only data changes are the standard clock/customization wrappers.

Core legality, transactional teaching/payment, save recovery and compatibility,
rematch reward ordering and canonical-page unit checks pass. Migration clears
only completed text/end presentation records in the new copy, after rejecting
active scenes and pending work; original progress and prior files are retained.
Native graphical/terminal builds, both production browser WASM checks, the lean
runtime check, workspace formatting and all-target Clippy pass. Existing
warnings remain; the changed code introduces none. Publication is pending.
Verification is restricted to builds, pure unit tests and static/content/release checks.
No gameplay, controller journeys or client launches are authorized for this goal.
