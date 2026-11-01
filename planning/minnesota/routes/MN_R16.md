> Current grass review: see [Red River verification](../revisions/route-grass-v01/current-r16-v01/README.md).
> Existing native terrain is accepted unchanged:1016grass walking tiles,6forced
> hayfield-climb grass steps, a dry ledge return and4forced main-route grass steps.
> Both native maps, all regional content and the ice-free North Shore stay intact.
> Local populated input/Continue, wildRUN, resources, battle PP/flags and stdio pass.

# Red River Prairie — hand-authored source review

## Current executed native evidence — 2026-10-26

Root published `content-packs/minnesota-red-river-prairie.crystalpack` with
**415 maps**. Exact comparison passed both authored maps, **1,152 metatiles and
17 outdoor objects**. All three current asset preservation/rejection tests pass.
The actual native seedbed capture was inspected; whole-route visual acceptance
remains separate.

Both production-controller tests **PASS** in the actual warm run
(`.local/red-river-controller-current.log`): empty-party geometry across both
clipped maps, bridges/rails, required habitat, ledge/reverse/ordinary return,
actors/resources, Start and Continue; and the populated Camper Beck battle with
selected PP, money/defeat state, actual post-victory walking and Continue.
The populated fixture supplies its legal party, badges and position initially;
one Camper victory is not all ten battles or earned SQ02/regional progression.
Current integration audio evidence is shared: the **443-map** composition passes
all **377** PCM hash/frame-count/loop-range checks. No redundant audio verification
was run for this documentation update. Older pending statements below describe
the original handoff and are superseded by these executed results.


The route is a horizontal farm-country crossing: the Red River, a smaller drainage channel, a hay meadow above a low bank, working seed and flower plots, and a cottonwood grass neck. It reuses unchanged native Johto art and the already implemented `mn_headwaters` post-rail recipes. No map generator, copied route geography, painted sprites or new asset catalog is involved.

**Current source proposal:** [whole route](../renders/red-river-prairie/whole-route-a8b52fde.png), [all player cameras](../renders/red-river-prairie/viewpoints.png), [directed return](../renders/red-river-prairie/ledges.png), [anchors](../renders/red-river-prairie/anchors.png). These ignored images are source evidence. Native R16 publication, exact source comparison and both focused controller fixtures have passed as recorded above; earned regional progression and final visual acceptance remain pending.

## Physical and integration contract

| Property | Exact value |
| --- | --- |
| Exterior | 48×24 metatiles; 96×48 gameplay tiles; 1536×768 source pixels |
| Native maps | `MnR16C00R00`, `MnR16C01R00`; western 32×24, eastern 16×24 metatiles |
| Native art | `mn_headwaters`; original Johto 00–7f plus existing native source-row recipes 80–82; no additional art requested |
| Detroit Lakes entry | east gate meta `(46,12)`, arrival `(44,12)`; eastern map local gate `(14,12)`, arrival `(12,12)` |
| Moorhead entry | west gate meta `(1,12)`, arrival `(3,12)`; western map local coordinates match global |
| Whole exterior walking edges | west core `(0,24/25)` and east core `(95,24/25)` only; remaining outer edge collision blocks walking |
| Internal neighbor | western map east ↔ eastern map west, offset 0; actual reciprocal native records required before acceptance |
| Story contracts | original `BADGE_6` and `SQ02` retained; no new earned badge or completed quest claim |
| New interiors | none; no false door, decorative cabin or service facade |
| Group/IDs | integration owner reserved group56, `MINNESOTA_RED_RIVER_PRAIRIE`, map IDs1/2; retain HOME and all recovery spawns |

The old 96×72 nine-chunk district lattice is replaced, so the shared world must retain only these two chunks and these ports. The integration owner reconciled the revised48×24 ports and world reservation before publishing the current native slice; the rejected64×24 draft is historical. All ten original `MN_R16_T01`–`T10` actor IDs, their parties/levels and `EVENT_MN_R16_TRAINER_01`–`10` are retained. All four original pickup IDs, item identities and saved flags are retained. Positions, local dialogue and trainer classes now belong to actual features.

## Traversal and discoveries

