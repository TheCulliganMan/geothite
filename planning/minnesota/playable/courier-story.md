# Minneapolis → St Paul: the DEEP BORE manifest

The local chapter pack is `content-packs/minnesota-northstar-courier-story.crystalpack`:
**538 maps,251 species,251 moves,377 audio programs**. File SHA-256:
`37a569f7cd6cb46bca8c3d34d5deb096b108f8c2db6db38b07408e6add4d64c0`.
Internal content identity:
`58a1dc84c01d3c5d7d0c5efb849a024a5c6ece1daadb525a69055a32c55b4047`.
Its exact input is the [river-story successor](river-cities.md), file SHA-256
`01944a98d5381de24bee3469f0f426a16a08c4e9bf96c4ec891655f2d2a21a85`.
Packs, saves, native captures and every render generation remain ignored.

## Travel and independent story actions

Minneapolis east `(63,24)/(63,25)` connects to R04 west `(0,18)/(0,19)`,
with native offsets `+3/-3`. One modern forest metatile at `(31,12)` becomes
ordinary road. R04's existing St Paul connection stays exact. Two wardens at
`(61,24)/(61,25)` check only Current (`ENGINE_PLAINBADGE`); refusal returns the
player to `(59,y)`, and inbound travel is free. The optional notebook, ICE note,
courier victory and manifest do not become road prerequisites. Complete
walking/Surf bounds and both lanes' native projections are verified.

The existing Union Depot courier is reused at `(20,16)`, approached from
`(19,16)`. The earlier outdoor actor reservation was corrected to this actual
published room and native object `MN_ST_PAUL_DEPOT_COURIER`.

1. **MS04:** Current and the actual recovered ICE dispatch copy are required.
   The three authored threat pages include the one-line “Who has them now?”
   page. Magnemite23 and Drowzee22 use legal native moves, original grunt DVs
   and the real trainer engine. Only an actual win records
   `EVENT_MN_COURIER_DEFEATED` (2306). Defeat uses ordinary money loss and clinic
   recovery; it sets neither chapter flag. The courier remains hostile, repeating
   the original claim that the notes are Lode property. Repeat interactions
   do not replay the battle or its money reward.
2. **MS04_MANIFEST:** A paper object at `(18,16)`, approached from the same
   `(19,16)` tile facing left, is locked before victory. Separately acknowledging
   all three pages names DEEP BORE, the diverted survey equipment and the order
   not to contact the original surveyor. Only then is
   `EVENT_MN_DEEP_BORE_MANIFEST_READ` (2307) recorded. No bag item, money,
   badge, later depot clearance or road permission is awarded. The paper stays
   present and has saved repeat text.

The courier receives unused GRUNTM number34. Every old trainer/event number is
preserved. St Paul's gym can be entered with Current while the courier remains
unfinished; the two city badges still define the planned R05 gate. Rail service
is not implemented by this chapter: the conductor still offers no ride. The
existing courier-victory flag is available for the separately planned rail
activation and does not replace station visitation requirements.

## Executed Rust publication and replay

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_courier_story -- \
  content-packs/minnesota-northstar-river-story.crystalpack \
  planning/minnesota/playable/courier-story.json planning/minnesota/story.json \
  planning/minnesota/routes/MN_R04.json \
  content-packs/minnesota-northstar-courier-story.crystalpack
cargo test --locked --profile wasm-dev -p crystal-assets --test courier_story -- --ignored
export RUST_MIN_STACK=33554432
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-courier-story.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/courier-story-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib courier_story_ -- --ignored
cargo build --locked --profile wasm-dev -p geothite
```

Five production-controller cases pass:

- Both denied departure lanes, directional input ownership, Continue outside
  the wardens, free unqualified inbound return and access to public care.
- Separate missing-Current and missing-dispatch refusals, a locked manifest,
  unchanged party/bag/money and no free chapter progress.
- Current-only R04/Union Depot/fourth-gym travel and return without the ICE note,
  courier victory or manifest flag. Initial party is deliberately empty.
- One continuous populated local trip from Minneapolis through R04, actual
  courier victory with both opponents/selected PP/EXP and native money, separate
  reading with page-by-page input ownership, repeat, Continue and return to
  Minneapolis. Initial three badges, dispatch note and level45 Mew are explicit
  setup. Nothing is teleported or granted afterward.
- A played Magikarp loss, actual clinic recovery, money loss, no defeat/record
  flag and Continue.

Two asset cases pass exact written identity, all three changed modules with
their caches removed, and complete unrelated-data preservation after reversing
only the declared changes. R04 terrain/encounters/actors, St Paul exterior,
every gym, all other story, art, audio, HOME and recovery stay exact. Duplicate
publication, changed anchors/party, premature record completion and an extra
courier road gate are rejected. The courier's original property-claim text
remains reachable in the post-win response; it is not a cached-only label.
The builder also compiles the battle-exit map command, which the first played
win exposed as missing before repair.

Actual stdio passes both the empty-party road/denial/return and the locally
earned courier-win Continue/three manifest pages/repeat/city return. Both open
the production Start menu, preserve badge/money counts and exit with byte-empty
stderr. Exact retained replays:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_courier_story_mcp.py \
  --save .local/courier-story-review-saves/winona-courier-local-road-ready-3456-1793088668253473000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_courier_story_mcp.py \
  --save .local/courier-story-review-saves/winona-courier-local-earned-win-3456-1793088670061038000.crystalsave \
  --earned-local-win
```

