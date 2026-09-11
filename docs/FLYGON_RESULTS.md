# Flygon implementation evidence — 2026-09-10/11

This is a measured implementation snapshot, not a claim that Flygon learned or
finished Pokémon. The plan remains [FLYGON_PLAN.md](FLYGON_PLAN.md); executable
commands and live-loop controls are in [the toolkit](../modpacks/flygon/README.md).

## Public-preview packaging and 3D view — September 11

Version 0.2.0 separates the Rust/WASM anatomy viewer from the Rust/WASM neural
worker. Pointer/touch and DOM transport are the only browser UI logic. Perspective
projection, depth shading, picking, connection extraction and rasterization are
Rust. The old duplicate projection implementation now delegates to this renderer.

A selected source MBON01 (source ID `10013`, simulation index 9) exposes 1,408
incoming and 671 outgoing drawable connections. The view draws the strongest 48
in each direction, with arrow direction, different incoming/outgoing colors and
measured presynaptic-spike highlighting. In this sample, 13 incident links lack
complete soma coordinates and are explicitly omitted. These are schematic
soma-to-soma segments, not reconstructed axons or proof of synaptic transmission.

The assembled preview loaded the entire graph with no missing assets or browser
exceptions. Chromium orbit/zoom changed the rendered pixel hash while the neural
trial counter and learned-edge count stayed fixed. Native-view frame time was
about 3.5–5.1 ms on this desktop with the selected neighborhood. Firefox rendered
at about 21–22 ms and WebKit at about 3–7 ms in the initial cross-browser review.
These are observed frame timings, not universal device guarantees. Both engines
responded to keyboard rotation and fit a 390-pixel viewport without horizontal
overflow. Chromium touch orbit and pinch also passed the render review.

The native Rust `flygon-package` command assembles and verifies a standalone
45-file distribution, including graph data and attribution. It refuses existing
output, validates profiles and source identities, and rejects hash mismatches,
missing/extra files and symlinks. Deliberately corrupting a shipped documentation
asset caused verification to fail; restoring it passed verification. The native
Rust loopback preview serves the packaged files and the existing real-time game
clock. No Pokémon rebuild or deployment was performed.

Reproduction: [toolkit](../modpacks/flygon/README.md). Review artifacts are under
`target/flygon-evidence/release/`; the optional distribution is
`target/flygon-release-0.2.0/`. Generated files stay outside Git. Public-release
preparation does not establish broader gameplay ability: the next Cherrygrove
attempt was paused in Elm's opening dialogue, with its observation and learned
brain preserved in `target/flygon-evidence/cherrygrove-pause.*`.

## Verified Elm arrival — September 11

The live local WASM controller reached **ElmsLab**, player `(4,4)`, facing Left.
A fresh game observation confirmed arrival and the controller paused automatically.
The rendered game shows Elm's opening dialogue alongside actual simulated spikes.
The run made 1,228 neural decisions and 631 reward/aversion conditioning updates;
1,784 existing plastic edges changed. All eight buttons were available throughout.

This is online teacher-assisted navigation, not demonstrated reward-off mastery.
There was one saved-brain resume at NewBarkTown `(15,6)` while upgrading the reward
teacher from Manhattan distance to observed-terrain distance. Game position was
verified unchanged. No manual directions, warps or progression edits were used.
The exported record marks the restore as an intervention. Its 449 transport
submissions count only the period after that reload; the persistent 1,228 neural
decisions cover the whole run. Summed trial time (7,241,800 ms) adds separately
reset neural probes and conditioning trials, not continuous brain time.

The action-memory interface hashes observed context plus each candidate button to
8 real KCs, measures actual MBON01/MBON11 spikes, and samples a button from their
learned response relative to naive calibration. Reward replays that cue with
PAM01; aversion replays it with PPL101. Existing KC→MBON synapses store the change.
The teacher knows exit coordinates and evaluates progress after the action; it
never supplies the decoder a direction. Broad dialogue reward initially produced
a help-message habit. Final shaping rewards changed dialogue near observed Mom,
forward transitions and decreasing observed-terrain distance, and penalizes
irrelevant prompts, retreat and no progress.

