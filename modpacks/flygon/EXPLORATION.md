# Exploration tuning

The controller is a hybrid: actual connectome dynamics and dopamine-gated
synaptic plasticity feed an engineered learned policy adapter. Story scents,
walkable-map memory, and sensory-context gates are explicit assistance. They do
not directly choose buttons. Neural activity alone is not evidence of learning;
map transitions and first visits establish what a particular run accomplished.

## Repository inspection, 2026-09-12

Clones were inspected outside this repository; no third-party game content or
neural datasets are bundled from them.

- [Doomfly](https://github.com/nftechie/doomfly), revision
  `71ecf53d78eaffaf1a57ed7b0ccf5d458abc9f33`: `doom/training.py` pairs
  nonfatal damage with measured PPL101 stimulation, cancels stale pulses at
  episode resets, and reports efficacy changes. It explicitly does not establish
  useful learning. `doom/reward.py` is a separate sugar-input path without
  plasticity. Lesson: record delivered stimulation and behavioral outcomes
  separately, and avoid assigning punishment across unrelated transitions.
- [Mario fly](https://github.com/ornata/fly), revision
  `f2f4114e53eaa326e54129f27a5383f93c6957af`: fixed descending-neuron control
  mapping, with no reward training or star objective. It is not an exploration
  solution to transplant.
- [FlyGym's navigation examples](https://github.com/NeLy-EPFL/flygym-gymnasium),
  revision `d285260a1c8a7b3494150cd1590f2c9fe4b5e06b`:
  `examples/olfaction/plume_tracking_controller.py` uses encounter history and
  stochastic transitions between walking, turning and stopping, following
  [Demir et al.](https://doi.org/10.7554/eLife.57524). This is an engineered
  navigation controller, not whole-connectome learning. The useful principle
  here is persistent exploration when a goal signal is unavailable.
- [FlyGym 2](https://github.com/NeLy-EPFL/flygym), revision
  `38c8ec61034cd59bc5ba0de20688d4a3c0000d60`: its README redirects the older
  olfactory navigation examples to the gymnasium repository above.
- [fly-brain](https://github.com/lixiang1076/fly-brain), revision
  `f6d525ab10c7a3ed4853d3d8b4b5e73f24cfe52b`: `fast_learning.py` applies
  post-simulation MBON-rate corrections. That method is not used here because
  the displayed activity should be the activity actually simulated.

## Changes being observed

Walking heads distinguish the current objective and available directional
sensory cues. All heads begin untrained and still score measured neural activity.
Exploration cues receive salient sensory current when no story scent is present.
Fully visible rooms retain curiosity toward unvisited land after visual frontiers
are exhausted. Explicit negative outcomes always produce a negative update for
the punished action; immediate shocks are not propagated to earlier neutral
turns. The reported advantage belongs to the latest outcome.
At a story object, directional scent identifies the facing needed to interact.
Correct orientation pays once per objective/object, preventing repeated turning
from farming the reward. An aligned interaction has its own sensory context.

Start is excluded from overworld sampling until the Mystery Egg is delivered to
Elm; setup and battle controls remain available. Both policy sampling and random
exploration honor this exclusion. B can still close existing menus.
Dialogue pages without a choice menu exclude directional inputs; choice menus
retain them. These are explicit interface constraints, not learned discoveries.
Browser transport waits for field animations to settle using read-only observations, so
one movement's outcome is not credited to the next button.

Use `tools/flygon-runtime-watch.mjs` and `tools/flygon-runtime-reload.mjs` as
described in the README. Evidence remains ignored under `target/`. Count distinct
map/tile coordinates and actual map transitions; do not count button submissions,
rendered brain activity, or reward totals as exploration. Hot reloads retain the
real game and neural checkpoint, and must be disclosed as runtime tuning rather
than presented as an untouched cold-start trial.

Run `node tools/flygon-exploration-report.mjs <evidence-directory>/runtime.json`
for conservative per-map tile counts, observed transitions, and input-exclusion
violations. Missing intermediate observations are not filled with invented paths.

## Additional source comparison, September 12

Inspected additional upstream source checkouts, rather than treating README
claims as validation:

| Implementation | Actual input/readout mechanism | Relevant difference from Flygon |
|---|---|---|
| [Minecraft](https://github.com/blendi-remade/fly-brain-minecraft/blob/6cfa30175003ef25da68a237d5eda958f8047b82/src/main/java/com/fruitfly/brain/SensoryEncoders.java) | Separate receptor/type populations for vision, odor, taste, touch and proprioception; luminance ON/OFF transients, adaptation and saturating transfer functions; optional analytic object channels | Our text/map signals are an engineered prosthesis. Preserve modality distinctions and temporal feedback; do not describe arbitrary feature hashing as measured receptor anatomy. |
| [Mario](https://github.com/ornata/fly/blob/f2f4114e53eaa326e54129f27a5383f93c6957af/fly64/retina.py) | Spatially ordered compound-eye samples from a six-face first-person atlas; seven samples per receptor | Spatial registration is explicitly approximate. A whole-scene cue that suppresses local geometry loses the distinctions this implementation preserves. |
| [Doomfly v6](https://github.com/nftechie/doomfly/blob/71ecf53d78eaffaf1a57ed7b0ccf5d458abc9f33/doom_learning_v6/visual.py) | R8 column mapping inferred from outgoing-contact votes, confidence reporting, explicit RGB proxy, target-specific sign correction | Model assumptions and unvalidated calibration are disclosed. Its README reports failed visual, conditioning and survival gates; changing synapses does not establish useful gameplay. |
| [Desktop fly](https://github.com/DenisSergeevitch/desktop-fly/blob/32b00011e83c3dc85fa3ea0b3934155b04f1635d/Locomotor.swift) | A 1,045-cell MaleCNS locomotor extract receives contact and joint feedback; normalized synaptic inputs and an engineered cross-specimen descending interface | Useful model for closing the feedback loop: observed displacement/turning should return to the controller. This is not a full MaleCNS brain or a story-solving policy. See its EVALUATION.md for lesion and feedback ablations. |
| [Fly Escape](https://github.com/dzhng/fly-escape/blob/06540138d55127f87e69f8cca9982c4f1328be90/crates/sim/src/lif.rs) | Selected graph, tonic drive/noise, population motor readout; olfactory steering uses spike fractions because reset voltage can invert apparent response | Readout calibration matters. Its explicit motor equations and selected graph cannot be interpreted as evidence that an untrained full graph solves Pokémon. |

The new candidate reserves separate channels for story and exploration scents,
leaving terrain/dialogue/battle currents unchanged when a scent appears. Nearby
terrain carries more weight than repeated distant background. Opponent input
pairs retain both signs of the feature projection. Visible-state repetition and
actual movement outcomes supply history cues; repetition raises the exploration
mixture to at most 35%, with the same action exclusions as the learned policy.
No candidate action is injected, no readout spike is fabricated, and no measured
graph edges are added. These are engineered interface changes, not biological
claims. Interface v4 checkpoints are intentionally distinct from v3 policies.

Observed-development corrections before release: the neighbor route probe now
rejects NPC-occupied starting tiles. Otherwise it could seed a path inside Mom's
tile even though ordinary path expansion excludes NPCs. Approach edges also pay
only once per objective: repeatedly backing away and recovering cannot farm the
same positive reward. Conversation completion has a one-time object ledger.
A separate browser successfully restored the real saved game and brain and
continued normal gameplay. The watcher validates that browser save bytes match
the exported game's checksum before resuming.

The prepared graph has 4,868 central sensory cells and 9,201 visual projection
cells, but this adapter stimulates at most 256 selected cells, including the
sparse KC memory prosthesis. Minecraft and Doomfly stimulate much larger
receptor arrays. Our selected channels are not a calibrated retina and their
map/text semantics are artificial assistance. The present changes address lost
modality information and missing outcome history; they do not demonstrate that
a larger input pool alone improves story completion. That requires matched
behavioral trials rather than a neuron-count claim.

## Observed gate failure on the production game binary

Two separate fresh-start browser runs reached Elm's lab and obtained a starter,
then reproduced the aide's unresponsive first Potion dialogue page. Repeated A/B
left `I want you to have this` unchanged, with no menu open, no reported animation,
and no reported engine error. Run 1 used an earlier diagnostic policy; run 2 used
the updated occupied-tile routing and reward ledger with action seed 17. Neither
is a successful Route 29 trial. No release is justified by these observations.

Run 2 recorded 1,146 decisions and 82 distinct map/tile pairs: 15 upstairs,
17 downstairs, 37 in New Bark, and 13 in Elm's lab. The observed transitions
include one return upstairs, then New Bark and Elm's lab. Its report found ten
short movement loops. Select had zero submissions in 1,146 excluded decisions;
Start had zero submissions in 974 excluded overworld decisions. Repetition and
slow progress remain visible despite successful menu exclusion.

A forced 1 ms screenshot timeout produced three skipped screenshots while the
controller continued and wrote another paired checkpoint. Separately, a real
checkpoint was restored via the normal Continue menu, its brain restored, and
play continued. These are interruption/recovery checks, not fresh-start success.

The isolated engine replay refined the diagnosis: the Potion was granted and its
receipt briefly appeared. B then discarded that notice through generic dismissal,
leaving the item state machine and script cursor unfinished. The previous offer
text resurfaced and subsequent A/B stalled. Item-receipt B now follows the same
fanfare, receipt, pocket-page, and script-continuation path as A.

A real pre-handoff save replayed with alternating A/B now observes the Potion
receipt, completes the following conversation, and exits the lab into New Bark.
The browser replay explicitly unlocks audio using the visible audio button, as
Flygon's Run button does; a suspended browser audio clock otherwise keeps waitsfx
pending. This restored-save regression is not a fresh-start policy trial.

## Final-binary opening gate, September 12

Both separate fresh-start trials used the same fixed game WASM, SHA-256
`5123bb6911d413641072075d75deab186bb455209169d1ab4c8e7dba5e1b4ee9`, and
content SHA-256 `91a41c4f931348805a62a4df8d06ab907f1b654c491ff825a319d4f71d9f8f8d`.
Their controllers were not reloaded or manually steered. Normal safe-boundary
checkpoint pauses were enabled. Separate restored-save tests are excluded.

| Fresh trial | First observed Route 29 transition | Elapsed from browser launch | Distinct map/tile pairs through arrival | Short movement loops | Plain-dialogue inputs outside A/B | Excluded Start / Select submissions |
|---|---|---|---|---|---|---|
| Action seed 0 | Decision 985; New Bark → Route 29 at (59, 8) | 898.5 seconds | 79 | 26 | 0 / 189 decisions | 0 / 0 |
| Action seed 17 | Decision 816; New Bark → Route 29 at (59, 9) | 755.9 seconds | 74 | 10 | 0 / 175 decisions | 0 / 0 |

This passes the repeated opening gate, not a statistical claim of general story
reliability. Loops remain measurable. Both runs subsequently reached Cherrygrove;
seed 17 then entered Route 30. Further milestones must be observed separately.
Ignored evidence lives under `target/release-opening-seed{0,17}/`: original
per-decision traces, arrival screenshots, paired checkpoints, exact opening-only
reports, and `route29-gate.json` with the first transition and binary identity.

Validation: 36 Flygon Rust tests, 86 browser tests, the focused Potion Rust
regression, and a real saved-game browser handoff replay passed. The final-binary
recovery test checked game/content hashes and matching saved-game bytes, restored
the real game and brain, survived forced screenshot failures, and saved again.

## Why most of the graph is quiet

The adapter caps its input pool at 256 cells (192 selected central sensory/visual
projection cells plus 64 KC memory inputs) and its descending readout at 256.
One live 150 ms decision delivered positive current to only 42 inputs: 31 input
cells fired, while all 256 descending cells remained subthreshold. Their observed
voltages ranged from -53.44 to -49.02 mV against a -45 mV threshold. The complete
graph is retained; structural inclusion does not establish functional engagement.

A separate native experiment reused one real observation with learning disabled
and a fresh graph for each gain. It did not change either gameplay trial:

| Contact gain (mV/contact) | Neurons firing in 150 ms | Total spikes | Descending readout spikes |
|---|---|---|---|
| 0.02 (release setting) | 40 | 381 | 0 |
| 0.05 | 63 | 485 | 0 |
| 0.10 | 158 | 989 | 15 |

This supports investigating sparse drive and weak propagation. It does not show
that raising gain improves gameplay, nor that widespread firing is desirable.
Compared with the larger receptor arrays in Minecraft and Doomfly, a next
experiment should separate input-pool size from gain, calibrate modality-specific
responses, and measure downstream stimulus discrimination and behavioral success.
The release does not apply the unvalidated gain increase. Current gameplay success
still includes substantial engineered story cues and learned policy adaptation;
no causal claim that the connectome alone learned the opening is warranted.

Post-gate continuation: seed 17 entered Mr. Pokémon's house at decision 1,398
(1,271.8 seconds from launch), received `MYSTERY_EGG` at decision 1,414
(1,284.3 seconds), and returned to Route 30. Its real game and brain were saved
there together. Seed 0 also crossed Cherrygrove and reached Route 30; the opening
repeat requirement does not imply two completed Mr. Pokémon interactions.

## Sensory population optimization in progress

The controller now supports an experimental `operant.sensory_neurons` budget
(256–16,384), independent of the 256-dimensional policy. Defaults retain the
released behavior. Selection still uses actual central sensory/visual projection
cells, type/hemisphere diversity, actual graph reachability, and 64 plastic KC
inputs. Incompatible population checkpoints are rejected rather than silently
applying a learned policy to different inputs.

A width-only sweep revealed that enlarging the hash table mostly adds undriven
cells. The experimental encoder therefore distributes each modality across
independently hashed banks of up to 256 neurons, retaining bank-zero behavior and
zero current for absent cues. This is an engineered population code, not a claim
of anatomical retinal calibration.

Twelve recorded situations from the observed seed-17 run were replayed with
learning disabled and transient neural state reset between observations. All
25,582,938 edges remained present. At the unchanged 0.02 contact gain:

| Requested sensory budget | Actual selected inputs | Mean driven inputs | Mean firing neurons | Mean native probe time | Situations with descending spikes |
|---|---:|---:|---:|---:|---:|
| 256 | 256 | 36 | 32 | 35 ms | 0/12 |
| 1,024 | 1,007 | 136 | 120 | 51 ms | 0/12 |
| 4,096 | 4,018 | 564 | 505 | 82 ms | 0/12 |
| 14,080 | 13,430 | 1,888 | 1,748 | 146 ms | 11/12 |

Larger populations also increased common activity: mean pairwise cosine similarity
between situation readouts rose from 0.434 to 0.887. More firing therefore cannot
be interpreted as more useful discrimination. Probe times include initial wiring
construction amortized across the observations and are not browser throughput.

Matched fresh gameplay trials (same seed 17 and pinned game/content, default vs
14,080-input budget at unchanged gain) are running under
`target/sensory-gameplay-{baseline17,wide17}`. No population change has been
promoted to production or the persistent trained run. Original sweeps are retained
under `target/sensory-{width,banked}-benchmark`; the reproducible assay is
`tools/flygon-sensory-benchmark.mjs`. Gameplay and repeat-run evidence remain
required before choosing the production configuration.

A third fresh trial now evaluates a 1,024-input budget at gain 0.05, using the
same seed 17 (`target/sensory-gameplay-middle17`). Its recorded-observation
readouts had mean cosine similarity 0.507 and mean native probe time 67 ms,
compared with 0.887 and 146 ms for the largest budget at gain 0.02. These metrics
motivate the comparison; they do not establish which policy plays better.

A separate operational improvement skips the interactive 3D connectome viewer
for headless training. Actual restored-save runs verified continued policy
updates and paired saves with and without the viewer. This change is installed
on the private training web copy for subsequent sessions. The three already
running fresh trials retain their original viewer setting throughout.

### Completed population comparison and the next bottleneck

All three initial seed-17 trials observed New Bark → Route 29 on the pinned game:

| Interface | Decisions to Route 29 | Page elapsed time |
|---|---:|---:|
| 256 inputs, gain 0.02 | 742 | 610.5 s |
| 14,080 requested inputs, gain 0.02 | 629 | 730.7 s |
| 1,024 requested inputs, gain 0.05 | 682 | 829.0 s |

The wider interface reduced decisions but did not improve elapsed time. These
shared-host trials retained the viewer and are not isolated hardware benchmarks.
Each was then stopped with original traces, arrival screenshots, and verified
paired checkpoints on Route 29 retained. This comparison does not establish
repeatability or progress through battles.

Live battle observations exposed a separate sensory omission: the encoder used
menu kind/selected fields, but most menus expose the cursor inside `entries`
strings. All battle submenus have kind `battle`; the selected Fight, move, item,
or party entry was therefore poorly represented. Raw battle-overlay tokens also
included debug fields such as AI flags and script names.

Experimental `visible_menu_inputs` now encodes visible row labels, row positions,
and cursor markers, and substitutes actual rendered text/battle messages for the
raw debug overlay. It does not prescribe a move or button. The default remains
false pending validation, and changed sensory identities reject old checkpoints.

Experimental `sensory_window_ms` separates observation duration from the previous
fixed 150 ms window and normalizes spike readouts by duration. In the same twelve
recorded situations, a 50 ms wide-input probe had mean same-situation cosine
similarity 0.9874 to the 150 ms readout (minimum 0.9688). This does not demonstrate
behavioral equivalence. Timing in this sweep was affected by concurrent browsers;
the configuration reduces integration steps, but gameplay must establish speed.

New fresh seed-17 trials use corrected menu inputs and no 3D viewer in both arms:
256 inputs/150 ms versus 14,080 requested inputs/50 ms, both at gain 0.02. Evidence
is under `target/menu-gameplay-{baseline17,wide17}`. This compares two candidate
configurations; it is not a claim isolating the causal effect of population size.
Forty Rust tests pass, including visible cursor/submenu discrimination and removal
of debug tokens. Existing release checkpoints still restore in the legacy mode;
incompatible menu-input checkpoints are deliberately rejected.

### Follow-up gameplay and rival loss regression

The corrected-menu/headless seed-17 comparison reached Route 29 at 689 decisions
and 512.1 seconds with 256 inputs/150 ms, versus 1,122 decisions and 1,188.6 seconds
with 14,080 requested inputs/50 ms. The wider trial was stopped with its evidence
retained. These shared-host measurements favor the smaller candidate; they do
not isolate population size from integration-window effects.

The 256-input seed-17 run subsequently reached Cherrygrove and Route 30. A second
fresh seed (0), also including the unchanged-battle-menu feedback correction,
observed PlayersHouse2F → PlayersHouse1F → NewBarkTown → ElmsLab → NewBarkTown →
Route29 → CherrygroveCity. The two runs differ in the feedback patch and therefore
are not a repeated evaluation of one final release. Both still use the old pinned
game engine. Final-binary opening validation remains required.

The persistent trainer exposed an engine defect: the first rival is loaded as
BATTLETYPE_TRAINER although the authored script sets BATTLETYPE_CANLOSE between
loadtrainer and startbattle. The parser captured the earlier type. An integration
regression against the actual external pack reproduced the incorrect freshly
compiled type; the runtime compatibility correction resolves the intervening
loadvar for existing packs. The parser now updates pending trainer requests only
before startbattle, preserving earlier completed battles when a later loadvar
occurs. The external-pack integration test now passes for all three starter
branches, for both existing metadata resolution and recompilation. Native
completion and normal-button browser loss-path checks remain
required before deploying this correction. Ending a battle or moving away does
not prove the rival scene has completed: its scene must reach NOOP.

The normal-button old-engine loss test is retained at
`target/rival-engine-before/result.json`: one battle, a recorded faint, no
CANLOSE type, no completed scene, and no Route 29 exit. It used engine SHA-256
`5123bb6911d413641072075d75deab186bb455209169d1ab4c8e7dba5e1b4ee9`.
The first corrected engine exposed a second stale-metadata path: snapshot/save
validation rejected CANLOSE against the cached TRAINER value. Save-origin
validation and runtime trainer keys now resolve through the same authored-command
lookup as battle creation. This revision still needs native and browser validation;
the first corrected binary is not a release candidate.

The revised native rival-loss test passes with snapshot validation enabled. The
normal-button browser regression now also passes on engine SHA-256
`8fdbab70ff719b0f90192a38a9d74df38adff2bafb6107de005fdecc7aec9e97`:
one CANLOSE battle, observed faint, completed Cherrygrove scene (NOOP), then an
actual Route 29 exit, with no page errors. Evidence is retained under
`target/rival-engine-corrected-dialogue`. This restored-save diagnostic deliberately
uses LEER; it is engine evidence, not a successful learned policy run.

The diagnostic driver initially failed to acknowledge a battle message remaining
visible after the model returned to the overworld. Both the driver and neural
plain-message action mask now include that state. The brain keeps A/B available
for plain battle messages and preserves directional inputs for menu choices;
41 brain unit tests pass. Two fresh neural runs of this final engine/message-mask
combination (seeds 17 and 0, 256 inputs/150 ms with visible menu inputs) are now
recording under `target/release-corrected-seed{17,0}`. Release reliability remains
unproven until their actual transitions are inspected.

### Release 0.4.0 opening gate

Two fresh runs of the corrected engine and brain observed New Bark → Route 29:

| Seed | Decisions at first observed exit | Page elapsed time |
|---|---:|---:|
| 17 | 592 | 432.1 s |
| 0 | 851 | 654.2 s |

Both used 256 sensory inputs, gain 0.02, a 150 ms sensory window, visible menu
inputs, and headless UI. Their paired checkpoints and exact game/content hashes
were verified. Evidence is retained under `target/release-corrected-seed{17,0}`.
The saved progress-report snapshots recorded zero submissions while Start/Select
were excluded and zero directional inputs on plain dialogue, while recording 10
and 19 directional menu inputs. They also recorded 7 and 35 short loops: menu
restrictions do not eliminate all navigation loops. These are shared-host elapsed
times, not isolated performance benchmarks. Longer exploration runs continue.

With the same message-aware driver, the old engine repeats the rival battle after
a loss (two battle entries, scene still active); the corrected engine completes
one battle and exits to Route 29. See `target/rival-engine-before-dialogue` and
`target/rival-engine-corrected-dialogue`. The native loss test and the external-pack
metadata/recompilation test pass. All 41 brain tests pass. The packaged release
startup check completed 39 neural decisions. These engine and startup checks do
not count as additional fresh opening trials.

### Extended story observations after the release gate

The corrected seed-17 and seed-0 runs both reached Cherrygrove, Route 30 and
Mr. Pokémon's house, then received the Mystery Egg. Seed 0 also returned it to
Elm. Their saved extended reports contain 2,170 and 4,334 recorded decisions,
respectively, with zero excluded Start/Select submissions and zero directional
inputs on plain dialogue. They still record 25 and 275 short loops; these results
demonstrate story progress, not loop-free play or full-game reliability. Evidence:
`target/release-corrected-seed17/extended-report.json` and
`target/release-corrected-seed0/extended-report.json`.

### Additional source inspection during reward repair

- `ad7584/flycoin`, commit `b8164227c862704a1fd20b517a4bc0031e05dd21`: inspected
  `train.py`, `pumpui.py`, and `test_transfer.py`. Male CNS model uses weighted
  forward/backward influence to select trainable cell types rather than broad
  hop reachability. Its form task gives clipped distance progress, completion
  bonuses and misclick penalties; synthetic form completion is not evidence of
  Pokemon navigation. Paired held-out seeds are a useful evaluation pattern.
  We should compare learned and baseline policies on identical unseen game
  starts, not infer improvement from training reward. Source:
  https://github.com/ad7584/flycoin .
- `vaibhavkedarisetti/fruit-fly-lab`, commit
  `26672e06427c12c61536ce1bd93dae7442944681`: inspected
  `brain/sensory/encoders.py`, `brain/motor/descending.py`, and model limitations.
  Female FAFB, not Male CNS; useful separation of measured anatomy, published
  tuning and chosen encoder parameters. Explicitly no plasticity, so it is not
  a source of learned game reward behavior. Source:
  https://github.com/vaibhavkedarisetti/fruit-fly-lab .

The user-mentioned Kimchi repository has not yet been identified from web and
GitHub repository searches. Awaiting its owner or link; do not substitute an
unrelated project with the same name.

## Local pooled-brain integration

The additional seven-repository comparison and measured readout/integrator
checks are in [FLYGON_MALECNS_FOLLOWUP.md](../../docs/FLYGON_MALECNS_FOLLOWUP.md).
The source's pooled adapter is integrated with this branch's later reward fixes.
A release now requires checkpoint evidence pinning both the game and neural
binaries; menu-input migration is explicit rather than retaining legacy inputs.

## Interaction curriculum development (not yet released)

The `flygon-goal-curriculum` branch adds explicit capture intent for uncaught wild
species when balls are present. It preserves enemy HP and encounter kind in
rich-menu inputs, which previously discarded the battle overlay. Foreground
menu contexts separate main commands, moves, balls and teaching prompts.
Main battle commands use their actual 2-by-2 geometry. Ball labels normalize the
accent in `POKé BALL`; targeting a ball does not mistake an unrelated item's USE
menu for a throw.

Cursor progress is signed and sums to zero around a menu loop. Highlighting the
same item repeatedly earns nothing. A recorded ball consumption and a completed
capture have separate feedback. Knocking out the intended capture target removes
the conflicting damage/victory reward and receives an explicit failure signal.
These are engineered outcome targets; every button remains sampled from measured
neural readouts. Unit checks are not evidence of successful learned capture.

The early route curriculum has now been checked against the pinned external
content pack: every configured exit through Goldenrod exists. This check caught
and corrected the Route 34/Ilex gate name and the difference between constant
warp names and display map names. Route 32's authored checks require both the
Zephyr Badge and the Togepi Egg, so the curriculum visits Elm's aide before
leaving Violet. Authored object roles identify the charcoal maker and the final
Slowpoke Well Rocket without assuming their sprite identifiers name their role.
This is explicit navigation assistance, not a claim that those routes are learned.
The engine export compiles; live route/HM verification remains required.

HM teaching development now identifies the actual HM01–HM07 labels, tracks the
selected machine in menu telemetry, distinguishes ABLE from NOT ABLE, and supplies
separate teaching/confirmation/replacement objectives. Existing HM moves are
excluded from replacement targets. The interface reports a missing field-use
badge separately from the ability to teach the move. Intermediate dialogue pages
are not treated as menu regressions. This is not yet a demonstrated HM-use skill;
actual teaching, obstacle use and prerequisite-return trials remain outstanding.

### Additional MaleCNS comparison: TraderFly (2026-09-13)

Inspected [SotoAlt/traderfly-brain](https://github.com/SotoAlt/traderfly-brain/tree/05a61a7dad295369c0ded8f030422047d8f2dca8),
commit `05a61a7dad295369c0ded8f030422047d8f2dca8`, in an ignored checkout.
Read `packages/brain/src/lif.ts`, `packages/mechanics/src/fly.ts`, and
`packages/validate/src/experiments.ts`; did not execute upstream code or copy it.

Its MaleCNS controller resets dynamics for each 600 ms decision and maps a few
named motor/descending readouts to fixed actions. Flygon instead retains learned
policy state and pools 1,314 reachable descending cells. TraderFly therefore does
not demonstrate learning a multi-step objective or recovering from menu loops.

The useful comparison is validation: silent, sugar, combined sugar/bitter, and
looming conditions test specific responses. Its documentation reports that
bitter alone activated feeding during a large burst. More spikes are therefore
not evidence of useful aversion. Its source also documents stimulus/refractory
and integration differences from its reference implementation; its numerical
response bands cannot simply be transplanted into Flygon.

Follow-up: measure action discrimination and saturation under reward-only,
aversion-only, combined, and quiet controls, then compare actual gameplay.
The requested “Kimchi” repository remains unidentified after another exact-name
search; TraderFly is a separate repository, not an assumed match.

### HM acquisition audit while the release trials run

Read the pinned external pack's authored event checks and item sources. The
remaining curriculum must cover these prerequisites; the current release does
not yet implement this entire chain:

| HM | Acquisition target | Prerequisite to represent |
| --- | --- | --- |
| Cut | Ilex Forest charcoal master | Return Farfetch'd |
| Fly | Chuck's wife, Cianwood City | `EVENT_BEAT_CHUCK` |
| Surf | Dance Theater Surf giver | Beat all five Kimono Girls: Naoko, Sayo, Zuki, Kuni, Miki |
| Strength | Olivine Cafe sailor | Receive the gift; then select a compatible learner |
| Flash | Sage Li, Sprout Tower 3F | Complete the elder's battle and reward dialogue |
| Whirlpool | Rocket Base electrode sequence | Complete the generator sequence and Lance handoff |
| Waterfall | Ice Path 1F HM item ball | Reach and collect the authored item ball |

Acquiring a machine is separate from teaching it, having field-use permission,
finding the relevant obstacle, and completing its interaction. Merely knowing
these target names does not establish a navigable route or a learned sequence.

The five native `tmhm_` menu regressions passed against the exact realtime-clock
pack used by the `28dc7a4` evaluation bundle. They cover text/confirmation and
replacement behavior, including rejection of overwriting an HM. These are
engine tests with constructed starting states, not autonomous gameplay proof.

### Current-binary Potion and HM engine checks

Native engine tests also passed for the Farfetch'd/Cut reward sequence, all five
Kimono Girls unlocking Surf, and the Strength gift/teaching/save/reload sequence.
These checks remove specific engine-response uncertainties; they do not prove
the brain will discover or execute those sequences reliably.

### Additional MaleCNS comparison: Minecraft motor persistence

Inspected [blendi-remade/fly-brain-minecraft](https://github.com/blendi-remade/fly-brain-minecraft)
at `6cfa30175003ef25da68a237d5eda958f8047b82`; source was cloned into ignored research storage, not executed.
Its [README](https://github.com/blendi-remade/fly-brain-minecraft/blob/6cfa30175003ef25da68a237d5eda958f8047b82/README.md)
describes MaleCNS sensory populations driving an LIF network and decoding descending/motor populations.
It documents gain calibration and absent spontaneous firing rather than treating widespread activation as success.

The [motor decoder](https://github.com/blendi-remade/fly-brain-minecraft/blob/6cfa30175003ef25da68a237d5eda958f8047b82/src/main/java/com/fruitfly/brain/MotorDecoder.java)
uses an authored priority order, hysteresis, minimum dwell time (250 ms), and separate escape/flight timing.
That persistence differs from Flygon's repeated independent button choices. It suggests testing whether a learned
interaction intent should persist until completion or explicit failure; it does not justify copying fly locomotion
channels into Pokémon button mappings. Its authored arbitration also does not establish learned menu navigation.

Our exact-binary control check completed Tackle (Spearow HP 27 to 17), while the 742689e neural trial completed
zero turns in 60 samples. The remaining battle problem is demonstrated at the policy/interaction level, not resolved
by circuit activity. Selected-move contexts and safe session rotation are queued experiments, not proven fixes.
Repeated searches for “Kimchi” with fly brain/connectome terms did not identify the requested repository.

### Additional comparisons: Doomfly and Fruitless

Inspected [nftechie/doomfly](https://github.com/nftechie/doomfly/tree/71ecf53d78eaffaf1a57ed7b0ccf5d458abc9f33),
particularly `doom_learning_v5/rule.py` and `conditioning.py`. Its conditioning
assay counterbalances cue assignments and includes backward pairing, frozen
weights, no imposed reinforcement, and memory-reset controls. Its learning rule
uses network spike rates and bounded efficacy; the assay does not establish
learned game survival. For Flygon, the useful comparison is cue-specific behavior
under matched inputs and frozen learning, rather than total circuit activity.
This does not justify resetting the continuing brain or replacing its rule without
matched gameplay evidence.

Inspected [nicodunks/fruitless](https://github.com/nicodunks/fruitless/tree/0943b2c00b47a97c8e22438b11192b33d6779f56),
particularly `experiment/followup/bounded_routes.py`. It separates excitatory
input from inhibitory conductance and uses 1.8 ms delays and 2.2 ms refractoriness.
Intact and blocked mAL conditions receive identical hashed sensory schedules,
with three seeds and two inhibitory reversal settings. Its animation is authored,
so it does not demonstrate learned navigation. These controls offer a useful
wiring assay; real map transitions, interactions and milestones remain our
separate gameplay gate. No equations, weights, or assumed cell identities were
copied into Flygon. Neither source identifies the requested Kimchi repository.

### Cut prerequisite: directional Farfetch'd chase

The story curriculum now selects forward approach tiles from the visible bird
position, based on the [authored chase branches](https://github.com/pret/pokecrystal/blob/master/maps/IlexForest.asm).
Wrong sides are excluded from the interaction-ready cue. Each stage has a distinct
navigation target and facing-reward identity. Observed stage changes supply a
signed potential difference: returning to an earlier stage cancels that progress.
Animations and unchanged positions do not earn stage reward. The existing engine
quest regression covers forward legs and a backward approach; the new neural
checks cover target sides, readiness and reward cancellation. None of these checks
proves that an autonomous run has acquired, taught and used Cut.

### Additional source inspection: eganeganegan/FlyDoom

Reviewed commit `a9d9dbaa352eecf9a7c270e9d3668033140b7c77` from
https://github.com/eganeganegan/flydoom (separate from nftechie/doomfly).
Read `models/connectome_network.py`, `training/ppo.py`, `training/rollout.py`,
`env/rewards.py`, and `env/doom_env.py` under `src/flydoom/`.
The rate model keeps graph edges fixed but can train edge magnitudes and node
biases; optional transmitter signs constrain effective weights. It uses sparse
recurrent aggregation rather than our LIF dynamics and bounded dopamine-linked
plasticity. PPO stores action probabilities, values and recurrent states,
computes backward GAE with terminal masking, and logs clipping/KL/entropy.
The actual VizDoom adapter uses `make_action` rewards; the simple forward-motion
bonus belongs to the mock environment, not evidence of learned Doom exploration.
Its README describes rewired-graph and conventional-network controls. We have
not reproduced its benchmarks and found no evidence here establishing Pokémon
interaction chains or long-form story reliability. Keep our settled-action
credit assignment and paired checkpoints; do not transplant its weights or
claim its PPO result validates biological learning in Flygon. A topology/readout
comparison needs matched game inputs, rewards, seeds and interaction budgets.
The user's “Kimchi” repository remains unidentified after another search.

### Westward HM curriculum follow-up

After Surf acquisition and Morty's badge, the declared goals now continue through
Route38EcruteakGate, Route38 and Route39 to Olivine Cafe's Strength sailor.
The target uses the observed `OlivineCafeStrengthSailorScript` role; receipt of
`EVENT_GOT_HM04_STRENGTH` switches the goal to leaving the cafe. Olivine gains the
same healing/PC detour rules as the earlier cities. The authored references are
[map connections](https://github.com/pret/pokecrystal/blob/master/data/maps/attributes.asm),
[the west gate](https://github.com/pret/pokecrystal/blob/master/maps/Route38EcruteakGate.asm),
and [the HM giver](https://github.com/pret/pokecrystal/blob/master/maps/OlivineCafe.asm).
Unit coverage verifies prerequisites, observed NPC approach tiles, completion
retargeting and the healing detour. No autonomous Olivine visit, Strength receipt,
or full HM teaching/use chain has been demonstrated. Lighthouse and shore/Surf
navigation remain further work; the current opening candidate is unchanged.

### Amphy medicine confirmation

The visible Jasmine medicine question now receives a YES-alignment objective only in Olivine Lighthouse 6F, with Jasmine as the observed speaker, SecretPotion in inventory, and the return-to-gym event still incomplete. Cursor round trips have zero net shaping reward; accepting the page alone does not earn completion credit. The actual story event remains authoritative.

Primary script: https://raw.githubusercontent.com/pret/pokecrystal/master/maps/OlivineLighthouse6F.asm . The native engine test `medicine_quest_requires_the_request_preserves_refusal_and_heals_amphy` passed, including refusal and eventual healing; 138 neural library tests passed (one ignored). These are controlled tests, not autonomous lighthouse navigation or medicine completion. Navigation beyond the Strength handoff and the sea crossing still require further work and observed play.

The medicine chain now has observed-NPC approach goals in Lighthouse 6F and Cianwood Pharmacy. The pharmacist is selected only after the request and before the collection event; Jasmine is revisited with actual medicine inventory, and her gym objective opens after her return. Mineral Badge uses the engine’s index 4 (Storm Badge is 5). 139 neural tests passed, one ignored. Full lighthouse ascent/descent and sea-route guidance remain incomplete; these goal selections alone do not demonstrate either journey. Pharmacy source: https://raw.githubusercontent.com/pret/pokecrystal/master/maps/CianwoodPharmacy.asm .

### Ball-only field Pack exit guidance

The 8c92ce0 continuation accepted the aide's Egg (party addition at sample 19,
story flag at sample 20), but remained in the Center browsing Pack. The recorded
Pack actions had no objective cue: utility exit shaping excluded every Pack.
The new case supplies signed exit guidance when the party is healthy, no HM
teaching is pending, explicit key-item inventory is empty, and regular inventory
contains only balls. Unknown inventory, other items, key items, HM teaching and
battle menus retain their existing behavior. Enter/exit cycles have zero net
shaping reward. Validation: 140 neural library tests passed, one ignored.
This change has not yet been evaluated in autonomous gameplay; the Egg handoff
is one success, not repeated reliability or capture/HM completion.

### Start guard during travel with no field inventory task

The 881af85 continuation reached Route 32, then reopened Pokedex at sample 79.
A timed cooldown alone did not prevent this interruption. During a declared
travel/exit goal, Start now remains excluded after the cooldown expires when
explicit inventory contains only balls, key items are empty, the party needs
no healing, and no HM teaching is pending. Open menus retain their controls;
battle controls, unknown inventories, other field items and key items remain
outside this added guard. This is declared action filtering, not evidence that
the neural policy independently learned to avoid browsing. 141 library tests
passed, one ignored. Autonomous validation of this guard is still pending.

### Voluntary capture abandonment

The cf97e98 continuation encountered wild Ekans on Route 32 with Quilava at
46/46 HP and two Poke Balls. It opened Pack, backed out, selected RUN, and pressed
A. The engine returned battle_result 2; inventory and owned species were
unchanged. No capture-specific aversion was emitted. The new aversion requires
an active uncaught/needed capture target, A on the explicit commands-surface RUN
row, an observed overworld escape result of exactly 2, and the active Pokemon
above one-third HP. Failed escape attempts, unrelated battle endings, unknown
health, depleted ball inventory, and low-health escapes do not qualify. The
existing battle-aversion path assigns the failed-objective penalty to that
submitted action. This does not force a throw or prove capture reliability.
Validation: 142 neural library tests passed, one ignored. Live validation pending.

### Leave moves when a capture target is ready

The same continuation showed a capture pocket cue while the move list was open;
a later wild target was knocked out and the existing knockout aversion fired.
When the capture objective judges a target ready for a ball, the move surface
now targets CANCEL before guiding Pack selection. Move selection remains
available when weakening is needed. Signed progress gives no net reward for
cycling to CANCEL and back. Tests cover ready targets, weakening and trainer
battles: 143 neural library tests passed, one ignored. Autonomous capture
validation remains pending.

### Restocking along the story route

Cherrygrove, Violet, Azalea, Ecruteak and Olivine now select their mart when
explicit inventory has fewer than three ordinary balls and visible money is
at least 200. Inside the mart, the clerk is the interaction target; sufficient
stock or insufficient money restores the city exit. Existing healing goals
retain priority. Dungeon travel continues toward the next town instead of
reversing through Union Cave. Door positions still come from authored exits.
147 neural tests passed, one ignored, including replenishment, resuming story,
low funds, healing priority and dungeon continuation. Department-store routing
and live purchase validation remain outstanding.

### Controlled ball purchase

A native Azalea Mart check now talks to the clerk, selects Poke Ball, increases
quantity to three, confirms the purchase, and verifies inventory 0→3 and money
1000→400. It also asserts the observed quantity/item/600 total and the resulting
notice surface. One targeted native test passed. An ignored pre-interaction
fixture was exported for testing the same flow in the browser. This validates
the transaction engine and observation, not autonomous navigation or purchase.
The updated WASM engine compiled successfully; browser validation is next.

### Interaction menu interruption

The controlled post-opening neural shop probe targeted the clerk but opened
Pack before returning to the field, without a purchase in its first six
samples. The idle-inventory Start guard previously covered Exit goals only.
It now also covers Talk and Visit goals when the same explicit idle-inventory
conditions hold. Open menus, useful inventory, healing needs and pending HM
teaching retain their existing behavior. 148 neural tests passed, one ignored.
This is declared input filtering, not independent learning evidence. The
ongoing probe remains on its original build for an unchanged baseline.
