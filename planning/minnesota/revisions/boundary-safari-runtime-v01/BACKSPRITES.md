# Safari battle with trainer backs

Safari wildlife uses the full native battle screen: the selected player's Chris
or Kris backsprite faces the wild Pokemon. Ball, Bait, Rock and Run occupy the
command window, with remaining balls and travel steps above it. The trainer
stays visible during encounter results; returning to exploration removes the
battle canvas and trainer portrait.

## Verified 2026-10-29

Current v14 Confluence-parent review: the fresh ranger/commands/Continue/
retirement journey passes in23.01s and its reached-checkpoint compositor passes
in14.11s. Both [Chris](../../renders/boundary-safari-runtime-v11/confluence-backs-v02/chris-run.png)
and [Kris](../../renders/boundary-safari-runtime-v11/confluence-backs-v02/kris-rock.png)
were visually inspected. The trainer changes while the wild picture stays exact;
native cursor input updates the command pixels and returning to the field removes
the trainer battle portrait. [VERIFICATION.md](VERIFICATION.md) records the exact
pack, the retained first invocation's incompatible cursor checkpoint and the
fresh production journey used for the passing replay.

The thicket review's generated Shedstag encounter retains the Chris back sprite,
actual creature/level, balls/steps and all four native commands. The v13 broad
default-profile suite also includes the Chris/Kris compositor. Its source epoch
and subsequent contextual Cut/Surf menu projection checks are recorded in
[VERIFICATION.md](VERIFICATION.md); [THICKET.md](THICKET.md) links the new captures.

The earlier Iron-parent v10 review repeats the real ranger encounter journey
and compositor successfully in8.19s and5.66s. Both use the freshly reached
current-pack encounter save. The new
[Chris](../../renders/boundary-safari-runtime-v10/iron-rebase-backs-v01/chris-run.png)
and [Kris](../../renders/boundary-safari-runtime-v10/iron-rebase-backs-v01/kris-rock.png)
frames retain the selected native backpic and the same wild GILLPUP. See the
current section of [VERIFICATION.md](VERIFICATION.md) for exact parent binding.

The preceding v09 public-trail review also passes the trainer-backed encounter
controller and compositor in its default-profile replay. The fresh Chris/Kris
frames were inspected against the same actual GILLPUP encounter; switching the
trainer preserves the wild picture and cursor input changes native pixels.
[Current Chris render](../../renders/boundary-safari-runtime-v10/final-suite-backs-v01/chris-run.png)
and [current Kris render](../../renders/boundary-safari-runtime-v10/final-suite-backs-v01/kris-rock.png)
remain immutable. See [VERIFICATION.md](VERIFICATION.md) for the v09 pack hash,
full replay and separately supplied ending-save prerequisite.

Fresh request verification also passed on
`content-packs/minnesota-safari-runtime-review-checkin-v08.crystalpack` (SHA256
`97b036e392a003a63b60d32c3a6af18de01ec37c569ee3560fe08f98346567b3`).
The default-profile controller journey passed in 11.84 seconds, followed by the
compositor case in 7.67 seconds against its newly reached wildlife save. Both
fresh trainer renders were visually inspected; the Kris image is an explicit
appearance-only comparison of that same encounter. Both trainer backs, all four
commands, live cursor pixels, Continue, and removal of the battle portrait on
return to exploration passed. This remains an unselected review pack; the
observed main selector was `route_grass_lower_woods_revision`.

- [Fresh Chris render](../../renders/boundary-safari-trainer-review-v11/battle-v01/chris-run.png)
- [Fresh Kris render](../../renders/boundary-safari-trainer-review-v11/battle-v01/kris-rock.png)
- [Verification receipt](../../renders/boundary-safari-trainer-review-v11/receipt-v01.json)

### Earlier verification

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
