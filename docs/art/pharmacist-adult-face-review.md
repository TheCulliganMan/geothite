# Pharmacist mature anime face review

Mode: built-in Imagegen edit. Source atlas was inspected before editing.
Generated asset: /Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-3974acfe-50ba-485e-937d-639b8e48d806.png.
Selected canonical atlas: assets/open-models/people/npc-textures/pharmacist.png.
SHA256: a0700936a1a486348e7e22a9a0b761614094f5dab7e833c4c446c93b96ed27be.
Canonical bytes exactly match the selected generated PNG. Previous atlas is
backed up externally in pending-model-review/pharmacist-before-adult-face.png.

The candidate was rendered on the production model in four angles of
Idle and Run 0.125/0.625. All three captures exited zero; all twelve views were
individually inspected using reviewer SHA256
00c369453b6fe538c8c8b5ae9053c872da4cc7485a3eb3721767c361f908583b.
Artifacts: /Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/pharmacist-adult-{idle,run125,run625}/.
All previews were silent. Narrower drawn lids and age lines make the face read
as an older adult while existing glasses and moustache remain readable.
Face-only replacement accepted. Long angular jaw/nape, painted side hair,
generic hands/shoes, robe knee/foot folds, under-hem leg layers and missing
medicine bag remain. No whole-model approval or live browser render claim.
Clothing/shoe/hand UV layout was visually retained; no pixel-equality claim is
made for the generated non-face regions. Rust bundle creation and validation
passed in session 42962. Browser installation remains pending.

Exact prompt:

```text
Use case: precise-object-edit
Asset type: existing game character UV texture atlas, square, same island layout as the input.
Input image: edit target, Pharmacist atlas. Redesign only the central face artwork in the upper-left head island, keeping its placement and eye/nose/mouth alignment for the existing 3D UVs. This must read as a mature elderly man in clean hand-drawn Japanese anime art, with narrow relaxed almond eyes, small restrained dark pupils, defined upper eyelids, subtle lower-lid age lines, gray brows, a simple angular drawn nose and a quiet closed mouth. Preserve warm skin tone, bald scalp and gray side hair. Use restrained flat cel shading and clear linework, not photorealistic skin or rendered facial volume. The current round baby-like eyes and cute button nose must be replaced. Do not paint glasses or a moustache: the model already supplies those separately.
Constraints: preserve exact square canvas and all UV island boundaries. Keep all white clothing, buttons, brown shoes, hands, surrounding skin/scalp, and hair artwork in their original locations and unchanged. Do not produce a portrait, scene, posed model or UV wireframe. No new text, labels, watermark or background. No huge glossy doll eyes, childish blush, plastic toy rendering, realistic pores or uncanny photorealistic face.
```


The validated bundle is now installed in the local served root. Exactly one
painting changed (pharmacist); all four rigs and the other 74 paintings are
identical to the previously served bundle. HTTP bytes match both installed
files. JSON SHA256: 44d0efe4e36c929739514b53955bd546f7e7ae5bc931b9498ed8e29c1bfa0210.
Gzip SHA256: dcc0f72393b338a8e8f10ebcd9f8b80a44c248c6831d5f9d5773622c41e7cabf.
Temporary trial asset root and bundle staging were removed after verification.
No refreshed live browser frame is claimed.
