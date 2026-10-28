# Northwoods overworld sprites

Scope: all **224 active Northwoods species**. The [roster](ROSTER.md) and
[accepted registry](accepted.json) record all **224 accepted sets**. Each passed source-anatomy, native normal/shiny
and production-map visual review. The complete overlay was compiled and reopened
as `content-packs/northwoods-overworld-v1.crystalpack` (ignored), preserving all
game data, audio and unrelated runtime files. The [publication receipt](reviews/full224-pack-publication.json)
records its exact hash and the actual full-pack party-menu regression.

The [live Pokédex](https://geothite.ryanculligan.com/pokedex/) now uses immutable
release `20261027-overworld-224-18`: all 224 species have animated normal/shiny
walking thumbnails and a Cry button. Desktop and mobile media/machine controls
pass on the public site. See the [web runbook](../../../../../docs/pokedex-web.md).

Each accepted set has six 16×16 poses in a vertical 16×96 runtime PNG, a 384-byte
2bpp bank, and independent eight-byte normal/shiny RGB555 palettes. Index 0 is
transparent everywhere; only indices 1, 2 and 3 are visible. East mirrors west.
Canonical anatomy and eye counts take precedence over a generic two-eye template.

The same installed banks also supply the field Pokémon and battle PKMN menu
icons: front idle/step frames0/3 stay16×16 and use the individual normal/shiny
palette. Eggs, legacy species, held-item/mail overlays and HP-dependent animation
cadence retain their existing behavior. See the [menu verification](reviews/party-menu-integration.json).

- [Native size audit](reviews/native-size-audit.json): exact bounds and frame hashes.
- [Sources](sources): original ImageGen renders, exact prompts, edits and failed trials.
- [Root selections](reviews): explicit visual decisions and production-context evidence.
- [Production diagnostics](diagnostics): native/nearest map and actor review boards.
- [Runtime audit](runtime-audit.md): loader, native format and real map-test contracts.

Rust owns image sampling, palette quantization and native export through
[`compile_minnesota_overworld`](../../../../../crates/crystal-assets/src/bin/compile_minnesota_overworld.rs).
Source crops share one transform per species; poses must not independently scale.
Review the native output, true rear, actual eye locations, walking movement and
scale beside the player. Never substitute a battle-sprite downscale or paint
missing eyes into the converted bank.

After inspecting normal, shiny and production map boards, record an explicit
selection with [the metadata registrar](../../../tools/register_overworld_review.py).
It checks pose rows, palette bytes, border, movement, context coverage and exact
tested-bank hashes before recording acceptance. It also requires a separate
source-feature receipt bound to the exact manifest, bank and source hashes:
all four nonrear views must retain their actual anatomical eyes, and both rear
views must be reviewed. A black hand, ear mark or contour cannot count as a pupil.
Bright visor windows and canonical single-eye designs retain their own eye forms.
New artists can supply the same normalized receipt shape as
[Rondive's source-feature proof](sources/mn_f143/source-feature-proof-rondive-v2.json),
with their own canonical eye form and detailed per-pose evidence. The registrar
also refuses a scale, alpha threshold or source crop changed after compilation.
The [independent source-eye audit](reviews/source-eye-audit-report-56-v1.md)
records why 17 earlier candidates were held for repair; their original assets
and revoked receipts remain available. The registrar does not create artwork.
All source renders and rejected attempts remain retained for reuse.

## Reviewed pack publication

The Rust `pack_reviewed_minnesota_overworld` binary installs only native runtime files.
It requires the complete current curated roster before writing a pack. It checks
reviewed source/manifest/file hashes and root acceptance, recompiles every selected
source manifest and compares the four native files byte for byte, and verifies
normal/shiny palettes against current species authoring. The reopened pack must
preserve all game data, audio and unrelated runtime files. Existing conflicting
art is rejected. Choose a fresh ignored output pack; the source pack is preserved.

Complete publication recompiled all 224 selected source manifests and matched
all 896 reviewed runtime files and current palettes before writing the fresh pack.
Reopening verified the result and repeated preservation checks. The independent
[native audit](verification/finish-independent-native-audit.json) checked 1,344
frames and 344,064 source-to-native pixels; its separate
[provenance addendum](verification/finish-independent-default-provenance-addendum.json)
confirms all 292 selected source images match retained default originals exactly.
All original and rejected renders remain retained.

Both focused publisher tests and all 17 exporter tests pass. Changed Rust files
pass rustfmt; whole-workspace formatting retains unrelated offset-allocator
differences, and required all-target Clippy stops at the existing
`crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask error.

Executed build and publication commands (choose a new output filename when repeating):

```sh
CARGO_TARGET_DIR=target/agent-northwoods-overworld-art-rebuild \
  cargo build --locked -p crystal-assets --bin pack_reviewed_minnesota_overworld \
  --config profile.dev.package.crystal-assets.opt-level=0

# Verify accepted banks against a supplied pack without writing a partial pack.
target/agent-northwoods-overworld-art-rebuild/debug/pack_reviewed_minnesota_overworld \
  --verify-reviewed planning/minnesota/pokemon \
  planning/minnesota/pokemon/overworld/20261027-v1 \
  content-packs/northwoods-spinner-overworld-v1.crystalpack

# Publication requires every current active species to be accepted.
target/agent-northwoods-overworld-art-rebuild/debug/pack_reviewed_minnesota_overworld \
  content-packs/northwoods-spinner-overworld-v1.crystalpack \
  planning/minnesota/pokemon planning/minnesota/pokemon/overworld/20261027-v1 \
  content-packs/northwoods-overworld-v1.crystalpack
```

Use the latest verified shared pack as the input when publishing; the example
base above is the current local Crappeller review pack. The native pack remains local and ignored. The separate
Pokédex deployment is recorded in the web runbook; it preserves the running game
container's existing configuration and pack mounts.