- The Red River bridge crosses meta x5–13. Its native north post rail is row11/block82 and south rail row12/block81. Only core rows23/24 walk between the rails. Removing that crossing disconnects the two route gates in the complete walking/grass graph.
- The drainage bridge is visibly shorter: meta x30–33 across a two-metatile channel, with its north rail at row14 and south rail at row15. The approach meets the farm lane at a different height. Removing this crossing also disconnects the endpoints; no grass, field or dry dam bypass exists.
- The fishing pier extends from the western bank **over the Red River**, through meta x3–8, rows7/8. Native corner fences42/4a at x9 close the end and its north/south corners. Fisher Lyle stands at core `(13,16)`. The player view at `(15,16)` sees water and the closed end together.
- The low hayfield bank uses native block57 across meta x16–27, row8. Its hop starts at core `(53,16)` from `(53,15)`, lands at `(53,18)`, and blocks reverse entry at `(53,17)`. The ordinary uphill return is **59 walking steps**, versus the three-tile directed displacement. Removing the sole western gate at core x30–31,y10–11 disconnects that return. Closed side anchors at `(31,17)` and `(56,17)` prevent a free uphill side opening.
- Camper Beck (`T07`) stands at `(29,25)` and looks north two tiles. Before defeat his body and sight seal the clear bridge apron. The northern grass alternative costs at least **two actual grass steps**. Removing its four-cell grass band while retaining the unbeaten trainer disconnects the route; after defeat the clear apron works without that grass band. This local choice is independent of the eastern required habitat.
- The cottonwood neck is a complete wall of trees except core x82–85,y18–27. Every ordinary route through that neck crosses at least **four tall-grass steps**. Optional windblown swales provide shorter cross-field movement at an encounter cost; the pale farm road stays two core tiles wide.
- A small stepped seedbed has an eastern working gate and reachable furrows. The southern pollinator habitat is an unfenced, grass-priced flower basin containing two trainers and Revive. The upper hay meadow contains Hyper Potion, the eastern prairie edge an Ultra Ball, and the western riverbank grass a Max Repel. Pickup approach points are checked against complete actor collision, rather than reserved image positions.

## Cut and water boundaries

**No Cut gate is authored here.** The old generic shortcut placeholder named no actual target or sealed opening. The current hayfield already has a meaningful directed return and both field gates are ordinary working entrances. A decorative tree would add no purposeful gameplay gate. Every placed native collision quadrant is inventoried: **zero `CUT_TREE` quadrants**. Ordinary canopy uses 5d/61/65 where needed; incidental 5b/5f/63/67 are not used.

The river and drainage channel have visible native blue-water rock caps, using 30/31/32 at their north ends and 38/39/3a at their south ends. Those IDs are included in `additional_water_blocks`, so source shoreline fitting retains WATER beside the rock glyphs. The complete outer perimeter contains **zero WATER/WHIRLPOOL/WATERFALL cells**. There are **38 actual WATER↔WATER pairs** across the internal x63↔64 chunk seam. Source review records these as requiring reciprocal native neighbors; it does not claim those map records exist before publication. Both endpoints remain walking routes; Surf is optional exploration, not required progression.

## Actual review loop and revisions

Read and applied the reusable map-authoring skill and user-guidance reference. Inspected the authentic Johto source atlas, native Route34/Route42, and the old R16 lattice by sight. Compared the resulting whole map, every player camera, the fishing pier, battle/grass apron and widest farm-road view.

The first whole source pass (`ea1a656b`) still had broad unpriced green shoulders, a false pier that ended before reaching water, and two similar waterways. The next pass narrowed the agricultural channel from four metatiles to two, extended and capped the pier over actual WATER, added intentional encounter swales and native farm gates, contoured the western cottonwood corners, and closed all outer walking edges except the planned ports. That pass added the sealed southern bridge-apron barrier and a proved trainer-versus-grass choice. The integration owner then rejected the still-empty widest camera and copied farm boxes; those images are not accepted. The current revision removes 16 metatiles of empty eastern reserve, replaces the two large boxes with a small stepped seedbed and an unfenced pollinator habitat, and brings safety fence rows, native hedges, encounter reeds and a real seedling work aisle into the actual road camera. A read-only largest-blank-ground search exposed another completely empty camera beside the drain; a deliberately shaped bird-survey reedbed now occupies that shore. The current automatically located emptiest view is core `(34,26)` and includes a trainer, cottonwood, grass shoulder and adjacent native fence rather than a whole screen of empty lawn. These changes are authored source changes, not edits painted onto rendered images. The largest uninterrupted blank-ground square is four metatiles, which is only a review aid; the actual whole image and worst cameras still require the integration owner's visual approval.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_red_river_prairie.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_red_river_prairie.py --serve --port 8786
```

The current warm watcher is running at `http://127.0.0.1:8786/`, process session40111, log `.local/red-river-prairie-review.log`. A measured complete warm rerender/check took **174.6ms** with source art cached. Watcher rebuilds preserve immutable whole-image hashes and maintain `revision.json` beside current terrain/anchors/player views.

The checked proposal has 32 native metatile varieties, ten trainers, three residents, four persistent pickups, five signs, twelve authored native camera views plus an automatically located emptiest-ground camera, two necessary bridges, one required habitat and one real directed return. The complete source/static checks pass, including actor approaches, dialogue widths, legal native class/species/item identifiers, bridge rail collision, no dry bypass, initial trainer sight alternative, native ledge return, Cut inventory and full Surf perimeter.

