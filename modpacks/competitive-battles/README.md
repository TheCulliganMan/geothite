# Competitive battles

This Rust mod combines an exhaustive physical/special move split, a move
inspection board, and a stronger trainer planner. Every trainer uses the
planner when this pack is loaded, including trainers with no authored AI flags.
Species stats, move properties, story, encounter tables, and audio come from
the supplied external pack.

## Build and play

Build a separate ignored pack from existing verified content:

```sh
cargo run --locked -p crystal-assets --bin pack_competitive_battles -- \
  content-packs/text-tui.crystalpack content-packs/competitive-battles.crystalpack
cargo run --locked -p geothite -- play content-packs/competitive-battles.crystalpack
```

For graphics, use `content-packs/core-modular.crystalpack` as the input and
`content-packs/competitive-battles.native.crystalpack` as the output. Launch
that output with the graphical client's `--pack` option. Browser packs use the
same builder. The initializer and hosted-pack preparation compose this mod by
default.

The completed local variants are `content-packs/competitive-battles.crystalpack`
(text/MCP), `content-packs/competitive-battles.browser.crystalpack`, and
`content-packs/competitive-battles.native.crystalpack`. They remain ignored.
The release text client was built and launched from this checkout with:

```sh
CARGO_TARGET_DIR=target/competitive-battles-release cargo build --release --locked -p geothite
./target/competitive-battles-release/release/geothite play content-packs/competitive-battles.crystalpack
```

The standard game build composes this mod with Wild Challenge using
`pack_default_battles`. It enables the same planner for ordinary wild battles,
alongside their scaled levels and legal movesets. Standard play uses
`content-packs/text-tui.crystalpack`, `content-packs/core-modular.crystalpack`,
or `content-packs/core-modular.browser.crystalpack`. The standalone competitive
variants above remain available for isolated rules testing.

The builder verifies its input, applies the move split, preserves audio storage,
writes a candidate, reloads and checks its identity, and publishes the output.
Reapplying the mod preserves its identity. Saves and link sessions use the new
mod identity; start a new save for these rules.

## Move inspection

FIGHT uses the production move cursor and shows current/max PP and Disable
state while keeping player HP and status visible. Its panel shows category,
resolved type and power, base accuracy, priority relative to ordinary moves
(0), STAB, effectiveness, and non-critical damage bounds. `LO`/`HI` are
damage on a successful hit, before survival effects and Substitute routing.
Multi-hit bounds describe one hit. Conditional/random effects display `COND.`.
Hidden Power uses the user's DVs for type and power and its special category.
Reflect and Light Screen follow move category.

TUI and MCP expose the same details, plus turn count, weather, stat changes,
screens, and Spikes. Competitive terminal layouts keep both HP readings and
the move forecast visible at 48, 80, and 120 columns with 24 rows. All keyboard
and MCP input continues through
`VisibleShellController::press`; the renderer never owns game state.

## Trainer planning

Two-turn search uses the production Rust battle engine: eight chance samples
on the first turn and two on the continuation, weighting the strongest opposing
response at 75% and the average response at 25%. Damage, priority, accuracy,
status, recovery, stat changes, weather, held effects, and residual damage
therefore use the mechanics that execute during play. Position evaluation
values surviving team members, HP, useful stat stages, status, screens,
Substitute, and hazards. Switches must repay lost tempo and a margin; trapping
and entry hazards remain authoritative. After a knockout, the trainer searches
its surviving team for a replacement rather than always sending the next slot.
Party order, reward indexes, and entry effects stay authoritative; replacement
does not spend an extra turn.

Every class receives the same search budget. Profiles express strategic
preferences with a bounded score bonus, rather than lowering difficulty:

| Profile | Example classes | Preference |
| --- | --- | --- |
| Balanced | Champion, rivals, Cooltrainers, RED, custom classes | Flexible play |
| Offensive | Youngsters, Sailors, Blackbelts, Bruno | Apply damage and keep pressure |
| Setup | Psychics, Sages, Scientists, Sabrina | Establish useful stat advantages |
| Disruption | Bug Catchers, Bugsy, Koga, Janine | Status and move denial |
| Endurance | Fishers, Hikers, Swimmers, Jasmine | Preserve HP and sustain pressure |
| Switching | Bird Keepers, Bikers, Rocket grunts, Blue | Preserve the team through favorable switches |

Major tactical gains outweigh these preferences. Profiles apply even when all
authored AI flags are zero. Both live input and mutation replay use the same
trainer class, including generated Battle Tower opponents.

Owned trainer consumables retain their inventory and one-use tracking. The
planner compares every supported, unused item with attacking and switching,
without the original random use gate or strongest-party-member restriction.
Only the chosen item's inventory slot is consumed. Equivalent outcomes prefer
the smaller healing item; Battle Tower opponents gain no trainer consumables.

