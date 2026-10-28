# Compact Minneapolis and St. Anthony Falls

The later 683-map visual branch includes the verified
[house tile-join correction](../revisions/minneapolis-houses-v01/source-v04/README.md)
in `content-packs/minnesota-house-joins-v04.crystalpack` (SHA-256
`63117daf68439663d7632293f8bded7352bf8e39b96d30edcc14f035871d4d6b`).
It removes interior façade endcaps and upper-storey foundations while retaining
all doors, collision and falls animation. The publication below remains the
earlier compact-city milestone; its pack alone does not contain this correction.

The40×32 exterior keeps40 playable entrances and44 existing interior modules.
St. Anthony Falls sits beside the mill, with a receiving pool, reachable
overlook and road bridge. The river continues into actual neighboring maps at
both city boundaries. This branch builds on the four-route1186a243 pack; it
does not include the separately owned fair, later gym revisions or223-species
regional pack.

## Exact native branch

`content-packs/minnesota-minneapolis-compact-falls-v02.crystalpack`

SHA256 `125ac44497f9fde7a4ebaae3a8326d2d75f626a86d6e838be12ce28d82283fc7`.
Packs and rendered outputs stay ignored. The unchanged parent is
`content-packs/minnesota-route-quality-next-v01.crystalpack`, SHA256
`1186a24371251f741db88cf35a7863145ed5423da4ad3d7c6485c8f242753ecb`.

The canonical inputs are `towns/MnMinneapolis.layout.json` and
`towns/MnMinneapolis.recomposition.contract.json`; exact publication inputs
also remain under `renders/cities/minneapolis-compact-falls-native-v02/inputs/`. Sourcev05,
the rejected uniform-water opening, every prior image and native prototypes
are retained. The native publisher clones the installed city and moves every
original indexed anchor; it does not recreate or relocate interiors.

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1 RUST_MIN_STACK=33554432 \
  cargo run --locked --profile wasm-dev -p crystal-assets \
  --bin recompose_authored_city -- \
  content-packs/minnesota-route-quality-next-v01.crystalpack \
  planning/minnesota/towns/MnMinneapolis.layout.json \
  planning/minnesota/towns/MnMinneapolis.recomposition.contract.json \
  content-packs/minnesota-minneapolis-compact-falls-rebuild.crystalpack
```

The source renderer remains a separate fast loop; native publication is not
needed for every composition revision. See [recomposition contract](../cities/RECOMPOSITION.md).

## Gameplay addresses

Coordinates below are16px walking tiles; metatiles are32px.

| Feature | Actual native address |
|---|---|
| Receiving pool Surf launch | Floor(31,10), face right into Water(32,10) |
| Falls face | x32..35,y6..9; upper Water y5; receiving Water y10 |
| Southern bank Surf launch | Floor(38,60), face left into Water(37,60) |
| North Water seam | City(x32..37,0) ↔ MnR05C00R02(x14..19,15), offset9 |
| South Water seam | City(x32..37,63) ↔ MnR03C00R00(x12..17,0), offset10 |
| North walking seam | City(40/41,0) ↔ R05(22/23,15) |
| South walking seam | City(40/41,63) ↔ R03(20/21,0) |
| East walking seam | City(79,36/37) ↔ R04(0,18/19), offset9 |
| Existing city recovery | Spawn260 →(40,48) |

Native Waterfall permits downward current without the ascent prerequisites.
The north river therefore has a visible six-tile buoy gate at x32..37,y2.
Its three closed metatiles are native31hex at meta(16..18,1), with real Buoy
collision in the top quadrants and Water below. All twelve approach tiles at
x32..37,y1/4 check both CURRENT/PLAIN and CAPITOL/FOG badge flags. Denial prints
an explanation without forced movement. Success restores exact shoreline
blocks58/35/59hex. A silent MAPCALLBACK_TILES repeats those checks after map
setup, submenu return and Continue. It adds no persistent flag or badge.
The eight original road events and all original programs remain exact.

The upper and lower bridges use native Crystal collision; their road decks
interrupt Surf travel. The artwork does not imply swimming underneath them.

## Verification and remaining work

The independent written-pack audit passes for all660 maps, split native
indexes, complete original programs, interiors, catalogues and audio. It checks
the exact appended gate instructions and the sole native waterfall glyph in
unused art storage, and rejects five independent whole-pack corruptions.
All five asset integration cases pass, including stale contracts, blocked
entrances, bad river projections and incomplete physical buoy barriers.

All19 final headless native frames match the source pixels exactly. Root
reviewed the complete source town, closed/open gate views, native decision
sheet and weakest street. This certifies this branch's bounded visual review;
it does not certify every Minnesota route or earned regional progression.

All17 controller cases pass: five Falls/river cases and twelve retained city
activity cases. Both complete six-lane water seams work in both directions;
unqualified Surf is physically denied at the gate, native Waterfall ascent and
downward current execute, and Start/save/Continue retain authorized open-water
footing. Four animation tests verify native cave pixels at thirteen exact tick
boundaries and unchanged legacy city animation. Five asset cases include the
two-cell dismount/remount shelf bypass rejection. The pack rebuild is byte
identical. All-target Clippy with location-tester passes with pre-existing
warnings. Changed leaf formatting passes; global formatting still reports
pre-existing differences across other repository files, including offset-allocator.

Canonical promotion is complete. Exact before/after files and the Root
acceptance receipt are retained under the frozen promotion preflight's
`preview-v05/application-1793117508578967000/`. Every byte outside owned
world/interior JSON fields is preserved. Original creation contracts remain
history; the new recomposition validator handles the current40×32 geometry.
Global plan validation passes20 routes/19 hubs/57 chunks/288 planned maps;
symbolic progression is separate from gameplay. Local test party/badge/location fixtures are explicit setup, followed by real
`VisibleShellController::press` inputs. They do not prove earning those badges.
The earlier ungatedv01 pack is a retained prototype and must not be treated as
the accepted checkpoint implementation.

Written evidence: `renders/cities/minneapolis-compact-falls-native-v02/`, with
the native19-frame capture receipt, independent audit and verified full
activity fixture. No generated pack, render, build cache or private test
executable is a source deliverable to stage.
