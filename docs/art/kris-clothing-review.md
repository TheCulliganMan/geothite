# Kris clothing painting review

The built-in imagegen tool repainted the blue clothing on the existing UV
atlas. The selected image is installed at
`assets/open-models/people/npc-textures/kris.png`, SHA256
`3c974a2bc9036a1518d4d8906fe2d5da0e0a6b144870de89d944cb21a24440a1`.
The original remains backed up outside the repository. Only Kris's source
painting changed; all other painting hashes were preserved.

The former large triangular folds made the jacket and trousers appear
faceted. The new cloth uses calmer mid-blue tones and restrained curved
creases, retaining the collar, zipper, cuffs, hem and belt layout.

The generator did not preserve protected pixels exactly. Both atlases are
1254 square. Mean protected-region RGBA delta is 1.222, and mean face delta
is 1.975; edge differences are larger. This is not a pixel-preservation claim.
The actual mapped face, hair identity and shoe island alignment were checked.
No generated face realism or texture fallback was introduced.

Six actual images were individually inspected, fifteen views total: larger
front, side and rear; both Run .125/.625 poses at four angles; and mounted
.25 at four angles. Sessions 31353 and 42544 exited zero. The outfit remains
clean through those sampled poses. The actual GPU mesh comparison preserves
positions, normals, UVs, joint IDs, weights and indices exactly, independently
of the small generated pixel differences.

Canonical-root Run capture 92798 exited zero and was individually inspected,
four additional views. Its decoded pixels match the isolated candidate Run
image exactly. The total clothing review is nineteen mapped views.
Browser deployment and full in-game review remain outstanding. Outfit
silhouette, nape, hands and shoes still require work; no whole model is
approved. All previews remain muted.

The Rust browser bundle builder completed successfully in session 4856. It
contains four rigs and seventy-five paintings; compared to the currently
served bundle only Kris changed. All rig bytes and other painting bytes are
identical, and gzip decompresses exactly to the new JSON. The engine build
11790 remains active; no browser deployment is yet claimed.

## Final prompt

Built-in imagegen tool, with the existing Kris atlas as the edit target:

> Use case: precise-object-edit. Edit target: supplied square game character UV texture atlas. Repaint ONLY the blue jacket and blue trouser clothing islands for a polished hand-drawn anime adventure game. Preserve EXACT atlas island boundaries, placement, scale, zipper, ivory collar, cuff/hem edges, belt placement, and blue palette. Replace the large jagged triangular high-contrast polygons on the jacket and trousers with restrained broad clean cel-shaded cloth tones and a few small curved crease accents at natural seams; calm mid-blue fabric, subtle darker navy underarms and inner-leg folds. These are albedo textures mapped onto an animated 3D model: no baked dramatic lighting, no photographic texture, no shiny plastic, no noisy triangles, no exaggerated angular folds. Keep ALL hair, face, eyes, eyebrows, mouth, ears, skin, finger marks and shoe islands unchanged; do not redraw face or hair. Do not rearrange, crop, add labels, logos, accessories or invent new islands. Output the complete same square opaque UV atlas, same proportions as input. Clean anime line work, flat color shapes, gentle clothing shading. All pixels outside the blue clothing areas should be preserved.

The pixel-preservation part of the prompt was not fully satisfied; retention
was based on actual mapped visual review and unchanged mesh/UV geometry.

The verified player browser engine is now served locally with hash
`28c8d5297be2d46df6af84626e6b1fe07544fcc8465ea75e2d1dc0db9c89853c`.
Index, glue, engine and JSON/gzip bundle bytes all match the staged files over
HTTP. The three cache references use its new prefix, and startup stays muted.
The browser open request was queued; actual fresh browser frame inspection
remains unverified. Completed staging/backup scratch was removed. Subsequent
shoe candidates are separate and are not in this served build.
