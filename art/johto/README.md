# Editable Johto art

Code, runtime meshes and editable authoring sources live in this repository.
These are original geometric assets, without imported game textures, ROM data,
content packs or third-party model files.

## Editable Blender sources

`source/manifest.json` lists 57 complete editable Blender sources. The original
New Bark, player-house, connected-Johto, character and foliage kits remain,
alongside human families, creature/prop actors, battle species and refinements,
interiors, world exteriors, dungeons and specialty room/structure kits. The latest
addition is `cable-club.blend`. The manifest is the complete filename inventory;
source counts do not imply visual approval of every model or scene.

The complete Blender files use gzip compression and are stored in small,
repository-contained base64 text chunks (at most 64 KiB decoded each). All `.b64`
chunks are ordinary Git text blobs. A normal checkout contains everything; no LFS object server, credentials,
network download or content pack is needed to reconstruct the sources.

Run:

```sh
python3 tools/johto_art_sources.py
```

This verifies every chunk and the compressed and expanded complete-file SHA256,
then reconstructs all canonical gzip-compressed `.blend` files in
`art/johto/source/`. Blender opens these directly. Existing locally edited files
are protected unless `--force` is explicitly supplied. `--output-dir` selects
another destination; `--check` performs the same decoding, hash and Blender-header
checks in memory without writing files. Reconstructed files are ignored by Git.
The latest check validates all 57 sources and 536 chunks with no orphaned chunks;
it does not claim a new Blender-open or visual review of every source.

`source/manifest.json` records order, sizes and hashes of every chunk plus the
compressed and uncompressed complete-file identities. `python3
tools/check-johto-models.py` validates the source bytes, Blender headers, runtime
model JSON and the renderer's embedded references. CI runs this after an ordinary
checkout. The gzip-compressed sources total 33,249,672 bytes (44,334,264 bytes
as base64 text), expanding to 305,027,040 bytes. The largest reconstructed gzip
file is 2,899,107 bytes. This bounded, fully in-repository format avoids missing
LFS objects and retains the exact authored files. Future asset edits must
regenerate affected chunks and manifest hashes before committing.

## Reproducible generators and runtime assets

The original generators live in `tools/`:

- `build-new-bark-models.py`
- `build-new-bark-player-house.py`
- `build-connected-johto-models.py`
- `build-johto-characters.py`
- `build-johto-foliage-lod.py`

Additional `build-*.py` generators and their per-kit documentation cover the
expanded catalog, including `build-cable-club.py`. Runtime model documents are
committed under `crates/crystal-voxel-view/models/`, with per-kit READMEs. Plain
JSON and lossless gzip/base64 envelopes stay in normal Git; use the
[storage helpers](../../docs/art/model-storage.md) when reading or regenerating
a compressed document.
Character animation is implemented in `src/new_bark_actors.rs`; the `.blend`
files contain editable joint/mesh authoring data, not prerecorded gameplay.

See `docs/new-bark-3d.md` for generation, controls, model scope, captures,
measured performance, verification and known limitations. Regenerated previews,
GLBs, compiled binaries and game packs remain outside tracked source.
