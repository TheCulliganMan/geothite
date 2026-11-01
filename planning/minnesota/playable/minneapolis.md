# Minneapolis and the integrated Northstar story

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

The current integrated story candidate is
`content-packs/minnesota-northstar-city-story.crystalpack`: 515 maps, 251 species,
251 moves and 377 audio programs. File SHA-256:
`fce69c6da8ad0d4579ef8aebc6b4922d8d6d2321c87891a572bde7b68150fb7d`.
It brings the reviewed Avrin gym and ICE chapter together with Ryan's House,
expanded Northwoods, guarded Grand Rapids/Duluth links, the drainage reveal and
the chief/receiver finale. The revised mine, limestone hollows, R10 gallery,
R12 ascent, R13 refuge, R20 cleft and shore bends from its exact input are retained.
Later standalone map candidates retain their own separate publication scope.

This is a locally verified composition, not an earned regional journey. The
southern opening, remaining city/route links, St. Paul courier, League finish
and Kettlurk's set piece/encounter remain implementation work.

## Exact inputs and the checked gym transition

The completed northern input is the 510-map
`minnesota-northstar-limestone-review.crystalpack`, SHA-256
`e549c178f37a3137e8ca7f119aeee79460164f1dbd3ea6d205be8928415fd04c`.
Its lineage and native cave proof are in [mine-and-hollows.md](mine-and-hollows.md).

The reviewed gym donor is the 473-map `minnesota-gyms-minneapolis.crystalpack`,
file SHA-256 `4facc79c0078ecc3648794a7dd42e07f8ff72852573545e4cce1839ce10ae206`.
Its native Gym3 content is pinned independently of its three warp destinations.
The original northern room was the earlier 10×9 mill layout with an obsolete
leader/roster. The donor supplies the 5×9 switch room, Avrin, Lenvi and Jossa.

`replace_installed_minneapolis_gym` is an explicit Rust transition before city
installation. It imports only the pinned gym module, its tileset/five art files
and three trainers. Class numbers are checked; each changed trainer class gets
an unused trainer number in the receiving pack. Unrelated trainer numbers,
existing event numbers, HOME/recovery, all other maps/art/audio and quest state
stay exact. Three new puzzle flags are appended without renumbering old events.
Old obsolete puzzle flags remain defined. Cached and genuinely uncached split
assembly must agree; shared script labels, wrong donors, stale originals and
repeat import are rejected. No player save is modified by publication.

The 510-map intermediate is `minnesota-northstar-canonical-gym3.crystalpack`,
SHA-256 `5b4aac2cb02672d10b6ea1c165244a00a29229cbf1c6a09f0e13dbccffbf512d`.
The ordinary city builder then adds its five maps and binds the new gym to the
real city. It still rejects an unsupported gym; it does not silently replace it.

## City and chapter

`towns/MnMinneapolis.json` and its explicit layout describe a 32×24-metatile
river city: west park, bridge, civic street, clinic, paid mart, gym, occupied
dispatch office and the dispatcher's home. All five facades have actual DOOR
quadrants, passive indoor arrivals, paired carpets and safe outdoor returns.
New maps use group 72; the gym retains group 63/map 3. Its reviewed art, scripts,
party, puzzle and rewards remain intact while the city binds its returns and
geographic landmark. A cached-only, unused shop greeting was removed so all
six city/gym modules reconstruct exactly from their actual split payloads.

The ICE chapter has three independent actions:

1. With badge 2, fight the actual enforcer at office `(8,9)`, approached from
   `(9,9)`. Raticate 19 and Growlithe 20 use the real trainer battle engine.
   Only a win sets `EVENT_MN_ICE_ENFORCER_DEFEATED`. ICE remains hostile.
2. Use the latch at `(12,6)`, approached from `(11,6)`. No leaves the captive
   held and permits retry. Yes still requires acknowledging the release text.
   The wall opens and the dispatcher physically walks to the exit, avoiding
   the player, guard and walls. Only then is `EVENT_MN_ICE_CAPTIVE_RESCUED` set.
   The open cell and the dispatcher's absence persist through Continue.
3. Read the separate dispatch copy at `(9,11)`, approached from `(10,11)`.
   Actual victory and rescue are required. Both pages must be acknowledged
   before `EVENT_MN_DISPATCH_NOTE` is set. The record names ICE, Lode and St. Paul.

