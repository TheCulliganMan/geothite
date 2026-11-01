# Clear, badge-paced Minnesota HMs

Current compact-lineage work is recorded immediately below. Older receipts
remain historical and do not establish current acceptance.

## Current compact-lineage candidate

`content-packs/minnesota-current-hm-r15-v01.crystalpack` is the selected current local review pack
built from the selected R15 habitat/cyclist pack, rather than the withdrawn
historical lineage below. Four native controller cases pass all seven badge
denials, actual gifts, repeats, Save/Continue, the Brainerd report guard,
visit-only Fly unlocking and an actual safe Fly landing. Nineteen regional
Fly destinations are additive; original spawns and Fly points remain exact.

A strict comparison confirms unchanged terrain, warps, actor geometry,
encounters, trainer parties, rod tables, field rules, items, art and audio data.
The current production client builds and loads this pack. All 377 audio
programs pass PCM/frame/loop verification; three rod-service, four habitat,
three cyclist and five reward cases also pass on this same pack. A real
fishing bite passes through the production stdio client with empty stderr. Gift fixtures do not
prove an earned full campaign or actual field use of every awarded HM.

## Historical lineage

**Historical publication withdrawn pending compact-geography rebase.** The former
`minnesota-field-progression-v03.crystalpack` inherits obsolete northern geometry.
Do not use it or its furnished-home parent for new builds. See the
[fishing lineage audit](fishing-rods.md#geography-status-historical-lineage-withdrawn)
and the [canonical compact geography](../revisions/superior-gunflint-quality-v01/VERIFICATION.md).
The receipts below describe historical gameplay validation, not current
compact-lineage acceptance.

The installed `field_moves` table is the permission authority. Minnesota uses
the existing Johto badge slots with regional names; no permission is inferred
from a gym's type. The source is [hm-progression.json](../hm-progression.json).

| Field permission | HM giver | Chapter |
| --- | --- | --- |
| Care, badge 1 | Hollow guide on Driftless Trail, `MnR01Cave1F` | Rochester |
| Prairie, badge 2 | Tool keeper in the Prairie Farm Road shed, `MnR02ToolShed` | Mankato |
| Current, badge 3 | Mill foreman's home, `MnMinneapolisMillForemanHome` | Minneapolis |
| Capitol, badge 4 | Angler House host, `MnStPaulAnglerHouse` | St Paul |
| Wind, badge 5 | Depot guide in `MnStCloud` | St Cloud |
| Pine, badge 7 | Pier fisherman in `MnBemidji` | Bemidji |
| Harbor, badge 8 | Lake Outfitter host, `MnDuluthLakeOutfitter` | Duluth |

These grant Flash, Cut, Strength, Surf, Fly, Whirlpool and Waterfall,
respectively. Each giver checks the matching badge before a free reusable gift,
checks actual ownership to avoid duplicates, explains teaching through PACK's
TM/HM pocket, and names the field-use badge. Failed gifts remain retryable.
The original actor's scene/service follows the handoff. Gym guide text, leader
victory text, gift text and repeat text name the giver/location and field badge.
There is no new HM at badge 6; Surf belongs at badge 4 under the native rules.

The current parent previously supplied Cut at Bemidji and Surf at Brainerd,
with no regional gifts for the other five HMs. The older separate all-TM/HM
shop overlay was not in this parent. If applied to a lineage with bulk regional
HM sales, this stage removes only HMs from `MART_MN_*` stocks so badge-three
shopping cannot bypass the intended distribution. TMs and other regions survive.

Brainerd's existing Surf/report scene stays behind the Lake Badge and its
acknowledged testimony. An earlier Surf HM does not set report/gift story flags
in St Paul. At the original Brainerd grant point, an ownership branch skips
only the duplicate item, sets the original flags after the actual report, and
continues the original repeat scene. Bemidji's Cut fallback already checks
ownership and is retained. Native field-use rules, teaching compatibility,
encounters, terrain, trainer parties and audio are not changed by the HM stage.

Implementation is `build_minnesota_hm_progression` and the ignored-output Rust
publisher `pack_minnesota_hm_progression`. The Rust source check passed. Local collision checks connect all seven public
approaches to real doorway/return anchors without using an HM; the Duluth
outfitter is reached from (5,5), west over the counter at (4,5).
Publication and verified readback passed for
`content-packs/minnesota-field-progression-v03.crystalpack` (716 maps, 475 species).
All five production HM controller tests passed: all seven badge denials, free
gifts, repeats and Continue; Brainerd report compatibility; town-visit unlocks;
menu labels; and a real St Cloud-to-Winona flight with native input ownership,
unvisited-destination exclusion, exact safe landing and reusable HM. These are
explicit local location/party/badge fixtures, not an earned full-region journey.

## Publication selection

The publishers accept an explicit caller-supplied parent; neither selects a
North Shore source or default pack internally. The former commands targeting
shore-gate/furnished-home/v03 parents have been withdrawn because they propagate
stale exterior geometry. A fresh command and acceptance receipt will be added
only after the compact parent and furnished-home rebase are verified. Do not
reuse the old command from shell history. Historical packs/renders are retained.

## Fly destinations

The parent has an empty Fly catalog. This stage also registers all 18 installed
Minnesota hubs, using existing dry public doorway returns or clear doorsteps.
The planned Mille Lacs town has no installed map and receives no invented point.
Each real town has a `MAPCALLBACK_NEWMAP` that unlocks only its own Fly flag.
Future northern towns are not unlocked by receiving HM Fly or by the Wind Badge.

Fly points reuse safe existing town spawns; towns without one receive a registered
additive spawn identifier starting at 512. Original
source/home/blackout/Teleport spawns are preserved. The Fly catalog validators
now accept the existing runtime's additive spawn range while retaining the
invalid source-byte gap 28–255 and requiring a real spawn and landmark. Fly menu
labels use the actual Minnesota town name, including towns whose older native
landmark identifies the surrounding region. Native destinations retain their
existing labels and behavior.

Source/data tests cover every visit callback and retained original spawn, plus
reserved-range rejection. Controller cases cover actual town exit/visit unlock,
unvisited-destination exclusion, the Start/Party/Fly menus, a real St Cloud to
Winona flight, the exact dry landing, HM reuse and Continue. All five controller cases passed against the published HM child.

The renderer-neutral flight regression found that the graphical Fly animation
clock was not advanced by `VisibleShellController`. The native departure/arrival
sequence now lives in `bevy_shell/field_fly.rs`, shared by both hosts. The controller
locks directional input during flight and commits travel at the same departure
boundary. The regression requires the real destination menu, input ownership,
travel commit and landing rather than accepting a Fly HUD or pending command.
The updated controller regression passed with this shared clock.

The withdrawn v03 file SHA-256 is
`5188b167369d29c28fb87527c6df77006ec5948df9020c7411d61dfbd267e132`.
Both asset regressions passed against the external parents, including unchanged
field-use rules, items, encounters, trainers, runtime source files and audio for
the HM stage, all 27 fisherman hints, the three protected house footprints,
main-corridor foot access and preserved original recovery/home spawns.
The original native Fly destination ordering/default regression also passed.

The v02 child shortens HM denial lines and rejects newly authored handoff rows
longer than eighteen characters. It passed verified publication, the HM asset
preservation regression, all five controller tests, and the real stdio MCP
movement/Start check with empty stderr. A direct comparison to v01 confirms all
716 map geometries, actors, events and non-dialogue data remain identical.

The withdrawn v03 publication used the completed protected gate-v04/furnished-home
parent `minnesota-interiors-fishing-life-v02.crystalpack`. Verified readback,
all eight rod/HM controller cases, HM preservation and the real stdio MCP check
passed again on v03. The three furnished guru maps are byte-identical to that
parent, and reviewed exterior geometry/actors/events are unchanged. All 28 gym
guide/victory/gift/repeat texts identify the HM, giver/location and field badge.

The historical stdio session used the withdrawn v03 output; it is not a
current launch recommendation.

Final logs include `/tmp/geothite-rods-composed-final.log`,
`/tmp/geothite-hm-composed-final.log`,
`/tmp/geothite-field-composed-preservation.log`,
`/tmp/geothite-field-mcp-composed.log` and the `current-v04` paired exterior
captures under ignored `renders/fishing-rods/`. Existing formatting/lint
failures are recorded in the fishing runbook; the historical gameplay, publication, preservation, Fly-range and native model
checks passed. Compact-lineage acceptance remains unverified.

The final roster audit confirms 62 party members across all 27 fishermen use
active original Minnesota species. Native 2D furnished interiors and paired
exterior model captures were reviewed; the HM composition preserves them.
