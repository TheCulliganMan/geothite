# Northstar legendary completion

The active curated roster contains seven legendaries and two mythicals. Frontagal
(MN_F115) was retired by the user and has no quest in this release. Kensigard is
an ordinary optional discovery, separate from this checklist. The earlier
nine-missing-species review described an older pack; the fauna/progression parent
already installs the nine active rare encounters below.

| Species | Level | Quest, access and trigger | Capture flag |
| --- | ---: | --- | --- |
| Babehemoth | 60 | From Bemidji Northwoods enter `MnTallTaleTrail`. Read the worker and culvert/hoofprint/wood-stack evidence, acknowledge the cleared camp path, inspect the marked log and report it. The worker reveals the pool encounter. | `EVENT_MN_TALL_TALE_BABEHEMOTH_CAUGHT` |
| Bunjant | 60 | The same tall-tale investigation reveals the camp keeper separately. Catching either creature never gates the other. | `EVENT_MN_TALL_TALE_BUNJANT_CAUGHT` |
| Pepivern | 55 | In the Lower Mississippi Woods compare the dock sketch, shore eddy and reeds. The reeds require actual Surf. Report all three to the operator, then approach the inlet in `MnMississippiLowerWoodsC01R00`. | `EVENT_MN_PEPIN_CAUGHT` |
| Icemyth | 50 | At the free Fair exhibition, acknowledge three accounts, the rear latch and the meltwater tray. Correct the label and inspect the moved ice in `MnIcemythServicePassage`. | `EVENT_MN_ICEMYTH_CAUGHT` |
| Strandrak | 60 | Take the optional R16 spur through `MnFormerShores`; read the three terraces and set the independent basin indicators to high/middle/low. Acknowledge the inner outlet in `MnInnerBasin`. | `EVENT_MN_BASIN_CAUGHT` |
| Flarkindle | 55 | From Bemidji Northwoods enter `MnLayeredFen`, record the three layers, then follow older root, pool edge and living rib. Acknowledge the folded path. No calendar event or real-time wait is required. | `EVENT_MN_FEN_CAUGHT` |
| Vergloom | 40 | In the R08 ancient mire (`MnR08C00R00`), acknowledge timber, footprint and moss observations to reveal the creature. | `EVENT_MN_ANCIENT_MIRE_VERGLOOM_CAUGHT` |
| Kettlurk | 60 | Follow the optional Devil's Kettle stairs from the Marais cedar approach. Both the intake release and acknowledged survey restoration must already be complete. The ranger points toward the twin-falls pool in `MnDevilsKettle`. | `EVENT_MN_DEVILS_KETTLE_CAUGHT` |
| Verdigrim | 60 | Complete the witnessed intake exposure and separate drill, dirty-outlet and clean-bypass controls. Acknowledge crew clearance and containment, then voluntarily approach the exposed chamber in `MnBoundaryIntake`. | `EVENT_MN_VERDIGRIM_CAUGHT` |

All nine encounters are voluntary and use the native optional-legendary battle
policy. Only a completed capture sets the caught flag. Refusal costs nothing;
fleeing or defeating the creature hides it for the visit. Physically leave its
map and return to retry. Continue preserves the withdrawal. Full party uses the
selected PC box; full storage does not consume the ball or mark the species
caught. Recovery after a loss follows the normal registered-clinic rules.

The rescue/restoration scene requires no capture. Its clean bypass releases
Kettlurk and completes Verdigrim containment before either optional capture.
Ordinary regional travel and one discovery never require capturing another.

## Master Ball

Tamarack gives one Master Ball at the Winona Field Station after
`EVENT_MN_SURVEY_RESTORED`. Champion victory and caught-species totals are not
requirements. `EVENT_MN_MASTER_BALL_RECEIVED` records the one-time claim. A full
Balls pocket or full stack leaves the reward available after making room.
The completion overlay adds a Winona return hint to the acknowledged intake
ending and explains that the ball is used in a wild battle. There is no shop
supply; the native guaranteed-capture rule and trainer restrictions remain.

The older reward used `verbosegiveitem` before setting its claim. A production
stdio autosave on the receipt screen reproduced a second Master Ball after
Continue and another conversation. The repaired authoring uses native `giveitem`
at the same instruction offset; the following full-bag branch and saved claim
finish before the confirmation becomes an input boundary. Fresh reward builds
and the overlay both use this behavior.

An additional production stdio interruption test reproduced loss of a confirmed
catch when saving on “Gotcha!”: the ball was spent, but the pending frontend
capture was not in the checkpoint. The controller now records that resolved
outcome and exact encounter in its existing saved script-memory envelope.
Continue resumes the naming prompt, preserving partial nickname input and its
cursor. It uses the original result and paid ball, then the existing native
party/selected-PC/Dex/static-quest completion path. It does not reroll capture,
spend another ball, change trainer restrictions or change the core save format.

Saves retain their exact pack identity. An old pack's receipt save must not be
relabelled or silently loaded into a changed pack. This work preserves that
boundary; it does not provide general migration between content packs. Original
running packs and saves stay untouched.

## Reproduce the overlay and checks

Build `pack_northstar_legendary_completion`, then run it with a completed native
parent, the canonical roster/curation and a fresh ignored output:

```sh
cargo run --locked -p crystal-assets --bin pack_northstar_legendary_completion -- \
  "$PARENT_PACK" planning/minnesota/pokemon/roster.json \
  planning/minnesota/pokemon/roster-curation.json "$CANDIDATE_PACK"
```

The builder requires the existing nine native actor/battle programs and saved
caught flags. It changes the eight retry text bodies, intake return hint, Master
Ball confirmation and exactly one reward opcode/typed grant. Map geometry,
objects, entries, battle requests/levels, other rewards, flags and command
indices remain exact. A second installation is rejected.

Source preservation test:

```sh
CRYSTAL_LEGENDARY_PARENT_PACK="$PARENT_PACK" cargo test --locked \
  -p crystal-assets --lib \
  legendary_completion_preserves_gameplay_programs_and_rejects_incomplete_roster \
  -- --ignored --nocapture --test-threads=1
```

Native tests use `CRYSTAL_WINONA_TEST_PACK` and `RUST_MIN_STACK=67108864`.
`legendary_master_ball::` checks pending-offer/confirmation Continue, actual
Tamarack award carried on foot into an ordinary R01 wild capture, interrupted
confirmed ordinary/static catches, repeated Continue, partial nickname input,
selected-PC delivery, ball consumption/caught-Dex persistence, and rejection of
the actual old-pack receipt save supplied in
`CRYSTAL_LEGENDARY_LEGACY_RECEIPT_SAVE`.

The inherited quest filters are `northwoods_discovery::`, `pepin_discovery::`,
`icemyth_discovery::`, `strandrak_discovery::`, `flarkindle_discovery::`,
`northstar_ancient_swamp::`, `devils_kettle::` and `verdigrim_scene::`. Vergloom's
checks additionally require a fresh `CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR`.
The two inherited Master Ball filters are `progression_rewards_master_ball`
and `progression_rewards_failed_ball`. Core capture tests cover guarantee,
trainer rejection and storage/capture validation.

These are declared local progression/party fixtures followed by production
controller inputs. They establish quest mechanics and recovery paths, not a
fresh uninterrupted New Game-to-Champion run or an earned complete collection.
Check the current integration receipt for the exact tested candidate, executable,
results and outstanding checks; merely authoring this runbook is not a test pass.
