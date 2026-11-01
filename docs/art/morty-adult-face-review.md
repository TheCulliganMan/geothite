# Morty face map review

Built-in imagegen edited the canonical UV atlas to replace the oversized round
eyes and smiling childlike face with narrower adult anime features and a calm
neutral expression. The square island layout, hair, scarf, clothes and plain
skin patches remain in place. No photorealistic detail was requested.

The candidate was rendered through the production people path with the external
candidate artwork root before copying it into the repository. All twelve views
in Idle and Run phases 0.125/0.625 were individually inspected; all three capture
processes exited zero. Eyes and mouth read clearly on the mapped face. The rigid
hair, angular jaw/nape, generic body, hands and shoes remain unfinished. This
accepts the face improvement only, not the full model.

Canonical asset: `assets/open-models/people/npc-textures/morty.png`.
SHA256: `e25d78e5694c997e90c4e08bd5fceac4fa78ab38a04ab6d3fdc4ebf070ea81c6`.
Canonical bytes equal the generated output exactly. Generated source:
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-db35a55a-ffa5-4b82-b02b-ede445957c5b.png`.

Mapped captures: `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/morty-adult-face-{idle,run125,run625}/morty.png`.
Reviewer SHA256: `5a6b3085b2c7b3dd2d041b5031bd7ba2977bb061756c0051511998c5e578f0a8`.
This reviewer predates the latest arm/foot changes; Morty uses neither of those
role-specific changes. All captures were muted.

The Rust people bundler produced four rigs and all 75 required paintings.
Compared with the previously served bundle, Morty is the only changed painting
and the rig payloads are identical. The local server's JSON and gzip bytes equal
the staged files; JSON SHA256
`2232752a323da34326537c5cf5905b348deb9ca1243e6e1c912c4c025f231131`.
This verifies delivered asset bytes, not a refreshed live browser frame.

## Exact built-in prompt

Use case: precise-object-edit. Asset type: production anime game character UV albedo atlas. Input image is the edit target, not a scene reference. Revise ONLY the face features in the existing upper-left face island for Morty, a calm young adult male ghost gym leader. Replace the huge round toddler eyes with narrow almond-shaped confident adult anime eyes, muted grey-green irises, stronger clean upper eyelid lines and restrained single highlights. Keep both eye centers, face placement, ears, hairline, skin silhouette and every UV island boundary exactly fixed. Give him a composed introspective neutral expression with a fine straight closed mouth instead of the broad baby smile, slender clean drawn nose line instead of two cute dots, gently defined jaw, understated brows. Art direction: hand-drawn Japanese adventure anime, clear flat color shapes and restrained cel shading, no photorealism, no pores, no glossy plastic, no doll or toy features. Preserve the blond hair, purple scarf/shirt, tan trousers, shoes, hand islands, plain skin swatches, all colors and all details outside the face feature area. Keep the exact square atlas layout and framing; no added scene, character render, text, borders or labels. This must fit the existing 3D model without shifting any feature landmarks.