The complete connected cities/R04, complete depot, actual Current checkpoint
and actual depot camera were visually inspected. Current source/native noon,
down-facing references match both1280×800 frames exactly,0/1,024,000 pixels
each. The checkpoint camera presents active-map actors with adjacent terrain;
the connected whole shows each map's own actors. The first comparison included
neighbor actors before their map became active; that rejected comparison is
retained. No terrain or player pixels were changed to force the match. Native
source art, all originals, crops and conversions remain under
`renders/courier-story/`.

All377 audio programs pass their PCM hashes, frame counts and loop ranges on
the final file. Scoped changed-leaf Rust formatting passes. Workspace
`cargo fmt --all -- --check` retains unrelated differences beginning at
`crates/offset-allocator/src/lib.rs:408`. All-target Clippy for assets/Bevy
completes with existing warnings under `never_loop`, `bad_bit_mask`,
`approx_constant` and `useless_conversion` allowances; no warning names the
new courier modules. This is not an unsuppressed workspace lint pass.
The plan check passes20 routes,19 hubs,57 chunks,86 adjacency/gate contracts
and162 planned maps, which establishes only the authored symbolic graph.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_courier_story.py
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-courier-story.crystalpack --audio --desktop
```

These are local mechanics and continuous local chapter evidence. They do not
earn three prior badges or the ICE note from New Game. Full ordinary-party
balance, Mankato's Cut/cache, Minneapolis/R05 and later links, rail, later story
chapters, League completion and the Kettlurk scene remain unfinished. User
visual acceptance is also open.

## Earned courier, Capitol and Minneapolis return — 2026-10-27

The separate 554-map [Ely story parent](ely-league.md), file SHA-256
`ab049e907b8712f5ee5f174b474c38fcc52460f272e60336aff17445d62cb6c4`,
now passes `northstar_campaign_earned_courier_manifest_capitol_and_return`
in 225.52 seconds. It continues the actual three-badge St. Paul arrival from
the [earned Current course](gym-four-approach.md#earned-current-and-st-paul-arrival--2026-10-27),
whose party and ICE dispatch come from the recorded fresh Winona game.
No progress, position, party, level, money, items or RNG are supplied.

The course uses St. Paul's actual compact clinic and mart entrances/counters,
registers real recovery, restores HP/status/PP and purchases only affordable
Super Potions. The guarded manifest refuses reading before the fight, retains
input ownership and grants no record. Quilava wins the actual courier battle
at level 22. The victory checkpoint still has the manifest unread. The shared
acknowledgment routine then checks all three canonical pages, rejects movement
during each, preserves party/bag/money, verifies repeat text and continues from
the separately completed record.

The same party defeats Belva, Orven and Brannic, earns Capitol and one Steel
Wing TM, crosses R04 back to Minneapolis with the evidence intact and uses free
care. Quilava finishes the leader fight at level 24. All four fights win on their first
attempt in this run; the course still retains native loss/recovery branches.
The final save has exactly four badges. Mine/depot clearance, harbor signal and
survey restoration stay unearned. This is an earned continuation, not another
independent New Game invocation or proof of every starter/battle outcome.

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/northstar-story \
CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-ely-league.crystalpack" \
CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-current-campaign-saves" \
CRYSTAL_NORTHSTAR_COURIER_SAVE="$PWD/.local/northstar-current-campaign-saves/winona-campaign-earned-st-paul-arrival-73673-1793120113107775000.crystalsave" \
cargo test --lib --locked --profile wasm-dev -p crystal-bevy \
  northstar_campaign_earned_courier_manifest_capitol_and_return \
  -- --ignored --test-threads=1 --nocapture
```

The passing log is `.local/northstar-earned-courier-capitol-rooms.log`.
The actual checkpoints under `.local/northstar-current-campaign-saves/` are:

- Before the separate record: `winona-campaign-earned-courier-win-85786-1793121518116849000.crystalsave`.
- After acknowledgment: `winona-campaign-earned-manifest-read-85786-1793121539313327000.crystalsave`.
- Four badges, record and care in Minneapolis: `winona-campaign-earned-capitol-manifest-minneapolis-return-85786-1793121697369773000.crystalsave`.

The rebuilt production client also passes the actual stdio record replay from
that earned victory checkpoint. It checks the hostile repeat, all three pages,
directional input ownership, record repeat, city return and Start, with unchanged
money/badges and byte-empty stderr:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_courier_story_mcp.py \
  --pack content-packs/minnesota-northstar-ely-league.crystalpack \
  --save .local/northstar-current-campaign-saves/winona-campaign-earned-courier-win-85786-1793121518116849000.crystalsave \
  --earned-campaign-win
```

The stdio log is `.local/northstar-earned-courier-mcp.log`. The original local
fixture case also passes after extracting the shared acknowledgment routine
(10.72 seconds); its saves remain separately in
`.local/northstar-current-review-saves/` and do not establish earned progress.
The game pack and audio are unchanged. The later current-gym publication has a
[different identity](../gyms/current-regional.md); these parent saves must not
be relabeled or treated as earned progress on that successor. Wind onward,
the remaining full campaign and Kettlurk's visible scene remain open.

Final source checks pass: the plan validator, scoped Rust formatting, JSON
parsing and `git diff --check`. Bevy's default-feature all-target Clippy check
passes with the existing `never_loop`, `bad_bit_mask`, `approx_constant` and
`useless_conversion` allowances; it reports no warnings in the campaign or
shared courier helpers changed here. The log is
`.local/northstar-earned-current-courier-clippy-reviewed.log`. Workspace
`cargo fmt --all -- --check` still reports unrelated existing differences,
starting at `crates/offset-allocator/src/lib.rs:408`; this work does not reformat
those files.