The dispatcher appears at home after rescue; their partner's dialogue changes.
These interactions grant no badge, item, extra money or later story flag.
Ordinary nurse/PC/paid-shop services remain available independently. A played
loss recovers in the actual clinic without awarding chapter progress.

## Executed verification on the written 515-map pack

| Production-controller scope | Passing cases |
| --- | ---: |
| ICE battle outcomes, separate rescue/note, refusal/retry, home, services | 4 |
| Avrin: both trainers, switches, actual badge/TM, both city carpets and Continue | 1 |
| Grand Rapids handover, service and guarded R09/R10 seams | 6 |
| Duluth checkpoints, earned harbor reading, reciprocal seams and free care | 3 |
| Full harbor dialogue and acknowledgment before the signal | 1 |
| Chief win/loss, separate receiver retry, prerequisites and free care | 4 |
| Ryan's actual cabin door, dialogue/sign, both exits and indoor/outdoor Continue | 1 |
| **Chapter/travel/cabin total** | **20** |

Every gameplay action uses `VisibleShellController::press`. Initial positions,
legal review partners, prior badges and deterministic divider are explicit local
fixture setup. The checks earn each tested battle/action afterward; they do not
earn the entire adventure from a new game.

Four asset cases pass: two gym-import cases and the city preservation/rejection
cases. The latter now remove cached modules before production reconstruction,
checking every typed field after normalizing only raw transport-section storage.
The full original dataset/art/audio is compared after undoing only authorized
changes. Duplicate/stale imports and premature rescue/note completion fail.

All 377 PCM hashes, frame counts and loop ranges pass. Shared Mom/New Bark and
real battle-menu/PP regressions pass, as do normal MCP movement/Start,
rescue yes/no highlight retry, tool catalog and spawned-client byte-empty stderr.
The actual stdio client also plays refusal after victory alone, declined/retried
rescue, opened-cell walking, both record pages/repeat, the walk home, changed
family dialogue and Start; its process exits cleanly with byte-empty stderr.

Whole-city/all-room source review passes. Headless native gym entrance and
leader-court views are inspected alongside the complete source room. The
leader-court visual fixture explicitly seeds switch/trainer flags; earned gym
proof is the controller case above. Earlier office/cell/home native views and
every original, conversion and rejected image remain in `renders/minneapolis/`.
Final user visual approval remains open; no outdoor water-frame pixel equality
is claimed. Plan validation passes 20 routes, 19 hubs, 57 chunks, 86 contracts
and 160 planned maps, which is symbolic reachability. The stale St. Paul paired
carpet reservation was aligned with its current authored exits `(2,19)/(3,19)`;
its native publication remains separate.

Scoped changed Rust leaves are formatted. Workspace formatting still first
fails at `offset-allocator/src/lib.rs:408`; affected-crate all-target Clippy
stops at the existing `crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask error.
The separate full-eight-room authoring fixture currently fails on the unfinished
St. Paul room's open outer edge `(13,15)`. The verified import uses the captured
single-gym donor and retains the other seven installed rooms. No full-eight-room
current authoring or end-to-end lint success is claimed.

## Executed publication and replay

All packs and retained review saves are local and ignored. Republishing the
city from the current sources reproduces the final file byte-for-byte.

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export RUST_MIN_STACK=33554432
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin replace_minneapolis_gym --bin pack_minneapolis --bin verify_crystalpack
target/northstar-story/wasm-dev/replace_minneapolis_gym \
  content-packs/minnesota-northstar-limestone-review.crystalpack \
  content-packs/minnesota-gyms-minneapolis.crystalpack \
  content-packs/minnesota-northstar-canonical-gym3.crystalpack
target/northstar-story/wasm-dev/pack_minneapolis \
  content-packs/minnesota-northstar-canonical-gym3.crystalpack \
  planning/minnesota/towns/MnMinneapolis.json \
  planning/minnesota/towns/MnMinneapolis.layout.json \
  planning/minnesota/story.json \
  content-packs/minnesota-northstar-city-story.crystalpack
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-city-story.crystalpack --audio --desktop
cargo test --locked --profile wasm-dev -p crystal-assets \
  --test minneapolis_gym_revision -- --ignored
export CRYSTAL_MINNEAPOLIS_PREDECESSOR_PACK="$PWD/content-packs/minnesota-northstar-canonical-gym3.crystalpack"
# These are the two city cases; the independent all-eight-room fixture is noted above.
cargo test --locked --profile wasm-dev -p crystal-assets --test minneapolis \
  minneapolis_adds_only_authored -- --ignored
cargo test --locked --profile wasm-dev -p crystal-assets --test minneapolis \
  minneapolis_rejects_skipped -- --ignored
export CRYSTAL_MINNEAPOLIS_TEST_PACK="$PWD/content-packs/minnesota-northstar-city-story.crystalpack"
export CRYSTAL_MINNEAPOLIS_SAVE_DIR="$PWD/.local/minneapolis-integrated-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  'bevy_shell::tests::minneapolis::' -- --ignored
cargo build --locked --profile wasm-dev -p geothite
```

