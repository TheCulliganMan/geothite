# Exterior review: three bounded candidates and implemented successor

The three proposals below were subsequently implemented and source-promoted on
final6071. [Matched comparison and executed evidence](../playable/road-bends-and-stairs.md)
record the actual results. The original audit remains below with its original
proposal wording and baseline hashes; it is not a claim that these changes were
already installed at audit time.

Read-only audit of the completed 510-map review pack `content-packs/minnesota-northstar-repel-review.crystalpack`, file SHA256 `31258a1788a986b158d0679e4f3a26c4dd476964311f707a6e09c9640cad48c6`. No terrain, actors, programs, source contracts or native pack were changed. No Cargo or publication ran for this audit. These are proposals, not installed improvements or production-controller acceptance.

Coordinates below explicitly distinguish 32px **metatiles** from 16px **core tiles**. Whole-route and player-size images linked below are existing native-art source composites personally inspected during this audit; they are not new GPU captures. R06 and R16 source block grids exactly match all current native exterior blocks. R10 matches after the five installed walking-seam overrides listed below. A complete current cave atlas is a separate task.

| Route | Extent, metatiles | Current dossier SHA256 | Current layout SHA256 |
|---|---|---|---|
| R06 Pine Country Road | 32×44 | `6cca686118609e4adae077eb062412ca4d3f737edd94243259ef9f13326f6187` | `575b2d6492cf93e5e803301349076bfb1b126edbc745f4c7f1934aabecea3e9d` |
| R16 Red River Prairie | 48×24 | `6750bec82241d5205d34ae06aa704ef18fc0570a308e9151f33cddf0e434aa0a` | `6301787fb6db5d929d39092751d4587eff8f4a406af8a32eaaae7789ded970fd` |
| R10 Iron Range Descent | 48×20 | `0fd088086912e9a13ec64bae47750e856ac01d1faaedad34f80720b89a268ffa` | `2db59b63a1184b5f0975c30f3ff8fec1bebbce2bf6cf18c81895410f3ff94e5a` |

## 1. R06: the northern road needs a useful bend

Inspect [whole R06](../renders/pine-country/whole-route.png) and [BRAINERD_APPROACH, core (43,7)](../renders/pine-country/viewport-brainerd_approach.png). The weak stretch is metatile x19–24, y3–11, inside the northern end of `Camp to Brainerd connected road`, [layout layer](../routes/MN_R06.layout.json:561). Its centerline remains x21 from y0 to y17. The road is only one metatile wide, but its continuous walkable lawn shoulders make the actual passage considerably wider. This camera shows plain paving/lawn and repetitive pine borders; the interesting lake, pier and passing-bay decisions are farther south.

Bounded candidate: keep the Brainerd gate (21,1) and landing (21,3), then hand-shape two attached pine shoulders and an orthogonal road bend inside x19–24, y3–11. Aim for a readable 2–3-core passage with one small passing pocket. A trial centerline could turn east from (21,3) to (22,3), descend to (22,6), turn west to (20,6), descend to (20,9), then rejoin (21,9)→(21,11). This is an actionable trial, not a frozen geometry contract: fit the canopy quadrants and actor approach before accepting it. The winding should produce a decision at player scale rather than replace straight paving with an equally broad winding lawn.

Retain the watcher at core (41,9), north sign at (44,4), north bird habitat metatile [22,12,25,15), and all existing southern road gameplay. Do not add a ledge to this flat approach as a quota. Existing `PINE_LOOKOUT_RETURN` at core x20–23/y30–31, Cut gate (29,32), meaningful southern grass crossing [12,76,22,80), Perry's battle-or-six-grass passing bay, camp recovery cabin and all lake/bridge closures remain exact.

Proof needed: whole route plus the same (43,7) camera and both bend approaches; actor-present two-way travel to the gate; exact endpoint/actor/door data; no incidental Cut canopy or new dry bypass of the existing grass/battle choices. Check actual directional collision, not just the visual road centerline. This is the first recommendation because it can improve a routine stretch with a small terrain-only edit.

## 2. R16: tighten the farm bend and shorten the existing grass-bank lip

Inspect [whole R16](../renders/red-river-prairie/whole-route.png), [WIDEST_FARM_LANE, core (75,30)](../renders/red-river-prairie/viewport-widest_farm_lane.png), and [HAYFIELD_BANK, core (53,15)](../renders/red-river-prairie/viewport-hayfield_bank.png). The east farm bend around metatile (38,15), between the drainage bridge and cottonwood neck, reads as a broad, flat sweep. The northern hayfield also has a 24-core-wide continuous jump face at x32–55/y16. Its extent overwhelms the local choice despite the good golden prairie palette and river/drain identity.