The separate `flygon-operant-assay` passes naive-uniform, reward preference,
aversion suppression, frozen-learning and memory-erasure checks. These establish
causal action-memory effects; they do not replace a matched whole-game comparison.
There is no claim of completing Pokémon or of biologically validated fly cognition.

Local evidence (ignored build output):

- `target/flygon-evidence/elm-lab.png`: verified full UI render.
- `target/flygon-evidence/elm-lab.observation.json`: actual game observation.
- `target/flygon-evidence/elm-lab.brain.json`: learned brain checkpoint.
- `target/flygon-evidence/elm-lab.record.json`: run and interventions.
- `target/flygon-evidence/operant-assay.json`: separate causal controls.

Game saving is disabled during Elm's opening dialogue, so these artifacts do not
include a new game backup at the arrival position. The live browser remains paused
there. No Pokémon rebuild was needed; only the changed neural crate was built.

## Source and runtime

Five upstream repositories were cloned and inspected; their pins and limitations
are in the plan. A new Geothite Rust implementation imports the official MaleCNS
v1.0 files, verifies their SHA-256 hashes and retains every pair connection among
neurons with assigned superclass, excluding Glia. No additional strength cutoff
or browser-only neuron subset is applied.

| Quantity | Measured value |
| --- | ---: |
| Retained neurons | 166,700 |
| Retained directed pair edges | 25,582,938 |
| Anatomical contacts represented | 124,177,617 |
| Neurons with source soma coordinates | 139,662 |
| Eligible KC→MBON11 edges | 4,184 |
| Eligible KC→MBON01 edges | 2,109 |
| Total eligible plastic edges | 6,293 |

Graph+metadata identity:
`0b9c7ba3984e0fb1425495429a15368669a808f4a41f4d0f9d24865ac392ab0e`.
Initial kernel: `flygon-lif-kc-dan-ltd-v1`. Current kernel family:
`flygon-lif-kc-dan-ltd-v2`, which adds explicit synaptic reset/refractory-input
options. The retained-current profiles keep those options off and preserve the
original dynamics. Current interface identity: `flygon-local-prosthesis-v5-dual-mbon-operant-v3`.
Checkpoints validate model, interface, graph and integration identities.
The graph binary is about 196 MiB; metadata about 17 MB. Loading copies, WASM
memory, browser overhead and the game require additional memory. The observed neural WASM linear-memory capacity was 552 MiB; total peak browser
memory has not yet been measured.

All rendered points use actual source soma coordinates. Cells without coordinates
remain simulated and are omitted from the anatomy display. Green and pink points
come from measured simulated spikes during the last integration/probe interval.
These are soma projections, not reconstructed neurites or measured ROI meshes.

## Causal artificial-current memory assay

The conditioning profile uses the entire retained graph, a smaller base contact
gain (0.02 mV), direct current to two disjoint sets of 64 source KCs, and a declared
6.5 mV-equivalent tonic current to the source MBON01 pair. These engineered inputs
isolate a plasticity mechanism. They are not natural olfaction, pixel vision or
Pokémon sensory learning.

Each paired trial applies the cue for 100 ms, then stimulates PAM01 while the cue
continues for 200 ms. Six trials are used. Evaluation clears transient voltages,
currents, traces and queued spikes, preserves learned efficacies, freezes
plasticity, applies no reward and counts actual MBON01 spikes over 500 ms.

| Condition | Cue A: two MBON spike counts | Cue B: two MBON spike counts | Changed edges |
| --- | --- | --- | ---: |
| Naive | 6 / 6 | 6 / 6 | 0 |
| A paired with reward, six trials | 4 / 0 | 6 / 6 | 66 |
| B paired with reward, six trials | 6 / 6 | 4 / 0 | 64 |
| Reward delayed 5 seconds after either cue | 6 / 6 | 6 / 6 | Small residual changes; no output effect |
| Learning frozen during training | 6 / 6 | 6 / 6 | 0 |
| Learned efficacies erased after paired training | 6 / 6 | 6 / 6 | 0 |

