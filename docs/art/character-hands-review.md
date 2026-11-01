# Character hand review

No complete hand or character is approved yet. All review builds and captures
run muted. Evidence lives outside the repository under
`~/.codex/artifacts/anime-character-review/pending-model-review/`.

The fitted monotone hand and circular-cap revisions each received individual
close-up review on Chris, Kris, Will, Nurse, Youngster and Black Belt. Both
players were also reviewed mounted, from front and three-quarter views.
The circular cap makes the fingertips broader and rounder, but leaves palm
ridges, dark seams and an uneven outer finger. Those defects prevent acceptance.

The circular-cap fresh full-cycle regression passed, including actual weighted
hand/grip and shoe/pedal surfaces throughout the animation. Resting-pose
preservation and white-glove mapping also passed. The separate digit-shape
regression failed on the first human-male outer pad: apex .4577726, shoulders
.44771504 and .45888197. One shoulder exceeds the sampled center, so this
asymmetry cannot be dismissed as only an overly strict curvature threshold.

GPU comparison preserves indices, UVs, joints, weights, and non-finger
positions/normals exactly. Nevertheless, circular-cap normal disagreement
increased to 247 bind and 261 posed faces, from 19 and 22 in its predecessor.
Normal disagreement is indirect evidence, not a proof of self-intersection;
the visible seams still require correction.

The next candidate widens the continuous pad-to-web blend, starts depth
rounding beyond the palm crease, and weights welded normals by corner angle.
Its native build and three capture jobs passed. All ten close-ups were
individually inspected. GPU comparison again preserves indices, UVs, skin
weights and non-finger geometry exactly; normal disagreement drops to 19 bind
and 23 posed faces. The complete animation regression remains pending.
Palm ridges, uneven outer fingers and residual seams still prevent acceptance.
Predictions alone do not establish production-render quality.

Source measurements identified another domain issue: the terminal cap is narrower
than the complete index-joint surface. All four supplied rigs were measured
directly. On the male left hand, the terminal span is about .212 versus .256
for the entire surface. Its terminal right edge is .086, while the previous
outer finger center was about .0975, outside the source cap. Measurements are in external
`supplied-terminal-finger-domain-check.json`.

The terminal-domain correction is now implemented and reviewed in ten fresh
close-ups on the same six identities. The isolated pointed outer finger is
removed in the mounted front views; palm ridges and uneven lengths remain.
The native build, three capture jobs, actual full-cycle bike contact test,
resting-pose preservation and glove mapping all passed. GPU comparison again
preserves UVs, joints, weights, indices and non-finger geometry exactly.
Normal disagreement is 18 bind and 22 posed faces, without zero-area faces.
This is local retained work for further refinement, not hand acceptance or
browser deployment.

The digit-shape check still fails on the first male outer pad. Its diagnostic
now collects shape failures across all four rigs while retaining the existing
thresholds and structural/animation assertions. That fresh audit is pending;
the preceding first-failure result does not prove the remaining rigs passed.

Four current strict-loading tests each ran one test and passed: missing
production rigs, missing native rigs, malformed optional rigs, and incomplete
texture rosters. These prove rejection of missing assets rather than old-model
substitution. They do not approve any character's appearance.
