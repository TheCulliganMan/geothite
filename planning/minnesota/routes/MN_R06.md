# Current garden/pine revision

This route now matches the471-map `minnesota-canopy-review.crystalpack`. See
[the current sight, preservation and native-input proof](../playable/garden-and-pine-revisions.md).
Earlier source/native milestones below are retained as history.

# Pine Country Road — hand-authored source review

This replacement is **32×44 metatiles**, two exteriors and one 5×4 recovery cabin. It replaces the old 64×120 repeated district reserve. Needle Lake fills the western silhouette; its outlet forces a timber crossing before the road follows the eastern pines. A real Cut gate seals the optional lake-view shelf, its Ether and Hiker. The camp, fishing pier, conifer fork and one-way lookout return offer distinct activities.

[Whole route before Cut](../renders/pine-country/whole-route-2d6709de.png) · [whole route after Cut](../renders/pine-country/after-cut/whole-route-8a783b9d.png) · [Cut gate before/after at player scale](../renders/pine-country/cut-gates.png) · [twelve walking player views](../renders/pine-country/viewpoints.png) · [two Surf-boundary views](../renders/pine-country/surf-boundaries.png) · [camp cabin](../renders/pine-country/interiors.png) · [post-Cut ledge return](../renders/pine-country/after-cut/ledges.png).

Current evidence establishes **source pixels and static before/after collision**. Native gameplay and final visual acceptance remain open. Root has now published the native overlay; exact source geometry matches. Production gameplay verification is still in progress.

## Player trip and physical contracts

Terrain coordinates are zero-based 32px metatiles; the table uses 16px gameplay tiles. Exact strokes, actor identities, parties, dialogue and room rows are in [the layout](MN_R06.layout.json) and [the dossier](MN_R06.json).

| Place | Authoritative contract |
| --- | --- |
| St Cloud approach | South gate `(16,84)`, inward landing `(16,80)`. The forest throat spans core habitat `[12,76,22,80)`. Every ordinary endpoint journey crosses at least four grass tiles. Both arrivals have clear floor. |
| Needle Camp | Cabin door `(9,61)` is the actual southeast DOOR quadrant of source `16`. Safe return `(9,62)` is ordinary FLOOR with separate passive warp index 2. Camp board, warden, optional Bug Catcher and Super Potion occupy short nearby approaches. |
| Recovery cabin | `MnR06NeedleCabin`, native template `VioletNicknameSpeechHouse`, house art, 5×4 explicit rows. Carpet exits `(5,7)` and `(6,7)` both return safely; clear arrival `(5,6)`. Keeper `(5,3)` has `service: heal_party`. Repel `(7,3)` has persistent item flag 03. Restoration and reciprocal entry still require native execution. No Cut is needed to reach recovery. |
| Outlet bridge | Deck metatiles x=13–18, y=33; full-floor core rows y=66/67. North rail y=32 has WALL above/FLOOR below. South rail y=34 has FLOOR above/WALL below; water remains directly below at y=35. Removing the deck and inner rail walking cells disconnects the endpoints. Lake/outlet reach the western/bottom boundaries, preventing a dry dam bypass. |
| Conifer fork | Perry `(45,55)` faces east with sight 2, controlling `(46,55)` and `(47,55)`. Forest and a western rail constrain the short lane. The outer side of the conifer island requires at least six grass steps through core `[52,54,58,60)` in either direction when avoiding his sight. Great Ball `(57,53)` has clear footing within that habitat. The ordinary forest divider is source `05` WALL, not an advertised Cut gate. Native trainer approach movement still requires testing. |
| Needle Lake pier | Westward deck metatiles x=16–19, y=21, reached from the eastern road. Source rail recipes frame both sides. Native gray end `44` blocks `(30,43)` and leaves `(31,43)` as the inner footing. Water below the south rail remains WATER. Angler `(33,43)` leaves a second deck lane. Selected eastern bank edges use real wall rails; forest controls other approaches. |
| Bird glade | Optional habitat beside the northern road and Bird Keeper `(51,31)` give a separate encounter location rather than extending mandatory grass through every segment. |
| Lookout Cut gate | Source `5b` at metatile `(14,16)`, CUT_TREE at `(29,32)`, HEADBUTT_TREE at `(28,32)`, approach `(29,33)`. Actual native `5b → 3c` replacement opens `(29,32)` to FLOOR. Existing rock borders and one-way ledge close all alternative walking entrances. The main road, camp and pier remain available before cutting. |
| Lake-view shelf | Native ascending ramp `74` remains at metatile `(14,15)`, immediately above the Cut gate. Ether `(13,27)` draws the player to high edge `(13,29)`, whose native camera shows 44 WATER core cells. Eli `(27,23)` supplies the optional Hiker challenge. All three are intentionally behind Cut. |
| Lookout one-way return | South-facing `56` spans metatiles x=10–11, y=15. Approach `(21,29)` enters HOP_DOWN `(21,30)`, lands `(21,32)`; reverse `(21,31)` is WALL. Side closures `(19,31)`/`(24,31)` are WALL. After cutting, the remaining Headbutt tree makes the ordinary uphill return **21 core steps** through required ramp passage `[28,30,30,32)`, saving 18 walking steps by taking the drop. |
| Brainerd approach | North gate `(42,2)`, inward landing `(42,6)`. Pine shoulders narrow the approach. Watcher `(41,9)` stands off the road. |

