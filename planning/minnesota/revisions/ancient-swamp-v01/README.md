# Ancient Mire — Northern Lake Circuit

R08's former open northern lake is now an old peat basin with branching
blackwater, raised moss tongues, reedbeds, ancient stump remains and an enclosed
sanctuary. Its optional path enters from the existing Long Lake saddle. Main
Leech Lake–Bemidji travel needs neither Surf nor the legendary encounter.

Vergloom (`MN_F048`, `MN_099`) is the existing Dark/Grass forest cryptid. The
separate `northstar_ancient_swamp` event overlay owns its optional clue sequence
and encounter; this terrain publication alone does not deliver that event.
Pepivern and Kettlurk retain their established locations and story roles.

## Source and publication

`MN_R08.before.layout.json` describes the selected compact R10 parent's exact
installed R08, including its real Bemidji dogleg and closed unused edge cells.
`MN_R08.layout.json` is the authored revision, also promoted to canonical
`routes/MN_R08.layout.json`; `MN_R08.json` records current native actors,
Minnesota teams/encounters and the fixed clue/creature approach coordinates.
Compatible earlier grass improvements are retained. The route remains40×48
metatiles, with its four existing maps and all connections/events unchanged.

The existing Rust terrain publisher was executed successfully:

```sh
target/fair-maze/wasm-dev/revise_authored_route \
  content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  planning/minnesota/revisions/ancient-swamp-v01/MN_R08.before.layout.json \
  planning/minnesota/revisions/ancient-swamp-v01/MN_R08.layout.json \
  planning/minnesota/revisions/ancient-swamp-v01/MN_R08.json \
  content-packs/minnesota-ancient-mire-terrain-v03.crystalpack
```

The same operation can compose onto a newer pack only while the exact R08
preimage and unchanged native actor/team/room contracts still match. Do not
reinstall an old regional source pack to apply this revision. Generated packs
and renders remain ignored; previous render versions are retained.

## Visual and collision review

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_ancient_mire.py \
  --pack content-packs/minnesota-ancient-mire-terrain-v03.crystalpack \
  --output planning/minnesota/renders/ancient-swamp-v01/published-v03
```

This executed review confirms source/native block parity, all three clue
approaches and the sanctuary reachable on foot, preserved grass alternatives,
two native ledges, three railed crossings, and the single real Cut quadrant.
The original island remains inaccessible before Cut, including by Surf; after
Cut its32 walking tiles, trainer and pickup become reachable. Clue targets
remain visibly solid terrain beside dry adjacent interaction positions.

The first native sanctuary capture exposed too much open pale shore. The second
pass attached an irregular northern canopy, enclosed the grove and broke the
eastern reed rectangle into moss fingers. Whole-route originals, every source
pass, camera crops and native2D captures are retained beneath
`renders/ancient-swamp-v01/`. Block61 depicts old stump/root remains.

The first two terrain versions passed exact whole-pack residual comparison outside R08 terrain,
encoded block payloads and derived identity. Terrain v02 changes164metatiles;
both versions pass all377 audio PCM/frame/loop checks. Two existing production-controller regressions passed the shoreline
walk and grass-climb/dry-hop paths. The longer journey reached its first trainer
battle but failed its obsolete Johto expected-party assertion; the source test
now expects the actual Minnesota teams. The v02 native Cut regression passed denial, opening, collection, native reload
and Continue behavior. Final v02 verification also passed all377 audio programs.
The fresh full journey and Vergloom event results remain separate until run.

This is a bounded terrain publication, not proof of earned campaign progression
or completion of the other regional map owners' changes.

The final v03 marker correction prevents contextual Headbutt from owning the
first clue interaction: a visible petrified-timber stone is WALL, beside old
cedar remains. All three inspection targets are now WALL beside reachable
dry approaches. The refreshed three-case Northern Lakes controller suite passes
on v03, including both real selected Surf PP decreases against the native
Minnesota trainers, resource collection and Continue. The optional encounter
overlay remains independently verified by its event owner before final delivery.