Both counterbalanced paired cases, delayed-reward controls, frozen controls,
erasure controls and exact checkpoint continuation passed the native
`flygon-assay` runner. Chromium, WebKit and Firefox independently reproduced the A-trained browser
result and its erasure control. WebKit and Firefox also restored the trained
response from a browser-exported checkpoint and retained it after live tuning,
without page errors. Game startup returned a valid observation in Firefox;
full game campaigns across all browsers and actual mobile-device validation
have not been completed.

This supports stored, cue-specific changes caused by the modeled KC/DAN rule.
It does **not** establish that those changes control useful game decisions. The
assay requires artificial currents; its successful readout is an MBON pair,
not the game's descending-neuron decoder.

An earlier ORN cue experiment failed specificity: paired and unpaired conditions
both altered output under the high-gain configuration. That failure is not hidden
by the successful isolated assay.

## Performance and integration choices

Measurements were made on an Apple M5 Mac with 24 GiB RAM, macOS 26.5.2. The
reference browser game controller typically took about 425–435 ms to simulate
50 neural ms (about 0.12× neural real time). Rendering and controls remain on the
main thread while integration runs in a worker. A quiescent-network timing is
not representative of this active workload.

A configuration-only native sweep stimulated every twentieth annotated
`cb_sensory` neuron at 8 mV-equivalent for 500 ms. Mean compute per 50 ms:

| Integration step | Compute | Total spikes during 500 ms stimulus |
| --- | ---: | ---: |
| 0.1 ms reference | 425 ms | 409,491 |
| 0.25 ms | 193 ms | 374,385 |
| 0.5 ms fast approximation | 115 ms | 386,941 |
| 1.0 ms | 62 ms | 239,542 |

The same isolated conditioning endpoints and all causal controls passed at each
step. That is not numerical convergence of the full active network. The 0.5 ms
option is explicitly labeled approximate, with a distinct configuration and
incompatible integration identity for checkpoint restore. The 0.1 ms reference
remains available. Activity persisted after stimulus removal in this high-gain
sweep at every step; useful stable sensory dynamics remain unresolved.

Pooling the identical dopamine-gating dot product once per MBON target preserved
all tested telemetry and checkpoint arrays exactly, including a broad-stimulus
comparison. Its wall-time benefit was not material in the active benchmark.

## Synaptic reset experiment

Inspection of the Shiu reference and DOOMFLY identified a consequential modeling
difference: clearing synaptic current after a spike and rejecting fast input
while refractory. The new flags make those choices explicit. They are enabled
in `refractory.json`, not silently applied to the retained-current profiles.

With those flags enabled, broad 500 ms stimulation at base gain 0.275 produced
143,469 spikes at dt=0.1 ms, followed by 153,079 spikes during 500 ms without
external stimulation. At dt=0.5 ms the corresponding counts were 139,693 and
153,830. Activity fell substantially relative to the retained-current model but
still persisted after input removal.

The low-gain isolated conditioning assay retained its 6/6 → 4/0 cue-specific
result and passed all controls with the flags enabled. At game gain 0.275 it
**failed** the strict delayed-reward control for cue A: naive A was 8/5 and the
unpaired condition became 8/6. This profile is not validated causal learning.
The eight-minute browser observation window reached the bedroom but emitted no
Down, B or Start inputs with the original decoder bindings; only six visited
positions were recorded. Its output interface needs further work.

## Pokémon baseline

The reference controller reached title, clock setup, Oak's introduction, naming
and the player's bedroom through its fixed neural decoder and ordinary game
buttons. First visits and repeated stationary movement attempts triggered the
actual reward evaluator. There is no route planner, scripted dialogue sequence,
battle solver or progression edit in this loop.

