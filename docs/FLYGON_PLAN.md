# Flygon: local connectome-controlled Pokémon

Research and implementation plan, 2026-09-10, updated during the two-hour
implementation window. The full-graph browser prototype and isolated conditioning
assay now run; [measured results](FLYGON_RESULTS.md) distinguish implementation
from the still-unproven goal of learned Pokémon story completion.

## Release preparation

The 0.2.0 preview has an independent Rust/WASM 3D viewer, source-neuron connection
selection, a simplified responsive UI and native Rust assembly/verification/preview
tooling. `modpacks/flygon/manifest.json` identifies the optional distribution.
The public browser layer transports events and typed arrays; it contains no neural
solver or geometry renderer. Distribution files and review artifacts are ignored
outputs. The base game Docker image and production deployment remain unchanged.

## Current milestone

The September 11 local WASM run reached Elm's lab with actual neural button
selection and dopamine-dependent changes to existing synapses. The engineered
action-memory BCI and reward teacher are documented in the toolkit and results.
One saved-state resume is disclosed. Initial Elm navigation is complete; next
validation gates are reward-off rollouts and matched frozen-policy episodes,
then Elm interactions, Mr Pokémon, battles and broader story progression.
Pixel-only perception and biological validation remain separate research work.

## Product and operating contract

Full-screen Pokémon beside an interactive anatomical connectome. Each visitor
runs their own neural simulation and game locally. Actual simulated activity
drives both the controller and the brain display. Rewards modify existing
connections through a declared dopamine-dependent learning rule. The intended
campaign target is the League, then Kanto and Red.

The connectome supplies measured wiring, not a complete reconstruction of all
cell physiology. Sensory encoding, neuron equations, dopamine dynamics and
button decoding are explicit modeling choices. Useful learned behavior must be
demonstrated; weight changes and attractive activity displays are insufficient.

## Repositories cloned and inspected

Independent shallow checkouts are in `/tmp/flygon-research/`, outside Geothite.
Git LFS smudging was disabled. No dependencies, upstream scripts, builds or
simulations were executed. The later implementation downloaded and hash-verified the official MaleCNS
source datasets separately; no upstream executable was used to prepare them.
The checkouts are temporary research material, not application dependencies.

