# Complete painting roster and removal of source-face fallback — 2026-10-11

The remaining eleven paintings are installed at `assets/open-models/people/npc-textures/<identity>.png`. They were generated with the built-in imagegen tool and each was individually inspected as an atlas, four-angle Idle render, and four-angle Run 0.125/0.625 renders before installation. Source-file bytes were copied exactly, canonical destinations were absent, review links now point to canonical assets, and accepted candidate duplicates were removed. Originals remain under the Codex generated-images directory.

The registry requires 74 source identities plus Outdoorsman: 75 paintings, zero missing files. This proves asset completeness, not art approval. None of the full models is approved as perfect. Older paintings with storybook eyes or realistic portrait styling still need redesign. Every model still needs adequate individual hands, costume, motion, bike/context and in-game review.

## Production contract and verification

The native texture loader rejects missing files. GPU construction always requires the supplied painting; the source-face recoloring/bald/borrowed-child-face branches are removed from production rendering. Legacy atlas transformations are compiled only for existing UV/source regression fixtures. Offline review export also requires the production painting. Browser bundle installation validates all required identities before publishing either cache, and the Rust bundler rejects missing paintings.

Fresh `render_people_review` and `bundle_open_people` builds succeeded. The complete browser verification bundle contained four supplied rigs and all 75 paintings (28,881,828 bytes). A fixture missing Will was rejected by both native rendering (exit 101, specific will.png error) and the bundler (exit 1); no fallback screenshot or incomplete output bundle was produced. The roster-identity/cache-install regression passed, and current production instance-isolation/nonaccumulation regression passed in 4.12s. Verification fixture and temporary JSON/gzip bundles were removed after checks.

Fresh strict-production Idle captures for all eleven were pixel-for-pixel identical to the individually hand-reviewed candidate captures. The GoldenrodDeptStore1F production location render completed and its actual image was inspected at `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/department-store-production.png`. It uses the local ignored realtime-clock pack and production image-target camera. It proves this room renders with complete required artwork; it does not prove gameplay or the native window screenshot path. Room/floor improvements belong to concurrent interior work, not this character edit. Overall art quality remains below the requested target.

## Individual findings

| Identity | Remaining visible work |
| --- | --- |
| bruno | Distinct anime face; topknot reads. Torso is still a tan shirt, belt and wrist wraps absent; no muscular bare-body treatment. Jaw/nape and hands remain unresolved. |
| chuck | Anime face with separate beard retained. Still a tan shirt instead of bare torso, belt absent; beard reads as a hard dark band. Jaw/nape and hands remain unresolved. |
| bug_catcher | Anime face and straw hat read; shorts retain exposed legs in both Run poses. Net absent; generic shirt and neck/chin seams need work. |
| gameboy_kid | Anime face and shorts read in motion. Handheld absent; generic shirt, dark chin/neck seam and hands need work. |
| standing_youngster | Anime face and shorts read in motion. Held ball absent; generic shirt, dark chin/neck seam and hands need work. |
| outdoorsman | Anime face, hat, backpack and separate beard read. Vest absent; beard edge is stiff and blocky, shirt generic, jaw/nape and hands unresolved. |
| unused_guy | Anime visitor face reads. Suit and teal tie absent; generic ochre shirt remains. Jaw/nape and hands unresolved. |
| will | Violet anime eyes and purple hair read. Mask, white gloves and fitted suit absent; generic red shirt remains. Hair silhouette, jaw/nape and hands unresolved. |
| swimmer_guy | Anime face reads. Still a tan long-sleeved shirt and blue trousers, shoes retained; no correct swimwear/bare-body treatment or goggles. Jaw/nape and hands unresolved. |
| swimmer_girl | Anime face and ponytail read. Still a blue long-sleeved shirt/trousers and shoes, no proper swimsuit or goggles. Stiff ponytail, jaw/nape and hands unresolved. |
| twin | Anime face and pigtails read. Closed skirt moves in both sampled Run poses, but stiff hem/leg silhouette and absent ribbon remain. Jaw, neck seam and hands unresolved. |

