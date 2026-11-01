# R07 current grass and sandbank review

The earlier 19-cell grass revision is already installed. This read-only review
changes no terrain and preserves the active selector. The current native terrain
contains 192 TALL_GRASS walking quadrants; the historical 196 claim is superseded
by this direct collision count.

The complete source map, authentic Crystal Route42, and three production 2D
cameras (meadow, sandbank and peninsula) were visually inspected. Lake crossings
and the narrow peninsula retain their identity. Encounter grass occupies the
outward meadow; the sandbank provides the dry return. The published whole-route
proof requires eight meadow grass steps outward and zero on return. The source
proof also holds after Cut and with actor bodies removed. Existing shared grass
outside that band may be crossed both ways. No 3D renders were created.

Original image/source/stdio binding: 704-map story pack
`minnesota-northstar-current-story-v03.crystalpack`, SHA256
`560e49986d14e48c07526e9878eddc69be46d546ad079774f04b10f58e7e5c88`.
Source receipt: `renders/route-grass-v01/current-r07-audit-v01/f2808268fbda/receipt.json`
(relative to planning/minnesota): all1536 metatiles and1,572,864pixels match;
2maps and15outdoor objects, parties, Cut, bridges and pier checks pass.
The three native captures remain in that render directory's parent.
The reached-save production client passed12calls with empty stderr, actual
movement(20,40) to(19,40), Start/Pack, Repel/Ether and GreatBall pocket checks,
and return to walking. Receipt: `.local/route-grass-review/current-r07-704-stdio.receipt.json`.

Current execution binding: `r03_bank_seep_revision`, pack
`minnesota-r03-bank-seep-v02.crystalpack`, SHA256
`c6fdc6bdad16f49b5a0cc8ea0efcf7f3500d2fa61e9494e69492d4b540e6a96a`.
`current-equivalence.json` proves both complete R07 modules and all runtime
assets, roster, audio and tilesets equal the original reviewed story pack.
Only R03Cave1F and its geometry mirrors differ statewide. Earlier images and
stdio receipts retain their original identity; they were not rerun on R03.
The published directional proof and all three adversarial negatives were rerun:
`renders/route-grass-v01/current-r07-audit-v01/R07-r03-directional-b95faa55304db510.json`.
All three native R07 cases pass on this current pack: populated journey, native
Cut denial/open/collection/reset, and Surf/rocks/reciprocal lake seam.
Log: `.local/route-grass-review/current-r07-r03-controller.log`.
The journey wins8trainer battles, spends selected PP, encounters4ordinary wild
battles, collects resources, and verifies sandbank traversal and Save/Continue.
Its54meadow steps are accumulated itinerary steps, not the minimum crossing.
Explicit initial Typhlosion60/Feraligatr60/Mew60, eight badges, location and clock
are fixture setup. This proves behavior/persistence, not earned or balanced play.

A fresh selected-pack scan of all8North Shore chunks, Duluth and GrandMarais
found zero ICE in blocks and scripted changes. R11 uses mn_north_shore.
Receipt: `.local/route-grass-review/current-north-shore-no-ice-r03.json`.

R08-R20, outdoor regional links and final current-story integration remain open.
