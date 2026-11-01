# St Paul compact expansion source handoff

The current candidate is **64×48 metatiles**, four times the original footprint, hand-recomposed after the user requested “smaller but tighter.” It is one native `MnStPaul` map (`MN_ST_PAUL`), not chunks. Its full original-resolution source composite is 2048×1536 pixels. Earlier 96×72 work and every rejected source/render pass remain in `renders/st-paul/expanded-20261027/`; this is a new source epoch.

## Executable inputs

- `towns/MnStPaul.tight.expansion.contract.json`: indexed old-record relocations, all new room definitions/warps/services/parties/items/flags and actual quest command arrays.
- `towns/MnStPaul.tight.layout.json`: explicit orthogonal terrain strokes using unchanged Kanto art, `source_pixels_modified:false`, `reject_cliff_cuts:true`. Only typed native material names are used;55 is ordinary native Floor brick,31 pebble,0a ordinary Floor turf. Native05 remains only the original ordinary brick return and has LADDER collision.
- `towns/MnStPaul.tight.candidate.json`: rich dossier and activity/facade roster. The original four town room source records remain exact. Union Depot is an unchanged external native dependency.
- `towns/MnStPaul.tight.plan.json`: exact indexed preimages and proposed old actor/BG/door coordinates; the executable contract is authoritative over historical descriptive fields.

Freeze `renders/st-paul/tight-20261027/v08/` for the readable-text native epoch. Its full whole view, actors RGBA, all37 new room originals,15 player-size decision views and5 weakest-space views are retained. These are source-art composites, not actual native screenshots. The day palette and exact standing source sprites are used; no native gameplay claim follows from them.

## Activities and density

Five original facades plus35 new paired entrances give40 street destinations. Thirty-seven new room modules include the three-floor Lowertown Exchange; this does not duplicate the original five rooms. Ten independently stocked native shops, eight optional two-Pokémon exhibitions, nine original-style field/room pickups, a native yes/no fair quiz and two distributed acknowledgement errands are authored. Original Gym4 and Union Depot courier/manifest/story programs remain dependencies with exact old room warp indices.

Fair concessions use actual two-block native blue fronts with real Mart interiors and stocks, sharing a seated brick food court. Agriculture/breeder/exhibition buildings have connected booth lanes. Capitol retains its genuine down-bank shortcut and ordinary return. Attached native street walls, garden beds and turf frame the urban street network; these walls have no fake doors. Two railed four-core river bridges cross a finite enclosed river; Surf reaches no unconnected outer edge.

Activities require native controller proof. `reviews/st-paul-tight-activity-contract.json` specifies question answers YES/NO/YES; three archive records in Archive/Museum/Annex; three outdoor walking measurement signs; wrong/missing-record refusals; full-pocket failure without success flags; native make-space/retry; once-only quantities; printed text/input ownership and Continue. No new main-story, HM, badge, or train travel is promised.

## Preserved native contracts

Preimage pack `content-packs/minnesota-northstar-seep-shelter-review.crystalpack`, SHA7e5f65b935b1f74766c810ee72f444ad60a7e71e096b5dec10c46f516595344a,541 maps. Its city module canonical SHA is2ce92ac1ece81bc9801ab48468221fd1fc9bced9bd666bc7774164e6eb66f675.

All9 old objects,8 background events and10 exterior warp records have explicit indexed before/after coordinates. Old object programs, IDs, scene/event fields and all room targets/indices remain exact. Only four separately approved obsolete fair text bodies change: both northern-annex guards, FairSteward and FairSign. Their exact old/new printed lines are in `text_revisions`; commands and flags are preserved. Public fair activities are inside the city; the northern annex remains closed by both original guards.

R04 meets the west edge `(0,48)/(0,49)` with reciprocal native offset15. Exterior recovery spawn271 becomes `(6,48)` on the same city map. Original clinic recovery272 and every original room arrival remain unchanged. Union Depot exterior1/passive2 moves to `(92,69)/(92,70)`; its complete16×12 native module including five warp records, courier party and manifest acknowledgement remains exact.

## Reproduction and limits

```
.local/minnesota-review-env/bin/python planning/minnesota/tools/author_st_paul_tight.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_st_paul_tight.py --revision NEW_IMMUTABLE_NAME
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_st_paul_tight_contract.py
```

Source review checks actor-present approaches,40 complete native door facades and safe returns, contiguous reciprocal room/stair/passive records, exact old indexed preimages/standing collision, unchanged old room authoring, legal source stocks/classes/species, no incidental Cut, and closed outer walking/Surf boundaries. Fifteen meaningful in-memory mutations are rejected against a clean source control. The former loose-source renderer accepted missing typed fields/custom material aliases and diagonal paths; these defects were fixed and failed passes retained. Native compile/readback/controller/capture review remains Root-owned and pending. No Cargo or pack publication was run by this author, and active32×24 town sources were not promoted.

Native v06 parsed all programs and room/old-anchor geometry, then correctly rejected the unconnected northern annex Floor edge. v07 replaces only that outer metatile with native13 stone wall; source now rejects unbound Floor terrain even when guard bodies make it unreachable. Failed native logs remain in their immutable stage folders. Native publication and actual controller review are still Root-owned.

Root successfully published v07 as the third tight city in `content-packs/minnesota-three-cities-tight-v07.crystalpack` (660 maps), with strict native program/room/connection/boundary checks. The authorized v08 epoch changes only six raw text bodies (and two mirrored source text fields) to fit18 columns; terrain and all source composite image pixels remain exact. Its contractSHA is758ba4f2cd5279c69c2a2fe7476a75759ea2154b3b84889e23b33cc92204d812. `v08/text-only-preservation.json` records every changed field. Root owns actual new publication/controller execution; the four new ignored StPaul controller cases are authored and formatted, not reported passed.

The authorized v09 correction moves only new `ARCHIVE_RECORD_1` from occupied passive arrival3 `(3,9)` to `(4,9)`, with a real approach `(4,8)`. `v09/arrival-only-preservation.json` proves no other contract field changed fromv08. Native commands/text/flags/warps/parties/items and exterior terrain are exact. Source now rejects occupied primary and passive arrivals;16 meaningful negatives pass. All37 rooms and the revised archive were rendered and visually inspected. The controller record case actually walks to `(4,8)`, presses Down against the actor, then A; no state moves are used. Native v09 publication/controller outcomes remain Root-owned and unreported until run.

### v10 bounded facility furnishing pass

Frozen source inputs and full paired source views are under
`renders/st-paul/tight-20261027/v10/`. Only terrain rows in 17 of the 37 new
rooms change from v09. All dimensions, native templates/tilesets, palette
source, indexed warps, actors, approaches, parties, flags, programs and shop
stocks remain exact; outdoor layout bytes are exact. The archive actor remains
at (4,9) with approach (4,8). The source proof rejects 14 meaningful bad variants,
including blocked original approaches and isolated Floor, with no positive-case
errors. Actual v10 native/controller acceptance remains pending Parent.

Parent's v01 activity execution passed three of four cases. The archive/river
case reached its final resource comparison and differed only in earned walking
friendship (70→71). The test now permits nondecreasing friendship and normalizes
only a cloned comparison snapshot's happiness before comparing all other party
fields exactly. Actual game state is untouched and actual party persistence is
still checked exactly across Continue. The original failed leaf/log are retained.
No gameplay failure or production change is inferred from that comparison bug.