## Pending native proof

Root owns Rust schema/route registration, pack publication, source/native exact comparison, shared world/atlas updates and production captures. No competing exporter or Cargo build was run here. After publication, compare every block and event; exercise both directions of each internal seam, Surf movement against each water-rock cap, bridge rails and the closed pier tip, native ledge denial/hop/return, both sides of the trainer/grass choice and required habitat, trainer battles/PP, pickup absence after Save/Continue, and same-position/time/facing native 2D views. The `BADGE_6` entry condition and SQ02 regional story remain separate earned-progression requirements.

## Native implementation handoff

The new focused leaf [red_river_prairie_modpack.rs](../../../crates/crystal-assets/src/red_river_prairie_modpack.rs) exposes:

```rust
pub fn build_red_river_prairie_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> anyhow::Result<CompiledGamePack>
```

It delegates parsing, native art installation, event lowering, native map payloads, atomic publication, shared required-grass/battle-choice/ledge validation and pack verification to the existing authored-route publisher. Its additional read-only validation uses the resulting native collision:32×24 and16×24 clipped maps, no incidental Cut quadrants, no unconnected outer Surf water, only the four actual exterior walking cells, exact two-row protected decks, both crossings independently necessary, and the river pier's closed end/corners. It does not implement another map exporter, gameplay dispatcher, hub scene or earned progression shortcut. Invalid source returns an error before a pack is returned; the supplied base remains borrowed.

[red_river_prairie.rs](../../../crates/crystal-assets/tests/red_river_prairie.rs) has three focused native compilation tests. The positive test checks both split map payloads, all old maps/runtime files/metadata/tilesets/trainers/items, unchanged audio and original spawn table/HOME/recovery, exact actor coordinates/IDs, native trainer parties/flags/level-derived moves, item-ball IDs/flags, encounter presence and the original128 native metatile byte recipes. Negative tests mutate actual source geometry: a river opened to the outer edge, incidental Cut canopy, missing rail, open pier tip, a bypassed drainage crossing, missing required habitat, missing native hop, an invalid ordinary ledge return and an unpaid path below the unbeaten Camper. These are static native publication tests; no controller movement is claimed.

Initial native implementation edited only these two new Rust leaves and this route runbook; the subsequent native preflight correction also fixed the dossier hop/camera anchors and its owned source checker. Root wired the following exact shared registration changes:

1. In `lib.rs`, add `mod red_river_prairie_modpack;` and `pub use red_river_prairie_modpack::build_red_river_prairie_modpack;`.
2. In `authored_route_layout.rs`'s supported layout match, add `"MN_R16" => ([48, 24], "mn_headwaters", "johto_8_neighbors")`.
3. In `authored_route_design.rs`'s dimension/event-count match, add `"MN_R16" => ([48, 24], 2, 1, 10, 3, 0, "MnR16")`. Keep four items through the existing default; add `"MN_R16" => 5` to the sign-count match. The dedicated R16 branch must retain `required_flags == ["BADGE_6"]`, one required grass crossing, one native ledge, one battle-or-grass choice, no Cut gates, no shortcut and no interior. Preserve the existing common actor/item/party/door validation.
4. In `authored_route_modpack.rs`, add the R16 RouteSpec below, and include `"MN_R16"` in the branch that calls `headwaters_art::build_headwaters_art_pack` when that art is absent. The integration owner explicitly reserved group56; do not race R09/group54 or R10/group55.

```rust
"MN_R16" => RouteSpec {
    group_id: 56,
    group: "MINNESOTA_RED_RIVER_PRAIRIE",
    manifest: "minnesota-red-river-prairie",
    landmark: "LANDMARK_MN_RED_RIVER_PRAIRIE",
    label: "MN_RED_RIVER_PRAIRIE",
    name: "RED RIVER PRAIRIE",
    position: [40, 64],
    tileset: "mn_headwaters",
    music_source: "Route34",
},
```

The two native maps are group56/map1 and group56/map2, with one reciprocal east/west connection at offset0. Regional hub connections remain pending. The proposed Pokegear marker `(40,64)` locates this westward route without changing existing landmarks; Root owns final regional chart binding.

Scoped `rustfmt --edition 2024` completed for both new leaves. No cold compilation or shared-cache build was run by this author. Root warm-ran `cargo test --profile wasm-dev --locked -p crystal-assets --test red_river_prairie`; the default source is the actual ignored `content-packs/text-tui.crystalpack`. A different completed source without R16 can be supplied with `CRYSTAL_RED_RIVER_PRAIRIE_BASE_PACK`; its path is canonicalized before verified reading. All three asset tests, native file publication, exact source comparison and audio verification passed as recorded below. Controller execution, whole-route native visual judgment, earned regional progression and applicable Clippy/workspace-format checks remain pending.

