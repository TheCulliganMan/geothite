# Sabrina mapped anime face review

Built-in imagegen edited the actual UV atlas. Selected file is
`assets/open-models/people/npc-textures/sabrina.png`, SHA256
`22755d34a298aff2823e068f3922af7cea253594cbeb76a74181ad1fd4fd8e09`.
Generated original:
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-e3964961-7254-40a0-ba1d-fc40e79f93cf.png`.

All twelve views in `sabrina-narrow-eye-idle`, `sabrina-narrow-eye-run125`,
`sabrina-narrow-eye-run625` were individually inspected through the production
weighted mesh and `Person::update`. They are beneath
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/`.
Reviewer SHA256 `ecab0ee247affb69673bf8947e6f4ed4c1dce633fc66f667f080d4397c21c5dc`.
The narrowed lids reduce the round doll-like expression. UV layout and outfit
identity remain readable. The square image dimensions are unchanged; read-only
comparison found mean RGB deltas of roughly 1–3/255 outside the face. The edit
is not pixel-identical outside the face, but the inspected mapped clothing and
skin islands retain their layout. This is a selected texture improvement,
not whole-model approval: rigid bob, long jaw/nape, generic body/costume,
hands and silver heel patches remain unfinished. Candidate bytes were copied
unchanged into the canonical asset after mapped inspection.

The strict Rust bundler regenerated four rigs and all 75 required paintings.
JSON/gzip were atomically installed into the ignored local preview. HTTP JSON
bytes exactly matched SHA256
`d4f40eba38afe80ac75a1ff1bedfaa6093f5c91f7d5053d499ee0aba51670778`.
Temporary stage and verification bundles were removed. This texture-only
deployment leaves the prior browser engine in place; torso/UV engine changes
remain pending separate native review and WASM installation.

## Exact generation prompt

Use case: precise-object-edit. Asset type: production UV texture atlas for a weighted 3D anime game character, Sabrina. Image 1 is the edit target, not a portrait reference. Keep the entire square UV atlas composition, exact feature centers and relative island boundaries unchanged. Edit ONLY her eyes, eyebrows, nose, mouth and facial shading in the upper-left head island. Preserve hair silhouette/hairline, ear positions, all skin/neck outside facial features, all maroon/gold clothing, shoes, fingers, right-side skin patches and all lower texture islands unchanged. Her current round shiny eyes make the mapped adult character look like a toy. Replace them with refined hand-drawn adult anime eyes: narrow horizontal almond shapes, upper lids with clean tapered dark lines, small muted plum irises, tiny restrained highlights, visibly reduced eye height. Keep iris centers precisely at the existing centers, keep eye width and spacing. Calm serious psychic trainer expression, subtly angled confident brows, closed understated mouth. Flat warm cel shading using only two restrained facial tones, no painted jaw silhouette or under-chin triangle (the 3D geometry supplies those). Avoid photorealism, pores, glossy skin, realistic rendered facial lighting, huge irises, childlike/chibi proportions, dolls, toy-face shine, smiling baby face, eyelid makeup, lipstick. No text, labels, border, perspective, objects, or rendered model. The result must remain a directly usable complete square UV atlas.
