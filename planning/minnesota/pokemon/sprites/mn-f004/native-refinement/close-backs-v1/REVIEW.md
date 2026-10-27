# Close rear portrait proposal

Selected manifest: `manifest-selected.json`. Proposal only: three back views; existing approved fronts are intentionally excluded. No canonical manifest, palette, profile, or native directory was changed.

Built-in ImageGen produced `backs.imagegen-v1.png` (all three rear portraits) and `voyagear-back-v2.png` (larger square adult refinement). Exact prompts are retained beside them. Both generated originals remain under Codex's generated-images directory as well. All prior art and native candidates remain available.

Selection uses Birchit/Portabeau from v1 and Voyagear from v2. Native output is `target/agent-northwoods-moves/beaver-close-backs-selected`; the unselected adult v1 and Solid comparison are in sibling output directories. Conversion uses `minnesota_sprite_sampling_v5`, `source_center`, `source_backed`, unchanged per-species RGB555 palettes, and 46x47 maximum fit inside each 48x48 rear frame. Solid comparison produced the same visible results.

Visual review: all three show head/upper back with lower bodies cut off, no feet or full standing silhouettes. Birchit retains large rounded ears, white forelock, and only a cropped low birch-tail corner. Portabeau is bareheaded, has no paddle, and shows the white-stripe canoe-pack flap and attached shoulder straps. Adult head/neck and shoulder mass are much larger than the first wide composition; only one narrow right-facing eye profile is exposed. The straight paddle has blade at screen left and short T-grip at screen right. The raised right paw wraps the shaft, and the bent forearm joins the right shoulder; shaft also rests at the opposite shoulder. This preserves the actual holding arm when viewed from behind. Rust tuque/navy band, mature downslant brow and shoulder markings follow the new fierce adult front. The lower sash is only a partial bottom-edge strip. No complete adult tail is introduced into an upper-body crop.

Limit: this is a back-only review, not a full-family approval or pack validation. Root must combine the selected fronts and these backs, recompute the full-family review SHA, inspect the combined board, and run the strict graphics loader.

Encoding checks passed for all three: exact 48x48 PNG, 576-byte native 2bpp, column-major back-tile decode equals metadata rows, and every PNG pixel equals its corresponding RGB555 palette entry. Visible occupied rows: Birchit43, Portabeau39, Voyagear43. Partial `line.native.png` SHA256: `3ada78a26c7677975bbfee1c0c29fb431f7c83d6177787bdfbe1963859d63447`.
