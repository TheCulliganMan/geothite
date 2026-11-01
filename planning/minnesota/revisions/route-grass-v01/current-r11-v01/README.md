# R11 warm North Shore grass candidate

Fully verified705-map candidate, awaiting the coordinated selector handoff. The active game remains R10. R12-R20, outdoor regional links and final
current-story coverage remain open. North Shore must remain ice-free.

Parent: `content-packs/minnesota-current-grass-r10-v01.crystalpack`, SHA256
`f9e3153964a966d610e1ec8c9ecfb45e980e80eab8ea48e336e061863747acd0`.
Child: `content-packs/minnesota-current-grass-r11-v01.crystalpack`, SHA256
`2296cfb81cf6d17e1f02ca3477b7ae3194a33d29e1d4c965a8eb2acff81a3303`.
Identity: `b9df04a73c42d83fe95569200855edd158c3bf8fdc8a17251805185e0350e493`.

## Terrain and 2D review

Eight native grass metatiles join the Birch switchback, Stoney trail neck and
Lester terrace to useful walking. One attached boulder closes a cliff-foot
bypass. Grass increases60→68metatiles,240→272walking quadrants. The existing
three ledges, all stairs, five river catchments,15cascade blocks, buildings,
shoreline and compact four-metatile northern headland remain intact.

Birch's named climb requires3grass steps and0on return; its two lanes require
4and3uphill,0downhill. Lester's named climb requires4and0return; its six lanes
require6,6,5,4,3,2uphill and0downhill. Both proofs hold without actor bodies.
The Stoney neck requires4grass steps in either direction; it is shared route
habitat, not falsely described as a one-way patch. No Cut state is invented.
Published proof and all4adversarial directional variants pass:
`renders/route-grass-v01/current-r11-v01/R11-published-directional-cad621a831eb11f6.json`
(relative to planning/minnesota).

The complete eight-map source footprint and three actual production2D captures
were inspected: Birch, Stoney and Lester. Warm native grass, stone ledges,
stairs and dry landings remain readable. No3D/2.5D capture was created.
Renders: `renders/route-grass-v01/current-r11-v01/source-review/`; actual views
in its parent are `r11-birch-v01-2d.png`, `r11-stoney-v01-2d.png`,
`r11-lester-v01-2d.png`. The omitted top-right canvas is transparent: no retired
headland terrain is invented to fill the actual compact footprint.

## Exact preservation and source contract

Guarded Rust publication validates warm compact geography, source/native
preimage, physical anchors, revised geometry and strict written readback.
Independent759checks prove only9cells in3R11chunks and their encoded block
mirrors change. All702other maps, all actors, scripts, parties, encounters,
rooms, art, audio, HOME and recovery remain exact, including the accepted park.
All4native-data/art mutation cases reject. Receipt:
`renders/route-grass-v01/current-r11-v01/preservation-v01/proof-d3dc9b201cf5af1b25921e62a877d888a6d313825195ec667d2675494809c42f.json`.

The auditor now reads ragged chunk dimensions from the installed parent and
requires omitted source padding to remain unchanged, entirely WALL. A separate
padding-grass mutation rejects; `ragged-padding-negative.json` records it.
The same auditor still passes R10's independent regression and four mutations:
`renders/route-grass-v01/current-r11-v01/auditor-r10-regression-v01/`.

All4896actual native metatiles and5,013,504terrain pixels match unchanged source
art recipes. `source-current-receipt.json` records actual dimensions/pixels,
21ordinary exterior actors, parties, signs, room anchors and current dialogue.
The comparison's rectangular canvas is in memory only, after every actual
ragged native cell is certified. The existing clinic's two exterior records and
Ryan's passive room arrival are separately certified, then omitted only for
the historical ordinary-event verifier. No pack bytes are written by Python.
The clinic and all three complete rooms remain unchanged by the native audit.
Fresh ICE scan covers all8R11chunks, Duluth and GrandMarais, including scripted
block changes: zeroICE; everyR11chunk retains mn_north_shore.

