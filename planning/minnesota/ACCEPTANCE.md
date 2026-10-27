# Minnesota verification and completion criteria

## What the current checks establish

`tools/check_plan.py` validates the authored planning records: unique identities, route dimensions, matching town/route endpoint reservations, map references, source species, encounter-slot counts, flags, source brush bounds and monotonic quest reachability. With `--render`, it renders all 20 terrain blockouts using source pixels read from the local pack.

These checks do not execute scripts, validate compiled event coordinates, establish final art quality, or make a playable pack. They must not be reported as gameplay acceptance.

## Map acceptance

The user clarified that maps do not all need to be compact. Reject sparse, overdesigned layouts with few worthwhile features. Give each map the space its landmarks, exploration and decisions justify. Existing native tests establish mechanics, not visual acceptance or enjoyable pacing.

- Keep ordinary paths about 2–4 gameplay tiles wide; justify wider destination spaces. Reject padded walks and area/count quotas.
- Validate every road segment and join. Provide purposeful choices such as a short trainer lane versus a longer grass detour with a reward.
- Include short required grass where appropriate and prove that a clear-floor path cannot bypass the declared crossing. Keep clear arrivals before/after it.
- Use native tile variety for distinct surfaces and readable landmarks. Confirm source artwork as well as collision: a roof fragment is not a freestanding sign.
- One-way ledges must use native HOP collision and have safe landing/side closures. Declare the return passage and minimum walking cost; reject trivial side bypasses and disconnected returns. Test downhill hopping, rejected reverse input and the complete alternate uphill walk through the production controller. See [the exact compact-route contracts](LEDGES.md).
- Fence selected water margins, leave deliberate dock entrances, close dock ends and test rail collision in the controller.
- Audit all reachable Surf boundaries, including clipped edge chunks. Unconnected outside water needs visible native buoys or rocks with blocking collision. Preserve open water only across actual reciprocal neighbors; prove Surf rejects the barrier and traverses each intended seam.
- Cut trees must seal named gameplay. Inventory every CUT_TREE quadrant, reject walking or unintended Surf bypasses, prove native missing-move/badge denial, remove the actual catalog block and walk to the gated target. Check the intended vegetation reset separately from persistent rewards on Continue.

- Every block ID, source tile, palette entry and collision definition resolves in the exact chosen pack or a typed derived tileset.
- Both town gates, every mandatory actor and every required door are reachable on the intended movement mode. The main journey stays walkable without optional Surf.
- Shared chunk edges have matching source quadrants and collision. Walk every active seam in both directions through the production controller.
- A warp arrives on safe FLOOR off the trigger, faces the intended direction, and allows leaving and returning without a bounce loop.
- Every visible door has a destination and return. Caves, clinics, shops, gyms and home stairs are tested individually.
- Trainer sight may control a deliberate battle lane; distinguish that challenge from an accidental trap. Defeated actors must leave a passing tile. Verify victory, item and scene flags persist, and validate recovery separately before regional acceptance.
- Water crossings use a real bridge/causeway recipe with matching collision. Do not accept a path painted over WATER or an invisible floor shortcut.
- Compare the map at full extent and at the actual 160 by 144 viewport. Inspect native pixels at 1:1; inspect the 2D/3D production pair only with equal map, position, time, facing and state.

Each route dossier's trainer, item and story anchors reserve space. The implementing agent must write resolved core-tile coordinates and assert their collision before map acceptance. The current blockout may leave an anchor on a wall; that is a fitting task, not evidence that the engine should ignore collision.

## Story acceptance

1. A fresh requested trainer starts in `MnWinonaHome2F`; the home warp and guardian scene work.
2. A advances one dialogue page, movement is rejected while text owns input, and completion releases input correctly.
3. Starter cancellation is recoverable; confirmation gives one level-5 choice once. Route 1 cannot open before starter receipt.
4. A full party or bag cannot lose a reward or incorrectly complete a quest. Delivery, pickup and reward checks are idempotent.
5. Rochester gym uses the authored team and real battle commands. A selected move consumes PP and resulting battle text is visible. Losing grants no badge and uses safe Minnesota recovery.
6. Badge slots 3 and 4 are both required for the northern Twin Cities gate. Returning from St Paul remains possible.
7. The headwaters marker and completed report are distinct conditions; the Iron Range gate checks the completed report.
8. A lost foreman or survey-chief battle cannot open the next story gate. A repeated win cannot duplicate rewards.
9. All eight badges and the harbor signal gate the North Shore; the restored survey separately gates the League approach, including arrival through Route 20.
10. Saving and reloading between each scene preserves flags, party, money, trainers, pickups and block overrides. Blackout never returns a Minnesota player to an unrelated Johto map.
11. League completion records the party through authoritative Hall of Fame behavior and permits a valid return to the region.
12. Boundary Waters' Safari expedition is optional. Its fee, ball/step exhaustion and rare catches cannot block the free Q07 shelter or R12/R13/R20 route junction. Safari capture-only commands, transactional entry, successful-step accounting, safe return and saved budgets require production-controller coverage before acceptance.

## Proposed regression names

These are test contracts to implement, not tests already present:

```text
minnesota_new_game_uses_requested_trainer_and_starter_menu
minnesota_guardian_dialogue_owns_input_and_advances_one_page
minnesota_route_one_crosses_each_chunk_and_returns_through_both_town_gates
minnesota_every_authored_door_has_a_safe_reciprocal_return
minnesota_rochester_delivery_and_first_badge_survive_reload
minnesota_first_gym_executes_move_pp_and_battle_dialogue
minnesota_twin_cities_north_gate_requires_both_badges
minnesota_headwater_report_and_depot_gates_enforce_quest_order
minnesota_border_loop_cannot_bypass_survey_restoration
minnesota_blackout_and_fly_destinations_stay_in_region
minnesota_pack_preserves_base_maps_assets_and_audio
minnesota_stdio_dialogue_emits_no_unconditional_stderr
```

## Existing repository checks

For Rust changes, use the pinned toolchain, `cargo fmt --all -- --check`, affected-crate `cargo clippy --locked -p <affected-crate> --all-targets` with applicable features, and relevant tests. Fix new warnings and report pre-existing failures without broad unrelated edits.

Run the root AGENTS.md controller, battle, MCP and clean-stdio regressions against the freshly built external canonical pack to guard the shared controller behavior. Those opening-story regressions intentionally expect `PlayersHouse2F` and the New Bark gate; do not change their expectations to make the Minnesota pack pass. Run separate Minnesota regressions against the new pack.

Use `verify_crystalpack --audio` on each published pack format to verify PCM hashes, frame counts and loop ranges. A Minnesota map overlay should preserve the base audio programs and payloads; adding routes does not authorize changing music synthesis. Keep all resulting PCM and verification output ignored.

`geothite dump` checks loading and text rendering only. A screenshot, coordinate change, successful compiler invocation or `--help` is not playability evidence. MCP and keyboard must both use `VisibleShellController::press`.

Automated native captures use:

```sh
export CRYSTAL_RENDER_TEST_HEADLESS=1
export CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image
```

Keep fullscreen stdout/stderr clean. Dialogue diagnostics belong behind `CRYSTAL_DIALOGUE_TRACE`, never unconditional printing. Only call a pack playable after the actual pack, production-controller journey, battle state changes, MCP path, stdio behavior and authorized production-client launch have all passed.
