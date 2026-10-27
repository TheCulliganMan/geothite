# R11 added cascades — native eligibility audit

Read-only audit of `content-packs/minnesota-geography-review.crystalpack`; no Rust, test, source terrain, pack or registration was changed. Native identity `f9b04e67f2bfa93d915dae15ffb005d732ccee2df58b21d2f6ca550206a21949`; file SHA256 `4874055e9328d875700fbd5008c5fd46c9b4525625511ab5a0fafc443fb08a56`.

## Evidence and likely cause

The actual run in `.local/geography-shore-controller.log` passes the original Waterfall journey and four other old cases. The added-cascade case fails at the `flygon_field_actions.contains("waterfall")` assertion, so it has not established a real HM ascent. The separate collision-helper failure is a fixture lookup against the wrong current chunk and is being repaired by Root; it does not establish a terrain error.

No old-map-only Waterfall registration was found. [execute_visible_contextual_field_move](../../../crates/crystal-bevy/src/bevy_shell/battle_messages.rs:3714) checks the **facing** collision permission, Surf mode, Up facing, a learned move and the badge. [field_move_rule_contains_target_collision](../../../crates/crystal-bevy/src/bevy_shell/battle_messages.rs:3989) reads the shared rule, with no map-name/id/terrain whitelist. The core [apply_waterfall_field_move](../../../crates/crystal-core/src/systems/field_moves.rs:1483) has the same movement/party/badge/collision requirements and no R11 chunk restriction. [use_waterfall_field_move](../../../crates/crystal-assets/src/game_data_items_and_battles.rs:1175) prepares the shared source climb. `use_visible_waterfall` records the action only after that authoritative call succeeds.

The actual pack has WATERFALL native block `80` at the new faces. Global `Script_UsedWaterfall`, `.loop@Script_UsedWaterfall` and `.WaterfallStep@Script_UsedWaterfall` exist. The shared rule has move WATERFALL, Johto badge index7 and target permissions `[51,48,49,50,59]`; `51` is native WATERFALL (`0x33`). All three R11 chunks use `mn_north_shore`, environment ROUTE and the same landmark. None contains a per-map Waterfall script; the working middle chunk also uses the globals.

| Global core probe, x18 | Actual native collision |
| --- | --- |
| y19 / y20 / y24 / y25 / y26 / y27 | WATER / WATERFALL / WATERFALL / WATERFALL / WATER / WATER |
| y99 / y100 / y101 / y102 / y103 | WATER / WATERFALL / WATERFALL / WATER / WATER |
| y104 / y105 / y106 / y107 | WATERFALL / WATERFALL / WATER / WATER |

The likely fault is the helper's unconditional extra Up input. The new fixture walks Up onto (18,26) before entering `climb_added_cascade`, so the player already faces Up. The helper then presses Up again. [attempt_step_with_occupied_tiles](../../../crates/crystal-core/src/world/movement.rs:151) turns without stepping only when the requested direction differs; an already-Up player instead enters (18,25). That native WATERFALL is a downward current: [forced_movement_direction](../../../crates/crystal-core/src/world/session.rs:1614) decodes permission `0x33` as Down. [forced_tile_step_checked](../../../crates/crystal-core/src/world/session.rs:822) explicitly sets `player.facing = direction` before moving. Settling can therefore return the player to (18,26), **facing Down**. A then samples ordinary water at (18,27), rather than the Waterfall to the north, and never dispatches the HM.

The old passing fixture finishes lateral movement at (20,72), facing Left. Its one initial Up press turns toward the fall without entering its current. The positional assertion permitting base or base−1 does not check this material facing difference. Repeated A cannot repair a wrong facing. The same hazard applies to the second southern stage after its Up move to y102.

This is source-supported diagnosis, **not an observed compact controller trace or a passing repair**. Root should confirm position/facing/mode/facing-permission immediately before the helper's Up, immediately after settling it, and before A. Do not print a full runtime/catalog snapshot.

## Smallest correct repair and proof

Keep the runtime, art, scripts and rule unchanged unless that trace contradicts the diagnosis. Fix the fixture helper to request an Up turn **only if not already facing Up**. Before A assert the settled player remains on the declared base, is Surf/SurfPika, faces Up and has WATERFALL immediately north. Then use the same production A/YES/text/clock path. Do not set position/facing directly or call the authoritative field move outside `VisibleShellController::press`.

Retain the real-action assertion and exact north (18,19), lower south (18,103), upper south (18,99), native source-loop completion and ordinary descent/bank-return checks. Root must rerun all seven cases on this actual published pack. A passing repaired fixture would establish the added climbs; current evidence establishes native eligibility and a likely input-sequencing error only. No new render or conversion was produced, and existing captures/logs remain retained.
