# R12 Gunflint grass pass

Candidate based on selected progression_rewards_revision, pack SHA256
27a71e6259fab624324a990957a059b009552ddfa39720fa9074db813aeadb42.
The selected pack retains the accepted warm North Shore. R12 is the existing
inland winter route; its existing ICE and Cut cells remain byte-identical.

## Deliberate terrain

18 explicit native metatiles change from snow FLOOR 3c to frosted TALL_GRASS 3d.
Grass grows from 16 to 88 gameplay tiles across the west entrance, southern lake
bank, cave-bank discovery, ranger approach and camp passage. The ordinary
endpoint journey requires at least ten grass steps each direction; the west
entrance accounts for six. Actor bodies, door/return positions and existing
field-move gates are protected. No new ledge is forced into a palette whose
available forest ledges have green ground. Existing regional topology remains.

Authentic Crystal Route30 was inspected alongside the complete 1024x1024 source
route and five player-scale views. Source renders are retained under
renders/route-grass-v01/current-r12-v01/source-candidate-v01/. All new reviews
are 2D. Whole-route review shows habitat on useful paths, with clear entrances
and distinct lake, cave, shelter and camp destinations.

## Verification in progress

Source-proof.json records the exact 18 cells, minimum grass costs and unchanged
ICE/Cut blocks. parent.json binds the current reward overlay and proves its R12
modules/art identical to the source preview parent.

The publisher exposed an overbroad edge-actor rejection and ordinary/Cut flood
checks that ignored straight native ice crossings. The validator now retains
same-chunk reachable approaches for talk and traverses straight ice crossings
to land without allowing turns on transient ice. Native wall-stop course tests
remain necessary. A negative cross-seam-only actor fixture uses native 08 tree
collision and must still be rejected.

Two review camera contracts were stale: LAKE_WEST_BANK stood on an NPC, and
FROST_RIM stood on transient ice. Their positions move to adjacent reachable
footing; viewpoint-contract-corrections.json records both. No native actor or
terrain is moved for these corrections.

Native pack publication, final preservation audit, controller/Continue/stdio,
actual native 2D views and affected Rust checks are pending. This candidate is
not selected. The statewide R13-R20 and remaining outdoor links remain in scope.

## Published candidate evidence

Rust publisher v05 passed strict preflight, terrain geometry and written-pack
verification. Candidate SHA256:
8c9a01edf629270392e43ae8761fd0ea78d57035b0cb4b097bf9eebc3fc38717.
The preservation audit passes727checks and4 deliberate mutation negatives:
`renders/route-grass-v01/current-r12-v01/preservation-v01/`.
Only two R12 block arrays change; all703other complete maps, native event/actor
modules, reward programs, rooms and source/runtime/audio catalogs are exact.
The cross-seam-only actor fixture is separately rejected without writing a pack.

Source/native terrain comparison passes all1024metatiles and1,048,576pixels.
The child-derived whole route and west/lake/camp actual native 2D captures were
visually inspected. No new non-2D render was made.
Native-terrain-integrity.json confirms all18changes are3c→3d and all existing
inland ICE/Cut metatiles remain exact. Every R11/Duluth/GrandMarais native module
is exact to the accepted parent and contains zeroICE, including script changes.

All377audio programs pass PCM hashes, frame counts and loop ranges.
The fresh production client passes15stdio calls with empty stderr: native
Continue from the actual Milo victory, walking(50,29)→(50,30), Start/Pack,
Items/Balls and return to walking. The original legal Mew60 local fixture is
explicit setup, not earned regional progression. Native Milo opponents remain
BOOMRUFF40andSOLSTORT40; actual PP expenditure, victory flag and exact party/bag
Continue passed.

The first seven-case controller run passed six cases. The extended Cut journey
met a battle after the original single Repel; the test now renews depleted
Repel through the production item menu before further steps. It changes no
native map, trainer, encounter, RNG, save or post-setup state directly. Final
replay after rebuilding this test driver is still pending. First failed logs
are retained. Scoped Rust format passed; workspace format has the existing
offset-allocator baseline. Affected all-target Clippy passed with existing
warnings; a final check follows the driver correction.

## Final acceptance

The corrected seven-case controller replay passes in full. Native input renews
Repel only after its duration expires, consuming actual bag stock. Cut denial,
opening/cache pickup, Continue, recutting, real departure/reentry and no duplicate
reward all pass. All six other cases pass again after the shared fixture helper
change. Timestamped reached saves remain under
`.local/route-grass-review/current-r12-reached-v02/`.

Final affected all-target Clippy passes with existing warnings; all four changed
Rust leaves pass scoped formatting. The existing workspace offset-allocator
format baseline remains. Final logs: `current-r12-native-final.log`,
`current-r12-clippy-final.log` under `.local/route-grass-review/`.

Four actual 2D captures were inspected, including a second camp footing. All
original renders remain. The candidate is now selected as
`world.route_grass_r12_revision`; canonical layout and dossier match the verified
child and actual installed trainer parties/room rows. The reward overlay, park,
all earlier route revisions, warm coast and every other map remain preserved.
Publication identity is343812aa6f506d3480d149f20d606912e68efa5a2f82fe5955e63fd4739fdd38.
R13-R20, remaining outdoor links and final current-story coverage remain open.
