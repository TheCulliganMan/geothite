# Compressed, lossless model source storage

The expanded runtime catalog stores 498 original JSON documents as deterministic
UTF-8 gzip/base64 envelopes. They occupy 13,009,256 bytes instead of 59,544,891
canonical JSON bytes. Each envelope records the exact original size and SHA-256.
The existing editable Blender scenes and generator definitions are unchanged.

Every catalog decodes inside its existing `OnceLock` loader. The parsed model is
cached; decoding, hashing and parsing do not repeat each rendered frame. Temporary
decoded JSON is dropped instead of being retained beside the parsed geometry.
Plain legacy JSON remains supported. The shared loader uses the workspace's
existing base64, flate2 and sha2 versions, bounds decoded documents to 16 MiB,
checks complete gzip boundaries, validates SHA-256 and rejects malformed input.

The shared human geometry envelope is the only compressed document large enough
to need bounded transport chunks. Its canonical `.json` path contains a small
ordered descriptor; text chunks are at most 350,000 bytes. A generated sibling
`.include.rs` uses `concat!(include_str!(...))` to provide its complete envelope
at compile time. It has no runtime file-system dependency. No LFS, credentials,
binary transport or duplicated full canonical model file is needed.

Python consumers use `read_model_bytes`, `read_model_text` or `read_model_json`
from `tools/model_asset_storage.py`. They verify the exact generated Rust wrapper,
ordinal paths, whole/per-chunk hashes, decoded hashes, byte bounds and missing or
orphaned files. A descriptor is never silently treated as an empty model.

```sh
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
python3 tools/test_model_asset_storage.py
python3 tools/check-johto-models.py
cargo test --locked -p crystal-voxel-view --lib
```

Reconstruct ordinary JSON for external tools into an ignored output directory;
existing outputs are protected from overwrite:

```sh
python3 tools/model_asset_storage.py reconstruct \
  crates/crystal-voxel-view/models/johto_characters/shared.geometry.json \
  target/reconstructed-models/shared.geometry.json
```

After regenerating a canonical asset, run `compress` with its path before
validation/publication. Compression is deterministic, with empty gzip filename,
zero timestamp and a fixed compression level. It also accepts an existing
compressed document or chunk descriptor. Only that asset's generated storage is
replaced; Blender sources are independent.

```sh
python3 tools/model_asset_storage.py compress \
  crates/crystal-voxel-view/models/johto_characters/shared.geometry.json
```

All 498 conversions were checked byte-for-byte against their committed canonical
predecessors. Existing geometric, material, joint and float-bit equivalence tests
validate the decoded models, not envelope metadata. Compression does not change
vertices, normals, indices, material colors, rig pivots, precision or signed zero.

## Representative loading check

Three alternating native headless runs loaded the same four maps (bedroom,
Goldenrod, Ecruteak and Ice Path) with the prior plain-JSON executable and the
compressed executable. All resulting coverage/mesh reports were identical.
Median wall time was 1.070 seconds plain and 1.069 seconds compressed; median
child peak RSS was 302,028 KiB and 294,672 KiB respectively. This small sample
checks loading regressions, not a universal startup or GPU-memory benchmark.
The native immersive-battle executable also built and displayed the same model
scene, and the `wasm32-unknown-unknown` check passed.
