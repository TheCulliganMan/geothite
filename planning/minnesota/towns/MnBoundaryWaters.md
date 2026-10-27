# Boundary Waters free basecamp — source terrain handoff

The complete town is authored at **32×24 metatiles (64×48 core tiles; 1024×768 source pixels)**, with one **8×6 Survey Shelter**, and is published in the native471-map review pack. All nine production-controller cases now pass: four story cases cover free healing, actual chief victory, separate receiver restoration, played loss and Continue; five connection cases cover the guarded regional seams, refusals, inbound care, transverse arrivals and Surf bypass attempts. See the [executed native runbook](../playable/boundary-basecamp.md). The successful connected story case also earns Q07 restoration, walks into R13 and Continues there. Safari expeditions remain closed. Source previews and static floods do not replace these input proofs.

The town is a pine-wrapped free trail hub: an enclosed public lake and railed landing west, a rustic survey cabin close to the eastern road junction, a separate closed northern expedition spur, and a small shaded bench stop southeast. The three route ports and the free shelter require no money, permit, Safari balls or expedition steps. The integrated southern regional checkpoint requires all eight badges, the lighthouse signal and earned survey restoration.

## Files and fast review

- `MnBoundaryWaters.json`: exact actors, text, doors, room rows, ports and progression contracts.
- `MnBoundaryWaters.layout.json`: schema2, named explicit orthogonal strokes, `reject_cliff_cuts:true`; original Johto art plus existing derived80–83 in `mn_boundary_safari`.
- `../tools/review_boundary_basecamp.py`: warm source render, collision/anchor validation and four meaningful negative cases. No exporter, random geometry or alternate runtime.

From the repository root:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_boundary_basecamp.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_boundary_basecamp.py --serve --watch --port 8803
```

The viewer is `http://127.0.0.1:8803/`. The process retains the supplied pack and source art in memory and rerenders changed dossier/layout files. Its default input is the ignored `content-packs/text-tui.crystalpack`; it creates the exact existing derived collision recipes only in memory if the preview pack lacks `mn_boundary_safari`. Current outputs stay ignored in separate `../renders/boundary-basecamp/states/{bare,closed,qualified}/` directories. Earlier root-level files remain retained. The default8803 server serves only `states/bare/`. Every generated terrain/whole original, RGBA preconversion, RGB preview, individual160×144 crop, converted640×576 viewport, whole room and camera sheet receives a versioned filename. All outputs now use shared `render_archive.save_render`: previous and new exact encoded bytes are retained in `history/` under full SHA256 filenames before updating each latest alias. Earlier `retained-aliases/` files are also preserved. Rejected full passes and sheets remain saved; no render cleanup is performed. Native capture retention is separate and Root-owned.

Current frozen terrain pixels are SHA256 `4ec0c05fff918958a7f93b589ab76c4e1dfd70c594532f2dd45c8a882a1a32db`; source-keyed immutable bare review **d109d28a** in `states/bare/`:

- `whole-town-d109d28a.png` — entire town at once.
- `viewpoints-d109d28a.png` — contact sheet; individual cameras are exact160×144 source pixels, with nearest-neighbor enlarged aliases.
- `camera-free_three_way_junction-d109d28a-160x144.png` — weakest central fork, now with the existing useful wayfinding sign moved into view.
- `camera-survey_road_junction-d109d28a-160x144.png` — cabin visible from the road decision.
- `camera-portage_rest-d109d28a-160x144.png` — reduced shaded bench clearing; guide is navigation only.
- `survey-shelter-d109d28a.png`, `camera-room-chief-receiver-d109d28a-160x144.png`, `camera-room-medic-arrival-d109d28a-160x144.png`.

Dossier SHA256: `ee11e455e314c9b16ddf730fdb635ac32025b7d4f4382cc78968742369f6ca73`.
Layout SHA256: `9e8ade95f5c5ab93ee8839d03d2aaae157d469ae3c4210d2f117298a1d9f98fe`.

## Visual review and bounded revisions

