# Three authored deciduous trees

Adapted from Quaternius's **Stylized Nature MegaKit, Standard/free edition**:
https://quaternius.com/packs/stylizednaturemegakit.html
https://quaternius.itch.io/stylized-nature-megakit

CC0 1.0; the publisher's original notice is preserved in `LICENSE.txt`.

| Runtime model | Original model | Form |
| --- | --- | --- |
| spreading | CommonTree_1 | Full spreading crown, low branching |
| forked | CommonTree_2 | Unequal forked limbs and an offset crown |
| upright | CommonTree_4 | Tall exposed trunk and a lighter high crown |

The Rust `import_deciduous_models` example preserves the artist's trunk, branch
and crown placement. It converts bark painting to vertex colors and replaces
square leaf cards with closed, cupped leaves. These are three distinct source
models, rather than scaled copies of the previous Kanto shrub. Original download
archives and textures remain outside tracked source.

The Rust renderer keeps the three branch architectures in four shared age classes
each. Runtime shaping gives mature crowns more spread, continuous yaw and finer
closed leaf sprays. Whole leafless terminal limbs are trimmed while the complete
support chains beneath foliage remain. Small modeled leafy shoots graft directly
onto exposed living boughs in mature trees; existing leaf coverage limits their
placement and their count is bounded by the authored crown's spray budget.

Stable source coordinates choose the model, age, height, width, yaw and root
offsets; scrolling and neighboring tree changes cannot switch the chosen model.
Root fitting uses the actual root outline. The tree roots and connected
undergrowth retain the original blocked source plots.
Johto pines, Ilex ancient trees, fruit/Cut trees and formal park trees keep their
existing models.

Regenerate from the extracted free Standard pack outside tracked source:

```sh
cargo run --locked -p crystal-voxel-view --example import_deciduous_models -- \
  "$NATURE_SOURCE/glTF" crates/crystal-voxel-view/models/deciduous
```
