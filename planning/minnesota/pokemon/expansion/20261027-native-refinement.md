# Native refinement of the eleven new families

Reviewed all 23 Pokémon / 46 native front and rear views against their source art. Applied 200 guarded pixel corrections across 22 fronts and five horizontal orientation corrections. The original source sheets, crops, fits, sampling and per-species palettes are preserved. Enemy fronts face left; rear portraits face the upper-right opponent.

These are native pixel refinements, authored directly in manifest recipes. No new ImageGen source was generated in this pass. Each edit records its expected original ink and replacement ink; the Rust converter and pack loader replay it against the original sampled source. Guarded edits now work with both area-vote and source-center sampling, avoiding a wholesale resampling of existing artwork.

| Family | Refinement | Reviewed front / rear sheet |
| --- | --- | --- |
| MN_F134 | Restored clear white eye windows and upright pupils on all three gardener moose. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F134/line.review.png) |
| MN_F135 | Clarified both eyes and turned Latchquill’s enemy view toward the player. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F135/line.review.png) |
| MN_F136 | Separated eyes from fur; rebuilt Dozebear’s tiny den-face so its two eyes and smile survive at native size. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F136/line.review.png) |
| MN_F137 | Clarified the turtle eyes without turning the muzzle glints into facial features. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F137/line.review.png) |
| MN_F138 | Tightened the three woodpecker eye shapes while retaining their beaks and crest silhouettes. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F138/line.review.png) |
| MN_F139 | Clarified Crookit’s paired eyes; turned both rear portraits toward the opponent. Crookveil’s existing front face is retained. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F139/line.review.png) |
| MN_F140 | Clarified both nursery mudpuppy eyes and removed one isolated pixel beside Mudmender’s face. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F140/line.review.png) |
| MN_F141 | Clarified the deer eyes, preserved Velvawn’s white tail patch, and turned both enemy fronts left. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F141/line.review.png) |
| MN_F142 | Separated sundew pupils from white face details while preserving dew glints and hat bands. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F142/line.review.png) |
| MN_F143 | Clarified the pupils in all four eye organs; retained the four-eyed design. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F143/line.review.png) |
| MN_F144 | Tightened the leaf-raft eye while preserving its overlapping puzzle leaves. | [Preview](../../renders/new-sprite-refinement-20261027-v1/MN_F144/line.review.png) |

All prior manifests remain in each family’s `native-refinement-20261027-v1/manifest-before.json`. Native alternatives and original source crops are retained in the linked render directories. Shared published game packs are managed separately; this review does not claim a web deployment.

Verification completed:

- All 46 active exports match their accepted sheet hashes and preserve the original sources, crops, fits, sampling and palettes. The 200 declared edits replay exactly; five mirrors are intentional.
- Independent readback confirms PNG/index/2bpp parity, correct front row-major and rear column-major tiles, exact four-color palettes, and clear native borders.
- `cargo test --locked -p crystal-assets native_loader_replays_guarded_edits_for_both_sampling_modes` passed its two-policy replay matrix, including rejection of undeclared painting, stale settings and unapplied edits.
- A temporary fixture containing the 11 families / 23 species built against `content-packs/core-modular.browser.crystalpack` with the freshly built `pack_minnesota_pokemon`. Its verified result contained 388 maps, 274 species and 251 moves. `verify_crystalpack --audio --desktop` passed all 377 audio PCM hashes, frame counts and loop ranges. The fixture and temporary pack were removed; no shared pack was overwritten.
- Direct `rustfmt --check` passes for both changed Rust files. Repository-wide `cargo fmt --all -- --check` still reports existing formatting differences, beginning in `crates/offset-allocator`. Required `cargo clippy --locked -p crystal-assets --all-targets` stops at the existing `clippy::bad_bit_mask` error in `crates/crystal-core/src/multiplayer/mod.rs:1332`; unrelated code was left intact.
