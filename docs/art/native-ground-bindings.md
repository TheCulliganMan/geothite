# Native ground bindings

Three maps contain complete existing object art but no sample of the catalog's
preferred lawn. The renderer reserves these drawings before legacy live-profile
masking and appends the existing cached sculpture onto proven native ground.
This is a source-binding correction; it adds no model, texture or generic prop.

| Map | Complete drawing | Native backing | Base-map source cells |
| --- | --- | --- | ---: |
| Route10South | Two lower rock quadrants in Kanto block `61` | Kanto block `31`, tile `39` | 8 |
| Route19 | All four rock quadrants in Kanto block `13` | Native water block `43`, tile `14` | 32 |
| RuinsOfAlphOutside | Complete Johto block `03`, sixteen tile `04` cells | Johto block `01`, tile `06` | 48 |

Rock drawings retain `[2a,2b;3a,3b]`, native phase, original land-rock fit and
materials. Grass retains the existing Johto folded-blade mesh and material. The
backing uses the selected live source UV and current palette. Land and grass stay
at zero height; Route19 uses the native water datum and water finish. Native
review caught and removed paving squares beneath its two offshore rock blocks.
The same land-rock sculpture, source ownership and footing remain unchanged.
The rock inset and grass footprint remain those of the existing adapters. Source
cells, collision, encounters, warp data and actor footing are never rewritten.

Both object and backing samples must belong to this map's native bounds using
the viewport's grid origin. The same numbered tile from a foreign atlas or even
a same-atlas connected-map halo is rejected. A normal native lawn sample keeps
the original adapter in charge. Unlisted maps and metatiles remain unchanged.
Cropped, rephased, mixed or edited drawings retain their prior renderer. Existing
ownership and source-matching custom profiles reject the whole object, including
partial custom profiles whose own ground is unavailable. Customized backing
cannot supply the native floor.

The focused tests cover all seven native source phases, padded/scrolled grids,
all single-cell source identity changes, absent or foreign ground, same-atlas
halo ground, crop rejection, custom ownership, original lawn priority,
unchanged unrelated maps, unchanged footing and sources, underlay UVs, duplicate
prevention and exact equality with the existing rock mesh/material at two tile
scales. Targeted native views now verify Route10South, both Route19 groups and the
RuinsOfAlphOutside grass strip. The Route19 correction also rejects wrong maps,
unclaimed/changed rock cells, foreign donors and altered water heights; native
rock geometry and actor footing remain unchanged. Source coverage alone is not
every-camera or gameplay signoff.