Inspected native-source Mahogany, Cherrygrove, New Bark, Azalea and Elm's Lab references, the complete town, and actual160×144 crops. Rejected first draft `be5e764f`: vacant central/eastern lawn, a detached survey yard and an oversized bench grove. Fitted pine fingers now frame the same three public choices; the shelter moved two metatiles south, and the shaded stop is deliberately small. Existing east-trail wayfinding was relocated into the central fork instead of adding decoration.

A collision-valid lab10 floor looked watery; lab0c repeated desk trim. Both were rejected by sight. Current ordinary lab01 floor is authentic gray timber; actual06/07 provide a desk/bin,14 bookshelves and08 the receiver PC. Chief, receiver, nurse, room arrival and carpets retain their fixed anchors. Subsequent exact native-pack comparison and actual engine review confirmed these source pixels and room approaches; see the executed evidence below.

## Exact exterior contracts

All coordinates below are16px core tiles unless marked metatile.

| Destination | Preserved metatile gate / landing | Actual outer seam pair |
|---|---|---|
| R12 east / Gunflint | 30,12 / 28,12 | 63,24 and63,25 |
| R13 south / League | 16,22 / 16,20 | 32,47 and33,47 |
| R20 west / Voyageurs | 1,12 / 3,12 | 0,24 and0,25 |

Root owns opposite route edge openings, all reciprocal native offsets and regional guards. No exterior facade/hidden-floor warp substitutes for these roads. Source verifies exactly these six reachable walking edge cells and no others.

`MN_BOUNDARY_SURVEY_DOOR` is real native Johto DOOR at **43,19**, with safe exterior FLOOR **43,20**. The complete facade is native `[[2c,2d],[2e,2f]]` at metatiles21,8. Exterior active warp1 and passive-return warp2 both target room passive warp3. Both interior carpets return to exterior passive2. This is the final revised exterior door; earlier43,15/43,16 evidence is stale.

Surface residents are ordinary native script actors:

| ID | Sprite | Tile | Purpose |
|---|---|---|---|
| MN_BOUNDARY_BORDER_RANGER | OFFICER | 10,27 | Free camp / western route wayfinding |
| MN_BOUNDARY_DOCKHAND | FISHER | 20,16 | Explains public landing and native Surf mouth |
| MN_BOUNDARY_ADMISSION_RANGER | OFFICER | 30,9 | Truthful CLOSED / no fee notice |
| MN_BOUNDARY_PENDING_GATE_WEST | OFFICER | 32,6 | Seals first core lane of unavailable admission |
| MN_BOUNDARY_PENDING_GATE_EAST | OFFICER | 33,6 | Seals second core lane |
| MN_BOUNDARY_PORTAGE_GUIDE | POKEFAN_M | 46,37 | Honest shelter / trail navigation, **no healing service** |

Seven physical Johto45 signs have real WALL anchors and reachable reading approaches. IDs and exact dialogue remain in the dossier. The existing `MN_BOUNDARY_EAST_TRAILS` is now the useful central direction sign at34,26; `MN_BOUNDARY_SURVEY_SIGN` is42,22. No incidental Cut tree exists anywhere in town.

## Public dock, closure and southern guard reservation

Public dock north rail uses existing82 at metatiles x7–11,y7 (`WALL/WALL/FLOOR/FLOOR`); south rail81 at x7–11,y8 (`FLOOR/FLOOR/WALL/WALL`). Actual closed west corner44/48 prevents walking off the end. Native Surf mouth is **FLOOR13,14 → WATER13,13**; land attachment is24,15/24,16. Water-under-art80 is actual Kanto78. The lake is fully enclosed in the town: **296 reachable Surf-water cells and zero reachable outer water**, with no neighboring water seam or invented canoe mode.

The northern check-in is **CLOSED_NO_FEE**. Two ordinary officers at32,6/33,6 fill its full aperture; native fence41 and pines seal the shoulders. Future32,5 is unreachable by walking+Surf, while early return32,10 and all free destinations remain reachable. The prospective preserve warp is disabled; global preserve15,73 would resolve to clipped-local15,25, but no paid/session behavior is implemented by this terrain.

