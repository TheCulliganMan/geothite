# Verdigrim field sprites

Selected source: `verdigrim.overworld.imagegen-v2.png`, generated with built-in
ImageGen using `prompt-verdigrim-v2.txt`. V1 and both exact prompts are retained.
The canonical identity is the float-copper wolf in
[`MN_F145`](../../../../sprites/mn-f145/DESIGN.md).

Selected compiler input: `verdigrim.manifest-v3-eye-registration.json`.
It uses one shared 14×14 fit within six 16×16 cells. Runtime order is down idle,
up idle, left idle, down step, up step, left step; right mirrors left. Three
visible inks are copper, green and black, with transparent index0. Normal and
shiny palettes exactly match the species authoring.

V1 lost face details at native size and its raised tail resembled a third ear.
V2 simplifies the copper mask, removes that silhouette defect and supplies
distinct paw steps. The v3 import crops align the existing source pupils with
native sample centers. No native pupil, contour or body pixel was hand-painted.
The separate source-feature proof binds all six actual source pupils across
the four nonrear views to the exact source, manifest and bank hashes. Both rear
views show the backs of the ears, shoulders and tail without a face.

Normal/shiny native views and all eight real production terrain scenes were
inspected. The production context regression passes with an explicit Route36
visual fixture; it proves rendering and native bank behavior, not Minnesota
story progression. Review artifacts remain in
`../../candidates/mn_f145/verdigrim-v3` and
`../../diagnostics/verdigrim-v3`. The accepted registry now contains 225/225
active species. All 224 earlier review records and 896 runtime files are unchanged
apart from dense display-number updates in the registry.

Installed field actor: **`SPRITE_VERDIGRIM`**. Runtime files under
`gfx/pokemon/verdigrim/` are `overworld.2bpp` (384 bytes), `overworld.png`
(16×96), `overworld.gbcpal` and `overworld-shiny.gbcpal` (8 bytes each).
Use the normal production object renderer and native movement facings.

Recompile using Rust:

```sh
cargo run --locked -p crystal-assets --bin compile_minnesota_overworld -- \
  --manifest planning/minnesota/pokemon/overworld/20261027-v1/sources/mn_f145/verdigrim.manifest-v3-eye-registration.json \
  --output-dir planning/minnesota/pokemon/overworld/20261027-v1/candidates/mn_f145/verdigrim-rebuild
```

To compose only this bank onto a verified story parent, use the existing Rust
`pack_minnesota_overworld BASE OUTPUT MANIFEST` tool. To publish the full
accepted regional art catalog, use `pack_reviewed_minnesota_overworld` after
installing the new species. Keep outputs ignored and choose fresh filenames.

The ready local reference pack is
`content-packs/verdigrim-assets-review-v02.crystalpack`. It combines the species,
this field bank and its original cry. It is based on the older Fairy v2 catalog
and is **not a current Minnesota geography parent**. The story owner must compose
the canonical source inputs onto the current selected world descendant.
