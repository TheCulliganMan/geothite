# R19 prairie oak approach — candidate v01

The candidate gives the central dry prairie road a short turn around an attached native oak hook. A southern oak field edge joins the existing forest boundary and frames the prairie observation board. The route remains32×30; eleven metatiles change inside approved `[10,19,15,24)`. This is a source-only candidate. Native publication, controller and GPU acceptance remain pending; no active layout or dossier is promoted.

Frozen directory: `planning/minnesota/renders/prairie-forest-link/prairie-approach/v01-oak-hook/`. [Whole candidate](../renders/prairie-forest-link/prairie-approach/v01-oak-hook/after/whole-route.png), [six before/after player-scale decisions](../renders/prairie-forest-link/prairie-approach/v01-oak-hook/before-after-decisions.png), [local weakest frame](../renders/prairie-forest-link/prairie-approach/v01-oak-hook/after/viewport-local_worst-native-rgb.png), [global weakest plain-ground frame](../renders/prairie-forest-link/prairie-approach/v01-oak-hook/after/viewport-emptiest_ground-native-rgb.png), and [remaining upper bend](../renders/prairie-forest-link/prairie-approach/v01-oak-hook/after/viewport-widest_routine_bend-native-rgb.png) were personally inspected. The whole and all six camera pairs show the change without hiding the rest of the map. These are exact native-art source composites; they contain the original NPCs at native-facing frames and source-minus4px standing origins, and do not claim production GPU output or a player fixture.

## Exact preimage and source

Native preimage: `content-packs/minnesota-northstar-map-quality.crystalpack`, SHA256 `9aa950a35453f06cfee2e4a9b674c6d902af7c311f9ca64a083665ac6c60ddb2`. Both actual R19 exterior chunks' entire blocks match the frozen source preimage. The candidate adds eleven explicit point strokes to the original layer prefix; no art/tileset/extent/other layout field changes. The entire dossier, all actors, parties, text, observation quest/reward, flags, native source art and both endpoint contracts remain unchanged.

| Input/output | SHA256 |
|---|---|
|`baseline-MN_R19.json` / unchanged active dossier|`94312842d9155b6597cb2339e6843fe858e44dcd75024063f3df1d5cd9533d8d`|
|`baseline-MN_R19.layout.json` / unchanged active layout|`95d33df286cc57a615163c49b06c16bdd7934a9c93c6719a483b79688b09977e`|
|`MN_R19.layout.candidate.json`|`75b15b4fbc2cec8fe9e1890d8936b726e5f51027ef73ca7c0973b11178ccbfb5`|
|`source-proof.json`|`75402ac1fb32b9b8ba192f15b6a2a086ff80ff84d26e385b99341fbfbfe56495`|
|`after/whole-route.png`|`fd5b07c996a744aa11dc6783357c835ff070192a471cd9a2ec30c4fd742c9f85`|
|`before-after-decisions.png`|`30fd468cb89d8f7558f0e11a13750c2b80a5d412b539f3c0ad02ce96cb0a6cb8`|

The eleven changed metatiles are `(11,19)02→61`, `(11,20)3c→61`, `(13,20)02→01`, `(11,21)02→5d`, `(12,21)01→5d`, `(13,21)02→01`, `(14,21)02→61`, `(13,22)02→01`, `(14,22)02→61`, `(13,23)02→61`, `(14,23)02→61`. Native61 is solid Headbutt canopy; native5d has Headbutt upper quadrants and Floor lower quadrants. Neither has Cut collision. Every old encounter-grass quadrant is retained.

## Actual source checks

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_r19_prairie_approach.py \
  planning/minnesota/renders/prairie-forest-link/prairie-approach/v01-oak-hook
```

Frozen `review-frozen.log` passes in1098.1ms after pack loading. It reuses the existing whole-route renderer and R19 reviewer; no parallel renderer or game-content exporter is introduced. Active source/pack hashes stay unchanged throughout. Actual directional collision checks include native enter and leave masks instead of treating roof side walls as solid.866 directional land squares remain reachable; the ordinary Floor/grass graph reaches854 squares. Both endpoints, the western ramp loop, eastern reward branch, all actor and sign interaction approaches and requested camera approaches are reachable with actor bodies present.

The compulsory southern habitat remains exactly four grass steps in both directions and has no dry bypass. The northern battle-versus-grass alternative remains eight grass steps in both directions; the optional meadow reward still costs at least two grass steps. Both original native74 ramps remain exact and their upper ordinary inspection path remains14steps. The accessible wetland and every perimeter block remain unchanged. Native Cut quadrants remain zero.

The small dry hook from `(25,41)` to `(25,44)` is five ordinary steps in either direction; the old straight path was three. Both core x26/x27 lanes y40..45 and the x24/x25 lower return at y46 remain Floor. The west loop `(21,44)`, east reward join `(29,41)`, board nook `(23,45)`, native walker `(27,39)` and board `(22,44)` remain exact and usable. This changes local movement without a new HM, actor, reward, ledge or hill.

All12 existing R19 negative cases pass. Four additional geometry mutations are rejected by actual collision checks after bypassing the editorial stroke guard: restoring the straight dry road through the oak hook, opening the eastern attached shoulder, blocking both ordinary hook lanes and sealing the west branch. Five editorial/preimage negatives also reject out-of-envelope changes, original layer edits, art-identity changes, an attempted dry habitat opening and a branch-blocking appended stroke. Total21negative cases; these do not establish production-controller execution.

## Review limits and retained history

The first safety check correctly rejected standing at the proposed `(25,42)`, which the new oak hook occupies. The honest legal approach camera is `(25,41)`; no player is rendered inside the tree. Keep `review-first.log`, first/second reviewer snapshots, earlier proof versions and all render histories. Final cameras are `(25,41),(25,46),(23,45),(21,44),(29,41),(27,40)`, all reachable Floor. The algorithmic local weakest camera is `(29,39)`; the shared renderer separately records the global weakest plain-ground square.

The upper widest oak bend `(35,33)` and broad northern arrival remain unchanged weaknesses. This pass does not claim that all of R19 is finished. The observation nook also deliberately retains room for the board and western loop junction; eliminating every open square is not the goal. Root must review the candidate before any native revision and preserve the entire538-map story epoch during publication.