South reserve: ordinary guards `MN_BOUNDARY_LEAGUE_GUARD_WEST/EAST` at **32,43/33,43**, backed by pine walls, are Root-owned conditional actors. Static blocking of those cells seals both south seam cells by walking **and Surf**, while western/eastern roads, town landing32,40 and survey door remain free. Removing those blockers restores the real FLOOR road. The source preview leaves them out to show the underlying authored road, not to grant story access. Their predicate is all8badges + `EVENT_MN_LIGHTHOUSE_SIGNAL` + `EVENT_MN_SURVEY_RESTORED`, with no money condition.

## Full Survey Shelter

Native map ID `MnBoundaryWatersSurveyShelter`, constant `MN_BOUNDARY_WATERS_SURVEY_SHELTER`; tilesetlab, templateElmsLab, dimensions8×6, border00. Exact rows are read from the dossier:

```text
00 05 09 09 05 09 05 00
00 01 01 01 01 01 08 00
00 14 01 01 01 01 01 00
00 01 01 01 01 01 01 00
00 06 07 01 01 01 14 00
00 00 00 00 02 00 00 00
```

Passive arrival **index3 at8,9 FLOOR**; active index1 at8,11 and index2 at9,11 are `WARP_CARPET_DOWN`, returning to exterior passive2 at43,20. Arrival suppression and return behavior require production controller proof, not just these coordinates.

- `MN_SURVEY_CHIEF`: SPRITE_ROCKET at9,5 facingUP; ordinary script placeholder uses the same object identifier and script ID. Clear approaches8,5 /10,5 /9,6. Pine-owned Q07 binder supplies actual battle and flag logic. Party MN_LODE_CHIEF is MAGNETON44 / WEEZING44 / CROBAT46. All8badges and LIGHTHOUSE_SIGNAL are prerequisites; only victory sets SURVEY_CHIEF_DEFEATED.
- `MN_BOUNDARY_SURVEY_RECEIVER`: physical BGEVENT_READ at12,3, actual lab08 WALL; safe approach12,4 FLOOR. Separate MS12 activation requires the chief-defeated flag and sets SURVEY_RESTORED. The ordinary placeholder is `jumptext MN_BOUNDARY_SURVEY_RECEIVERText`; chief placeholder is `jumptextfaceplayer MN_SURVEY_CHIEFText`.
- `MN_BOUNDARY_SURVEY_MEDIC`: SPRITE_NURSE at3,7 facingRIGHT, the **sole** `service:heal_party`; native binder must execute actual parsed `special HealParty`, including HP/status/PP restoration. No fee.
- `MN_BOUNDARY_SURVEY_ASSISTANT`: SCIENTIST at3,3 facingRIGHT, ordinary receiver/story information.
- Physical route-map BG3,0 is readable from3,1; supply-bookshelf BG12,8 from12,9 or11,9. All ordinary residents and BG objects have native text and clear interaction approaches.

Static room flood reaches93 unoccupied walking/carpet cells, all required approaches and both active exits. It does not resolve battle, healing, dialogue ownership or save persistence.

## Story preservation and acceptance still required

Q07 paid-state independence is explicit. Existing SQ03/MS17 remains preserved: arrival6,24 from R20, all8badges + BORDER_REPORT + LIGHTHOUSE_SIGNAL, one PP_UP and BORDER_CIRCUIT_REWARDED only after successful delivery. Root owns its native arrival transaction; the preview awards nothing.

Source checker passes all ports, door/room collision, full walking+Surf closure, closed admission, southern guard reservation, actor/sign approaches and text widths. Four named valid-schema negative cases reject the intended defect: removing one admission officer opens a bypass; replacing a dock rail with Floor loses its barrier; extending the lake to the unconnected edge exposes water; targeting active room warp1 instead of passive3 loses safe arrival. Warm rendering with full archival is about0.19seconds after pack/art load.

