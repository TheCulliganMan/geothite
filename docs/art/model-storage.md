# Canonical model sources and compiled storage

Animated humans use one standard glTF 2.0 binary catalog at
`crates/crystal-voxel-view/models/johto_characters/catalog.glb`. Its named scenes
preserve all 75 character identities, rigid joint hierarchies, linear RGBA
materials, shared geometry, and node animation channels. Static scenery and
Pokémon awaiting articulation keep their existing raw JSON representation.
There is one canonical geometry representation for each asset. GitHub-only STL
copies and expanded human JSON copies are not tracked.

Cargo's `build.rs` compresses original JSON or GLB bytes into deterministic gzip
and identity metadata only in ignored `OUT_DIR`. The build does not flatten
hierarchies, bake transforms, discard clips, or rewrite source files. Compression
uses a zero timestamp, fixed level, and fixed OS byte. Runtime size/SHA-256 and
complete single-member gzip boundaries are checked before decoding; input is
bounded to 16 MiB. JSON parsing retains UTF-8 validation. The binary GLB loader
reads binary bytes directly. Tests compare every embedded source byte with its
canonical file.

The human catalog is parsed once. Only encountered character kinds assemble
joint meshes, each through its own bounded cache. Geometry accessors are shared
inside the catalog. Production retains sixteen rigid joint meshes per kind,
per-instance transforms, existing material behavior, and map-scoped entities.
No model is decoded or parsed per frame, and runtime loading requires no network
or filesystem. Animation sampling updates transforms, not mesh buffers.

The supported human animation contract is documented in
[animated models](animated-models.md). Standard GLB also supports skinned models,
but this catalog uses rigid joint-owned geometry: it has no skin weights. The
loader rejects unsupported features explicitly. Battle Pokémon articulation is
a separate incremental conversion; a format migration does not imply every
species already has authored motion.

```sh
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
python3 tools/check-model-layout.py
python3 tools/test_animated_glb.py
python3 tools/check-johto-characters.py
python3 tools/check-room-furniture-bindings.py
python3 tools/check-facility-table-rigs.py
cargo test --locked -p crystal-voxel-view --lib
cargo check --locked -p crystal-bevy --bin crystal-bevy \
  --features fullscreen-scaling,voxel-view,location-tester \
  --target wasm32-unknown-unknown
```

Normal game builds consume the canonical files and do not require Blender or
Python. Authoring tools may create ignored intermediate exports and editable
Blender scenes; reproducible scripts remain in `tools`. Unique authoring scenes
are retained. The optional [local STL exporter](model-stl.md) is for developer
inspection only and writes ignored output.

`tools/migrate_asset_storage.py` remains a strict reader for historical transport
formats. It validates source identity before writing and refuses to overwrite
local edits. No base64, gzip wrappers, transport chunks, or reconstruction step
is required to open current canonical model files. No Git history is rewritten.
