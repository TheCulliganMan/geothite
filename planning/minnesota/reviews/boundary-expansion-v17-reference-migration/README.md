# Boundary Waters v17 shared-reference migration proposal

This is a **source-reference audit and dry-run proposal**, prepared after Root accepted the immutable64×56 candidate. It edits no shared world/interior/live-zone files, no native code and no pack. A prospective **505-map** combined North Shore/Safari revision is the publisher's target, **not a file verified by this audit**. Its actual file SHA256, internal content identity and executed verification results must be added by Root after publication; no earlier pack hash is substituted.

Accepted inputs remain exactly:

- `renders/boundary-safari/expanded-candidate/cliff-lakes-v17/boundary-waters-safari.layout.json`: `6c07a960b1d5b5f7e0f00c5931ebe6407eef552b6af53c788b66afb5de0ea516`.
- Same folder `boundary-waters-safari.json`: `5ae5aacc71f82d948e15b9817528c47ecc121a512c8578497628bcef143d0a1c`.
- Machine geometry/actor/sign handoff: `handoff-e2f363751cd03997df8f1fa6ae072815b39b70cb541814900da219a4c4d9ae3e.json` in that folder.
- Baseline sources, every previous candidate, whole render, original160 camera and conversions remain retained. `revisions/northwoods-expansion/{before,after}` are also frozen review epochs, not live sources to rewrite.

## Exact current shared JSON edits

`guarded-field-proposal-f42898799dab11e81140078ed4c0630a57b186a17d6f69b244a550573d6c5944.json` contains **11 exact test/replace operations**, current file hashes, entity selectors and semantic preservation checks. Each replacement was applied in memory and reversed; reversal reproduces the entire original JSON tree. Nothing was written to those sources. Re-resolve array indices by unique entity ID if unrelated authors reordered records; reject any changed old value instead of overwriting another writer.

| Shared source | Stale binding | Required expanded binding |
|---|---|---|
| `interiors.json`, contract `MnBoundaryRootBeerCabin` (observed index67) | exterior `MnBoundarySafariC00R00` / `MN_BOUNDARY_SAFARI_C00R00` | `MnBoundarySafariC01R01` / `MN_BOUNDARY_SAFARI_C01R01` |
| Same contract | global DOOR1 `[51,39]`, passive FLOOR2 `[51,40]` | global `[71,55]` / `[71,56]`; owner-local `[7,7]` / `[7,8]` |
| Same contract presence prose | one outside Dorothy `[49,43]` | same `BWS_ROOT_BEER_LADY`, global `[69,59]`, owner-local `[5,11]`; approach `[70,59]` |
| `towns/MnBoundaryWaters.json`, `admission_contract.pending_preserve_warp` | target C00R01; global arrival `[15,73]`, local `[15,25]` | target C00R02; global `[15,105]`, local `[15,9]` |
| `world.json`, `special_zones` Safari status | four published chunks; regional access pending | six exterior chunks plus unchanged cabin; distinguish expanded publication evidence from already separate free basecamp/story access |
| `playable/root-beer-cabin.json` | **No fixed exterior coordinates or exterior map owner** | **No coordinate/row/actor/warp-index edit needed**; existing dynamic `BWS_ROOT_BEER_CABIN_DOOR` contract is correct |

Keep town pending warp **`enabled:false`**, admission **`CLOSED_NO_FEE`**, `money_taken:0`, `session_started:false`. Do not move the separate free basecamp's officers `[32,6]/[33,6]`, its early-return `[32,10]`, town/shelter doors, public regional connections or Q07 actors. Preserve every interior field not listed in the guarded proposal: native group61/map5, empty5×4 room, exact20 rows/blocks, arrival `[5,6]`, carpets `[5,7]/[6,7]`, indices1/2/passive3, target3 inbound/target2 outbound and services.

## Live zone promotion is a whole-source operation

The current `zones/boundary-waters-safari.{json,layout.json}` still describe the old48×40 terrain. Do not search/replace arbitrary numbers: many terrain cells and unrelated region maps legitimately contain the same numbers. Root should promote both accepted v17 documents together, preserving their exact geometry/actor/sign/program identities, then perform narrowly recorded **presentation metadata** updates outside the immutable candidate:

