# Earned survey-chief finale and League trail release

The [544-map North Shore pack](north-shore-story.md) now passes the actual
earned continuation from Boundary Waters through the chief, separate receiver
and complete R13 League Approach. This is new integration evidence on the
existing native story, not a new map or pack publication. File SHA-256 remains
`644d6dba7ded59795ab890febcad5d5a63878186743521c74d5acc294101cd8e`, content identity
`ab793e8df7a4f16f4b811aad5e2fc5e34c59fa394c33907f718e5a9fd4ea3c8f`.
All source art/audio, geometry, scripts, items, trainers, HOME and recovery
records remain unchanged. The previous full377 audio audit applies to these
identical pack bytes.

## Actual ancestry, rather than supplied finale prerequisites

The input is the save at Boundary Waters `(60,24)` earned by the complete
Brainerd–Pine–Itasca–mine–depot–Harbor–Grand Marais–Gunflint course. Its original
six badges, Mew45 with Cut, Brainerd placement and divider were explicit local
setup. Pine, reading/report, foreman victory, separate meter recovery, depot
handover, Harbor, every signal page and the regional arrival were played.
The finale test loads that actual save without changing state, party, money,
position, clock, RNG, badges or flags. Chief defeat and restoration begin false.
This does not establish a fresh Winona playthrough or ordinary-party balance.

The old local Q07 cases supplied Mew80/eight badges/signal. Those still cover
individual prerequisite denials and local loss/repeat behavior; this earned
continuation closes the distinct upstream-to-finale integration gap.

## Verified production input

`northstar_finale_controller_earned_chief_separate_receiver_and_league_release`
passes in 29.32 seconds. Every action uses `VisibleShellController::press` and
ordinary presentation waits:

- The southern League checkpoint refuses before the chief encounter. The
  receiver is separately locked. Both reject movement while dialogue owns input.
- The existing field medic restores actual HP/PP for free. At the chief, all
  five canonical admission pages match MS11: retained survey readings, copper
  and nickel under the lakes, known risks, deliberate drainage, ICE taking
  suspected opponents and Kettlurk as the intended pump captive.
- The real native chief battle spends selected Fire/Psychic PP and wins with
  the earned party. Chief defeat is saved independently; restoration stays false.
  Repeat dialogue still says he has not changed his mind. It grants no reward
  or battle, and the League checkpoint still refuses afterward.
- The separate receiver offers a retryable NO. YES still waits for alignment
  acknowledgment before setting restoration. Its canonical completion pages
  report real readings sent, crews stopping the pumps before the lakes are
  emptied, Kettlurk escaping the trap and the Ely trail reopening. This remains
  an offscreen report, not an implemented animated pump or legendary encounter.
- Receipt/repeat/Continue preserve the post-chief party, storage, bag, badges
  and money. No optional lamp repair, legendary capture or Safari admission is
  required. The Safari engine flag remains false.
- After free field care, the player exits via the other shelter carpet, crosses
  the newly qualified checkpoint into `MnR13C00R00 (30,0)`, saves/Continues and
  walks the entire road to `MnR13C00R02 (34,30)`—global `(34,126)`. Encounters
  settle through normal battle Run input. A final save/Continue retains the
  earned progress at the Ely frontier.

Ely is absent from this pack. Its town, lobby, four League rooms, champion,
active-run reset, Hall of Fame and postgame return are still implementation
work. The R13 frontier is not an Ely entrance or a completed League.

## Actual stdio replay

Both `chief-win` and `restored` modes of `check_northstar_finale_mcp.py` pass on
the same pack. They load earned controller saves, repeat the unrepentant chief
or active receiver, decline then accept the separate console where applicable,
cross the real League seam and open the production Start menu. Money/badges
remain unchanged and stderr is byte-empty. These use the existing production
controller path, not a second terminal dispatcher.

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export RUST_MIN_STACK=67108864
export CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-north-shore-story.crystalpack"
export CRYSTAL_NORTHSTAR_BOUNDARY_SAVE="$PWD/.local/north-shore-story-review-saves/winona-shore-earned-boundary-arrival-86046-1793106585607078000.crystalsave"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-finale-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib northstar_finale_controller -- --ignored
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_northstar_finale_mcp.py \
  --mode chief-win --save .local/northstar-finale-review-saves/winona-northstar-earned-chief-victory-93509-1793107447510764000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_northstar_finale_mcp.py \
  --mode restored --save .local/northstar-finale-review-saves/winona-northstar-earned-survey-restored-93509-1793107453271076000.crystalsave
```

The complete-road run also retains
`winona-northstar-earned-ely-frontier-96053-1793107836100966000.crystalsave` in
the finale save directory. Packs/saves remain ignored; no generated content,
render or build product is staged. No map/render was changed in this step.

An unrelated sprite-test compile error (`u16 & i32`) was repaired with the
explicit `31_u16` mask, preserving the intended RGB555 conversion. Scoped Rust
formatting and diff whitespace checks pass. Bevy all-target Clippy with the
location tester and four established allowances passes with existing warnings
and no warning in the new finale/helper modules. Planning validation passes
287 planned maps/88 links, including separate expanded-city sources. Full
workspace formatting retains the known offset-allocator difference.


## Ely successor

The separate [554-map Ely/League publication](ely-league.md) now continues the
regional map beyond this frozen frontier, with five native battles, Hall of Fame
and postgame Continue. Its League tests use explicit prior-access/party/medicine
fixtures; this earlier earned save remains on its544-map identity. No migration
or fresh Winona campaign is claimed.
