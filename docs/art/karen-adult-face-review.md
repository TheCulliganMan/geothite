# Karen face map review

Built-in imagegen replaced the oversized round eyes and smiling childlike face
with narrower adult anime eyes, drawn eyelids, a fine nose and restrained closed
mouth. The candidate was inspected on the production model in four angles of
Idle and Run phases 0.125/0.625 before installation. All twelve mapped views were
individually inspected. The face reads more clearly and has no obvious landmark
shift. This accepts the face improvement only. The rigid silver bob, long angular
jaw, generic hands, navy trouser-like legs, white sneakers and knee-shaped skirt
deformation remain unfinished; the whole model is not approved.

Canonical asset: `assets/open-models/people/npc-textures/karen.png`.
SHA256: `b04cdb34caf1ae4ed4df6ea6dc2af0fb996c8c33767ba0b38b11f1892be32fe6`.
Canonical bytes equal the generated output exactly. Generated source:
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-b18d43c6-7000-4e9a-b8f1-1ebd8887ae37.png`.

Muted production-path captures:
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/karen-adult-face-{idle,run125,run625}/karen.png`.
Reviewer SHA256:
`627f074f60c5342c6ff0324049209f88917d790bf4345b3f38d0873ad651f0b1`.
The reviewer predates the short-skirt weighting experiment; these captures use
the existing 0.60 thigh share.

## Exact built-in prompt

Use case: precise-object-edit. Asset type: production anime game character UV albedo atlas. The input image is the edit target. Change ONLY the painted face features in the existing upper-left face island for Karen, a composed confident adult female dark-type elite trainer. Replace the enormous round toddler eyes with slender almond-shaped adult anime eyes, muted amber-brown irises, clean upper eyelids and restrained single highlights. Keep both eye centers, nose and mouth anchor locations, ears, hairline and every UV island boundary exactly fixed. Refine the nose to a fine drawn anime line rather than two cute dots. Replace the broad baby smile with a delicate closed mouth and subtle confident neutral expression. Slim understated brows and restrained adult facial planes. Art direction is hand-drawn Japanese adventure anime with crisp flat colors and light cel shading; no photorealism, pores, glossy plastic, doll cheeks or toddler features. Preserve the existing silver hair, navy wrap outfit, legs, shoes, hand islands, all plain skin patches and all details outside the face feature area exactly. Keep the square atlas layout, framing, palettes and island placements identical; do not add a scene, 3D render, labels, text, border or accessories. This image must map onto the existing 3D model without moving facial landmarks.

## Local delivery

The strict Rust bundler validated all 75 paintings and four rigs. Only Karen
changed relative to the previous served bundle; rig payloads are identical.
HTTP JSON and gzip bytes equal the staged output. JSON SHA256:
`ecad391e6fc84f1c61ea2716bd9c3710ea6a9ab693e1d326b139072caec9846f`.
Gzip SHA256:
`82c26fc049e0af653b43b29f74ef6c0e620416661bba0b8d6ec3dfee9613db82`.
This verifies delivered bytes, not a refreshed live browser frame.