A retained save starts after an earned local enforcer win, before rescue/reading:

```sh
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-city-story.crystalpack \
  --load .local/minneapolis-integrated-review-saves/ms03-enforcer-win-54155-1793072874402696000.crystalsave
python3 planning/minnesota/tools/check_minneapolis_mcp.py \
  --pack content-packs/minnesota-northstar-city-story.crystalpack \
  --save .local/minneapolis-integrated-review-saves/ms03-enforcer-win-54155-1793072874402696000.crystalsave
```

Use `mcp` instead of `play` for the same session over stdio. For the targeted
text regression, set `CRYSTAL_RENDER_TEST_PACK` and
`CRYSTAL_MINNEAPOLIS_ENFORCER_WIN_SAVE` to those absolute pack/save paths and run
`cargo test --locked --profile wasm-dev -p geothite --bin geothite mcp_ -- --include-ignored`.

The verified merged-pack cabin approach is
`.local/minneapolis-integrated-north-saves/ms10-ryans-house-porch-54327-1793073003729602000.crystalsave`.
Use it with the same pack to start beside the falls. The cabin's original source
and proof are in [ryans-house.md](ryans-house.md). Older saves inside the former
Gym3 need separate position/terrain-override migration review after its resize;
use the matching retained local fixtures above for these checks.

## Remaining progression and retained predecessors

R03/R04/R05 port reservations remain unbound, with a closed walking/Surf
perimeter. Earned arrival from Mankato, departure to St. Paul/St. Cloud, the
Strength gift and north checkpoint still need native work. A production new
game still starts in the original opening; local controller fixtures do not
establish the authored Winona opening or a continuous Minnesota playthrough.

The retained historical 515-map `minnesota-northstar-minneapolis-story.crystalpack`
has file SHA `7f5f64100c5f375689e1aa8fb93355c6c5b970b5b26f4c08b185fd290a593685`.
It used the earlier gym and pre-refinement northern maps. The earlier separate
478-map `minnesota-minneapolis-canonical-story.crystalpack`, SHA
`875934a0fd105ab211f902ced687df539b11b04659f9fcafeef4acaa146ff6e6`,
verified the new gym/city independently. Both earlier packs, saves and all their
render generations remain retained; their old checks keep their original scope.

## Earned campaign and native damaging stat effects — 2026-10-27

