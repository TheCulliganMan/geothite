# Northstar remote feed and relay shutdown

The ignored `content-packs/minnesota-northstar-relay-v2.crystalpack` extends the
[selected-gym story branch](../gyms/current-regional.md) with the existing
reviewed species/move catalog and a native Kettlurk monitor scene. It contains
554 maps, 474 species, 270 moves and 377 original audio programs. Its content
identity is `5328454f7ea4e3b138d4dea6e98e527b1c432b189b0157eb865b62740c15d446`;
file SHA-256 is `c4d6bcc668c442f65d98360d2fa6f5d4e07d13091d4882db716fbab15423eaff`.

This is a new identity. Earlier earned campaign saves must not be relabeled or
loaded as this branch's progress. A fresh New Game course and its actual saved
continuations now earn four badges, the ICE rescue/evidence and the courier's
separate manifest on this exact revised-gym/species branch. The complete
Winona-to-League campaign remains open.

## Dialogue quality revision — 2026-10-27

The separate ignored `content-packs/minnesota-northstar-dialogue.crystalpack`
contains the original dialogue revision described in [STORY.md](../STORY.md#dialogue-revision--2026-10-27).
Its parent is the completed supplies pack recorded later in this runbook. The Rust publisher reads
`planning/minnesota/story.json` and updates existing native text programs,
parsed text and split script mirrors. It does not alter executable commands,
party balance, progression, geometry, art, items or audio.

This is the final stage when rebuilding the reviewed story. The earlier
`refresh_northstar_story` utility predates the inserted relay feed and is not
the publisher for this successor. The following command has been executed,
and its written pack has passed Rust runtime verification:

```sh
CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
RUST_MIN_STACK=67108864 cargo run --locked --profile wasm-dev \
  -p crystal-assets --bin pack_northstar_dialogue -- \
  content-packs/minnesota-northstar-supplies.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-northstar-dialogue.crystalpack
```

The final source was republished with the compiled Rust publisher:

```sh
target/northstar-story/wasm-dev/pack_northstar_dialogue \
  content-packs/minnesota-northstar-supplies.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-northstar-dialogue.crystalpack
```

The resulting content hash is
`1c66ba1d8c4d06daa960a036691f03681f030e18c40f740e4a4de9a1a402fd54`;
file SHA-256 is `1460390d5446370b019992a722eacf24703402c26b528ed1952c94a107a8a4c7`.
The final consistency pass removed an implied sound from a written log and a
narrated chief movement that the native actor does not perform. The depot now
points to repeated gaps; the chief guards the controls without an invented step.

The publisher rejects tracked outputs and source aliases. Reapplying the same
canonical dialogue is idempotent. This pack has a new content identity; all
historical earned and boosted saves continue to belong to their original
packs. On this exact dialogue identity, all six native story cases pass (415.47
seconds): fresh Winona/Rochester delivery and return; the optional Mankato
notebook; the ICE battle, five-page rescue, separate three-page record and
home return; the courier's three confrontation/manifest pages and Continue;
the nine-page harbor signal; and the chief/relay picture, refusal/retry,
Continue, shutdown and reentry. Initial access and parties in the local chapter
fixtures are supplied explicitly. The Rochester case starts from New Game.
A full new campaign and unboosted balance are not claimed.

The two Rust source/preservation cases pass in 95.52 seconds. They reject bad
scene locations, missing/duplicate support and invalid native lines; compare
the complete dataset with only approved text changes; reparse native bodies;
check split text mirrors, runtime files and all compiled audio; and verify
idempotence and the written identity. All non-text executable programs remain
exact. Six scene checks are run with the following native test selectors:

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export RUST_MIN_STACK=67108864
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-dialogue.crystalpack"
export CRYSTAL_MINNEAPOLIS_TEST_PACK="$CRYSTAL_WINONA_TEST_PACK"
export CRYSTAL_DULUTH_STORY_TEST_PACK="$CRYSTAL_WINONA_TEST_PACK"
export CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK="$CRYSTAL_WINONA_TEST_PACK"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-dialogue-reviewed-saves"
export CRYSTAL_MINNEAPOLIS_SAVE_DIR="$CRYSTAL_WINONA_SAVE_DIR"
export CRYSTAL_DULUTH_STORY_SAVE_DIR="$CRYSTAL_WINONA_SAVE_DIR"
export CRYSTAL_BOUNDARY_BASECAMP_SAVE_DIR="$CRYSTAL_WINONA_SAVE_DIR"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib -- \
  --ignored --test-threads=1 --nocapture \
  rochester_new_game_walks_r01_delivers_acknowledged_originals_and_returns_home \
  prairie_story_local_route_notebook_rooms_return_and_continue \
  minneapolis_controller_actual_win_rescue_retry_open_cell_note_and_return_home \
  courier_story_actual_win_separate_manifest_pages_repeat_and_continue \
  duluth_story_controller_names_kettlurk_requires_every_page_and_retains_only_signal \
  northstar_relay_picture_owns_input_and_console_stays_off_after_continue_and_reentry
cargo test --locked --profile wasm-dev -p crystal-assets --lib story_revision \
  -- --include-ignored
```

Actual stdio replays pass from these same-pack native checkpoints:

| Mode | Checkpoint under `.local/northstar-dialogue-reviewed-saves/` |
| --- | --- |
| rescue | `ms03-enforcer-win-96368-1793141193384556000.crystalsave` |
| harbor | `ms10-ready-96368-1793141158213313000.crystalsave` |
| relay | `q07-relay-chief-win-96368-1793141045139517000.crystalsave` |

Run `planning/minnesota/tools/check_northstar_dialogue_mcp.py --mode MODE
--save CHECKPOINT` with `.local/minnesota-review-env/bin/python`. Its default
pack is the final dialogue pack. Each replay compares every displayed page
with the canonical lines, rejects movement while text owns input, checks
repeat/Start behavior and unchanged money/badges/dex, and requires byte-empty
stderr. The relay includes the actual picture and separate shutdown. A
30-second initialization timeout during concurrent builds was retried with a
90-second harness timeout; all correctness assertions stayed intact. Other
MCP checkers retain their 30-second default.

Final evidence is in `.local/northstar-dialogue-reviewed-native.log`,
`northstar-dialogue-reviewed-assets.log` and the three
`northstar-dialogue-reviewed-{rescue,harbor,relay}-mcp.log` files. Earlier
intermediate fixtures were removed; the completed supplies campaign saves
were preserved.

Scoped Rust formatting and affected-crate all-target Clippy pass with existing
warnings. Clippy uses the previously recorded allowances for `never_loop`,
`bad_bit_mask`, `approx_constant` and `useless_conversion`; no warning names
the new dialogue modules. Workspace-wide `cargo fmt --all -- --check` still
fails in unrelated existing code, beginning at `crates/offset-allocator/src/lib.rs:408`.
No pack, build product, source checkout, generated audio or verification output
is staged or committed.

## Earned revised-gym campaign — 2026-10-27

The fresh opening course passes in 270.32 seconds: Winona New Game, original
delivery, native capture/training/evolution, Care, the Mankato notebook,
Prairie, R03, ICE victory, separate captive release and dispatch reading, then
the dispatcher's home. No party, position, level, badges, resources, quest
results or RNG are supplied. Ordinary blackouts, partner switching, free
clinic recovery and Save/Continue retain earned progress. The caught partner
finishes battles after the starter faints; this is a party victory, not a
claim that the starter alone can win every battle.

The actual dispatcher-home save then passes the 55.97-second Current/R04
continuation. Both gym trainers, the staged native switches, Avrin, the badge
and ThunderPunch are earned before the ordinary road reaches St Paul. The
earlier Headbutt and Mud-Slap remain learned. These are actual saved
continuations with matching pack identity, not migrated historical saves.

The same party then passes the 80.54-second courier/Capitol/return course:
actual courier victory, three separately acknowledged manifest pages, both
Capitol qualifiers, Brannic, Steel Wing and the real R04 trip back to Minneapolis.
The native same-map pads carry the player through the revised gym. At the final
continued save the party has four badges and the manifest; depot clearance,
harbor signal and survey restoration are still unset. No later plot result is
supplied. Wind onward remains unearned on this branch.

`northstar_earned_gyms.rs` reads the selected native room contract and verifies
its published geometry, arrival and trainer bodies. Rochester uses the real
lower-left, upper-open and lower-right switch sequence; Prairie uses its
stair/ledge course. The initial pre-delivery Rochester visit checks the closed
lane and guide; the separate local prerequisite cases cover the leader's parcel
refusal. Every move and interaction uses `VisibleShellController::press`.
The historical room branch remains available to its earlier tests.

Executed commands (the following save is produced by the preceding course):

```sh
export RUST_MIN_STACK=67108864
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-relay-v2.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-relay-earned-saves"
cargo test --lib --locked --profile wasm-dev -p crystal-bevy \
  northstar_campaign_earned_party_defeats_ice_and_releases_dispatcher \
  -- --ignored --test-threads=1 --nocapture
CRYSTAL_NORTHSTAR_CURRENT_SAVE="$PWD/.local/northstar-relay-earned-saves/winona-campaign-earned-ice-dispatcher-home-14933-1793126732969768000.crystalsave" \
cargo test --lib --locked --profile wasm-dev -p crystal-bevy \
  northstar_campaign_earned_current_badge_and_walks_to_st_paul \
  -- --ignored --test-threads=1 --nocapture
CRYSTAL_NORTHSTAR_COURIER_SAVE="$PWD/.local/northstar-relay-earned-saves/winona-campaign-earned-st-paul-arrival-16260-1793126935458695000.crystalsave" \
cargo test --lib --locked --profile wasm-dev -p crystal-bevy \
  northstar_campaign_earned_courier_manifest_capitol_and_return \
  -- --ignored --test-threads=1 --nocapture
```

Logs are `.local/northstar-relay-earned-opening-ice-revised.log` and
`.local/northstar-relay-earned-current.log`, plus
`.local/northstar-relay-earned-courier-capitol.log`. The first pre-parcel room attempt
failed because its test still expected the historical arrival `(6,17)` rather
than `(6,11)`; correcting the input course required no game or pack change.

The actual ICE victory checkpoint is retained as
`.local/northstar-relay-earned-saves/ms03-enforcer-win-14933-1793126727275505000.crystalsave`.
The production stdio checker passes its refusal/retry, separate rescue,
opened cell, ICE/Lode record, return home and Start with byte-empty stderr:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_minneapolis_mcp.py \
  --pack content-packs/minnesota-northstar-relay-v2.crystalpack \
  --save .local/northstar-relay-earned-saves/ms03-enforcer-win-14933-1793126727275505000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_courier_story_mcp.py \
  --pack content-packs/minnesota-northstar-relay-v2.crystalpack \
  --save .local/northstar-relay-earned-saves/winona-campaign-earned-courier-win-16517-1793126998364136000.crystalsave \
  --earned-campaign-win
```

The second actual stdio replay passes all three manifest pages, repeat, city
return and Start with unchanged money/badges and byte-empty stderr. Their logs
are `.local/northstar-relay-earned-ice-mcp.log` and
`.local/northstar-relay-earned-courier-mcp.log`. The final four-badge save is
`.local/northstar-relay-earned-saves/winona-campaign-earned-capitol-manifest-minneapolis-return-16517-1793127061599374000.crystalsave`.

The pack SHA-256 remains the value above. The four-badge verification changes
adapt input courses without supplying game state or changing maps, opponents,
difficulty or audio. One live-RNG earned course does not establish all-starter balance.
Bevy all-target Clippy with `location-tester` passes with the four allowances
listed below and pre-existing warnings; none identifies these adapted leaves.
Scoped formatting/whitespace checks pass. Workspace formatting still fails
first at the unrelated offset allocator difference described below.

## Earned St Cloud continuation — 2026-10-27

The actual four-badge party walks through Minneapolis's two-badge north check
and all of R05 into St Cloud. Before Wind, its northern officer names Saven,
owns movement through the refusal pages and returns the player to `(24,4)`.
All five R05 trainers and both revised Wind qualifiers are now beaten through
native input. The lower and upper forced-wind courses, clinic recovery,
purchases and Continue retain those victories. No later investigation flag is
supplied. The original postponed-courier road round-trip case also passes on
this exact pack (17.54 seconds) after sharing its recovery-aware route walker.

`northstar_campaign_wind.rs` continues either the prior course or an actual
same-identity four-badge checkpoint in Minneapolis/St Cloud. An initial test
approach to R05's fourth trainer was unreachable; the native reachable west
approach `(28,9)` corrected that input recipe. The first three victories were
retained by continuing their actual saved checkpoint. The first trace records
86.04 seconds; the next records the remaining route trainers, both qualifiers
and four Saven losses in 175.17 seconds.

The present two-partner course then trains through ordinary R05 wild outings
to level 31, learns Flame Wheel through the production prompt, and retains
blackouts and losses. That preparation run also loses to Saven (343.14 seconds).
Selling the unused earned Steel Wing and buying medicine succeeds after the
mart fix below, but that run still loses (133.21 seconds). The native once-only
R05 Super Potion is then collected and used; the last recorded attempt leaves
Noctowl with 6 HP before the backup falls (73.38 seconds). These are failed
leader attempts, not a fifth badge or a Brainerd arrival. They expose an ongoing
party/preparation and input-strategy problem; mandatory-trainer-only balance
is not established. The following direct-damage/priority retry, funded by the
other unused earned TM, also loses (56.88 seconds). The current course needs
further ordinary partner/status preparation before Wind can be claimed.

The latest completed loss checkpoint is
`.local/northstar-relay-earned-saves/winona-campaign-MN_GYM_5-loss-0-23156-1793129008902490000.crystalsave`.
It retains the seven new trainer wins, actual trained party and resource use.
No prior boss loss is erased or relabeled as victory.

### Native TM sale correction

The earned resupply attempt exposed a production mismatch: the sell list
included owned TMs, but `carried_item_quantity` omitted the distinct TM pocket.
That produced an invalid inventory label and no sale quantity. The lookup now
checks that pocket after the other carried stacks. Transactions retain native
prices and use actual carried quantities; the pack identity stays unchanged.

`mart_tm_sale_reads_owned_count_and_waits_for_confirmation` passes in 1.93 seconds
with one initially owned TM and an already-open shop as explicit unit setup.
All subsequent actions use the controller: select the TM, cancel without
mutation, reopen, confirm the quantity without payment, then confirm the sale
for its native half price. The earned campaign also sells its actual TM and
buys Super Potions for their native prices. The existing mart group now passes
19 cases with 3 external-pack cases ignored (73.44 seconds). Its stale source
cancellation check now reads the actual `deterministic_session_settling.rs`
location; the behavior assertion is retained.

A second production issue hid every shop surface from renderer-neutral
observation: the core's retained text-window bit suppressed the live mart
owner. The controller snapshot now projects that owner through the existing
scene entries. Those shared entries expose the real quantity and confirmation
above the underlying inventory. No frontend dispatcher or authoritative
renderer cursor is added. `renderer_neutral_mart_projects_its_live_owner_without_extra_wait_frames`
passes in 2.69 seconds, acknowledging Welcome and selecting SELL, quantity,
confirmation and cancellation through direct `press`, without auxiliary wait
frames between inputs.

The rebuilt production client passes actual stdio `wind-mart` from the earlier
earned loss checkpoint with an unsold ThunderPunch. It checks the owned TM
label/count, quantity prompt, cancellation, shop exit, city return and Start;
money and badges stay unchanged and stderr is byte-empty. The first blank-shop
observation and failed replay remain recorded; the input handlers already
owned the shop, while its view was missing.

After this projection change the full mart group passes 20 cases with 3 ignored
(141.35 seconds). The production stderr-cleanliness regression also passes
(6.45 seconds). Bevy all-target Clippy with `location-tester` and the existing
four allowances passes with pre-existing warnings. The later Awakening input
course is still being checked separately; these passes do not grant Wind.

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/northstar-story \
CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
cargo test --lib --locked --profile wasm-dev -p crystal-bevy mart_ \
  -- --test-threads=1 --nocapture
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_lake_country_mcp.py \
  --pack content-packs/minnesota-northstar-relay-v2.crystalpack \
  --save .local/northstar-relay-earned-saves/winona-campaign-MN_GYM_5-loss-0-21760-1793128772016829000.crystalsave \
  --mode wind-mart --earned-campaign
```

Relevant logs are `.local/northstar-relay-earned-wind.log`,
`northstar-relay-earned-wind-revised.log`, `northstar-relay-earned-wind-prepared.log`,
`northstar-relay-earned-wind-native-sale.log`,
`northstar-relay-earned-wind-route-supply-fixed.log`,
`northstar-relay-earned-wind-priority.log`,
`northstar-relay-tm-sale-regression-quantity.log` and
`northstar-relay-mart-suite-current-path.log`, all under `.local/`.
Shop presentation and actual stdio evidence are
`.local/northstar-relay-shop-presentation.log` and
`.local/northstar-relay-earned-wind-mart-visible-mcp.log`.
The full rechecks are `.local/northstar-relay-mart-visible-suite.log`,
`.local/northstar-relay-shop-stdio-clean.log` and
`.local/northstar-relay-shop-visible-clippy.log`.
The later `brainerd-return` stdio checker is prepared but not counted as passed
until this party actually earns that arrival.

## Published local sleep-recovery successor — 2026-10-27

The ignored `content-packs/minnesota-northstar-supplies.crystalpack` retains the
entire relay candidate and appends the existing Awakening to St Cloud's native
mart. File SHA-256 is
`cca0b30fb4ed24f6902a638546aa3bd106a27b147aea3859c9e7d01ee08028cf`;
content identity is
`0d88655630c5dd2234c85d2450e06c076a7f3b513ddb9a3ade1aaa9c96dd7a0e`.
It retains 554 maps, 474 species, 270 moves and the original 377 audio programs.

The earned attempt found that the older St Cloud stock omitted Awakening;
the first regional seller was Brainerd, beyond the sleep-using Wind leader.
St Cloud's broader inventory contract already named the cure, while its native
clerk stock did not include it. The current town source now appends that one
item as well. The publisher checks the exact parent identity and seven-item
stock, requires the existing ¥250 item with native SLEEP recovery, and appends
it without changing prices, opponents, rewards, terrain, scripts or access.
It grants no medicine, money, party member, badge or progress to a save.

The Rust publisher and written-pack readback pass. The full preservation case
passes in 14.91 seconds: every dataset is exact except the declared mart entry,
runtime files and audio remain equal, the actual written identity matches a
fresh build, and duplicate/stale/wrong-mart/wrong-item/changed-stock/unknown-field
contracts are rejected. Both earlier packs remain intact. The new identity
requires its own earned campaign saves.

```sh
CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_northstar_supplies --bin verify_crystalpack
target/northstar-story/wasm-dev/pack_northstar_supplies \
  content-packs/minnesota-northstar-relay-v2.crystalpack \
  planning/minnesota/northstar-supplies.json \
  content-packs/minnesota-northstar-supplies.crystalpack
CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
cargo test --locked --profile wasm-dev -p crystal-assets --test northstar_supplies \
  -- --ignored --test-threads=1 --nocapture
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-supplies.crystalpack --audio --desktop
```

All 377 PCM hashes, frame counts and loop ranges pass on the written candidate.
Assets and Bevy all-target Clippy with `crystal-bevy/location-tester` and the
four existing allowances pass with pre-existing warnings; no diagnostic names
the new supplies or wind leaves. Scoped formatting and whitespace checks pass;
workspace formatting retains the unrelated offset allocator difference.

The fresh New Game → Wind course ran on this publication with all five
Northstar checkpoint override variables unset, using
`.local/northstar-supplies-earned-saves/`. It earned Care, the notebook, Prairie,
all four R03 trainers and native training, then failed after four ICE losses
(2034.69 seconds). Its last actual clinic checkpoint has a level-22 starter:
`winona-campaign-ice-training-22-7-34638-1793132738330220000.crystalsave`.
No ICE win, rescue or later badge is claimed from that course. Earlier saves
are not relabeled. Its executed command was:

```sh
env -u CRYSTAL_NORTHSTAR_WIND_SAVE -u CRYSTAL_NORTHSTAR_COURIER_SAVE \
  -u CRYSTAL_NORTHSTAR_CURRENT_SAVE -u CRYSTAL_NORTHSTAR_ICE_SAVE \
  -u CRYSTAL_NORTHSTAR_EARNED_SAVE \
  RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/northstar-story \
  CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
  CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-supplies.crystalpack" \
  CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-supplies-earned-saves" \
  CRYSTAL_MINNEAPOLIS_SAVE_DIR="$PWD/.local/northstar-supplies-earned-saves" \
  cargo test --lib --locked --profile wasm-dev -p crystal-bevy \
  northstar_campaign_earned_wind_and_brainerd_arrival \
  -- --ignored --test-threads=1 --nocapture
```

Logs are
`.local/northstar-supplies-publication.log`,
`.local/northstar-supplies-preservation.log`,
`.local/northstar-supplies-new-game-wind.log`,
`.local/northstar-supplies-audio.log` and
`.local/northstar-supplies-all-clippy.log`.

## User-authorized level-75 story review — 2026-10-27

The user requested higher party levels to continue story verification. The
explicit `northstar_campaign_boosted_story_review_ice_to_brainerd` resumes the
actual supplies-pack clinic save above and supplies minimum level 75 through
`CRYSTAL_NORTHSTAR_REVIEW_LEVEL`. The test-only helper uses the pack's native
experience curve and calculated stats, retaining species, DVs, moves, PP,
status and lost HP. Existing fainted partners still need ordinary clinic care.
It checks that flags, badges, money and bag remain unchanged by the boost.
Battles, gates, evidence, rescue, rewards and later travel still require
production input. Boosted saves live separately under
`.local/northstar-supplies-boosted-review-saves/` and receive an explicit
`boosted-review` filename prefix. Earned-course test entry points reject the
boost variable. This review is not unboosted party or balance evidence.

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/northstar-story \
  CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
  CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-supplies.crystalpack" \
  CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-supplies-boosted-review-saves" \
  CRYSTAL_MINNEAPOLIS_SAVE_DIR="$PWD/.local/northstar-supplies-boosted-review-saves" \
  CRYSTAL_NORTHSTAR_ICE_SAVE="$PWD/.local/northstar-supplies-earned-saves/winona-campaign-ice-training-22-7-34638-1793132738330220000.crystalsave" \
  CRYSTAL_NORTHSTAR_REVIEW_LEVEL=75 \
  cargo test --lib --locked --profile wasm-dev -p crystal-bevy \
  northstar_campaign_boosted_story_review_ice_to_brainerd \
  -- --ignored --test-threads=1 --nocapture
```

The review passes in 483.48 seconds on the supplies identity: actual ICE win,
separate release and dispatch reading/home, Current, courier victory and three
manifest pages, Capitol and the R04 return, all five R05 trainers, both Wind
qualifiers, Saven, the fifth badge/Swift and the R06 arrival in Brainerd. No
ranger, independent reading/report, mine/depot, harbor or relay result is
supplied. The latest actual save is
`.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-brainerd-arrival-53274-1793133702492689000.crystalsave`.
The retained log is `.local/northstar-supplies-boosted-story-review-scope-fixed.log`.
The first executable attempt wins the ICE battle but stops at the older
accuracy-drop assertion: boosted Mud-Slap faints its targets before that
secondary effect applies. That assertion remains required by unboosted
courses. The boosted review still requires actual victory and PP spending.
The earlier failed log remains `.local/northstar-supplies-boosted-story-review-party-fixed.log`.

Both actual stdio evidence replays pass on this same supplies identity and the
boosted checkpoints. ICE checks refusal/retry, release, opened cell, separate
dispatch reading, return home and Start. The courier checks all three manifest
pages, repeat, city return and Start with unchanged money/badges. Both require
byte-empty stderr. Their logs are
`.local/northstar-supplies-boosted-ice-mcp.log` and
`.local/northstar-supplies-boosted-courier-mcp.log`.

The Lake continuation passes in 92.12 seconds from that Brainerd save: native
Lorla/Bravel/Sivren victories, both physical winch modes, the sixth badge,
once-only Rain Dance, city return and the ranger's separately acknowledged
report/Surf. The starter gains level 76 through actual battle experience.
Later reading/report, mine, depot, harbor and relay flags remain unset.
Its source compiles in 53.64 seconds and affected
Bevy all-target Clippy passes in 50.13 seconds with the existing four allowances
and pre-existing warnings; scoped formatting passes. The exact compiled test executable
is used after that successful compile, without another source build:

```sh
RUST_MIN_STACK=67108864 \
  CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-supplies.crystalpack" \
  CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-supplies-boosted-review-saves" \
  CRYSTAL_NORTHSTAR_REVIEW_LEVEL=75 \
  CRYSTAL_NORTHSTAR_LAKE_SAVE="$PWD/.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-brainerd-arrival-53274-1793133702492689000.crystalsave" \
  target/northstar-story/wasm-dev/deps/crystal_bevy-0739f9b63aa7a27b \
  northstar_campaign_boosted_lake_and_ranger_report \
  --ignored --test-threads=1 --nocapture
```

The controller log is `.local/northstar-supplies-boosted-lake.log`. Its final
actual save is
`.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-lake-ranger-report-55643-1793133836780249000.crystalsave`.

Actual stdio Brainerd care/ranger refusal/R06 return/Start passes from the same
fifth-badge save, with unchanged money/badges and byte-empty stderr. The first
attempt stopped when a wild encounter interrupted a pending route step. The
checker now mirrors the native road course: retry the direction only when
fleeing leaves the exact previous tile; every completed tile must still equal
the planned target. Unexpected displacement still fails. The first failure
remains `.local/northstar-supplies-boosted-brainerd-mcp.log`; the passing replay
is `.local/northstar-supplies-boosted-brainerd-mcp-settled.log`. No production
game, map, encounter, RNG, save or audio content changes in this input fix.
The sixth-badge ranger stdio replay also passes from the actual gym-return
save, with both report pages, repeat, Start, unchanged money/badges and
byte-empty stderr. Its log is `.local/northstar-supplies-boosted-ranger-mcp.log`.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_lake_country_mcp.py \
  --pack content-packs/minnesota-northstar-supplies.crystalpack \
  --save .local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-brainerd-arrival-53274-1793133702492689000.crystalsave \
  --mode brainerd-return --boosted-review
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_lake_country_mcp.py \
  --pack content-packs/minnesota-northstar-supplies.crystalpack \
  --save .local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-lake-gym-return-55643-1793133831352819000.crystalsave \
  --mode ranger --boosted-review
```

Separately, the older relay identity's actual St Cloud party catches a third
local partner and retains it through Continue in the 170.42-second
`northstar_campaign_earned_r05_recruitment` case. The four prior badges and
trainer flags stay unchanged. Its actual save is
`.local/northstar-relay-earned-saves/winona-campaign-earned-wind-local-partner-50432-1793132754395493000.crystalsave`;
the log is `.local/northstar-relay-earned-recruit.log`. Recruitment is an
optional test course; the boosted review adds no mandatory capture.

## Continued Pine and Itasca review — 2026-10-27

`northstar_campaign_pine.rs` continues the same supplied-level party without
replacing partners or supplying later flags. Its first actual course walks
R07/Leech Lake/R08, reads the empty-net and ICE-pressure witnesses, reaches
Bemidji and checks the early Lake-only Itasca visit through the native
Ilex-Azalea gatehouse. The marker refuses without Pine. That attempt stops in
the old Cut helper after 142.84 seconds because it assumes the first TM/HM
entry is Cut. Its log remains `.local/northstar-supplies-boosted-pine-itasca.log`.

The corrected helper explicitly selects the owned Cut HM, replaces the chosen
Smokescreen through the native forget menu and retains Headbutt. Teaching the
reusable HM must leave the entire bag unchanged, including other TMs. The
existing guide/gift/gym/both-city-exits regression passes in 32.46 seconds;
its Mew100/six-prior-badge setup remains a local fixture. It is recorded in
`.local/northstar-cut-guide-existing-regression.log`.

The actual Bemidji arrival checkpoint retains the first course's played travel
and witness reading:
`.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-bemidji-arrival-58587-1793134366028390000.crystalsave`.
Continuing it with the corrected Cut selection passes in 187.30 seconds:
the early Itasca refusal, actual gift/learning, Moslen and Elvren, both native
hedges, Orlis, the seventh badge/Giga Drain, the victory return, the qualified
marker reading and the officer's separate two-page comparison. Brindel remains
optional. Continue retains each checkpoint. Mine/custody/depot/harbor/relay
flags stay unset. No later result is supplied by the level boost.

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/northstar-story \
  CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
  CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-supplies.crystalpack" \
  CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-supplies-boosted-review-saves" \
  CRYSTAL_NORTHSTAR_REVIEW_LEVEL=75 \
  CRYSTAL_NORTHSTAR_PINE_SAVE="$PWD/.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-bemidji-arrival-58587-1793134366028390000.crystalsave" \
  cargo test --lib --locked --profile wasm-dev -p crystal-bevy \
  northstar_campaign_boosted_pine_itasca_and_independent_report \
  -- --ignored --test-threads=1 --nocapture
```

The passing log is `.local/northstar-supplies-boosted-pine-itasca-hm-selected.log`.
The final actual report save, at Bemidji `(21,20)` with seven badges, is
`.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-headwaters-report-60453-1793134800321312000.crystalsave`.
The retained Pine return is
`.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-pine-city-return-60453-1793134758869932000.crystalsave`.
An earlier filename replacement shortened `pine-learned-cut` to `pine-lcut`;
that actual save remains untouched. The helper now removes only the complete
`-earned-` token when naming boosted checkpoints.

Bevy all-target Clippy with the existing allowances passes in 49.20 seconds,
with pre-existing warnings. Its first rerun exposed an unrelated new interior
builder calling `as_str` on an optional environment; the one-line match now
checks `as_deref()`/`Some`. No pack content changes. The failed and passing
logs remain `.local/northstar-boosted-pine-final-clippy.log` and
`.local/northstar-boosted-pine-final-clippy-fixed.log`. Scoped formatting and
whitespace checks pass.

The actual stdio chapter now passes the gatehouse/sign, marker, separately
acknowledged comparison and repeat, qualified Iron Range entry, complete R09
round trip to Grand Rapids, return and Start. Money/badges stay unchanged and
stderr is byte-empty. Its passing log is
`.local/northstar-supplies-boosted-itasca-mcp-notice-fixed.log`.

The earlier replay stops on interior R09 grass at `(104,35) → (104,34)`;
rebuilding the client reproduces a similar stop in R14. Packed cells are
ordinary grass, with no nearby actor or scripted replacement. Public MCP
diagnostics expose the cause: `observe` shows **Got away safely!**, while
compact `status` reports `dialogue: false`. Pressing A dismisses the notice and
the next direction immediately moves. `mcp_status_payload` now includes the
controller snapshot's visible text as well as its core pending-text wait.
No movement timing, encounter, RNG, script or pack change is needed. The
shared native RUN checker asserts this status/visible-dialogue agreement and
waits for the public flow state to settle.

All earlier logs remain: `.local/northstar-supplies-boosted-itasca-mcp.log`,
`.local/northstar-supplies-boosted-itasca-mcp-current.log`,
`.local/northstar-supplies-boosted-itasca-mcp-flow-settled.log` and the public
observation diagnostic `.local/northstar-itasca-input-diagnostic.log`.

The fixed production client builds in 49.91 seconds. The core external-pack
MCP movement/Start case and tool catalog pass (2 tests, 1 unrelated external
case ignored, 4.91 seconds); the spawned-client Mom/stderr case also passes
(4.91 seconds). The retained log is
`.local/northstar-mcp-visible-notice-regressions.log`.
Geothite all-target Clippy also passes in 1 minute 07 seconds with the existing
allowances and pre-existing dependency warnings; its log is
`.local/northstar-mcp-visible-notice-clippy.log`. Main-file formatting
still reports the pre-existing long external-pack assertion at line 842;
the changed status expression introduces no formatting diff. Workspace
formatting retains the earlier unrelated offset allocator difference.

```sh
CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
  RUST_MIN_STACK=67108864 cargo build --locked --profile wasm-dev -p geothite
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_itasca_story_mcp.py \
  --pack content-packs/minnesota-northstar-supplies.crystalpack \
  --save .local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-pine-city-return-60453-1793134758869932000.crystalsave \
  --mode chapter --boosted-review
CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
  RUST_MIN_STACK=67108864 \
  CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked --profile wasm-dev -p geothite --bin geothite \
  --test stdio_clean mcp_ -- --test-threads=1
```

## Actual scene

[northstar-relay.json](../northstar-relay.json) owns the two new native text
bodies and fixed scene anchors. The player still wins the actual survey-chief
battle, then independently chooses the receiver. NO remains retryable. YES
first waits for alignment acknowledgment, then displays the remote intake feed
and Kettlurk's reviewed native front picture. This is a monitor view, not a
creature standing inside the shelter.

The picture uses Crystal's native grayscale Pokepic window and A/B close path.
Its native button wait retains the source continuation through Continue. There
is no field textbox beneath the picture. Directions cannot move or dismiss it.
Afterward, both cut-link pages require acknowledgment before restoration. The
console screen turns off; the original completion pages coordinate the crews
stopping the drainage before the lakes empty. Repeats do not replay the feed.
Neither faction is redeemed. Capture, a Pokédex record, Safari admission,
payment and items remain unnecessary.

The shelter keeps its 8×6 geometry, all actors, carpets, exterior bindings and
receiver event at core `(12,3)`, approached from `(12,4)`. Private native lab art
preserves every powered terrain pixel and collision quadrant. The shutdown
changes only the screen within the original bezel: source image difference
bounds `(197,53)..(203,56)`. `changeblock 12,2` addresses metatile `(6,1)`;
the tile callback keeps it off after reentry. Other maps and original runtime
files are retained. Two unreachable pre-Q07 placeholder text bodies are removed
from this owned room so cached and uncached scene closures agree.

Physical pump animation, a field apparition and the optional Devil's Kettle
encounter are separate unfinished work. The scene does not simulate emptied
lakes or instant environmental recovery.

## Executed Rust publication

All packs remain ignored. The species pipeline uses the existing reviewed
native assets under `target/minnesota-pokemon/native`; it exports no new source
catalog, ROM, audio program or PCM file.

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_fairy_type --bin pack_minnesota_pokemon \
  --bin pack_northwoods_moves --bin pack_northwoods_animations \
  --bin pack_northstar_relay --bin verify_crystalpack

target/northstar-story/wasm-dev/pack_fairy_type types \
  content-packs/minnesota-current-gyms-story.crystalpack \
  content-packs/minnesota-northstar-relay-types.crystalpack
target/northstar-story/wasm-dev/pack_minnesota_pokemon \
  content-packs/minnesota-northstar-relay-types.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/minnesota-northstar-relay-species.crystalpack
target/northstar-story/wasm-dev/pack_northwoods_moves \
  content-packs/minnesota-northstar-relay-species.crystalpack \
  content-packs/minnesota-northstar-relay-moves.crystalpack
target/northstar-story/wasm-dev/pack_fairy_type moves \
  content-packs/minnesota-northstar-relay-moves.crystalpack \
  content-packs/minnesota-northstar-relay-fairy.crystalpack \
  planning/minnesota/pokemon/fairy-moves.json
target/northstar-story/wasm-dev/pack_northwoods_animations \
  content-packs/minnesota-northstar-relay-fairy.crystalpack \
  content-packs/minnesota-northstar-relay-base.crystalpack
target/northstar-story/wasm-dev/pack_northstar_relay \
  content-packs/minnesota-northstar-relay-base.crystalpack \
  planning/minnesota/northstar-relay.json \
  content-packs/minnesota-northstar-relay-v2.crystalpack
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-relay-v2.crystalpack --audio --desktop
```

The species/gym parent has content identity
`cf56f46118758135b12bb3e03ef644efa246973c015142a00fc13903b42d42a3`.
All 377 PCM hashes, frame counts and loop ranges pass on the v2 publication.

## Controller evidence

`northstar_relay_picture_owns_input_and_console_stays_off_after_continue_and_reentry`
passes in 6.60 seconds after the picture-wait fixes. Eight prior badges, the
harbor signal, an initial legal Mew and a fixed divider are explicit local
fixtures. The chief victory, refusal/retry, feed, picture close, shutdown,
reentry and Continue are actual production input. No progress is supplied after
that initial fixture. This is not an earned regional arrival or balance proof.

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/northstar-story \
CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK="$PWD/content-packs/minnesota-northstar-relay-v2.crystalpack" \
CRYSTAL_BOUNDARY_BASECAMP_SAVE_DIR="$PWD/.local/northstar-relay-v2-review-saves" \
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  northstar_relay_picture_owns_input -- --ignored --test-threads=1 --nocapture
```

The passing log is `.local/northstar-relay-v2-controller-persisted.log`.
Its actual checkpoints are under `.local/northstar-relay-v2-review-saves/`:

- `q07-relay-chief-win-4727-1793124494375771000.crystalsave`.
- `q07-relay-kettlurk-picture-4727-1793124494982984000.crystalsave`.
- `q07-relay-powered-down-4727-1793124495772013000.crystalsave`.

Seventeen script-text unit cases and the saved-continuation validation case
pass. The actual full-data preservation case passes in 15.85 seconds, including
all earlier story/gym/cabin maps, unchanged opposing parties and recovery/audio
through species preparation, exact unrelated datasets/files, collision, cached
versus uncached scene closure and rejection of changed source contracts.

## Production stdio and native review

The rebuilt production client passes both modes below. The second starts inside
the saved picture and closes with B. Both verify observable picture ownership,
blocked directional input, separate cut-link acknowledgment, shutdown repeat
and Start. Money, badges and Pokédex counts remain unchanged; stderr is empty.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_northstar_relay_mcp.py \
  --mode chief-win \
  --save .local/northstar-relay-v2-review-saves/q07-relay-chief-win-4727-1793124494375771000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_northstar_relay_mcp.py \
  --mode picture --close b \
  --save .local/northstar-relay-v2-review-saves/q07-relay-kettlurk-picture-4727-1793124494982984000.crystalsave
```

Headless production captures from the actual picture and powered-down saves
are inspected in `../renders/northstar-relay/v2/`. A third checkpoint,
`relay-powered-observer.crystalsave`, walks from the played chief victory to
the same `(12,4)` north-facing console approach through stdio autosave. At noon,
the matched 640×576 power-state cameras differ only at `(383,231)..(401,240)`:
162 screen pixels. Both states match the exact indoor source palette and pixels.
The first source preview incorrectly used the day palette; the corrected indoor
previews and every original, conversion and rejected pass remain archived.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_northstar_relay.py \
  --output planning/minnesota/renders/northstar-relay/v2 \
  --native-powered planning/minnesota/renders/northstar-relay/v2/native-console-powered-final.png \
  --native-off planning/minnesota/renders/northstar-relay/v2/native-console-off-final.png
```

The picture retains the original Gen II grayscale window, the reviewed hollow
silhouette and native scale. The cabinet frame, keyboard, surrounding wall,
actors and traversal stay intact when the screen darkens. No visible window was
opened for these checks. Physical machinery animation and user visual acceptance
are not established by this monitor scene.

Bevy/TUI all-target Clippy with `crystal-bevy/location-tester`, assets all-target
Clippy and core library Clippy pass with the four existing allowances
(`never_loop`, `bad_bit_mask`, `approx_constant`, `useless_conversion`) and
pre-existing warnings. There are no lint warnings in the new relay leaves.
Core all-target Clippy still fails on missing external radio-program fixtures
and the existing `save_sharing` GameState-default mismatch. No raw command dumps
were restored to satisfy those unrelated tests.

The original grayscale picture test passes (2.79 seconds) after correcting its
stale repository root and using the established supplied-pack helper. The
original New Game → Mom → Elm → rendered starter regression also passes
(50.40 seconds), offscreen with the completed TUI pack. Workspace formatting
still reports unrelated existing differences, starting in
`crates/offset-allocator/src/lib.rs:408`; scoped new Rust leaves and whitespace
checks pass. No unrelated source was reformatted.

## Continued mine, depot and Harbor review — 2026-10-27

The same supplies-pack party now passes the complete seven-badge report to
Harbor signal course in **359.53 seconds**. Its user-authorized minimum remains
75; the starter reaches 77 through actual battle XP. This is supplied-level
story verification, not unboosted balance evidence. No badges, quest results,
money, items, moves or position are injected.

The course uses the actual Bemidji report save listed above. It walks R09,
checks the locked meter, wins the native Lode foreman battle and saves the
victory while custody/depot remain false. The separate meter choice and full
recovery acknowledgment establish custody. Grand Rapids still refuses the east
road until the separate technician handover is acknowledged. After R10 and
free care, the Harbor receiver refuses seven badges. All three selected-room
qualifiers, their native gold gates, Norek, the eighth badge and Icy Wind are
played with the continued party. Clinic returns restore actual HP/PP. Every
one of the ten harbor pages must be acknowledged before the signal is set;
movement is rejected during text. Repeat and Save/Continue preserve the result.

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export RUST_MIN_STACK=67108864
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-supplies.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-supplies-boosted-review-saves"
export CRYSTAL_NORTHSTAR_REVIEW_LEVEL=75
CRYSTAL_NORTHSTAR_IRON_SAVE="$PWD/.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-headwaters-report-60453-1793134800321312000.crystalsave" \
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  northstar_campaign_boosted_mine_depot_harbor_and_signal -- --ignored --nocapture
```

The log is `.local/northstar-supplies-boosted-iron-pack-selected.log`; the first
attempt stopped before gameplay because its command omitted the Winona pack
variable (`.local/northstar-supplies-boosted-iron.log`). Actual checkpoints are:

- Foreman only: `winona-boosted-review-campaign-foreman-win-69193-1793136313745645000.crystalsave`.
- Instrument custody: `winona-boosted-review-campaign-instrument-custody-69193-1793136318294294000.crystalsave`.
- Depot handover: `winona-boosted-review-campaign-depot-handover-69193-1793136377314711000.crystalsave`.
- Harbor city return: `winona-boosted-review-campaign-harbor-city-return-69193-1793136585465755000.crystalsave`.
- Reveal ready: `winona-boosted-review-campaign-harbor-reveal-ready-69193-1793136599149909000.crystalsave`.
- Acknowledged signal: `winona-boosted-review-campaign-harbor-signal-69193-1793136605715406000.crystalsave`.

All are under the separate review save folder. `check_northstar_iron_mcp.py`
passes `cache`, `depot` and `reveal` on those actual saves with
`--pack content-packs/minnesota-northstar-supplies.crystalpack --boosted-review`.
The replays include native refusal/retry, the opened east road and reverse
seam, all ten reveal pages, repeats and production Start. Money/badges remain
unchanged and stderr is byte-empty. Logs are
`.local/northstar-supplies-boosted-meter-mcp.log`,
`.local/northstar-supplies-boosted-depot-mcp.log` and
`.local/northstar-supplies-boosted-harbor-mcp.log`.

The pack remains byte-identical: SHA-256
`cca0b30fb4ed24f6902a638546aa3bd106a27b147aea3859c9e7d01ee08028cf`.
No map, art, audio, game script or pack is changed by these continuation checks.

## Continued coast, chief and relay review — 2026-10-27

The same party walks the full R11 coast, enters Ryan's optional Baptism River
cabin, speaks to Ryan and Saves/Continues indoors. It crosses the native
Ilex Forest–Azalea shore gatehouse and saves its actual Grand Marais arrival:
`winona-boosted-review-campaign-grand-marais-arrival-70221-1793136736070416000.crystalsave`.
The prefix is recorded in `.local/northstar-supplies-boosted-finale.log`;
that first course then stops at the clinic's unanswered native yes/no prompt.
Ryan's cabin itself passes and grants no required story progress.

The corrected clinic accepts free care through the native nurse choice. The
local course now uses the existing floor-path helper with strict map/tile
assertions; the earlier general gym path stopped at `(8,11)` on its approach
to `(15,10)`. After R12, the League checkpoint and separate receiver refuse
before the chief. The free field medic restores actual HP/status/PP. The real
Lode chief battle wins with the continued Typhlosion77/Hoothoot75 party and
spends native PP. The original victory checkpoint is
`winona-boosted-review-campaign-chief-victory-71905-1793137109815761000.crystalsave`.
The chief remains unrepentant, and the League still refuses after his defeat.

The receiver continuation passes in **36.22 seconds**. NO remains retryable;
YES acknowledges alignment, shows the native Kettlurk remote-feed picture and
retains input ownership. Save/Continue preserves that picture. The separate
cut-link pages must be acknowledged before restoration and the actual console
block override are committed. Party/storage, bag, badges and money remain
unchanged by the interaction. Ordinary poison damage on the walk to the console
is accounted before that comparison; the earlier assertion incorrectly included
that two-HP walk. After repeat dialogue and free field care, the player walks
the reopened R13 trail to the actual Ely frontier.

```sh
CRYSTAL_NORTHSTAR_FINALE_SAVE="$PWD/.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-chief-victory-71905-1793137109815761000.crystalsave" \
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  northstar_campaign_boosted_coast_chief_relay_and_ely_trail -- --ignored --nocapture
```

Use the supplies-pack, save-folder and level75 exports from the preceding
section. The passing log is `.local/northstar-supplies-boosted-finale-poison-accounted.log`.
Actual checkpoints in the same review folder include:

- Cabin: `winona-boosted-review-campaign-ryans-house-70221-1793136729775494000.crystalsave`.
- Boundary arrival: `winona-boosted-review-campaign-boundary-arrival-71905-1793137073816938000.crystalsave`.
- Picture: `winona-boosted-review-campaign-kettlurk-feed-72189-1793137198449071000.crystalsave`.
- Restored survey: `winona-boosted-review-campaign-survey-restored-72324-1793137277736849000.crystalsave`.
- Ely frontier: `winona-boosted-review-campaign-ely-frontier-72324-1793137298121524000.crystalsave`.

The stopped runs are retained, including
`.local/northstar-supplies-boosted-finale-clinic-answered.log` (review-helper
compile reference), `...-finale-review-helper-fixed.log` (walking assertion),
`...-finale-floor-course.log` (a phrase spanning native text lines) and
`...-finale-receiver-continuation.log` (the poison-inclusive comparison).
Each continuation loads an actual saved checkpoint on the same pack identity;
none supplies a chief victory, receiver result or League access.

The monitor is the implemented remote scene. This proof does not establish
physical pump animation, a field apparition or an optional legendary encounter.
No capture, seen record or Safari fee/admission is required.

Actual production stdio replays also pass on this party's checkpoints:
`check_northstar_relay_mcp.py --mode chief-win --boosted-review`, the saved
picture with `--mode picture --close b --boosted-review`, and
`check_northstar_finale_mcp.py --mode restored --boosted-review`. All use this
supplies pack. They prove retryable NO/YES, picture Continue and directional
input ownership, both ordinary picture-dismiss buttons across the two replays,
the separate shutdown, repeat dialogue, the reopened League seam and Start.
They preserve money/badges and keep stderr byte-empty; both relay replays also
check the unchanged dex. Logs are
`.local/northstar-supplies-boosted-relay-mcp.log`,
`.local/northstar-supplies-boosted-picture-continue-mcp.log` and
`.local/northstar-supplies-boosted-league-release-mcp.log`.

## Continued League and Hall of Fame review — 2026-10-27

This supplies-pack lineage now reaches native Hall of Fame and postgame
Continue. It is a continued story review with explicitly supplied levels:
minimum75 from the actual fresh two-badge clinic save through the first two
League wins, then minimum100 for the last three matches. It is not an
unboosted campaign or a clean loss-free run. Original saves remain untouched
and no save is migrated between pack identities.

The Ely course crosses the actual R13 seam, accepts the nurse's native care
choice and buys six Max Potions, two Full Heals and one Revive through the mart
using earned money. It retains ¥17088 after buying them. Registration declines
and retries before entering the first room. The party wins Fern and Flint at
levels77/78, then actually loses to Veil. The log
`.local/northstar-supplies-boosted-league.log` retains that228.16-second run.
Its actual second-win save is
`winona-boosted-review-campaign-league-win-2-72615-1793137458239680000.crystalsave`.

The minimum100 continuation loads that checkpoint without supplying any run
counter, battle result, badge, resource or story flag. Ordinary Max Potion use
restores the carried party before matches. The battle input helper can also
use the League party's carried Full Heals and stronger potions through the
native Pack, retaining the earlier campaigns' existing medicine strategy.
Every win must spend PP and advance the authoritative run counter. At the
native level cap XP must remain unchanged; the first100 trial won Veil but
stopped at the old XP-increase assertion (`...-league-level100.log`).

With that assertion corrected, the continuation passes in **518.97 seconds**:
Veil, Reed, Casey, blocked doors before each win, repeat dialogue without new
money, and Save/Continue after each match. Defeating Casey leaves championship
recording pending. The separate teacher interaction records the real two-member
party, clears the active run, sets the champion event and creates the native
Hall of Fame autosave. The native ceremony and credits complete. Loading that
autosave returns to Ely's clinic `(3,5)` with the exact winning party, one Hall
of Fame entry and completed survey; the temporary post-champion spawn is consumed.

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export RUST_MIN_STACK=67108864
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-supplies.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-supplies-boosted-review-saves"
export CRYSTAL_NORTHSTAR_REVIEW_LEVEL=100
CRYSTAL_NORTHSTAR_LEAGUE_SAVE="$PWD/.local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-league-win-2-72615-1793137458239680000.crystalsave" \
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  northstar_campaign_boosted_ely_league_hall_of_fame_and_postgame -- --ignored --nocapture
```

The passing log is `.local/northstar-supplies-boosted-league-level100-cap-checked.log`.
The actual final files in the separate review folder are:

- Native fifth victory: `winona-boosted-review-campaign-league-win-5-75341-1793138451952197000.crystalsave`.
- Ceremony autosave: `boosted-review-league-hof-75341-1793138457365767000.crystalsave`.
- Verified postgame Continue: `winona-boosted-review-campaign-league-postgame-75341-1793138474779686000.crystalsave`.

The actual registered-save stdio replay passes native exit/reset,
decline/retry, re-registration, Ely/R13 walking and Start with unchanged
money/badges and byte-empty stderr. Its checkpoint is
`winona-boosted-review-campaign-league-registered-72615-1793137376611160000.crystalsave`
and its log is `.local/northstar-supplies-boosted-league-registration-mcp.log`.

The existing explicit Mew100/medicine-fixture Hall regression also passes
in109.35 seconds after sharing the native medicine helper. That fixture is
separate from this party's progress; its task-generated autosave was removed.
Bevy all-target Clippy passes with the four established allowances and existing
warnings (`.local/northstar-boosted-cap-final-clippy.log`,2m05s), with no warnings
in the new continuation leaves. Scoped Rust formatting and Python syntax pass.
Full workspace formatting still fails at the existing offset-allocator
difference (`.local/northstar-iron-finale-workspace-fmt.log`).

The actual postgame stdio replay also passes, including free care, new
registration decline/retry, rematch exit/reset, the Ely/R13 return and Start.
It preserves money/badges and leaves stderr byte-empty. Its log is
`.local/northstar-supplies-boosted-postgame-mcp.log`:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_ely_league_mcp.py \
  --pack content-packs/minnesota-northstar-supplies.crystalpack \
  --save .local/northstar-supplies-boosted-review-saves/winona-boosted-review-campaign-league-postgame-75341-1793138474779686000.crystalsave \
  --mode postgame --boosted-review
```

All review packs, actual saves, logs and build products remain ignored. The
supplies pack is unchanged, and no generated content or compiled artifact is
staged. Story continuation through the ending is now verified for this boosted
party; unboosted balance and optional physical legendary/machinery scenes remain
separate work.