A bounded damage-race estimate uses the engine's non-critical damage bounds
and accuracy to value positions beyond the search horizon. This rewards useful
screens and setup, and prevents a lucky sampled status or critical hit from
outweighing a reliable faster knockout. Fractional damage-race estimates account
for speed and the extra turn required by charging and recharge attacks.
Paired complementary chance streams balance speed ties. When search scores
are closer than half a sample's HP resolution, reliable damage breaks the tie.

The prediction combines observed moves with plausible threats from public
species, level, and pre-evolution learnsets until four moves are revealed. Seeing
one weak move no longer makes the trainer forget other possible attacks. The
planner does not read the player's current command, unrevealed move slots, PP,
held item, or bench. Independent simulation streams leave live battle RNG
untouched.

At the root decision, a fixed-point fictitious-play solver examines the
simultaneous move matrix. It mixes moves only when the mixture improves the
worst-case sampled value beyond the uncertainty band; otherwise it retains
the reliable move choice. This reduces exploitation by a single predicted
counter using a strategically justified distribution. A mixed choice or genuine tie consumes
one replay-recorded byte. Link battles retain player input.

This bounded planner does not search every opposing switch, hidden set, or
future turn. It is not an optimal-policy guarantee.

## References and verification

The implementation draws on category-specific damage, accuracy, matchup,
setup/recovery, and switching ideas from
[poke-env's SimpleHeuristicsPlayer](https://github.com/hsahovic/poke-env/blob/master/src/poke_env/player/baselines.py).
[Pokémon Showdown](https://github.com/smogon/pokemon-showdown) provides the
reference for explicit categories and battle decisions separated from
presentation. The existing [move category catalog](../modern-move-split/data.json)
records its pinned Platinum revision for all 251 moves. These are design
references; Geothite has no external battle simulator/exporter dependency.
All 251 categories were independently compared with that pinned source on
2026-10-16, including the cross-type physical/special changes.
The bounded lookahead and position evaluation also draw on the
[poke-engine search interface](https://github.com/pmariglia/poke-engine).

Run the pack-backed behavioral and preservation regressions:

```sh
CRYSTAL_MODPACK_TEST_BASE="$PWD/content-packs/core-modular.browser.crystalpack" \
  cargo test --locked -p crystal-assets --test competitive_battles
```

They cover identity and content/audio preservation, category-aware decisions,
immunity, recovery, switching, inventory, PP/Disable, hidden-move independence,
and forecasts against actual turn damage. Also run the controller and stdio-MCP
gates in [the content guide](../../docs/game-content.md) against the generated
competitive pack before treating a new publication as playable.

Run the strength comparison explicitly with the same test target and
`competitive_ai_strength_across_trainer_profiles -- --ignored --nocapture`.
It runs 288 equal-stat mirrored singles against the original Champion policy,
48 per profile, with both sides using the same move split. Every profile must
score at least 70%, counting a draw as half a win. On 2026-10-16:

| Profile | Wins | Losses | Draws | Score |
| --- | ---: | ---: | ---: | ---: |
| Balanced | 34 | 10 | 4 | 75.0% |
| Offensive | 36 | 10 | 2 | 77.1% |
| Setup | 34 | 10 | 4 | 75.0% |
| Disruption | 34 | 10 | 4 | 75.0% |
| Endurance | 35 | 10 | 3 | 76.0% |
| Switching | 34 | 10 | 4 | 75.0% |

The six sets cover Alakazam, Gengar, Machamp, Typhlosion, Blastoise, and
Meganium with attacks, status, setup, recovery, screens, and priority.
The single-profile `competitive_ai_strength_against_original_champion_policy`
test retains the shorter balanced check. Tactical regressions additionally
exercise every profile with authored flags disabled, physical/special counter
mixing, distinct recovery choices, forced replacements, trapping, and inventory.
This measures the supplied singles suite, not an online competitive rating.

The production controller battle/PP and Mom/New Bark regressions passed with
the generated competitive text pack, as did MCP movement/Start and the spawned
client's empty-stderr check. All 14 text-client tests passed, including tactical
layout at 48, 80, and 120 columns. The release client was launched and exercised.

Native UI review followed the actual Route36 SquirtBottle interaction into
FIGHT. Paired 2D and 3D-mode captures preserve the same authoritative state and
LCD frame. That location currently retains source art because its terrain or
camera evidence is rejected; these captures verify the tactical LCD panel,
not modeled 3D scenery. `CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1` stops the QA input
route only after that panel is rendered. The explicit UI-review option
`CRYSTAL_RENDER_TEST_CAPTURE_SOURCE_FALLBACK=1` permits this existing fallback
and records the rejection reason in the temporary paired receipt. Ordinary
model/scenery captures still require the 3D renderer to be active.

For native diagnosis, `CRYSTAL_BATTLE_AI_TRACE=1` enables move-score tracing.
It is silent by default, including fullscreen TUI and stdio MCP sessions.
