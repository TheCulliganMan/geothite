# Original Minnesota Pokémon

The Rust `pack_minnesota_pokemon` publisher extends a supplied, verified local
`.crystalpack`. It reads the canonical roster, complete authored profiles and
reviewed native front/back sprites. It does not export source content or change
the source pack. Generated packs remain ignored and must never be committed.

The source catalog retains 313 original proposals; curation selects 220 active
species across 100 families. The complete active pack contains 471 species,
including the original 251. Runtime IDs are uppercase authored names. Stable
source IDs retain their global numbers even when a family is retired; the web
regional index is separately ordered and contiguous from 1 through 220.

Current authoring includes the [Fairy ruleset](../../docs/northwoods-fairy.md).
Apply its type overlay before compiling regional species, then the Northwoods
and Fairy move overlays. The detailed runbook provides the exact local commands.
Older 266-species examples below are historical; they must not regenerate or
overwrite the current curated profiles. Family snapshots include the original
game and a whole evolutionary family. All outputs remain ignored.

## Authoring contract

A profile file contains `{"schema_version":1,"species":[...]}`. A directory of
such JSON batches is also accepted. Every canonical species must have exactly
one profile; unknown and duplicate profiles fail loading. Names, types and
evolution targets come from the canonical roster rather than duplicate fields.

Each profile requires these fields:

- `id`: canonical regional identifier, such as `MN_001`.
- `classification`: 1–11 printable ASCII characters.
- `height_inches`: positive integer; compiled into feet/inches digits.
- `weight_tenths_lb`: positive integer, stored consistently in species and dex data.
- `dex_pages`: exactly two strings. Each has three nonempty lines of at most 18
  ASCII characters, separated by ` @ `. This fits the production 18×3 dex box.
- `base_stats`: `hp`, `attack`, `defense`, `speed`, `special_attack`,
  `special_defense`, each 1–255.
- `catch_rate`, `base_exp`, `gender_ratio`, `hatch_cycles`, `growth_rate`.
- `egg_groups`: two existing exact egg-group tokens.
- `level_moves`: sorted `[level,"MOVE_ID"]` pairs, levels 1–100, including a
  damaging level 1 move. Every stage has its own complete level-up list.
- `tmhm_moves` and `egg_moves`: explicit arrays, including empty arrays.
- `evolution`: `null` for final/single stages, otherwise an object with `method`
  and the appropriate `level`, `item`, `held_item`, `happiness`, `stat_ratio`
  fields. Supported methods are `LEVEL`, `ITEM`, `HAPPINESS`, `TRADE`, `STAT`.
  The target is inferred from the canonical family.
- `held_items`: two existing item IDs or nulls.
- `menu_icon`: an explicit existing base icon token.
- `cry`: explicit existing base cry ID plus signed `pitch` and `length`.
- `footprint_species`: explicit existing base species whose footprint is reused.
- `shiny_rgb5`: two distinct RGB5 color triplets, excluding white and black.

Move, item, icon, cry, footprint, growth and egg-group references are checked
against the supplied base. The authoring reference vocabulary is ignored local
output at `target/minnesota-pokemon/profile-vocabulary.json`; it contains ID lists,
not source audio programs. Roster `PSYCHIC` translates to engine `PSYCHIC_TYPE`.
The current supplied base includes modern per-move physical/special categories.

Native art must have a current accepted visual-review SHA and matching original
source SHA, crop, fit, conversion settings and species palette. The builder
independently compares all PNG pixels with indexed rows and verifies the front
row-order/back column-order 2bpp bytes. It ships the exact 56×56 front and 48×48 back,
normal and shiny palettes, explicit static frame 0 main/idle programs, menu icon,
cry and footprint references. Static presentation is not a two-frame animation.

## Build and validate

The implementation's validation commands use an isolated build directory:

