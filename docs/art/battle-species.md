# Exact-species Pokémon model catalog

The render-only registry in `crates/crystal-voxel-view/src/battle_species_models.rs`
contains all 251 species. Canonical GLB skins cover 276 variants, including all
26 Unown letters; the base Unown identity resolves to I. Generic
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

`tools/build-battle-species.py` dispatches the species authoring recipes and
canonical GLB exporters before its legacy Blender batches. Most recipes build
original geometry with the Python standard library. Gengar, Raticate,
Pidgeotto and Totodile retain their stored geometry as authoring input, and
Meganium retains its reviewed-container equivalence path. See
[authoring portability](pokemon-authoring-portability.md) for the exact scope.
Local pack pixels serve only as review references; they are not exported into
the original model geometry or committed as a duplicate catalog.

The GLBs retain named anatomical parts, joint hierarchies, skin weights, inverse
binds and idle, attack and hit clips. World locomotion uses the same canonical
anatomy with species-specific movement and support controls. Pack measurements
determine physical game size; normalized authoring heights do not replace them.
Presentation preserves source-specific fallback for transformations and forms
the exact model does not represent.

Normal send-outs retain the same species mesh through the two reveal ticks,
using the controller's source frame to scale it around its checked contact.
Capture uses that same model and the source ball animation by default; the
isolated-image capture experiment is optional. Neither effect replaces the
whole encounter with the classic LCD. Human participants reuse the field's
weighted character rigs and paintings, preloaded through the shared actor queue
before the battle-entry wipe. Their visibility follows the retained entry,
trainer-exit and result narration, without advancing the controller.


Coverage and numerical validation do not establish appearance. The
[roster review](pokemon-authoring-portability.md) records windowless GPU pose
inspection and its limits. The [angle review](pokemon-roster-angle-review.md)
adds front, side and rear inspection of every current variant. Individual species guides document matched-state
production reviews. Earlier partial-coverage and static-mesh audit reports are
historical; the current geometry validator follows canonical GLB routing and
the unchanged Rust loader limits.

## Rebuild and review

Use Blender 4.3 or newer with a focused authoring selection:

```sh
blender -b -t 2 --python tools/build-battle-species.py -- \
  target/battle-species-review --only seel,dewgong
```

Canonical recipes emit GLBs; their own CLI modules can also export without
Blender. The legacy batching path emits mesh JSON, editable Blender scenes and
optional previews only for recipes still using that path. Names
and registry identities are ASCII and match the public battle species IDs,
including `FARFETCH_D`, `MR__MIME`, and `PORYGON2`.

Only approved canonical runtime assets belong in `models/battle_species`.
Actor-space species retain their single canonical files in `models/actor_props`.
Do not copy
preview images, logs, Python caches, or other build products into tracked paths.
Editable Blender scenes are transported through the existing hash-verified
`art/johto/source` source-chunk convention; restore them with the common art
source tooling. Keep each updated registry and all of its mesh files together
as a coherent increment.

## Geometry contract and verification

Exports use a consistent stylized authoring scale, +Y up, front +Z, and a
floor-centered root. Blender sources use +Z up and front -Y. Each primitive has a
semantic `part` name, XYZ positions, outward unit normals, triangle indices, and
linear RGBA material color. Objects are fully three-dimensional; fins and leaves
have thickness rather than being billboards. The renderer remains independent
of game simulation and does not mutate battle state.

Run:

```sh
python3 tools/check-battle-species.py --progress
python3 tools/check-model-layout.py
```

The check proves generator/registry/file parity, no repeated registry identities,
named parts, finite attributes, valid indices, unit normals, consistent outward
triangle winding, positive closed volumes, volumetric bounds, floor anchoring,
current runtime storage budgets, bounded encounter-scale dimensions, and distinct species geometry
independently of palette colors. Young and evolved stages also have explicitly
authored presentation heights.
For a deliberately partial authoring increment only, `--allow-pending` permits
uninstalled builders; it does not waive any installed geometry checks.

Rust unit tests in `battle_species_models.rs` exercise every installed model and
explicitly reject generic icon names, unknown species, and texture paths. The
battle renderer decides whether a currently visible battler is eligible for its
exact model; transformations and source-specific presentation take precedence.
