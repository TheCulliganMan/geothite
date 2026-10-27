# Sprite publication: 20261027-sprites-11

The published Pokédex updates Dozebear, Denumbra and Mudmender with their accepted new front and rear sprites and matching shiny versions. Flarkindle and the earlier native refinements were already present in release10.

This release starts from the verified `20261027-northwoods-10` interface. The Rust example `publish_pokedex_sprite_revision` copies that immutable baseline, rewrites its immutable asset base and replaces the selected native/shiny sprites using registered indexed art and authored palettes. All 1,611 paths remain present. After asset-base normalization, exactly twelve PNGs differ: two views × two palettes × three species. Every other file, including the complete 220-active/93-cut catalog, eighteen-type UI, move facts, learnsets, controls and existing signature previews, is identical.

The parallel fourteen-import animation UI was unfinished during this release. It is not part of this sprite-only publication. Broad test attempts against that work exposed its missing presentation overlay; their logs are retained without passing claims. The published site uses the proven release10 interface rather than an incomplete feature branch.

## Art and visual proof

- [Dozebear and Denumbra design/source/prompt](../sprites/mn-f136/differentiation-v2/DESIGN.md): low sleepy quadruped versus tall upright den guardian.
- [Mudmender design/source/prompts](../sprites/mn-f140/mudmender-redesign-v2/DESIGN.md): exposed broad head, shorter gills and open side cradle tail.
- [Local desktop/mobile screenshots](../../renders/sprite-publication-20261027-v11/local-browser/).
- [Public desktop/mobile screenshots](../../renders/sprite-publication-20261027-v11/public-browser/).
- Fresh actual battle captures remain in `planning/minnesota/renders/roster-battles-20261027-sprites-v4/`, including all220 species and both battler banks. Dozebear, Denumbra and Mudmender were visually inspected in the production battle view.

All six selected normal PNGs match canonical native art; all twelve normal/shiny files are the only content changes. Both image controls and shiny toggles pass at1280px and390px for all three species, with correct56px/48px source dimensions, no horizontal overflow and no page errors.

## Validation

The rebuilt external review pack contains471 species,270 moves and377 audio programs; all PCM hashes, frame counts and loop ranges pass. Rebuilding the species overlay with the current compiler produced the exact same bytes as the initial verified build. Production roster, starter, Fairy combat/controller, native donor and signature-controller checks passed. The five existing signature sequences also pass fresh production raster/OAM/APNG checks with legal level60 signature actors; twenty palette/side variants are retained under `planning/minnesota/renders/signatures-20261027-sprite-final-v8/`.

The new Rust example builds and passes scoped rustfmt. Whole-workspace formatting still fails first in `offset-allocator/src/lib.rs:408`; all-target Clippy remains blocked by the existing `crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask error. These are not passing checks.

The atomic deployer preserves the existing service image, environment, ports, networks and mounts. It confirms root/health availability and exact served index HTML. All1,926 public exact-byte comparisons and six health/private-route checks pass. Public desktop/mobile front/back/shiny controls also pass. [Public verification](../../renders/sprite-publication-20261027-v11/public-verification.json) records every checked route and hash. No shared gameplay pack or running game executable was overwritten.

The durable creature archive now retains **86,922 files** in [immutable inventory `d42b59bc001636d38a4ee7b909c3c47d3b1298b03dd61125e0e877142415ed0a`](../../renders/creature-archive/inventory-d42b59bc001636d38a4ee7b909c3c47d3b1298b03dd61125e0e877142415ed0a.json), including the new sources, native alternatives, production captures, exported palettes and browser/public proof.
