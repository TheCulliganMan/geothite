# Cal jacket construction review — 2026-10-12

Cal's `jacket` appearance previously produced no separate costume mesh. The
mapped body rendered as a plain red long-sleeved shirt with backpack straps.
`uniform_details::build` now constructs a continuous fitted waist hem, front
zipper opening and pull, two collar folds and diagonal pocket openings. The
hem follows sampled torso sections around the sides and back; front details
inherit the source triangle's skin influences. The painting is unchanged.
This change applies only to Cal, not to all appearances labeled `jacket`.

Silent native reviewer build completed successfully in 1m26s. Executable SHA256:
`4e4c111be97165dc0abc506305328627bf031bddeec0e9ff6a0bc1dc0ddb5966`.
Default production proportions and animation were used, without experimental
knee-flex or stride controls. `CRYSTAL_PREVIEW_MUTE=1` was set for every run.

Each of these three images was individually inspected in all four angles:

- `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/cal-jacket-idle/cal.png`
- `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/cal-jacket-run125/cal.png`
- `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/cal-jacket-run625/cal.png`

The zipper, collar and pockets now distinguish the front as a jacket, while
the hem remains continuous in the side and rear views and follows both Run
poses. This is a local clothing improvement. The collar remains a shallow
surface detail; jacket sleeves/cuffs still use the original body shape.
Thin cap/brim, rigid hair, angular jaw/nape, generic hands and shoe construction
remain unfinished. Cal's whole model is not approved. The twelve views do not
prove complete gait clearance or a live browser presentation.

Cal was added to the existing supplied-rig uniform regression, which checks
valid triangles/winding, normalized skin influences and garment/body fit
through sixteen Run samples. The fresh regression passed: one test, 1329
filtered out, 1.32s test execution after a 5m15s build. The other three selected
libraries had no matching tests; this does not establish broader gameplay
correctness.

The browser build passed in 2m47s and wasm-bindgen completed. The engine,
JavaScript loader, page and unchanged JSON/gzip people bundle were checked
byte-for-byte against HTTP responses from port 3013 after deployment.
Current engine SHA256:
`9513db2ad65b592aae2d479ce2827c83214ce9675630539f981d4a59d5bbd9cc`.
Loader SHA256:
`cf3328c23768f77bc545cc14b529522ca4f2af85dfb02b90b3cb4d5109770293`.
Page uses build key `9513db2ad65b592a` in all three engine references and keeps
startup mute enabled. Painting/rig bundle SHA256 remains
`c6b0465bc728eb6d48e27b7a902a55fb1280a2316f85bf62d06c4dc2777ab01f`.
Temporary wasm staging was removed. The user's already-open browser frame
was inaccessible and was not refreshed or visually verified.
