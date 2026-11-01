# Clerk, Outdoorsman and Unused Guy waist UV review — 2026-10-12

The gray waist strips came from separator padding above and beside each shirt
island. All three actual canonical atlases were individually inspected before
extending the existing complete-cloth-face UV correction to these identities.
The correction clamps the affected shirt triangles into painted fabric. It
copies geometry, normals, colors, joints and weights without moving the rig.
No painting was edited for this correction.

Fresh Cargo session 40487 passed
`supplied_shirt_gutters_sample_cloth_and_preserve_other_islands`: one actual
test, 1.77 seconds, 1330 filtered out, after a 5m47s build. It checks all 75
source roles: corrected roles sample fabric at triangle corners and interior
barycentric points, while unaffected roles retain geometry and UVs. Green and
ochre classifications were added for the three supplied paintings. Other
selected libraries had zero matching tests. This is a UV/rig regression,
not proof of whole-character quality.

Native reviewer build 14818 passed. Reviewer SHA256:
`de5c24a2840f3813affc90b7381d8c68bf67f043263ca44d66d340c3e4e92bbe`.
Capture jobs 89193, 46521 and 97951 passed with `CRYSTAL_PREVIEW_MUTE=1`, normal
production animation and proportions. Each of the nine images was individually
inspected in four angles: 36 views, across Idle, Run 0.125 and Run 0.625.
Artifacts remain external in
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/staff-gutter-{idle,run125,run625}/`.

Outdoorsman and Unused Guy now have clean side/rear shirt edges in all three
sampled poses. Clerk's gray texture strip is gone; his separate beige apron
waist band is retained. These local corrections are accepted. Remaining defects:

- Clerk: shallow apron, weak rear ties, rigid hair, angular jaw/nape, generic
  hands and trainers.
- Outdoorsman: absent vest, suspender-like backpack straps, thin flat hat brim,
  stiff beard and angular jaw/nape, generic hands and footwear.
- Unused Guy: absent suit and teal tie, generic ochre shirt, angular jaw/nape,
  generic hands and shoes.

No whole model is approved. Two Run poses do not establish complete motion,
bicycle coverage or in-game quality.

## Local served verification

WASM session 4403 failed on a concurrently edited room-renderer call to
`authored_face_color`. Current source already contained the corrected qualified
call before retry; no character change was made to bypass it. Fresh build
81045 and wasm-bindgen session 17164 passed. The engine and glue were installed
after checking that the previous served files and HTTP bytes were unchanged.
The people bundle, including the reviewed Nurse painting, remains unchanged.

| File | SHA256 verified against HTTP on port 3013 |
| --- | --- |
| crystal-bevy_bg.wasm | a5797c6020d35b32595885d3e92ca544cb1d45d49be0154965c3fef624b8366a |
| crystal-bevy.js | 6da58aebe7e19321ffe2d3332a08969846a06451a54b8a442a71df7fa5984a16 |
| index.html | 786017d1d9d2aa26e2bedff99561eab765b16a077588aa12734fb933961a2e5b |
| open-people.json | 782fa0c927ed24ea83f6acdf3fd1761ec7bd83b183226868c9b18e85517a8834 |
| open-people.json.gz | be6def987489bf85452d5c5d9864181e94f0779f68456488042924f606843006 |

All three cache references use `a5797c6020d35b32`. Startup mute is preserved.
HTTP verification proves served files, not a refreshed live-browser frame.
Finished WASM staging was removed; captures and working build cache remain
external for the ongoing roster work.
