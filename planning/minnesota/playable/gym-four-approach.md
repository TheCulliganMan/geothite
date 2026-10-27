# Minneapolis → River Crossing → Brannic's gym

[Complete connected source view](../renders/gym-four-approach/whole-connected-cities-crossing.png) ·
[Native east checkpoint](../renders/gym-four-approach/native-east-checkpoint.png) ·
[Native St Paul arrival](../renders/gym-four-approach/native-st-paul-arrival.png)

The ignored `content-packs/minnesota-gym-four-approach.crystalpack` contains
538 maps and all eight current gyms. Internal content identity:
`d239a69185af96196c3ff96785f106e462cde2f67eff3ebb5e3104245ad0d001`.
It extends the exact River Valley candidate, internal identity
`f98c5446760288142f6e56727f910b585d9dc67cbd9eaec5674c4bc542960821`.
These identities describe pack contents, not file SHA-256 hashes.

The corrected v2 candidate follows the authored R04 road contract: Avrin's
Current Badge alone opens both anonymous crossing wardens. The dispatch record
belongs to the separate [courier confrontation](courier-story.md), which this
focused overlay does not implement. The initial v1 mistakenly applied that
record check at the road; it is preserved only as ignored historical evidence
in `minnesota-gym-four-approach-record-gate-v1.crystalpack`.

Minneapolis's east road reaches the existing R04 river park and St Paul's
actual Gym4 facade. The checkpoint uses native `ENGINE_PLAINBADGE`. Either
state of `EVENT_MN_DISPATCH_NOTE` leaves a badge holder qualified and cannot
qualify someone without the badge. Inbound return to Minneapolis is free.
No progress is granted by the road or its guards.

| Connection | West endpoint | East endpoint | Reciprocal offsets |
| --- | --- | --- | --- |
| New Minneapolis/R04 seam | Minneapolis `(63,24/25)` | R04 `(0,18/19)` | city east `3`, route west `-3` |
| Retained R04/St Paul seam | R04 `(55,18/19)` | St Paul `(0,24/25)` | route east `-3`, city west `3` |

The city forest metatile `(31,12)` changes from native05 to01. Wardens stand at
`(60,24/25)`, outbound triggers at `(59,24/25)`, inbound triggers at `(61,24/25)`.
Denial owns input and returns the player to `(58,y)`. Returning from outside
temporarily removes their bodies, walks through and reconciles visibility.
Both lanes and save/Continue outside the wardens are verified. A complete
walking/Surf flood cannot bypass the guards or reach an unbound perimeter;
the open city and whole river park reach their exact declared mouths.

Only Minneapolis and R04 modules change. The old R03 connection, all St Paul
maps, all eight gyms, R04 terrain/encounters/actors, existing scripts and doors,
numeric flags, trainer catalogs, source art, audio, HOME and recovery remain
exact. Cached modules agree with reconstruction from split payloads. The
builder verifies both lanes in both directions at both ends of R04, including
the previously installed St Paul seam. Modified input identities, prerequisites,
checkpoint coordinates and offsets are rejected.

## Executed publication and tests

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_gym_four_approach -- \
  content-packs/minnesota-northstar-river-story.crystalpack \
  planning/minnesota/playable/gym-four-approach.json \
  planning/minnesota/routes/MN_R04.json planning/minnesota/story.json \
  content-packs/minnesota-gym-four-approach.crystalpack

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets --test gym_four_approach -- \
  --ignored --test-threads=1

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-gym-four-approach.crystalpack" \
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-gym-four-approach.crystalpack" \
CRYSTAL_GYM_FOUR_SAVE_DIR="$PWD/.local/gym-four-approach" \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  gym_four_approach_controller -- --ignored --nocapture --test-threads=1

.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/check_gym_four_mcp.py
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_gym_four_approach.py
target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-gym-four-approach.crystalpack --audio --desktop
```

Two asset preservation/rejection cases pass. Three production-controller cases
cover missing badge with either dispatch state on both lanes, free inbound
return and guard reconciliation after Continue, qualified repeat crossings
without an extra gym-victory or dispatch event, and one continuous city-to-city course.

That course starts with **one explicit Minneapolis placement, the first two
badges, a legal level45 partner and10,000 money**. All later progression uses
production input. It wins the ICE battle, separately releases the dispatcher
and reads the record, then proves that evidence alone still cannot open the
east checkpoint. It earns Lenvi/Jossa's wins, solves Avrin's adjacent switches,
earns badge3/TM, and refills spent PP through the actual free clinic service.
It walks the entire river park, enters St Paul's real facade, defeats Belva,
Orven and Brannic through the pad course, earns badge4/TM, saves/Continues and
walks back to Minneapolis. Selected attacks spend native PP; native experience,
rewards and saved puzzle state are observed. No state, position, party or RNG
is rewritten after the initial setup. Wild encounters use the native RUN menu;
route trainers are avoided through legal paths.

The separate stdio fixture has an **empty party and third badge supplied
as initial setup, with no dispatch record**. It walks Minneapolis → R04 → St Paul, enters/exits the actual
Gym4 facade, opens the production Start menu, and walks home through the same
guard reconciliation. Money/badges stay unchanged and stderr is byte-empty.
Its exported save is `.local/gym-four-approach/qualified.crystalsave`; it is
identity-specific, ignored and supplied only when the save-directory variable
above is set. This proves text-client traversal, not earned qualifications.

The complete connected map and both actual native player views were inspected.
Both1280×800 frames match their noon/south-facing source references exactly:
0/1,024,000 differing pixels each. Existing town proportions and the river park
are preserved; dark areas of the sheet represent unmapped space. All originals,
conversions and latest aliases are archived. This is visual review evidence,
not user approval. All377 audio PCM hashes, frame counts and loop ranges pass.

Changed Rust leaves pass rustfmt. Workspace formatting retains unrelated
differences beginning in `offset-allocator/src/lib.rs:408`. Full affected-crate
clippy remains blocked by the pre-existing always-false bit mask at
`crystal-core/src/multiplayer/mod.rs:1332`. Scoped all-target lint passes for
assets and Bevy with existing `never_loop` findings allowed; no warning names
the changed modules.

## Remaining progression

These local tests use explicit initial fixtures. They do not prove ordinary-party
balance or the complete earned starter-to-League journey. The [courier](courier-story.md), [lake-country](lake-country-story.md) and
[Leech shore](leech-shore.md) successors now implement their separate later
chapters. This focused overlay never awards their completion. Rail and the
complete earned starter-to-League journey remain unfinished.
