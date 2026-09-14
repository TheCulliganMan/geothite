# Reward audit in progress

| Area | Current evidence and remaining work |
|---|---|
| Navigation | Fixed missing post-egg Route 30 northbound objective; path routing still uses observed terrain and failed edges. Restored cooldown trial advanced north to (13,30); Route 31 pending. |
| Credit assignment | Navigation rewards can now override stale inactivity, while collisions, short loops and battle penalties retain priority. |
| Menus | Reopening Start without progress now costs; closure does not pay. First opening remains available for useful management. |
| Dialogue and objects | Existing once-only pages, completion and facing rewards; retain directional choice controls. |
| Healing and PP | Added restoration reward limited by permanent progress; damage/heal repetition, party substitutions and blackouts tested against reward farming. |
| Battles | Existing opponent HP low-water damage, defeated opponents, victory/capture and defeat signals. Restored trial won a battle; expanded setup telemetry and full-engine battles still under validation. |
| Items and party | Key items, machines, species and level milestones exist; consumable/ball discoveries are now once-only; useful party restoration pays. Equipment and switches receive downstream battle outcomes rather than rewards just for moving items or slots. |
| Story | Existing event milestones, field moves, badges and Hall of Fame. Curriculum beyond the opening requires continued validation. |

49 neural-crate tests pass at this checkpoint. The isolated candidate preserves
the current game WASM and resumes the copied game/brain pair through Continue.
Evidence: `target/reward-audit/trial`; source pair preserved as
`target/reward-audit/stalled-pair.tar.gz`. The original trainer remains running.

Route 30 north / Route 31 south connectivity checked against
https://github.com/pret/pokecrystal/blob/master/data/maps/attributes.asm .
No disassembly or generated game data is included in this repository.

The first restored trial remained near Route 30's southern entrance after more
than 600 outcomes, with no positive reward. A read-only graph check of its
source navigation memory found only 12 reachable tiles around (6,48), owing
to accumulated failed-edge records. Failed-edge records now expire after 64
feedback observations; observed wall terrain remains excluded. A regression
test verifies both behaviors. Frontier approach can pay once per directed edge
when the final route is unknown, without requiring that stepping-stone tile to
be novel. The second paired trial is `target/reward-audit/frontier-trial` on
port 33141; the first runs on 33140. Neither is a proven navigation success yet.

Game telemetry now exposes carried consumables/balls and existing battle status,
active-party, last-move, trapping, substitute and mist state. The neural ledger
rewards first item discovery and effective battle setup once, with buy/sell and
status/cure loop tests. WASM-target cargo check passed; the release game build
and actual battle verification remain pending. No production assets changed.

A 32-outcome menu reopening cooldown is now under test. It leaves an already
open menu usable and closable, and is preserved through checkpoints. It addresses
repeated menu interruption while the learned navigation policy adjusts.
`target/reward-audit/cooldown-trial` uses the same source pair on port 33142.

The additional-telemetry game build passed for wasm32, release profile. Candidate
engine SHA-256 is `58bf08fba2cf79c39ce5de48cfce42221ba22aed12ef9ab8a0be68af2c5631d4`
on port 33143 (`target/reward-audit/battle-candidate`). Source-pair migration
is being checked before battle trials. The candidate engine must receive new
fresh-opening evidence before release; prior-engine gates cannot be reused.

Early cooldown-trial observation: after 114 recorded outcomes it reached
Route 30 (12,48), paid five frontier advances and three once-only recoveries,
and submitted six Start actions. This is initial escape from the small loop,
not yet a Route 31 transition. The first two variants were stopped gracefully
after showing no comparable positive progress, retaining their evidence.
Migration to the additional-telemetry engine verified identical learned values
(`target/reward-audit/battle-migration-v2/migration.json`). A full-engine
restored trial is running at `target/reward-audit/full-trial`.

