# Minnesota berry and apricorn stops

The current contract is `groves.compact-v01.json`, composed on the compact
`minnesota-r10-rock-gallery-v01.crystalpack` descendant selected in `world.json`.
It restores the two accepted winding river routes with twenty trainers each,
six supplies, two medics and ten berry trees per route. The final bank refinement
brings the routes to 29 grass patches and 1,744 grass gameplay tiles.
There are 211 native fruit trees across 86 outdoor chunks: 138 berries and
73 apricorns, covering all nine berry varieties and all seven apricorn colors.
Clear walking rings, reachable approaches and existing events are checked.

North Shore remains warm. Publication rejects obsolete northern geometry and
any icy R11 chunk. Existing northern footprints, river ports and HM gates stay
fixed. The Rust publisher preserves unrelated content and audio. The production
daily reset renews harvests without clearing permanent rewards.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/author_fruit_groves.py
CARGO_TARGET_DIR=target/river-fun cargo build --locked --profile wasm-dev \
  -p crystal-assets --bin pack_minnesota_fruit_groves --bin pack_frozen_water \
  --config profile.wasm-dev.package.crystal-core.opt-level=0 \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0
target/river-fun/wasm-dev/pack_minnesota_fruit_groves \
  content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  planning/minnesota/fruit-groves/groves.compact-v01.json \
  content-packs/minnesota-fruit-compact-v01.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_fruit_groves.py \
  --written content-packs/minnesota-fruit-compact-v01.crystalpack \
  --output planning/minnesota/renders/fruit-compact-v01/written-v01
```

Sources and review epochs must be fresh; do not overwrite a reviewed artifact.
The final delivery is `content-packs/minnesota-fruit-ice-compact-v02.crystalpack`.
Apply the [explicit bank refinement](../connections/mississippi-bank-refinement-v01/README.md)
after fruit publication and before freezing. The final frozen-inland overlay is documented in [FROZEN-WATER.md](../regions/ice/FROZEN-WATER.md).
[Historical receipts](HISTORICAL-v01.md) and all old render epochs are preserved.
The old seven candidate packs are quarantined as `.retired` files under
`.local/retired-map-candidates/`, with a SHA256 inventory. They cannot be
selected as current `.crystalpack` inputs. Old source contracts have a
`.json.retired` suffix. These historical verification claims do not certify
this current compact rebase; see its new publication receipts.
