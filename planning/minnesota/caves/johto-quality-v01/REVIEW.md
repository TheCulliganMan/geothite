# Crystal reference cave pass

The completed eleven-room pack is
`content-packs/minnesota-voyageurs-ice-v02.crystalpack` (683 maps), SHA-256
`546f17a16db8e47b409e870f821a826dd5a9b9f6d8dd0c526d8cf5ad397c9f0f`.
Its nine revised stone caves were published as
`content-packs/minnesota-cave-quality-v01-09.crystalpack`, built from
`content-packs/minnesota-cohesion-r01-upper-loop-v01.crystalpack`.
The nine JSON files here are explicit terrain revisions; their `.previous.json`
partners record the installed preimages required by the Rust publishers.
Original residents, item rewards, story programs and exterior mouth pairings
are retained. Generated packs and images remain ignored.
The nine route dossiers now contain these same cave definitions. Source
promotion changed only rows, border and review prose/contracts; the retained
room fields were checked against the recorded preimages. The prior dossiers
and promotion receipt remain under the ignored render history at
`source-promotion-20261027T193902Z/`.

The reference study uses the original Union Cave, Slowpoke Well and Mt. Mortar
maps in the supplied pack. Native cave wall faces, corners, ordinary stairs,
rock shoulders and water banks determine the visual vocabulary. The dark-room
reference is not evidence for visible detail while its darkness palette is active.

| Room | Composition |
| --- | --- |
| R01 hollows | Separate guide and Antidote shelves around a stone spine; two ordinary stairs and a short lower return. |
| R03 bank cave | Small seep beside the entrance, attached rock nose and blind reward bend. |
| R05 Rock Cut | Two retained mouths joined by a bent neck, a fitted healer turnout and one stair. |
| R09 mine | Three offset working benches above a turning entrance shaft; original foreman and survey cache. |
| R10 gallery | Narrow upper gallery, split ore spine, stepped spring bank and separate Revive shelf. |
| R12 shelter | Forked shelter and cache ascents above the lower seep basin and short drop return. |
| R13 refuge | Spring bank leading to a protected healing terrace and a separate Ether notch. |
| R17 tor | Brief bent passage around an attached stone tongue to the original Revive. |
| R20 cleft | Two mouths below a crossed stair route, supply shelf and small banked pool. |

Whole rooms and player views are retained under
`planning/minnesota/renders/caves/johto-quality-v01/`, including rejected passes.
`candidate-v08/all-nine-whole.png` shows all rooms at the same native scale.
Whole-room composition and production-frame equality are separate checks:
matching pixels establishes the rendering comparison, not enjoyable exploration.

## Verification scope

The stone-cave pack contains 681 maps. Its native audio verification passes all 377
programs' PCM hashes, frame counts and loop ranges
(`.local/cave-quality-audio-v01.log`). Source readback confirms all nine published
block arrays and solid borders match these candidates
(`.local/cave-quality-published-v01.log`).
There are 68 declared camera positions, 17 ordinary stairs and 11 cave mouths.
The earlier source-only regional plan check passed. The latest shared-checkout
check now stops in the city recomposition validator's original-room preservation
assertion (`.local/cave-quality-source-plan-turn3-v01.log`). Comparing the entire
current plan with the stone pack also stops on a city block-array mismatch
(`.local/cave-quality-current-source-plan-v01.log`). Neither is a passing current
full-region comparison; the cave-specific source, preservation and controller
checks are separate.

All 68 production camera captures pass with zero differing pixels across each
complete 640×576 frame. The report is
`renders/caves/johto-quality-v01/native-v01/capture-progress.json` relative to
`planning/minnesota/`; the final comparison execution is
`.local/cave-quality-native-v02.log`. All nine production camera sheets were
visually inspected alongside whole-room layouts and the Crystal references.
The pack SHA-256 is
`131787bb92d47270c7cd779d39ba783f0a4c6547835a136e6a217a5ddd3c4b2d`.
An inventory of the actual installed Minnesota maps confirms these are all
nine cave environments/tilesets in this pack.

