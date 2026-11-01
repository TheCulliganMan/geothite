# R07 / R09 / R10 / R20 visual audit — 2026-10-26

The clearest bounded candidate is the empty northwest shoulder of R07's north
elbow. R10's carrier bend also has a quiet, largely uninterrupted ground camera,
but this audit does not yet justify changing its cliff/ramp geometry. R09 and
R20 have specific quieter stretches worth retaining in review rather than
adding a prop quota. None of these findings is final user visual acceptance.

## Inputs and limits

Read the current map-authoring skill and user guidance. Rendered the actual
published native exterior modules in
`content-packs/minnesota-boundary-basecamp-connected.crystalpack`, content hash
`9cb8936dbd1afeeae3abe69e237aeddf12d5c8bccb3a8eb9f8b7ed53bdb45841`.
The assembled native blocks exactly match the frozen layouts: R07 1,536,
R09 2,304, R10 960 and R20 2,048. This identifies the reviewed geometry; a
matching block count or passing test does not establish visual quality.

These are static native source-pixel renders with initial authored bodies,
not new engine captures or an earned player session. Current-map standing
directions/default palettes were used. Interiors, battle timing, live visibility
and regional balance are outside this exterior sight audit. Each whole map,
all four valid-walking contact sheets, the four selected weak raw160 frames,
and the authentic references below were opened in the image viewer.

All outputs are retained under
`renders/route-visual-audit/9cb8936dbd1a/`: original RGB terrain, full-resolution
RGBA scenes, RGB conversions, each raw160×144 crop, enlarged variants and
contact sheets. `save_render` and `render_stage` preserve every intermediate
and previous version. No source, native pack, compiler or runtime was changed.

Camera centers were checked against actual native collision before selecting
walking evidence. Initial exploratory R07 `(18,18)/(21,46)`, R20 `(115,43)` and
reference Route42 `(37,10)` / Route44 `(31,13)` are WALL-centered and are
retained, but excluded from walking comparisons. The corrected contact sheets
are explicitly named `valid-walking`; the corrected R20 frame is `(117,43)`.
Tall-grass centers are labeled separately from clear Floor in those sheets.

## Authentic Johto comparison

The actual pack's [Route26 whole](../renders/route-visual-audit/9cb8936dbd1a/Route26-authentic-native-terrain-original.png),
[Route42 whole](../renders/route-visual-audit/9cb8936dbd1a/Route42-authentic-native-terrain-original.png)
and [Route44 whole](../renders/route-visual-audit/9cb8936dbd1a/Route44-authentic-native-terrain-original.png)
were inspected. These reference renders show terrain, without simulated NPC
poses. Their useful lesson is coherent nearby boundaries and explained space,
not a universal narrow-road rule. Route42 has a broad turnout because a cave,
signs, shore and forest frame it; Route26 uses bank recesses and ledges to
explain turns; Route44 places shore, tree fingers and grass close to travel.

Native Floor-centered reference crops:

- [Route26 `(9,27)`](../renders/route-visual-audit/9cb8936dbd1a/Route26-walking-9-27-original-160.png): the turn and two contrasting bank faces explain direction within one camera.
- [Route42 `(9,9)`](../renders/route-visual-audit/9cb8936dbd1a/Route42-walking-9-9-original-160.png): substantial open ground still contains a visible cave, signs and changing edge silhouettes.
- [Route44 `(45,12)`](../renders/route-visual-audit/9cb8936dbd1a/Route44-walking-45-12-original-160.png): the clear route is bounded by a tree/grass/water shoulder and an irregular stone edge.

## Current route findings