Several old review annotations did not describe the accepted game. They were
corrected in review contracts, without moving terrain or altering scripts:

- Ryan's exterior passive return points at its actual native room record3.
- The old southern grass box126..132 is the existing clinic facade. The actual
  new Stoney encounter neck138..142 is required habitat with4grass steps.
- The road annotation follows actual reachable floor around that clinic.
- A48turn ordinary-water itinerary stays in the real eight-map footprint,
  replacing coordinates in the retired headland. The compact bank probe is
  actual adjacent WATER/WALL, while the two outer sea probes retain BUOY.
- Lester's stair landing is declared TALL_GRASS; the stair itself remains FLOOR.

`MN_R11.installed-dossier.json` is the successful terrain-publisher input.
`MN_R11.controller-dossier.json` has the corrected compact test itinerary.
`MN_R11.current-event-review.json` freezes three already-rewritten native
Minnesota dialogue fields for source comparison, without changing the game.
`source-static-review.json` passes with13adversarial geometry variants rejected.

## Native evidence so far and pending replay

Already passed on this exact child: all4compact coast cases (both road lanes,
reciprocal town Surf seams/Continue, prerequisite denial on both lanes, and
all northward overhang-reef approaches), native Waterfall cascades, and the new
Stoney grass loop with6ordinary wild encounters across128steps followed by RUN.
All377audio programs pass PCM hashes/framecounts/loop ranges.
Logs under `.local/route-grass-review/`: `current-r11-v01-compact-final.log`,
`current-r11-v01-controller.log`, `current-r11-v01-audio.log`.

Final source-driven walking/ledge/keeper/sea/Continue and trainer victory replays
pass after correcting the driver. Initial failed logs remain retained.
The driver now bounds pathfinding by actual ragged map dimensions, applies the
named return-distance contract to its named lane while proving every other lane
still has a useful shorter hop, and chooses legal effective attacks against the
current Ground/Flying trainer opponent. Both Surf and Thunderbolt PP must be
spent and the current Minnesota roster must be observed; no trainer is changed.
Water battle input now settles native animation frames after every input.

Local Mew80/HM/badge/divider and location setup for travel, and Mew60 for the
trainer case, are explicit test fixtures. These do not establish earned access
or balanced play. Selection will wait for successful native Continue and fresh
production stdio on this exact child. The full statewide goal remains active.

## Completed final driver replay on v01

All9fresh native cases now pass:4full-route expansion cases, actual Minnesota
trainer victory with spent effective Surf and Thunderbolt PP/Continue, and all4
compact-coast cases rerun using the final driver. Logs:
`.local/route-grass-review/current-r11-v01-controller-final.log`,
`current-r11-v01-battle-final.log`, `current-r11-v01-compact-final-v02.log`.

The trainer reached-save passes15production stdio calls with empty stderr:
actual local movement(42,13)→(41,13), Start/Pack, Items and Balls pockets and
return to walking. The bag is not seeded with fabricated resources. Receipt:
`.local/route-grass-review/current-r11-v01-stdio.receipt.json`.
The separate sea reached-save verifies exact party/bag/flags/money/position
Continue after visiting all8chunks and dismounting. Both are retained.

Final scoped Rust formatting, Python compilation and affected-crate all-target
Clippy (`--no-deps -A clippy::never_loop`) pass with existing warnings.
Workspace formatting still reports the existing offset-allocator baseline.
Logs `current-r11-clippy-final.log` and `current-r11-workspace-format.log`.

The story author requested a brief selector hold on R10 while completing a
four-map reward successor. R11 remains a verified candidate until that selector
lease is released or the completed successor is provided. The reward programs
must be preserved; no old parent or snowy North Shore pack may be selected.

The reward author explicitly released the selector hold for this accepted R11
revision and will compose the disjoint four-map reward overlay onto it. No R12
selector publication will occur until that completed reward handoff. Canonical
R11 terrain now matches the child, and current authoring contracts retain the
real compact itinerary, clinic-aware habitat and native return indices.
