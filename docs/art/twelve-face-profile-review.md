# Twelve face profiles and Bugsy repaint — 2026-10-11

Production now applies the existing refined head profile to Bruno, Chuck, Bug Catcher, Bugsy, Gameboy Kid, Standing Youngster, Outdoorsman, Unused Guy, Will, both swimmers and Twin. It narrows the lower jaw and refines cheek normals while preserving UV coordinates, weights and the crown/neck anchors. The existing profile contract regression passed; the fresh production reviewer build completed successfully.

All twelve models were individually inspected in four-angle Idle, Run 0.125 and Run 0.625 images (36 images, 144 views). External captures are under `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/twelve-face-profile-{idle,run125,run625}/`. These use the production weighted mesh and proportions in the reviewer's image target. They do not establish in-game presentation or native window capture quality.

The jaw improvement does not resolve the angular lowest neck/nape strip. Hats, topknots, pigtails and backpack remain attached in sampled poses. Chuck and Outdoorsman retain hard dark beard edges. Bruno and Chuck still have shirt-shaped bare torsos and lack muscular anatomy and belts/wraps. Bug Catcher, Gameboy Kid and Standing Youngster retain generic sleeves and lack their net, handheld and ball. Unused Guy lacks suit/tie; Will lacks mask, gloves and suit. Swimmer feet retain weak arches, shelf-like heels and dark patches; female swimsuit lip and armpit transitions remain imperfect. Twin has a stiff skirt hem and lacks a ribbon. None of these whole models is approved.

Bugsy's realistic portrait atlas was replaced by a drawn anime painting after all three sampled poses were inspected. Eyes, nose, mouth, violet hair, green shirt, olive shorts and white shoes map to their intended surfaces. The new face has graphic lines and simpler cel tones. Hair remains a rigid curtain, the neck angular, hands generic, shirt sleeves unsuitable for the final costume and net/belt details absent. This is a painting improvement, not final character approval.

The built-in ImageGen edit tool produced the selected output at `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-166bf770-f77b-40b8-a8a1-55ca2243fe1c.png`. The bytes were copied unchanged to `assets/open-models/people/npc-textures/bugsy.png`; SHA-256 `e40215493c0f4fd2e0c1abcb26d6d66bd208e7d0253155ed74358498a3716a07`. A hash guard verified the replaced atlas had not changed concurrently. No CLI/API fallback was used.

After installation, a fresh canonical-root Idle capture was individually inspected at `captures/bugsy-canonical-idle/bugsy.png`. Its pixels exactly match the reviewed candidate Idle capture. The temporary candidate symlink fixture was removed. The focused source/document whitespace check passed.

Exact edit prompt:

```text
Use case: style-transfer.
Asset type: square UV texture atlas for an actual animated anime game character.
Input image: edit target, existing Bugsy UV atlas. Keep every island's exact normalized location, silhouette and boundaries. This is a FLAT atlas, never a render or character sheet.
Primary request: Replace the realistic portrait face and fuzzy realistic hair with clean hand-drawn anime art. Bugsy is a youthful male bug specialist with short violet hair, calm alert brown eyes, green shirt and olive shorts. Use restrained almond anime eyes, clear upper lash strokes, flat cel-colored irises, subtle tiny nose marks and a simple closed mouth line. Short graphic violet hair clumps with two or three broad cel tones. Clean warm skin with broad restrained cel shading; no pores, realistic lips, sculpted realistic nose, glossy cheeks, doll eyes, baby face or storybook rendering.
Constraints: face eyes/nose/mouth/ears stay centered at the existing exact atlas coordinates. Keep all UV islands including the right skin patch, white shoes, lower green torso and olive lower garment aligned exactly. Preserve the green shirt, olive shorts and white shoe palettes and neutral skin patches; retain the lower garment waistband within its existing island. No facial hair, no mask, no accessories painted over the face, no extra objects, no text or watermark. Do not change face placement or atlas composition. No photorealism. No 3D lighting baked into the facial plane.
```