| Route | Whole / walking evidence | Concrete reading |
| --- | --- | --- |
| R07 Leech Lake | [Whole](../renders/route-visual-audit/9cb8936dbd1a/r07-native-whole-original-rgba.png), [walking sheet](../renders/route-visual-audit/9cb8936dbd1a/r07-valid-walking-contact-sheet.png), [weak north elbow `(20,14)`](../renders/route-visual-audit/9cb8936dbd1a/r07-walking-20-14-original-160.png) | The outlet circuit, two protected crossings, north fishing landing and eastern spit give the whole a recognizable lake route. At the north elbow, however, the camera shows an empty white turn between uninterrupted lawn shoulders; neither T08 nor the landing sign is visible. The problem is the shoulder composition, not the width of the actual two-core-tile walking lane. |
| R09 Iron Range West | [Whole](../renders/route-visual-audit/9cb8936dbd1a/r09-native-whole-original-rgba.png), [walking sheet](../renders/route-visual-audit/9cb8936dbd1a/r09-valid-walking-contact-sheet.png), [north bench `(53,19)`](../renders/route-visual-audit/9cb8936dbd1a/r09-walking-53-19-original-160.png), [field medic `(90,53)`](../renders/route-visual-audit/9cb8936dbd1a/r09-walking-90-53-original-160.png) | The quarry basin, raised ore shelf and rail crossing are materially different places. The widest bench still shows the geologist, pine edge and lower cliff in the camera, so it is more explained than an empty shelf. The medic clearing is a repetitive round lawn with flowers, but a real service and close pine boundary explain this short destination. No immediate terrain edit is warranted from these views alone. |
| R10 Iron Range Descent | [Whole](../renders/route-visual-audit/9cb8936dbd1a/r10-native-whole-original-rgba.png), [walking sheet](../renders/route-visual-audit/9cb8936dbd1a/r10-valid-walking-contact-sheet.png), [carrier bend `(17,17)`](../renders/route-visual-audit/9cb8936dbd1a/r10-walking-17-17-original-160.png), [neighboring bend `(17,15)`](../renders/route-visual-audit/9cb8936dbd1a/r10-walking-17-15-original-160.png) | This is the weakest ground treatment in this four-route set: brown carrier-floor repetition occupies most of two adjacent cameras, with no interaction visible. North trees and the lower bank show its edges, but the intervening surface offers little visual direction. The later mine mouth, pine threshold and protected shore recover a stronger identity. Keep the carrier bend flagged; do not scatter boulders across the proven ramp/ledge travel or shrink the entire route in response. |
| R20 Voyageurs | [Whole](../renders/route-visual-audit/9cb8936dbd1a/r20-native-whole-original-rgba.png), [walking sheet](../renders/route-visual-audit/9cb8936dbd1a/r20-valid-walking-contact-sheet.png), [eastern clear lane `(117,43)`](../renders/route-visual-audit/9cb8936dbd1a/r20-walking-117-43-original-160.png), [cliff-foot lane `(69,36)`](../renders/route-visual-audit/9cb8936dbd1a/r20-walking-69-36-original-160.png) | The two outlet crossings, granite recess and opposed cave mouths read distinctly from R07. The east bank still repeats a straight road/grass strip, but `(117,43)` includes the pine finger and trainer-side recess; it is weaker than the cliff-foot composition, not empty enough to justify another generic interruption. The northern rock mass is mostly inaccessible scenery, so its whole-map area should not be mistaken for an empty walking plaza. |

R07's two bridge middles `(35,37)/(35,69)` also remain visually similar:
water, posts and straight pale deck dominate both cameras. Their distinct
two-/four-core-tile deck widths and different shore approaches are visible in
the whole map, but a route-length play review should still judge whether the
two crossing sequences feel repetitive. Their necessary, protected water
crossings are a reason to retain them; adding an object in the lane is not an
answer. The bank and Cut discovery provide different activities elsewhere.

For R20, the cave's 16/18-step relationship should not be advertised as
a dramatic shortcut: the source review measures 16 inside steps and 18 outside.
Its resource/discovery and two honest return mouths are the stronger purpose.
The separate wind-shelter ledge has a short ordinary return; preserve that
local directed choice rather than introducing Cut as another quota.

## One bounded candidate for Root selection — not applied

**R07: extend the existing north pine lobe into exactly two lawn metatiles.**
After the existing `Pine shoulder beside north arrival` layer, propose a native
`05` forest rectangle with exclusive bounds **`[7,5,9,6]`**, replacing only
metatiles **`(7,5)` and `(8,5)`**, currently `02`. This is core footprint
`x=14..17, y=10..11`. It connects to the existing canopy immediately north,
brings a shaped boundary into camera `(20,14)`, and distinguishes the western
T08 recess from the main arrival/landing turn. It adds no prop, reward,
permission gate or new activity claim. The existing two-core-tile road and
east landing/item branch remain where they are.

A read-only in-memory ordinary-walking graph trial loses exactly those eight
replaced lawn tiles and no other reachable tile. Every current resident,
trainer, sign and item retains its prior reachable/unreachable interaction
state; the Ether cache stays sealed. T08 at `(13,13)` retains its east approach
`(14,13)`. **Do not extend the rectangle into metatile row6:** that would close
the trainer pocket's only approach. No hypothetical terrain render or source
edit was made.

If selected, rerun full source fitting, pre/post-Cut walking+Surf checks,
crossing/meadow/ledge contracts and player-scale `(20,14)` / T08 approach views,
then native preservation and production traversal as appropriate. The coarse
trial is not native fitting or gameplay proof. This is a modest composition
candidate, not a claim that two trees resolve all route quality concerns.

Read-only logs: `.local/leech-voyageurs-visual-audit.log`,
`.local/iron-routes-visual-audit.log`, `.local/route-audit-walking-center-check.log`
and `.local/r07-bounded-proposal-readonly.log`. The ignored artifact metadata
records native/source block equality and exact camera centers; generated game
catalogs and extracted source dumps are not deliverables.
