# Historical reward review - 2026-10-28

This retained review describes the older 705-map, 224-species candidate. It does not describe the selected world or final comprehensive campaign. Use [the current status](README.md) for scope and current evidence.

# Comprehensive Minnesota game

The verified reward successor is
`content-packs/minnesota-progression-rewards-v04.crystalpack` (705 maps), recorded
as `progression_rewards_revision` in `world.json`. The comprehensive-game goal
remains active. [requirements.json](requirements.json) retains the full scope and
open work; individual reward fixtures do not prove a complete campaign.

## Earned rewards

- Minneapolis's tool workshop gives a free rebuilt **Bicycle** after the first
  badge. The existing storefront sign names the bicycle workshop. Instructions
  explain PACK / KEY ITEMS, registration and SELECT riding.
- Tamarack gives one **Master Ball** after acknowledged Boundary Waters
  restoration. Failed delivery retains the claim; repeat visits cannot duplicate it.
- The Winona aide rewards **caught active Minnesota species**: 10 Great Balls at
  10, Itemfinder at 25, Lucky Egg at 100, and 10 Rare Candies for
  all 224. Each tier is claimed separately, in order. Seen-only, retired and
  foreign species do not count. Saved variable mirrors cannot spoof the count.
- Grand Marais's station recovers a missing **Machine Part** after the original
  welcome, before lighthouse repair. Recovery gives neither another starter nor
  a replacement part after repair.

The Rust overlay wraps existing actor programs. It preserves terrain, actors,
warps, map events, furnishings, teams, encounters and original conversations.

## Verified publication — 2026-10-28

Parent: `content-packs/minnesota-current-grass-r11-v01.crystalpack`.

- File SHA256: `27a71e6259fab624324a990957a059b009552ddfa39720fa9074db813aeadb42`.
- Content identity: `96402ad8fb055ece38d119e21db4ef42ee44b16c57c886885fb283058f848a0b`.
- Four script/sign maps change; **701 parent maps are exact**. All native art,
  geometry, actors, warps, events, species, moves, teams, encounters, fishing,
  items, marts, evolutions and trades are preserved.
- **Five native controller reward cases pass** on this identity, in 56.94 seconds.
  These include actual bicycle registration and outdoor SELECT riding, all five
  research tiers, full-ball-pocket refusal followed by native TOSS/retry,
  Master Ball persistence and Machine Part recovery/repair-complete checks.
- **Fresh earned opening passes**, in 39.29 seconds: Reed, Ivy, actual egg
  refusal/retry and Continue, without a supplied party.
- **Three production stdio replays pass**: regional Dex denial despite foreign
  catches, actual Itemfinder inventory/repeat checks, and actual Master Ball
  inventory/repeat checks. Every process exits with byte-empty stderr.
- All **377 audio PCM hashes, frame counts and loop ranges pass** the actual
  verifier. Scoped Rust formatting and affected-crate all-target Clippy pass.
  Workspace formatting differences and baseline Clippy warnings remain.

The reward tests explicitly declare prior badge/restoration/Dex records. After
setup they use production controller input and Save/Continue. The capacity case
fills all 12 native ball slots and frees room through the real TOSS menu. These
fixtures do not prove an earned full campaign or complete Dex.

Source text rows fit 18 columns. A production-input intro save and native 2D
capture remain under ignored `renders/progression-rewards-v01`; the v03 typing
frame is predecessor evidence, not completed-page visual acceptance.

## Build and run

These Rust build and publication paths were executed. Outputs remain ignored.
Use a fresh output name when preserving a frozen review pack.

```sh
CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
  RUST_MIN_STACK=67108864 cargo build --locked --profile wasm-dev \
  -p crystal-assets --bin pack_northstar_progression_rewards \
  -p geothite --bin geothite
RUST_MIN_STACK=67108864 \
  target/northstar-story/wasm-dev/pack_northstar_progression_rewards \
  content-packs/minnesota-current-grass-r11-v01.crystalpack \
  planning/minnesota content-packs/minnesota-progression-rewards-v04.crystalpack
./target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-progression-rewards-v04.crystalpack
```

Use this newly built client for `VAR_MINNESOTA_DEX_CAUGHT`; historical binaries
may lack the query. For native verification use a **fresh** ignored save directory.
Frozen v01-v03 saves remain bound to their exact original packs.

```sh
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-progression-rewards-v04.crystalpack" \
  RUST_MIN_STACK=67108864 \
  target/northstar-story/wasm-dev/deps/crystal_bevy-0739f9b63aa7a27b \
  progression_rewards_ --ignored --nocapture --test-threads=1
```

That executable is the verified local test build; Cargo may give a rebuilt test
binary a different hash. `cargo test --locked --profile wasm-dev -p crystal-bevy
--lib progression_rewards_ -- --ignored --nocapture --test-threads=1`, with the
same target/profile/pack environment above, builds and runs the matching tests.
The three stdio cases use `tools/check_progression_rewards_mcp.py` with the
reached `reward-dex-before-9`, `reward-dex-before-25` and
`reward-master-ball-before` checkpoints retained by
`CRYSTAL_MAP_RETAIN_SAVES_DIR` during the native course.

## Remaining full-game work

The read-only `tools/audit_species_acquisition.py` follows every evolution method
as a generous upper bound. The accepted pack has 224 active species in 101
families, 67 potential direct species, 104 potential species after all evolutions,
and **54 families without a potential acquisition source** in the inspected data.
Reachability, conditional starter choices, evolution items and actual capture
can further reduce that pool. The all-roster award exists; earned full Dex
completion is not yet accepted.

HM/fishing-house integration, complete key-item use and an uninterrupted campaign
remain open in the requirements ledger. Current written story and earlier
chapter evidence are retained in `../current-story-v01/README.md`.
