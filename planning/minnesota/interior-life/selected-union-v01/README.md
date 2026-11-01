# Interior and selected-terrain composition

This native 2D candidate combines all 230 local non-gym interiors with the
selected R12 terrain, Mille Lacs park and progression rewards. Red Wing has
no station or timetable listing. The candidate is
`content-packs/minnesota-interiors-selected-r12-v01.crystalpack` (730 maps).
`world.active_local_review_revision` now selects
`interior_life_selected_union_revision`, retaining R12 as its recorded selected
parent and preserving all earlier revision records.

File SHA-256:
`1d60940da0f42dbce7bd5a595ccd8d8b8d68c2e84dcfa4ca505c67d5abd6595b`.
Content identity:
`548e9ab437aa9c8c1f9c00f1bfb804f5c997d5da63174a95ba48cc5b862c4463`.

The foundation is the completed
`minnesota-interiors-voyageurs-surveyor-v01.crystalpack`. It retains the repaired
St Paul facades, both Centers, riverbanks, bridges and Surf underpasses. Its
230-room inventory establishes authored character/purpose coverage; it does not
establish every later story state, an earned statewide campaign or completed
passenger service. See [the foundation](../VOYAGEURS-SURVEYOR.md).

## Executed Rust build chain

All publications use fresh ignored output paths. Preserve the completed packs
and retained failure logs; replay into new paths. Source export is unnecessary
for these guarded overlays on completed packs.

| Step | Existing Rust publisher | Contract or source | Output under `content-packs/` |
| --- | --- | --- | --- |
| R09 terrain | `target/interior-life/wasm-dev/rebase_accepted_terrain` | `r09-terrain.json` | `minnesota-interiors-union-r09-v01.crystalpack` |
| Mille Lacs | `target/runtime-fps/wasm-dev/pack_mille_lacs_park` | `mille-lacs-park.json`, original park town/layout | `minnesota-interiors-union-mille-lacs-v01.crystalpack` |
| R10 terrain | `target/interior-life/wasm-dev/rebase_accepted_terrain` | `r10-terrain.json` | `minnesota-interiors-union-r10-v01.crystalpack` |
| R11 terrain | `target/interior-life/wasm-dev/rebase_accepted_terrain` | `r11-terrain.json` | `minnesota-interiors-union-r11-v01.crystalpack` |
| Rewards | `target/northstar-story/wasm-dev/pack_northstar_progression_rewards` | current regional roster/curation in `planning/minnesota` | `minnesota-interiors-union-rewards-v01.crystalpack` |
| R12 terrain | `target/interior-life/wasm-dev/rebase_accepted_terrain` | `r12-terrain.json` | `minnesota-interiors-selected-r12-v01.crystalpack` |

The terrain publisher takes current pack, accepted base, accepted child,
contract and fresh output. Each contract pins all three content identities and
the complete changed-map list. It verifies exact cell preimages and refuses
accepted pairs that change actors, programs, events, art, audio or catalogs.
All stages pass the native geometry guard and strict written-pack readback.

The park publisher takes current pack, original park town JSON, original park
layout JSON, contract and fresh output. The local contract changes only the
parent binding: the complete R18 source module and both source hashes match
the original accepted inputs. Existing global identities and recovery points
are preserved; the new park receives a free native identity and one new spawn.

The reward publisher takes current pack, `planning/minnesota` and fresh output.
It wraps the installed actor programs, retaining the local conversations and
furnishings. It adds the earned bicycle, caught-only regional research awards,
restoration Master Ball handoff and missing Machine Part recovery. The existing
Minneapolis workshop sign changes text without adding another background event.

Preflight and comparison evidence is retained under
`planning/minnesota/renders/interior-life/selected-union-v01/`. Build and
controller logs are under `target/interior-life/interior-union-*`.

## Preservation and native review

The strict comparison preserves all original objects/events, old map and spawn
identities, old numeric flags, native art/audio and unrelated game catalogs.
716 original complete map modules are exact. Only eight accepted route block
arrays, the single park road connection and four intended reward modules change;
the park adds one map. Every original conversation body is retained, including
the four wrapped actor roots. The existing workshop sign is the only replaced
text body. All 705 shared selected maps survive, and the complete new park and
accepted R09–R12 terrain match the shared selected pack. The local park's new
spawn is **289**, with native group/map **101/1**; do not copy spawn282 from
the older shared park receipt.

All 377 audio programs pass PCM/hash/frame/loop verification. The complete
repaired St Paul module remains exact. Three fresh headless native 640×576
2D frames—both clinics and the west river bend—match every source pixel without
exclusions and were personally inspected alongside the whole native city.
The old Red Wing station and visible-text scans remain empty on this child.

The registry/native indexed-port audit exposed 84 stale anchor points in 28
reservations: 18 Duluth exteriors had retained positions from before the
four-tile northward correction, and ten St Paul facades retained positions one
tile east of the accepted repaired doors. These metadata fields now match both
the actual indexed ports and the explicit Duluth town / St Paul facade sources.
No clear-floor arrival, index, warp, terrain, actor or game program changes.
The full second audit passes **2,502 checks with zero issues**. The original
registry snapshot, initial failed inventory and exact 84-point correction are
retained. Clear walking-floor arrivals remain distinct from indexed stair
arrivals; the Winona bedroom and department-store reservations were not blindly
rewritten from native warp arrays.

## Verification boundary

All **26 targeted production-controller cases pass** on this exact combined
pack: five room/resident/errand/Continue cases, five reward cases, three park
cases, seven current R12 cases, three city cases and three Staples cases.
The city cases cover 39 ordinary door pairs, both clinic/PC services, bridge
walks, Surf underpasses, river seams and Continue. The Staples cases include
the already-correct wall ledger, repeated facing, first foot visit and shop.
The retained publication receipt records each final log and duration.

The first R12 attempt used an older six-case binary and was stopped. The second
used the pre-correction driver and failed two fixtures on unexpected encounters.
The existing corrected driver (SHA-256
`4fb8c02132a23c9dc7973378537c0ebc1e2bee40ad6d8e2e361c1bd4550c49d5`)
passes all seven cases. Those failures remain retained. No terrain or gameplay
guard was weakened. This composition changes JSON contracts/planning metadata;
it introduces no new Rust source change or new Rust lint exception.

Selection was revalidated against `route_grass_r12_revision` immediately before
publication; no newer accepted revision was replaced. Reread `world.json` before
selecting any later child so newer accepted work is kept.
The five-stop passenger service, International Falls regional circuit and
remaining whole-game room/story acceptance remain open. No gym is authored by
this composition, and no new voxel work is included.
