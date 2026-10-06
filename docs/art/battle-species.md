# Exact-species battle sculpture kit

The render-only registry in `crates/crystal-voxel-view/src/battle_species_models.rs`
contains 220 original, volumetric species sculptures. Together with the 31 exact
species in `new_bark_actors::actor_props`, this covers the 251-species game catalog. Generic
icon families never count as species support. `mesh(species)` returns `None` for unknown identities, and
`supported_species()` enumerates only models with embedded runtime geometry.

## Design and source ownership

The kit uses a consistent small-sculpture, low-poly art direction: solid rounded
anatomy, sculpted eyes and mouth details, extruded fins and membranes, material
boundaries, closed tapered limbs, and curved articulated tails. Evolutionary
families share modeling helpers, but each species has explicit anatomy such as
an ear form, horn, flower, shell, crest, number of limbs, tail shape, or wing
structure. No generic family model is assigned to a different species through
palette substitution.

`tools/build-battle-species.py` is the original editable generator. It reads no
content packs, PNGs, models, textures, disassembly, ROMs or external catalogs.
Temporary contact sheets from the locally supplied ignored content pack were
used only as a visual reference during authoring. Neither those sheets nor
source game pixels are part of the repository or exported meshes.

Every Blender collection has a species-named floor root and separate named mesh
objects for relevant anatomy. Parts are suitable for subsequent animation
rigging and remain editable; these exports currently present the authored idle
pose. The kit does not claim a skeletal animation rig, alternative forms,
gender-specific variants, or shiny palettes. Presentation must preserve the
source-specific fallback for forms the model does not represent.

## Rebuild and review

Use Blender 4.3 or newer, bounded batches, and two workers:

```sh
blender -b -t 2 --python tools/build-battle-species.py -- \
  /tmp/geothite-battle-species --preview
```

The output includes original runtime `.mesh.json`, numbered editable `.blend`
source scenes, and optional review-only PNGs. Use `--only raichu,wartortle` for a
focused authoring pass, or `--batch-size 12` to adjust bounded scene size. Names
and registry identities are ASCII and match the public battle species IDs,
including `FARFETCH_D`, `MR__MIME`, and `PORYGON2`.

Only approved runtime mesh JSON belongs in `models/battle_species`. Do not copy
preview images, logs, Python caches, or other build products into tracked paths.
Editable Blender scenes are transported through the existing hash-verified
`art/johto/source` source-chunk convention; restore them with the common art
source tooling. Keep each updated registry and all of its mesh files together
as a coherent increment.

## Geometry contract and verification

Exports use meters in a consistent stylized scale, +Y up, front +Z, and a
floor-centered root. Blender sources use +Z up and front -Y. Each primitive has a
semantic `part` name, XYZ positions, outward unit normals, triangle indices, and
linear RGBA material color. Objects are fully three-dimensional; fins and leaves
have thickness rather than being billboards. The renderer remains independent
of game simulation and does not mutate battle state.

Run:

```sh
python tools/check-battle-species.py
```

The check proves generator/registry/file parity, no repeated registry identities,
named parts, finite attributes, valid indices, unit normals, consistent outward
triangle winding, positive closed volumes, volumetric bounds, floor anchoring,
triangle budgets, bounded encounter-scale dimensions, and distinct species geometry
independently of palette colors. Young and evolved stages also have explicitly
authored presentation heights.
For a deliberately partial authoring increment only, `--allow-pending` permits
uninstalled builders; it does not waive any installed geometry checks.

Rust unit tests in `battle_species_models.rs` exercise every installed model and
explicitly reject generic icon names, unknown species, and texture paths. The
battle renderer decides whether a currently visible battler is eligible for its
exact model; transformations and source-specific presentation take precedence.