- Set live dossier/layout status to the actual expanded publication state and evidence, never automatically inherit either the old450-map success claim or a new controller success claim.
- `terrain_review_command` currently intentionally targets immutable v17 files. For a live-source default command use `tools/review_boundary_safari.py` (whose defaults resolve the shared zone pair), and keep the exact candidate command as retained review provenance.
- Candidate `baseline_sources[*].snapshot` values are relative to its own folder. If copied live, prefix them with the full repository-relative `planning/minnesota/renders/boundary-safari/expanded-candidate/cliff-lakes-v17/` path, or retain an explicit `candidate_root` resolver. Never imply `zones/baseline-sources` exists.
- Keep `expanded_candidate_contract` and old review pack SHA/content identity labeled **candidate/baseline evidence**. Its `37a09cd5…` file is not the future505-map pack. Update current publication evidence in a separate explicit field, with the actual new file SHA.
- Layout `root_beer_lady.door.native_status` and `room_proposal.status`, and dossier `root_beer_lady.item.implementation_status`, inherit old pending implementation prose. Replace only their explanatory prose with the actual installed unchanged room/item/service status plus expanded verification state. Do not modify native room/service fields or create a second Dorothy.
- Dossier `entry_gate_contract.admission` / `return`, `expedition`, `capture_encounters`, `acceptance`, and layout admission/retirement text are **future Safari specifications**. Add an explicit `runtime_status` note that overrides their present-tense ambiguity: no current fee, permit, balls, steps, habitat encounter installation, capture-only mode, retirement or admission warp. Native guards/rangers and signs01/06 retain the installed truthful closed text. An expansion is terrain and relocated existing hospitality, not capture implementation.

## Native geometry expected in the next revision

All coordinates below are global16px core unless marked metatile. These are review expectations, not a claim that a written file passed them.

| Native identity | Stable group61 ID | Metatile extent |
|---|---:|---|
| C00R00 |1|origin0,0;32×24|
| C01R00 |2|origin32,0;32×24|
| C00R01 |3|origin0,24;32×24|
| C01R01 |4|origin32,24;32×24|
| MnBoundaryRootBeerCabin |5|existing5×4 interior, unchanged|
| C00R02 |6 appended|origin0,48;32×8|
| C01R02 |7 appended|origin32,48;32×8|

Seven internal neighbor pairs mean14 reciprocal records, offset0. Core seams are x64, y48 and y96. Exactly3584 exterior blocks replace1920. The source has20 preserved actors plus the same two permanent native closed-admission guards, and18 signs (13 preserved plus the exact five allowed expansion IDs). Exterior native art keeps original Johto and imports80–83 unchanged; new84 is exact cave12 art/collision, no warp. The written-pack reviewer expects133 compiled preserve blocks. Interior art/room geometry and all unrelated maps/assets/audio remain unchanged.

| Anchor | Old | Expanded expectation |
|---|---|---|
| Protected arrival/return |15,73; C00R01 local15,25|15,105; C00R02 local15,9|
| Physical gate |11/12,68/69|11/12,100/101|
| Native guards |11/12,69|11/12,101|
| Reed ranger / approach |10,71 /11,71|10,103 /11,103|
| Entry inside / launch bank / water |see retained old layout|12,98 /3,90 /3,89|
| Granite ranger / approach |83,27 /82,27|115,35 /114,35|
| Quiet ranger / approach |71,67 /70,67|103,99 /102,99|
| Dorothy / approach |49,43 /50,43|69,59 /70,59|
| Cabin door / passive return |51,39 /51,40, C00R00|71,55 /71,56, C01R01|
| Pine Cut / approach |10,31 /10,30|unchanged; native63→3f, rare target9,35|