## Exact built-in prompt set

Each final prompt was the common text below followed by its identity specification. Inputs were the exact renderer-exported atlases in `captures/remaining-eleven-source/atlases/`; transparent background was false. No corrective generation was needed in this batch.

Use case: style-transfer. Asset: exact square UV texture atlas for a production animated game mesh. Input is an edit target, not a portrait sheet. Preserve all UV island boundaries, original eye/nose/mouth centers and hair coverage exactly. Redraw only facial and hair surface painting in clean Japanese anime cel style: narrow almond eyes, simple brows, tiny nose mark, small mouth line, two-tone warm skin. No pores, photorealistic lips, stubble, doll gloss, toy aesthetic, round baby eyes, or open mouth. Keep clothing, hands, shoes, trousers and other islands in their exact original locations and colors. All shared cloth patches must remain generic fabric: NO collars, buttons, ties, badges, zippers, pockets or front-specific details. No text, labels, diagrams, extra characters or scenery. 

### bruno

Saved path: `assets/open-models/people/npc-textures/bruno.png`.

Bruno: composed adult martial artist, strong straight brows, dark brown hair, deep brown narrow eyes. No facial hair. Preserve warm tan upper-body patch and ivory pants.

### chuck

Saved path: `assets/open-models/people/npc-textures/chuck.png`.

Chuck: mature sturdy martial artist, dark brown hair, warm expression, subtle age lines and narrow dark eyes. NO painted beard or moustache, since facial hair is separate geometry. Preserve warm tan upper-body patch and ivory pants.

### bug_catcher

Saved path: `assets/open-models/people/npc-textures/bug_catcher.png`.

Bug Catcher: youthful anime adventure face, dark brown hair, alert almond brown eyes and cheerful tiny closed mouth. No facial hair. Preserve blue shirt and khaki shorts islands.

### gameboy_kid

Saved path: `assets/open-models/people/npc-textures/gameboy_kid.png`.

Gameboy Kid: youthful anime adventure face, dark hair, thoughtful small almond brown eyes, tiny closed mouth. No facial hair. Preserve blue shirt and khaki shorts islands.

### standing_youngster

Saved path: `assets/open-models/people/npc-textures/standing_youngster.png`.

Standing Youngster: youthful anime adventure face, dark hair, confident almond brown eyes, subtle closed-mouth smile. No facial hair. Preserve blue shirt and navy shorts islands.

### outdoorsman

Saved path: `assets/open-models/people/npc-textures/outdoorsman.png`.

Outdoorsman: mature adult hiker, brown hair, strong calm brown eyes, warm sun-touched skin. NO painted beard or moustache because the model has facial hair geometry. Preserve green cloth and khaki pants islands.

### unused_guy

Saved path: `assets/open-models/people/npc-textures/unused_guy.png`.

Adult male visitor: brown side-part hair, approachable narrow brown eyes, small understated smile. No facial hair. Preserve ochre shirt and brown pants islands.

### will

Saved path: `assets/open-models/people/npc-textures/will.png`.

Will: elegant adult anime psychic trainer, purple hair, composed narrow violet eyes, small confident mouth line. No facial hair. No painted mask because that is separate geometry. Preserve red shirt and burgundy pants islands.

### swimmer_guy

Saved path: `assets/open-models/people/npc-textures/swimmer_guy.png`.

Adult male sports character: dark brown hair, relaxed narrow dark eyes, clean friendly face. No facial hair. Preserve warm tan upper-body island and blue lower-body island.

### swimmer_girl

Saved path: `assets/open-models/people/npc-textures/swimmer_girl.png`.

Adult female sports character: brown hair, confident restrained almond brown eyes, subtle smile, simple graphic face. Preserve blue clothing islands.

### twin

Saved path: `assets/open-models/people/npc-textures/twin.png`.

Youthful anime adventure girl: brown hair, lively almond brown eyes and tiny closed-mouth smile. Preserve pink shirt/dress islands. Avoid baby round eyes.
