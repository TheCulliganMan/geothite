# R10 carrier-bank grass and dry return

Selected as `world.route_grass_r10_revision`, retaining the accepted705-map
Mille Lacs park/current-story parent.
R11-R20, outdoor regional links and final current-story coverage remain open.

Child: `content-packs/minnesota-current-grass-r10-v01.crystalpack`.
SHA256: `f9e3153964a966d610e1ec8c9ecfb45e980e80eab8ea48e336e061863747acd0`.
Identity: `c289e60bd25f637553cdccbeceecb880595a662a0b1f1fc76d857de1bfe7a8e2`.
Parent: `content-packs/minnesota-mille-lacs-park-v03.crystalpack`.
SHA256: `6d5e7a486adc7508c8adf42d43543a02c9a1d279c6521e01e0c4b3a94490ecaa`.
Identity: `b20bc929c3aa8d1067b5cdc14586b6bee346de5fa7ed39820a200160a215bd16`.
Parkv04 is byte-identical to v03; `park-v03-v04-equivalence.json` records it.
No rebase or repeated child publication is required for that filename change.

## Grass placement and 2D review

Seven grass metatiles bring habitat onto the carrier climb, upper cache meadow
and required pine neck. Five attached boulders close actual cliff-foot shortcuts.
Grass increases22→29metatiles,88→116walking tiles. The carrier circuit requires
four grass steps uphill and zero on the return hop; the lower road remains dry.
Its ordinary climb is39steps versus the three-tile hop. This measures the named
carrier circuit, not every intercity itinerary. Both routine bench lanes retain
their actual four-walking-tile width. The upper optional reward costs4grass steps.
No extra ledge, Cut gate or incidental Cut quadrant was invented.

Authentic Crystal Route45, the complete source map and western stair, drop and
widest-bend source cameras were inspected. Three actual production2D views on
this exact child were then inspected: stair/climb, dry ledge landing and weakest
quiet bend. Native grass joins the rocky upper trail; stair footing and the dry
landing stay clear. All views are2D; no3D/2.5D capture was created.

Renders (relative to planning/minnesota):
`renders/route-grass-v01/current-r10-v01/source-review/` retains whole-route,
room, weak cameras and originals. Actual captures in its parent directory are
`r10-stair-v01-2d.png`, `r10-ledge-v01-2d.png`, `r10-quiet-bend-v01-2d.png`.
The original source-preview parent was R09; `parent.json` retains that binding.
`publication-parent.json` proves all complete R10 modules, art, catalogs and
field rules equal that preview parent before applying the grass delta.

## Preservation and source checks

Guarded Rust publication validates exact native preimage, anchors, complete
revised geometry, compact warm coast before/after, and strict written readback.
Independent726checks prove only12cells in MnR10C00R00 and its encoded block
mirror change. All704other maps, including MilleLacs, its R18seam, all story and
caves, plus actors, trainers, encounters, art, audio, HOME and recovery stay exact.
Four preservation mutations are rejected. Receipt:
`renders/route-grass-v01/current-r10-v01/preservation-v01/proof-2b268d6d77d536460ae5fce11d7291c1e825f478920e3f99b40754f17c39ba3d.json`.

All960native metatiles and983,040terrain pixels match source recipes. Direct
installed verification checks3maps,12ordinary exterior objects, current parties,
signs, doors and room geometry. Five existing city-mouth patches are retained
in both inputs: (0,5),(47,10),(47,11),(47,12),(47,13). They are not counted as
new edits. The legacy mouth adapter expects unpatched source, so direct native
verification is used instead of applying those patches twice. The original
128Johto art blocks and appended native right-hand cave stair remain exact.

The published4/0proof holds with and without actor bodies. All4directional
negatives reject dry uphill/cliff-foot bypass, removed reverse wall and grassy
return. R10 has no Cut state. Receipt:
`renders/route-grass-v01/current-r10-v01/R10-published-directional-4bb97d00484da576.json`.
Source checks also prove65outer buoys,509reachable Surf tiles without an
unconnected outer escape, the reciprocal internal water seam,26lookout water
tiles and all resource/sign/door approaches. `source-current-receipt.json`
records this plus zeroICE in blocks/script changes across8R11chunks, Duluth
and GrandMarais. R11 retains mn_north_shore.

## Native input and client proof

Six cases pass on this exact child: full geometry/resources/mine returns,
Calder victory/spent Surf PP/Continue, right stair both directions/side-wall
denial, Surf seam/outer rocks, new-grass encounters/RUN, and complete current
cave cameras/stairs/drops/mouth/Continue. Five actual wild encounters occur in
128new-grass steps. Calder's opponents must equal the actual current runtime
Minnesota roster. Logs `.local/route-grass-review/current-r10-v01-controller.log`
and `current-r10-v01-cave.log`.

Initial local placement, source-created Mew60 with Surf, seven badges and initial
divider/party are explicit setup; subsequent actions use production inputs.
The geometry case has an empty party. These prove behavior and persistence,
not earned regional access or balanced play. No native Minnesota creature
placement is changed by the test fixture.

The geometry reached-save passes13production stdio calls: native movement
(6,10)→(5,10), Start/Pack, collected HyperPotion/MaxRepel and UltraBall pockets,
and return to walking. Stderr is empty. Receipt:
`.local/route-grass-review/current-r10-v01-stdio.receipt.json`.
The populated trainer reached-save independently verifies party/bag/flags/
position Continue. All377audio programs pass PCM hashes/framecounts/loop ranges:
`.local/route-grass-review/current-r10-v01-audio.log`.

One stale fixture assertion was updated from the old Johto team to the exact
current runtime trainer roster; PP, survival, victory and Continue remain strict.
Fresh build, scoped rustfmt and affected all-target Clippy (`--no-deps
-A clippy::never_loop`) pass with existing warnings. Full workspace formatting
still fails at pre-existing offset-allocator differences beginning at lib.rs408.
Logs `current-r10-controller-build.log`, `current-r10-clippy-focused.log` and
`current-r10-workspace-format.log` under `.local/route-grass-review/`.

All earlier source-only notes remain in `source-candidate-review-history.md`.

A separate production-client check also passes on the park parent itself:
15calls on parkv04 plus its native roost-collected reached-save, actual position
(36,29)→(37,29), Start/Pack/collected PPUp, Balls pocket and return to walking,
with empty stderr. `.local/route-grass-review/park-parent-v04-stdio.receipt.json`
retains this original parent binding. It does not replace R10's own13-call
child check or the map owner's parent acceptance decision.

Parent acceptance and selector lease were explicitly released by the map owner.
`publication.json` records the accepted mille_lacs_park_revision parent and exact
same payload used in publication/testing. Its revision record and all earlier
audit records remain in world.json.
