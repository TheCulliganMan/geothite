# Shared optional artwork

These are the current runtime-ready scenery, Kenney character rigs, and eight
NPC texture paintings from PR #5. They are stored in Git LFS so a new checkout
can reproduce the current artwork without the author's local asset directory.
The rejected realistic Quaternius people prototype and rounded-rig experiment
are excluded. This records the current art, including the character issues
listed in `docs/art/pr5-polish.md`; it does not imply the cast is finished.

Install Git LFS, then run from the repository root:

```sh
git lfs install --local
git lfs pull
export CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models"
```

Native clients and character reviews use that environment variable. The browser
build `sh tools/new-bark-3d-web.sh` selects this directory automatically unless
an explicit `CRYSTAL_OPEN_MODEL_ROOT` overrides it. It validates and bundles
these assets into ignored build output using the existing Rust tools.

Game content packs, ROMs, saves, source download archives, generated bundles,
and review renders remain external and ignored. A compatible user-supplied
content pack is still required; this directory does not contain game content.

## Provenance

- `interiors/`: converted Kenney Furniture Kit geometry (CC0).
- `new_bark/` and `johto/`: adapted Quaternius Ultimate Nature geometry (CC0),
  including runtime tree variants.
- `people/kenney/`: adapted Kenney Animated Characters Protagonists and Retro
  rigs (version 1.1, CC0), retaining authored animation clips.
- `people/npc-textures/`: project-specific image-generated costume paintings.

Original publisher license notices are in `licenses/`. Conversion utilities
remain in `tools/`; original source packs are not included in this runtime set.