An initial run stalled at Wooper's cry because browser audio had not been
unlocked. A normal sound-enable gesture removed that blocker. The cockpit now
forwards its trusted Run click to the existing audio-unlock function; it does
not issue a gameplay input for this fix.

A retained-current fast-mode eight-minute observation window recorded 1,056
neural decisions, 15 visited positions and 58 evaluator events, with no badges or
party member. It stayed in the bedroom. These are descriptive observations, not
proof of learning or an uninterrupted-duration benchmark.

A separate fixed-gain decoder calibration balanced the measured directional
scores and exposed an empty-Bag cursor error in the existing game bundle:
`field Pack pocket bag:balls has no valid cursor`. Observation fails in that state
and the controller pauses. Its final experiment export also failed when asking
for that observation; it is not a completed campaign comparison. The relevant
strict cursor check is in `crates/crystal-bevy/src/bevy_shell/field_pack.rs`, reached
from the `webmcp.rs` Pack-menu observation. No fallback button script, invented
state or progression edit was used to bypass it. The game bundle was not rebuilt.

Navigation beyond the bedroom, a starter, battles, badges and learned improvement
have not been demonstrated in this snapshot. The broad controller also generates
endogenous DAN activity and changes some weights before outcome rewards occur;
those changes are not evidence of task learning.

## Live-loop and adapter checks

CSS hot reload and `view.json` palette hot reload both changed the view while a
trained brain's exported checkpoint remained byte-identical. A headed Chromium
rerun completed with no page errors. One earlier headless session reported an
`unreachable` error while the neural checks still passed; its precise origin was
not isolated, so this is not a blanket headless game-rendering guarantee.

The reward adapter was exercised against a captured real game observation with
explicit synthetic mutations. It recognized the actual Johto/Kanto boolean badge
schema, rejected repeated badge/revisit rewards, did not penalize menu-cursor
movement, and broke stationary-movement streaks on other actions. Scheduling
these reward pulses did not directly change any synaptic weight. These fixture
changes are evaluator checks, not Pokémon achievements.

The current source inspector shows actual outgoing contacts/weights (largest 64
listed, with the full count disclosed). Population totals and spikes carry their
neural time window. Decoder readouts distinguish raw per-cell spikes, gain and
weighted score. Display filters and palette changes do not change the simulation.

The explicit offline-development mode was exercised with all non-loopback
browser requests blocked. It loaded the full graph, emitted neural game inputs,
reached clock setup and reported no external requests or page errors. Neural
linear-memory capacity was 552 MiB; one steady sample reported 132 ms compute and
510 ms for the complete observation/decision/game/render loop. This is local
development mode with multiplayer off, not a change to the hosted game clock.

A browser reset check trained cue A, used **Reset brain**, then recovered the
naive 6/6 response while a marker confirmed the same game iframe survived. This
supports changing neural configurations without repeating game startup. Mode
selector navigation still reloads the whole page and is labeled accordingly.

## Remaining acceptance gates

- Show that learned MBON changes reach the fixed action decoder and improve a
  controlled decision after reward is removed.
- Establish stable, cue-sensitive game sensory dynamics; compare trained,
  frozen, reset and matched-seed agents on held-out navigation.
- Measure total peak memory and sustained game/brain performance across browsers
  and actual mobile hardware; isolate the headless rendering error.
- Save/restore game and brain atomically; current brain exports and game saves
  are separate.
- Integrate optional graph assets with attribution/caching into production
  deployment. Isolated offline development now works with an explicit local
  UTC clock; hosted game clock semantics remain unchanged.
- Fix and validate the existing game Bag cursor/observation failure before a
  longer calibrated controller campaign.
- Attempt story milestones only with an honest intervention and failure record.

## Story incentives follow-up (September 11)

Added an explicit reward curriculum for Mom's floor, New Bark Town, Elm's lab,
Route 29, Cherrygrove City, Route 30 and Mr. Pokémon's house. Location arrival,
strict best-so-far distance to a matching visible named character, and capped
visible dialogue novelty schedule appetitive DAN current. No route or selected
button enters the reward evaluator. The sensory interface now includes a hashed
map-name feature; interface identity is `flygon-local-prosthesis-v3-story-feedback-v1`.

