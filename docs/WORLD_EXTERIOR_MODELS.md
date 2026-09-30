# Authored world exterior kits

The exterior lane adds original editable meshes for complete source drawings in
Johto, modern Johto, Kanto, forest, park and Battle Tower outdoor tilesets. It
reuses the existing New Bark art where the source actually uses that architecture.
It does not infer objects from collision or treat every used metatile as converted.

## Architecture

- An east/west passage gate has two solid wings, a real open transverse passage,
  lintels, jambs, paving and a threshold fitted on the east edge
- Johto landmarks have separate five-tier pagoda, broken-roof Burned Tower,
  tapered lighthouse with open balcony and lantern drum, and timber barn meshes
- Modern Johto has independent residential/blank/storefront, Center, Mart, Gym,
  station, arcade, Day-Care, department-store and radio-tower forms. The radio
  antenna is triangulated open geometry
- Kanto has pitched/flat/doorless homes; distinct curved Center and stepped civic
  roofs; Mart, Gym, station, arcade, department-store, columned Museum, balcony
  Mansion, Silph curtain-rib tower and industrial power-station architecture
- Forest gateways have dedicated open/closed entrance variants with full deep
  roof, rear/side fenestration, timber braces, piers and nameboard. The small
  forest shrine has its own lattice-door, bell and rope model
- Battle Tower has a separate wide four-storey glass facade, side piers, projecting
  belts, twin entrance bays, podium and tournament crest

Every model contains full side/back surfaces; authored windows, trim, recessed
frames, sills and roof edges are geometry rather than source-art cards. Blank
facades do not fabricate an entrance. Source branches that cannot be proved
remain on the previous renderer.

## Nature and props

The kit includes conifers, broadleaf canopy trees, cuttable crowns, clipped
hedges, stratified shore boulders, thin signboards, slatted park benches, picket
rails and capped posts. Grass and flower geometry reuse the authored Johto
foliage vocabulary through exact outdoor source identities. Park long grass
retains a taller silhouette. Ground samples must match the originating tileset;
a same-number sample from a neighboring connected map cannot substitute.

## Matching and ownership

`src/mesh/modeled_exteriors.rs` validates the complete native metatile grammar and
every source-cell phase before selecting a building. It preserves source plot
edges and locates front thresholds from complete 2x2 door artwork. East gates
use a separate longitudinal fit; the lighthouse keeps its narrow physical depth
while leaving its south door seam fixed. Existing exact grouped-tree classifiers
supply complete nature placements. Cut replacements are resolved from the live
source frame, so the model disappears with its authoritative block change.

`preferred_cells` reserves only replacements with a valid ground sample before
legacy live-profile masking. Reservations do not count as authored output.
Successful append routines mark source cells with the specific asset label.
The coastal-rock/sign live-profile adapter additionally requires exact equality
with the checked-in canonical profile, including drawing, map scope, materials,
footing and dimensions. A user-edited profile with the same name stays on its
custom renderer. Legacy footing metadata is preserved.

## Editable source and reproduction

Run Blender from the repository root:

```sh
blender -b --threads 2 --python tools/build-world-exteriors.py -- \
  target/world-exteriors --skip-preview
```

The generator composes original named geometric parts using the existing New
Bark authoring vocabulary. It never opens a pack, imports a game texture or
exports game source data. `--only=name,name` supports memory-bounded batches.
Runtime JSON is material-indexed Y-up geometry; editable Blender collections
retain part names. Blender sources are published through the repository's
`tools/johto_art_sources.py` source-chunk workflow; generated renders and loose
working Blender files remain verification output.

Focused regression commands:

```sh
cargo test -p crystal-voxel-view exterior_models
cargo test -p crystal-voxel-view modeled_exteriors
```

These cover mesh integrity/budgets, directional door fitting, source phase,
unknown/cropped drawings, distinct regional mapping, same-tileset underlays and
canonical-profile preservation. Whole-world renderer coverage and screenshots
must be checked separately; mesh validity alone is not gameplay evidence.