```sh
CARGO_TARGET_DIR=target/agent-minnesota-species cargo build --locked -p crystal-assets --bin pack_minnesota_pokemon --config 'profile.dev.package.crystal-assets.opt-level=0'
CARGO_TARGET_DIR=target/agent-minnesota-species cargo test --locked -p crystal-assets --lib minnesota_pokemon --config 'profile.test.package.crystal-assets.opt-level=0'
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" CARGO_TARGET_DIR=target/agent-minnesota-species cargo test --locked -p crystal-assets --lib species_overlay_preserves_real_base_and_rejects_unknown_moves --config 'profile.test.package.crystal-assets.opt-level=0' -- --ignored
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-geography-review.crystalpack" \
CRYSTAL_MINNESOTA_POKEMON_ROOT="$PWD/planning/minnesota/pokemon" \
CRYSTAL_MINNESOTA_NATIVE_ROOT="$PWD/target/minnesota-pokemon/native" \
CRYSTAL_MINNESOTA_PROFILES="$PWD/planning/minnesota/pokemon/species-profiles.json" \
CARGO_TARGET_DIR=target/agent-minnesota-species cargo test --locked -p crystal-assets --lib authored_catalog_constructs_parties_battles_evolutions_and_preserves_base --config 'profile.test.package.crystal-assets.opt-level=0' -- --ignored
CARGO_TARGET_DIR=target/agent-minnesota-species cargo clippy --locked -p crystal-assets --all-targets
cargo fmt --all -- --check
```

The per-package command-line override keeps this content-authoring crate quick
to compile while retaining the configured optimization of its engine dependencies.
It does not modify the repository's Cargo profiles or the game client.

Publisher command, after all authored profiles and native reviews are complete:

```sh
target/agent-minnesota-species/debug/pack_minnesota_pokemon \
  content-packs/minnesota-geography-review.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/minnesota-pokemon.crystalpack \
  --families-dir content-packs/minnesota-pokemon-families --jobs 4
```

Use `--family MN_F001` instead of `--families-dir` to publish one family. To retain
the regional maps, select an already verified regional pack as the base. Native
and browser bases retain their own original audio storage. Every output path must
be ignored by Git, untracked and different from the immutable base; the CLI
strictly reloads every written pack and compares its identity.
`--jobs` selects 1–8 independent family workers (default 1). Workers share the
immutable source catalog and base; every output path is validated before writing,
and each completed pack retains the same strict readback verification.

The builder preserves all original audio programs, manifests and payload bytes,
all original runtime files and unrelated game data. It adds explicit Heavy Ball
weight entries, empty or authored egg lists, and sufficient Oak-rating coverage.
Every added species must pass the same runtime catalog checks as the core.

After publication, validate the actual output, including its original audio:

```sh
target/runtime-fps/wasm-dev/verify_crystalpack content-packs/minnesota-pokemon.crystalpack --audio --desktop
```

Verified on 2026-10-26: the aggregate and all 123 family snapshots were written,
read back and identity-checked. The aggregate contains 471 maps, 517 species,
251 moves and 377 audio programs. Every original audio PCM hash, frame count and
loop range matches. [Local pack index](PACKS.md) links every family and the
aggregate; `content-packs/minnesota-pokemon.sha256` covers all 124 files.

All eight focused Rust tests pass, including the actual full authored catalog.
The actual aggregate also passes the existing production-controller Mom/exit-gate
and battle-turn tests, MCP movement/Start-menu test and clean-stderr regression,
run through the cached production test binaries. The separate sprite gate passes
all 532 views and 266 recorded visual reviews. All 26,600 default move sets
(every new species at every level 1–100) retain a usable damaging attack.

Frozen profiles SHA-256:
`9a313c9047511046dbe4dd710532bc6bd1b82a52453b67132b156a68bcbd8440`.
Aggregate SHA-256:
`4914298567bffb13a59a2baab10607b4487c5e9a0c0628572514e1e9c372b638`.

New Rust files pass their direct formatting checks and targeted all-target Clippy
inspection. The required unrestricted Clippy command encounters pre-existing
`bad_bit_mask` in `crystal-core/src/multiplayer/mod.rs:1332` and existing
`never_loop` errors in the assets crate's mutation/script parser modules.
`cargo fmt --all -- --check` also reports pre-existing workspace differences.
Those unrelated files were not reformatted or repaired by this work.

The opt-in authored-catalog regression builds in memory without writing a pack.
It constructs every new species at levels 1, 50 and 100, validates party and
Day Care save references, resolves Heavy Ball metadata, registers every species
in the dex, exercises authored level evolutions, and executes real battle turns
with the highest-numbered species and a new Water/Steel species. It checks real
damage and PP consumption and exact preservation of original audio/runtime bytes.
This checks Day Care references and egg-move closure; it does not simulate a full
breeding cycle or substitute for the controller's interactive gameplay checks.
