# Repel exporter — exact-label source correction

The original reviewed `REPEL` payload carried 200 steps, while the pinned native source declares 100. The repaired review pack now carries 100. `SUPER_REPEL` and `MAX_REPEL` retain 200 and 250. The exporter had selected `RepelEffect:` by substring and found that suffix inside the earlier `SuperRepelEffect:` label.

Actual user-local source inspected: `/Users/ryanculligan/.local/share/geothite/export-workspace/vendor/pokecrystal/engine/items/item_effects.asm`, SHA256 `95fe7b6c23c9b3db40259dc23d2ceea6c307349c1b01ae7d66d92cf0ae85e9a5`. Its Git HEAD is the repository's exact pinned `3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6`, tree `64288c8eacf03b7797a9e18b16def659a4ff4dd7`. `git diff --quiet HEAD -- engine/items/item_effects.asm` exits0. Both `pret-source.lock.json` and the external workspace's `asm-source.lock.json` identify that same revision/tree; no source checkout was changed or copied into this repo.

| Item | Exact routine declaration / literal line | Source duration | Existing native duration | Narrow correction |
|---|---|---|---|---|
| REPEL |RepelEffect2072 /2073|100|200|100|
| SUPER_REPEL |SuperRepelEffect2064 /2065|200|200|unchanged|
| MAX_REPEL |MaxRepelEffect2068 /2069|250|250|unchanged|

The literal order in that file is Super, Max, then ordinary Repel. Both current reviewed packs were read directly: tight-gallery SHA `fe57f83bdddf43fd1e617abb4cac7855f6118691ee07479dd77be27c1c815fb0`, and tor SHA `4891e6a084e21ea05866740fb04cc8c9891fca2433dbb5a93b58269aed78b72a`. Both carry200/200/250 respectively. The existing external generated `apps/web/assets/data/content-packs/core-modular/items/REPEL.json` also contains200, corroborating the source exporter origin. No catalog was generated or changed by this audit.

Focused fix: `crates/crystal-assets/src/bin/pack_core/source_items.rs` now finds an exact whole declaration line after stripping assembly comments/whitespace, then reads the first nonempty literal instruction. It cannot match the Repel suffix inside another routine or inside a comment. Missing/empty routines and a following label without a literal still fail. Other item/table parsing is untouched.

Two synthetic regressions were added alongside the existing status-mask test. The first reproduces overlapping names with Super before ordinary Repel and checks all three durations, including hexadecimal literal parsing. The second rejects missing/comment-only/empty routines and verifies a changed synthetic literal is read instead of hardcoding100. These are small synthetic snippets, not copied native source/catalog dumps. Existing integration target `pret_source_items.rs` already includes this leaf, so no registration is needed.

Scoped `rustfmt --edition 2024` and `--check` passed. This worker ran **no Cargo, exporter or pack publication**. Root performed the test and publication execution recorded below.

A current-pack repair must certify only `data.items.REPEL.repel_steps:200→100`, with the expected derived content identity change. Keep the remaining REPEL fields, SUPER_REPEL200, MAX_REPEL250, all other items/maps/programs/assets/audio/identities and external catalogs untouched. Root should assert the captured baseline pack/hash and all three original durations before mutation, normalize that one field for complete before/after equality and verify the actual written pack. The exporter fix independently prevents future regeneration from restoring200. No generic cave module was edited.

The focused implementation is now registered in `original_repel_revision.rs`. Both exporter and repair use `parse_source_repel_duration`, which also rejects duplicate exact labels. `repair_original_repel_duration` requires source literals 100/200/250 and matching native effects/durations 200/200/250, verifies the base, clones it, changes only ordinary Repel's duration, derives its content identity, and verifies the result. Already repaired packs are rejected. The `repair_original_repel` CLI uses the same function, retains storage, and strictly reads the written result back for complete equality.

Root executed the focused tests and strict publication below. The actual-pack tests require geometry-stage pack SHA `96a10295af34deeb57cd8ebdaf882d8eb80a70e49f63f8bc27ef84980dc1bcc1` and the pinned source SHA above; environment overrides select canonicalized paths without relaxing either hash assertion.

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1 RUST_MIN_STACK=33554432 \
  cargo test --locked -p crystal-assets --profile wasm-dev \
  --config profile.wasm-dev.opt-level=0 \
  --config profile.wasm-dev.package.offset-allocator.opt-level=2 \
  --test original_repel_revision --test pret_source_items -- \
  --include-ignored --test-threads=1
./target/runtime-fps/wasm-dev/repair_original_repel \
  content-packs/minnesota-northstar-rockcut-review.crystalpack \
  /Users/ryanculligan/.local/share/geothite/export-workspace/vendor/pokecrystal/engine/items/item_effects.asm \
  content-packs/minnesota-northstar-repel-review.crystalpack
```

`CRYSTAL_ORIGINAL_REPEL_REVISION_PACK` and `CRYSTAL_ORIGINAL_REPEL_SOURCE` override test input paths. The four actual-pack cases prove complete remaining data/storage/audio/report/manifest equality, actual source parsing and source-drift rejection, specific preimage rejection, uncached reconstruction of every map, and actual written-pack roundtrip. The ordinary synthetic test covers overlapping/missing/comment-only/duplicate/nonliteral labels without requiring a pack. All five repair tests passed in 8.50 seconds, and all three exporter tests passed; actual log `.local/repel-source-exact-tests.log`. The strict publisher passed, with written readback equality; log `.local/repel-source-native-publication.log`.

Published review pack: `content-packs/minnesota-northstar-repel-review.crystalpack`, file SHA256 `31258a1788a986b158d0679e4f3a26c4dd476964311f707a6e09c9640cad48c6`, derived content identity `be6c444a94b990020e56433609628723e4a1a0164a386ff544ef88c4080b5a13`. Root's read-only final audit passed 222 checks, including four rejected negative mutations, with the exact new field `data.items.REPEL.repel_steps:200→100`; `.local/cave-batch-final-repel-review01.log`. All geometry sheet pixels remained unchanged. This certifies the pack mutation; production item-use/counter acceptance is separate. Its first run found a runtime request for nonexistent `RepelUseText`; the pack already retains native `_ItemUsedText`, so the bounded presentation correction is a runtime change rather than a catalog mutation. Existing saves are not migrated, and an already active Repel counter is not rewritten.

No native ASM, duplicate catalog, compiled pack or source dump was added to tracked content. Local scoped formatter checks passed. Clippy results are not claimed here. The worker ran no Cargo, exporter or publication; Root performed the execution reported above.
