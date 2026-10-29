# Grand Marais on Superior — source proposal v01

The complete current town is preserved and given a direct southern coastal
arrival beside a broad Lake Superior water seam. The west exit becomes the
inland Gunflint/Ely departure. This is a **source proposal**; native publication,
production input, GPU acceptance and inland winter authoring remain pending.

The actual parent is `content-packs/minnesota-superior-warm-coast-v01.crystalpack`,
SHA `ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8`,
identity `5009f2898563ecf699d7036b19d988ef5f4d43a8b39cbd48f8616cc9c45eca32`.
The whole native town preimage is
`377210a771f41efaa9c5bd18a9035462a61a14da7e34e0967d79421b5e6c93c1`.
The edit contract SHA is
`994f327b0874a7bd346139d2274cfbbfc01e3248521d96308f552310bea93ee6`.

## Geography and composition

The 20×19 town moves to proposed world origin `[202,-419]`, directly north of
R11's existing origin `[190,-400]`. The shared geometry owner is `river_native`;
its placement-v01 receipt checks the exterior graph, cycles and overlap.

| Seam | Grand Marais core tiles | Neighbor core tiles |
| --- | --- | --- |
| Direct coastal Foot | `(16/17,37)` south | R11C00R00 `(40/41,0)` north |
| Same-Superior Surf | `(24..37,37)` south | R11C00R00 `(48..61,0)` north |
| Inland Foot | `(0,8/9)` west | relocated Cedar `(63,8/9)` east |

Town south/R11 north offsets are `-12/+12`; west/Cedar east offsets remain 0/0.
Seven explicit R11 north reef openings are required: meta `(24,0)` `30→58`,
`(25..30,0)` `31→35`. Native before tiles are BUOY, so connection attributes
alone would leave the lake blocked. The joined proposal includes exactly these
counterpart patches, supplied by their owner. The remaining north reefs stay.

The town changes 41 metatiles: its lighthouse headland continues as a fenced
stone quay at meta 8, rows 15..18; seven south water blocks open the lake seam;
unmatched west/east water gains visible native reefs; the obsolete east Foot
port closes with ordinary pines. Six original facades/doors and their actual
rooms remain exact. The quay's two walking lanes retain a clear route to the
lighthouse and all town services. The town itself remains green and coastal;
the substantial inland snowline belongs to the separate Gunflint/Ely authoring.

## Gate and service contract

Current East officers are already hidden on successful passage and free inward
entry. Rotate that existing mechanism south: officers `(16/17,35)`, outgoing
land events at 34 and incoming events at 36. Four facing/movement arguments rotate
RIGHT→DOWN or LEFT→UP. Badge/signal conditions, denial text and existing flags
remain. The closed-state officers intentionally block both walking lanes;
the passing state hides them. Inward hides them, scripts two steps 36→35→34,
then restores them behind the player if the prerequisites remain unmet.

A two-officer land checkpoint cannot gate the broad Surf seam by itself.
Directional water wrappers reuse those same native programs: outgoing events
cover all 34 WATER lanes at row35; incoming events at 36 act only when facingUP.
Denial35→34 and inward36→35→34 remain WATER throughout. The scripted incoming
crossing skips the outgoing event, as the existing inward mechanism does.
There is no bank launch below the checkpoint, and the other lake edges are
BUOY. Actual Surf-mode/control behavior still requires the planned native tests.

All six room modules—including nurse, shop, field-station program and optional
local lamp quest—are preserved by complete native hashes. The local lamp flag
remains separate from the Duluth signal required by the existing road gates.
One existing sign at `(12,14)` points south to North Shore and west to Gunflint
and inland Ely snow. No seventh gatehouse or alternate shore-story dataset is
imported. That 544-map successor was studied for its useful native reef art;
its room/recovery/story lineage differs from this actual 683-map parent.

## Evidence and implementation handoff

The matching render epoch is
`planning/minnesota/renders/north-shore-ely-geography-v01/grand-marais/proposal-v04/`.
I inspected the whole town before/closed/open, joined coast, all town player
views, both actual Foot/Surf seam source cameras, and the four-bank art sheet.
`geometry-proof.json` models closed/open bodies and the planned directional gate
edges: no closed outbound Foot/Surf escape; open ports reachable; free inward
care and all six ordinary Foot door approaches. This is source geometry and
planned event semantics, not production execution.

Fifteen appended metatiles use exact old Johto quadrants; no new glyph or source
pixel is introduced. Ten retain the reviewed shore primitives, four form quay
rails/caps, and one closes the preserved northwest harbor corner. All 524,288
old-block pixels and 61,440 new donor-quadrant pixels compare exactly across
morn/day/nite/dark. All old glyph/palette resources and native behaviors must
remain exact; inherit Johto animation and field-action behavior for the alias.
There are no Cut quadrants in the placed town.

The dossier/layout/connection candidates and `source-preimages.json` are detached
from canonical source and world data. The full layout compiles to the exact
candidate array, starting from the current matching native harbor source.
Use the narrow native contract for owner-reviewed integration. The existing
seven-room planning lineage must not be transplanted into this six-room town.
Only the integration owner may apply a guarded canonical/world port update.

`controller-input-plan.json` covers missing-badge and missing-signal refusal,
both lane passes, free inbound care, all 34 water denial lanes, Surf-mode inward
and Continue, boundary attempts, six room/service returns and inland departure.
No input courses, pack build or Rust changes were performed by this worker.

The complete source/proof/render loop executed in 12.606 seconds, with no Cargo:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/revisions/north-shore-ely-geography-v01/grand-marais/propose_coast.py \
  proposal-v05
```

That command uses a new, unused render epoch; all originals and earlier attempts
remain. The latest successful epoch is v04; the v05 command is the next rerun,
not an already-executed result. Native publication and the wider map goal remain
open.
