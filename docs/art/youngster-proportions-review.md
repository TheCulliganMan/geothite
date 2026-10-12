# Youngster and Standing Youngster proportions — 2026-10-12

The seventh motion batch showed small heads and narrow scaled-adult bodies on
both boys. Standing Youngster also showed a pale rear/side waist strip. Its
actual atlas was viewed and contains gray separator padding above and beside
the blue shirt island.

An offline head-scale 0.84 trial was individually inspected in both four-angle
Idle images at `captures/youngster-head084-idle/{youngster,standing_youngster}.png`.
The head/body balance improved, but the bodies remained too narrow. The next
source candidate sets only these two appearances to head-joint scale 0.84
(previously 0.72) and root width factor 0.98 (previously 0.88). Height stays 1.05.
Animation rebuilds the head scale each frame, avoiding accumulated scaling;
all child head attachments inherit it in the same hierarchy. Other characters
retain their existing proportions.

Standing Youngster was added to the whole-shirt-face gutter guard. The guard
duplicates affected triangles and clamps their cloth UVs inside the blue fabric
rectangle. Geometry, normals, colors and skin attributes are copied unchanged;
adjacent head, hands, shorts and footwear islands are not remapped. Neither
painting was edited.

Fresh native build passed in 1m16s; reviewer SHA256:
`4c465d0083f58a772de038d02675e92d3e953a79cc534e3da6334cc39997a02f`.
Default production animation/proportions were used for the final candidate,
with no head-scale override, stride or knee-flex experiment. Every capture
set `CRYSTAL_PREVIEW_MUTE=1`.

Each of the six images was individually inspected in its four angles:

- `captures/youngster-proportions-idle/{youngster,standing_youngster}.png`
- `captures/youngster-proportions-run125/{youngster,standing_youngster}.png`
- `captures/youngster-proportions-run625/{youngster,standing_youngster}.png`

These paths are relative to
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/`.
The 24 inspected views show a better head/body balance and less spindly front
and rear silhouettes; Standing Youngster's pale waist strip is gone in all
three poses. Drawn faces remain readable, and shorts/shoe paintings retain
their alignment. The changes are accepted local improvements, not whole-model
approvals. Thin side torsos, rigid hair, angular jaw/nape and shorts/knee
boundaries, generic hands and shoes remain. Standing Youngster's held ball is
still missing. Two Run samples do not prove the complete gait or bicycles.

The supplied instance-animation regression now includes both boys and checks
independent joint palettes, stable repeated-frame proportions, wrist/finger
inheritance and cleanup. The existing 75-role gutter regression includes the
new affected role and checks interpolated cloth samples, preserved skin/UV
attributes and unchanged islands on unaffected roles. Fresh instance regression
passed (one test, 1329 filtered out, 4.67s test execution after a 6m06s build).
The other selected libraries had no matching tests. The gutter regression was
then run against that same freshly compiled test executable and passed (one
test, 1329 filtered out, 1.31s). This is character rendering/skin evidence, not
a broader gameplay test.

Browser build passed in 3m17s and wasm-bindgen completed. Engine, loader, page
and unchanged people JSON/gzip bundle were verified byte-for-byte over HTTP
after deployment on port 3013. Current engine SHA256:
`49c65d3fc68003473c6b5ca68e376e46743745243c252a8a2f8ba3d25d3647f3`.
Loader SHA256:
`692e5ea8318d5a916a6ac2929c81eb5c2f8cb35be79f3fc14eb49a1263d2f874`.
All three page engine references use build key `49c65d3fc6800347`.
People bundle remains
`8ab02fdd33909f15449a124a5e94abf49bf0de5b2248cc4fd04b8d21d197de05`.
Startup mute is preserved and temporary wasm staging was removed. The user's
already-open browser frame was not refreshed or visually verified.
