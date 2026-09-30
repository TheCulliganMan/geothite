# Editable Johto art

Code, runtime meshes and editable authoring sources live in this repository.
These are original geometric assets, without imported game textures, ROM data,
content packs or third-party model files.

## Editable Blender sources

`source/` contains five self-contained Blender 4.3 files:

- `new-bark-kit.blend`: original houses, laboratory, trees, flowers and early character kit
- `player-house.blend`: refined two-story player home
- `connected-johto.blend`: Center, Mart, gate, cottages/academy, gym, Sprout Tower and grass
- `characters.blend`: nine articulated character looks with editable joint hierarchy
- `foliage-lod.blend`: source-bound, palette-matched distant tree/grass geometry

The complete Blender files use gzip compression and are stored in small,
repository-contained base64 text chunks (at most 64 KiB decoded each). All `.b64`
chunks are ordinary Git text blobs. A normal checkout contains everything; no LFS object server, credentials,
network download or content pack is needed to reconstruct the sources.

Run:

```sh
python3 tools/johto_art_sources.py
```

This verifies every chunk and complete-file SHA256, then reconstructs the five
canonical `.blend` filenames in `art/johto/source/`. They open directly in Blender;
all five were checked in Blender 4.3.2 with no external libraries or unpacked
image dependencies. Existing locally edited files are protected unless `--force`
is explicitly supplied. `--output-dir` selects another destination; `--check`
verifies without writing files. Reconstructed files are ignored by Git.

`source/manifest.json` records order, sizes and hashes of every chunk plus the
compressed and uncompressed complete-file identities. `python3
tools/check-johto-models.py` validates the source bytes, Blender headers, runtime
model JSON and the renderer's embedded references. CI runs this after an ordinary
checkout. The decoded sources total about 2.5 MB (3.3 MB as base64 text); the largest reconstructed file is about
1.3 MB. This bounded, fully in-repository format avoids missing LFS objects and
retains the exact authored files. Future asset edits must regenerate affected
chunks and manifest hashes before committing.

## Reproducible generators and runtime assets

Generators live in `tools/`:

- `build-new-bark-models.py`
- `build-new-bark-player-house.py`
- `build-connected-johto-models.py`
- `build-johto-characters.py`
- `build-johto-foliage-lod.py`

Runtime JSON is committed under `crates/crystal-voxel-view/models/`, with
per-kit READMEs. It stays in normal Git for small, inspectable build inputs.
Character animation is implemented in `src/new_bark_actors.rs`; the `.blend`
files contain editable joint/mesh authoring data, not prerecorded gameplay.

See `docs/new-bark-3d.md` for generation, controls, model scope, captures,
measured performance, verification and known limitations. Regenerated previews,
GLBs, compiled binaries and game packs remain outside tracked source.
