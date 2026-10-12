# Unused Guy suit details — 2026-10-12

The previous mapped render was a plain ochre shirt without the declared suit
or teal tie. The torso-sampled uniform builder now includes this identity:
ivory shirt opening, raised lapels, teal tie, brown front opening and pocket
slits, plus a continuous hem following the sides and back. The canonical
painting is unchanged. Gentleman and Gym Guide keep their existing colors
and details. Unused Guy is added to the existing supplied-rig fit regression.

Native build 87323 passed in 1m30s. Reviewer SHA256:
`fbad1c5a874fac0110e47b839030364eac3f12ac6d7f3f19d96cff89c9a10a43`.
Capture jobs 12275, 33950 and 78956 passed with `CRYSTAL_PREVIEW_MUTE=1`, normal
production proportions and animation. All three images were individually
inspected in their four angles: twelve views in Idle, Run 0.125 and Run 0.625.
External captures: `captures/unused-suit-{idle,run125,run625}/unused_guy.png`
under `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/`.

The teal tie, shirt opening, lapels and pockets improve the suit identity.
The hem stays continuous through the sampled poses. Lapels remain shallow,
the sleeve/body silhouette is generic, the rear jacket is plain, and angular
jaw/nape, generic hands and shoes remain unfinished. No whole-model approval,
complete gait, bicycle or in-game quality claim.

Fresh Cargo session 60071 passed the supplied uniform fitted/animated torso
regression: one actual test, 2.92 seconds, 1334 filtered out, after a 7m46s
build. Unused Guy is included alongside the previously supported uniform
roles. It checks valid faces and normalized skin weights and samples fitted
geometry through sixteen Run phases. Other selected libraries had zero
matching tests; this is not a broad suite result.

WASM build 63605 and wasm-bindgen 99497 passed. Engine and glue installed
in ignored `target/outdoor-browser-review`; all five served files match
HTTP responses on port 3013. The people bundle is unchanged. Verified hashes:

| File | SHA256 |
| --- | --- |
| crystal-bevy_bg.wasm | 0cf2533d85d35ef981e161012c88a038f4c0053a63d3e240839f6d7746bbeea3 |
| crystal-bevy.js | 76a8d1fe2a4432d543ed79a1402dc7b3a79e1334e0cb1abe5615fd9d4b755767 |
| index.html | d38cc3f175c3b1848df3cdd209bcac7da7c40aca7e0479f993129cb9a1587067 |
| open-people.json | 782fa0c927ed24ea83f6acdf3fd1761ec7bd83b183226868c9b18e85517a8834 |
| open-people.json.gz | be6def987489bf85452d5c5d9864181e94f0779f68456488042924f606843006 |

All three cache references use `0cf2533d85d35ef9`. Startup mute remains enabled.
HTTP bytes do not prove a refreshed browser frame. Finished staging removed.

Separate next-action evidence: Bug Catcher and Rocker canonical atlases were
individually viewed. Both have gray separator padding above/right of the
shirt island, matching the waist strips in the Tenth motion batch. Their UVs
have not yet been changed.
