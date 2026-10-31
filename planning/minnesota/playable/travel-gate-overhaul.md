# Travel without repeated faction roadblocks

The user's October 31 correction concerns **Team ICE**, not frozen terrain.
Ordinary regional travel should not repeatedly depend on defeating ICE,
obtaining an officer's permission, or returning with another document. This
revises the travel design within the full story overhaul.

## Installed behavior audited

Read-only inspection covered the selected
`minnesota-iron-crosscut-bends-v01.crystalpack` (SHA256
`a69310b47d2e3583d9fb8378784c919584591d6e260d5f548ae10ea0cd3409ce`)
and the unselected dialogue v27 candidate. Both contain **23 paired road
checkpoints, using 46 officer actors**. Each counted pair has a reachable badge
or event check and coordinate interception. Ordinary wardens who only offer
conversation, conductors and interior guards are excluded from that count.

These road actors are generally public wardens, not ICE trainers. Nevertheless,
their repeated officer appearance and stop/check/push-back behavior make the
world feel controlled by the same faction. Rewording refusals or changing their
uniforms would leave that pacing problem intact.

The selected pack also makes all five Glacier Line conductors require
`EVENT_MN_COURIER_DEFEATED` before offering travel. Its separate destination-visit
checks already prevent riding to an undiscovered stop. That global story lock
makes the courier encounter control unrelated return trips.

This is installed-script evidence, not a new controller traversal or earned
campaign pass. Detailed read-only evidence is retained, ignored, at
`.local/dialogue-overhaul/travel-gate-audit-v02.json`.

## Revised access design

**Open public travel; gate the relevant action or dangerous site.** Keep ICE's
presence concentrated around its headquarters, captives, cargo and controls.
Show coercion and its consequences there. A routine trip between towns should
not require another confrontation with ICE or another official's clearance.

| Installed checkpoint group | Revision target |
| --- | --- |
| Winona, one pair | Preserve the short first-partner safeguard; stage it as personal opening guidance instead of two officers policing a hometown exit. |
| Rochester and Mankato, one pair each | Remove badge inspection from ordinary town departure. Keep actual Gym prerequisites at the Gym and real field-move restrictions at their terrain. |
| Minneapolis east and north, two pairs | Separate the public bridge/road from ICE's headquarters operation. Explain actual Strength/Surf obstacles where the player encounters them; badges must not function as generic road permits. |
| St. Cloud, one pair | Let the north road be a journey the player can choose. Sera's situation supplies a human reason to intervene rather than permission to leave town. |
| Brainerd, three pairs; Leech Lake, one; Itasca west, one | Remove repeated Lake Badge checks around the same regional circuit. Keep shore advice, useful destinations and optional native Surf rewards. |
| Bemidji, three pairs | Make the public reference outing and northern visit accessible. The comparison establishes evidence for the investigation; it should not license ordinary regional travel. |
| Grand Rapids and Duluth, two pairs each | Put report/meter/log requirements on the mine inquiry, depot handover and harbor planning. Public roads and city services should not require those documents. Remove the mirrored checks at both ends of one journey. |
| Grand Marais, two pairs | Keep public coastal travel and the camp approach separate from expedition preparation. Stop repeating the same eight-badge and radio checks at successive exits. |
| Boundary camp, three pairs | Keep public camp care and return routes open. Locate genuine operation hazards at the intake. League entry retains its eight badges and completed resolution, checked at registration rather than at every neighboring road. |
| Glacier Line | Offer return travel between previously visited stations independently of the courier battle. Preserve destination discovery, cancellation and safe arrivals. Rewrite service dialogue to match. |

Do not replace each officer with a differently named NPC, a new permit, an
arbitrary repair errand or a freshly placed Cut tree. Existing cliffs, rivers,
native field abilities and route difficulty can structure exploration where
their actual geometry supports it. New obstacles require a useful place and
playable solution, not an excuse to maintain the old number of gates.

## Implementation and acceptance

This document changes the target design; **the access revision is not yet
published**. The existing progression tables describe installed behavior until
the replacement passes production input tests.

Implement the revision as a guarded Rust overlay on the then-selected descendant,
and share its rules with the original story composition. Preserve numeric event,
trainer, map and reward identities. Do not grant an investigation result when a
road opens. Retain the acknowledged rescue, evidence custody, battle, shutdown,
release and League actions. Existing saves and older earned checkpoints remain
bound to their original packs.

Acceptance must include early and out-of-order visits: useful, truthful NPC
responses; intact Gym prerequisites; no premature rewards, boss access or ending;
and no dead-end instructions. Walk each revised seam both ways, including
diagonal/continuous input, reentry and Continue. Verify all station visit masks
without supplying a courier win. Render ordinary roads and operation entrances
in native 2D so public staff and ICE read as different roles. Complete the earned
campaign and review directions again before promotion. Preserve unrelated terrain,
roster, audio and newer regional services.