| Local folder / upstream | Pinned commit | Finding and use |
| --- | --- | --- |
| `shiu` — [philshiu/Drosophila_brain_model](https://github.com/philshiu/Drosophila_brain_model) | `91bdd1e7dcf193f3e7ca5a8933497fcef63b7960` | Scientific LIF reference in `model.py`, including synaptic current decay, refractory periods and stimulation. MIT code. Use as a numerical reference, not a ready learning agent. |
| `doomfly` — [nftechie/doomfly](https://github.com/nftechie/doomfly) | `71ecf53d78eaffaf1a57ed7b0ccf5d458abc9f33` | Best inspected reference for existing-edge plasticity, data provenance and honest negative results. Python/C++ whole-graph simulation; its browser is a spectator. MIT original code. |
| `fly-escape` — [dzhng/fly-escape](https://github.com/dzhng/fly-escape) | `06540138d55127f87e69f8cca9982c4f1328be90` | Actual Rust/WASM implementation, selected graph and buffered playback. Useful runtime comparison. No root LICENSE or license declaration in the inspected package/crate manifests; do not copy its code without resolving reuse rights. |
| `browser` — [snedea/flybrain](https://github.com/snedea/flybrain) | `9191824d17871b7851645782d53d23f213ddb938` | Worker/typed-array and point-rendering reference. Fixed weights; application-level food steering means observed movement cannot establish neural navigation. MIT code. |
| `fly-brain` — [lixiang1076/fly-brain](https://github.com/lixiang1076/fly-brain) | `f6d525ab10c7a3ed4853d3d8b4b5e73f24cfe52b` | `dopamine_learning.py` manually multiplies active KC→MBON weights. `fast_learning.py` records associations and post-processes outputs. Do not adopt its fast memory path as in-network learning. MIT code, separate data terms. |

### Findings that change the design

DOOMFLY's README reports 166,700 retained neurons, 25,582,938 directed edge rows
representing 124,177,617 contacts. Its current candidate changes 4,184 existing
KC→MBON11 connections. `doom_learning_v6/brain.py::step` obtains actual KC/DAN
spikes, advances `rule.py`, and writes the changed efficacies back to the graph
before subsequent neural integration. Game damage becomes stimulation rather
than a direct weight assignment. This is the causal structure to preserve.
Its visual, conditioning and survival gates failed; it is not a successful
trained controller to transplant.

The v6 rule uses activity traces, a baseline-centered anti-Hebbian drive, memory
decay and bounded efficacy deviations. It is an adaptation, not a reproduction
of the complete biological model. See `docs/doom-live-training.md`,
`doom_learning_v6/{brain,rule}.py`, and
[Huang, Luo et al.](https://doi.org/10.1038/s41586-024-07819-w).

Fly Escape's actual `data/processed/brain/manifest.json` has 70,000 neurons and
798,715 edges; `graph.bin` is 9,864,604 bytes. `crates/sim/src/lif.rs` propagates
spikes through sparse connections and exposes neural diagnostics, with fixed
graph weights in the inspected step. Its default 1 ms integration, drive/noise
choices, extraction and decoder differ from DOOMFLY. This establishes an
implementation example, not full-graph performance or reward learning.

Browser FlyBrain's `js/main.js::computeMovementForBehavior` explicitly computes
the angle to nearby food and replaces the target direction. Its worker also
uses a different, coarse time model and weight normalization. These shortcuts
cannot be included in a claim of connectome-learned Pokémon pathfinding.

## Data choice and provenance

Prefer [MaleCNS v1.0](https://male-cns.janelia.org/download/): full brain and VNC,
downloadable annotations/connectivity, and source CC BY 4.0 terms. Preserve source
IDs, source hashes, release, retention rules, neurotransmitter predictions and
uncertainties. Adopt a documented neuron-retention policy comparable to DOOMFLY
and independently reproduce counts; its counts above are upstream reports.
Do not silently threshold weak edges or substitute a smaller graph for speed.

Use a native Rust importer to produce versioned, chunked adjacency and metadata.
No Python runtime in the browser. Keep the graph outside the game executable and
the existing Crystal content pack; load it on demand and cache it locally.
Respect AGENTS.md's bundled-artifact inventory: publish separately versioned data
assets, not raw scientific dumps or new compiled artifacts in this repository.

Connectivity alone is insufficient for anatomical rendering. Import soma or
skeleton-derived coordinates and neuropil membership in the same source space.
Fetch skeleton detail only for selected cells. Missing anatomy stays marked
unknown; do not invent coordinates and present them as anatomy. Keep source IDs
as strings/u64 across interfaces rather than assuming JavaScript integer safety.

## Proposed implementation in Geothite

| Location | Responsibility |
| --- | --- |
| `crates/crystal-flygon/` | Rust neural state, sparse LIF integration, delay queue, dopamine/eligibility state, plastic weights, fixed readouts, checkpoints. Native and WASM targets. No game-state access inside the neural kernel. |
| `crates/crystal-flygon/src/import.rs` | Deterministic source conversion and provenance, independent of game pack generation. |
| `modpacks/flygon/` | Reloadable sensory, decoder, learning and reward configuration; dataset reference; documentation. |
| `crates/crystal-bevy/src/bevy_shell/webmcp.rs` | Existing game observations and ordinary joypad inputs; no game rebuild for Flygon. |
| `crates/crystal-flygon/src/{campaign,interface}.rs` | Reward evaluator, sensory prosthesis, fixed neural decoder and actual-activity pixels. |
| `web-client/` minimal glue | Worker lifecycle, transferable buffers, local persistence and fullscreen integration. No TypeScript or separate application stack. |

Run the neural WASM in a dedicated Worker with its own memory. Keep game/render
WASM responsive on the main thread. Start with one worker and transferred compact
messages; SharedArrayBuffer/threaded execution is an optional measured
optimization with its hosting requirements, not a launch prerequisite.
See [Web Workers](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers).

Use outgoing CSR and contiguous neuron arrays. Aggregate multiple anatomical
contacts into their published pair weight, retaining contact counts. Keep
plastic deltas only for identified eligible edges. Preserve delay semantics and
seeded noise. Avoid dense neuron-by-neuron matrices and per-frame graph copies.

A rough storage calculation for 25,582,938 edges at one u32 target plus one f32
weight is 204,663,504 bytes (~195 MiB), before offsets, neuron state, source
metadata, delay queues, plasticity, loading copies and the game. This is an
estimate, not measured browser memory. Source Feather connectivity is ~1.1 GB;
that need not be the final download format. Incremental decoding and bounded
queues are necessary. Mobile feasibility remains open.

Keep neural time, game time and display time explicit. Render at display cadence;
do not skip neural steps to keep the animation smooth. Synchronize action-sized
game advances with completed neural intervals. If the model is slow, show its
measured speed and slow game progress. A fixed visual fade may show recent spikes
but must not fabricate events. Never show buffered gameplay as current live play.

## Causal input, reward and action boundaries

`visible observation → sensory neurons → graph dynamics → fixed neural readout
→ Game Boy buttons → game outcome → DAN stimulation → local plasticity`

The existing `web-client/webmcp.js` and Rust `bevy_shell/webmcp.rs` already provide
observe/press interfaces through ordinary joypad input. Reuse that ownership and
human-interruption behavior. The current bridge is presentation-frame paced, so
a synchronized faster training scheduler is a later explicit change.

Start with a documented structured sensory prosthesis derived from visible
tiles/objects, facing, screen mode and visible menu/battle presentation. Show
exactly what it receives. It is not a claim of pixel-only vision or natural fly
reading. The current observation API contains more metadata than needed: allowlist
fields rather than feed all of it. Pixel/retinal encoding is a separate later
experiment, with its own result label and data contract.

No global route, shortest-path direction, hidden target coordinates, battle
solver, automatic dialogue sequence or stuck-recovery policy may feed the
controller. Calibrate a fixed neuron-population-to-button interface, including
no-op and hold duration, and freeze it during learning comparisons. The mapping
is an engineered interface; flies have no natural A button. A campaign controller
must learn menu actions as well as locomotion. Avoid using game mode to choose
the winning action outside the neural network.

The evaluator may read objective completion events for scoring. Keep that channel
separate from sensory observations and action decoding. Emit finite pulses into
source-identified DANs; plasticity consumes simulated neural activity and local
traces, not a scalar that directly edits arbitrary weights. Audit compartment
membership before selecting both appetitive and aversive pathways. DOOMFLY's two
aversive DANs alone are not a complete Pokémon reward system.

Do not assume reward globally strengthens all KC→MBON connections. Plasticity's
sign and timing depend on the circuit. Start with one source-supported
compartment and conditioning assay, then add identified compartments.
[Experimental work](https://pubmed.ncbi.nlm.nih.gov/30332645/) shows that the same
DAN can participate in both learning and forgetting depending on timing.

Reward design candidates, to calibrate rather than hard-code as biology:

- One-time story progress: starter, mandatory event, key item, badge, League.
- Small capped novelty reward for first visits, with visitation state persisted.
- Battle outcome rewards and bounded loss signals; avoid an indiscriminate HP
  penalty that rewards refusing every necessary battle.
- Small bounded collision/repeated-state penalties, after a reliable observation
  window; a stationary dialogue is not a collision.
- No repeat reward for entering the same doorway, cycling heals, loading a save,
  or restarting a session. Save event deduplication with the brain and game.

Sparse story rewards may not teach long action chains. Use a curriculum of short
navigation, interaction and battle tasks with explicit task boundaries. Carry
learned neural memory forward. Training fixtures can have declared starting
states; the final campaign must be a fresh normal game with no progression edits.
Curriculum shaping is training assistance and must be visible in the run record.

## Display and checkpoints

Large game panel, rotatable anatomical brain, and a compact timeline of sensory
input, DAN pulses, action population rates, buttons and reward events. Selected
neurons expose their source ID, voltage, spike times and changed outgoing edges.
Region brightness is an aggregation of actual member activity, not an event
animation labeled as activity. Stimulation overlays and neural responses are
visually distinct. Display source membership and sampling/windowing rules.

Pause, speed, save/export brain, restore, learning toggle and manual stimulation
are useful controls. Manual intervention marks the run as assisted. Persist game,
neural state, delays, traces, plastic deltas, decoder filters, RNG, pending reward
pulses, event ledger, configuration and dataset identity together. Reject
incompatible restores. Optional trained checkpoints run locally and keep learning;
they need a disclosed training history, not a hidden remote decision service.

## Delivery sequence and exit criteria

1. **Full-graph browser feasibility.** Rust importer/kernel and worker harness,
   source counts/hashes, bounded memory loading, pulse response and native/WASM
   agreement. Measure Chromium, Firefox and Safari on desktop before promising
   support; measure mobile separately. Report download, peak memory, neural
   seconds/wall second, decision latency and renderer responsiveness with the
   game loaded. No subset result presented as full-graph success.
2. **Live cockpit.** Attach the real game bridge, fixed decoder and spike-driven
   anatomical renderer. Every displayed spike and emitted button must trace to
   one simulator snapshot. Pause, reload and checkpoint continuation work.
3. **Causal reward learning.** Paired cue/reward conditioning with reward removed
   at evaluation, counterbalanced cues, unpaired/shuffled reward, frozen weights
   and erased learned deltas. Improvement must persist after transient currents
   and traces wash out, and depend on stored changes to identified edges. Verify
   changed output reaches the action readout; changed weights alone do not pass.
4. **Pokémon curriculum.** Bedroom exit, unfamiliar local navigation, interaction
   and menu selection, first battle, first badge. Compare trained/frozen/reset
   brains across matched starting states and multiple seeds. Evaluate held-out
   layouts or routes and report failures; do not tune on the evaluation set.
5. **Story attempt and browser release.** Fresh game through League, then Kanto
   and Red if achieved. Publish milestones, action budget, interventions and
   failure/stall history. Package as optional Flygon mode with local caching,
   resumable sessions and live configuration reload.

These are focused numerical/behavioral experiments required by the user's
request for genuine learning, not a broad routine test campaign. Builds are
needed for kernel or ABI changes; sensory mappings, reward magnitudes, display
settings and learning parameters reload without rebuilding. Stamp changes into
the run record. Do not erase memory on configuration reload implicitly.

If full-graph WASM is too slow, first optimize sparse delivery and transfer costs;
then measure an optional local WebGPU kernel against the CPU reference. WebGPU
is a different compute backend and must be labeled. Do not silently crop the
brain, stream someone else's run, or replace it with a policy. If conditioning
fails, continue investigating that failure before making a campaign-learning
claim. Browser execution looks technically plausible; beating the story through
connectome plasticity is an open research outcome.

## Current completion boundary

Implemented: pinned full-graph importer; local Rust/WASM worker; actual soma/spike
renderer with source-cell inspection; neural game buttons; live reward tuning;
brain checkpoint continuation; live view reload preserving state; explicit
offline development mode; intervention records; artificial-current
conditioning lab; reproducible counterbalanced causal assay; and an explicitly
approximate faster integration option. The toolkit is documented in
[modpacks/flygon/README.md](../modpacks/flygon/README.md).

The tiny neural crate was built only for kernel/ABI changes. UI, reward strengths,
learning parameters and integration-profile experiments did not rebuild Pokémon.
The existing game bundle supplies its ordinary game bridge. A browser audio
startup blocker was resolved through the existing trusted-gesture unlock.

The untrained decoder reached the bedroom and triggered actual outcome rewards.
Isolated memory controls passed, but natural sensory learning, transmission of
learned changes into useful actions, held-out navigation and campaign success
remain open. The calibrated BCI experiment also exposed an empty-Bag cursor
failure in the existing game observation adapter; it is recorded in the results.
The next learning work is that causal sensory-to-action connection,
not a claim that more weight changes constitute story progress. See the measured
results for failed experiments, performance limits and remaining release gates.

### September 11 story-feedback extension

Implemented explicit location, visible-character approach and capped dialogue
incentives toward Mom, Elm and Mr. Pokémon, with live tuning and persistent
anti-farming ledgers. Map identity is now a sensory feature. These rewards use
the existing PAM01/PPL101 hypotheses, not invented generic neurotransmitter gains.
The next learning gate remains reward-off behavioral comparisons: stored synaptic
change and active regions alone do not establish that the reward improves choices.
Use `tools/flygon-play.mjs` for actual ordinary-button browser evidence.
