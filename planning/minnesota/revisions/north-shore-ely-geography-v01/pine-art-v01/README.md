# Native snowy pines

The coast stays warm. Inland winter should preserve the narrow paired Johto
pine silhouette rather than silently replace it with the broad forest tree.
The frozen [contract](native.contract.json), SHA256
`7b6a19623f04ad706259211a9cb6411ae95e50202c87eff67f7430fbb058b36e`,
adds one recipe, `mn_winter_north_shore:8b`, with four WALL quadrants.
The asset name is historical; assigning winter terrain to North Shore is not
part of this operation.

Four snowy pine tip/base glyphs already exist in the retained roof recipes.
Only native Johto `2e` and `2f` are missing. Copy their native bitplanes into
unused private slots `fe` and `ff`, applying the exact existing canopy shade
map and foreground profile. Assemble `8b` with the same glyph arrangement as
Johto `05`. No new bitmap shapes, old recipe edits or palette substitutions
are needed. Source and target definitions, nine source files, exact remapped
bitplanes and the complete new recipe are guarded.

Root inspected the retained native-source comparison at
`renders/north-shore-ely-geography-v01/pine-art-v01/pine-silhouette-comparison.png`.
It preserves the paired pointed trees across the snowline. This source preview
does not establish native publication, controller traversal or GPU acceptance.

`build_inland_snow_pines_pack` changes only the owned definition append, six
runtime art mirrors and derived identity. Full inverse equality preserves all
maps, scripts, existing glyphs/recipes, other art, audio and report. The CLI
requires `BASE CONTRACT EXPECTED_PARENT_FILE_SHA NEW_OUTPUT`, verifies the actual
parent file, rejects an existing output and strictly reloads the result.
Later approved parents may compose when every source preimage remains exact.

Three external regressions use `CRYSTAL_INLAND_PINES_TEST_PACK`: native prefix
preservation, stale-source/contract rejection, and residual counterexamples for
old glyphs/recipes/palettes, unrelated terrain and native programs. Execution
and written-native acceptance receipts belong in fresh ignored render epochs;
their status must not be inferred from the existence of this source bundle.
