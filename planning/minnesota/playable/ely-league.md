# Ely rest camp and native League

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

The [written League finale](../revisions/league-finale-v01/README.md) adds
distinct Fern, Flint, Veil, Halden and Casey scenes, title-aware rematches and
the Hall of Fame ending to the current 691-map regional composition. Its
authoritative dialogue is `revisions/league-finale-v01/dialogue.json`; the
554-map publication and earlier fixture results below are historical.

The `554`-map successor is
`content-packs/minnesota-northstar-ely-league.crystalpack`.
File SHA-256: `ab049e907b8712f5ee5f174b474c38fcc52460f272e60336aff17445d62cb6c4`.
Content identity: `e96cd37fd94ca71f39d72a4a7924fe613ddb1cecf64f12f5a5985095a5fea7d4`.
It builds only from the frozen [544-map North Shore parent](north-shore-story.md),
content identity `ab793e8df7a4f16f4b811aad5e2fc5e34c59fa394c33907f718e5a9fd4ea3c8f`.
It remains a separate lineage from the expanded-city and geography compositions.

Five production controller cases pass on this exact publication in85.00seconds.
The native build, two preservation cases, all377 audio PCM/hash/frame/loop checks
and three exact exterior comparisons pass. Both actual stdio replays also pass
on this identity with byte-empty stderr. The clinic's unused upstairs stair
is replaced with native floor, and League music continues in the record room.
No source export is needed.

## A complete final rest stop

[Ely source](../towns/MnEly.json) and its
[explicit layout](../towns/MnEly.layout.json) compose a `20×18`-metatile forest
clearing with a bending approach, stone League lodge, clinic, shop and garden
loop. Roads are two walking tiles wide, except the useful registration turnout.
All four facade lanes have real warps and separate safe floor returns. The
clinic removes the unused link-club stair with original room floor. There are
no incidental Cut quadrants or reachable unconnected outdoor/Surf edges.

The original **IlexForestAzaleaGate** is the registration room, preserving the
requested Crystal gatehouse art and both side carpets. Its western and eastern
exits lead outside; the attendant offers registration. Every public service is
available before registration: native free healing, PC access, real money/item
checkout and visit-selected blackout recovery. The stock is Ultra Ball, Hyper
Potion, Max Potion, Full Heal, Revive and Max Repel. No free starter or supplies
are granted by the League.

R13's final chunk `(34/35,31)` connects south to Ely `(24/25,0)` with offset
`+5` metatiles; Ely's north connection uses `−5`. Both lanes are checked with
the native connection resolver. The upstream Boundary Waters road checkpoint
remains intact. Registration separately checks all eight native badge flags
and `EVENT_MN_SURVEY_RESTORED`.

## Native identities and room contracts

All new maps use group `81`, `MAPGROUP_MINNESOTA_ELY`. The full coordinate,
sprite, service, exit and passive-arrival contracts are in the town source.

| Map ID | Map | Native floorplan | Metatiles |
| --- | --- | --- | --- |
| 1 | MnEly | Explicit Kanto source art | 20×18 |
| 2 | MnElyPokecenter1F | GoldenrodPokecenter1F; stair patch `(0,3):12→04` | 5×4 |
| 3 | MnElyMart | CherrygroveMart | 6×4 |
| 4 | MnLeagueLobby | IlexForestAzaleaGate | 5×4 |
| 5 | MnLeagueRoom1 — Fern | KogasRoom | 5×9 |
| 6 | MnLeagueRoom2 — Flint | BrunosRoom | 5×9 |
| 7 | MnLeagueRoom3 — Veil | WillsRoom | 5×9 |
| 8 | MnLeagueRoom4 — Reed | KarensRoom | 5×9 |
| 9 | MnLeagueChampion — Casey | LancesRoom | 5×12 |
| 10 | MnLeagueHallOfFame | HallOfFame | 5×7 |

Parties follow `story.json:league`: Fern's four Bug Pokémon at47–48, Flint's
four Rock Pokémon at48–49, Veil's four Ghost-themed partners at49–50, Reed's
four Dragon-themed partners at50–51 and Casey's six partners at51–53. The Rust
builder obtains legal native learnset moves and source-class DVs. Original
trainers and their numeric IDs remain unchanged.

`EVENT_MN_LEAGUE_RUN_ACTIVE` is2310; `EVENT_MN_LEAGUE_CHAMPION` is2311. Persistent
native memory `wMnLeagueRun` stores `0..5` completed members. Registration resets
it and starts a run. A real trainer victory advances it; repeat dialogue cannot
award another battle payout. Native tile callbacks reopen only earned north
doors, including after Continue. South doors allow retreat through earlier
rooms; reaching the lobby ends the run. Blackout selects Ely's clinic and its
entry callback clears only the run. Prior badges, survey completion and Hall
of Fame history survive. No special frontend dispatcher is introduced.

Casey's victory opens the record room but does not itself award championship.
Acknowledging the separate record attendant sets the champion flag, clears the
run and executes the existing `halloffame` command. It records the actual party,
autosaves, plays the native ceremony/credits and returns to title. Continue
consumes the native one-shot marker and returns to clinic spawn280 `(3,5)`;
Ely's outdoor spawn279 is separate. The Hall map declares its recovery using
`GEOTHITE_POST_CREDITS_SPAWN` in map-scoped constants. Direct loading and title
Continue share that path; original Lance/New Bark and Red/Mt Silver constants
and return behavior are retained. A new registration starts a rematch without
erasing prior championship or Hall of Fame records.

## Source and visual review

All originals, conversions, crops and rejected passes are retained under
`../renders/ely-league/history/`. Useful latest aliases:

- [Whole town](../renders/ely-league/ely-whole-source.png).
- [Actual R13/Ely composition at one scale](../renders/ely-league/r13-ely-connected-source.png).
- [Native lodge approach](../renders/ely-league/ely-registration-native.png).
- [Native Ilex gatehouse registration](../renders/ely-league/ely-lobby-native.png).
- [Native record room](../renders/ely-league/ely-hall-native.png).

The initial pale clearing hid the paths; native green lawn and a central grove
now separate the path loop and useful forecourt. Two mistaken roof blocks used
as signs were replaced with original Kanto sign08. A nonexistent Hiker overworld
sprite was replaced with native Pokéfan M. The record room now follows the
source `LoadSpecialMapPalette` exception: Ice Path art uses the common indoor
bank in the Hall of Fame, while real Ice Path caves retain their special bank.
The primary source is the external pinned checkout's
`engine/tilesets/tileset_palettes.asm`. Its pixels and palettes remain external.

Three matched noon, south-facing source/native exterior cameras have zero
differing pixels. All nine indoor rooms and the weakest supply-loop view have
also been inspected in headless native captures. Exact whole-room pixel parity
is not asserted for room cameras with different native viewport cropping.

## Build, review and play

Run from the repository root with the completed parent present:

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_ely_league -- \
  content-packs/minnesota-northstar-north-shore-story.crystalpack \
  planning/minnesota/towns/MnEly.json \
  planning/minnesota/towns/MnEly.layout.json planning/minnesota/story.json \
  content-packs/minnesota-northstar-ely-league.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_ely_league.py \
  --pack content-packs/minnesota-northstar-ely-league.crystalpack --published
cargo build --locked --profile wasm-dev -p crystal-bevy --features location-tester \
  --example render_at_location
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_ely_league.py \
  --pack content-packs/minnesota-northstar-ely-league.crystalpack --published --capture
cargo build --locked --profile wasm-dev -p geothite --bin geothite
mkdir -p .local
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-ely-league.crystalpack \
  --save .local/ely-campaign.crystalsave
```

`--save` supplies the native ceremony/F5 save destination. Resume that campaign
with the same pack plus `--load .local/ely-campaign.crystalsave --save .local/ely-campaign.crystalsave`.

`--capture-view hall` restricts the review capture when iterating on that room.
All captures are offscreen. Packs, extracted game art, build products and saves
stay ignored and are never staged or shipped as source.

## Verification and limits

The controller cases use explicit prior-access fixtures. The winning course
starts with eight badges, a restored survey, Mew100, slight HP damage and five
Full Restores. Native Pack input consumes medicine, restores HP/status and leaves
the run count unchanged. All five battles spend real move PP and earn their run
counts through the authoritative battle engine. Initial fixtures do not prove
a fresh Winona playthrough or ordinary-party balance.

```sh
export RUST_MIN_STACK=67108864
export CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-ely-league.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/ely-league-final-saves"
export CRYSTAL_ELY_HOF_SAVE="$PWD/.local/ely-league-final-saves/ely-native-hof.crystalsave"
cargo test --lib --locked --profile wasm-dev -p crystal-bevy --features location-tester \
  ely_league_controller -- --ignored --test-threads=1
cargo test --locked --profile wasm-dev -p crystal-assets --test ely_league -- --ignored
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-ely-league.crystalpack --audio --desktop
```

Coverage includes both road directions; independent badge/survey refusals;
registration decline/retry; real HP/PP care, PC input ownership and priced
checkout; sequential five wins, repeat guards and per-room Continue; native
recording/credits/postgame Continue and rematch withdrawal; played loss recovery;
original credits destinations; and indoor/cave palette separation. Preservation
cases compare every previous map (only R13's south connection differs), original
constants, all original runtime files, audio, trainers, Pokémon/moves, encounters
and recovery records, plus reconstructed native script payloads.

The registered replay uses the controller-earned save below. The postgame
replay uses the native ceremony autosave:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_ely_league_mcp.py \
  --mode registered \
  --save .local/ely-league-final-saves/winona-ely-registered-23695-1793112230628258000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_ely_league_mcp.py \
  --mode postgame --save .local/ely-league-final-saves/ely-native-hof.crystalsave
```

Both replay modes exercise normal input, registration, withdrawal, the Ely/R13
seam and Start while requiring byte-empty stderr and unchanged money/badges.
The original two legacy `continue_consumes_*` tests require their hardcoded
default pack path, which is absent here; the new external-pack controller case
independently verifies both original destinations on this complete pack.

Earlier [earned finale evidence](northstar-finale.md) remains on its544-map
identity; old saves are not relabeled to load into this successor. A continuous
fresh Winona campaign and Kettlurk's visible pump/escape set piece remain open.
The villain alliance, its kidnapping/drainage plan, Ryan's Baptism River cabin,
and existing Ilex gatehouses are preserved.


Workspace-wide validation has unrelated limits: `cargo fmt --all -- --check`
first reports existing formatting at `crates/offset-allocator/src/lib.rs:408`.
The global plan checker currently stops on concurrently edited fairground
contracts (`MnFairGrandstand`, `MnFairBarn`, `MnFairDairy` exterior coordinates);
its last pre-fair-change pass counted288 planned maps/88 links. Ely's native
geometry, connection, controller and preservation checks pass independently.
Scoped changed Rust formatting and `git diff --check` pass. The combined all-targets Clippy run stops on private-field accesses in the
concurrently edited `crates/crystal-assets/tests/city_recomposition.rs`
(`CompiledGamePack.data`, `runtime_files`, `audio_manifest`). Targeted asset
Clippy and Bevy all-targets Clippy pass separately, with no new warnings in Ely
files. Both use the existing
four allowances (`never_loop`, `bad_bit_mask`, `approx_constant`,
`useless_conversion`); unrelated warnings remain.
