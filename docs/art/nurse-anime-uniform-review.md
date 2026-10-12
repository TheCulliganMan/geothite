# Nurse mapped face and uniform review — 2026-10-12

The canonical Nurse painting now uses narrower drawn anime eyes and a restrained
closed mouth. Production short-sleeve UV mapping samples white shirt fabric on
the upper arms and skin on the forearms; the costume skirt samples the actual
white shirt color instead of the previous cream constant. This is incremental
acceptance of those corrections, not whole-character approval.

Built-in Imagegen edit mode was used with the existing canonical atlas as the
single referenced image. No CLI or post-generation image editing was used.
The exact generated PNG replaced `assets/open-models/people/npc-textures/nurse.png`.
Both old and new images are 1254 × 1254. The requested preservation outside the
face is a prompt constraint, not a claim of pixel equality.

- Original SHA256: `a70ef30e97c9770d25b6ad357e5777d90043c03ef0baa594f44d690c7e3b72e1`.
- Canonical SHA256: `9d41e26f09c033afd5dc44266f4143a42413f9088aad727bbd844e4012efc895`.
- Generated file: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-f31c002d-014b-4c0a-b01a-9cd744c4aee2.png`.
- External backup: `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/nurse-before-anime-face.png`.

## Exact generation prompt

```
Use case: precise-object-edit. Asset type: existing production Nurse character UV texture atlas, not a portrait, scene or character sheet. Input image 1 is the exact edit target. Change only the front facial artwork inside the existing pink hair and ears in the upper-left head island. This adult female anime adventure-game nurse needs warm attentive horizontally drawn eyes with tapered angular corners, clean dark upper lids, restrained eyelashes, small warm reddish-brown irises with one tiny highlight and readable white sclera. Reduce eye vertical opening to roughly half the current round doll-eye height while keeping both eye centers and spacing at their exact UV anchors. Replace the tiny plastic smile with a restrained friendly closed-mouth line and use a tiny graphic nose mark, flat warm skin base and two restrained cel shadow tones. Keep face boundary and eye/nose/mouth centers precisely positioned for the existing mapped head. Preserve pink hair, hairline, ears, skin palette, white uniform, pink sleeve/collar trim, buttons, pockets, white skirt and shoes, hands, limbs and all other UV islands at their exact existing coordinates and layout. Do not move, rotate, resize, stretch or rearrange any island, crop the image, add a face or change the opaque background. Keep original square dimensions. Avoid baby-face softness, circular glossy eyes, plastic doll styling, photorealism, realistic skin texture/pores, airbrushed blush, realistic nostrils, realistic eyelash hairs, 3D-rendered facial shading, text, labels, watermarks or added objects. Outside facial artwork faithfully retain the existing atlas.
```

## Mapped verification

Reviewer SHA256: `7324e5f6d68bff66dff0dccf691250e303f56cab71833627a748265aef858c11`.
Native build 51057 passed. Capture jobs 57882, 24273 and 11141 passed with
`CRYSTAL_PREVIEW_MUTE=1`, production proportions and normal production animation.
The three images in external `captures/nurse-face-sleeves-{idle,run125,run625}/`
were individually inspected in all four angles: twelve views. The face reads
more clearly, the raised-forearm pink patch is gone and the skirt color fits
the shirt. The flat cap, rigid faceted pink buns, angular jaw/nape, scalloped
moving skirt with trouser-like underlayers, generic hands and trainers remain
unfinished. These samples do not prove a complete gait or in-game quality.

Fresh Cargo session 7653 passed the supplied-painting short-sleeve regression:
one actual test, 0.31 seconds, 1330 filtered out. The test includes Nurse and
four existing roles, checks sleeve/skin coverage on both arms, and preserves
positions, normals, colors, joints, weights and unaffected UVs. The other
selected libraries had zero matching tests; this is not a broad suite result.

## Local served update

Browser build 33728 and wasm-bindgen session 42070 passed. Rust bundle job
99072 passed. The old/new bundle comparison proves four identical rigs and
74 identical paintings, with Nurse the only changed painting. Both bundle
forms were validated before installation in ignored `target/outdoor-browser-review`.
All five files below match their HTTP responses from port 3013:

| File | SHA256 |
| --- | --- |
| crystal-bevy_bg.wasm | f4bae98d72e1ceebfced0bfc1664b31962f0a762b14dd36c1c2544b1f1716c8d |
| crystal-bevy.js | 202a107bab3e6db1b8cbc246273657c2610b17a459129418cee521573698493e |
| index.html | f574ba019037ed5dd0aebf784258b58c04505d5a8e1d2906d9607fa3a225a710 |
| open-people.json | 782fa0c927ed24ea83f6acdf3fd1761ec7bd83b183226868c9b18e85517a8834 |
| open-people.json.gz | be6def987489bf85452d5c5d9864181e94f0779f68456488042924f606843006 |

All three engine cache references use `f4bae98d72e1ceeb`. Startup mute remains
enabled. No system volume or personal-app audio was changed. HTTP verification
proves served bytes; the already-open browser tab was not refreshed or inspected.