The initial source handoff required group62 publication with existing mn_boundary_safari art; exact source/native blocks/actors/warps; actual opposite route seams/guards; Q07 prerequisite denial, chief battle selectedPP/victory, separate console use and Continue; free native healing; doorway/passive-return input; dialogue ownership; actual public Surf/return and boundary closure; CLOSED Safari with zero fees/session state; matching native player-camera inspection. Root is the sole pack publisher.

## Focused whole-town visual revision, 2026-10-26

Root reviewed and accepted source candidate60b0ae6e for native implementation review; this is not final user visual approval. Dossier and all event anchors remain unchanged. The prior source candidate5a9a683d and its original/converted renders remain saved.

Exact160×144 review exposed two defects. The Gunflint entrance had blank lawn aprons about three walking tiles deep on both sides. The center's staggered main/League stems atx30/31 and check-in stem atx32/33 made the close intersections read as a pale plaza. Root approved a bounded correction before publication: north/east pine bounds[28,10,30,12], south/east[25,13,30,15], one continuous central spine atcorex32/33, and closure of the unused offset stem atmetatile15,13 and15,15..17. No props, buildings, actors, signs, room rows or port/guard/door coordinates were added or moved. The dock branch joins the same spine throughmetatile16,12→14,12→14,8→12,8.

Re-inspected `whole-town-60b0ae6e.png`, `camera-free_three_way_junction-60b0ae6e-160x144.png`, `camera-gunflint_arrival-60b0ae6e-160x144.png`, `camera-league_trail-60b0ae6e-160x144.png` and `camera-survey_road_junction-60b0ae6e-160x144.png`. The new center reads as one north/south route beside a readable direction sign; the eastern road reads as a deliberate forest entrance. Its quiet stretch connects the reserved route port to the survey/rest branches, rather than providing a second empty clearing.

Style comparison used the actual saved native-source `reference-MahoganyTown.png`, `reference-CherrygroveCity.png`, `reference-AzaleaTown.png` and `reference-ElmsLab.png`. Source landmark checks also inspected the public dock, closed check-in, small rest grove and both room camera frames. The dock's water, rail faces and closed end read clearly; the admission officers fill a visible fence opening; the small wood cabin has a coherent authentic facade; the indoor nurse sits on a clear approach beside the actual work table. The road-junction view reveals only the lower part of the cabin at its top edge, so its sign provides immediate identification and the full facade appears after walking north. No claim is made that the entire facade is visible from every branch.

These are native-source pixel previews, not engine captures. Actual471-map publication, eleven production2D captures and all four bare-town controller cases now pass; see the native runbook. Native widest fork, dock launch, shelter facade and medic/receiver room were reviewed at noon. Guarded regional input remains a separate acceptance step.

## Regional checkpoint contracts — source metadata applied

The dossier now records the separate `regional_checkpoints` collection and each port's `native_connection` contract for the atomic `boundary_basecamp_connections` integration. Ordinary residents, source terrain and door/room anchors are unchanged. The actual bare471-map town is now published at `content-packs/minnesota-boundary-basecamp.crystalpack`. It does not yet contain these six regional checkpoint actors or certify connected travel. Its two closed northern Safari officers are existing separate residents and must stay distinct.

Use a separate top-level `regional_checkpoints` metadata collection, rather than appending six more ordinary `residents` to the bare-town dossier. The native connection leaf adds these six bodies atomically; blindly adding them to the original resident constructor too would publish duplicate objects. Keep the existing League reservation's IDs/coordinates and requirements; migrate or reference it explicitly instead of creating a second independent League rule.

The metadata describes the following checkpoint fields: `id`, `route`, `native_target_map`, `direction`, `offset_metatiles`, `guard_ids`, `guard_tiles`, `inner_coord_tiles`, `outer_inward_coord_tiles`, `script_root`, `inward_facing`, `inward_movement`, `refusal_destinations`, `requires` and `native_binding`. Coordinates are core tiles; connection offsets remain metatiles. The exact proposed values, copied from Pine's integration contract, are:

| Field | Gunflint | League | Voyageurs |
|---|---|---|---|
| id / route | gunflint / MN_R12 | league / MN_R13 | voyageurs / MN_R20 |
| native_target_map | MnR12C00R01 | MnR13C00R00 | MnR20C01R00 |
| direction / offset_metatiles | east / +8 | south / +1 | west / −6 |
| guard_tiles | 61,24;61,25 | 32,43;33,43 | 2,24;2,25 |
| inner_coord_tiles | 60,24;60,25 | 32,42;33,42 | 3,24;3,25 |
| outer_inward_coord_tiles | 62,24;62,25 | 32,44;33,44 | 1,24;1,25 |
| script_root | MnBoundaryGunflintCheck | MnBoundaryLeagueCheck | MnBoundaryVoyageursCheck |
| inward_facing / native movement | LEFT / `step LEFT` | UP / `step UP` | RIGHT / `step RIGHT` |
| refusal_destinations | 59,24;59,25 | 32,41;33,41 | 4,24;4,25 |

Full body IDs are:

- `MN_BOUNDARY_GUNFLINT_GUARD_NORTH`, `MN_BOUNDARY_GUNFLINT_GUARD_SOUTH`.
- `MN_BOUNDARY_LEAGUE_GUARD_WEST`, `MN_BOUNDARY_LEAGUE_GUARD_EAST`.
- `MN_BOUNDARY_VOYAGEURS_GUARD_NORTH`, `MN_BOUNDARY_VOYAGEURS_GUARD_SOUTH`.

All are actual `SPRITE_OFFICER` / `OBJECTTYPE_SCRIPT`, statically face inward and use native `event_flag=-1`. They are physical checkpoint actors, not ordinary navigation residents or saved permission flags. Six original residents plus six checkpoint officers yield twelve objects in the connected town. Preserve ordinary residents, both closed Safari officers, all seven signs and the sole indoor Medic.

All three require the eight existing badge aliases plus `EVENT_MN_LIGHTHOUSE_SIGNAL`; League additionally requires `EVENT_MN_SURVEY_RESTORED`. Their actual native `checkflag` names are `ENGINE_ZEPHYRBADGE`, `ENGINE_HIVEBADGE`, `ENGINE_PLAINBADGE`, `ENGINE_FOGBADGE`, `ENGINE_STORMBADGE`, `ENGINE_MINERALBADGE`, `ENGINE_GLACIERBADGE`, `ENGINE_RISINGBADGE`. The checkpoint scripts never grant these conditions, the chief victory event or restoration. No fees, permits, Safari balls/steps or new saved visibility events belong in this collection.

Inner planes are always active coordinate checks; native refusal displays text then moves one step inward to the listed Floor destination. Qualified checks disappear the exact pair and allow ordinary outward walking. Outer planes skip the escort only for actual outward facing (Gunflint RIGHT, League DOWN, Voyageurs LEFT). Inward and transverse arrivals receive the two-step inward-care passage. Native two-step movement crosses outer→temporarily hidden guard→inner, then the script reevaluates prerequisites and restores unqualified officers behind the traveler. No off-map escort, room warp, alternative dispatcher or synthetic facing flag is proposed. Visibility reset/reentry/Continue behavior must follow and be tested against native `appear`/`disappear`; do not claim permanent saved removal.

The three preserved port records could gain a `native_connection` metadata object containing the target/offset above and these exact reciprocal records, without changing any existing gate or landing:

- Camp east63,24/63,25 ↔ R12 local0,8/0,9; reciprocal west−8. R12 global arrival is0,56/0,57.
- Camp south32,47/33,47 ↔ R13 local30,0/31,0; reciprocal north−1.
- Camp west0,24/0,25 ↔ R20 local63,36/63,37; reciprocal east+6. R20 global arrival is127,36/127,37.

Source validation/preview must distinguish three explicit states instead of treating all exterior gate cells as simultaneously free:

