# Raw model source and compiled storage

Each model also has an adjacent [GitHub-viewable STL](model-stl.md) with the
exact indexed geometry, while JSON and Blender retain materials and rigs.

Every canonical runtime model is a normal UTF-8 `.json` file under
`crates/crystal-voxel-view/models/`. The 655 documents contain the exact original
82,988,790 decoded bytes. They can be read, edited and diffed directly. No model
payloads are stored as base64, gzip wrappers, or transport chunks.

Cargo's `build.rs` generates deterministic gzip and identity metadata only in
ignored `OUT_DIR`. This keeps the executable and WebAssembly data segment compact
without changing source readability. Build compression uses a zero timestamp,
fixed level and fixed OS byte. It does not rewrite source files. Each catalog
still decodes inside its existing `OnceLock`; parsed meshes/rigs are cached and
temporary JSON is dropped. Shared human geometry remains shared. No model is
decoded or parsed per frame, and runtime loading needs no filesystem or network.

The runtime validates compressed and decoded size/SHA-256, UTF-8, and complete
single-member gzip boundaries. Input is bounded to 16 MiB. A regression compares
every build-embedded document with the exact canonical bytes, including signed
zero and original float precision.

```sh
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
python3 tools/test_model_asset_storage.py
python3 tools/test_johto_art_sources.py
python3 tools/check-johto-models.py
cargo test --locked -p crystal-voxel-view --lib
cargo check --locked -p crystal-bevy --bin crystal-bevy \
  --features fullscreen-scaling,voxel-view,location-tester \
  --target wasm32-unknown-unknown
```

Authoring tools use `read_model_bytes`, `read_model_text` or `read_model_json`.
These accept raw model JSON only. `store_model_bytes` validates before writing
and preserves exact bytes. Validation rejects obsolete model chunks, wrappers
and encoded siblings. No reconstruction step is needed for external editors.

`tools/migrate_asset_storage.py` is a one-time strict reader for the historical
transport formats. It verifies every input before writing, preserves all 70
native `.blend` identities and all 655 decoded model identities, removes only
validated obsolete payloads, and refuses to overwrite local source edits.
`--check` is read-only; a second migration is idempotent. No Git history is
rewritten. The Blender files open directly in Blender, which understands their
native compressed-save format.
