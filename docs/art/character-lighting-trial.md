# Character lighting trial

Four-angle Idle renders of Chris, Kris, Officer and Sabrina compare existing
lit materials with the reviewer's unlit mode. Eight images (32 views) were
individually inspected. Both capture batches completed with exit 0; temporary
reviewer copy was removed. Reviewer SHA256
`ecab0ee247affb69673bf8947e6f4ed4c1dce633fc66f667f080d4397c21c5dc`.
All captures were muted and used production `Person::update`, current head/hand
scales and canonical paintings (including revised Sabrina).

Artifacts:
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/lighting-trial-lit/`
and `lighting-trial-unlit/`, named by role.

Unlit treatment makes drawn eyes and outfit seams more legible and avoids the
dark face/under-jaw shading in these lit review captures. It also removes
useful depth: hands become pale flat silhouettes, curved hats and backpacks
become flat color shapes, and anatomy/clothing contours depend almost entirely
on baked paint. Officer's painted rear-neck triangles are conspicuous even
without lighting, proving that defect also involves the atlas. Kris's sculpted
ponytail shapes lose their volume cues. Both versions still have generic
body/footwear and unresolved model silhouettes.

Pure unlit rendering is rejected as a global solution. No production material
or environment lighting was changed. A subsequent stylized-lighting trial
should preserve readable facial paint while retaining directional depth and
shadows on the character, hats, bags and clothing. This limited Idle comparison
does not approve any full model or lighting treatment for all environments.

## Partial painted-color fill

The reviewer now supports `CRYSTAL_PEOPLE_REVIEW_ALBEDO_FILL` from 0 to 0.5.
It copies the existing albedo texture into an emissive texture with the selected
linear intensity. Untextured hats and props retain their existing lighting.
This is a reviewer-only trial; production materials and lighting are unchanged.

Three fresh batches compare zero fill, 0.12 and 0.25 for Chris, Kris, Officer
and Sabrina using canonical paintings and the same snapshot reviewer SHA256
`c40618711f29680be55894403f3e8ed6c568b6925dbe919e89e3e9421006d437`.
All batches exited zero and all twelve images / 48 views were individually
inspected. Artifacts are `captures/albedo-fill-baseline`, `albedo-fill-012`
and `albedo-fill-025` under the external pending-model-review directory.

The fill improves face readability while retaining directional contour shading
on hats, hair, hands and clothes. The 0.25 trial is brighter but makes sneaker
uppers paler; rear heel gutters, Officer's angular rear-neck paint, Sabrina's
rigid bob and generic body silhouettes remain visible. Neither setting is
approved for global use: this Idle-only comparison does not cover motion,
different environments or all 75 roles. The trials do not solve the unfinished
geometry or character-specific outfit shapes.