1. **Bare town471:** current six ordinary residents and underlying Floor seams, matching the currently published town exactly; no connected checkpoint gameplay claim.
2. **Connected unqualified:** six checkpoint bodies and their checked planes seal departure by walking and Surf, while public inward landings56,24 /32,40 /6,24, dock launch13,14 and shelter safe43,20 remain reachable. Inbound free-care escort is native scripted behavior, not a static flood through occupied guard tiles.
3. **Connected qualified:** requirements checked through native scripts, exact guard pairs disappear, and both lanes of all reciprocal walking seams work. Explicit fixture badge/signal setup is not an earned regional journey.

The source renderer should label whichever state it shows. Current native exact comparison of the bare town must not silently accept six extra actors until publication and source expectations change together. Root owns this metadata/schema/compiler/world integration. The prepared connection leaf's own asset/component and future controller results remain separate from the four bare-town Q07 fixtures. No geometry change is required by this proposal.

## Published native visual review and legal camera correction

The published bare471-map pack is `content-packs/minnesota-boundary-basecamp.crystalpack`. Leech's read-only native comparison preserves its whole town and room in `../renders/boundary-basecamp/native-compare/`, including `native-town-whole-original-rgba-5a109b56ffdf.png`, `native-town-camera-sheet-df3e0f3c76bf.png`, `native-shelter-whole-original-rgba-d87a087613e0.png` and the corresponding source images. Those are native-pack source-pixel renders, distinct from actual engine captures.

Inspected every actual production engine image in `engine/{junction,gunflint,dock,shelter-yard,closed-check-in,rest-grove,room-arrival,chief,receiver,medic}.png`, plus the added `engine/dock-launch.png`. These raw640×576 captures show a wider desktop viewport at the renderer's3× source-pixel scale. The direct `junction-lcd.png` is480×432. Its render wrapper rejected the dimensions after saving because that wrapper requires at least640×576; the remaining nine directLCD commands were never run. No successful480 CLI batch or engine validation change is claimed.

A centered crop box **(80,72,560,504)** of the640 junction is pixel-identical to the saved480 junction. For fair handheld framing, every wide frame has explicitly named `*-derived-lcd-center-480x432.png` and `*-derived-lcd-center-160x144.png` images; the160 version is a nearest-neighbor3× reduction of the480 crop. Raw-SHA-versioned variants also remain. Every derived PNG embeds the raw path, full raw PNG SHA256,640×576 source dimensions, exact crop box and derivation operation. Shared full-SHA history retains raw files, previous derived versions, previews and rejected passes. These are labeled derived crops, not separate successful directLCD invocations.

The source160 view and each derived nativeLCD view were checked by sight. The center has a continuous clear walking spine with its direction sign visible; Gunflint has two defined pine edges rather than blank lawn aprons; the survey front has a coherent rustic facade and small clear sign/door approach. The dock's rails, blocked western cap and actual one-core north Surf mouth are visible in the native launch frame. The closed expedition checkpoint has a legible fence opening filled by its two ordinary officers. The medic, chief and PC each have visible clear standing/interaction space; the gray laboratory floor and furniture remain coherent in actual engine rendering. The wider road view only reveals the lower survey facade at its upper edge; its nearby sign supplies immediate identification, and walking north reveals the full cabin. No claim is made that every destination appears in every fork frame.

One meaningful **camera metadata** defect was found: PORTAGE_REST46,37 was also the guide's NPC anchor, so the initial engine capture obscured the player. Root approved changing only this viewpoint to clear Floor **46,38**. The corrected native frame now shows guide and player distinctly on adjacent tiles, with both wooden seats in view. No actor, terrain, port, sign, door, room or story coordinate moved. A full dossier comparison against the saved prior snapshot confirms this is its sole semantic change.

Current source review key is **c9988c24**; the accepted spatial revision remains layout SHA256 `9e8ade95f5c5ab93ee8839d03d2aaae157d469ae3c4210d2f117298a1d9f98fe` and whole pixels SHA256 `4ec0c05fff918958a7f93b589ab76c4e1dfd70c594532f2dd45c8a882a1a32db`. The new dossier SHA256 is `bf5acadfc78e321ee62f40ba52b9090351dc44a710e81177e8c051f6d630c540`. Previous60b0ae6e/5a9a683d images remain saved. Static source validation and all four defect negatives pass again; those checks do not replace native controller or user visual acceptance.

