# Native sprite reduction review

Preserve the original source sheets and existing manifests. The user rejected
the giant-eye Casserogre revision: changing eye anatomy is not a conversion fix.
The source-center batch across 23 other families was also rejected as a general
solution. Only the explicitly preferred MN_F134 comparison is selected.

The opt-in Rust `coverage_preserving` sampler works from the original source
sheet at the existing crop and fit. It uses palette-coverage error diffusion,
bounded to neighboring footprints with evidence for the same ink. It then
retains small enclosed light components and source-backed dark centers together.
This addresses the case where a reduced eye loses either its white edge or its
pupil. Default conversion behavior remains unchanged.

Original-source candidates:

- `sprites/mn-f032/coverage-downscale-20261103-v1/manifest-candidate.json`
- `sprites/mn-f043/coverage-downscale-20261103-v1/manifest-candidate.json`

Native candidate boards are ignored review output under
`target/sprite-coverage-study/MN_F032-v2/line.review.png` and
`target/sprite-coverage-study/MN_F043-v2/line.review.png`.

Nine converter tests passed, including a source-backed Wreckquiem regression
requiring both eye white and a dark center. Six candidate views independently
matched source hashes, crop/fit, indexed rows, palette colors, PNGs and native
2bpp tile encoding. These checks establish conversion consistency, not acceptance
of all visual details. No pack rebuild or deployment was performed. The rest of
the sprite refinement tasks remain open.

The user rejected the first coverage candidate's eye readability: a few retained
white pixels did not preserve Wreckquiem's original bounded eye. The next
iteration quantizes the small enclosed feature's bounds together with its dark
center, so its light pixels form a recognizable connected shape. Do not select
the earlier coverage candidates merely because their encoding tests passed.
