# Kurt anime painting review — 2026-10-11

Both `assets/open-models/people/npc-textures/kurt.png` and `assets/open-models/people/npc-textures/kurt_outside.png` now contain the same selected graphic anime painting. The originals were byte-identical, and both appearances must retain the same character identity. The built-in ImageGen edit tool was used; no CLI/API fallback or raster editing was used. SHA-256 of installed bytes: `63f78b4180360e973f365a1f224f43daee3d5104098f1b0862bbca746ca3f5ad`. Hash guards confirmed neither original had changed concurrently before installation.

The first candidate was rejected after both four-angle Idle images were individually inspected: it still read too much like a portrait. The second candidate has narrower drawn eyes, graphic silver brows/hair, small line facial marks and simpler skin/cloth tones. Both models were individually inspected in four-angle Idle, Run 0.125 and Run 0.625 captures (six images, twenty-four views). Eyes/nose/mouth remain correctly located; blue kimono, exposed skin and brown shoes remain mapped to intended surfaces.

External evidence is under `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/`: `captures/kurt-before-idle/`, `captures/kurt-anime-idle/` (rejected), and `captures/kurt-anime-v2-{idle,run125,run625}/` (installed). These are production weighted meshes/proportions captured via the reviewer's image target, not a live gameplay window. No Rust geometry changed in this painting pass, so no new compilation/test result is claimed.

Neither whole model is approved. Squared lower jaw/nape, warm angular shading patches on scalp/back, generic hands, stiff robe and segmented lit pocket edge remain. Running rear views show the foot within the robe silhouette and require a clearance check across the animation, rather than approval from sampled captures. Outdoor Kurt's basket is absent: `apricornbasket` is declared in the look but has no renderer implementation. The full 75-role/player/context goal remains active.

The missing basket and angular painted scalp findings above are historical: see [subsequent basket implementation and review](kurt-basket-review.md) and [scalp painting correction](kurt-scalp-review.md). Other unresolved findings still apply.

Selected original generated file: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-5c7d5310-e556-493d-b802-47b3f6d4185a.png`.

Fresh canonical-root Idle captures for both installed assets were individually inspected in `captures/kurt-canonical-idle/`; both match the reviewed candidate pixels exactly. The temporary candidate symlink fixture and hash guard scratch file were removed. Focused document whitespace checks passed.

Exact selected edit prompt (the input was the rejected first candidate):

```text
Use case: style-transfer.
Asset type: flat square UV texture atlas for an animated anime game model.
Input image: edit target, Kurt character atlas. Preserve every UV island's normalized location, boundaries, scale and feature placement. Output a flat atlas, never a character render.
Primary request: Repaint the elderly male face as clean Japanese anime game art. Kurt is a stern kindly elderly craftsman, balding with silver side hair and thick silver brows. Use narrow almond brown eyes with clean upper lids, graphic brows, a small angular nose indicated by minimal strokes and a thin closed mouth. Convey age with just a few crisp cheek and forehead lines. No facial hair. Restrained flat warm skin tones and two-tone silver hair clumps. No realistic portrait, pores, flesh gradients, glossy eyes, realistic lips, doll cheeks or storybook face.
Constraints: keep eyes, nose, mouth, ears and side hair at exactly the existing coordinates. Keep the top-right skin patch, right neutral skin islands, brown shoes, blue crossed kimono collar and lower navy garment aligned and recognizable. Simplify cloth to broad cel tones without adding props or markings. Keep warm skin palette coherent across face and hands. No text, watermark, lighting baked into face or extra objects.
Targeted correction from the first render: the face is still too realistic. Make the eyes clearly NARROW drawn anime eyes, approximately half the current opening height, with a strong single upper lid, small flat brown iris and no glossy highlight. Replace sculptural lip/nose shading with tiny angular line marks. Remove the long realistic nasolabial folds; retain only short graphic cheek age lines and two forehead strokes. Use only three broad skin tones without soft airbrush gradients. Keep exact facial feature coordinates and every atlas island unchanged.
```