A Chromium/full-graph worker check with synthetic observations passed nine
criteria: arrival pulse, closer-character pulse, no pacing reward, first dialogue
reward, no repeated-page reward, no revisit reward, all three target locations
recognized, no direct weight write on reward scheduling, and actual PAM01 spikes
after integration. These observations were artificial evaluator inputs and are
not gameplay achievements. The first 50 ms produced no changed weights in that
quiescent probe; dopamine activity alone is not proof of an association.

The first five-minute headed gameplay run issued 601 evaluated decisions,
visited 18 recorded positions, and generated 33 outcome events. The neural model
had 2,394 changed eligible connections after 30,100 neural ms. It reached the
bedroom, then reproduced `field Pack pocket bag:balls has no valid cursor`.
The rendered anatomy visibly contained active cells; the last window reported
418 KC spikes, one PAM01 spike and two PPL101 spikes. No target story location
was reached. Much of the weight change occurred before overworld rewards, so
these counts are not evidence of learning the story curriculum.

Added bounded sensor-dropout handling in browser transport. An explicit injected
outage using a captured observation as a labeled test fixture produced 24 neural
button submissions with unknown outcomes, 49 failed observation/press responses,
zero rewarded outcome events, and an automatic pause at the limit. Restoring
observations allowed the loop to resume. This is a controlled fault-injection
check, not an additional gameplay achievement or a repair to the Bag engine bug.

A second six-minute headed run completed without observation failures or browser
page errors: 793 evaluated neural decisions, 25 recorded positions, 36 outcome
events, and 2,365 changed eligible connections at about 39.65 seconds of neural
time. The final default-palette canvas contained 29,300 active-neuron pixels and
105 dopamine-neuron pixels; these are rendered pixels, not neuron counts. Its
final position was PlayersHouse2F (2,5), facing Down. No target story location
was reached. This run did not encounter an outage, so it does not establish that
the Bag error itself can be escaped by the recovery behavior.

During stale-sensor integration the browser worker now temporarily freezes
plasticity and restores the configured learning setting afterward. A repeat of
the controlled outage check passed with this behavior: 24 unobserved neural
submissions, zero outcome events and a bounded pause, followed by successful
resume. Normal fresh observations continue using the configured plasticity.
The cockpit also reports remaining scheduled PAM/PPL current duration separately
from measured spike counts. No Pokémon build was performed in this follow-up;
the one neural-crate build took 1.86 seconds.

## Story teacher and handoff validation (2026-09-11)

The default operant teacher is now `story-events-v1`. Route ordering, coordinate
incentives, Manhattan fallback, arbitrary menu penalties and the Route 30 stop were
removed. Read-only game telemetry supplies completion flags, scene changes, key
items, machines, successful field moves, badges, caught species and battle results.

Eight Rust checks pass, including story/field-move coverage and viewer invariants.
The full retained-graph causal assay also passes eight controls: naive uniformity,
reward preference, aversive suppression, erasure, frozen probabilities/weights,
and a synthetic late-story completion passing through `operant_feedback` to actual
PAM01 spikes and changed existing synapses. This last check is adapter/mechanism
validation, not a claim that the game defeated Red.

The actual Chromium handoff run used trusted manual input, then Run brain. It
recorded 13 neural button submissions, zero observation failures and zero unobserved
actions, and advanced the actual game from intro to title. Game observations
contained reward telemetry version 1. The render review loaded the brain on entry,
selected 96 actual connections, rotated without changing neural metrics, and fit a
390px viewport. No page errors or missing assets occurred in the headed review.
Headless Chromium on this host lacked WebGL; use the documented headed review.

Evidence: `target/flygon-evidence/story-release/` (ignored, not shipped). The existing
86 browser regression checks passed. These validations do not establish autonomous
full-story completion or reward-off story performance.
