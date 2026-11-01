# Minnesota city residents and trades

The authoritative [population contract](../city-life.json) adds eight distinct
residents to each small town and sixteen to each major city. Each cast contains
one native Pokémon trader and two or four walking residents. There are 144 new
characters, 36 walkers and 15 traders across the 15 implemented towns. Every
character has local dialogue; the existing residents and activities remain.

Four entries in `world.json` (Detroit Lakes, Moorhead, International Falls and
Mille Lacs) do not yet have installed city maps in these parents. This population
work does not invent terrain or claim those planned towns are playable.

## Publications

These ignored outputs preserve their respective completed parent lineages:

| Output in `content-packs/` | Parent | Coverage |
| --- | --- | --- |
| `minnesota-city-life-v3.crystalpack` | `minnesota-minneapolis-compact-falls-v02.crystalpack` | 660 maps, 14 towns, 136 new residents; compact Minneapolis/falls preserved |
| `minnesota-city-life-ely-v3.crystalpack` | `minnesota-northstar-ely-league.crystalpack` | 554 maps, 15 towns, 144 new residents; Ely/League preserved |
| `minnesota-city-life-fair-v3.crystalpack` | `minnesota-state-fair-battles-v1.crystalpack` | 664 maps, 14 towns, 136 new residents; 474 species and all existing fair actors/battles preserved |

File SHA-256 receipts:

- Compact: `ded1790259b8f4b51d7a3398877706c400bb9c7599d33b57f58392f9835da10c`.
- Ely: `b8b961bd68d51ca2be1a93c8b5ab8bf071e3387a0a37ff1a58f4409587d52c7f`.
- Fair: `7cc6bafffcf3bec06c0ef23fa7880157408c2db07b0033499d42a7663c8a8368`.

The compact, League and fair parents contain different accepted map revisions.
The population overlay supports their explicit positions without merging or
replacing those unrelated revisions. Use a save made for the chosen pack's
identity. Old renders and first-pass packs remain review history.

## Local trades

The first resident of every cast is the city's sole trader. Native YES/NO,
party selection, cancellation, wrong-species feedback, cable acknowledgement,
exchange and repeat dialogue run through `VisibleShellController::press`.
Completion is stored in the existing `completed_trades` save memory. The
received Pokémon keeps the requested partner's level and receives its own
nickname, OT, native trade DVs and a Berry. No story badge or quest flag is granted.

| City | Trainer | Requested | Offered | Nickname |
| --- | --- | --- | --- | --- |
| Winona | Mara | Hoothoot | Natu | BLUFF |
| Rochester | Iris | Bellsprout | Magnemite | PULSE |
| Mankato | Otto | Wooper | Machop | TERRACE |
| Minneapolis | Tess | Abra | Machop | MILL |
| St Paul | Remy | Drowzee | Growlithe | SUMMIT |
| St Cloud | Sue | Pidgeotto | Drowzee | DREAM |
| Brainerd | Len | Geodude | Slowpoke | BOBBER |
| Leech Lake | Jo | Wooper | Poliwag | RIPPLE |
| Bemidji | Hal | Oddish | Pineco | CONE |
| Grand Rapids | Ed | Geodude | Teddiursa | CEDAR |
| Duluth | Gus | Machop | Seel | HARBOR |
| Grand Marais | Nell | Tangela | Chinchou | LANTERN |
| Boundary Waters | Kit | Marill | Wooper | PORTAGE |
| Lake Itasca | Ada | Sentret | Hoppip | SPRING |
| Ely | Bea | Onix | Skarmory | AURORA |

All requested species have actual Minnesota grass encounter slots in these
parents. This does not assume an unimplemented fishing-rod gift or require
leaving the region for an otherwise absent species.

## Gameplay contracts

Positions and three-tile walking lanes are explicitly authored. Native movement
radius 2 excludes its boundary and permits only the center and one tile either
side. All lane tiles must be Floor, distinct from other residents, and clear of
warps, coordinate events and arrivals. Treating every lane as occupied at once
must preserve the original reachable walking area. Every new actor must remain
approachable from the inhabited city component; closed edge checkpoints are not
mistaken for that entire component.

The engine retains byte-sized object identities with slot 0 reserved for the
player and up to 255 NPCs. The old twelve-slot truncation also no longer affects
maps with 13–15 records. Renderer-neutral idle frames now advance the same
walking/wandering/spinning scheduler as the graphical game, while modal input
owners suspend it. Idle overworld frames also poll button releases when every
nearby NPC is standing, so a cancelled trade can immediately be offered again
with A. Script continuations stop before an uncompleted trade so
a preceding `waitbutton` cannot bypass its confirmation/party UI.

The overlay compiles only new programs and appends their typed metadata. It
preserves existing command indices, objects, events, terrain, connections,
clinics, marts, title/HOME, encounter tables, species, items, source files and
audio. Duplicate publication, missing city coverage, invalid species/art,
unsupported extents, blocked placements and lost access fail the Rust build.

## Build and review

Executed publication commands use the pinned repository toolchain:

