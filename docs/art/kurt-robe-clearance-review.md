# Kurt running robe clearance audit — 2026-10-11

The three-pose character review left rear-foot clearance unresolved. A further four-angle production reviewer image at Run 0.25 was individually inspected at `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/kurt-robe-baseline-run250/kurt.png`. The raised rear foot remains within the robe silhouette in that camera view. A silhouette overlap alone is not proof of surface intersection.

The new supplied-rig regression `garment_body::tests::kurt_running_shoes_do_not_cross_robe_panels` deforms the actual source shoe triangles and production Kurt robe with the same skin matrices at 32 samples through Run. It checks shoe edges against lower robe triangles, rather than relying on valid weights, bounds or outward winding. It is a narrow physical crossing check, not whole-costume approval; it does not prove clearance for the full roster, between every sample, or for all body/cloth surfaces.

The first baseline execution ended before running the regression: an unrelated missing `mesh/dragons_den.rs` module prevented compilation. That file subsequently appeared in the shared checkout, so the same command was retried. The retry completed and confirmed a crossing at Run 6/32, shoe triangle `[377, 413, 410]` against robe panel `[797, 838, 839]`. The fresh compiled regression also ran directly and reported the same crossing (5.47 seconds; Cargo run 5.70 seconds). External logs are `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/kurt-robe-clearance-baseline.log`, `kurt-robe-clearance-baseline-retry.log` and `kurt-robe-clearance-direct.log`.

A first source trial narrowed the left/right leg blend, increased lower-leg contribution and introduced shin influence below the knee for `kurt` and `kurt_outside`. The full build completed in 11m34s and the supplied-rig check failed at Run 7/32: shoe `[281, 275, 413]` crosses panel `[797, 839, 798]` (5.83s). Log: `kurt-robe-shin-trial.log`. The ineffective trial was removed, retaining the baseline drape and the regression that exposes its known defect. No improved cloth clearance is claimed; a different geometry/drape correction is still needed. Passing requires crossing checks, garment winding/bind checks, fresh production renders for both appearances and direct visual judgement of the resulting cloth motion.

Command: `CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" CARGO_TARGET_DIR=target-bevy20-verify CARGO_INCREMENTAL=0 CARGO_PROFILE_WASM_DEV_DEBUG=0 cargo test --locked --profile wasm-dev --jobs 2 -p geothite -p crystal-bevy -p crystal-voxel-view -p crystal-render-api --all-features --lib kurt_running_shoes_do_not_cross_robe_panels -- --ignored`.

Kurt's jaw/nape, scalp painting, hands, pocket and cloth motion remain unfinished. Both Kurt contexts and the complete player/NPC roster remain in scope.

## Production-palette stride audit — 2026-10-12

The fresh source-rig regression again fails at Run 6/32 (5.04 seconds). The production reviewer also exports the refined body, costume and actual animated skin palette, including production proportions and relaxed hands. Its native executable SHA-256 was `959dad9f5f44f89a16ec32fd327d15229f2e9eaf3a99a8c335984db413fe0d97`. No audible preview was launched: every native invocation used `CRYSTAL_PREVIEW_MUTE=1`.

Reviewer-only `CRYSTAL_PEOPLE_REVIEW_RUN_WEIGHT` controls the ordinary Idle/Run blend. `CRYSTAL_PEOPLE_REVIEW_EXPORT_RUN_SWEEP=1`, together with `CRYSTAL_PEOPLE_REVIEW_EXPORT_MESHES`, exports 32 production-update palettes at each of four stride weights, then restores the requested capture pose. The restored full-stride Run 6/32 palette differs from the corresponding sweep palette by at most `7.263183610106694e-08` per matrix component (phase arithmetic rounding). This does not alter production gait.

The external diagnostic `pending-model-review/inspect-robe-stride.py` checks strict triangle intersections in both edge directions between all 1,050 refined shoe triangles with bind-space vertices below 0.27 and all 3,628 costume triangles below 1.0. Exact edge contacts and coplanar overlaps are excluded. These numbers measure sampled triangle pairs, not continuous collision freedom or whole-model quality. Both Kurt appearances give identical results:

| Run blend | Samples with crossings (of 32) | Total crossing pairs |
| --- | --- | --- |
| 1.00 | 12 | 1,473 |
| 0.85 | 12 | 1,068 |
| 0.70 | 12 | 863 |
| 0.55 | 13 | 741 |

Exports and complete per-sample records are in `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/meshes-robe-stride-sweep/`. Four-angle Run 0.1875 full-stride Kurt and four-angle Run 0.25 weight-0.55 Kurt/KurtOutside captures were individually inspected. The latter still shows the lifted foot cutting through the rear robe; reducing the whole stride is rejected as a clearance correction.

An offline lower-shell depth-envelope sweep retained the existing skin weights and apron and increased only the first 2,706 robe vertices' depth below the waist, with a smooth fade. Depth multipliers 1.2, 1.4, 1.6 and 1.8 all retained crossings at the six checked phases (6, 8, 10, 21, 23, 25); no candidate was installed. These are diagnostic deformations, not newly generated meshes or mapped-render approvals. `kurt.envelope-candidates.json` retains the exact sampled counts. The next experiment isolates knee flexion rather than suppressing the entire stride or widening the neutral garment indefinitely.

Reviewer-only `CRYSTAL_PEOPLE_REVIEW_KNEE_FLEX` blends each shin's Run rotation toward its Idle rotation before the ordinary whole-stride blend. It leaves the production default and bicycle poses unchanged. Fresh reviewer build completed in 1m52s; executable SHA-256 `f85d1caf4c951a769d48ba5b9c5673960b01af7057fec1bd47923f8c54cd76e0`. With flex 0.15, Kurt retains 20 crossing pairs at full stride (Run 21/32 and 22/32), and 17 pairs at stride 0.70 (21/32–23/32). With flex 0.0, all 128 sampled Kurt palettes have zero shoe/robe crossing pairs.

Neither knee candidate is accepted. Four-angle captures for both appearances at flex 0.15/full stride Run 0.25, flex 0.15/stride 0.70 Run 0.25, and flex 0.0/full stride Run 0.6875 were individually inspected (24 views). The gait becomes stiff, and a second diagnostic exposes worse floor penetration: over the 32 full-stride samples the refined body's minimum bind-space-deformed Y reaches -0.05015 in the original Run, -0.25494 with flex 0.15, and -0.25741 with flex 0.0. These are mesh-space minima, not game-world coordinates; they identify a regression relative to the same original body and palette convention. Body vertical placement and ankle pose must be resolved before this can become a usable robe gait. No production gait, garment or served engine was changed by these trials.
