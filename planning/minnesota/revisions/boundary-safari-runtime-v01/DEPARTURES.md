# Leaving the preserve with field moves

The Safari permit must end at the player's actual chosen destination. It must
not redirect Fly or Teleport to basecamp, leave an active permit outside, charge
another fee, spend a ball, or change a partner's HP/status/PP.

The current candidate is unselected:
`content-packs/minnesota-safari-runtime-review-departure-rainy-v15.crystalpack`,
SHA256 `8a3e5db6734898bd0cc9edc420d2adb840bbab84b05ba33cdb3daa08005b9f4b`.
Its exact parent is selected Rainy Icefields
`2bd46f3452af8ea3699a75938862d07e62771fc64dab7096648fa6509f0f03b8`.
Publication and independent preservation pass. The fresh default-profile
controller run passes all three cases in 140.24 seconds: four outward journeys,
the full-party Cancel paths and the complete refusal/cabin/retirement trip.
Four production stdio sessions also pass: 76 successful calls, zero stderr and
unchanged input saves. Native visual review is still in progress.
The selected main pack still has no paid Safari admission.

## Native environment rules

All six exterior chunks are `ROUTE`; Dorothy's cabin is `INDOOR`. There is no
cave/dungeon in this expedition. Preserve those real environment restrictions.

| Input | Outdoors | Cabin |
| --- | --- | --- |
| Fly | Native visited-destination menu, then actual selected town spawn | Refuse |
| Teleport | Actual saved last-clinic town spawn | Refuse |
| Dig | Refuse | Refuse |
| Escape Rope | Refuse, retaining both carried ropes | Refuse, retaining both carried ropes |

Initial setup declares previously earned GILLPUP with Surf and Fog Badge,
LOOBIT with compatible Fly and Storm Badge, DUSTIP with compatible Dig, and
DRIFTILE whose actual level-25 moves include Teleport. Initial last-clinic
memory is `MN_GRAND_MARAIS`; visited Winona enables its native Fly
destination. These are local fixtures, not earned regional progression.
All setup ends before the first ranger input. No movement, RNG, permissions,
budget or source environment is repaired afterward.

## Four actual outward journeys

Each case starts at the real hub ranger `(30,10)`, confirms one 500 fee and
arrives in the permit court at global `(15,105)` with 30 balls and 500 steps.
Walk the actual 27-step path to `(3,90)`. Water cases additionally launch Surf
to `(3,89)` through the native script. Expected steps at the menu are therefore
473 on land and 472 on water.

1. Open Start → Pokémon, select the actual partner and its learned field move.
2. Fly cases select visible `WINONA`; inspect the selected destination's native
   spawn identifier. The first land case also cancels Fly and proves the source,
   permit and resources remain intact before selecting it again.
3. Teleport cases use the declared last-clinic memory. Native clinic entry records
   the outdoor source town, not the clinic's interior constant. Resolve the exact
   current spawn row for `MN_GRAND_MARAIS`; do not hardcode an old city tile.
4. Whenever the prepared native move is still at its source after the ordinary
   input boundary settles, require the same permit/resources and queued native
   destination. Directional input cannot walk during that presentation. Timed
   Teleport may already have settled at this boundary; no pending travel is
   acceptable only at the exact committed destination with the permit cleared.
5. Acknowledge real text and settle native animation. Require that exact map
   and tile in ordinary walking mode, no permit and no pending field travel.
6. Check unchanged party, bag and money, save at the actual arrival and Continue.
   The real destination and absence of a permit must persist.

## One complete refusal/cabin trip

After admission and the ordinary walk to `(3,90)`, try Dig through the actual
party action and Escape Rope through Start → Pack → Items → Use. Each refusal
must show the native notice, own directional input, keep the source position,
permit, step/ball counters, bag and party, then release back to the field.

Launch at `(3,89)`, paddle to `(55,58)`, land `(56,58)`, cross the real island
grass to `(71,56)` and enter the cabin at `(71,55)`. Continue indoors, then try
Dig, Escape Rope, Fly and Teleport. All four must refuse according to `INDOOR`.
Fly opens its destination chooser first; actually select Winona before checking
its departure refusal. Merely opening that menu does not execute the field move.
Use the actual south carpet to return to `(71,56)`, walk back to `(56,58)`,
launch west at `(55,58)` and paddle to `(3,89)`. Land `(3,90)`, walk to
`(10,104)` and retire through the real Reed ranger at `(10,103)`.

The controller cases live in
`crates/crystal-bevy/src/bevy_shell/tests/safari_departure_controller.rs`.
They use `VisibleShellController::press` and presentation settling. Path finding
only plans inputs over the authored terrain. Exact execution epochs, native
captures, stdio results and limitations will be added after running these cases.

## Retained investigation

The first replay proved both Fly journeys, then rejected an invalid initial
last-clinic constant. Native entry into a clinic records its outdoor source town;
Grand Marais uses `MN_GRAND_MARAIS` and its real current spawn, rather than an
invented interior recovery row. No production spawn data was changed to fit the
fixture. The later replay also exposed two production menu defects: a four-partner
roster exhausted the submenu rows, and indoor Fly destination confirmation let
its environment refusal escape as an error. The source now reserves submenu
rows, gives the Fly chooser its proper visible ownership, and handles that
refusal with the normal native notice. The fresh three-case run verifies these
corrections. A later harness failure used only one Down input at the real cabin
carpet; the corrected trip uses its actual two-input exit without changing
position or map state. All failed logs and reached inputs are retained.

A separate full-party case adds active regional VOYAGEAR and DOZEBEAR before
input, admits with six partners and available PC space, then exercises both
the action submenu's Cancel and the roster's seventh Cancel row. The real
selected field action must stay visible, the submenu must scroll, and neither
cancel path may spend steps/balls or mutate the team.

The broader original party-menu regression was also attempted. It stops in its
existing title setup (`intro should be skipped before title-only test setup`),
before reaching a party menu. That failure is retained and is not counted as
party-menu verification. The regional full-party case avoids that unrelated
intro fixture while driving the same production menu/input implementation.

The first stdio timing probe sent only 48 flight frames and stopped before
arrival. The passing probe uses the existing bounded `hold_button` tool to
advance actual native presentation; it does not write position or counters.
The first native UI capture also exposed two QA assumptions: directional menu
input was being checked as overworld movement, and a long held Down scrolled
past the intended partner. The next capture starts with a real legal floor
walk and uses the existing short input pulse for party/Start/Fly menus too.
This changes review-tool timing, not gameplay dispatch or authoritative cursors.

Native source holds were then inspected directly. The six-partner action popup
is readable in `native-v04/full-party-actions.png`. The v05 Fly hold confirms a
real Fly destination cursor and entries, but the LCD still displays the party
popup. This is an unresolved native presentation defect; the successful stdio
projection and four reached destinations do not establish native Fly-screen
acceptance. The cabin refusal screenshot also exhausted its input script
without the requested visible page. Both failed capture epochs remain retained.
These native UI checks are pending while the requested Duluth terrain repair
is addressed; the review pack is not promoted.

Story acceptance is separate. The selected parent still has an unintended
indoor Dorothy duplicate. The independent
[open story issue](../route-grass-v01/current-boundary-safari-v01/open-story-issues-v01.json)
requires its canonical volunteer reconciliation and a new selected-pack cast
review. These movement/menu checks do not accept that cast or open admission.