No further frozen geometry change is proposed from this native visual pass. Root owns actual controller results, connected checkpoint integration and final user review. The checkpoint synchronization was still a proposal at that visual-review stage; Root's subsequent metadata application is recorded below.

## Root-owned checkpoint metadata synchronized; bare preview remains explicit

Root has now applied the reviewed checkpoint tables to `regional_checkpoints.checkpoints[]` and reciprocal native connection metadata to all three `ports[]`. The collection contains three checkpoint contracts/six guard bodies/twelve coordinate-event tiles and preserves all eight badge requirements plus signal, with restoration additionally required for League. `residents` still contains exactly the six original ordinary town actors. Terrain, room, story and actor/door/port/guard anchors remain unchanged. The earlier proposal above is historical planning; its metadata is now source-synchronized by Root, while connected publication/controller acceptance remains owned by Root and is under review.

The source watch is explicitly labeled **BARE_SOURCE_TOWN**, draws only those six ordinary actors, and reports `regional_checkpoint_overlay_drawn:false`. It does not silently append the new regional officers to the bare town. The closed northern Safari officers are already ordinary residents and remain visible. This describes the original bare-only pass; the optional static checkpoint states added subsequently are recorded below.

Current dossier SHA256 after Root's metadata synchronization is `a396d65203fae6dc44c9e51ee6044dc5d24d08ca059e9c6b896ef9818048ea90`; the new source-art review key is **9af1b280**. Geometry remains layout9e8ade95 with identical full whole-town pixels4ec0c05f. The prior c9988c24 viewpoint-only candidate and all earlier source/native images remain retained. The current actual-engine derivedLCD review sheet is `engine/derived-lcd-engine-review-cc1d8bffcca6.png`; it shows the bare471 town, corrected legal rest-grove viewpoint and actual dock-launch opening, not the pending connected checkpoint overlay.

## Optional static checkpoint preview states

