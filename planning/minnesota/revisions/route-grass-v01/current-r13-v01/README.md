# R13 League Approach grass pass

Current candidate is an exact terrain-only child of accepted R12:
`content-packs/minnesota-current-grass-r12-v01.crystalpack`, SHA256
8c9a01edf629270392e43ae8761fd0ea78d57035b0cb4b097bf9eebc3fc38717.
R13 remains the existing inland winter route near Ely. North Shore, Duluth and
Grand Marais remain warm, exact to the accepted parent and zero ICE, including
scripted terrain changes. All new visual work is 2D.

## Hand-authored changes

19 explicit native snow FLOOR 83→frosted TALL_GRASS 84 points add habitat across
six useful places: the northern pine entrance, Wind Tor ramp foot, Border Lake
resource bank, Long Lake high bank, southern pine neck and Ely approach.
Actor/sign/entrance/return metatiles are protected. Native ICE, Cut, ledge faces,
rock ramps, banks and route/room topology remain unchanged.

Grass grows from 116 to 192 walking tiles. The ordinary endpoint journey requires
18 grass steps each direction, including six in the newly grassy southern pine
neck. Straight native ice-crossing planning retains the same minimum. Wind Tor
now has at least two grass steps on its nineteen-step ordinary climb, while
its existing southbound hop returns to clear floor with zero grass. These are
measurements of hand-chosen terrain, not a generated route.

Authentic Crystal Route45 was inspected for mountain paths and short ledge
returns; Route30 was inspected during the sequential route work. The complete
1024×2048 source route, meaningful player cameras and three actual native
captures at legal footing were visually inspected. The first native Tor capture
used wall coordinate(43,25), is retained as a rejected camera fixture, and is
superseded by native-v02/wind-tor-valid-footing.png at clear(43,26). All originals
remain. Source/native comparison passes all 2048 metatiles and 2,097,152 pixels.

## Candidate proof

Pack: `content-packs/minnesota-current-grass-r13-v01.crystalpack`
SHA256: 9c1813ad83c35f0cfa0c54bcb422efbd49a107ad12b22293b8fbb5f839a9eccf.
Rust publication and strict read-back pass. Preservation audit passes 732 checks
and 4 deliberate mutation negatives. Only three R13 block arrays change; all 702
other complete maps, native actor/event/script modules, rooms, reward overlay,
park, source/runtime art, audio, HOME and recovery data are exact.
Final proof: `renders/route-grass-v01/current-r13-v01/preservation-v02/`.
Native-terrain-integrity.json records floor-only changes, six preserved native
ICE/Cut/ledge metatiles and exact zero-ice coastal modules. All 377 audio programs
pass PCM hashes, frame counts and loop ranges.

An old source/fixture expected Noctowl dialogue. The installed native program
already says BOOMRUFF; source and fixture now bind that actual page. No NPC
program changes. The corrected source dossier passes a second typed publication
with output byte-identical to this candidate. Both source bindings and proofs
are retained.

## Native verification in progress

Existing keeper healing, Cut denial/removal/cache/reset/Continue and current
cave cameras/stairs/return/pickup/Continue cases pass on this exact child. The
old geometry fixture failed only its retired Noctowl text assertion. The old
battle fixture repeatedly chose Psychic and lost to the current native team.
Neither failure was hidden by changing opponents or terrain.

The fixture now expects BOOMRUFF, keeps the same Mew 60 setup and selects among
source-legal effective attacks through native menus against actual Avrin's
BUOYOSTIC 43/REEDRAFT 43. New cases measure actual grass on the uphill ramp,
exercise dry hop/reverse denial and Continue, and trigger a wild encounter in
the new southern neck, use native RUN and Continue. No post-setup state, terrain,
RNG or position injections are permitted. These are explicit local fixtures,
not earned badges/access or balance claims. Rebuild/replay, final affected
Clippy and production stdio are pending. Candidate is not selected yet.

The complete objective remains R14-R20, remaining outdoor regional links and
final current-story coverage after this route; earlier revisions remain intact.


## Current continuation checkpoint

Final source-bound audit is preservation-v04: 732 checks and four mutation
negatives pass. The installed source encounter table now matches the unchanged
native Minnesota table in all three chunks; all 23 trainer/encounter species
are active original Minnesota species. A final typed revalidation produces a
byte-identical pack. The v03 audit detected an input changing during review and
stopped; that failed log is retained, and v04 ran after source binding finished.

