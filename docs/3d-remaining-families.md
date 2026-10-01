# Remaining source-family review

This is an implementation checklist, not an art-signoff. Counts are residual
8-pixel source cells from the inspected base maps. They are not triangle counts,
completed objects or percentages of finished scenes. Plot counts can include
backing ground. Dynamic map states and individual species art remain separate
work in the broader conversion checklist.

The latest production audit verifies all 388 base maps without a mesher error.
It records 243,314 cells consumed by authored geometry, including named surface
finishes. Per-map source reports stay local under `target/`; no content pack,
source image or generated audit manifest belongs in the shipped repository.

## Actionable families

All 22 entries below still need integration or refinement plus native review.
The three Kanto boundary-rock families are installed and tested, with native
Route17, Route20 and SilverCaveOutside captures inspected. Source-aware water
finishing removes mismatched underlay squares without changing the water datum.

| Family | Residual cells | Representative scene / requirement |
| --- | ---: | --- |
| Kanto capped posts | 1,904 | Route13; narrow two-row courses |
| Lighthouse perimeter | 3,504 | Six floors; add junction, end, corner and window orientations |
| Gate counters | 1,482 | 28 maps; connected straight/return/cap topology |
| Gate phone consoles | 96 | 24 maps; handset and keypad forms |
| Cable Club partitions | 730 | Pokecenter2F and beta rooms; preserve booth openings |
| Gym display-planter boxes | 720 | GoldenrodGym and AzaleaGym |
| Broad central Gym tree | 9 | AzaleaGym; rounded canopy and trunk |
| Gym maze walls | 356 | ViridianGym; retain all native gaps |
| Gym round hedges | 168 | CeladonGym; complete crowns on native floor |
| Outdoor sign frames/posts | 348 | 40 maps; retain meaningful live source lettering |
| Facility instrument cabinets | 208 | PowerPlant and four other maps; equipment silhouettes |
| Facility workstation plots | 152 | Rocket Base and related rooms; plot count |
| Radio desks/counter plots | 380 | Six maps; plot count |
| Ship table/captain-desk plots | 288 | Five cabins; plot count |
| Department-store wall courses | 704 | 12 maps; joined walls, lift sides, directories |
| Department-store U displays | 64 | Both 5F maps; preserve the closed central topology |
| Timber divider screens | 40 | WiseTriosRoom; panels and separate returns |
| Barn stall rails | 24 | Route39Barn; complete panels |
| Theater stage/backdrop | 240 | DanceTheater; refine existing raised fascia and surface |
| Magnet Train body/boarding plots | 128 | Both stations; retain boarding opening; plot count |
| Park small round props | 44 | Three maps; confirm identity in native context |
| Park fountain rim/basin | 48 | Three maps; refine existing grouped animated treatment |

## Verified modeled boundary families

The current production audit counts 5,480 land-boundary, 2,488 shore and 1,168
pale-path rock cells. Each complete 2×2 drawing becomes a closed sculpture in the
shared terrain batch. Source coherence and same-atlas native backing are required;
changed or incomplete art retains its previous renderer. The counts differ from
the earlier residual-family estimate because the production adapter checks all
eligible drawings, including cells already represented by earlier partial forms.
Native terrain edge strips outside these rock footprints remain a refinement gap.

## Resolved source bindings

- Route40 and RuinsOfAlphOutside now use their actual same-atlas path underlay
  for 88 complete standard trees (704 cells), with source completeness retained
- The two department-store 4F maps now select their native floor for 24 existing
  shelf/fridge drawings (192 cells); custom profiles retain their own guards
- Kanto Center/Mart roles and the house block-08 window/wall roles are corrected;
  positive model counts had hidden both errors
- Rocket Base B2F no longer requests retired divider paint when the authored kit
  has already replaced every legacy divider

## Preserve legitimate source surfaces

The 656 waterfall cells already have explicit geometry and live animation.
The 3,148 buoy-course cells intentionally sit at water height. Both may need
material refinement, but neither is evidence of missing static prop volume.
Floors, water, voids, source inscriptions, framed pictures, sign lettering,
carpets and live markings can remain surfaces. Domestic facade classifications
also include floor/threshold cells behind already-modeled walls.

Completion requires faithful source ownership and whole-object rejection,
unchanged footing/warps/controller behavior, appropriate dynamic-state fixtures,
and native visual review of every family in representative layouts. A positive
map count or a generic replacement does not close an entry.