The existing ledge already affects encounters. Native (53,16) is `HOP_DOWN`, (53,17) is `WALL`, and landing (53,18) is `FLOOR`. An actor-present 0/1-cost graph over native `FLOOR`/`TALL_GRASS` requires **six grass steps** from that landing to the farm road at core (56,30); a Floor-only search cannot reach it. This is geometric evidence, not a production encounter test. Do not report the existing bank as a free grass bypass. The saved review records a 59-step ordinary return through the western working gate.

Bounded candidate: first contour the farm bend within metatile [35,13,40,17), using attached ditch-edge/reed or working-field shoulders to make a 2–3-core turn. Keep the open prairie identity: do not fill the field with conifers or scatter boulders. Separately, test a two-core jump opening around retained core x52–53/y16, with a visible solid retaining edge across the rest of the old bank. Keep the same upper approach (53,15), hop lane (53,16), landing (53,18), and western uphill gate. Preserve at least the current six grass steps after the drop; a dry clearing or shoulder must not become a new route around the habitat.

Retain every trainer/item/sign, including hayfield trainers (39,9)/(50,13), Hyper Potion (49,7), and the seed grower (47,29). Keep native bridge rails/decks, river access and visible Surf closures, golden derived blocks83/84, the existing bridge battle-or-grass choice, and `COTTONWOOD_NECK` core [82,18,86,28) with its four required grass steps. Neither the river nor the full route needs enlargement.

Proof needed: whole map plus unchanged camera anchors (75,30)/(53,15); both hop lanes, blocked reverse and course-side bypass checks under native directional collision; actor-present ordinary return; minimum six grass steps from the drop and four at the cottonwood neck; every actor/reward approach and all bridge/water boundaries. A narrower lip is a proposed design revision, not permission to silently change the existing ledge contract or its return semantics.

## 3. R10: make the western rock climb legible with native stairs

Inspect [whole R10](../renders/iron-range-descent/whole-route.png), [WESTERN_RAMP, core (11,22)](../renders/iron-range-descent/viewport-western_ramp.png), and [WEST_ROUTINE_LANE, core (7,11)](../renders/iron-range-descent/viewport-west_routine_lane.png). The weakest local transition is the two-way carrier ramp at metatile (5,10), [block74 placement](../routes/MN_R10.layout.json:972). The rock faces already give this route a strong identity. At player scale, the ramp reads as an abrupt lawn slit between brown walls, followed by a broad lower apron. Adding more rocks elsewhere would not clarify this climb.

Bounded candidate: fit one authentic cave-stair metatile into this climb and contour only its attached lower shoulder, roughly metatile [4,9,8,13). Keep a useful rest turnout and readable upper/lower turn. **Outdoor cave stairs are a proposal and are not implemented on R10.** Current johto74 collision is `[WALL,FLOOR,FLOOR,FLOOR]`; cave12 is `[FLOOR,FLOOR,WALL,FLOOR]`, and cave30 is `[FLOOR,FLOOR,FLOOR,WALL]`. A blind visual or collision swap would change legal side approaches. Select the orientation from the actual walking lanes, require exact native source quadrants, and use a bounded typed art extension if needed. R11's already installed stairs demonstrate the art approach; their numeric IDs must not be assumed valid in R10's johto tileset.

Preserve the two-way connection between lower core (11,22) and upper (11,19), the nearby signs at (8,24)/(4,12), rest turnout and resident/trainer approaches. Keep `CARRIER_BANK_DROP`, its ordinary western return, six-step `PINE_NECK_HABITAT`, the refined cave and exact exterior mouth records, and the protected Superior landing/rails. Do not move any actor, reward, endpoint, story flag or room to fit the stairs.

The current native walking links deliberately differ from the source layout at five metatiles. Preserve them exactly in any later native replacement: `(0,5):68→01`; `(47,10):02→05`; `(47,11):02→05`; `(47,12):02→01`; `(47,13):02→05`. These are the installed Grand Rapids/Duluth seam apertures and shoulders, not source defects to normalize away.

Proof needed: whole route and the same ramp/west-routine cameras, both directions on every intended stair lane, blocked wrong-side quadrants, no stair/ledge reverse bypass, retained actor-present ordinary return and required grass, and strict complete preservation of the five native seam overrides plus every nonterrain record. A native staircase build and real controller input would follow source visual acceptance; none is claimed by this audit.
