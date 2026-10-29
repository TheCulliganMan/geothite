# Winona: the Northstar opening

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

The [526-map Rochester successor](rochester.md) now binds R01 and verifies
earned delivery and return. This runbook retains the original opening-only
521-map publication scope.

The written opening candidate is
`content-packs/minnesota-northstar-winona.crystalpack`: 521 maps, 251 species,
251 moves and 377 audio programs. SHA-256:
`2f8c74a72a51affc94fc416763505fca5e8842bbb52b17f5662ee212e40eb2da`.
Its exact input is the 515-map [combined story candidate](minneapolis.md),
SHA-256 `fce69c6da8ad0d4579ef8aebc6b4922d8d6d2321c87891a572bde7b68150fb7d`.
Every original map, including Avrin's gym, ICE's chapter, the northern story
and Ryan's Baptism River cabin, remains exact. Later independent cave/city
branches have not been substituted into this lineage.

## What executes

Production New Game enters `MnWinonaHome2F` at `(3,3)` with no partner or
story progress. Walk to `(7,1)` and up the actual stairs. The guardian's three
pages own input; completion records `EVENT_MN_HOME_INTRO`. The home carpet
returns to town `(7,8)`. Enter the field station from town `(18,20)`.

Three native tabletop balls offer Chikorita, Cyndaquil or Totodile at level 5.
Their approaches are `(6,4)`, `(7,4)` and `(8,4)`, facing up. Declining leaves
all choices available. Confirming executes the actual `givepoke`, then waits
for the native nickname question and, when selected, its naming grid. Only
after naming finishes do the starter and selected-ball events advance.
The other two balls remain present and offer follow-up, never a second gift.

Two acknowledged survey pages then record
`EVENT_MN_R01_PARCEL_RECEIVED`. This is the canonical survey-bundle event,
not an additional bag item. Five Poké Balls use a separate once-only event;
a full pocket leaves that event unset and the supply interaction retryable.
Tamarack and the remaining balls recover unfinished handover/supplies. The
guardian provides repeat rest. No badge or downstream quest is supplied.

The 16×14-metatile town has four fitted facades and five interiors:
home1F/home2F, clinic, mart and field station. Authoritative coordinates are in
`towns/MnWinona.json`, its explicit layout and `interiors.json`. The original
blueprint is retained under `revisions/winona-before/`. Native identity is
group 73, maps 1–6.

The title still writes `SPAWN_HOME` 0. That destination now names Winona's
upstairs floor, and the title's previous landmark names Winona. The original
PlayersHouse2F recovery destination survives as addon identifier 262; all
other original recovery records are exact. This changes HOME semantics for
this variant; it is not a migration guarantee for old saves.

## Evidence and limits

- Three production New Game journeys pass without position, party, inventory,
  badge, event, money or RNG seeding: guardian pages/input ownership,
  decline/retry, real starter/moves, naming, acknowledged parcel, five balls
  once, repeat interactions and Continue at multiple stages. Cyndaquil also
  receives a nickname through the actual grid and Start/END.
- A fourth regression explicitly seeds twelve full native ball stacks after
  the earned decline. Refusal preserves the whole bag and reward event;
  freeing five spaces in the fixture permits an actual interaction retry,
  once only, across Continue. This is not an earned purchase trip.
- Two asset cases pass: all 515 original maps and unrelated data/art/audio
  are exact, new maps reconstruct uncached, recovery/title changes are bounded,
  and duplicate, wrong-starter and premature-parcel contracts are rejected.
- Actual stdio MCP passes New Game, guardian, declined/retried starter,
  nickname refusal, parcel/balls, repeats, return and Start, with byte-empty
  stderr. The runtime composed-gift boundary regression passes too.
- Eleven shared controller cases pass on the original TUI pack, including
  Mom/New Bark and an actual battle command/PP mutation. Those literal New
  Bark expectations are not claimed for this Winona-start variant.
- All 377 PCM hashes, frame counts and loop ranges pass on the written pack.
  Whole town, all interiors and headless noon/down-facing town, bedroom and
  laboratory captures were inspected. Every generation remains under
  `renders/winona/`, including rejected facades and reference stages.

R01 remains unbound: this is an earned local opening, not a Winona-to-
Rochester journey. Rochester delivery/gym1, remaining regional travel, courier,
League finish and the Kettlurk set piece remain work. Clinic healing/PC and
paid shopping are published with native commands and source collision checks;
service-specific health/PP/money regressions are not claimed here. The bedroom
PC is scenery at present. Final user visual approval remains open.

Workspace `cargo fmt --all -- --check` retains unrelated failures beginning in
`offset-allocator/src/lib.rs:408`. All-target Clippy for the three affected
crates stops at existing `crystal-core/src/multiplayer/mod.rs:1332`
(`clippy::bad_bit_mask`). No broad green formatting/Clippy claim is made.
With the four previously documented `never_loop`, `bad_bit_mask`,
`approx_constant` and `useless_conversion` allowances, the same three-crate
all-target Clippy check completes with existing warnings; none names the new
Winona leaves. Shared MCP movement/Start and spawned-client clean-stderr tests
also pass on the original TUI pack.

## Executed build and checks

Use the pinned toolchain. Packs, saves, renders and build outputs stay ignored.

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export RUST_MIN_STACK=33554432
cargo build --locked --profile wasm-dev -p crystal-assets -p geothite \
  --bin pack_winona --bin geothite
target/northstar-story/wasm-dev/pack_winona \
  content-packs/minnesota-northstar-city-story.crystalpack \
  planning/minnesota/towns/MnWinona.json \
  planning/minnesota/towns/MnWinona.layout.json \
  planning/minnesota/story.json \
  content-packs/minnesota-northstar-winona.crystalpack
cargo test --locked --profile wasm-dev -p crystal-assets --test winona -- --ignored
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-winona.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/winona-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  'bevy_shell::tests::winona::' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-runtime --lib \
  runtime_composed_script_leaves_gifts_for_frontend_presentation
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_winona_mcp.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_winona.py \
  --pack content-packs/minnesota-northstar-winona.crystalpack
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-winona.crystalpack --audio --desktop
```

Start the opening with the production terminal or stdio controller:

```sh
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-winona.crystalpack
target/northstar-story/wasm-dev/geothite mcp \
  content-packs/minnesota-northstar-winona.crystalpack
```

The interactive command is provided for local play. Verification used the
actual stdio process and headless captures without opening a window.
