# Original connected Johto environment kit

Original geometric art authored by `tools/build-connected-johto-models.py` using
reusable geometric helpers in `tools/build-new-bark-models.py`. No game graphics,
map exports, collision data, or scripts are stored in these meshes. The external
ignored pack remains authoritative. Blender is an authoring dependency only.

## Rebuild and inspect

```sh
blender -b --python tools/build-connected-johto-models.py -- target/connected-johto-assets
```

This writes editable Blender collections, GLBs, runtime JSON, and an art-kit
preview under ignored `target/`. `--skip-preview` omits the studio render. Only
the original `.mesh.json` assets in this directory are runtime inputs. Nothing
from the presentation lineup is exported into them.

## Runtime models

- `pokecenter`: red tiled roof, carved round badge, glass entrance, lit canopy
- `mart`: blue tiled roof, striped awning, modeled M sign and glazed windows
- `route_gate`: teal-roofed gateway, open-looking recessed threshold, route plaque
- `traditional_house`: violet tiled roof, timber frame and latticed windows;
  reused as the source-fitted Earl's Academy facade
- `violet_gym`: broad violet roof, carved nameboard and restrained wing crest
- `sprout_tower`: three diminishing timber storeys, tiered roofs and brass finial
- `grass`: eleven folded geometric blades, 88 triangles per source cell

Each mesh is floor-centered, +Y up/front +Z. Buildings carry an explicit
south-threshold anchor. Exact occupied bounds include their roof eaves,
thresholds, awnings, and trim. All materials use linear RGBA values. Model
import validates finite geometry, indices, normals, and colors before fitting.
Buildings remain below 4,200 triangles each. Trees, flowers, and cottages reuse
the original New Bark kit. Grass, buildings and trees are combined into terrain
meshes; they do not create an entity for each decorative element.

## Verified slice and placement contract

Scenery, character appearance, and camera finishing share the single
`new_bark_models::supports_map` scope: New Bark, Route 29, Cherrygrove, Route 30,
Route 31, and Violet. Selection requires the named map, a complete source
metatile drawing with correct subtile phases, and native doorway art. It never
uses collision as an art classifier. Cherrygrove and Violet keep distinct Mart
and Center identities. Route 30's two cottages retain their own source plots.

Traditional Violet buildings start below the original two source rows of tree
crowns. Those crown rows remain outside each building's claimed plot. Sprout
Tower is its full three-block-row drawing with those crown rows excluded;
the native platform and stairs below it are not promoted to a second house.
That separately verified platform is a flush timber landing with low outer
rails, an unobstructed native stair opening, and its original zero-height
walking datum. Its lower two source rows remain ordinary path.
Route 29's gate spans the connection to Route 46: a complete roof/facade
drawing is preferred when both source rows are in the frame. When the roof is
outside the source grid, only Route 29's native facade plot is fitted. Route 31's different east-facing gate remains on the existing
renderer instead of receiving an incorrect south-facing door.

Both plot edges and the asymmetric doorway column are retained by the same
piecewise horizontal fit as New Bark. Tree art requires its complete native
2×2 or 2×4 drawing. Mixed route blocks use individually verified crown origins,
including Cherrygrove's three-crown shoreline block; lawn and Cut-tree quadrants
remain separate. Signs require the exact 2×2 board art. Flowers require their
actual metatile positions; grass requires its full Johto source-cell identity.
Unknown, altered, clipped, or unrelated drawings retain the existing renderer.
No collision, warp, encounter, movement, script, or footing data is changed.
Native door artwork can be decorative: adding a model does not add a warp.

Tests cover shared map scoping, complete drawing signatures, doorway mutations,
traditional crown rows, all model bounds/normals, source-scoped grass and flowers,
live-tree/grass ownership (including Route29's production profile), stable
terrain coordinates, a flush Sprout landing, and unchanged footing. Low jump
ledge tops and lips receive a tightly source-scoped material finish at their
existing height; their vertices, normals and footing are preserved.

## Foliage performance

Nearby foliage keeps its original mesh. The outer source halo uses separately
authored LODs: tree 248 triangles/716 vertices (original 724/2,041), grass
36 triangles/108 vertices (original 88/264). They retain exactly the original
bounds, palette, ground origin, placement and source-cell ownership. No tree
or grass cell is removed. The six-lobe tree envelope is retained; distant tiny
root details and excess grass blades are reduced. Regenerate with:

```sh
blender -b --python tools/build-johto-foliage-lod.py -- target/johto-foliage-lod
```

Contact shadows use per-source-cell occluder buckets, with a regression proving
bit-identical colors against the previous full-scene scan at cell interiors,
edges and outside the grid. This removes a forest-wide scan per ground vertex.

Native A/B diagnostics: `CRYSTAL_SCENERY_METRICS=1` logs geometry counts, bytes
and terrain build time. Set `CRYSTAL_SCENERY_FULL_DETAIL=1` to disable foliage
LOD for comparison; leave it unset for the default adaptive meshes. Opt-in
Rust tests `benchmark_dense_modeled_terrain_budget` and
`benchmark_dense_forest_contact_shadow_lookup` measure those separate costs.
These geometry/build-time metrics are not a substitute for actual frame-time
measurements on the target renderer and hardware.
