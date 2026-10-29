# Paid Boundary Waters travel review

The main pack's paid entrance remains closed. The current unselected review is
`content-packs/minnesota-safari-runtime-review-itasca-v06.crystalpack`, SHA256
`cf799e822151f62d0078936bf317dd39403a2f79b93debc37ed7221ef8061bc5`.
It is a direct overlay of the accepted Lake Itasca parent at SHA256
`89c70e8f5df7c53c9d74bbf9c5197ae22c6580cb4c0d193538f24e439fc9af22`.
All 730 maps retain exact terrain, actors, events and executable commands.
The only data changes are Safari configuration and seven related native
closure-text bodies with their existing split mirrors; 727 whole maps, all
other data, runtime art and compiled audio remain exact.

The published terrain has six exterior chunks, each at most 64×48 walking
tiles, arranged as a 128×112 walking-tile preserve. The bottom row is clipped
to 16 tiles high. Coordinates below are global walking tiles. Convert to local
coordinates using the exact `origin_core` values in [zone.json](zone.json).

## Native input itinerary

Start beside the real admission ranger in the free Boundary Waters hub at
`(30,10)`, face north, read the offer and confirm. The explicit local test
fixture supplies a previously earned regional GILLPUP with Surf, its coherent
badge representations, money and deterministic divider samples before the
first input. Admission itself, travel, encounters, commands and Continue use
production input. These fixtures do not establish earned statewide access.

| Stop | Global tile | Native behavior to prove |
| --- | --- | --- |
| Gate-court arrival | `(15,105)` | Fee charged once; 30 balls and 500 steps; permit opens actual guard bodies |
| Reed portage west bank | `(3,90)` | Walk through actual north chunk connection, not a local placement |
| Reed west lake | `(3,89)` | Native Surf script launches; turning/text/animation cost no extra steps |
| Reed east bank | `(13,92)` | Dismount, cross encounter grass, retain balls when using Run |
| Reed Pocket Lake | `(14,92)` | Relaunch into the separate pocket lake; return by the grassy portage |
| Main lake launch | `(3,89)` | Relaunch from the original western bank for the connected-lake paddle |
| Reciprocal water seam | `(63,75)` ↔ `(64,75)` | Cross the real west/east chunk connection in Surf mode, charging once each way |
| Island west landing | `(56,58)` | Paddle from `(55,58)` and dismount; cross real water seams |
| Island east landing | `(85,61)` | Traverse grassy portage and its east/west walking seam |
| Dorothy's cabin | outside `(71,56)` | Enter active door `(71,55)` and arrive on indoor passive FLOOR `(5,6)` |
| Cabin return | `(71,56)` | Actual carpet exit returns safely; permit/budgets survive indoor Continue |
| Second lake | `(86,61)` | Relaunch after cabin visit; save/Continue retains Surf mode and permit |
| Palisade south landing | `(36,67)` from water `(36,68)` | Dismount and cross grass toward the cliff |
| Palisade downhill hop | `(35,58)` → `(35,60)` | One successful travel debit; blocked intermediate `(35,59)` is skipped |
| Palisade uphill return | `(35,60)` → `(35,57)` | Up is blocked; ordinary 15-step return crosses three grass tiles |
| Palisade north landing | `(36,42)` → water `(36,41)` | Traverse the grassy shoulder, relaunch and retain Surf through Continue |
| West water barrier | water `(1,92)` beside buoy `(0,92)` | Repeated west input stays in bounds and costs no travel steps |
| Paid walking exit | court local `(14,15)` → Pine Entry `(16,0)` | Actual connection ends permit at the chosen destination, retaining party and fee history |
| Unpaid court return | Pine Entry `(16,0)` → court local `(14,15)` | Both guard bodies return; either approach remains blocked |

Wildlife may interrupt the grass and paddling portions. Run must release each
real encounter through the four-command capture screen without consuming a
ball, travel step or partner PP. No ordinary combat or party switch substitutes
for the Safari encounter. Saved checkpoints must be reached through this path;
never repair the itinerary by rewriting position, mode, permit or counters.

## Budget and boundary checks

A separate expedition walks 500 actual adjacent FLOOR steps in the entry court.
The 500th committed movement must exhaust the budget once, retain the real
last position, suppress a new encounter and present the return owner. Save and
Continue this state, prove arrows cannot keep walking, acknowledge, then verify
the actual free-hub return `(30,10)` in ordinary walking mode. The original fee
remains the only debit. Continue again after return; the permit must stay absent.

These journeys supplement the existing full-body closed-gate, full-perimeter
water-barrier and terrain tests. They do not replace the remaining ledges,
other portages, outward-field-move, final-ball, canoe and animation/audio checks.
The free Pine Entry → court walking connection is now part of the current
terrain. Older whole-map verification assumes a sealed outer perimeter and
therefore rejects this newer connection; do not delete the legitimate free
entrance to satisfy that historical verifier. The current preview audits all
476 perimeter tiles: 474 WALL/BUOY tiles and exactly two paired FLOOR exits to
Pine Entry, with no outer water openings. Static actor-body floods preserve the
free court/approach, deny lake access without admission and reach the lake and
cabin approach with the two permit guards removed. These static checks do not
replace native controller travel. The current actual-map composite is retained
under `renders/boundary-safari-runtime-v09/itasca-whole-v01/`. The Itasca rebase
preserves all seven configured maps exactly against the Marais-parent review.
See [LEDGES-EXIT.md](LEDGES-EXIT.md) for the detailed native input contracts.

## Review publication

The executed Rust publisher is `pack_safari_expedition`. Build with the pinned
toolchain and default two-job configuration:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo build --locked -p crystal-assets \
  --bin pack_safari_expedition
```

It takes `BASE BASE_SHA256 ZONE ZONE_SHA256 NEW_OUTPUT`; it verifies both hashes,
rejects an existing output and an already configured parent, derives a new
identity, writes outside tracked source, and verifies strict readback. The zone
source SHA256 for this publication is
`972c39632f0248e58ed98639fde682a81baed6cf81950d70b11b62dae71bbda7`.
Replays must use a new ignored output path. Before future publication, recheck
the live world selector; this pinned review is not permission to roll back a
newer accepted map revision.

Execution results and remaining gaps belong in [VERIFICATION.md](VERIFICATION.md).

## Fast actual-map preview

The read-only renderer uses existing native art helpers and the actual six
compiled maps. It accepts the legitimate free entrance, verifies reciprocal
connections and preserves full-resolution terrain/actor originals plus a
nearest-neighbor preview in a new directory every run. Its first measured
render pass reported 6.72 seconds; no Rust rebuild is required for a preview.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_safari_runtime.py
```

The default resolves and hashes the live selected pack. To preview this
unselected Safari review, add
`--pack content-packs/minnesota-safari-runtime-review-itasca-v06.crystalpack`.
The printed report identifies which pack was used and whether it was selected.
It timestamps that selector observation and records the observed revision;
later accepted packs do not retroactively change an archived preview's status.
An explicit `--output` must name a fresh directory; omitting it creates a unique
version automatically. This static source-art preview does not simulate actor
movement, time/weather, a paid permit or gameplay. Native input/captures remain
separate evidence. Its whole-preserve pixels match the earlier actual-pack
composite exactly.
