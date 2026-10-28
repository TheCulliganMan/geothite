# Rochester → Prairie Farm Road → Mankato

The chapter candidate is `content-packs/minnesota-northstar-prairie-story.crystalpack`:
**538 maps, 251 species, 251 moves and 377 audio programs**. File SHA-256:
`f7efd72c181143eef61834343c5a2b9fd2664a0d8f14cb250ed3721f8259ed8c`.
Internal content identity:
`f9853ad3269c91f53871f50ce766e4d59b8228bcfdbbeded44243ccad48981be`.

It preserves the [eight-gym regional composition](eight-gyms.md), including
Winona's opening, earned Rochester delivery, Minneapolis's ICE rescue/dispatch,
the northern drainage story and Ryan's Baptism River cabin. The input is frozen
as `content-packs/minnesota-northstar-prairie-base.crystalpack`, file SHA-256
`7c9e9591cf5bb9fa918e9655513c57594852122f10d5b6bad6b689f09e416434`,
content identity `09dbcb239adf58805875b478ed837924f0084aa8ee85a215982231dabd752cae`.
Other tasks may republish the shared eight-gym filename; the chapter deliberately
rejects a different input identity. Packs, saves and renders stay ignored.

## Physical and story contracts

| Seam | Town endpoint | R02 endpoint | Town → route offset |
| --- | --- | --- | ---: |
| Rochester west / R02 east | `(0,18)` and `(0,19)` | `(59,14)` and `(59,15)` | +2 |
| Mankato east / R02 west | `(47,18)` and `(47,19)` | `(0,30)` and `(0,31)` | −6 |

Both directions and both lanes use native connection resolution. The two forest
mouths replace metatile05 with01 at Rochester `(0,9)` and Mankato `(23,9)`.
Rochester's R01 link stays intact. R02's terrain, farm rooms, grass, encounters,
actors and battles remain exact. Mankato's north/R03 mouth remains closed.

Two Rochester wardens stand at `(2,18)` and `(2,19)`. Departure checks only the
Care Badge's native `ENGINE_ZEPHYRBADGE`; there is no extra delivery or Gym1
event requirement. Refusal owns input and returns the player to `(4,y)`.
The guards seal the entire walking/Surf approach. Inbound return is free even
without Care and permits ordinary travel back to the Rochester clinic.

`MN_MANKATO_NOTEBOOK` is a native `SPRITE_PAPER` at `(26,25)`, approached from
`(26,26)` beside the university garden. The gardener mentions it. MS02 requires
Care and shows two pages: “Different lakes. / The same numbers.” then “Who
copied these?” Only acknowledging both pages records
`EVENT_MN_PRAIRIE_NOTEBOOK`, allocated as2305 after the old maximum2304.
All previous numeric constants stay fixed. Repeat text identifies the Mankato
copy; the paper remains present. The interaction grants no item, money, badge
or gym victory. Gym2 entry remains independent of this optional clue.

## Executed publication and checks

The regional builder creates the frozen input from the actual completed
Rochester pack, current pinned gym donor and authored cities:

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_regional_gyms -- \
  content-packs/minnesota-northstar-rochester.crystalpack \
  content-packs/minnesota-gyms-remake.crystalpack planning/minnesota \
  content-packs/minnesota-northstar-prairie-base.crystalpack
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_prairie_story -- \
  content-packs/minnesota-northstar-prairie-base.crystalpack \
  planning/minnesota/playable/prairie-story.json planning/minnesota/story.json \
  content-packs/minnesota-northstar-prairie-story.crystalpack
