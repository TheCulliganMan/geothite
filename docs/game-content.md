# Game content setup

Geothite does not distribute game packs or ROMs in Git. Download the source
separately from [pret/pokecrystal](https://github.com/pret/pokecrystal) and
follow its [assembly instructions](https://github.com/pret/pokecrystal/blob/master/INSTALL.md).
Keep the disassembly and its build outputs outside this repository.

## Assemble the disassembly

Install the tools specified by pret for your platform, then run outside Geothite:

```sh
git clone https://github.com/pret/pokecrystal.git
cd pokecrystal
make
```

This assembles a ROM. A ROM is not a `.crystalpack` and cannot be renamed into
one. Pret supplies the disassembly, not a ready-made Geothite pack.

## Compile a Geothite pack

The existing Rust pack compiler expects an external export workspace containing:

- `vendor/pokecrystal/`: the compatible disassembly source.
- `apps/web/assets/`: exported runtime assets, including
  `data/content-packs/index.json`, the generated core manifest, and all data,
  graphics, and audio referenced by those manifests.

From Geothite, compile an already prepared export workspace with:

```sh
cargo run --release -p crystal-assets --bin pack_core -- /path/to/export-workspace
```

The compiler writes `content-packs/core-modular.crystalpack` and
`content-packs/core-modular.browser.crystalpack` inside that workspace.

**Current limitation:** this repository does not contain a complete exporter
that creates the intermediate runtime assets from a fresh pret checkout or ROM.
The command above is the final packing step, not an end-to-end disassembly
converter. A prepared compatible export workspace or a separately obtained
compatible pack is required to play. A clean clone alone cannot build game
content yet.

## Supply content locally

For Docker/browser deployment, copy your compatible browser pack into the
ignored local directory before running Docker Compose:

```sh
mkdir -p content-packs
cp /path/to/export-workspace/content-packs/core-modular.browser.crystalpack content-packs/
docker compose -f docker-compose.production.yml up -d --build
```

Docker copies that local file into your deployment image. Missing content causes
the Docker build to fail; it is not downloaded automatically. Native play accepts
a separately supplied pack through `--pack /path/to/core-modular.crystalpack`.

Keep packs, ROMs, disassembly checkouts, and generated assets out of commits.
After the history cleanup, use a fresh clone. Merging or pushing an old branch
can reintroduce the removed history.
