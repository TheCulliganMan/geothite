# Palisade and the paid walking exit

The unselected Itasca-parent v06 review adds working permit directions to the
native preserve and retains the connected lakes, grassy portages, cliff paths,
campsites and Dorothy's cabin. The map geometry is unchanged in this revision.
The work closes native traversal and presentation gaps before paid admission
can be selected in the main game.

## Native traversal contract

All outdoor coordinates in this section are global walking tiles unless a map
is named explicitly. The production-controller journey starts beside the
public basecamp ranger. Initial regional Surf partner, badge, money and divider
samples are declared fixtures; payment, movement, wildlife and returns use
actual input after setup.

1. Admit once, walk north out of the court, launch at `(3,89)` and paddle to
   `(36,68)`. Dismount at `(36,67)`, then cross at least three grass tiles to
   the Palisade's upper approach `(35,57)`.
2. Walk to `(35,58)` and hop south to `(35,60)`. The native hop skips blocked
   `(35,59)` and costs one successful travel commit. Upward attempts remain
   blocked and cost no steps. Save and Continue preserve the landing.
3. Return uphill by the ordinary side route: 15 walking steps, including three
   grass tiles. Continue across the grassy shoulder to `(36,42)`, launch at
   `(36,41)`, and preserve Surf/permit state through Continue.
4. Paddle to `(1,92)` beside the actual buoy at `(0,92)`. Four west inputs
   stay in Surf at `(1,92)`, spend no steps and retain all 30 balls.
5. In a separate paid trip, walk to court local `(14,15)` and leave south.
   The real connection arrives in `MnPreservePineEntry` at `(16,0)`, in normal
   walking mode with no Safari permit. The party is preserved and the fee is
   still the only money debit. Continue preserves this outside state.
6. Walk north back into the free court. Approach both guards at local `(12,6)`
   and `(11,6)`; their actual bodies at `(12,5)` and `(11,5)` block northward
   travel. Reentry must not manufacture another permit or charge a second fee.

The two controller cases now exercise these steps against v06. A wildlife
encounter immediately after the downhill landing is valid: the original
checkpoint is retained, then native Run creates a separate field-view save.
No position, counter or permit is rewritten to obtain a camera view.

## Native permit directions

The obsolete closure notice was misleading in an enabled review. Publication
now replaces exactly seven related native text labels and their existing split
mirrors, after checking their old closure anchors and rejecting shared labels.
No actor, event, terrain, executable command table or unrelated text changes.

The guard dialogue is two actual native pages:

| Page | Line 1 | Line 2 |
| --- | --- | --- |
| 1 | Permit required. | Ask the ranger |
| 2 | at basecamp to | start a trip. |

Both pages own input and survive Continue. The first test incorrectly expected
the second page to be present before A; that failure is retained. The corrected
case advances the actual page and verifies it separately. Native image review
then caught a separate real defect: the ordinary script continuation survived
Continue but its frontend page cursor did not. Matching `pending_text_wait`
alone was too weak to establish visible-page persistence.

The production save path now stores a later ordinary dialogue page on a
checkpoint copy, leaving the live state unchanged. Continue validates the
native label, complete resolved text and page/character bounds, restores the
production printer, then consumes that checkpoint metadata. First-page and
non-dialogue saves retain their existing representation. The controller case
now checks the actual second-page text after Continue, live-memory preservation
while saving, and complete dialogue closure without a stale page after another
Continue. The three affected travel/ending cases pass on the corrected source;
the original Johto Mom/New Bark controller regression also passes.

The two check-in signs direct the player to basecamp and explain that leaving ends the trip and
keeps catches. The selected main pack still has closed admission.

## Whole-map boundary audit

The fast actual-map renderer now checks all 476 outer walking tiles. Exactly
474 have native WALL/BUOY collision. The two remaining FLOOR tiles, `(14,111)`
and `(15,111)`, pair with Pine Entry `(16,0)` and `(17,0)` through the real
reciprocal connection. There are no unconnected outer water openings.

The actor-aware static FLOOR/TALL_GRASS/WATER flood reaches 54 free-court tiles,
including the two exit tiles, with both guards present. It cannot reach water
or the lake launch. Removing only the two permit guards opens access to the
lake and cabin approach. This is static connectivity evidence; the paid native
journeys above separately prove movement and input ownership.

Negative checks reject an opened outer water edge, a wet walking aperture, a
moved guard body and a missing Pine Entry connection. Original bytes are
restored in memory after every negative case. Every render is immutable.

## Retained review

- [Whole preserve](../../renders/boundary-safari-runtime-v09/itasca-whole-v01/whole-preview.png)
- [Downhill landing after native Run](../../renders/boundary-safari-runtime-v09/native-v03/paid-palisade-landing-field.png)
- [Grassy uphill return](../../renders/boundary-safari-runtime-v09/native-v02/paid-palisade-grassy-return.png)
- [Buoy boundary](../../renders/boundary-safari-runtime-v09/native-v02/paid-west-buoy-blocked.png)
- [First permit page](../../renders/boundary-safari-runtime-v09/native-v03/paid-gate-permit-directions.png)
- [Second permit page](../../renders/boundary-safari-runtime-v09/native-v03/paid-gate-permit-next-page.png)
- [Second page retained after Continue](../../renders/boundary-safari-runtime-v09/native-v03/paid-gate-permit-next-page-continued.png)

Exact publication hashes and build commands are in [TRAVEL.md](TRAVEL.md).
Remaining acceptance includes other ledges/portages, outward field moves,
final-ball/last-grass-step journeys, canoe prerequisites and complete native
throw/shake/audio presentation. These local fixtures do not prove earned
statewide access or regional balance.