The new interaction controller passes across all nine rooms, including fully
printed original resident pages, actual reward quantities/flags and once-only
pickup after Continue (`.local/cave-quality-controller-v01.log`). The companion
traversal test initially assumed straight approaches at both ends of every
stair. R09's middle and lower stairs have deliberate bends; the test now walks
those connected floor approaches in both directions. Both stone-pack controller
tests now pass in `.local/cave-quality-controller-final-turn3-stone.log`, covering
all 68 viewpoints, all 17 stairs, native drops and returns, every cave mouth,
rewards and Continue. Both cases also pass on the newer ice pack in
`.local/cave-quality-controller-final-turn3-with-ice.log`. The exact executable
and both input-pack hashes are recorded in
`.local/cave-quality-controller-final-turn3-receipt.json`. Earlier
reruns encountered concurrent ice-test registration and TM-cache coordinate
type errors in the shared checkout; the missing module path and unsigned
coordinate comparison are corrected. The live run uses the pinned toolchain
with zero optimization for the three large leaf crates to shorten verification.
The preservation tests also required canonicalizing their repository fixture
path before strict pack loading. All three corrected tests pass in
`.local/cave-quality-assets-turn2-v01.log`: rebuilding all nine revisions matches
the written pack's identity/data exactly; every other map and unowned data value,
runtime art file and compiled audio value remains exact; uncached room assembly
agrees with the revised terrain and border. The tests reject unsafe borders,
stale preimages, extra ladders and altered reward contracts.
The stone-cave traversal and interaction checks are complete. The two ice
floors below also pass their complete production-controller course on v02.
The final replay runs both stone-cave tests and the complete ice course against
the same v02 pack; all three pass. Its pack/executable hashes and execution logs
are recorded in `.local/cave-final-controller-turn3-receipt.json`.
Explicit empty-party position fixtures do not earn access to any cave.

## Rust publication

Build the two bounded publishers with the pinned toolchain. Each step reads
the completed prior pack and writes a separate ignored output; R09 certifies
its retained Q06 story programs as well as terrain.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo build --locked --profile wasm-dev \
  -p crystal-assets --bin revise_authored_cave --bin revise_r09_mine
cave_parent=content-packs/minnesota-cohesion-r01-upper-loop-v01.crystalpack
cave_step=0
for cave_room in MnR01Cave1F MnR03Cave1F MnR05RockCut MnR09Cave1F \
  MnR10Cave1F MnR12Cave1F MnR13Cave1F MnR17Cave1F MnR20Cave1F
do
  cave_step=$((cave_step + 1))
  cave_output=$(printf 'content-packs/minnesota-cave-quality-v01-%02d.crystalpack' "$cave_step")
  cave_source=planning/minnesota/caves/johto-quality-v01/$cave_room
  if [ "$cave_room" = MnR09Cave1F ]; then
    target/gym-interiors/wasm-dev/revise_r09_mine "$cave_parent" \
      "$cave_source.previous.json" "$cave_source.json" \
      planning/minnesota/story.json "$cave_output" || exit 1
  else
    target/gym-interiors/wasm-dev/revise_authored_cave "$cave_parent" \
      "$cave_source.previous.json" "$cave_source.json" "$cave_output" || exit 1
  fi
  cave_parent=$cave_output
done
```

```sh
export CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-cave-quality-v01-09.crystalpack"
export RUST_MIN_STACK=67108864
CARGO_TARGET_DIR=target/gym-controller cargo test --locked --profile wasm-dev \
  -p crystal-bevy --lib cave_quality_controller -- --ignored --test-threads=1
CARGO_TARGET_DIR=target/gym-interiors cargo test --locked --profile wasm-dev \
  -p crystal-assets --test cave_quality_revision -- --ignored --test-threads=1