The restored cooldown trial subsequently reached Route 30 (13,30) and won a
battle, compared with the old trainer's hours around rows 48–53. Its starting
source remains identical to the unsuccessful first variants. This proves some
real escape/navigation progress, not full-game reliability. Two fresh starts
(seed 17 and seed 0) on the additional-telemetry engine are now running in
`target/reward-audit/fresh17` and `target/reward-audit/fresh0`.

## Repeatable goal progress (candidate after 0.5.0)

The live trainer later stalled at Route 30's northern end. Its saved policy had
exhausted the once-only approach edges: 179 sampled outcomes had no positive
reward, while the Start menu policy favored Right over closing the menu.

The candidate adds a separate signed navigation component after completed
movement: `0.3 * (old_route_distance - new_route_distance) - 0.01`. Both distances
use the same observed terrain, occupied tiles, and fixed goal set for that action.
Moving closer pays even on a familiar edge; retreat loses credit; a closed walk
under the same route model has negative total shaping. First-time exploration,
facing, and story completion retain their existing bounded bonuses. This is
engineered reward shaping, not a biological claim or a guarantee of optimality.
Menus, idle turns, and map changes do not receive distance credit. The next map's
actual observation determines its new objective.

An early gameplay trial exposed a conflicting short-loop penalty on recovery
steps. Signed movement now replaces that duplicate loop penalty, while collision
and battle penalties remain. Inactivity penalties no longer override measured
positive distance progress. Reward traces expose `navigation_progress` separately
from the novelty/story event list so the components can be audited.

The map-exit cue now persists at the boundary tile until an actual map transition;
previously reaching row zero removed the outward cue. It remains a sensory cue,
not a forced button. A runner fix also preserves the supplied checkpoint config's
action seed when no explicit seed override is requested.

Remaining progression gaps: the explicit destination curriculum currently ends
at entering Route 31. Later gyms, HM acquisition, prerequisite badges, teaching a
compatible party member, and returning to field obstacles are not connected into
a complete goal sequence. The ledger recognizes badges, machine acquisition,
field moves learned/used, captures and newly caught species, but those outcome
rewards do not themselves teach the preceding menu sequence. Battle victory is
also rewarded, so catching needs a context-specific objective rather than an
assumption that the agent will prefer capture. The preserved background brain
uses `visible_menu_inputs=false`; its legacy encoder lacks detailed menu labels
and some embedded cursor positions. A richer menu interface requires a separate
validated migration/training experiment, not a silent checkpoint setting change.

Final-build opening evidence: fresh seed 17 entered Route 29 at outcome 684;
seed 0 at outcome 868. Both ordinary watcher checkpoint pairs validate and are
saved on Route 29. Engine SHA remains
`58bf08fba2cf79c39ce5de48cfce42221ba22aed12ef9ab8a0be68af2c5631d4`;
tested neural WASM SHA is
`60739fad7b17cac77cf72fda78d052ddd1930d46b24c2123da6389951d83f51d`.
The final-build long-trained-save repeat remains on Route 30 after 761 outcomes
across a saved/resumed boundary. Consequently the first restored escape is not
reliable-recovery proof. The released-code comparison ended at 459 outcomes with
no map transition; the earlier signed-feedback variant reached Route 31 and
Dark Cave within 490. Trial reports remain under `target/goal-progress`.

Runner interruption verification: SIGINT produced a new paired checkpoint,
validated both hashes and exited with code 0. That pair resumed for 10 real
outcomes and saved again. The attempted alternate seed in that smoke test was
actually preserved as seed 17 by checkpoint restore; it is not an independent
seed-0 experiment. The runner now rejects such conflicting resume seed overrides
before launching the browser. Explicit fresh-start seeds remain supported.

The final-build repeat subsequently crossed Route 30 → Route 31 at combined
outcome 952 (decision 32591), preserving learning through its save/resume. This
is a second observed escape, with substantial variance in recovery time. The
route graph currently stores land traversal only; Surf and other mode-dependent
connections need explicit support alongside future HM objectives.