Five trainers have eight opposing Pokémon: Fern's Pineco 23/Ariados 23, Perry's Oddish 23/Growlithe 24, Nora's Slowpoke 24, Beck's Pidgeotto 24/Noctowl 24, and Eli's Stantler 25. Native-width dialogue and defeat flags 01–05 are explicit. Former trainer reservations 06–08 remain unused. Three outdoor residents, six signs and three outdoor pickups join the indoor keeper and Repel.

## Cut gate and progression

The applied `cut_gates` record is:

```json
{
  "id": "MN_R06_LOOKOUT_CUT_GATE",
  "tile": [29, 32],
  "approach": [29, 33],
  "replacement_block": "3c",
  "targets": [[13, 29], [14, 27], [26, 23], [21, 29]],
  "gated_actor_ids": ["MN_R06_T05", "MN_R06_ITEM_04"]
}
```

`shortcut` remains null: this is an optional-area gate, not a second shortcut or decorative tree. The checker requires exactly the declared CUT_TREE quadrant; an extra tree fails review.

| Target | Before Cut | After native replacement |
| --- | --- | --- |
| Brainerd gate `(42,2)` | Reachable | Reachable |
| Camp return `(9,62)` and recovery approach | Reachable | Reachable |
| Pier inner end `(31,43)` | Reachable | Reachable |
| Cut approach `(29,33)` | Reachable | Reachable |
| Lake-view edge `(13,29)` | Sealed | Reachable |
| Ether interaction approach `(14,27)` | Sealed | Reachable |
| Eli interaction approach `(26,23)` | Sealed | Reachable |
| Ledge upper approach `(21,29)` | Sealed | Reachable |

The ordinary walking flood reaches **1,489 tiles before** and **1,684 after** cutting: 195 optional-area tiles open. Every existing actor except the explicitly gated Hiker/Ether retains a pre-Cut approach. Every sign and the recovery cabin remain approachable. All four target points are unreachable before and reachable after the actual catalog mutation. Post-Cut ordinary traversal also verifies the preserved ramp and the meaningful 21-step ledge return.

The actual supplied pack requires party-known **CUT** and Johto badge **index 1**, the second badge slot. R06 separately retains its `BADGE_5` regional entry condition and Q04 dependency. Q04 does not guarantee a Cut teaching-item award; this optional area therefore cannot become required for either the main journey or recovery. The keeper does not award badges or complete Q04. Real hub gates, badge enforcement, Minnesota recovery and earned progression remain regional integration work.

Source replacement is not evidence of native Cut execution. Native tests must independently deny a missing move and a missing badge without mutation, then perform actual removal through production input, walk through the opening, interact with the previously sealed targets, and verify the intended block-override persistence after leaving/returning and save/Continue. A fixture spawned inside the lookout does not prove the gate works.

## Surf and outer-water boundaries

The west lake and southern outlet previously touched unconnected outer map edges. Those openings are now visibly closed with unchanged **native blue-water rock recipes**, inspected both as source pixels and collision. Johto `34` has BUOY along its left column/WATER on the right; `39` has WATER above/BUOY below. Source `30`, `38` and `3a` round the corners. These are the native white-and-blue water rocks, not brown land boulders placed in water.