```

The geometry controller traverses every declared camera, ordinary stair and
native drop, tests reverse denial and ordinary returns, saves and continues,
and tests both directions at every retained exterior mouth. The interaction
controller requires fully printed original resident pages and actual item
quantity/flag changes, then checks that Continue cannot duplicate each reward.
The Q06 foreman battle and instrument recovery require their dedicated story
regressions as well.
Direct invocation of the older controller test executable overflowed its
default worker stack while loading this larger pack. The retained effect
regressions then passed with the explicit 64 MiB test worker stack above;
the original failed logs are retained separately. The 11 passing cases cover
Antidote pickup/use/Continue, Repel pickup/use/walking/Continue, gallery reader
pages and Revive persistence, full-bag refusal/Potion use/pickup retry, Otta and
Max Ether use, Ether/Elixir PP effects and refusals, Harper's actual party
healing, the refuge's original Max Ether, and all three Q06 foreman prerequisite,
win/recovery/Continue and loss cases. Execution receipt:
`.local/cave-quality-effects-receipt-v02.json`.

The native reference comparison must include the four source water frames and
their eight horizontal scroll positions, consistently across the entire camera.
The captured production image is never modified to obtain a match.

Scoped `rustfmt --check` passes for the changed cave Rust files. The required
workspace formatting check retains unrelated existing differences
(`.local/cave-quality-format-final-v01.log`). Default all-target Clippy for both
affected crates stops in unchanged `crystal-core/src/multiplayer/mod.rs:1332`:
`clippy::bad_bit_mask` rejects a comparison against an always-zero mask.
This is recorded as a pre-existing failure, not a passing Clippy run.
Additional all-target, crate-local Clippy runs pass for both assets and Bevy
with four existing lints allowed (`never_loop`, `bad_bit_mask`,
`approx_constant`, `useless_conversion`). Their remaining warnings are retained
in `.local/cave-assets-scoped-clippy-turn3-v01.log` and
`.local/cave-quality-scoped-clippy-turn3-v01.log`; neither reports a warning in
the changed cave modules. These scoped runs do not replace the failed default
workspace checks.

The production battle-command/PP regression passes on the cave pack
(`.local/cave-quality-shared-turn3-renderer_neutral_controller_exposes_battle_commands_and_executes_a_turn.log`).
The original Mom/New Bark regression passes on the canonical local
`content-packs/text-tui.crystalpack`
(`.local/cave-quality-canonical-mom-turn3-v01.log`). Its run against the Minnesota
pack stops at the fixture's original-home assertion: this pack intentionally
starts in `MnWinonaHome2F`, rather than `PlayersHouse2F`. The cave preservation
test confirms that this existing Minnesota HOME configuration remains exact.
The MCP movement/production Start-menu regression passes on the cave pack
(`.local/cave-quality-mcp-turn3-v02.log`). Canonical Crystal's spawned stdio
regression passes (`.local/cave-quality-canonical-stdio-turn3-v01.log`). A separate
actual Minnesota stdio session verifies the configured `MnWinonaHome1F` dialogue,
movement rejection while text owns input, A changing the visible dialogue and
byte-for-byte empty stderr
(`.local/cave-quality-mn-home-stdio-turn3-v01.json`). The original stdio fixture's
Minnesota run fails only its hardcoded `PlayersHouse1F` map-name assertion.

## Additional Minnesota ice cavern

The separate ice-region task has published
`content-packs/minnesota-voyageurs-ice-v02.crystalpack`, with two new floors and
683 total maps. Its preservation check confirms that all nine stone caves and
their runtime art/audio remain exact. It adds a paired R20 entrance with two
exterior metatile changes; the existing R20 cave mouths are retained.
All 377 audio PCM hashes, frame counts and loop ranges pass on that pack too.

Both whole-room ice layouts and the first native camera sheets were inspected
against original Ice Path. Their teaching slide and deeper stopper puzzle use
the native ice movement and pale source palette. All nine retained ice cameras
match complete 640×576 source frames with one coherent animation clock per
frame, bringing the combined visual review to 77 views. The independent retained
capture check is `tools/verify_ice_phase_captures.py`, with its report at
`renders/ice-caves/voyageurs/native-phase-review-turn3-v02/phase-proof.json`
relative to `planning/minnesota/`. It checks the original production image and
pack hashes before comparison and never modifies production pixels.

The crystal-teeth camera's initial 513-pixel difference is source glyph `$b5`'s
horizontal scroll: coherent clock4 uses horizontal stage1 and vertical stage0.
The checker accounts for both `$b5` and `$b1` on the native 19-frame schedule.
The complete two-floor production course now passes on v02: all three pickups
change actual quantities/flags once, perpendicular input preserves a forced
slide, both ladders return correctly, an ice-stop save continues stationary,
and the same exterior mouth exits/reenters. Execution:
`.local/ice-controller-final-v06.log`. Stdio MCP also replays the actual exported
checkpoint, verifies the production Start menu and guide input ownership,
exits/reenters, saves/continues and keeps stderr empty
(`.local/ice-mcp-receipt-v01.json`). The final-pack battle-menu/move-PP/dialogue
regression passes (`.local/ice-battle-v02.log`).

The v01 pickup-table defect is corrected by compiling `script_field_pickups`
with the existing native parser. The final publisher requires an entry for each
item and reproduces the v02 pack byte-for-byte. The route planners now apply
native `UP_WALL` entry/departure masks. All nine final-pack ice captures match
zero pixels differently; receipt:
`renders/ice-caves/voyageurs/native-v02/capture-progress.json` relative to
`planning/minnesota/`. See the [ice runbook](../../ice-caves/README.md) for the
final pack, source, build commands and explicit fixture scope.

The current Rust publisher was also executed in memory against the original
stone pack: its native directional-slide graph and post-pickup return checks
pass, and its entire resulting data and identity equal the written v02 pack
(`.local/cave-ice-builder-in-memory-turn3-v01.log`). Final preservation readback
confirms 680 unchanged predecessor maps, all original exterior warps, exactly
two exterior metatile changes, all eleven source room arrays, typed ice rewards,
unchanged native art/audio and unowned data. It also rechecks all 77 original
production image hashes and zero-difference reports
(`.local/cave-final-preservation-turn3-receipt.json`).

The [final eleven-room review](../../renders/caves/eleven-final-v02/all-eleven-final.png)
was rendered directly from the actual v02 pack, using its native palettes,
art and actor positions at one scale. Original source/native renders and rejected
comparisons remain retained alongside it.
