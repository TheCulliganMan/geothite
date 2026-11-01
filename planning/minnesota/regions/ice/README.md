# Native outdoor winter art

`winter-art-v01.json` is the frozen recipe contract. Its SHA256 is
`4efd8ef08fc06bfc870144014602aca5ca639b34015bade3dddb5ac769aedd74`.
This is an additive art source, with no terrain or encounter-table edits.

| Private tileset | Unchanged source prefix | Snow FLOOR | Icy TALL_GRASS | Sliding ICE | Snow canopy WALL | Snow tree WALL |
|---|---|---|---|---|---|---|
| `mn_winter_outdoor` | `mn_border_pines` 00–3b | 3c | 3d | 3e | 3f | 40 |
| `mn_winter_north_shore` | `mn_north_shore` 00–82 | 83 | 84 | 85 | 86 | 87 |

The speckled walkable snow comes from native `ice_path:02`, the encounter blades
from native `johto:03`, and the directional ice streaks from native
`ice_path:1f`. Snow canopy/tree reuse `mn_border_pines:05/00`. Each recipe retains
its source collision in all four walking quadrants. Ice uses the existing
native forced-movement behavior; no collision or movement engine is introduced.
The dedicated ice-region author chooses geometry and legal stopping banks.

Only the appended glyphs receive winter foreground colors. Snow uses near-white
and restrained blue shadows; grass uses pale frosted blades with blue-gray
stems; ice retains a clear cyan/white diagonal pattern. Canopy uses snow-white
underlay and dark green foliage. Warm wood, water, shore, signs and all original
blocks retain their original native appearance. The two original 96-glyph
banks are relocated into two 128-glyph banks while logical addresses stay exact,
following the existing native tile-art extension convention. Eight source files
and four typed tileset definitions have exact preimage hashes in the contract.

Grass retains **every native indexed pixel**, including zero holes. Native
TALL_GRASS priority paints nonzero pixels over the player; making all grass
background pixels opaque would hide Chris. The source comparison applies that
same priority algorithm and retains 65 visible player pixels in both native and
icy grass (176 before priority; 111 masked). The original mint outdoor ground
shows between the pale blades. This is an intentional functional distinction
from the snow-white FLOOR background.

Snow/canopy use explicit native index map `[1,2,3,3]`; ice uses `[1,2,1,3]`.
These recolor original pixel positions without rescaling or redrawing shapes.
The three-color winter snow/canopy merge the two darkest source shades. The
snow/ice foreground backgrounds are opaque; neither has grass priority. Native
FLOOR and ICE remain visibly and mechanically different.

Run the read-only fast source preview from the repository root:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/regions/ice/preview_winter_art_v02.py \
  content-packs/minnesota-cohesion-harbor-water-v01.crystalpack \
  planning/minnesota/renders/ice/art-review-new-epoch
```

Other authors may import `derive(pack, specification)` from that helper and use
`SourceArt` with the returned in-memory pack. It never writes a game pack or
exports another source-art catalog. Original prefixes are checked for identical
pixels in all four native outdoor time banks. Every original runtime file and
map is retained. Review images and rejected passes remain under
`renders/ice/art-v01/history/`; the final source inputs are snapshotted under
`renders/ice/art-v01/inputs/`.

The material study is a deliberately small tile demonstration, **not a region
map or gameplay proof**. Review the complete Rainy River region and substantial Border Pines/North Shore
winter sections separately. The earlier tiny overlay proposal is rejected history. Actual publication and production GPU/controller tests remain the
publisher's responsibility. Asset code is in
`crates/crystal-assets/src/winter_outdoor_art.rs`, with focused child tests and
`bin/build_winter_outdoor_art.rs BASE SPEC NEW_OUTPUT`. The CLI requires a fresh
output and verifies full native written readback. No publication was run by the
art worker.

The corrected native publisher guards the actual224-entry source palette map,
then extends only the private target to256. New glyphs occupy E0–F5; all old224
palette entries remain exact. The former v01 helper remains archived because
its allocation reused unused old palette addresses. The corrected address mirror
is `preview_winter_art_v02.py`, with evidence in `renders/ice/art-v02-source224/`.
All final swatch/material/priority images match v01 pixel-for-pixel. See
[mixed-edges.md](mixed-edges.md) for the separate append needed by substantial
winter sections.

The initial winter art was successfully published as
`content-packs/minnesota-winter-art-v01.crystalpack`, file SHA256
`87660aad1dea2a811dbb070c1a592e3ab46246edd05852aa4033d4ec44c6c536`,
content identity `f05f841d747753e4093560b1d8a14b77b9746a3ebe4a2dd589299d9270e22815`.
All three asset tests passed in `.local/winter-art-assets-v02.log`. This proves
art publication and preservation; new terrain/controller and GPU acceptance
remain separate. The preview command uses the exact pre-art3d1c source parent
and a new output directory. Do not derive an already-installed alias twice.