Production stdio passes 15 calls with empty stderr from the actual Avrin win:
Continue, movement(23,24)→(22,24), Start/Pack, Items/Balls and walking return.
The revised five-case suite passed four cases. The populated Tor case met an
equal-level native wild encounter despite Max Repel; the test driver now uses
native RUN and resumes the same planned climb. It preserves actual injuries,
PP and bag state rather than changing wild scaling, party levels or terrain.
The shared RouteGrid fixture has an optional native battle callback; default
callers continue to reject unexpected battles. No game/runtime implementation
changes. The final replay and affected Clippy must run after the live build.

Live build: exec session87599, log
`.local/route-grass-review/current-r13-native-build-v02.log`.
It is compiling the existing test executable, not blocked. Do not restart it
because a status observation times out. Current selected revision is still R12;
R13 must remain unselected until the corrected populated Tor case passes.
Earlier failed test logs and every render, including the rejected wall camera,
remain retained. Final replay should retain timestamped reached saves and avoid
overwriting the first Avrin stdio save.


### Automated continuation handle

Completion workflow is running in exec session18060; its log is
.local/route-grass-review/current-r13-completion-workflow.log. It waits for the
exact existing build62287/session87599 to exit successfully, then runs the five
R13 cases, fresh Cut and seven R12 regressions, affected Clippy, scoped format,
fresh production stdio and the guarded selector update. Do not launch duplicate
test/build/selection commands while this workflow is live. On failure it stops
before selection; inspect the named failing log. Finalizer also verifies every
audit input hash, current active species and the unchanged selected R12 parent.
It does not complete the statewide goal. Owner handoff and updated remaining
route inventory follow successful selection.


### Broader outdoor scope inventory

A checksum-bound snapshot of the selected R12 parent confirms 98 Minnesota
outdoor maps: 53 numbered-route chunks, 31 other ROUTE maps and 14 towns or
basecamps. The read-only inventory is
.local/route-grass-review/outdoor-scope-r12-snapshot-v01.json. The 31 additional
ROUTE maps cover Bemidji Northwoods, six Boundary Safari chunks, Detroit Lakes
approach, Ely Shore Pass, Lake Itasca, Marais Cedar approach, Mille Lacs park,
four Mississippi confluence chunks, seven lower-river/Winona-mouth maps, six
upper-river woods chunks, Preserve Pine Entry and Rainy River Icefields.
Counts do not establish placement quality or acceptance. Towns and water reaches
retain their own purpose; zero grass is not automatically a defect. These links
remain in scope after the sequential numbered routes. No other route terrain
was edited while R13's native verification waits on compilation.


### Updated native regression checkpoint

The original build finished successfully after33m19s. All five R13 route cases
and the fresh Cut case passed, including the populated Tor climb/RUN, dry hop,
Avrin victory and wild pine-neck escape. The broader R12 regression found two
older fixtures still assuming Repel prevents level-matched encounters. No map
or game rule changed. A shared opt-in battle handler now drives native RUN for
R12 populated grid walks/talk approaches while preserving all Cut, Max Ether,
June/no-healing, room return, bag/PP and Continue assertions. Default grid tours
still reject unexpected battles. The tested R13 escape helper moved into the
shared fixture leaf; the route-specific test logic is retained.

Old completion session18060 is terminal/failed and must not be resumed as live.
The current build is session99165/PID86314, log current-r13-native-build-v03.log.
It uses package-level crystal-bevy opt-level0 for test compilation to reduce
memory/optimization cost; it does not change the game content or test scope.
The next completion workflow is complete_current_r13_after_build_v03.py, log
current-r13-completion-workflow-v03.log. It waits for this exact build, discovers
the reported executable, and runs native processes serially because the host
was under substantial swap pressure. It reruns all seven R13 cases, all seven
R12 regressions, affected Clippy, scoped formatting and fresh production stdio.
Selection stays guarded on all passes, exact audit inputs and unchanged parent.
Do not launch duplicate jobs while this updated workflow is live.

Current updated completion workflow handle is exec session1419. Poll this handle
and current-r13-completion-workflow-v03.log; session18060 is an earlier terminal
failure. Current build remains session99165/PID86314. The updated workflow runs
native suites serially, discovers the new opt-level0 executable from Cargo's
terminal output and will not select until every final guard passes.
