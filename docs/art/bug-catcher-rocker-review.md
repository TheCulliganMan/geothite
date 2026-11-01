# Bug Catcher and Rocker candidates — 2026-10-12

Both canonical atlases were individually inspected. Gray separator padding
above/right of each shirt island matches the waist strips seen in the Tenth
motion batch. Both identities now use the existing complete-cloth-face gutter
correction; the 75-role regression includes Bug Catcher's blue fabric and
Rocker's green fabric. Paintings remain unchanged.

Bug Catcher also changes width from 0.88 to 0.98 and head scale from 0.72 to
0.84, retaining height 1.05. The independently animated instance regression
includes him and checks the expected head scale and repeated-frame behavior.

Native build 37876 passed in 2m03s. Reviewer SHA256:
`4bb6912a56872e858d912a991c06d454301080708db00147a41bfef2b6b16675`.
Capture jobs 55577, 29966 and 95804 passed with `CRYSTAL_PREVIEW_MUTE=1`, normal
production animation and proportions. All six images were individually
inspected in four angles: 24 views in Idle, Run 0.125 and Run 0.625.
External artifacts:
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/bug-rocker-{idle,run125,run625}/`.

Both gray waist strips are gone in the sampled views. Bug Catcher's front/rear
balance is less spindly and the larger face reads more clearly; hat and hair
remain aligned. His thin side torso, flat brim, angular jaw/nape, shorts/knee
transitions, generic hands/shoes and absent net remain unfinished. Rocker's
rigid faceted orange hair, angular jaw/nape, plain green shirt, generic hands
and shoes and absent music props remain unfinished. No whole-model approval,
complete gait, bicycle or live-browser quality claim.

WASM build 60652 and binding stage 15149 passed. The first regression build
40438 failed on an unrelated missing `body` test helper. Fresh retry 52621
passed the independent animation/proportion regression: one actual test,
6.13 seconds, 1336 filtered out, after a 6m31s build. Bug Catcher is included.
Other selected libraries had zero matching tests. Fresh direct session 32874
then passed the supplied 75-role shirt-gutter regression: one actual test,
2.47 seconds, 1336 filtered out. This validates local contracts, not whole-model
quality or complete motion.

The reviewed engine and glue are installed in ignored
`target/outdoor-browser-review`. All five files match HTTP bytes on port 3013:

| File | SHA256 |
| --- | --- |
| crystal-bevy_bg.wasm | 51dbbc06d982a362d2f492a77d15ac3c0925c9f1cfe8e97a8935ecd30fa96930 |
| crystal-bevy.js | 9f6cdb1f23084cbf6df9582d8c0abf069fcf5f2f63137ff4993f14919a132e2f |
| index.html | 7918ce14e162a0b9ee52734e519d5757e5ca9dede893c2ab1107aa600d3e2afd |
| open-people.json | 782fa0c927ed24ea83f6acdf3fd1761ec7bd83b183226868c9b18e85517a8834 |
| open-people.json.gz | be6def987489bf85452d5c5d9864181e94f0779f68456488042924f606843006 |

All three cache references use `51dbbc06d982a362`. Startup mute is preserved.
The people bundle is unchanged. Finished WASM staging removed. No live browser
refresh was performed; this is served-byte evidence, not an in-game frame.