```sh
export CARGO_TARGET_DIR=target/city-life
cargo build --locked -p crystal-assets --bin pack_minnesota_city_life \
  --config 'profile.dev.package.crystal-assets.opt-level=0'
target/city-life/debug/pack_minnesota_city_life \
  content-packs/minnesota-minneapolis-compact-falls-v02.crystalpack \
  planning/minnesota/city-life.json content-packs/minnesota-city-life-v3.crystalpack
target/city-life/debug/pack_minnesota_city_life \
  content-packs/minnesota-northstar-ely-league.crystalpack \
  planning/minnesota/city-life.json content-packs/minnesota-city-life-ely-v3.crystalpack
target/city-life/debug/pack_minnesota_city_life \
  content-packs/minnesota-state-fair-battles-v1.crystalpack \
  planning/minnesota/city-life.json content-packs/minnesota-city-life-fair-v3.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_city_life.py \
  --pack content-packs/minnesota-city-life-v3.crystalpack \
  --output planning/minnesota/renders/city-life/v03-final
```

The read-only review tool renders every complete installed town and trader/walker
player views from the pack's own source art. `render_archive` retains originals
and previous versions. The source review caught facade placements and entrance
crowding; connectivity checks additionally caught blocked St Cloud/Brainerd and
legacy Duluth lanes, which were relocated before publication.

Native captures use the production `render_at_location` example with
`CRYSTAL_RENDER_TEST_HEADLESS=1`, `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`,
`CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576`, view 2d and hour 12. Trader camera fixtures
use the actor's x and y+1, facing down. These are local visual/party fixtures,
not an earned campaign traversal.

## Run the completed local game

The production client was built with the same pinned toolchain and launched as
an actual stdio MCP process against `minnesota-city-life-v3.crystalpack`.
`observe` returned `MnWinonaHome2F` and `press start` opened the production menu;
stderr remained empty. Its dump also loaded this exact pack, as a separate
render smoke check. The MCP movement/Start and spawned Mom/stderr regressions
pass against the completed original TUI pack.

```sh
CARGO_TARGET_DIR=target/city-life cargo build --locked --profile wasm-dev -p geothite \
  --config 'profile.wasm-dev.opt-level=0'
target/city-life/wasm-dev/geothite play content-packs/minnesota-city-life-v3.crystalpack
# Or run the same controller through stdio MCP:
target/city-life/wasm-dev/geothite mcp content-packs/minnesota-city-life-v3.crystalpack --name CHRIS
```

## Verification

Verified on 2026-10-27:

- All three published outputs pass strict read-back and deterministic rebuild
  identity checks. Three preservation cases retain original maps/programs,
  assets, title/spawns, encounters, items, trainers and audio; they also reject
  duplicate publication, omitted cities and blocked placements. All requested
  species have a real regional grass encounter slot.
- Six production controller cases pass: local dialogue/input ownership and
  walking, plus the complete trade course, on each of the three packs. Across
  their 43 installed town instances, the course declines the offer, cancels the
  party selector, rejects a wrong species, exchanges only after the cable
  acknowledgement, checks received species/nickname/OT/Berry and party removal,
  saves/Continues, then verifies a second exchange is denied. Retry after decline
  uses A immediately, without a direction input. Closing A is released by idle
  overworld frames. Explicit initial city/party/divider fixtures are setup.
- The runtime composed-run regression pauses before a trade even when stale
  party-selection input is supplied. The 13–15 resident regression and both
  expanded-capacity cases pass, including 20 loaded NPCs with interaction and
  save/restore, plus the final byte-sized slot 255 without overflow.
- The original Mom/New Bark input-ownership/gate course and retained native
  battle-menu/selected-move PP regression pass with the completed TUI pack.
- All 377 PCM hashes, frame counts and loop ranges match on each of the three
  final packs. No audio programs or catalogs were copied into source.
- Fifteen native trader-camera images were captured headlessly and inspected
  against the same noon/down-facing source fixtures. Full source maps and
  ordinary walking views are retained under ignored
  `planning/minnesota/renders/city-life/v04-source`, `v04-ely-source` and
  `v04-native`; earlier originals/rejected passes remain. Native captures retain
  the source's proportions, four-color character art, readable bodies and clear
  entrances. Existing terrain was not redesigned by this overlay.

The standalone newly added Rust files pass `rustfmt --check`. The required whole
workspace format check remains blocked by unrelated existing formatting,
including `offset-allocator` and other working edits. Ordinary all-target Clippy
is blocked by the existing `bad_bit_mask` error in core multiplayer, existing
`never_loop` errors in asset mutation/parsing code, and missing radio-oracle
integration fixtures. A libraries-only check with exactly those two existing
lint denials allowed completes; it reports no warnings in the new population
builder, publisher or capacity module. Existing warnings elsewhere remain.

```sh
export RUST_MIN_STACK=67108864
export CARGO_TARGET_DIR=target/city-life
export CRYSTAL_CITY_LIFE_BASE="$PWD/content-packs/minnesota-minneapolis-compact-falls-v02.crystalpack"
export CRYSTAL_CITY_LIFE_PACK="$PWD/content-packs/minnesota-city-life-v3.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test minnesota_city_life --config 'profile.wasm-dev.opt-level=0' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-bevy --lib city_life \
  --config 'profile.wasm-dev.opt-level=0' -- --ignored --test-threads=1
cargo test --locked --profile wasm-dev -p crystal-runtime --lib --features test-fixtures \
  runtime_composed_script_leaves_uncompleted_trades_for_controller_choices \
  --config 'profile.wasm-dev.opt-level=0'
cargo test --locked --profile wasm-dev -p crystal-core --lib --features test-fixtures loaded_map_object_roster \
  --config 'profile.wasm-dev.opt-level=0'
cargo test --locked --profile wasm-dev -p crystal-core --lib --features test-fixtures expanded_map_ \
  --config 'profile.wasm-dev.opt-level=0'
```