Native preflight correction: the first warm native build rejected `HAYFIELD_BANK` because the former `(45,15)` approach was actual TALL_GRASS, while the shared native hop validator requires FLOOR. The hop face `(45,16)` was already native57/HOP_DOWN. The dossier now selects an existing clear part of that same bank: approach `(53,15)`, hop `(53,16)`, reverse wall `(53,17)` and landing `(53,18)`. No source terrain or collision was changed to pass validation. The bank review camera moved to the corrected approach; the whole render remains `a8b52fde`. Complete source review still passes, with a59-step return/56steps saved and the same sole western return gate. The integration owner completed the successful warm rerun recorded below.

Root warm native result: `cargo test --profile wasm-dev --locked -p crystal-assets --test red_river_prairie` using `target/runtime-fps` passed **all three tests** in2.53s (`.local/red-river-native-tests.log`). This includes real in-memory native compilation/verification and preservation plus the negative source mutations. The first retry had two passing tests and one overly narrow Cut-error wording expectation; that expectation now matches the exact earlier shared rejection `undeclared decorative Cut tree at 41,44`. Existing `pack_core` warnings were unrelated. No file publication, controller movement, earned SQ02 journey or final native visual acceptance is implied by these asset tests.


## Verified native file publication and pending controller fixture

Root executed the native `pack_red_river_prairie` publisher with base `content-packs/minnesota-iron-range-descent.crystalpack` and output `content-packs/minnesota-red-river-prairie.crystalpack`. The published ignored pack has **415 maps**. The dedicated `--verify-published` comparison passed **two maps,1152 metatiles and17 objects**. All **377 audio** PCM hashes, frame counts and loop ranges passed. Root inspected [native seedbed](../renders/red-river-prairie/native-seedbed.png), and the atlas now uses this matching native slice. The author also inspected that actual native image against the source seedbed context; a single native camera does not establish whole-route acceptance.

```sh
target/runtime-fps/wasm-dev/pack_red_river_prairie \
  content-packs/minnesota-iron-range-descent.crystalpack \
  planning/minnesota/routes/MN_R16.layout.json \
  planning/minnesota/routes/MN_R16.json \
  content-packs/minnesota-red-river-prairie.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_red_river_prairie.py \
  --pack content-packs/minnesota-red-river-prairie.crystalpack --verify-published
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-red-river-prairie.crystalpack --audio --desktop
```

These are the integration owner's executed publication/verification facts. The map author did not run a competing publisher. The original HOME, recovery spawns, source maps, prior routes and audio remain preserved; this does not establish earned regional access or SQ02 completion.

The new focused controller leaf is [red_river_prairie.rs](../../../crates/crystal-bevy/src/bevy_shell/tests/red_river_prairie.rs). Root owns registration under the existing tests module:

```rust
#[path = "tests/red_river_prairie.rs"]
mod red_river_prairie;
```

Two ignored production-controller fixtures are prepared. The first explicitly starts an **empty-party geometry fixture** at global `(88,24)`, then uses `RouteGrid` for planning and only `VisibleShellController::press` for gameplay. It traverses both clipped maps and the drainage seam, both walking rows of each bridge in both directions, actual rail/tip denial, the corrected `(53,15)` → `(53,16)` → `(53,18)` native hop, blocked reverse and59 separately verified ordinary return steps. It interacts with all three residents, five signs and four pickups, verifies input ownership and exact item/flag changes, opens production Start, saves and Continues. After Continue it checks item flags/bag state and walks onto each formerly occupied pickup tile through actual input; no trainer victories may be claimed by this geometry fixture.

The separate **populated battle fixture** explicitly installs native-created level60 Typhlosion and Feraligatr plus six native obedience badge bits at initial placement `(28,25)`. Those partners/badges are setup, not an earned or balanced team claim. After setup no party, badge, position, collision or flag state is rewritten. Normal input talks to Camper Beck/T07, selects actual legal native damaging moves, checks selected PP consumption, faces all three authored opposing partners, completes victory and verifies money plus the actual defeat flag. It then walks the newly usable clear apron, saves/Continues, compares the full party and defeat flag and talks to the defeated trainer without a repeat battle. The other nine trainer flags must remain unset: one win is not represented as a full populated route journey.

The controller tests default to the actual415-map pack above; `CRYSTAL_RED_RIVER_PRAIRIE_TEST_PACK` can select its canonical path. Scoped Rust formatting passed. Root's warm compilation and both fixture executions passed, as recorded at the top. Native Surf travel, the rest of the populated route, earned badges/SQ02/hub recovery and full comparative visual judgment remain separate work.
