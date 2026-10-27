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

The strict ignored `prairie_story_earned_new_game_care_r02_notebook_and_return`
case starts at New Game, earns its starter and caught partner, and requires
actual wins. Attempts do not establish Varel's victory flag before reaching R02.
An added failure diagnostic confirms a normal loss result1, return to the
Rochester clinic and a restored party; a previous final-enemy-faint trace did
not prove victory. This is not a passing chapter journey. Early-party
balance/retry coverage, an earned
Mankato badge, Cut teaching/cache, R03/Minneapolis links, later regional gates,
courier, rail, League completion and the Kettlurk scene remain unfinished.