The later554-map [Ely successor](ely-league.md) includes the connected Winona,
Care, R02, Mankato, Prairie and R03 course. Its file SHA-256 remains
`ab049e907b8712f5ee5f174b474c38fcc52460f272e60336aff17445d62cb6c4`.
The new `northstar_campaign` and `northstar_campaign_ice` controller modules
continue actual exported campaign state, perform all movement/battles through
production input and keep native losses. Their default course starts at Winona
New Game; optional development saves are explicitly identified as continuations.
See the [earned Prairie evidence](prairie-story.md#earned-prairie-badge-and-minneapolis-arrival--2026-10-27).

The earned Mud-Slap TM exposed a runtime bug in that unchanged pack. Its move
record has `ACCURACY_DOWN_HIT` and100% effect chance, but omits the optional
`stat`/`amount` annotations. The secondary-effect path previously required those
annotations and silently skipped the accuracy drop. The Rust engine now resolves
the parameters from the native damaging effect, using the existing direct-stat
resolver. Explicit annotations, including zero/partial pairs, retain precedence.
Chance, stage caps, targets, Mist/Substitute and faint handling remain in their
existing effect paths. No pack record, save identity or audio program changes.

Eight stat-stage regressions pass, including the actual damaging turn with both
annotated and unannotated move records. The real external-pack controller course
also observes Mud-Slap PP consumption and accuracy drops, earns the enforcer
victory, separately releases the captive, acknowledges both dispatch pages and
returns the dispatcher home. That 158.19-second invocation continues the earlier
fresh game's actual post-loss/training save, with an earned level22 partner;
it wins at level23. It does not supply a party, badge, quest result, money, items,
position or RNG and is not independently a complete New Game invocation.

The rebuilt production text client passes the existing actual stdio rescue
checker from its earned victory save:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_minneapolis_mcp.py \
  --pack content-packs/minnesota-northstar-ely-league.crystalpack \
  --save .local/northstar-fresh-campaign-final-saves/ms03-enforcer-win-56994-1793117667722753000.crystalsave
```

This replays release refusal/retry, the opened cell, ICE/Lode record, return home
and Start, preserves the two badges/money and requires byte-empty stderr.
Full campaign balance and later earned chapters remain open. A passing local
continuation is not proof of all starters or all live-RNG outcomes.

Default-feature Bevy all-target Clippy and core-library Clippy pass under the
four established allowances with existing warnings. Core all-target checks are
blocked by missing external radio-oracle JSON fixtures and the existing
`save_sharing` constructor mismatch; those are unrelated to the effect resolver.
Changed core functions and the new leaf modules match rustfmt. Workspace
formatting still fails first at `crates/offset-allocator/src/lib.rs:408`; unrelated
formatting is preserved. The `location-tester` compilation overflow reported in
the Prairie runbook remains outside these default-feature checks.

## New Game through two badges and ICE — 2026-10-27

The complete default controller case now passes from a brand-new Winona game
in 341.44 seconds, with neither checkpoint override set. It earns the guardian
scene, Cyndaquil, originals and five balls; walks R01; acknowledges delivery;
defeats all three route trainers and both Care qualifiers; catches Rattata;
trains through ordinary battles/free care; and wins Care and Headbutt. It teaches
Headbutt through Pack, reads both Mankato notebook pages, wins all three R02
trainers and both Prairie qualifiers, evolves to Quilava, then wins Prairie and
Mud-Slap. It walks R03 into Minneapolis, teaches Mud-Slap while retaining
Headbutt, wins all four R03 trainers and performs real training before ICE.

Two actual wild blackouts preserve the earned events and resources through
clinic recovery and Continue. Care, Torven and the enforcer are each defeated
on the first leader/enforcer attempt in this passing run. Quilava enters the ICE
battle at 18 and wins at 19. Mud-Slap spends native PP and visibly lowers accuracy.
The separate rescue checks refusal/retry, acknowledgment before release, the
opened cell and absence of the captive, both dispatch pages/input ownership,
repeat/Continue and the dispatcher at home. No next badge or later plot result
is supplied. There are no position, party, level, badge, item, money, quest or RNG
fixture writes in the default course. Earlier failed live-RNG attempts exposed
weak audit policies; this one success is not an all-seed balance claim.

```sh
env -u CRYSTAL_NORTHSTAR_ICE_SAVE -u CRYSTAL_NORTHSTAR_EARNED_SAVE \
  RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/northstar-story \
  CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
  CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-ely-league.crystalpack" \
  CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-campaign-earned-final-saves" \
  CRYSTAL_MINNEAPOLIS_SAVE_DIR="$PWD/.local/northstar-campaign-earned-final-saves" \
  cargo test --lib --locked --profile wasm-dev -p crystal-bevy \
    northstar_campaign_earned_party_defeats_ice_and_releases_dispatcher \
    -- --ignored --test-threads=1 --nocapture
```

The run is recorded in `.local/northstar-campaign-earned-final.log`. Its earned
victory checkpoint is
`.local/northstar-campaign-earned-final-saves/ms03-enforcer-win-64348-1793118972385463000.crystalsave`;
the final home checkpoint is
`winona-campaign-earned-ice-dispatcher-home-64348-1793118978678766000.crystalsave`
in that same ignored directory. These bind the exact 554-map identity above.
The rebuilt text client also passes the existing stdio checker from this exact
fresh victory checkpoint: release refusal/retry, opened cell, record, home,
Start, stable badges/money and byte-empty stderr. Its log is
`.local/northstar-fresh-earned-ice-mcp-final.log`. Current/Capitol and the
later earned regional journey remain unverified by this course. Kettlurk's
visible set piece remains unfinished.
