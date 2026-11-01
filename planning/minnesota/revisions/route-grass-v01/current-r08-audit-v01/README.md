# R08 current lake circuit grass review

This route is reviewed on the selected704-map `r03_bank_seep_revision`, pack
`content-packs/minnesota-r03-bank-seep-v02.crystalpack`, SHA256
`c6fdc6bdad16f49b5a0cc8ea0efcf7f3500d2fa61e9494e69492d4b540e6a96a`.
The historical13-cell revision (12grass,1boulder) is already present. No terrain,
story, runtime policy or active selector changes were needed. Current terrain
includes the ancient mire and360TALL_GRASS walking quadrants, superseding the
old284-count pre-mire snapshot.

## Grass and return pacing

The complete current native-source map and three actual production2D cameras
were inspected against the saved authentic Crystal Route42 reference. Birch
bank and Cedar pier each have a four-grass-step climb and zero-grass downhill
hop. Both ledge lanes execute natively, block reverse movement and retain their
ordinary return. This holds before/after catalog Cut and with/without actors.
Cedar is a local shoulder proof: the whole lake circuit deliberately remains
available. Two shorter habitat choices cost2grass steps; the longer dry shore
walk is still usable. Bridge rails, fishing pier, lake outlines and the new
mire paths remain readable. No3D/2.5D images were produced.

Current whole route and mire cameras:
`renders/route-grass-v01/current-r08-audit-v01/mire-current/` (relative to
planning/minnesota). Native captures in its parent:
`r08-birch-r03-v01-2d.png`, `r08-cedar-r03-v01-2d.png`,
`r08-mire-entry-r03-v01-2d.png`. All originals remain archived.

## Source and native evidence

`receipt.json` compares all1920metatiles and1,966,080source/native pixels,
four clipped maps, the15ordinary actors, signs, parties and gate rules.
The bound dossier freezes actual current native trainer names/parties.
The Vergloom actor and three clue signs are separately inventoried exactly;
only those known story extensions are excluded in the in-memory comparison of
the older ordinary-event dossier. No pack bytes are written by these checks.
The current mire reviewer independently checks full native terrain parity,
all clue approaches, sanctuary and both route ports. See its `review.json`.

Published directional proof:
`renders/route-grass-v01/current-r08-audit-v01/R08-directional-9f51595c27f3cf4b.json`;
all3adversarial variants rejected. `directional-four-states.json` records both
4/0loops before/after Cut and with/without actor bodies. The single63→3f Cut
gate opens32walking tiles; Surf cannot land behind it. All1396water tiles are
closed from unconnected outer edges; four reciprocal internal pairs checked.
Three railed crossings and both drops pass source collision checks.

The generic legacy whole-route review rejects a diagonal MAIN_ROAD annotation.
That limitation is recorded in the receipt; it was not hidden by editing
annotations or terrain. The current mire-specific renderer proves actual
geometry/approaches, and native journey tests prove real traversal separately.

All4native R08 cases pass on this exact pack: populated two-trainer journey,
shore/meadow alternatives, both banks/two lanes with reverse denial and return,
and denied/allowed native Cut, sealed cache collection and reload/reset.
Logs: `.local/route-grass-review/current-r08-r03-controller.log` and
`current-r08-r03-reached.log`. The populated journey spends selected PP,
collects three ordinary resources, encounters wildlife and verifies native
Save/Continue. Explicit source-created Mew60/Ampharos54, initial badges, position
and divider are setup; this does not prove earned access, balanced play or all
eight trainer victories. No new interior or healing service is invented.

The reached save passes15production stdio calls: native movement(40,6)→(39,6),
Start/Pack, collected Repel/Ether, Balls pocket and return to walking. Stderr is
empty. Receipt `.local/route-grass-review/current-r08-r03-stdio.receipt.json`.
All377audio programs pass PCM hashes/framecounts/loop ranges on this exact pack:
`.local/route-grass-review/current-r08-r03-audio.log`.
North Shore/Duluth/GrandMarais scan on the same selected SHA finds zero ICE,
including scripted changes; R11 retains mn_north_shore. Receipt:
`.local/route-grass-review/current-north-shore-no-ice-r03.json`.

R09-R20, outdoor regional links and final current-story integration coverage
remain open. This audit preserves the selected full-story/cave lineage.