Cabin exterior records1/2 target `MN_BOUNDARY_ROOT_BEER_CABIN` passive3. All cabin records1/2/3 target `MN_BOUNDARY_SAFARI_C01R01` passive2. Rebind typed events, split raw event commands and global `map_scripts` mirror coherently, without changing other commands. Preserve cabin ID5 when appending row02. The actual outside Dorothy gift/shop/rest/item/first-gift flag must migrate by the same actor identity; root reported finding and repairing a typed service-index rebuild defect during asset testing. Actual service/controller reruns, not these coordinates, establish the fix.

## Documentation patch boundaries

Do not erase old failing tests, captures, coordinates or pack hashes. Apply current-state summaries only after Root's written-pack verification. Suggested exact scoped changes:

| File | Current section to update | History to retain |
|---|---|---|
| `playable/boundary-waters-safari.md` | Opening status and map composition through entry paragraph (currently lines3–64):64×56, six chunks/new table, seven pairs/y96, four unchanged habitats with new bounds, entry105/gate100–101; new six-node2×3 Mermaid topology with C00R02 arrival. Admission/return graph arrows remain explicitly future. Add actual505 evidence separately. | From previous map review/runtime audit/controller logs onward (especially current153,155,205,241,243,277,291,311), all51,39/51,40 and old map counts belong to the exact old reviewed file. Label the entire retained section **48×40 historical publication evidence**, don't rewrite old logs as if new tests ran. |
| `towns/MnBoundaryWaters.md` | Only prospective preserve sentence at current71: global15,105→C00R02 local15,9, still disabled. | All free town/source/native/Q07 evidence and regional actor positions. |
| `WORLD.md` | Safari paragraph at88: accepted64×56 six-chunk preserve; actual publication result when available; optional closed access separate from free hub. | Generic chunk naming/origin formula and unrelated maps. |
| `README.md` | Safari paragraph at145: distinguish actual terrain/cabin publication from future capture/session design; six chunks and unchanged room5. | Other destination inventories; don't add cabin to regional town count or alter rail roster. |
| `STATUS.md` | Add a new expanded epoch immediately before old48×40 paragraph at378 with actual new evidence; label old450 proof historical. | Existing exact native450/controller/audio and cloned-house proof, all failures and retries. |
| `playable/region-review.md` | Add a scoped next-epoch note for expanded six-chunk preserve and needed same-position review; existing four-preserve regional proof is historical. | Original471-map journey, source/frame hashes and earned-progress limitations. |
| `reviews/northwoods-expansion-native-contract.md` | Cross-link finalv17 and actual verification from the new source migration note. | This is a read-only before/after native-hardcode audit: old51,39 and15,73 are valid legacy observations. Do not overwrite another agent's audit. |
| `playable/root-beer-cabin.json` | No required coordinate changes. Optionally label old `required_native_change`, unimplemented shop/backend phrases and `source_review.static_evidence` as historical/future indoor-counter notes; root_beer implementation status already records actual outside hospitality. | Exact room and conditional-serving preview remains future; no new serving actor or service relocation. |

The live zone pair's old51,39/51,40 and arrival73 values are removed by whole accepted-source promotion. All remaining occurrences in `renders`, `baseline-sources`, `revisions/before`, native comparison reports, historical prose and saved captures are intentionally preserved. There is no unambiguous global replacement for `C00R00`: it remains a real valid northwest exterior and appears throughout unrelated maps.

## Story and admission wording

Use this current narrative after the enlarged terrain is actually published:

> The free Boundary Waters basecamp remains the mandatory route junction and Q07 survey site. Beyond its northern closed checkpoint lies the optional lake preserve: connected lakes, pine shores, six grassy portages, four camps, five short cliff drops with ordinary stair returns, and Dorothy's existing Isle of Pines hospitality. The expanded landscape does not open expedition admission. Native officers and rangers explain that permits are unavailable and no fee is charged. Ball/Bait/Rock/Run, paid entry, balls/steps, habitat encounters and retirement remain future work. Review fixtures inside the preserve are explicit location setup; they do not prove an earned entry or active Safari capture mode.

No new story flags, admission permission, reward, capture table or actor is proposed here. Free care, the chief battle and separate receiver restoration keep their existing Q07 semantics. Dorothy's optional actual gift/shop/rest grants no Q07 progress and creates no Safari state.