The source renderer now accepts `--checkpoint-state bare|closed|qualified` and writes each state into its own `states/<state>/` directory, with complete full-resolution terrain/town/room originals, RGBA and RGB160×144 crops,640×576 previews, sheets, immutable version names and exact-byte full-SHA history. PNG metadata itself records the static source state and explicitly denies controller/earned-progression proof. Original root-level outputs and every prior state pass remain retained.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_boundary_basecamp.py --checkpoint-state bare
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_boundary_basecamp.py --checkpoint-state closed
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_boundary_basecamp.py --checkpoint-state qualified
```

`--pack content-packs/minnesota-boundary-basecamp-connected.crystalpack` can use the actual completed connected pack as the art/collision input. This does not dispatch native scripts or change its state. The default live watcher on8803 remains `bare` and draws only the six ordinary residents. No additional review server is required.

- **bare / BARE_SOURCE_TOWN:** six ordinary residents; regional bodies omitted to show the original bare-town contract. Physical outer Floor seams are visible, with no claim of connected native permission.
- **closed / STATIC_UNQUALIFIED:** six ordinary residents plus the six exact Officers from separate `regional_checkpoints.checkpoints[].guards`, never duplicated into `residents`. All three departures are physically sealed in both walking and conservative walking+Surf graphs. Public inward landings56,24 /32,40 /6,24, dock13,14 and shelter43,20 remain reachable from within town.
- **qualified / STATIC_QUALIFIED_ASSUMPTION:** the six regional bodies are omitted under an explicit assumption of all eight badges, signal and League restoration. The report lists those assumptions, `earned_progression_proved:false` and `native_escort_executed:false`. This is not a native saved-visibility result or earned journey. It also assumes all three regional pairs absent; it does not claim possessing badges automatically hides unvisited officers. The two ordinary northern Safari officers remain in all states; Safari stays CLOSED.

All states include additional exact160×144 views `CHECKPOINT_GUNFLINT` at60,24, `CHECKPOINT_LEAGUE` at32,42 and `CHECKPOINT_VOYAGEURS` at3,24. These are requested static town-only framing points on clear inner-plane Floor. Native inner-plane scripts would own input there; this renderer executes none. The three checkpoint windows clamp within the town extent using the same framing rule as the native-pack comparison, rather than inventing neighboring route terrain. Every camera PNG and the report record its requested tile and exact crop bounds. The original centered/clipped passes remain retained in history; no image was deleted.

The body-geometry validator checks every pair independently against its corresponding outer seam, then the entire closed composition against all public-care anchors. A real three-metatile east-shoulder Floor bypass is rejected with the specific Gunflint walking/Surf-bypass diagnostic; moving a real ordinary NPC onto a public landing is rejected as obstructed care. These read-only negative copies do not modify authoritative terrain or JSON. They supplement the existing admission/rail/outer-water/passive-arrival negatives. Static flood does not execute an inbound escort, facing branch, refusal, appear/disappear, room warp or cross-map connection.

With current dossier SHA256 `ee11e455e314c9b16ddf730fdb635ac32025b7d4f4382cc78968742369f6ca73`, the review keys are bare **d109d28a**, closed **feb1e77b**, qualified **3deeacf1**. Layout remains9e8ade95. Bare/qualified whole pixels remain4ec0c05f; closed pixels are04a37389 because only the six regional Officer sprites are added. The complete closed source image is RGB-identical to the published connected native whole `native-town-connected-unqualified-whole-original-rgba-9b11cc1fff50.png`, and actual native positions/sprites match the12 visible source actors. The whole and all three closed checkpoint crops were inspected by sight: each pair fills its narrow road lanes without actor/art overlap, with visible pine side boundaries. This establishes static composition; it does not establish native controller acceptance, including transverse edge approaches and inbound care.

Only this renderer and this runbook changed for this preview-state slice. Geometry, dossier, shared integration and packs remain untouched by this author.


## Final static preview review and connected input result

The final source pass inspected `states/bare/whole-town-d109d28a.png`, `states/closed/whole-town-feb1e77b.png` and every exact160×144 camera in `states/closed/viewpoints-feb1e77b.png`, including all three new checkpoint frames. No further concrete defect justifies changing the frozen terrain. The central sign marks the connected fork; Gunflint has defined pine shoulders; the shelter yard has a clear door/sign approach; the dock has a visible rail boundary and actual launch mouth. The full shelter facade is revealed by walking north from the junction, rather than being completely visible in that road frame. Each closed regional pair fills its two road lanes without actor/art overlap. Static closed source whole and all three checkpoint crops match the connected native-pack images pixel-for-pixel in RGB.

All49 versioned PNGs in each of the three current state folders were checked against their exact-byte full-SHA history copies and embedded preview-state metadata. The default8803 watch remains bare/six ordinary residents and has been restarted with the final extent-clamped checkpoint framing code. Previous centered/clipped passes, raw native originals, derived LCD crops and every rejected render remain saved. Layout9e8ade95 and the bare whole pixels4ec0c05f are unchanged.

Root reports **all nine production-controller cases passing** on the actual471-map connected publication (`e7257308`): four story and five connection cases. The earned story fixture supplies Mew80, all eight Johto badges and the upstream signal only at setup; chief defeat and restoration begin false. Actual selected attacks spend PP and win the chief battle, the separately acknowledged receiver sets restoration, and ordinary walking then crosses the southern checkpoint into `MnR13C00R00` at `(30,0)` and saves/Continues there. No post-setup position, party or story-flag injection supplies that release. The [story runbook](../playable/boundary-basecamp-story.md#connected-production-result-and-earned-league-release) records the exact limits: upstream badges/signal are supplied fixture prerequisites, the full earned regional journey is still open, and Safari expeditions remain closed. Source static floods do not execute those native scripts or replace the controller result; final user visual approval remains separate.
