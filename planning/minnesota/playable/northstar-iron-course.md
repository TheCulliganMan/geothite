# Earned Pine report → mine custody → Harbor reveal

The [543-map Itasca pack](itasca-story.md) now has a continuous production-input
course from the local Brainerd setup through Duluth's ten-page reveal. This
audit changes test/replay support and corrects planning status; the published
game pack is unchanged. Its SHA-256 remains
`75e31a904065d00275b4ac932d38c77346485112e856ca5222d29b50055f8fc1`,
content identity
`4b9dbd5018544719027de20886cafd1a662699d69c9e39e5f3a8562cc8be8bc2`.
The earlier 377-program audio verification therefore describes these same bytes.

## One continuing party and story state

The only initial setup is the existing local Brainerd fixture: six prior badges,
one legal level-45 Mew with its four specified moves including Cut, placement
and the initial replay divider. No partner, HP, PP, money, position, badge,
quest flag or RNG is rewritten after setup. Later Continue uses production saves.
This is not an earned Winona start or an ordinary-party balance claim.

The course actually:

1. Walks R07/Leech Lake/R08, uses care, receives HM01 and plays the Pine gym's
   trainers and Cut gates. It loses the first leader attempt with Cut, receives
   ordinary clinic recovery and money loss, continues and wins the Fire retry.
2. Walks R14 through the Ilex–Azalea gatehouse, acknowledges the marker and
   returns. R09 refuses the raw reading. Both officer pages are acknowledged
   before the report, then the same party enters the Iron Range.
3. Enters the real R09 mine from `(40,42)`, checks the locked cache, defeats
   the actual Graveler/Machoke/Magneton foreman and continues. Victory sets
   only his defeat. Separately accepting and acknowledging the cache recovers
   the meter/test log; depot clearance remains false.
4. Exits the mine, walks R09 and enters Grand Rapids. Free care restores HP/PP.
   The east checkpoint still refuses the carried meter: the actual depot
   handover and its reading must occur before clearance. Continue retains it.
5. Walks R10 to Duluth, uses care and visits the workshop before Harbor. Its
   receiver refuses the missing eighth badge without setting the signal.
6. Plays Livra, Tordel and Anver, both bay releases and the physical ice course.
   Court opening awards no badge. It returns through the real facade for care,
   re-enters, defeats Norek and receives Harbor plus exactly one Icy Wind TM.
7. Continues, uses care and enters the actual workshop at `(48,35)`, safe return
   `(48,36)`, room arrival `(8,9)`. From `(10,5)` it faces receiver `(11,5)`.
   All ten canonical pages are checked, including Boundary Waters drainage,
   Kettlurk powering the pumps, ICE's kidnappings and the prior warnings.
   Movement cannot acknowledge pages. The signal remains false until the last
   acknowledgment; bag, party, money and badges stay unchanged by the reveal.
8. Continues with the signal and sees its truthful repeat direction. Survey
   restoration remains false: the chief and separate receiver action are later.

The final full course passes in 87.35 seconds. A preceding complete run passed
in 75.75 seconds. Selected attacks spend actual PP, native battle results award
the victories, and no strong reserve partner is inserted after Pine. The existing
two-partner eighth-gym approach also passes after the shared route helper change
(32.38 seconds); it remains a separate fixture with a supplied report.

## Custody survives a played loss

The first long course blacked out on R09 after the foreman and cache. Its old
route helper then assumed a route map and panicked when it observed Bemidji's
clinic. That failed run is not counted as completed progression. The helper now
reports a native recovery instead of parsing a clinic name as chunk coordinates;
the continuous course allows bounded ordinary clinic/road retries.

A focused second case loads the **actually earned meter save**, without supplying
its flags, walks to real R09 grass `(54,42)/(55,42)` and plays encounters with
the partner's already learned Cut until an actual loss. No HP, opponent or RNG
is edited. Native blackout returns to Bemidji `(10,13)` with money loss and
restored HP/PP. Pine, reading, report, foreman victory and meter custody persist;
depot clearance and harbor signal remain false. Bag/badges remain unchanged.
Continue retains that state, and the real R09 checkpoint permits re-entry.
This case passes in 5.45 seconds.

## Actual stdio and limits

Four actual-client replays pass from these earned saves:

- Foreman win → decline cache recovery → accept and acknowledge it → repeat.
- Completed depot handover → repeat instruction → real east checkpoint/R10
  entry and free return to Grand Rapids.
- Harbor-earned workshop save → all ten exact MS10 pages, movement ownership,
  final acknowledgment and repeat direction.
- Custody-blackout Continue → clinic exit → report-qualified R09 re-entry.

Each uses public buttons/movement, opens Start, preserves money/badges and exits
with byte-empty stderr. The replay explicitly faces the interaction target again
before each fresh conversation. These are bounded scene/checkpoint replays,
not a second complete MCP battle campaign.

Changed Rust leaves pass formatting. All-target Bevy Clippy completes under the
four established allowances with pre-existing warnings and no new course-module
warning. Global formatting retains the existing offset-allocator difference.
No art, audio, encounter table, authored party, map geometry or pack is changed
by this audit, and no new visual acceptance is claimed.

The remaining mandatory geographic gap after the signal is concrete: the native
Grand Marais slice has only its six local doors and no R11/R12 bindings. Those
route ends likewise do not connect to it. Boundary Waters has its own local
chief/receiver proof, but that is not continuous access from this course. Ely
is absent from this frozen pack. North Shore/Gunflint links, Kettlurk runtime
integration, League, remaining branches/rail and full earned progression remain
unfinished. The goal stays active.

## Executed commands

```sh
export CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export RUST_MIN_STACK=67108864
export CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-itasca-story.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-iron-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  northstar_iron_course_earns -- --ignored --nocapture
export CRYSTAL_NORTHSTAR_CUSTODY_SAVE="$PWD/.local/northstar-iron-review-saves/winona-northstar-earned-meter-40282-1793099235504134000.crystalsave"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  northstar_iron_course_played_wild_loss -- --ignored --nocapture
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  gym_eight_approach_controller -- --ignored
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_northstar_iron_mcp.py \
  --mode cache --save .local/northstar-iron-review-saves/winona-northstar-earned-foreman-39767-1793099119989698000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_northstar_iron_mcp.py \
  --mode depot --save .local/northstar-iron-review-saves/winona-northstar-earned-depot-39767-1793099125133209000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_northstar_iron_mcp.py \
  --mode reveal --save .local/northstar-iron-review-saves/winona-northstar-reveal-ready-39767-1793099153722911000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_northstar_iron_mcp.py \
  --mode custody-loss --save .local/northstar-iron-review-saves/winona-northstar-earned-custody-loss-41645-1793099511963190000.crystalsave
```

All packs, saves and verification output stay ignored. Source tests and replay
tools remain in this repository; no generated game content is committed.
