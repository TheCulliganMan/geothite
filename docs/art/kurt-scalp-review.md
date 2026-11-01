# Kurt scalp painting correction — 2026-10-11

Both `assets/open-models/people/npc-textures/kurt.png` and `assets/open-models/people/npc-textures/kurt_outside.png` now use the selected correction that removes the broad baked shadow wedges surrounding the face. Those painted planes wrapped onto the bald scalp/back and produced angular patches. Facial anime lines, silver hair/brows, navy kimono, skin islands and brown shoes retain their intended placement. This improves painting continuity; it does not change geometry.

The built-in ImageGen edit tool produced `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-d25198d8-1e95-4d06-b943-b4256f826c44.png`. Bytes were copied unchanged into both canonical files; SHA-256 `d163225c95be47265a134301e4853de2dabee440d44174811f73585f18c18644`. Hash guards verified both replaced paintings still matched the prior installed version. No raster editing or CLI/API fallback was used.

Both appearances were individually inspected in four-angle Idle, Run 0.125 and Run 0.625 captures (six images, twenty-four views), external at `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/kurt-flat-{idle,run125,run625}/`. The side/back painted wedges are absent in these sampled views. Facial placement, garment/skin mapping and outdoor basket remain intact. Fresh canonical-root Idle images were individually inspected in `captures/kurt-flat-canonical-idle/`; both match the reviewed candidate pixels exactly. The temporary candidate symlink fixture was removed.

The images use the production meshes and proportions through the reviewer's image target, not a live gameplay window. Squared lower jaw/nape, generic hands, stiff robe, pocket edge and full-cycle foot/hem clearance remain unresolved. The clearance regression is tracked separately in [Kurt robe audit](kurt-robe-clearance-review.md); no collision-test result is inferred from this painting improvement. Neither whole model is approved; the full player/NPC goal remains active.

Exact edit prompt:

```text
Use case: precise-object-edit.
Asset type: flat square UV texture atlas for Kurt, an animated anime game model.
Input image: edit target, existing Kurt atlas. Preserve all UV island coordinates, silhouettes, boundaries and normalized scale. This is a flat atlas, never a model render.
Primary request: Remove all baked brown shadow wedges, bands and gradients in the large upper-left skin island around the face, above the head, around ears and below the jaw/neck. These marks incorrectly wrap onto the rear of the bald 3D head. Fill all skin background around the facial landmarks with a single continuous flat warm midtone matching the center forehead. The 3D renderer provides lighting. Keep the current narrow drawn brown eyes, stern silver brows, silver side hair clumps, thin mouth, short graphic age lines and tiny angular nose marks exactly in their existing locations. No broad shaded cheek, jaw or neck planes, no realistic lips or portrait gradients. No facial hair.
Constraints: retain every other island including top-right neutral skin patch, right skin/hand islands, brown shoes, navy crossed kimono collar and lower navy garment, exactly in place with its current palette. Keep facial feature size and position unchanged; keep anime line art clear. No new props, text or watermark.
```