| Boundary | Exact closure |
| --- | --- |
| West Needle Lake | Metatile `(0,16)` uses `30`; `(0,17)` through `(0,21)` use `34`; `(0,22)` uses `38`. All outer core x=0, y=32–45 are BUOY. |
| South outlet | Metatile `(14,43)` uses `38`; x=15–17, y=43 use `39`; `(18,43)` uses `3a`. All outer core y=87, x=28–37 are BUOY. |
| Internal water seam | Core x=2–39 have matching WATER across y=47/48 between `MnR06C00R00` and `MnR06C00R01`. This internal seam is preserved. Native publication must supply actual reciprocal south/north connections, offset 0. |

The five rock recipes are in `additional_water_blocks`, already supported by the Rust layout compiler. Shore fitting therefore recognizes their blue-water portions and does not draw a false brown bank around the rocks. The cabin's former Python-only `house` stroke is also replaced by the identical four explicit native points (`18 19` above `16 1e`), preserving its source pixels and exact DOOR quadrant without a new Rust shape.

The checker scans **all 300 outer core tiles** and independently floods conservative walking-plus-Surf access in both Cut states. Each flood reaches **1,000 WATER tiles** and **zero outer-edge WATER tiles**; the perimeter has **24 actual BUOY cells** at the closures. The lookout's targets stay sealed even when Surf is allowed before cutting. All main walking, recovery, bridge, pier, grass, trainer-choice and post-Cut ledge contracts continue to pass.

Surf camera probes `(5,39)` and `(32,83)` visibly show the barriers within native 160×144 framing. For a published R06 pack, the checker additionally requires real reciprocal native map connections at the open internal seam. The source/base-pack review explicitly reports that native-neighbor verification is pending; it does not treat a planned connection as an implemented one. Seven invalid water revisions were rejected: west/south gaps, reversed west/south rock orientation, a brown land-rock substitute, missing water-fitting declarations, and sealing the internal water seam. A separate synthetic-metadata check rejects a missing reciprocal native connection; that test is not native publication evidence. Production Surf input, collision against the rocks and crossing the actual internal seam remain native verification work.

## Source review and fast revisions

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_pine_country.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_pine_country.py --serve
```

The live source review is `http://127.0.0.1:8776/`. It watches both JSON files and regenerates complete before/after maps, player crops, cabin, anchors, Cut comparison and post-Cut ledge evidence without Cargo. Earlier single-state warm renders took 164–187ms; the complete two-state render/check measured 313–343ms. Invalid geometry is visibly marked invalid.

The exterior uses **40 metatile varieties**, original source pixels/palettes and existing `mn_headwaters` recipes 80–82. Native references studied by sight were Route30's clearing rhythm/tree borders, Route42's shelf/water boundaries and the Johto atlas. Geography consists of explicit hand-selected strokes; no random placement or copied source map is used.

Whole-map and player-view revisions removed an empty eastern tail, reduced the brown shelf, closed a southern grass bypass, corrected the actual facade door quadrant, moved a watcher off a tree and a pickup off grass collision, and replaced the broad lower road with a meaningful battle/grass fork. Root's visual critique prompted tighter forest shoulders and relocation of the inland shelf onto a visible lake cove. The Cut tree now seals that whole optional place below its real ramp, preserving the established ledge rather than placing a removable tree beside an open road.

Static checks cover both Cut states, all actor/room approaches, legal text/classes/species, required grass, both directions around the conifer fork, necessary bridge, rail/end/water collision, lake visibility and native ledge return. Seven deliberate Cut failures were rejected: a removed gate, a wrong catalog replacement, a wrong quadrant, a free western entrance, an unreachable post-Cut target, an extra decorative Cut tree, and an incomplete gated-actor contract. These checks establish physical contracts; they do not establish fun or final visual acceptance.

## Integration handoff

Root owns shared Rust registration and pack publication. Proposed specification:

- Group **51**, `MINNESOTA_PINE_COUNTRY`; manifest `minnesota-pine-country`; music source `Route37`; compiled tileset `mn_headwaters`, unchanged Johto plus existing 80–82 recipes. Its native field-move behavior must inherit the Johto `5b → 3c` rule.
- `MnR06C00R00`: 32×24; `MnR06C00R01`: 32×20, origins `(0,0)`/`(0,24)` metatiles. One reciprocal north/south seam; the main lane crosses global core x=50/51, y=47/48. Remove the obsolete second column and rows 2–4.
- South gate: C00R01 local meta `(8,18)`, landing `(8,16)`; north: C00R00 meta `(21,1)`, landing `(21,3)`. Preserve ST CLOUD/BRAINERD identities and `BADGE_5` when replacing world contracts.
- Room `MnR06NeedleCabin`, conventional constant `MN_R06_NEEDLE_CABIN`, with native metadata, parsed healer script, two safe return exits and persistent Repel. Outdoor door: C00R01 local core `(9,13)`, passive FLOOR return `(9,14)`.
- Typed route contract: `[32,44]`, one column/two rows; five trainers, three outdoor residents, three outdoor pickups, six signs, one door/room, one required grass crossing, one ledge, one battle/grass choice, one `cut_gates` record. Explicit gated actors need pre-Cut exceptions backed by full sealed-target/post-Cut proofs; all remaining actors stay initially approachable.

After publication, `--verify-published` compares the actual pre-Cut authored blocks/events/rooms with the pack. Inspect headless native 2D before/after captures at equal position/time/facing/state. Execute seams, crossing/rails, pier end, Cut prerequisites/removal/targets/persistence, ledge/rejected reverse/full return, trainer battles and move PP, wild habitat, collected medicine, keeper healing, both cabin exits and save/Continue through `VisibleShellController`. Verify MCP, clean stdio, TUI and unchanged audio hashes/frame counts/loops. Explicit party, badge and spawn fixtures do not prove earned regional progression.

## Native publication evidence

Root published `content-packs/minnesota-pine-country.crystalpack` with the Rust `pack_pine_country` builder. `review_pine_country.py --pack content-packs/minnesota-pine-country.crystalpack --verify-published` passes: 3 authored maps and 1408 exterior metatiles match the native pack, along with outdoor objects and room rows where present. Open water seams have actual reciprocal offset-zero native neighbors. Controller Cut/Surf inputs, populated battles and earned regional progression remain separate verification work.

## Production field-move results

The combined `content-packs/minnesota-northern-lakes.crystalpack` contains 401 maps (R05–R08 plus the preserved source). Six production-controller field-move regressions pass: all three Cut gates independently reject missing CUT and a missing Hive badge, perform the actual catalog replacement, allow walking to the sealed reward, and preserve the item/flag through Continue. Native temporary vegetation resets on Continue; cutting again from inside permits exit. R05/R06/R07 mount Surf through A/Yes, reject movement against real BUOY quadrants, and cross each actual internal river/lake seam in both directions. This explicit Mew 60/position/badge fixture proves mechanics; it does not establish route battle balance or earned regional progress.

```sh
CRYSTAL_CUT_ROUTES_TEST_PACK="$PWD/content-packs/minnesota-northern-lakes.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib cut_gated_routes -- --ignored
```

The new route composition/source-preservation regression passes, including rejected fake gate targets, replacements, unknown actors and incidental decorative Cut trees. All 377 audio PCM hashes/frame counts/loop ranges match in the combined pack. Full populated journeys, R06 keeper/room service execution, MCP/TUI coverage for these new routes and final visual acceptance remain pending.

## Prepared populated production journey

A focused new leaf, `crates/crystal-bevy/src/bevy_shell/tests/pine_country.rs`, prepares `pine_country_controller_populated_five_trainers_cabin_resources_cut_return_and_continue`. **This is prepared coverage, not a passing controller result.** Root owns its shared registration, warm build and run; this author did not build or export a pack. The concrete supplied `content-packs/minnesota-pine-country.crystalpack` currently contains395 maps and retains the existing default wild-challenge rules. The same test can consume the completed combined northern-lakes pack through its explicit environment variable.

Initial setup creates a legal level60 Mew with Psychic, Thunderbolt, Cut and Surf, five Johto badge bits, an injury60HP below maximum and poison, plus one deterministic divider stream installed before inputs. This deliberately strong review partner is not earned progression or a route-balance claim. No position, party, inventory, flag, RNG or map override is rewritten after setup. The existing native battle policy remains enabled, so ordinary wildlife may scale from its authored level according to the pack's party-based policy.

The prepared journey requires:

