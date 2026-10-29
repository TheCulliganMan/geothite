# Safari battle with trainer backs

Safari wildlife uses the full native battle screen: the selected player's Chris
or Kris backsprite faces the wild Pokemon. Ball, Bait, Rock and Run occupy the
command window, with remaining balls and travel steps above it. The trainer
stays visible during encounter results; returning to exploration removes the
battle canvas and trainer portrait.

## Verified 2026-10-29

The default-profile production-controller ranger journey and native compositor
regression both pass on the unselected review
`content-packs/minnesota-safari-runtime-review-guidance-v05.crystalpack` (SHA256
`1976290aab92e6916c4e8457dbf9f9940cd45f835a87626e3a7a8fc817d861ef`).
The journey reaches wildlife through paid admission and native travel, executes
all four commands, checks saved input ownership and Continue, and retires.
The compositor checks the complete battle canvas, actual command cursor pixels,
distinct Chris/Kris backs, unchanged wild portrait when switching trainer
appearance, and removal of the trainer portrait after Run. Both fresh renders
were visually inspected. The Kris comparison changes only trainer appearance
as an explicit render fixture.

- [Chris battle render](../../renders/boundary-safari-runtime-v09/trainer-battle-v01/chris-run.png)
- [Kris battle render](../../renders/boundary-safari-runtime-v09/trainer-battle-v01/kris-rock.png)
- Reached save: `renders/boundary-safari-runtime-v09/trainer-review-v01/wildlife-menu.crystalsave`
- Controller case: `safari_controller_real_ranger_generated_wildlife_commands_continue_and_retirement`
- Compositor case: `safari_native_compositor_draws_and_refreshes_the_saved_command_window`

This confirms the battle presentation in that review pack. At this verification
epoch the selected main revision was `grand_marais_partner_retry_revision`;
paid Safari admission there was closed. Check the live world selector for later
map revisions. Throw/shake/audio presentation and remaining travel acceptance
are separate open work recorded in the existing verification plan.