```

Two asset cases pass exact identity, reconstruction of all three changed maps
with their caches removed, duplicate/stale-contract rejection and preservation
of every unrelated dataset field, runtime file, audio program and recovery record.
Three local production-controller cases pass both denied checkpoint lanes, free
inbound travel, Care-only departure, both road directions, both notebook pages,
input ownership, repeat text, actual gym/clinic/mart doors, Continue and unchanged
party/bag/money/badges. These fixtures explicitly supply position, an empty party
and Care; they do not establish earned regional progression.

The actual stdio MCP client also passes Care departure, both R02 directions,
both notebook pages, dialogue input ownership, repeat and the production Start
menu, with unchanged badge/money and byte-empty stderr. Use the exported
`winona-prairie-local-road-ready-92066-1793085601904769000.crystalsave` in
`.local/prairie-story-review-saves/`. This save is explicit local setup.
All377 PCM hashes, frame counts and loop ranges pass on the final pack.
The two previous Rochester preservation cases also pass, proving the shared
checkpoint's badge-only support preserves the existing starter gate.

```sh
cargo test --locked --profile wasm-dev -p crystal-assets --test prairie_story -- --ignored
export RUST_MIN_STACK=33554432
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-prairie-story.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/prairie-story-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib prairie_story_local_ -- --ignored
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_prairie_story.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_prairie_story_mcp.py \
  --save .local/prairie-story-review-saves/winona-prairie-local-road-ready-92066-1793085601904769000.crystalsave
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-prairie-story.crystalpack --audio --desktop
```

The whole Rochester, R02 and Mankato maps, native Care checkpoint and native
notebook camera were inspected. The full notebook camera matches original source
pixels exactly:0/1,024,000 differences. Every original, crop, conversion and
rejected generation is retained under `renders/prairie-story/`. Existing broad
town streets remain a visible limitation; this chapter does not establish user
visual acceptance or a redesign of the cities.

Scoped Rust formatting passes. `cargo fmt --all -- --check` still fails first
in unrelated `crates/offset-allocator/src/lib.rs:408`. All-target Clippy for
`crystal-assets` and `crystal-bevy` completes with existing warnings under the
four existing allowances `never_loop`, `bad_bit_mask`, `approx_constant` and
`useless_conversion`; no warnings name the new chapter modules. This is not an
unsuppressed workspace lint pass. The plan check passes20 routes,19 hubs,
57 chunks,86 adjacency/gate contracts and162 planned maps; its symbolic
champion reachability does not execute the game.

## Fresh New Game replay on the Ely successor — 2026-10-27

The ignored `prairie_story_earned_new_game_care_r02_notebook_and_return` now
passes on the exact554-map [Ely successor](ely-league.md), file SHA-256
`ab049e907b8712f5ee5f174b474c38fcc52460f272e60336aff17445d62cb6c4`.
The executed case took93.91 seconds after compilation. It starts at production
New Game, chooses the real Cyndaquil, receives the originals/five balls, delivers
the readings, purchases supplies, wins all three R01 battles and both qualifying
gym battles, catches a second partner and earns level11 through ordinary wild
fights. It then defeats Varel, receives Care/Headbutt, heals, walks R02, separately
acknowledges both notebook pages and returns to Rochester. Production Save and
Continue preserve the earned checkpoints. No position, party, level, badges,
quest flags, money, inventory or RNG are supplied to this journey.

Earlier failed attempts exposed actual blackout recovery while the audit kept
following its old route. The route helper now stops at a map transition; the
caller preserves that loss, saves/continues at the clinic, heals and makes a
new departure. Bounded leader retries likewise keep the actual post-loss state.
Attack selection accounts for native physical/special stats, stages, STAB and
type effectiveness; training is performed through the ordinary battle menus.
The passing run includes starter faints and a caught partner finishing fights.
This is one live-RNG course, not a statistical balance assessment or a completed
fresh campaign through Gym2 and the later story.

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export RUST_MIN_STACK=67108864
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-ely-league.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/northstar-fresh-campaign-saves"
cargo test --lib --locked --profile wasm-dev -p crystal-bevy \
  prairie_story_earned_new_game_care_r02_notebook_and_return \
  -- --ignored --test-threads=1 --nocapture
```

Earned saves from that passing run remain in the stated ignored directory:
`winona-rochester-earned-care-badge-42706-1793114882468546000.crystalsave`,
`winona-mankato-earned-notebook-42706-1793114887140141000.crystalsave` and
`winona-prairie-earned-return-42706-1793114889223547000.crystalsave`.
They require this exact pack identity. The earlier local fixture and its MCP
route script above still represent empty-party setup; they are separate proof.

The actual stdio client also loads this earned return save, moves from `(4,18)`
to `(5,18)` through its authoritative movement tool, opens the production Start
menu, closes it and returns to `(4,18)`. Care and money stay exact and stderr is
byte-empty. This replay checks continuation from earned state, not another
complete stdio battle course. Its local log is `.local/northstar-fresh-mcp.log`.

Scoped Rust formatting and default-feature Bevy all-target Clippy pass with the
four established allowances and existing warnings; no warning names the three
changed audit-helper files. Workspace formatting still fails first in
`crates/offset-allocator/src/lib.rs:408`. A separate attempt with
`--features location-tester` fails compilation with E0275, overflowing an
`IntoIterator` requirement for nested Bevy `Res` types. The passing command above
does not enable that feature; no claim is made that the feature failure is fixed.
Only audit helpers and documentation change here. The pack and audio are exact
to the earlier Ely publication; this replay does not republish content.

## Earned Prairie Badge and Minneapolis arrival — 2026-10-27

`northstar_campaign_earned_party_wins_prairie_and_reaches_minneapolis` passes in
80.83 seconds on the same554-map identity. This development invocation loads
the actual Mankato-notebook checkpoint listed above. It teaches the earned
Headbutt TM through Start/Pack, consumes that one TM, wins all three R02 trainer
battles and both Gym2 qualifiers, and earns level14 through a wild fight and
native evolution. The first Torven attempt loses. The course preserves the
actual clinic recovery, resources and gained experience, saves/continues and
retries; Quilava wins at level16. Prairie and one Mud-Slap TM are earned, then
the same party walks the whole R03 connection to Minneapolis and continues there.
No prerequisite, party, money, level, inventory, RNG or position is edited.

The default invocation starts with the Winona New Game course. For the executed
development continuation, set `CRYSTAL_NORTHSTAR_EARNED_SAVE` to the absolute
path of `winona-mankato-earned-notebook-42706-1793114887140141000.crystalsave`.
The resulting checkpoints include
`winona-campaign-MN_GYM_2-loss-0-46847-1793115439476075000.crystalsave`,
`winona-campaign-MN_GYM_2-win-46847-1793115456454330000.crystalsave` and
`winona-campaign-earned-minneapolis-arrival-46847-1793115458721091000.crystalsave`.
All remain ignored under `.local/northstar-fresh-campaign-saves/` and bind this
exact pack. This proves continuation of the recorded fresh game; this particular
invocation does not independently replay its earlier Winona segment.
