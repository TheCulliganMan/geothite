# MaleCNS readout and animation follow-up — September 13, 2026

Seven additional repositories were inspected at the commits below. Only source
was read; no upstream simulation, installer or data-preparation script was run.
These are references, not dependencies or copied implementations.

| Repository | Commit | Source inspected and implication |
| --- | --- | --- |
| [nicodunks/fruitless](https://github.com/nicodunks/fruitless) | `0943b2c00b47a97c8e22438b11192b33d6779f56` | `experiment/run.py` records sparse 10 ms spike bins; `connectome.js` displays recorded bins with afterglow. Use measured temporal information for animation. Its illustrative movement is not evidence of learned control. |
| [DenisSergeevitch/desktop-fly](https://github.com/DenisSergeevitch/desktop-fly) | `32b00011e83c3dc85fa3ea0b3934155b04f1635d` | `Locomotor.swift` groups actual circuit cells into population-rate motor readouts; `data/LOCOMOTOR_PROVENANCE.md` describes a bounded MaleCNS subgraph and modeled interfaces. Population pooling is a useful engineered adapter, but does not establish physiological validity. |
| [Mukhsin0508/cerebra](https://github.com/Mukhsin0508/cerebra) | `9a8d7623dc3eac0dbe6964598db54f938aaaa8d0` | `web/src/components/cerebra/atlas.ts` separates overview geometry from selected source detail. The README explicitly identifies traveling pulses as visual aids. Keep Geothite's measured spike display distinct from schematic connection travel. |
| [blendi-remade/fly-brain-minecraft](https://github.com/blendi-remade/fly-brain-minecraft) | `6cfa30175003ef25da68a237d5eda958f8047b82` | `LifNetwork.java` integrates exponentially decaying synaptic current analytically and tracks activity on a thresholded MaleCNS graph. `BrainRunner.java` separates neural and wall time. Its numerical treatment motivates the optional Rust current integrator; its graph threshold and body rules are not adopted. |
| [emberian/chreatures](https://github.com/emberian/chreatures) | `c30fbccb541ae5bc2cbdb88f7ef38848c611a8e6` | `chreatures/cns_adapter_contract.py` makes neuron/adapter dimensions explicit; `connectome_controls.py` implements degree/strength-matched rewiring controls. The README distinguishes initialized CNS dynamics from demonstrated useful behavior. Simple transmission ablation is added here; a degree-matched null remains future work. |
| [ad7584/flycoin](https://github.com/ad7584/flycoin) | `b8164227c862704a1fd20b517a4bc0031e05dd21` | Only neural files `flysim.py`, `lesion.py`, and `validate.py` were inspected. The lesion comparison asks whether surrounding circuitry actually contributes beyond a short reflex. Geothite now checks sensory responses against quiet and disabled-transmission controls. No financial integration was used. |
| [TuragaLab/flyvis](https://github.com/TuragaLab/flyvis) | `92b3845cc426dd309a1a0e1b3890156c42e14021` | `flyvis/network/dynamics.py` separates fixed synapse counts/signs from effective strengths and models graded recurrent activity. This is a visual-system research reference, not MaleCNS or an LIF implementation. It supports treating voltage readout as an explicit model choice; none of its trained parameters are imported. |

## Changes

The old circuit selected at most 256 descending cells. The new circuit retains
every descending cell reachable along nonzero-conductance directed edges within
the existing 12-hop sensory search. Type/side round-robin ordering distributes
these cells across 256 policy features. Each pool sums measured spike/voltage
signals, divides by the square root of its membership, then the feature vector
is normalized as before. Silent signals stay zero. Pool collisions can lose
information; broader coverage is not proof of improved task performance.

The graph, sensory population cap and synaptic learning rule are unchanged.
The policy now uses a different feature mapping, so interface identity
is `flygon-connected-sensory-descending-actor-critic-v4-pooled`. Previous policy
checkpoints are rejected instead of being silently interpreted under new pools.
Decision reports include every readout cell and its pool, and the UI reports the
fraction of all retained neurons that spiked during the measured window. Spiking
fraction is not CPU utilization or a measure of cognition.

Animation exports one bounded record per spiking cell: index, total window count,
and measured last-spike position within that window. Rust replays those last
spikes over 800 presentation milliseconds, with afterglow. This preserves their
ordering but is not a complete spike-train replay or neural real time. Connection
traces begin with the source flash; their speed and soma-to-soma curves remain
illustrative. Reduced-motion mode displays the static window summary.
Background links are dimmer than selected connections, and their endpoints no
longer glow simply because the presynaptic cell fired.

Story mode now enables `exponential_current_integration`. For a passive membrane
with time constant `tm` and a decaying current with time constant `tc`, its voltage
contribution is `current * tc/(tm-tc) * (exp(-dt/tm)-exp(-dt/tc))`. The equal-time-
constant limit is handled explicitly. This applies to synaptic and adaptation
currents; constant external drive keeps its exponential membrane solution.
Threshold crossings, refractory behavior and delivery delays remain discretized.
Existing lab profiles default to the original endpoint-current approximation.
Changing this mode requires reset, and cross-mode checkpoint restore is rejected.

## Reproduction

```sh
cargo test --locked -p crystal-flygon --lib
FLYGON_TEST_DATA=/path/to/external/prepared cargo test --locked -p crystal-flygon --lib \
  full_graph_pooled_coverage_and_timed_activity -- --ignored --nocapture
sh tools/flygon-build.sh
```

The explicit full-graph check validates readout coverage and disjointness,
timed-record/count agreement, changing rendered frames, unchanged brain state
after rendering, and exact checkpoint continuation. It uses a controlled sensory
stimulus, not a Pokémon campaign. External data and generated build products
remain ignored and are not deliverables.

## Validation reported by the source checkout

- Full external graph: **1,314 descending readouts**, compared with the previous
  256-cell cap, pooled into the same 256 policy features.
- The controlled 150 ms broad-input trial produced **314 spiking neurons** in
  integrated-current mode (305 with the previous approximation). This is a
  stimulus response, not an optimization target or a gameplay result.
- Eight disjoint sensory-channel trials from rest produced distinct pooled
  responses: maximum pairwise cosine **0.35478738**. Quiet input produced zero
  readout; disabling graph transmission left sensory spikes but zero readout.
  This establishes transmission dependence, not a benefit from the particular
  biological topology over a matched rewired graph.
- Rendering preserved the complete brain checkpoint; checkpoint continuation was
  exact. Timed display records retained every window spike count.
- The rebuilt WASM passed a Chrome browser harness using the real neural and
  renderer workers and full external graph: animation changed frames, reduced
  motion was static, keyboard orbit worked, and the checkpoint stayed identical.
  No page/worker errors were reported. The harness checks the brain/viewer;
  no complete game bundle or Pokémon campaign was run in this checkout.
- All **18 non-story unit tests** passed; the explicit full-graph test also passed.
- Passive-cell tests at 0.1, 0.5 and 1 ms steps, including equal and nearly equal
  membrane/synaptic time constants, agreed with the analytic 10 ms response to
  within **0.00015 mV**. Cross-mode live tuning and checkpoint restore were rejected
  without changing state.
- The full library run has three pre-existing story-reward failures:
  `healed_blackout_does_not_reward_respawn_location`,
  `nondamaging_battle_turns_buy_time_without_rewards`, and
  `exploration_and_backtracking_have_no_distance_cost_but_loops_expire`.
  Running `story.rs` extracted from unchanged Git HEAD independently reproduced
  all three. These reward rules were not changed in this follow-up.

No trained-versus-untrained Pokémon navigation improvement has been established.

## Integration into the release branch

The brain change was copied from the local `/Users/ryanculligan/GitHub/geothite`
checkout (base `5fb09b2`) without changing that checkout. The release branch keeps
its newer story feedback, menu exclusions and signed navigation rewards. Its
library suite passes 57 tests, with the external graph test run separately and
passing. The three failures listed above belong to the older source checkout;
they are not failures of this merged branch.

Fresh pooled-brain gameplay trials are being evaluated separately. Passing the
neural tests does not establish Route 29, gym, capture or HM reliability.
