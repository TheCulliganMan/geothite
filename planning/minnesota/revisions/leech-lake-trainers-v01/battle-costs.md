# What the three Leech Lake fights cost

Three ordinary five-Pokémon parties met Nell, Silas and Wren in succession on three different battle seeds. All nine runs finished in both revisions without a Bag item or a heal. The revised stretch took a median of 13 turns and 131 party HP, up from 11 turns and 86 HP. Wren accounted for the sharpest individual fight. On the hardest revised run, Pouchid’s team entered her battle with all five Pokémon standing and 348 of 430 HP. Eleven turns later two Pokémon stood with 181 HP between them. The same starting team beat her in three turns on each of the other two runs.

| Across the three consecutive fights | Trail Signs | Leech Lake Trainers |
| --- | ---: | ---: |
| Successful runs | 9 of 9 | 9 of 9 |
| Total turns, median (range) | 11 (10–16) | 13 (10–20) |
| Party HP spent, median (range) | 86 (59–166) | 131 (86–249) |
| Move PP spent, median (range) | 11 (9–16) | 12 (10–18) |
| Partners left standing, median (range) | 4 (3–5) | 4 (2–4) |

Starting each fight at full health shows where the cost changed. Each trainer faced the three parties on three seeds and lost all nine matches in both revisions. The table gives medians over those nine starts.

| Trainer | Turns before → after | Party HP spent before → after | Fights with at least one player faint before → after |
| --- | ---: | ---: | ---: |
| Nell | 4 → 4 | 0 → 15 | 0 → 0 |
| Silas | 4 → 6 | 60 → 86 | 2 → 5 |
| Wren | 4 → 5 | 58 → 90 | 2 → 5 |

Nell remains the gentlest introduction. Silas usually takes longer to beat. Wren can cost multiple partners even from full health. In the Pouchid party’s two hardest isolated fights with Wren, two partners fainted. These results support keeping the teams at level 30 and the way around the encounters open.

## Conditions

Each party started with its level-34 starter evolution and four level-32–33 partners obtainable on earlier routes. Six badges were granted via their normal award commands. The teams retained the moves available from actual gifts, daytime captures, leveling and evolution. The player chose the legal move with the highest estimated damage, used the first healthy reserve after a faint, kept its current four moves when prompted and declined optional switches. The installed trainers chose their own moves and actions. Both archive editions used the same three divider seeds (1701, 4242 and 9001). Intro and Field produced identical outcomes, turns and resource totals in all 108 matched fights across the parent and the revision. The tables count each matchup once.

The consecutive fights shared a single party, random state and Bag. Damage, move usage, status, earned experience and each trainer’s victory receipt carried forward. The run goes directly from battle to battle and excludes walking, poison damage from steps and optional wild encounters. It starts healed after Brainerd. These parties and decisions describe the tested matchups; other leads, switches and party sizes can change them.

The [battle trial](../../../../crates/crystal-assets/src/leech_lake_battle_trials.rs) reads the archive supplied in `MN_ROUTE7_BATTLE_PACK`. Set `MN_BATTLE_REVIEW_SEED` for each divider seed and run the ignored `leech_lake_route_battle_costs` test with output enabled.