- Actual walking from the southern gate through required habitat, at least one real native wild encounter and production RUN. If the initial crossing rolls none, bounded repeated walking in that same grass uses the continuing RNG stream rather than injecting a battle.
- All five authored trainer challenges and eight opposing species, with real selected Psychic/Thunderbolt PP spent and experience awarded per victory. Exact trainer identities and all five defeat flags are checked; the final post-Continue Beck interaction must remain dialogue only.
- Super Potion pickup `(23,73)`, actual Start→Pack→Use→party consumption and native50HP restoration. No service is substituted for item use.
- Real cabin entrance `(9,61)`, walking onto clear interior Floor, actual keeper `(5,3)` `HealParty` restoring HP/status/every move's PP, persistent Repel `(7,3)`, both carpet exits `(5,7)`/`(6,7)` returning to safe `(9,62)`, and passive exterior Floor without bounce.
- The necessary outlet crossing, actual conifer-fork/pier approaches, Great Ball, native Cut into the formerly sealed lookout, Eli's challenge and Ether. Actual ledge landing `(21,32)`, blocked reverse and21-step ordinary ramp return are retained. Any native vegetation reset is handled by another real Cut interaction, never an override restoration.
- A real Save/Continue preserving exact party, bag, location, badge state, all trainer flags and all four pickup flags. The consumed potion remains gone while Repel, Great Ball and Ether persist exactly once.

The existing conventional single-mouth cabin currently enters native warp1 at `(5,7)` on its carpet; the declared clear Floor `(5,6)` is reached with normal input. It does not yet declare a separate passive indoor arrival record like the newer Boundary Waters cabin. This test asserts the actual published record and proves safe entry/exit, rather than silently claiming the inbound warp targets Floor. Any genuine runtime bounce/blocking defect must be repaired before declaring the room playable.

Read-only planning against the actual published blocks confirmed every pre-Cut interaction approach in the itinerary, including Perry from `(45,56)`, Nora from `(42,43)` and Beck from `(51,32)`. The actual post-Cut native collision gives the expected21-step walking return. Native map constants in this older overlay are the same PascalCase strings as their map names; its warp records match those actual metadata keys, unlike the newer preserve's uppercase constants. No arbitrary renaming was applied.

Actual headless production2D captures were generated with the existing compiled renderer and inspected before any terrain proposal: [busy conifer fork](../renders/pine-country/native-conifer-fork.png) at global `(47,59)` and [quieter northern-road camera](../renders/pine-country/native-northern-road-weak.png) at `(43,19)`, both noon and ordinary pre-Cut state. The fork's rail/forest divider and grass reward read clearly. The northern segment has a narrow connected lane, visible forest lobes and a habitat at the next bend; this audit found no unambiguous terrain defect requiring a source change. These explicit-location empty-party render fixtures do not prove gameplay or final user acceptance.

After Root registers the leaf, the intended warm test command is:

```sh
CRYSTAL_PINE_COUNTRY_TEST_PACK="$PWD/content-packs/minnesota-pine-country.crystalpack" \
CRYSTAL_PINE_COUNTRY_JOURNEY_SAVE="$PWD/.local/pine-country-populated.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy pine_country_controller_ -- --ignored --test-threads=1
```

The temporary Continue save uses a unique process/time name and is removed. The optional journey-save environment retains the actual final review state for later MCP/native inspection only after successful completion. Scoped `rustfmt --check` passes. Existing Cut/Surf denial/removal/reset/seam tests remain independent proof; earned BADGE5/Q04 progression, regional recovery, balance and final visual acceptance remain open.

### First populated run: fixture move identifier corrected

Root's first warm execution in `.local/pine-country-populated-current.log` stopped during initial party construction, before gameplay: the fixture used the displayed Psychic name `PSYCHIC` as a catalog key. The actual native move identity is `PSYCHIC_M`; the battle/party model uses that same identity while presentation displays `PSYCHIC`. Both initial learned-move construction and preferred-attack matching now use `PSYCHIC_M`. Read-only inspection of the actual395-map pack confirmed `PSYCHIC_M`10PP, `THUNDERBOLT`15PP, `CUT`30PP and `SURF`15PP; the other fixture keys already match. No engine rule, source move, initial learned legality, battle policy or post-setup state is changed. Scoped `rustfmt --check` passes. The populated journey itself remains unexecuted pending Root's rerun.
