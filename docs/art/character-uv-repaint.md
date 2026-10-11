# Character UV atlas repaint — 2026-10-10

Current art direction supersedes the earlier face paintings below: see [anime prototype and intermediate conversion record](stylized-character-faces.md). The user rejected both realistic faces and the subsequent rounded storybook direction; the roster is not approved or complete.

The user clarified that “maps” meant the UV texture maps wrapped onto characters. The terrain concept revisions from the previous turn were a misunderstanding of that request.

Built-in imagegen edited the existing square Chris and Super Nerd atlases. Selected outputs replaced `assets/open-models/people/npc-textures/chris.png` and `super_nerd.png`, which are the existing external model-root texture paths loaded by the production GPU skin renderer. Source GLB geometry, UV coordinates, skinning, clothing meshes and accessories were unchanged. These are production albedo paintings, not terrain art references.

Each source atlas and generated output was inspected. Both atlases retain their square framing and region layout; imagegen interprets small painted feature shapes, so exact pixel preservation is not claimed. The actual production GPU skins were reviewed from four angles before and after at idle, and after at Run phase 0.25, using a candidate model root before installation. Chris has clearer eyes, smoother skin and organized hair shading. Super Nerd has clearer eyes behind his glasses and less muddy skin/fabric. Their existing red/blue clothing identities remain. The source jaw and neck geometry remain visibly angular and the cast is not finished.

Validation after installation: all 16 supplied appearance checks pass with the
local model root, and both texture-loader checks pass. SHA-256 comparison confirms
the installed textures are byte-identical to the candidate textures reviewed on
the models. The texture-loader tests validate decoding and normalization behavior;
the actual painted UV alignment was checked visually, not proven by those tests.
Owned review processes finished before temporary captures and candidate-root
symlinks were removed. No packs, generated PCM or compiled artifacts were staged.

## Chris prompt

Use case: style-transfer
Asset type: production UV albedo texture atlas for a small stylized 3D male player character, Chris.
The input image is the actual UV texture atlas to EDIT. Output a square texture atlas, not a portrait, model render or concept sheet.
Preserve the input's exact normalized UV layout: every island rectangle, every color boundary, hairline, eyes, nose, mouth, ears, collar, zipper, sleeve trim, shoes, hands and trouser seam stays in the identical normalized pixel position. Preserve orientation and square framing with no margin or cropping. Keep identical shapes and size of all features so the existing rig samples correctly. Face remains centered exactly where it is in the source.
Improve paint quality and consistency: clean crisp readable brown eyes with subtle white catchlights, clear eyebrows and understated natural mouth; warm even peach skin with very subtle soft gradients, no muddy streaks on neck or cheeks. Brown hair with organized broad smooth locks, restrained strand accents instead of coarse noisy mottling. Bright red jacket, cream collar and side stripes, dark undershirt, dark-blue trousers, white sneakers, all in their source locations. Clothing has clean seams, understated fabric shading and a few intentional folds, no gritty noise or excessive baked shadows. Make the artwork cohesive polished hand-painted game albedo, soft dimensional color, clear shapes readable at 512px texture resolution.
Most important: keep original UV coordinates and island geometry exactly unchanged. Do not enlarge or move the face or add features elsewhere. No text, guide lines, grid, borders, props or background scene. The entire output is only the revised square atlas.

## Super Nerd prompt

Use case: style-transfer
Asset type: production square UV albedo texture atlas for the Super Nerd NPC.
Input 1 is the actual atlas EDIT TARGET; input 2 is a revised Chris atlas used ONLY for coherent painting style.
Repaint input 1 to the same clean polished hand-painted game finish as input 2. Keep the EXACT normalized layout and all island boundaries, hairline, face, eyes, ears, mouth, collar, buttons, brown suspenders and buckles, sleeve regions, shoes, hands and trousers at their original coordinates and dimensions. Do not borrow Chris's clothing, face or hairline.
Preserve this NPC's gentle friendly expression, brown hair, warm peach skin, blue collared shirt with cream collar/buttons, brown suspenders, dark navy trousers and cream sneakers. Give clear readable individual eye shapes and small catchlights, natural eyebrows, a restrained smile. Smooth even skin gradients instead of grainy muddy cheeks and neck; hair painted as broad organized locks with restrained strand accents. Fabrics have clean seams and a few intentional folds with subtle surface detail, no coarse noise, dirty shadows or excessive wrinkles.
Important: output ONLY a square flat UV atlas, not a character illustration or model render. The source atlas's face stays centered in the exact same place and size; do not stretch, rotate or move any island. Same square framing, no margins or cropping. No text, labels, grid or new accessories.


## Lass, Nurse and Cooltrainer F repaint — 2026-10-10

Built-in imagegen revised these three existing production atlases using Chris only as a painting-style reference. The retained files are `assets/open-models/people/npc-textures/lass.png`, `nurse.png` and `cooltrainer_f.png`. The original atlas and generated image were inspected for each. Production GPU model captures were compared before/after at idle from four angles, and the candidates were examined again at Run phase 0.25 before installation. The model-review executable was freshly built from the current worktree.

Lass keeps her red school outfit and friendly expression. Nurse keeps pink hair and white uniform, with rounded clear eyes replacing coarse eye pixels. Cooltrainer F keeps her blue jacket and confident expression, with clearer hair locks and face painting. Skin and clothing are less grainy. No obvious new eye/ear/hairline misplacement appears in those samples, but exact UV pixel invariance is not claimed for imagegen paintings. Geometry, skinning and UV coordinates were not changed. Skirts remain plain and several painted collar/trim details are not well exposed by existing UV corrections; angular jaws/necks also remain unfinished. These art views are not gameplay proof. The full cast goal stays active.

After installation, all 16 supplied appearance regressions pass. SHA-256 pairs
confirm all three installed PNGs are byte-identical to the candidate atlas files
used for the reviewed renders. Owned build and capture processes finished before
removing temporary screenshots, logs and the candidate-root symlinks. No generated
content packs or compiled outputs were staged.

### Lass prompt

Use case: style-transfer
Asset type: production square UV albedo texture atlas for the Lass NPC.
Input 1 is the atlas EDIT TARGET. Input 2 is the revised Chris atlas: use only its clean cohesive painting quality.
Repaint input 1 with clearer eyes, smooth warm skin, organized brown hair locks and clean red fabric. Preserve the source's EXACT normalized UV layout, all region rectangles, every island boundary and hairline, face center, eye/ear/mouth positions and size, cream rounded collar, buttons, sleeves, skirt hem, shoes and hand shading. Do not copy Chris's clothing or male face.
Keep this young female NPC's friendly gentle smile, brown eyes, brown bangs, red outfit with cream collar/buttons and white sneakers with red trim. Replace muddy neck shadows, coarse grain and dense fabric wrinkles with subtle soft gradients and a few intentional folds. Clearly painted eyes and lashes with small catchlights, soft eyebrows and restrained cheek warmth. Hair should have broad clean locks and restrained strand accents. Crisp albedo readable at 512px, no heavy baked light.
The complete output must be ONLY a flat square texture atlas, not a character portrait or 3D render. All feature placements and UV boundaries must remain unchanged. No cropping, margins, grid, text, new accessories or background scene.

### Nurse prompt

Use case: style-transfer
Asset type: production square UV albedo texture atlas for the Nurse NPC.
Input 1 is the actual atlas EDIT TARGET. Input 2 is revised Chris, reference ONLY its clean cohesive hand-painted finish.
Repaint input 1 preserving EXACT normalized UV island rectangles, all color boundaries and face proportions/coordinates. Keep pink hair and hairline, pink eyebrows, rose-brown eyes, mouth and ears in their identical positions. Keep the white medical uniform with pink collar and sleeve trim, three buttons, apron belt and pockets, white lower garment region, plain white shoes and skin/hands in their original atlas regions. No new accessories or borrowing Chris's outfit.
Improve clarity: softly painted natural eyes with small highlights instead of blocky square eye pixels; warm even peach skin, gentle smile, subtle cheek warmth and no muddy streaked neck shadows. Pink hair in broad elegant organized locks, subtle strand accents, not coarse noisy strokes. White fabric needs a few soft intentional folds, clean seams and gentle neutral shading rather than heavy gray wrinkling. Preserve the pink tone and distinct caring female identity. Polished stylized game albedo, readable at 512px.
Most important: only repaint the existing layout; do not move, resize or rotate any island or face feature. Same square framing with no crop/margins. Output ONLY the flat UV atlas, not a character render, portrait or sheet. No text, grids, guide lines or background scene.

### Cooltrainer F prompt

Use case: style-transfer
Asset type: production square UV albedo texture atlas for Cooltrainer F NPC.
Input 1 is the actual atlas EDIT TARGET. Input 2 is revised Chris used ONLY for consistent polished hand-painted albedo style.
Repaint input 1 without changing its normalized UV layout: keep every island rectangle, skin/hair boundary, face shape, eye/brow/nose/mouth and ear placement, blue jacket, cream undershirt and zipper, navy collar and side panels, sleeve skin, navy skirt/hem, hands and navy-trimmed white shoes in exactly their existing coordinates, orientation and dimensions.
Keep the young adult female's confident gentle expression, brown eyes, brown parted hair and blue athletic jacket with cream stitching, navy skirt. Improve skin into warm even peach with subtle gradients and no muddy neck/cheek streaks. Clear readable eyes with restrained lashes and tiny highlights, graceful brows and understated smile. Hair as broad organized locks with subtle strand accents. Cleaner fabric with a few intentional folds and crisp seam detailing, no gritty noisy paint, heavy dark baked shadows or dense wrinkles.
Only polish this character's existing texture. Do not copy Chris's face or clothing. Most important: all UV regions and facial feature positions stay locked, with square framing, no crop or margins. Output only the flat texture atlas; no 3D character render, portrait, grid, text, border, scene or new accessories.


## Youngster, Gramps and Granny repaint — 2026-10-10

Built-in imagegen repainted the remaining three existing UV paintings, preserving their age and clothing identities. Youngster's cheerful child face remains distinct from the adult player; Gramps retains the bald top, gray side hair/moustache and age lines; Granny retains gray hair, gentle age definition and cream shawl over brown clothing. Smooth skin and ordered hair/fabric strokes replace coarse noisy paint. Exact original UV pixel preservation is not claimed, but no rig positions, UVs, bones or skinning changed.

Production-model before/after idle views were hand reviewed from four angles for Youngster, Gramps, Granny, Bugsy, Bug Catcher, Gameboy Kid and Standing Youngster. The latter four borrow only the central Youngster face through the existing production skin painter, retaining their own hair, ears, clothing and skin-tone matching. All seven candidates were also hand reviewed at Run phase 0.25. Eye highlights and smooth skin read better; no obvious new blended-face seam appears in those samples. The viewer executable was freshly built. These views are art checks, not gameplay proof. Angular head/neck geometry, repeated child expressions and incompletely exposed clothing detail remain; the full cast goal is active.

The selected outputs replaced `assets/open-models/people/npc-textures/youngster.png`,
`gramps.png` and `granny.png`. All 16 supplied appearance regressions pass against
the candidate root, including child-face contrast, unchanged hair/ears/clothing
outside the face blend, and alpha preservation. SHA-256 pairs confirm the installed
files are byte-identical to those reviewed and tested candidates. An earlier test
build failed on a sampler enum equality assertion in concurrently edited stone
tests; that assertion was corrected in the shared worktree before the passing run.
All owned builds and render processes finished before temporary captures, logs and
candidate-root symlinks were removed. Nothing was staged. The eight existing atlas
files have now all received repaint reviews; most other cast members still use
fallback textures, so this does not finish the full character goal.

### Youngster prompt

Use case: style-transfer
Asset type: production square UV albedo atlas for the Youngster child NPC.
Input 1 is the actual UV atlas EDIT TARGET. Input 2 is revised Chris: use ONLY its clean organized hand-painted quality, not its adult face or outfit.
Repaint input 1 preserving its EXACT normalized island layout, rectangle boundaries, dark-brown hairline, child face shape and eyes/brows/nose/open smiling mouth/ears in precisely their original positions and size. Keep the big friendly oval brown eyes and cheerful open smile, clearly childlike expression; add restrained tiny eye highlights without enlarging or relocating the eyes. Smooth warm peach skin, soft natural gradients, no grainy rough or muddy streaked neck. Dark-brown hair as organized broad locks with subtle strand accents, same source hairline.
Keep vivid blue tee with cream neck/sleeve bands, navy shorts and belt, bare leg and hand regions, white sneakers, all in their original UV coordinates. Cleaner fabric with just a few intentional folds and crisp seams, no gritty noise or excessive baked shadows.
Only repaint existing color and details. Do not copy adult Chris's features or clothes. Highest priority exact region and feature placement so existing model samples remain correct. Square framing, no crop, margin, rotation, grid, guide lines, text, new accessories or background. Entire output is ONLY the flat square UV atlas, not a portrait or 3D character render.

### Gramps prompt

Use case: style-transfer
Asset type: production square UV albedo texture atlas for elderly male NPC Gramps.
Input 1 is the actual atlas EDIT TARGET. Input 2 is revised Chris, reference ONLY its clean organized hand-painted finish.
Preserve the exact normalized UV layout: all rectangles and islands, bald scalp and gray side-hair boundaries, eyebrows, eyes, nose, gray moustache, smile and ears, cardigan collar/buttons/pockets, brown shirt, trousers/belt, shoes and hands remain at their identical existing coordinates and size.
Retain an unmistakably elderly kind male face, bald top, soft gray side hair and moustache, gentle wrinkles at the original corners of eyes and cheeks. Improve small eyes into clean readable shapes with tiny restrained highlights without enlarging or relocating them. Even warm peach skin with soft natural shading rather than muddy streaks, subtly varied scalp tone and age lines. Gray hair/moustache should use broad ordered locks and restrained strands rather than dense noisy brushstrokes.
Keep cream knitted cardigan over brown shirt, dark brown trousers and cream shoes. Clean crisp seams/buttons with understated knit detail and a few folds. No dirty shadow bands, excessive wrinkles or noisy grain. Preserve elder identity rather than making him young or copying Chris.
Entire output ONLY a flat square texture atlas. Highest priority locked UV and feature coordinates. No cropping, margins, rotation, text, grid, guide lines, new clothing/accessories or 3D render/background scene.

### Granny prompt

Use case: style-transfer
Asset type: production square UV albedo texture atlas for elderly female NPC Granny.
Input 1 actual atlas EDIT TARGET; input 2 revised Chris is ONLY a clean organized painting-style reference.
Repaint input 1 while keeping every normalized UV rectangle/island/color boundary, gray hairline, eyes/brows/nose/smile/ears, cream shawl knot and edges, brown dress, trousers/leg regions, hands and cream shoes at exactly their original positions, orientation and size. No copying Chris's adult male identity.
Keep a warm elderly woman with soft gray hair, gentle brown eyes, subtle original age lines and kind restrained smile, rosy cheeks. Do not make her a young girl. Clear readable eyes with small highlights, subtle eyelid/cheek age definition, clean gray brows. Even warm peach skin with soft gradients and no coarse grain or muddy neck streaks. Gray hair in broad ordered locks and restrained strands, preserving parted hairline. Cream tied shawl over brown dress, tidy seam and knot details, a few deliberate fabric folds, no dense wrinkle noise or heavy baked shadows.
Output ONLY the flat square UV atlas. Highest priority exact UV and feature coordinates. Same framing, no crop/margins, rotation, guide lines, grid, text, new accessories, portrait or 3D scene.

## Native clothing UV placement repair

The production rig's front shirt starts near UV v=.747, while the paintings
place collars near v=.475. This left collars and upper clothing detail out of
the front render. `painted_torso` now maps the actual front shirt triangles
across the painted clothing region for Chris, Lass, Nurse, Cooltrainer F,
Super Nerd, Gramps, Granny and Youngster. The GPU calls this only when the
external painting loaded successfully. Narrow female and wider male clothing
islands have separate horizontal ranges, bounded to avoid neighboring skin.

Youngster's rear shirt previously sampled a neckline at the lower back;
Granny's rear sampled the front shawl knot behind her neck. Both now sample
plain matching fabric. Existing corrections for the other six rear shirts
remain in place. Every changed triangle keeps its exact positions, normals,
colors, winding, joints and weights; only duplicated clothing corner UVs
change. The original vertex buffers remain intact.

Reviewed all eight before and after in four-angle production GPU captures,
then reviewed the final mappings in Idle and Run at phase .25. Collars,
buttons, suspenders, jacket openings, cardigan pockets and the shawl knot
now appear on the fronts. Faces, arms and footwear were visually preserved.
Granny's rear shawl and Youngster's rear shirt no longer repeat front details.

Validation: the new supplied-rig regression passed across all eight roles and
Officer as an unaffected control. It verifies exact triangle geometry and
skin attributes, preserved source buffers, bounded UVs and collar coverage.
The final production reviewer built successfully. A subsequent broader
`npc_appearance::tests -- --include-ignored` run could not compile because
`mesh/ledge_approach.rs:54` references an unresolved `ledge_finish` module in
the concurrently changing terrain code; this run is not reported as passing.
Temporary render captures and verification logs were removed after review.

This repairs clothing placement on the eight painted atlases. Remaining cast
textures, costume details, material seams and model silhouettes still need
individual review; it does not complete the full character-quality goal.


## Kris, Red and Mom: first individual paintings

Used the built-in imagegen tool in edit mode from Rust-exported fallback atlases.
Installed new project assets in `assets/open-models/people/npc-textures/`:

- `kris.png`: teal hair, youthful feminine face, navy jacket and trousers.
- `red.png`: youthful male face, readable red vest front over white shirt, blue jeans.
- `mom.png`: mature feminine face, brown hair, red blouse with cream collar/buttons.

Selected generation files: `exec-69bc6e1e-480d-4bb3-bee6-623fc1c9cd6d.png`,
`exec-b820a011-4615-43cb-b9f5-11a462ce750f.png`, and
`exec-6e1c7ed5-63be-4a47-9d27-0fdf83f03e48.png`, all in the thread's
`.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef` directory.
The first Mom candidate (`exec-193acae8-dde1-4796-9a02-c2ef630f78c7.png`)
was rejected after GPU review showed a pale scalp wedge: its head island ended
too early. The targeted second edit extends the hair island and removes that
wedge in the actual final model. Installed hashes match selected generations.

Extended the painting-only native front/rear shirt mapping to these three roles.
The optional offline reviewer flag `CRYSTAL_PEOPLE_REVIEW_EXPORT_ATLASES=1`
writes native fallback/loaded atlases into its ignored output directory, avoiding
an independent reconstruction of the renderer's texture palette. These three
exports match the texture path used by their models; exports are scratch inputs.

Reviewed all three before/after, four angles each, then final Idle and Run .25.
Facial features and hair paintings are substantially clearer than the blurry
fallback. Front collars and vest opening are readable; rear collars no longer
appear at the waist. Final Mom no longer has the pale crown wedge. Build passed,
expanded supplied-rig UV regression passed (eleven painted roles and Officer
control), and all sixteen appearance tests passed with the supplied rigs.
Temporary atlases, captures and logs were removed after inspection.

Remaining issues from this review: Mom's bob accessory still has an angular,
plain surface and differs in finish from the painted scalp. Red's arm UVs still
show long red sleeves rather than the intended white short sleeves, and native
shoe tinting obscures much of the new white/red footwear painting. These need
geometry/material work; these three models are improved, not declared finished.
There are now eleven individual painted atlases; the rest of the cast still
needs individual painting and model review. The full goal remains unfinished.



## Jasmine and Karen: individual UV paintings

Used the built-in imagegen tool in edit mode with the Rust reviewer's exported
fallback atlas as each edit target and Mom as a finish-only reference. Selected
outputs were copied without pixel edits into the project:

- `assets/open-models/people/npc-textures/jasmine.png`: chestnut hair, youthful
  feminine face and cream dress bodice.
- `assets/open-models/people/npc-textures/karen.png`: silver hair, narrower adult
  eyes and a dark navy wrap neckline.

Generation sources in this thread's `.codex/generated_images` directory:
`exec-a9d3aa91-a0ca-42e6-ba03-5ce8ded28660.png` and
`exec-d461674d-a3ec-40be-a601-f70de9b24abd.png`.

### Jasmine prompt

Use case: style-transfer. Production flat square UV albedo atlas for Jasmine, a young adult woman with long chestnut-brown hair and a cream dress. Input 1 actual renderer fallback atlas EDIT TARGET. Input 2 Mom is ONLY a reference for clear hand-painted finish, not face identity or red outfit. Preserve input1 exact normalized rectangles, eye/brow/nose/mouth/ear coordinates, size, orientation and hairline. Head/hair island fills x0.. .625,y0.. .475; extend brown hair all the way to x=.625 in the upper band, never leave a shoe-colored scalp wedge. Repaint blurry face as delicate youthful feminine face with distinct soft brown eyes, restrained catchlights, fine brows, gentle small smile and smooth warm peach skin, no stubble, dirty chin or muddy vertical neck streaks. Brown hair in ordered broad locks, subtle strands. Lower-left x0.. .625,y.475..1 cream dress bodice with a modest rounded neckline at TOP CENTER u=.3125,v=.48, subtle small stitched seam and soft folds; keep plain cream at u=.16,v=.65.. .85 for rear sampling. Existing lower-right cream clothing region remains cream, preserve shoe islands x=.625.. .8125,y=.125.. .5 and all arm/hand skin coordinates. Shoes white, hands same skin. No new accessories. Output ONLY square flat texture atlas, no model/scene/text/grid/border/crop/rotation. Exact UV boundary and feature alignment highest priority. Clean broad shading, no dense grain or heavy baked directional shadows.

### Karen prompt

Use case: style-transfer. Production flat square UV albedo atlas for Karen, a confident adult woman with silver-gray long hair and dark navy dress. Input 1 actual renderer fallback UV atlas EDIT TARGET. Input 2 Mom reference ONLY for clean painted surface finish, not identity or clothing. Preserve input1 exact normalized atlas rectangles, eye/brow/nose/lip/ear coordinates, size/orientation and hairline. Upper-left head island fills x0.. .625,y0.. .475; silver hair extends to x=.625 across upper band with no pale shoe wedge. Repaint a distinct mature confident feminine face, almond-shaped dark brown eyes with clear small highlights, elegant slightly arched gray brows, subtle eyelids, narrow composed smile. No childlike round eyes or copied Mom face. Warm peach skin with smooth soft gradients, no beard/stubble, muddy chin or vertical neck streaks. Silver-gray hair with broad organized flowing locks, restrained strands, no brown hair. Lower-left x0.. .625,y.475..1 dark navy dress bodice, clean shallow V neckline centered u=.3125,v=.48, minimal seam detail and a few soft fabric folds, no buttons or cream collar. Keep plain navy at u=.16,v=.65.. .85 for rear torso sampling. Lower-right clothing rectangle stays navy. Preserve all arm/hand skin islands, white shoes in x=.625.. .8125,y=.125.. .5 at original coordinates. No new jewelry or accessories. Output ONLY square flat UV atlas, exact boundary and facial alignment highest priority; no scene/model/portrait/text/grid/border/crop/rotation. Broad subtle albedo shading, no dense noise or heavy directional shadows.

### Neckband repair prompts

Use case: precise-object-edit. Edit this Jasmine UV atlas only to remove the blue-gray rectangular strip at the bottom of the head/neck island, approximately normalized x=.118.. .480,y=.454.. .478. Replace that strip with the adjacent warm peach neck skin, smooth and color-matched, continuing through y=.478. The cream clothing starts at y=.479 and stays unchanged. Preserve every other pixel region, face features, eyes/ears/hair coordinates, brown hair, cream collar/buttons, all UV rectangle boundaries, white shoes, arm/hand skin and square framing. No added details, no redesign, crop, text/grid or scene. Output only flat square UV atlas. This repairs an unwanted source blue band visible around the model neck.

Use case: precise-object-edit. Edit this Karen UV atlas only to remove the blue-gray rectangular strip at the bottom of the head/neck island, approximately normalized x=.118.. .480,y=.454.. .478. Replace that strip with adjacent warm peach neck skin, smooth and color-matched, continuing through y=.478. Navy clothing starts y=.479 and stays unchanged. Preserve face identity, narrow eyes, ears, silver hair, navy wrap V-neckline, all UV rectangle boundaries and feature coordinates, shoes, arm/hand skin, framing. No redesign, crop, extra details, text/grid/model/scene. Output only flat square UV atlas. Repair the unwanted blue neckband without moving the islands.

The initial candidates retained a source blue-gray neck strip. Targeted built-in
edits replaced it with matching neck skin. Final selected files are
`exec-f2ece5b2-9574-4cf3-b67c-d1ab1164e683.png` (Jasmine) and
`exec-6fef189d-4c32-4b62-b5c1-aae29a6114a2.png` (Karen), installed at the same
project paths above. The initial two outputs remain generation history only.
Front/rear clothing UV placement now includes these two roles. Light shoe
uppers preserve their actual painted white shoes instead of the generic dress
tint. The separate long-hair curtain samples each painting's hair color.

Compared both characters before/after from four production GPU angles, then
checked the repaired atlases in Idle and Run phase .25. The faces are clearer,
front necklines are readable, the blue neckbands are gone, and white shoes now
retain their painted detail. Rear clothing avoids front neckline repetition.
Installed asset hashes match the selected generation files. Both UV regressions
pass across thirteen painted roles plus Officer, all sixteen appearance tests
pass, and the supplied footwear geometry/attribute regression passes. Final
reviewer build and captures succeeded. Temporary outputs were removed only
after all owned processes finished.

These paintings do not finish the models: the long hair still has a faceted
crown and plain, repeated curtain masses; the dress skirt remains a plain
separate surface. Thirteen individual atlases are now installed out of the
75-role cast. The full quality goal remains active.



## Beauty and Daisy: individual UV paintings

Used built-in imagegen in edit mode, with the Rust-exported fallback atlas as
each edit target and Karen as a painting-finish reference. Installed selected
outputs without bitmap postprocessing:

- `assets/open-models/people/npc-textures/beauty.png`, sourced from
  `exec-1093cca6-04df-47a9-bf1c-b3467271b7c6.png`.
- `assets/open-models/people/npc-textures/daisy.png`, sourced from
  `exec-3e5c7cdc-bc18-4942-b22a-6360e541aa38.png`.

Sources remain in this thread's `.codex/generated_images` directory. Beauty
retains chestnut hair with a blue wrap bodice; Daisy has honey-golden hair,
hazel eyes and a rounded blue neckline. Both remove the source blue neckband.
Extended painting-only front/rear torso placement and light shoe tint to the
two new paintings; hair curtains sample their actual hair painting colors.

Reviewed before/after from four production GPU angles and checked both final
paintings in Idle and Run phase .25. Face features are clearer, the neckbands
are gone, different front necklines are visible, rear fabric avoids front
neckline repetition, and painted white shoes remain readable. Installed hashes
match the selected generated images. Both supplied UV regressions pass across
fifteen painted roles plus Officer; all sixteen appearance tests pass. Final
reviewer build and captures succeeded. All owned processes finished before
removing temporary captures, exported fallback inputs and logs.

Remaining model issues are visible: long curtains are still plain compared
with the painted crown, and the separate skirt lacks the bodice's fabric finish.
Beauty and Daisy still share too much of the same facial structure despite
their palette/eye/neckline differences. Fifteen of 75 roles now have individual
paintings; this is progress, not completion of the full cast quality goal.

### Beauty prompt

Use case: style-transfer. Production square flat UV albedo atlas for Beauty, confident adult woman with long rich chestnut hair and medium blue dress. Input1 renderer fallback EDIT TARGET; input2 Karen style reference ONLY for clear painted finish, not identity or gray hair. Preserve exact normalized rectangle boundaries and face feature coordinates/scale/orientation from input1: upper-left head x0.. .625,y0.. .479, ears/eyes/brows/nose/lips in same places. Brown hair fills top band right through x=.625, no shoe wedge. Paint an individual elegant adult face with almond-shaped hazel-brown eyes, slightly arched fine brown brows, fuller rosy lips with small composed smile and soft cheek definition. No copied Karen face or childlike oversized eyes. Smooth warm peach skin, no muddy neck stripes/stubble. Entire head island below face INCLUDING the blue-gray source strip at x=.118.. .48,y=.454.. .479 must be matching warm neck skin; remove that blue band completely. Clothing x0.. .625,y.479..1 medium blue dress bodice with modest shallow curved neckline centered u=.3125,v=.48, subtle curved seams, a few soft folds; no cream collar/buttons or jewelry. Leave plain blue at u=.16,v=.65.. .85 for rear sampling. Lower-right garment region same medium blue. Original shoes rectangle x=.625.. .8125,y=.125.. .5 white, original arm/hand skin islands unchanged in position. Output ONLY square flat atlas, no model/scene/portrait/text/grid/border/crop/new islands. Exact UV alignment highest priority; organized broad hair locks, restrained strands and albedo shading, no dense grain.

### Daisy prompt

Use case: style-transfer. Flat square production UV albedo texture for Daisy, warm young adult woman with long honey-golden hair and medium blue dress. Input1 actual fallback atlas EDIT TARGET, input2 Karen ONLY painted-finish reference, not identity or gray hair. Preserve input1 exact normalized islands and original eye/brow/nose/mouth/ear feature positions, sizes and orientation. Head upper-left x0.. .625,y0.. .479; honey blonde hair covers upper band through x=.625. Distinct gentle youthful face with rounded soft green-hazel eyes, delicate golden-brown brows, light peach cheeks and a narrow friendly smile, not Karen's mature arched-brow expression or a child. Smooth warm peach skin, readable catchlights, subtle eyelids; no stubble, grain or muddy neck streaks. Remove the original blue-gray strip x=.118.. .48,y=.454.. .479; entire lower head island stays warm skin through y=.479. Hair broad organized golden locks with restrained strands. Lower-left clothing island x0.. .625,y.479..1 medium blue dress bodice, small rounded neckline centered u=.3125,v=.48, two restrained vertical shaping seams and soft folds. No buttons, collar, V-neck or wrap seam; no painted jewelry. Plain matching blue at u=.16,v=.65.. .85 for rear sample, lower-right garment also blue. White shoes remain x=.625.. .8125,y=.125.. .5; all arm/hand islands retain exact coordinates and warm skin. Output only flat square atlas, no portrait/model/scene/text/grid/border/crop/new islands. Exact UV and feature alignment highest priority.



## Erika: individual UV painting

Used built-in imagegen edit mode with the Rust-exported fallback atlas as the
sole edit target. Installed the exact selected output at
`assets/open-models/people/npc-textures/erika.png`, generated as
`exec-ca676b39-f909-46a3-ac91-e5caeb10f918.png` in this thread's
`.codex/generated_images` directory. The painting has a calm young adult face,
dark bob hair, overlapping ivory kimono collar, muted rose fabric and pale
leaf details. Native painting-only torso mapping places the collar on her
front, with a plain rose patch for the back. Her separate kimono skirt samples
the painted rose fabric color; neutral upper shoe tint retains the charcoal
painting. No geometry or rig replacement in this texture pass.

Hand-reviewed the final production model from four angles in Idle and Run
phase .25, comparing the original fallback capture. Her facial features are
clearer, the ivory collar stays on the front, leaf details read on the sleeves,
and the back remains plain rose fabric. The first model capture showed the
old skirt palette mismatch and nearly black tinted shoes; final native color
sampling/tint changes improve both. Installed PNG SHA-256 matches the selected
generation. All three supplied-rig UV tests and 17 appearance tests passed,
including geometry/skin/animation-weight preservation across the painted cast
and the articulated costume check. Reviewer build and diff check passed;
owned scratch captures/logs removed after processes finished.

Remaining: crown-to-curtain joins are visible, the skirt is still plain rather
than painted, and sleeve shapes remain generic narrow sleeves. Her flower and
obi were subsequently added as fitted native geometry; see `character-review.md`,
Erika's fitted obi and hair flower.
Footwear still resembles
closed laced shoes. This is an individual texture improvement, not a completed
Erika model or a completed cast.

### Erika prompt

Use case: style-transfer. Edit target: supplied actual renderer UV atlas for Erika. Produce a clean flat square hand-painted stylized game albedo texture, with EXACT original rectangular islands and facial feature positions, sizes and orientation. Head upper-left x0.. .625,y0.. .479; hair fully covers upper band through x=.625; eyes/ears/brows/nose/mouth stay at original coordinates. Erika is a composed young adult woman with dark brown short bob hair, warm brown almond eyes, fine gently curved brows, small serene closed smile, smooth warm ivory-peach skin. Distinct calm face, not generic large-eyed smiling doll, not photorealistic; broad organized dark hair locks with restrained fine strands. Remove blue-gray neck strip x=.118.. .48,y=.454.. .479, replace with clean warm neck skin extending to y=.479. Lower-left garment rectangle MUST FILL x0.. .625,y.479..1: muted rose kimono torso, a clean overlapping ivory folded collar beginning at TOP CENTER u=.3125,v=.48, narrow ivory bands crossing diagonally toward mid-chest v=.62, modest neckline, rose fabric surrounding it. Restrained embroidery of small pale leaf sprigs confined to torso side panels below v=.65, no floral clutter. Leave plain unembroidered muted rose at u=.24,v=.66.. .88 for rear body sampling. No buttons or front zipper. Lower-right garment region original placement x=.625..1,y=.75..1 matching plain muted rose fabric. Existing shoe islands x=.625.. .8125,y=.125.. .5 charcoal footwear with muted gray sole, simple understated seams, no white sneakers. Preserve arm/hand skin islands positions. Output ONLY flat square UV atlas, no portrait/model/scene/text/grid/border/crop/new islands, no accessories pasted onto face. Exact UV alignment priority; restrained albedo shading without heavy directional shadows or dense noise.

## Clair and Sabrina: individual UV paintings

Used built-in imagegen edit mode from each Rust-exported fallback atlas, without
a second face reference. Installed exact selected outputs:

- `assets/open-models/people/npc-textures/clair.png` from
  `exec-e663a962-83ba-49a0-bdc3-0b7c2eba6e27.png`.
- `assets/open-models/people/npc-textures/sabrina.png` from
  `exec-0ba3388c-37bd-4caf-895d-a6b6a5446323.png`.

Sources remain in the thread's `.codex/generated_images` directory. Clair has
blue hair/eyes and a high blue collar; Sabrina has dark hair, narrow brown-violet
eyes and a burgundy collar with restrained gold stitching. Both replace the
source blue-gray neck strip with skin. Added these roles to painting-only front
and rear torso placement. Neutral upper shoe tint preserves the paintings'
dark footwear color. Existing painted-curtain sampling picks up their hair.

Hand-reviewed both production GPU models from four angles in Idle and Run at
phase .25, against their original fallback captures. Eyes, eyebrows, lips and
hair parting are clearer; torso panel seams and dark footwear now read on the
rig. Sabrina's usual rear patch crossed gold stitching and produced a stretched
diagonal stripe. Repositioned that patch to u=.24 with width scale .01 and
confirmed the stripe is gone in fresh Idle and Run captures. Installed PNG
SHA-256 hashes match the selected generated files. Two supplied-rig UV tests
and all 17 appearance tests passed; production reviewer build and diff check
passed. Temporary capture files were removed after review.

These are texture improvements, not completed character sculpts. Both still
have plain separate hair curtains and visible crown joins, a generic outfit
silhouette, and shoes
whose texture reads as laced footwear rather than authored boots. Clair's cape
and Sabrina's distinctive outfit accessories are not established by this pass.

Follow-up: fixed the skin-colored shoulder gaps in native UV placement. The
raised center collar and lower shoulder fabric require different top sampling
heights: retained v=.475 at the center, smoothly raised the sampling floor to
v=.510 across |x|=.10.. .16. This applies only to Clair and Sabrina's painted
front garment faces. The first, narrower adjustment left a pale speck on
Clair; the final coordinates removed it in four-angle production Idle and
Run (.25) captures of both characters. Three supplied-rig UV regressions pass,
including actual atlas-color sampling of both shoulders from |x|=.16 outward
and exact preservation of geometry, winding, skinning weights, and original
vertex buffers. Reviewer build and diff check passed. No bitmap regeneration
was necessary; the selected paintings are unchanged. Review scratch removed.

### Clair prompt

Use case: style-transfer. Edit this actual renderer fallback into a clean production flat square UV albedo atlas for Clair, a confident adult female trainer with long cobalt-blue hair and fitted blue outfit. Keep EXACT normalized atlas islands and original facial feature coordinates, scale, orientation: upper-left head x0.. .625,y0.. .479, same eyes, eyebrows, nose, lips, ears and hairline. Paint a distinctive composed adult face with sharp clear blue eyes, slender dark-blue brows, small confident closed smile, subtle cheek planes and smooth warm peach skin. Not a child or an elderly woman. Blue hair in organized broad sweeping locks with a few clear strands; hair fills entire upper band through x=.625. Remove the blue-gray strip at bottom of head x=.118.. .48,y=.454.. .479: replace with warm neck skin continuous to y=.479. No stubble/muddy neck streaks. Lower-left clothing x0.. .625,y.479..1 fitted cobalt-blue bodysuit torso, modest high blue collar centered u=.3125,v=.48, subtle central seam with restrained side panels/folds, not an open vest/dress neckline, no cream collar/buttons. Plain matching blue u=.16,v=.65.. .85 for rear torso sampling. Lower-right garment region same blue with restrained panel seams. Shoe islands x=.625.. .8125,y=.125.. .5 dark navy footwear with simple seams and muted gray soles, no white trainers. Preserve exact arm/hand skin islands. Output ONLY flat square UV atlas, no scene/model/portrait/text/grid/border/crop/new islands. UV and feature alignment highest priority, soft organized albedo painting, no dense grain/heavy shadows.

### Sabrina prompt

Use case: style-transfer. Repaint this renderer fallback UV atlas for Sabrina, composed adult woman with very dark brown long hair and burgundy fitted outfit. Output a clean hand-painted stylized game texture with readable expressive features, not photorealistic. Preserve EXACT normalized rectangles and original feature coordinates, size/orientation: head x0.. .625,y0.. .479; same ears, eyes, eyebrows, nose, mouth and hairline. Paint a distinct cool calm feminine face with narrow dark violet-brown eyes, straight fine brows, small neutral closed mouth and smooth warm peach skin, restrained catchlights, no copied generic smiling doll, no heavy makeup/stubble. Dark brown hair broad ordered locks, restrained clear strands, upper band fully hair through x=.625. Remove source blue-gray neck strip x=.118.. .48,y=.454.. .479; warm skin everywhere below face through y=.479. Lower-left clothing island MUST fill x0.. .625,y.479..1 exactly, burgundy bodysuit torso with modest high burgundy neckline at TOP CENTER u=.3125,v=.48, restrained gold edge stitching and subtle central seam/panels, no V neckline, cream collar or buttons. Leave plain burgundy at u=.16,v=.65.. .85 for rear sample. Lower-right garment x=.625..1,y=.75..1 matching burgundy with restrained seams. Shoe islands x=.625.. .8125,y=.125.. .5 dark burgundy footwear with understated seams/gray soles at original coords. Arm/hand skin islands stay unchanged in placement. No accessories/text/grid/borders/crop/scene/portrait/model/new islands. Exact UV alignment priority, broad subtle albedo shading and a few deliberate folds, no dense grain.


### Kris prompt

Use case: style-transfer. Production square UV albedo atlas for Kris, a young female adventurer. Input 1 actual fallback UV atlas EDIT TARGET; input 2 Chris reference ONLY for clean hand-painted finish and readable features. Preserve input 1's exact normalized atlas rectangles, seams and the positions/size/orientation of eyes, eyebrows, nose, lips, ears and hairline. Head fills upper-left x0..0.625,y0..0.475; face features stay at input 1 coordinates. Teal hair (not brown), organized broad locks with restrained strands. Young feminine face, clear brown eyes with tiny catchlights, gentle confident smile, even warm peach skin without muddy vertical neck streaks or stubble. Keep a navy blue jacket and navy trousers. Paint the jacket in the lower-left rectangle x0..0.625,y0.475..1 with a small cream collar at TOP CENTER u=.3125,v=.48, neat central zipper, subtle seams/folds. Leave plain navy fabric around the details, especially u=.16,v=.65.. .85 for rear torso sampling. Shoes, hand/arm skin islands and trousers stay at their original coordinates; shoes white, hands same skin. Output ONLY flat square UV texture atlas, no model, scenery, pose, text, grid, borders or cropping. UV and feature alignment highest priority. Do not copy Chris's male face or red clothing.

### Red prompt

Use case: style-transfer. Production square UV albedo atlas for Red, a young male adventurer. Input 1 actual renderer fallback atlas EDIT TARGET. Input 2 Chris is ONLY a reference for clean hand-painted finish and feature readability. Preserve input 1's normalized UV rectangles and head/face feature positions, scale and orientation: same ears, eyes, eyebrows, nose and mouth coordinates, same hairline. Make a youthful male face with clear brown eyes and small highlights, a restrained determined smile, warm peach skin with soft gradients, remove adult beard/stubble shadow and muddy vertical neck streaks. Very dark brown hair in organized locks, not a flat black slab. Clothing island lower-left x0.. .625,y.475..1: RED OPEN VEST over white short-sleeved shirt, centered at u=.3125, small collar at v=.48, white shirt visible centrally, red vest sides with two subtle pockets and seams. No Chris zippered jacket. Plain red fabric at u=.16,v=.65.. .85 for rear sample. Preserve pants rectangle lower-right as blue jeans; shoes white with red accents, arms/hands skin peach, all at existing UV coordinates. No oversized grain or heavy baked shadows. Output only flat square atlas, no model, scene, text, grid, border, crop, rotation or new accessories. Exact UV alignment priority.

### Mom prompt

Use case: style-transfer. Production flat square UV albedo atlas for Mom, a warm mature adult woman with brown bob hairstyle. Input 1 actual fallback UV atlas EDIT TARGET. Input 2 Chris is ONLY a reference for the clean organized hand-painted finish; do NOT copy his face or outfit. Preserve all input1 normalized rectangle/island boundaries, original eye/brow/nose/lip/ear feature coordinates, sizes, orientation and hairline. Repaint the blurry source into a readable mature feminine face, warm brown eyes with restrained highlights, friendly small smile, soft cheeks and subtle adult eyelids; not a young girl or elderly grandmother. Warm peach skin with smooth soft gradients, remove muddy neck streaks and dirty dark chin. Brown hair in ordered broad locks with few strands, respecting the original bob hairline. Lower-left clothing island x0.. .625,y.475..1: modest warm red blouse, small rounded cream collar at TOP CENTER u=.3125,v=.48, neat small buttons, subtle fabric folds; keep plain red fabric at u=.16,v=.65.. .85 for rear sampling. Lower-right existing garment region remains burgundy skirt/leggings tone; original white shoe islands and arm/hand skin coordinates intact. Only a few folds, no noisy grain. Output ONLY square flat UV texture, exact atlas placement highest priority; no portrait/model, scene, text, border, grid, crop, rotation or new islands/accessories.

### Mom atlas boundary repair prompt

Use case: precise-object-edit. Input 1 is the Mom UV painting EDIT TARGET, input 2 is the original renderer atlas for EXACT NORMALIZED RECTANGLE BOUNDARIES only. Repair an atlas seam: input1 incorrectly ends its upper-left head/hair island around x=.600; it MUST extend to x=.625 (5/8 of square width), matching input2. The upper-left x0.. .625,y0.. .475 rectangle is entirely head/hair/skin, never shoe/gray background. In particular extend brown hair rightwards to x=.625 along y0.. .125 so the back scalp cannot sample the white shoe wedge. Shoe island must start at x=.625,y=.125, width=.1875 to x=.8125. Arm skin begins x=.8125. Likewise clothing island lower-left must end at x=.625. Keep Mom's existing mature feminine face, original eyes/nose/mouth/ears coordinates and painting quality; keep cream collar/buttons, red blouse/burgundy clothing and all skin unchanged except needed rectangle boundary correction. Maintain full square framing. No crop, new details, text/grid, scene, model. This is an exact UV layout repair, not a redesign. Align all rectangles to input2; preserve face feature positions from input1.

### Bugsy individual UV repaint — 2026-10-10

Mode: built-in image_gen edit, one actual fallback atlas as the sole input.
Final selected output: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-76fe4fca-61ca-4a6f-82b8-5da0f244eff1.png`.
Installed unchanged at `assets/open-models/people/npc-textures/bugsy.png`.

Final prompt:

```text
Use case: precise-object-edit
Asset type: production 3D character UV texture atlas for Bugsy, a youthful male insect-themed trainer.
Input image: sole edit target is the exported actual UV atlas. Repaint this exact square atlas, retaining the precise island layout, silhouette boundaries and relative placement. Output a high resolution square atlas.
Primary request: replace the crude source painting with a polished soft hand-painted stylized game character texture. Give the youthful face a calm friendly closed-mouth expression, distinct small brown almond eyes, clean brows, gentle nose, warm natural skin with subtle cheek shading. Deep violet-purple short bob hair has fine restrained painted strands and soft highlights. Rich leaf green shirt with subtle fabric folds and neat collar; olive khaki shorts. White clean trainer shoes with subtle gray laces and soles.
Layout constraints: the left 62.5% contains the head in its upper 47.5%, green torso in its lower 52.5%. The central face must remain precisely positioned within the head island; keep purple hair extending fully to the head island edge. Preserve the source skin neck strip at the very bottom of the head island, no green collar band in that strip. The middle vertical narrow shoe strip at x62.5%-80% stays shoes. Rightmost skin islands stay skin. Lower-right olive shorts islands remain olive cloth. Paint only inside the corresponding existing regions; no new islands or shifted regions.
Style: elegant readable stylized character painting with soft directional value shading, clean edges and realistic restrained facial proportions appropriate to a child. No cartoon open mouth, no huge white eyes, no overly photorealistic adult face, no black outlines. No labels, diagrams, text, watermarks, scene or model render. Preserve atlas registration exactly.
```

Hand-reviewed four-angle production Idle and Run .25 renders. The old coarse,
open-mouth shared child face is replaced by a closed-mouth individual face.
Purple strand painting covers the crown; the existing curtain samples only the
hair strip. Green fabric now has a readable collar and folds, khaki shorts retain
skin legs, and neutral shoe tint preserves the painted white footwear. Added
Bugsy to front torso mapping and a plain rear fabric patch, preserving source
geometry, normals, skin islands and weights through exact copied corners.
The three supplied UV regressions pass, now including Bugsy in torso coverage;
all 17 appearance regressions pass after installing the final atlas.
Reviewer build and diff check pass. No neck color band or newly displaced facial
features was observed in the sampled poses.

This is the nineteenth individual painted atlas. The side/rear bob remains
plain, the crown/curtain join remains visible, and the source hand silhouette
and costume are simplified. Bugsy and the full cast remain unfinished.

### Receptionist individual UV repaint — 2026-10-10

Mode: built-in image_gen edit; actual exported fallback atlas was the sole input.
Selected output: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-f6e80952-af63-48f1-8648-365930015239.png`.
Installed unchanged at `assets/open-models/people/npc-textures/receptionist.png`.
SHA-256: `a10b9abcc900804396ee8419692cdbef737f102f8d92400d9cfea72f2619f20c`.

Final prompt:

```text
Use case: precise-object-edit
Asset type: production 3D character UV albedo atlas for the Receptionist, an adult woman in a navy skirt suit.
Input image: sole edit target is the actual exported source atlas, not a render. Repaint this exact square atlas at high resolution, retaining all island positions and boundaries.
Primary request: a polished soft hand-painted stylized game character texture. Give her an individual composed friendly adult face: warm light skin, moderately narrow hazel brown eyes, straight dark eyebrows, softly defined oval face, small gentle smile with closed lips. Chestnut brown side-parted bob hair with restrained smooth painted strands and natural highlights, continuing fully to the head island boundaries.
Clothing: deep muted navy fitted jacket over an ivory blouse with small crisp collar, a slim burgundy tie at the front center, subtle jacket lapel seams and two small navy buttons. Small ivory rectangular namebadge on one upper chest, with no lettering. Navy cloth folds are subtle and readable. Lower-right clothing island remains solid deep navy tights with gentle shading, no bare skin. Shoe strip remains charcoal black simple low shoes, with subtle seams, no bright white shoe body.
Layout constraints: left62.5% head above y47.5%, left62.5% torso below y47.5%. Face stays precisely registered to the existing central face. The bottom strip of the head island stays skin and must never contain a blue clothing collar. Start all blouse and jacket features below the original torso boundary at y47.5%. Middle narrow x62.5%-80% shoe strip remains shoes. Rightmost upper and middle islands remain natural skin. Lower-right island below y74.5% remains navy cloth. Preserve source island boundaries exactly; no new islands.
Keep plain navy fabric available away from front-center details around x24%, y75% for skirt color sampling.
Style: clean hand-painted soft stylized 3D game material, restrained facial proportions, gentle shading. No eyes enlarged into cartoon circles, no open mouth, no hard outlines, no text, labels, watermarks, scene, model render, or UV diagram.
```

Hand-reviewed baseline and final production GPU captures from four angles in
Idle and Run .25. Individual adult face and chestnut strand painting replace
the coarse fallback; front blouse collar, burgundy tie, lapels and ivory badge
read clearly on the source torso. Added front UV mapping with exact copied
positions/normals/joints/weights. The first rear patch sampled a front lapel,
creating a broad false shoulder fold; selected plain fabric at u=.075 instead.
The separate skirt samples the painting's navy fabric to keep its palette fitted
to the jacket. Neutral shoe tint reveals the painting, with a role-specific
smooth rear tint correcting pale UV-surround sampling at the heels while
retaining front laces. Final Idle/Run captures show clean rear fabric and dark
heels. No bitmap postprocessing was used.

Reviewer build passes. Three supplied UV regressions pass on final source,
including Receptionist torso mapping; all 17 appearance regressions pass,
including all 29 costume profiles in eight Run samples. Diff check passes.
Owned scratch output was removed after processes finished.

This is the twentieth individual painted atlas. Crown/curtain joins, the plain
rear hair, simplified hands and separate generic skirt remain unfinished.
The full 75-role character objective is still active.

### Whitney individual UV repaint and crown — 2026-10-10

Mode: built-in image_gen edit, exported actual fallback atlas as sole input.
Selected output: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-c2e04b67-f635-4f89-8cf4-4454173e6396.png`.
Installed unchanged at `assets/open-models/people/npc-textures/whitney.png`.
SHA-256: `b04754759e5d93c8c8458ed8b7f83a70f3288fb71afec80e57c68b6f255e3a7c`.

Final prompt:

```text
Use case: precise-object-edit
Asset type: production stylized 3D character UV albedo atlas for Whitney, a cheerful young woman with rose-pink hair and a casual athletic outfit.
Input image: sole edit target is the exported actual atlas. Repaint the exact square UV layout at high resolution; retain all island positions and boundaries.
Primary request: clean soft hand-painted game character material. Give her a distinct youthful round-cheeked face, warm light skin, small warm brown almond eyes, curved natural brows, small nose, gentle confident closed-mouth smile. Rose pink hair with a softly parted fringe and restrained painted strands; all source head hair areas remain fully hair to their boundaries. Hair is muted warm rose, not neon magenta.
Clothing: warm cream casual shirt with a neat red ribbed collar, one broad red horizontal chest stripe centered around normalized y=.67 across the torso fabric, subtle soft cloth folds. No lettering or logos. Lower-right cloth island is deep brick red shorts with subtle seams. Shoe strip is red trainers with cream laces, toe trim and soles.
Layout constraints: left62.5% head above y47.5%, left62.5% torso below y47.5%. Central face retains exact registration. The bottom strip of head stays skin, no red clothing band in this skin neck strip. All shirt features start below the existing torso boundary. Middle narrow x62.5%-80% shoe strip remains shoes. Upper/middle right skin islands remain skin, lower-right island below y74.5% remains shorts cloth. No shifted islands, new islands, rendered model, diagram, labels, text, watermarks or background scene.
Keep plain cream fabric around normalized x=.075,y=.78 for rear shirt sampling. Clean readable face and cloth, restrained stylized proportions, no photoreal adult face, no enlarged cartoon eyes, no open mouth or hard black outlines.
```

Hand-reviewed baseline and final four-angle production Idle and Run .25
captures. Individual youthful face, painted rose hair, cream/red front shirt,
brick-red shorts and red/cream trainers replace the coarse fallback. Added
front torso mapping and a plain cream rear patch; exact copied face corners
preserve body positions, normals, skin UV islands and animation weights.
Source sleeve UVs produce red raglan sleeves. The stripe remains a front detail.
Neutral shoe tint retains the painted laces and toes.

Enabled crown sculpting only for Whitney among previously excluded tied styles:
three refinements, soft ellipsoid support and a shallow center part round the
flat-sided crown. Face/body/skin, original UVs and original joints/weights stay
protected. The existing 75-role source regression now checks 27 distinct crown
profiles, closed welded scalp edges, valid winding, finite unit normals and
eight Run samples; it passes. The first repaint render exposed bright fallback
pink pigtails against the muted painted scalp. Sampled the hair-only painting
strip for Whitney's gathers; final Idle/Run captures show closer color without
changing their shape, roots or head attachment.

All three UV regressions and 17 appearance regressions pass, including the
updated torso coverage and all 29 costume profiles. Reviewer build and diff
check pass. Scratch renders/logs were removed after all owned processes ended.
This is the twenty-first individual painted atlas. Pigtail joins, angular neck,
simplified hands and generic sleeve construction remain unfinished; the full
75-role objective remains active.

### Misty individual UV repaint, crown and sleeves — 2026-10-10

Mode: built-in image_gen edit. Actual exported fallback atlas was the first
sole input; the first output was the sole input for a targeted collar repair.
Initial output: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-d251c6ca-eb14-4e8b-8e69-6467701be558.png`.
Selected repair: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-15419c1a-6693-490d-b02b-e07d12b5b95b.png`.
Installed unchanged at `assets/open-models/people/npc-textures/misty.png`.
SHA-256: `39dde48688880e6ec26b84f873c4df2828b6f4a7e9b1f220cedd30a09296f03b`.

Initial prompt:

```text
Use case: precise-object-edit
Asset type: production stylized 3D character UV albedo atlas for Misty, a sporty young woman with copper-orange hair, yellow shirt, red suspenders and blue denim shorts.
Input image: sole edit target is the actual exported fallback atlas. Repaint this exact square atlas at high resolution. Preserve all island placements and boundaries.
Primary request: polished soft hand-painted character material. Give her an individual youthful face with light warm skin, softly angular cheeks, small teal-green almond eyes, expressive slightly angled copper brows, small nose, faint freckles, and a subtle self-assured closed-mouth smile. Copper-orange hair with a softly side-swept fringe and restrained directional strands, all hair extending to the source head island boundaries.
Clothing: sunny warm yellow crew-neck shirt with subtle fabric folds and a neat neckline. Two slim red suspenders run vertically down the front at normalized x=.23 and x=.40, with small understated metal clips near the shirt hem. Keep plain yellow fabric around normalized x=.075,y=.77 for sleeve and rear samples. Lower-right cloth island is muted blue denim shorts with small seams, no lettering or logos. The shoe strip is red sporty trainers with white laces, white toe details and neutral gray soles.
Layout constraints: left62.5% head above y47.5%; torso below y47.5%. Exact central facial registration. Head bottom neck strip must remain natural skin, never yellow or red clothing. Start neckline and suspenders below the original torso boundary. Middle narrow x62.5%-80% shoe strip remains shoes. Upper/middle right skin islands stay natural skin. Lower-right below y74.5% stays denim cloth. No new or shifted islands.
Style: soft clean readable stylized 3D game painting with restrained facial proportions, no huge eyes, open mouth or black outlines. No labels, text, watermark, UV diagram, scene or model render.
```

Repair prompt:

```text
Use case: precise-object-edit
Asset type: existing Misty UV atlas, exact registration required.
Input image: sole edit target.
Change ONLY the unwanted blue-gray horizontal strip immediately above the yellow shirt collar, at normalized x=.12 through .50 and y=.475 through .500. Replace that blue-gray strip with matching yellow shirt fabric so that the yellow shirt begins at the original torso boundary y=.475. The blue-gray must be completely gone. Keep the natural skin of the head and neck above y=.475 exactly as it is; do not create a colored strip in the skin. Preserve the existing yellow neckline fold below this strip, both red suspender straps and clips, all face features, orange hair, skin islands, shoes, denim and every other pixel/layout as closely as possible. Do not redraw the face or clothing. No new objects, labels or text. This is one tiny registration repair, not a redesign.
```

The first painting introduced a blue-gray strip above the yellow collar. It was
repaired with imagegen before installation; no bitmap postprocessing was used.
Hand-reviewed four-angle final production Idle and Run .25 captures, plus Red's
shared sleeve control. The individual teal-eyed freckled face, copper hair,
yellow shirt/red suspenders, denim shorts and red/white trainers improve the
coarse fallback. No false blue collar was observed on the final model.

Added front torso mapping and plain rear yellow sampling. Extended the existing
whole-face short-sleeve mapping to Misty with yellow fabric and exposed forearm
skin; Red retains its original white sleeve mapping. Complete source arm rings
avoid a sawtooth hem. Geometry, normals, colors, joints and weights remain exact
at all corresponding triangle corners; actual painting tests verify yellow/white
sleeves and warm skin on both sides of both supplied rigs.

Misty now uses a softly rounded crown with a shallow swept part, protecting
source face/body/skin, UVs, joints and weights. The all-role crown regression
passes 28 distinct sculpted profiles, closed scalp seams, valid winding,
finite unit normals and eight Run samples. The ponytail samples the painting's
hair-only strip so it follows the copper scalp palette; its shape and roots
remain unchanged. All three UV and 17 appearance regressions pass, reviewer
build and diff check pass. Owned scratch output was removed after all processes
finished.

This is the twenty-second individual painted atlas. Angular neck, simplified
hands, plain rear fabric and the ponytail root remain unfinished. The full
75-role objective remains active.


## Janine — 2026-10-10

Built-in imagegen precise-object-edit mode; actual exported fallback atlas was
the sole input, inspected before generation. Installed selected output at
`assets/open-models/people/npc-textures/janine.png`. No bitmap postprocessing.

Source: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-b717bf4c-77ef-46db-b321-af53874ad410.png`

SHA-256: `8c3998e221c687c4cfe50a3fa1e1b294563f5812b858872dd6c446b79fc7dfe2`

Distinct focused hazel-brown eyes, painted plum hair, red scarf and overlapping
plum tunic replace the generic fallback. Front torso mapping uses width .56;
rear fabric samples the plain left strip at [.075,.64,.008,.15]. Shoe upper
palette allows the painted charcoal-plum panels through. The rounded high
ponytail crown preserves face, body, UVs and skin weights, and the existing
ponytail samples the painting's hair-only strip for its palette.

Hand-reviewed actual production Idle and Run .25, four angles each. The front
scarf and diagonal seam remain on the front; rear cloth is clean and the
previous stray white waist line is absent. Ponytail color matches the scalp.
Grey heels and soles, simplified neck/hand anatomy, plain rear fabric and
missing gloves remain unfinished; this is not a complete ninja costume.

Three supplied UV tests and the all-cast crown regression pass (29 distinct
sculpted crown profiles). Twelve supplied appearance regressions pass,
including tied hair, costume animation and protected face/skin checks.
The five ordinary appearance tests also pass (21 targeted tests total).
Reviewer build and diff check pass. This is the twenty-third individual
painted atlas; the full 75-role objective remains active. Owned scratch output
was removed after every build, test and capture process finished.

Generation prompt:
```text
Use case: precise-object-edit.
Asset type: square character UV texture atlas for an existing skinned female game mesh. Image 1 is the sole edit target, an actual exported Janine atlas.
Repaint its surface artwork into a polished hand-painted stylized young ninja character. Preserve exactly the normalized UV island positions, boundaries, orientation and all eye/nose/mouth positions. Output a square atlas only, no rendered character, layout labels, text or watermark.
Face: distinct youthful focused warm hazel-brown almond eyes, clear brows, small closed confident smile; soft warm skin, delicately painted cheeks and ears, readable features with restrained stylization. Keep the existing front face centered at u .3125 v .21 and skin islands across the entire upper strip and right middle.
Hair: muted lavender plum, layered smooth painted locks, gentle highlights and rich roots. Fill all original hair territory through u .625 with hair. Preserve hairline and the face silhouette.
Clothing: deep plum ninja tunic with subtle woven fabric, a clean overlapping diagonal front seam and a red scarf folded just below the neck. Clothing occupies ONLY u 0.. .625 and v .475..1; skin directly above v .475 stays skin, no dark collar band there. Place scarf front centered u .3125 at v .49.. .57; keep left-side fabric u .04.. .11 v .60.. .95 plain deep plum for sleeves and back. Front overlap seam within u .18.. .46, not left plain patch. Restrained stitching, no large symbols or badges. Lower-right pants island remains deep plum with subtle seams, avoiding new belts or white stripes.
Original shoe islands at u .63.. .81, upper half: dark charcoal plum ninja shoes, modest grey sole, subtle panel seams, no bright white laces. Other skin including right-side hands must stay warm skin and mapped identically. Flat painted lighting, no dramatic baked cast shadows. Seam-safe generous color gutters.
```


## Koga — 2026-10-10

Built-in imagegen precise-object-edit mode. Sole input was the actual exported
Koga fallback atlas, inspected before generation. Selected output installed
unchanged at `assets/open-models/people/npc-textures/koga.png`.

Source: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-52b80b0d-a2d1-478f-886a-bef02a07c675.png`

SHA-256: `618a14946f758df5f1ded0d1d501e237f05746b79abc0d42d7480087273cd15f`

Koga generation prompt:
```text
Use case: precise-object-edit.
Asset type: square character UV atlas for an existing skinned male game model.
Input image 1 is the sole edit target: the actual exported fallback Koga atlas. Repaint this exact atlas, preserving all UV island placements, orientation, hairline, eye/nose/mouth coordinates and skin boundaries. Output the square atlas only, no rendered figure, labels, lettering or watermark.
Character: mature confident ninja master, calm narrow dark brown almond eyes, strong dark eyebrows, closed neutral mouth, subtle cheek and nasolabial definition, restrained fine crow's feet. Warm natural skin, clean shaved face without beard or moustache. Face centered at u .3125 v .21. Retain original hairline silhouette and skin across the entire original upper strip and right middle.
Hair: deep charcoal brown with dark warm slate highlights, clearly layered swept short locks, painterly strands, no colored streaks or symbols. All original hair territory through u .625 stays hair.
Clothing: deep navy blue ninja tunic with quiet woven fabric and diagonal overlapping front panels, dark red folded scarf at the upper center of the shirt. Clothing ONLY u 0.. .625 and v .475..1. Skin above .475 stays skin; no extra dark band crossing neck. Scarf centered at u .3125 v .49.. .56. Front overlap within u .15.. .48; left fabric strip u .05.. .11 v .60.. .95 must remain plain navy for sleeves/back, without scarf or seams. Modest navy fabric waist tie at the bottom center; no white belt. Lower right pants island remains navy with restrained panel seams and cloth folds.
Shoe islands u .63.. .81 in upper half become charcoal navy shoes with understated grey soles and subtle dark stitching; avoid white laces. All hand/skin islands including right middle remain skin, no clothes painted across skin. Soft painted surface detail, restrained highlights and gradients, no strong directional shadows, generous matching color gutters, coherent seamless colors. Preserve the technical layout precisely.
```


Koga review results: hand-reviewed actual four-angle production Idle and Run
.25 captures. Mature focused facial features, layered charcoal hair, navy
overlap panels and red scarf replace the generic fallback. Front mapping
uses width .90; rear maps plain left cloth [.075,.64,.008,.15], eliminating
the fallback white waist stripe. Painted shoe panels pass through a neutral
upper palette. Existing crown shape remains unchanged.

Initial rear shoe captures exposed pale gutter bleed. Native upper-heel tint
now applies only to Koga and Janine: the existing height blend protects the
sole and a smooth forward blend protects front panel artwork. The grey upper
heel is darker, with grey soles retained. Both final roles were hand-reviewed
four angles each in Idle and Run .25. No bitmap postprocessing was used.

Three supplied UV regressions pass, including Koga's corresponding triangle
positions, normals, joints and weights. The existing supplied footwear
regression passes for all four base rigs with its Officer control; the new
ninja heel palette was verified in actual renders. Reviewer build and diff
check pass. Task-owned scratch was removed after every process finished.

This is the twenty-fourth individual painted atlas. Missing wrist wraps,
simplified hand/neck anatomy and plain rear cloth remain unfinished; the full
75-role objective remains active.


## Morty — 2026-10-10

Built-in imagegen precise-object-edit mode. Sole input was the actual exported
Morty fallback atlas, inspected before generation. Selected output installed
unchanged at `assets/open-models/people/npc-textures/morty.png`.

Source: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-61d48e40-706c-4aa1-aafb-079957c7e24a.png`

SHA-256: `002a52d46b2704dd98adbc9e74eab1f65159153797965190fef60e0b29578790`

Morty generation prompt:
```text
Use case: precise-object-edit.
Asset type: square game character UV atlas, polished hand-painted stylized surfaces.
Image 1 is the sole edit target, the actual exported Morty fallback atlas. Repaint its artwork while retaining exact normalized island positions, skin boundaries, face orientation and existing eye/nose/mouth coordinates. Output only a square texture atlas; no figure render, labels, lettering or watermark.
Morty is a young adult with a composed, slightly enigmatic expression: soft grey-green almond eyes, natural brows, youthful clean-shaven cheeks and a small closed neutral smile. Distinct from an older stern ninja face. Warm fair skin with restrained cheek shading and gently defined nose/lips. Face centered u .3125 v .21, ears and neck remain aligned. All original upper-strip and right-middle skin stays skin.
Hair: warm sandy blond, golden beige highlights, gentle darker honey roots, clearly swept side-part strands, soft painterly locks, no headband painted into hair or forehead (a mesh accessory will be fitted separately). Preserve the original hairline silhouette and all hair territory through u .625.
Clothing: muted deep plum long-sleeved shirt, soft woven jersey, understated seams, a warm pale lavender scarf folded across the front upper chest. Clothing only u 0.. .625 v .475..1. Keep skin directly above .475 unchanged and no additional neck band. Scarf centered u .3125 v .49.. .57; plain plum fabric u .04.. .11 v .60.. .95 for sleeves and rear without knots, stripes or collar motifs. Shirt hem quiet, no symbols or logos. Lower-right pants island becomes warm khaki with restrained pocket seams, subtle fabric folds and no white waistband.
Shoe islands u .63.. .81 upper half: modest charcoal brown shoes with dark warm grey soles and subtle seams, no bright white laces. All right hand skin and other skin islands remain warm skin. Seam-safe color gutters. Flat painted lighting, gentle gradients, no strong baked cast shadows. Preserve atlas geometry exactly.
```


Morty now has a distinct grey-green-eyed youthful face, swept sandy blond
hair, a plum shirt with pale lavender scarf, and khaki trousers. Front torso
mapping uses width .90 and plain rear sampling [.075,.64,.008,.15]. Existing
Morty side-part crown geometry remains unchanged. Painted shoes use a neutral
upper tint, with the existing dark-heel correction extended to Morty to reduce
pale gutter bleed while retaining the soles and front panels.

Added `headband.rs`: a thin closed lavender cloth ring fitted by radial
intersections with the same refined head surface used in production. Three
rows and 64 sides form inner/outer surfaces and closed top/bottom caps. The
band sits above the eyes at normalized head heights .24.. .38, with .007
thickness and an inner radial clearance .007. The source face UVs and body
remain unchanged; the head accessory uses the production inverse head bind.
The supplied regression verifies finite unit normals, nondegenerate faces,
closed consistently oriented edges, thickness, bind-space correspondence,
eight Run samples within rig bounds plus .03, and a Koga no-op control.


The initial shirt edge picked up khaki from the adjacent trouser island at
u=.605. Morty's front upper limit now stays at .595. Actual final four-angle
Idle and Run .25 views were hand-reviewed: the khaki shirt streak is absent,
scarf remains on the front, back is clean, eyes stay visible under the fitted
headband, and heels retain the darker upper / grey sole distinction. No bitmap
postprocessing was used.

Three UV regressions and the supplied headband regression pass (four targeted
tests), with reviewer build and diff check passing. Source clothing geometry
and animation weights remain exact at corresponding triangle corners.
Task-owned scratch output was removed after every process finished.

This is the twenty-fifth individual painted atlas. Plain headband without ties,
plain rear fabric and simplified neck/hand anatomy remain unfinished. The
full 75-role objective remains active.


## Falkner — 2026-10-11

Built-in imagegen precise-object-edit mode. Sole input was the actual exported
Falkner fallback atlas, inspected before generation. Selected output installed
unchanged at `assets/open-models/people/npc-textures/falkner.png`.

Source: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-ee6fc944-c14b-4d57-98a5-adf4c1e56e73.png`

SHA-256: `c15c5b17298118fe1f1ce24daac7a7a156536efd268d664ae2d2f55a1ca86599`

Falkner generation prompt:
```text
Use case: precise-object-edit.
Asset type: square character UV atlas for an existing male skinned game mesh, polished hand-painted stylized surfaces.
Image 1 is the sole edit target: actual exported Falkner fallback atlas. Repaint the exact normalized layout, preserving all islands, hairline silhouette, skin boundaries and original eye/nose/mouth locations. Output only a square atlas, no rendered character, labels, lettering or watermark.
Character: youthful poised bird trainer, clear slate-blue almond eyes, natural narrow eyebrows, small closed confident smile, clean-shaven face, gently defined cheekbones, warm fair skin. Distinct from a generic older male. Face centered u .3125 v .21. Preserve original ears/neck/hand alignment.
Hair: rich dark navy blue, layered swept side-part locks with muted blue slate highlights and quiet deep roots. Keep original hairline and all hair territory through u .625. No new fringe covering the face or moving eyes.
Clothing: muted indigo-blue kimono fabric with subtle woven texture, a clean diagonal overlapping front lapel, a slim warm ivory inner collar. Clothing strictly u 0.. .625 v .475..1. Skin above .475 stays skin, no extra neck band. Place front lapels within u .15.. .48, ivory collar within u .22.. .41 v .49.. .59. Plain blue fabric u .04.. .11 v .60.. .95 for sleeves/rear, no seams or ivory stripes there. No waist belt or obi in this painting. Lower right pants island remains dark indigo blue with understated fabric seams and folds; no white waistband.
Shoe islands u .63.. .81 upper half become understated dark charcoal shoes with dark grey soles, no bright white laces. All skin including right middle hands and upper skin strip stays warm skin. Seam-safe matching color gutters. Flat painted surface lighting, restrained gentle gradients, no strong baked shadows. Preserve precise UV placement.
```


Falkner now has a distinct slate-blue-eyed youthful face, swept navy hair,
painted indigo kimono lapels and ivory inner collar. Front torso mapping uses
width .90, with .595 right limit to stay inside the shirt island. Rear fabric
samples the plain left strip [.075,.64,.008,.15]. Separate lower garment
color samples the same painting's plain fabric at u=.075,v=.75; the existing
crown and lower garment geometry remain unchanged. Shoes use a neutral upper
palette and the established darker upper-heel fade, preserving grey soles.

Extended native kimono sleeves from Erika to Falkner. Initial shared shape
looked too rounded at the back on the male rig; final Falkner hang .38 and
depth .18 replace Erika's .29/.45 parameters. Whole skin/cloth seam rings
remain pinned, with selective refinement and original joints/weights retained
at source vertices. The supplied regression now covers both male and female
rigs: changed cloth on both sides, preserved skin/UVs/influences, finite unit
normals, normalized weights, eight Run bounds samples, and Mom no-op controls.
Erika retains the same production parameters.

Hand-reviewed actual final four-angle Idle, Run .25 and Run .75 captures for
Falkner, plus Erika's four-angle Idle/Run .25 control. Wrist seams remain
connected in reviewed poses; front collar stays on the front and rear cloth
is clean. No bitmap postprocessing was used. Three UV tests, the expanded
sleeve test and all-profile costume animation test pass (five targeted
regressions). Reviewer build and diff check pass. Task-owned scratch output
was removed after every build/test/capture process finished.

This is the twenty-sixth individual painted atlas. The obi was added in the
following native review. Plain lower garment/rear fabric and simplified
hand/neck anatomy remain unfinished.
Full 75-role objective remains active.


### Falkner's fitted obi — 2026-10-11

Extended the native closed fitted sash from Erika to Falkner. Falkner uses
ivory fabric with quiet darker edges, a .24-high waist band (y1.87..1.63),
and a compact flattened rear knot with two short cloth ends. Erika retains
her previous .29 band, colors and wide rear bow. Every added component uses
the fitted torso influences; leg influences transfer to the pelvis to avoid
pulling the sash apart during running. No texture asset was changed.

Expanded the supplied closure/fit test to both male and female rigs. It passes
consistent face winding, closed edge counts, finite unit normals, normalized
weights without thigh influences, and .019.. .056 radial body clearance.
The all-profile costume regression passes all 29 garments and eight Run
samples. Actual four-angle Falkner Idle/Run .25/Run .75 captures were reviewed;
waist and rear knot remain fitted in sampled views. Erika's Idle control is
pixel-identical to the pre-change capture. Reviewer build and diff check pass.

The shared renderer diagnostic initially failed to compile because Vec2 was
unqualified. Qualified that single expression with bevy::prelude::Vec2;
behavior is unchanged. Subsequent build and costume test succeeded. Removed
owned scratch after every build/test/capture process finished. Plain lower
garment/rear fabric and simplified hand/neck anatomy still need work. Full
75-role objective remains active.


## Lance — 2026-10-11

Built-in imagegen precise-object-edit mode. Initial sole input was the actual
exported Lance fallback atlas, inspected before generation. The initial output
added a grey shoulder band and shirt-edge strip; a second built-in edit repaired
those strips before installation. No bitmap postprocessing was used.

Initial rejected output: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-d4b81f4f-524a-4bb4-a295-ab2bfb0b62c2.png`

Selected source: `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-bd23db9a-b379-4cc1-a1e2-16fcb8fdd059.png`

Installed at `assets/open-models/people/npc-textures/lance.png`.
SHA-256: `194b1fa04ef3e70fb4b9b00e8bc5198ccba2b73c74d90d64c50fe5d2b2ea340b`.
Atlas 1254 square, 1,703,652 bytes, within production decoder limits.

Lance initial generation prompt:
```text
Use case: precise-object-edit.
Asset type: square character UV atlas for an existing male skinned game mesh; polished hand-painted stylized surfaces.
Image 1 is the sole edit target, the actual exported Lance fallback atlas. Repaint its artwork without moving normalized UV islands, skin boundaries, original hairline silhouette or eye/nose/mouth coordinates. Output atlas only, no rendered figure, labels, text or watermark.
Character: confident young adult dragon trainer, clear warm amber-brown almond eyes, expressive firm brows, strong but youthful cheekbones, small closed composed smile. Clean-shaven warm fair skin. Face centered u .3125 v .21, ears and neck aligned. Avoid copying the generic male fallback or making a stern middle-aged face.
Hair: rich burnt crimson and auburn red, layered swept spiky locks painted as connected clumps with fine strands and muted copper highlights. Retain original hairline, don't put bangs over the eyes. Hair occupies all original hair territory through u .625.
Clothing: tailored deep crimson jacket, subtle heavy woven fabric, small antique-gold buttons down the center and restrained gold piping, dark charcoal standing collar. Clothing only u 0.. .625 v .475..1; skin directly above .475 remains skin, no added dark strip across that skin. Collar within front u .20.. .43 v .48.. .55. Buttons at center u .3125 v .60.. .86, small at game scale. Plain deep crimson fabric u .04.. .11 v .60.. .95 for sleeves and rear, no piping or buttons there. No cape painted into this atlas; that must be a separate mesh. No white waistband. Lower-right pants island stays charcoal navy with subtle seams and cloth folds.
Shoe islands u .63.. .81 upper half become charcoal leather shoes with dark grey soles, quiet panel seams, no white laces. Other skin including right-middle hands remains warm skin. Seam-safe matching gutters, soft painted lighting with restrained gradients, no strong baked shadows. Preserve technical atlas placement precisely.
```

Lance cleanup prompt:
```text
Use case: precise-object-edit. Input image 1 is the sole edit target, the Lance UV atlas just generated. Make one technical cleanup only. Replace every light grey/blue-grey strip in the upper shirt island with the neighboring deep crimson woven jacket fabric. Specifically the horizontal grey shoulder strip at u 0.. .19 and u .42.. .625, v .475.. .502, and the narrow vertical grey shirt-edge gutter at u .605.. .625, v .502.. .744. These are shirt pixels and must be crimson with matching texture, no remaining pale band. Preserve the central charcoal/gold standing collar exactly. Preserve ALL face, skin above v .475, hair, shoe islands including their grey surround u>=.625, hands, navy pants island and all other jacket artwork/button/piping pixels unchanged. The navy trouser island starts at v .745 and must stay navy. No redesign or new symbols. Same exact square UV layout, colors, resolution and island boundaries. Output corrected atlas only.
```


Lance now has a distinct amber-eyed youthful face, painted auburn/crimson
hair, tailored crimson jacket, charcoal/gold collar and small gold buttons.
Front mapping uses .90 width with .595 right limit; rear maps the plain left
fabric [.075,.64,.008,.15]. Painted charcoal shoes use neutral uppers and the
established darker upper-heel fade while retaining grey soles.

Native crown: Lance's rise drops .66 to .42 and rear sweep -.16 to -.12.
His clump support grows .42 to .50, with a cosine profile rounding the peak
and easing into the scalp. Other identities retain their original clump
profiles. Existing hairline/face/body/UV/joint-weight protection remains in
place. The all-cast crown regression passes 29 distinct profiles with closed
seams, winding, unit normals and eight Run samples; all three UV regressions
also pass (four targeted tests total). Reviewer build and diff check pass.

Hand-reviewed actual final four-angle production Idle and Run .25 captures.
The corrected shoulder/edge strip is crimson, dark collar/buttons remain
on the front, rear fabric is clean, and shorter crown clumps have rounded
tips. No bitmap postprocessing. Owned scratch was removed after every
generation/build/test/capture process finished.

This is the twenty-seventh individual painted atlas. The missing red cape
is resolved in the native pass below. Proper boot shafts, plain rear cloth
and simplified hand/neck anatomy remain unfinished. The full 75-role
objective remains active.

### Lance native cape follow-up — 2026-10-11

No raster generation or atlas modification in this follow-up. A closed native
folded cape uses crimson outer and dark lining vertex colors with chest/hip
skinning. The supplied mesh regression passes closed edges, consistent
winding, unit normals, normalized weights and production bounds across 122
Idle/Run samples. Actual four-angle Idle and opposing Run .25/.75 captures
were reviewed for shoulder fit, folds and arm/leg clearance. Koga's baseline
Idle capture is pixel-identical. The shared GPU costume/independent pose
control passes with Erika; reviewer build and diff check pass.

The installed Lance atlas/source/hash and builtin prompts above remain
unchanged. There are still 27 individual painted atlases. The cape follows
torso skinning without cloth simulation. Boot shafts and simplified hand/neck
anatomy remain unfinished. Owned verification scratch was removed after all
processes ended.

## Rival — 2026-10-11

Builtin imagegen precise-object-edit, using the actual exported fallback atlas
as sole target, inspected before generation. Installed
`assets/open-models/people/npc-textures/rival.png` from
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-5b404f7d-3f1b-408e-908a-aa6166fea59d.png`.
SHA-256: `a5dfc4820ba2e7b48541d0655eb28c92c17f57771484fd01a0ae9025484bbeed`.

Prompt:
```text
Use case: precise-object-edit.
Asset type: production character UV color atlas for a stylized 3D game.
Image 1 is the sole edit target: actual exported Rival fallback UV atlas. Repaint artwork while preserving every normalized UV island, skin boundary, original hairline silhouette, ear placement and eye/nose/mouth coordinates. Square atlas only, no rendered figure, labels, text or watermark.
Face: a youthful male rival, sharp but natural hazel-grey eyes, focused slightly skeptical expression, strong fine auburn brows, clean skin with subtle warm shading, no beard, no painted cheek contour lines. Keep eyes and all features at the existing exact locations so they wrap correctly.
Hair: rich dark crimson/auburn layered hair strands within existing upper-left hair area, softly varied locks, gentle highlights, no changes to the skin hairline shape.
Clothing: tailored charcoal navy jacket with restrained red piping along the center opening, dark standing collar and small silver zipper. Shirt island is u 0.. .625, v .475..1; skin above v .475 stays skin. Entire outer shoulder strip at v .475.. .51 must be navy fabric, never grey or skin. Central collar stays within u .22.. .405 v .48.. .55. Zipper at u .3125 v .58.. .88. Plain charcoal navy fabric at u .04.. .11 v .60.. .96 for sleeves and rear, no trim or zipper there. All fabric edges and gutters navy, no white waistband. Lower-right trouser island stays deep charcoal navy with subtle seams.
Footwear: upper-right two shoe islands become charcoal leather with restrained red accents, subtle laces and grey soles; keep exact silhouettes and surrounding gutters. All skin and hand island positions stay exact. Painterly polished game texture with clean details readable at small scale, subtle ambient shading, no baked cast shadows. No new islands, accessories, symbols or text.
```

Distinct youthful hazel-grey face, layered crimson hair, charcoal navy jacket
with red piping/zipper and dark leather shoes. Native torso mapping uses .90
front width, .595 right clamp, and plain rear patch [.075,.64,.008,.15]. This
removes the baseline pale rear waist stripe and keeps the opening on the front.
Shoes use neutral painted uppers and the established dark upper-heel fade.
The native crown rise drops .56 to .40, sweep -.20 to -.18, with .50 cosine
clumps replacing pointed .42 support. Other profiles are unchanged.

Actual four-angle production Idle and Run .25 captures were hand-reviewed.
The face, hair strands, jacket opening, rear fabric and shoes read cleanly in
these studio views. Three supplied UV regressions pass, including Rival's
exact geometry/skin/weight preservation, and the all-cast crown regression
passes 29 profiles with eight Run samples. Reviewer build and diff check pass.
No bitmap postprocessing. Owned verification scratch removed after processes
finished. This is the twenty-eighth individual atlas. Simplified hands, neck,
plain rear cloth and in-game scale review remain unfinished; full cast active.

## Blue — 2026-10-11

Builtin imagegen precise-object-edit using the inspected actual fallback atlas
as sole target. Installed `assets/open-models/people/npc-textures/blue.png` from
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-0323b295-c828-4025-a741-cd77c1f68b66.png`.
SHA-256: `163a4cf9a650f04920eb0bce0ced0a4039fba94a871c655f62a8fb604c98deb7`.

Prompt:
```text
Use case: precise-object-edit.
Asset type: production stylized 3D character UV color atlas.
Image 1 is the sole edit target: actual exported Blue fallback atlas. Repaint the artwork preserving every normalized UV island, skin boundary, original hairline silhouette, ear locations and eye/nose/mouth positions. Square atlas only, no figure render, no labels, text or watermark.
Face: youthful confident male trainer, warm brown eyes, distinct lifted auburn brows, restrained confident half-smile, clean naturally shaded skin, no beard or painted contour lines. Keep feature positions exact to fit the existing mesh. Hair upper-left island: rich chestnut brown layered swept locks, subtle golden-brown highlights, polished painterly strands within the exact original hairline, no new silhouette or islands.
Clothing: plum purple fitted long-sleeve shirt with subtle woven texture, a simple dark purple crew neckline and small antique-gold pendant on a fine chain. Shirt island u 0.. .625 v .475..1; all skin above v .475 stays skin. Outer shoulders at v .475.. .52 must remain purple fabric, no skin or grey strip. Neckline centered within u .23.. .395 v .48.. .53. Fine gold chain shallow U shape u .25.. .375 v .53.. .64 with a tiny pendant centered u .3125 v .66. Pendant simple single oval, no letters. Plain purple fabric at u .04.. .11 v .60.. .96 for sleeves and rear, no necklace there. Entire shirt gutter stays purple, no white waistband.
Lower-right pants island deep navy denim with understated seams. Two upper-right shoe islands charcoal leather with subtle navy accents, laces and grey soles, preserve exact island silhouettes and gutters. Skin and hand islands stay exact.
Polished painterly game texture, subtle ambient shading, clean details readable at small scale. No hard cast shadows, new accessories, extra islands, symbols or text.
```

Blue now has warm brown eyes, a confident youthful face, layered chestnut hair,
a purple woven shirt with gold pendant, navy denim and charcoal shoes. Torso
front mapping uses .90 width and .595 right clamp; the rear plain patch
[.075,.64,.008,.15] removes the pale waist strip and prevents rear necklace
repetition. Painted shoes use neutral uppers and the established dark upper
heel fade. Crown rise .64 to .42 and .50 cosine clumps round the long conical
peaks while retaining the forward sweep and original hairline. No bitmap
postprocessing. Actual four-angle Idle and Run .25 renders were inspected for
face, pendant placement, hair outline, rear fabric and shoe readability.

Reviewer build passed after widening an existing cave_finish::append helper
from parent-only visibility to crate::mesh scope for its existing sibling
callers. This changes visibility only. This is the twenty-ninth individual
painted atlas. Simplified hand/neck anatomy, plain rear fabric and game-scale
review remain unfinished; the full cast objective remains active.

Verification: three supplied UV regressions pass, including Blue geometry,
skin and weight preservation. The all-cast crown regression passes 29 distinct
profiles with eight Run samples. Build and diff check pass. Owned scratch was
removed after all generation/build/test/capture processes finished.

## Brock — 2026-10-11

Builtin imagegen precise-object-edit with the actual exported fallback atlas
as sole target, inspected before generation. Initial output
`exec-247df7dc-3125-4918-b8bb-05a0fb7f43b2.png` had a thin grey shirt border
and was repaired before installation. Installed
`assets/open-models/people/npc-textures/brock.png` from
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-88b05308-a13f-424d-97a2-4baee911d347.png`.
SHA-256: `4fdec7b7a0381810daca54c2970aec7d8eafe07d210f130c838e2539815d5b8b`.

Generation prompt:
```text
Use case: precise-object-edit.
Asset type: production stylized 3D character UV color atlas.
Image 1 is the sole edit target: actual exported Brock fallback atlas. Preserve every normalized UV island, skin boundary, original hairline silhouette, ears and eye/nose/mouth coordinates while repainting. Square atlas only, no rendered figure, labels, text or watermark.
Face: youthful sturdy male trainer, warm medium tan skin, distinctive gently closed narrow eyelids at the original exact eye locations, calm friendly expression, strong dark brown eyebrows. Closed eyes are natural eyelid curves, not sunglasses or black bars. No iris visible. No beard, no cheek contour lines. Keep feature positions exact to fit the mesh. Hair upper-left island: deep warm brown layered short spiky locks with restrained chestnut highlights, natural painterly strands, exact original hairline.
Clothing: open olive-green hiking vest over a warm burnt-orange crew-neck short-sleeve shirt. Shirt island is u 0.. .625 v .475..1; skin above v .475 remains skin. Shoulder gutters all olive fabric, no gray or skin band. Orange undershirt front centered u .22.. .405 v .48.. .96, neckline at v .49. Olive vest side panels u 0.. .21 and .415.. .625, subtle canvas weave, a small flap pocket on each side of the front at u .18 and .445 v .70, understated brown seams. No buttons or zipper down the orange center. Plain olive fabric u .04.. .11 v .60.. .96 for rear, no pockets there. Plain orange fabric u .29.. .335 v .70.. .80 for short sleeve sampling, no symbols. No white waistband or light shirt-edge gutter.
Lower-right pants island warm dark brown canvas, subtle seams and folds. Two upper-right footwear islands brown hiking leather with subtle laces and dark grey soles, preserve exact silhouettes and gutters. All skin/hand island positions exact, warm tan consistent with face.
Polished painterly game texture, subtle ambient shading, details clear at small scale, no hard cast shadows. No new UV islands, accessories, labels, symbols or text.
```

Border repair prompt:
```text
Use case: precise-object-edit. Image 1 is the sole target, the generated Brock UV atlas. One technical cleanup only: replace the thin light grey border between skin and shirt at v .475.. .481 across u 0.. .625 with adjacent clothing pixels. Olive across the vest panels u 0.. .215 and .405.. .625; burnt orange across center u .22.. .395. Also replace the thin vertical grey gutter at u .612.. .625 v .481.. .744 with neighboring olive cloth. This is clothing, must have no remaining grey or skin band. Do not move boundaries. Preserve ALL skin above v .475, face with closed eyes, brown hair, shoe islands and grey shoe surrounds u>=.625, hands, brown pants starting v .744, orange shirt, vest seams/pockets and all other pixels unchanged. Same square atlas exact layout and resolution. No redesign, new detail, labels or text.
```

Brock has an individual narrow-eyed friendly face, layered brown hair and
painted olive vest over an orange shirt, brown canvas trousers and hiking
shoes. Front torso width .90/right clamp .595; plain rear patch
[.075,.64,.008,.15] avoids repeated front pockets and removes the pale waist
strip. Short-sleeve UVs use the existing complete source arm-ring split, orange
shirt for upper arms and exposed skin for forearms. Initial forearm skin was
too pale against the hands; the final Brock-only forearm patch uses
u=.87+.06*z, v=clamp(.69+.08*(abs(x)-.8),.67,.733), below finger artwork and
above the trousers. No geometry or joint weights change in this mapping.
Painted shoe uppers remain neutral with the established dark upper-heel fade.
Native crown rise .36 to .30 and .50 cosine clumps round the old peaks while
keeping his distinct short spiked outline and original hairline. Existing
native hiking pack/straps are retained. No bitmap postprocessing.

Final four-angle production Idle and Run .25 captures were hand-reviewed after
the forearm correction. The face, vest/pockets, orange sleeve rings, warmer
forearms, rear fabric and brown shoes read cleanly. Palms remain lighter and
the source wrist anatomy remains simplified. Three supplied UV regressions
pass, now including Brock torso and sleeve/skin/weight preservation and skin
patch bounds. The all-cast crown regression passes 29 distinct profiles with
eight Run samples. Reviewer build and diff check pass. All owned processes
finished before verification scratch cleanup.

This is the thirtieth individual painted atlas. The vest is painted on the
source body rather than a separately layered garment. Simplified hands/neck,
plain rear fabric and game-scale review remain unfinished; full cast active.

## Elm — 2026-10-11

Builtin imagegen precise-object-edit using the actual exported fallback atlas,
inspected before generation. Initial output
`exec-a7286ddb-fe76-49c2-a946-51143d60f61b.png` was rejected for its grey
moustache-like upper-lip shadow. Installed repaired
`assets/open-models/people/npc-textures/elm.png` from
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-72c47bcf-b5a1-4d97-a23c-ff5229ce1a0b.png`.
SHA-256: `d4e1cbc922a9ef6b80a3d16f89b54d1bbf5e74a1c0963a64bddcdc47e5462843`.

Generation prompt:
```text
Use case: precise-object-edit.
Asset type: production stylized 3D character UV color atlas.
Image 1 is the sole edit target: the actual exported Elm fallback UV atlas. Repaint artwork without moving any normalized island, skin boundary, original hairline silhouette, ear or eye/nose/mouth coordinates. Square atlas only, no rendered figure, text, labels or watermark.
Face: adult male professor in his thirties, gentle thoughtful expression, warm brown eyes with small natural highlights, distinct soft dark-brown brows, subtly rounded features, faint under-eye creases and a restrained friendly smile. Clean-shaven, no cheek contour lines. Exact facial feature positions preserved. Do not paint glasses into the face: fitted 3D glasses already exist. Hair upper-left island: warm brown short tousled layered locks with restrained chestnut highlights, preserve exact hairline.
Clothing: clean warm-white lab coat with subtle cotton weave, lapels and a pale cream shirt with narrow muted teal tie. Shirt island u 0.. .625 v .475..1; all skin above v .475 stays skin. Outer shoulder fabric at v .475.. .51 must stay warm white, no grey border. Lapels front around u .20.. .425 v .49.. .68. Teal tie centered u .3125 v .53.. .79, narrow and readable. Small pen in a tiny chest pocket u .44 v .64.. .73, no badge or text. Plain warm-white cotton at u .04.. .11 v .60.. .96 for sleeve/rear/lower-coat color matching, no pockets or contrasting trim in that patch. Front coat edges and entire gutter all white fabric, no grey skin seam or white waistband. Keep collar artwork on front only, no separate new islands.
Lower-right pants island warm medium-dark brown canvas with understated seams. Two upper-right shoes charcoal leather with restrained brown accents, subtle laces and grey soles, preserve island shapes and grey surrounds. Preserve all skin and hand island positions and natural warm skin tone.
Polished painterly game texture, subtle ambient shading, clear small-scale details, no baked cast shadows. No added accessories, symbols, writing or new islands.
```

Upper-lip cleanup prompt:
```text
Use case: precise-object-edit. Image 1 is sole target, generated Elm UV atlas. One cleanup only: remove the grey moustache-like shadow directly above the upper lip. Elm is clean-shaven. Blend that narrow upper-lip skin area to the adjacent natural warm skin tone with gentle normal shading, no grey beard, stubble, moustache, contour line or dark patch. Preserve lips, mouth expression, nose shape, eye coordinates and all facial feature positions exactly. Preserve every other pixel/UV island: hair, eyes/brows/ears, all surrounding skin, shoes, hands, lab coat/lapels/tie/pen and trousers. Same exact square layout and resolution. No redesign, extra detail or text.
```

Elm now has a distinct clean-shaven adult face with warm brown eyes, tousled
brown hair, lab coat lapels, teal tie and chest pen. Native fitted glasses
remain separate. Front torso width .90/right clamp .595, plain rear patch
[.075,.64,.008,.15]. A first production render showed a pen repeated on an
upper sleeve; the final Elm-only whole-face arm mapping samples plain fabric
u=.075+.04*z, v=.68+.10*(abs(x)-.4). The body positions/normals/colors and skin
influences remain identical. The lower coat now samples the same painting's
plain fabric (.075,.75), rather than the old palette swatch. Shoes use neutral
painted uppers and the established dark upper-heel fade. Crown rise .30 to
.25 and .50 cosine support round the peaks while preserving the original
hairline and mild rear sweep. No bitmap postprocessing.

Final actual four-angle production Idle and Run .25 captures were reviewed
after the sleeve fix. The upper-lip grey patch and repeated sleeve pen are
gone; lapels/tie/pen stay on the front. The existing lower-coat waist join is
still visibly uneven at Idle and pulls away near the front waist in Run .25.
Matching its base color does not resolve that geometry/weight issue.

Four supplied UV regressions pass, including the new Elm plain sleeve test
with exact geometry/normals/skin influence preservation and a Blue no-op
control. The all-cast crown regression and 29-profile costume animation
regression pass (six targeted tests total). The latter verifies bind shape,
weights, motion and bounds; it does not prove coat/body clearance. Reviewer
build and scoped diff check pass. A global diff check found unrelated trailing
blank-line whitespace in tools/sculpt_geometry.py; it was left untouched.
Owned scratch removed after all owned processes ended. This is the thirty-first
individual atlas. Coat waist join/overlap, simplified hands/neck, plain lower
cloth and game-scale review remain unfinished. Full 75-role objective active.

### Elm native shading/rim follow-up — 2026-10-11

No raster change or generation. Elm's fitted coat normals are rebuilt from
its actual triangles, improving the dark waist shading in motion. A new mesh
closure test exposed reversed profile-shell rim orientation; top/bottom/open
front cap winding is corrected for all 29 garment profiles. Bind positions,
UVs, colors and joint weights are unchanged. Padding and higher-join
experiments were rejected and removed.

Three targeted mesh/profile/29-garment animation tests pass. All 29 garments
were reviewed at four angles in Idle and Run .25, plus Elm in opposite Run
.75. The front waist transition is still uneven; neither color matching nor
these normal/rim fixes prove it repaired. Build and scoped diff check pass.
Owned scratch removed after all processes finished. The Elm atlas/hash and
prompts above are unchanged, and the count remains 31. Full cast unfinished.

## Oak — 2026-10-11

Builtin imagegen edit of the actual exported fallback atlas, inspected before
generation. Selected source:
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-fa574d5f-c7ee-4040-b762-19c62f8b4eca.png`.
Installed at `assets/open-models/people/npc-textures/oak.png` without bitmap
postprocessing. This pass follows the user's clarification that maps means
character UV textures.

Generation prompt:
```text
Use case: precise-object-edit.
Asset type: production stylized 3D character UV color atlas.
Image 1 is the sole edit target: actual exported Oak fallback atlas. Repaint artwork while preserving all normalized UV island positions, original hairline silhouette, ears, eye/nose/mouth coordinates and skin boundaries exactly. Square atlas only, no rendered figure, labels, text or watermark.
Face: older male professor around sixty, kind alert expression, warm brown eyes with small highlights, thick silver grey eyebrows, subtle age creases at eye corners and forehead, gentle smile. Clean-shaven face: remove generic brown beard and stubble entirely; warm natural skin with restrained painterly shading, no dark grey upper-lip patch. Keep every facial feature coordinate exact. Upper-left hair island: silver grey swept short layered hair with restrained pale highlights, keep existing hairline.
Clothing island u 0.. .625 v .475..1: warm cream lab coat with subtle cotton weave, front lapels framing a muted brick-red collared shirt. Lapels front around u .20.. .425 v .49.. .76. Red shirt centered u .3125 v .54.. .93, broad enough to read at game scale, small collar and simple buttons. Small pen in chest pocket near u .44 v .64.. .73. No tie, badge or text. Outer shoulder fabric v .475.. .51 must remain warm cream with no grey border. Plain cream fabric across u .04.. .11 v .60.. .96 for rear/sleeve/lower-coat matching; no lapel, pocket, stripe or detail in this patch. Front edges and all gutters cream fabric, no waistband line. All skin above v .475 stays skin.
Lower-right trousers island tan canvas with subtle seam and folds. Two upper-right shoe islands dark brown leather with restrained laces and grey soles, preserve existing shapes and grey surroundings. Preserve hands/skin island positions with natural warm skin tone consistent with the face.
Polished painterly game texture, readable restrained details, subtle ambient shading, no baked cast shadows. No added UV islands, accessories, symbols or writing.
```

Front torso width .90/right clamp .595; rear plain fabric patch
[.075,.64,.008,.15]. Plain-fabric whole-face lab-coat sleeve mapping now covers
Oak and Elm; positions, normals and joint influences remain exact. The lower
coat samples the same atlas at (.075,.75). Painted shoe uppers stay neutral
with the established dark upper-heel fade. No new hair or garment shape.
SHA-256: `dfe1ad241e2b57f2c1674cdb57656b652c6b1b98efcced9e6346ebcd9191e9c7`.
Actual four-angle production Idle and Run .25 renders were hand-reviewed.
The generic beard is gone; age creases, silver brows/hair and brick-red shirt
read clearly. Chest pen remains on the front, sleeves and rear use cream
fabric, and shoes carry their brown leather detail. The existing separate
coat waist transition is still uneven, especially in Run, and hands/neck
remain simplified. Reviewer build and all four supplied UV regressions pass,
including Oak's exact body/skin/weight preservation and plain-fabric sleeves,
with Elm and Blue retained as mapped/no-op controls. Scoped diff check passes.
All owned processes completed before scratch cleanup.
This is the thirty-second individual painted atlas. Full cast unfinished.

## Scientist — 2026-10-11

Builtin imagegen edit of the actual exported fallback atlas, inspected before
generation. Initial `exec-92ebdca0-de22-46fc-9d19-190c1e5b4acc.png` was rejected
for blue shoulder/right-gutter borders and a grey hem stripe. Selected repaired
source `/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-b9003f44-1e45-4e5f-b926-a1a9ec748375.png`.
Installed at `assets/open-models/people/npc-textures/scientist.png`, without
bitmap postprocessing.

Generation prompt:
```text
Use case: precise-object-edit.
Asset type: production stylized 3D character UV color atlas.
Image 1 is the sole edit target: actual exported Scientist fallback atlas. Repaint within the existing normalized islands; preserve exact original hairline silhouette, eye/nose/mouth coordinates, ears, skin boundaries and all island positions. Square atlas only; no figure render, labels, words, symbols or watermark.
Face: male scientist in his early fifties, reserved thoughtful expression, clear grey-green eyes with small natural highlights, slim ash-grey eyebrows, subtle forehead and eye-corner creases, small restrained smile. Distinct slightly leaner facial shading with warm natural skin. Clean-shaven: remove brown beard shadow and stubble entirely, no dark upper-lip patch or dark contour lines. Keep feature positions exact. Do not paint glasses; separate fitted 3D glasses already exist. Upper-left hair island: short neatly combed ash-grey layered hair with a gentle side direction and restrained silver highlights; preserve exact hairline silhouette.
Clothing island u 0.. .625, v .475..1: clean warm-white lab coat, subtle cotton weave, folded front lapels around u .20.. .425 v .49.. .74 over a pale muted powder-blue collared shirt. Shirt centered u .3125 v .54.. .93 with small simple buttons, no tie. Tiny dark-blue pen in small front chest pocket near u .44 v .64.. .73, no badge. Outer shoulders at v .475.. .51 must remain warm white, no grey or skin border. Plain warm-white fabric patch u .04.. .11 v .60.. .96 for rear, sleeves and lower-coat matching; no trim, lapels, pockets or contrasting detail there. White coat front edges and all gutters, no waistband stripe. All skin above v .475 remains skin.
Lower-right trousers island dark warm-brown woven fabric with restrained seams/folds. Two upper-right shoes dark charcoal-brown leather with readable small laces and grey soles; preserve shoe island shapes and grey surrounding gutters. Hands and skin islands keep exact positions and natural warm skin matching the face.
Polished painterly game texture with small-scale readability and subtle ambient shading, no baked cast shadows or exaggerated photo pores. No added accessories, new islands or writing.
```

Border cleanup prompt:
```text
Use case: precise-object-edit. Image 1 is the sole edit target, generated Scientist UV atlas. One garment cleanup: remove the accidental powder-blue border from the lab coat's outer shoulders and right gutter, and remove the pale grey horizontal hem stripe. Replace those border pixels with the adjacent warm-white cotton texture. Specifically, within clothing island u 0.. .625, v .475..1: keep the intended central blue collared shirt u .21.. .40, but make outer shoulder strips u 0.. .20 and u .43.. .625 at v .475.. .51 warm white; make the rightmost vertical gutter u .605.. .625 at all v .475..1 warm white; blend away the horizontal grey band near v .965..1 into continuous warm-white coat fabric. Preserve lapels, pen, pocket, front buttons, the central shirt and all other artwork. Preserve exact UV island layout, original skin boundary at v .475, entire face/eyes/brows/ears/hair/hands, shoes and trousers, same square resolution. No redesign, added details, text or new islands.
```

Front torso width .90/right clamp .595, plain rear patch [.075,.64,.008,.15].
Whole-face plain lab-coat sleeve mapping includes Scientist, preserving exact
positions/normals/colors/joint influences. Lower coat color samples the same
painting at (.075,.75). Neutral painted shoe uppers use the established dark
upper-heel fade. Separate native glasses remain; no native shape change.
SHA-256: `6963b138e90aafab09fce38ac9a7901b69fdca98224b04bbef79ef2d0cc51fed`.
Hand-reviewed actual four-angle production Idle and Run .25 captures. The
generic beard is gone; grey hair, subtle age creases, blue shirt, lapels,
chest pen and dark shoes read clearly. Blue border and grey hem stripe are
gone. Chest pen stays off sleeves; rear remains plain white fabric. Separate
native glasses still fit his face. The source hand/neck anatomy remains
simplified and the separate coat waist join still gaps in Run. This is a UV
improvement, not approval of the garment's geometry.

All four supplied painted-torso regressions pass. Lab-coat sleeve and torso
checks include Scientist and preserve exact positions, normals and skin
influences; Elm/Oak and Blue controls remain covered. Reviewer build and
scoped diff check pass. Owned scratch removed after all owned processes
completed. This is the thirty-third individual atlas; full cast unfinished.

## Blaine — 2026-10-11

Builtin imagegen edit of the actual exported bald-scalp fallback atlas,
inspected before generation. Initial `exec-f8e34fa8-263d-4771-ae83-366a2b2c45d3.png`
was rejected for hard rectangular silver hair blocks and a grey shoulder
stripe. Selected repaired source:
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-526cff43-0e9c-4118-883a-7ee44e56c323.png`.
Installed at `assets/open-models/people/npc-textures/blaine.png`, without bitmap
postprocessing. The existing fallback bald-scalp repaint is bypassed for this
individual atlas by the production painting path.

Generation prompt:
```text
Use case: precise-object-edit.
Asset type: production stylized 3D character UV color atlas.
Image 1 is the sole edit target: actual exported Blaine fallback UV atlas. Repaint within existing normalized islands. Preserve exact eye/nose/mouth positions, ear coordinates and all island boundaries. Square atlas only, no figure render, labels, writing or watermark.
Face: elderly male scientist in his late sixties, shrewd warm expression, alert amber-brown eyes, distinct thick silver-grey eyebrows, soft forehead wrinkles and crow's feet, subtle age creases and a small closed-mouth smile. Natural warm skin with gentle painterly shading. Remove the generic brown beard/stubble shadow entirely. Do not paint moustache or glasses: both already exist as separate fitted 3D geometry; the upper lip must be clean natural skin underneath. Preserve all facial feature coordinates exactly.
Upper-left head/scalp UV area u 0.. .625 v 0.. .475: preserve the existing bald crown region as smooth natural warm skin, matched seamlessly to the forehead/ears. The existing grey side/back hair band must become sparse short silver-white strands with soft feathered edges, avoiding hard horizontal stripes or solid grey blocks. Follow the current scalp/hair placement, no new hairstyle, no hair over the bald crown or face. Thin silver wisps within existing side/back band, subtly blended at its edges. No freckles or spots.
Clothing island u 0.. .625 v .475..1: warm-white lab coat with subtle cotton weave, readable front folded lapels around u .20.. .425 v .49.. .76. Pale ivory shirt, narrow deep burgundy tie centered u .3125 v .53.. .88 with simple knot. No pen, badge or text. Outer shoulder fabric v .475.. .51 stays warm white; no grey/skin border or colored stripe. Plain warm-white fabric u .04.. .11 v .60.. .96 for rear/sleeve/lower-coat matching: no lapels, pockets, tie or colored details in that patch. All front coat edges/gutters warm white; no waist or hem stripe. All skin above v .475 remains skin.
Lower-right trousers island rich muted burgundy woven fabric, subtle seam/folds. Two upper-right shoes dark charcoal leather with restrained laces and grey soles, retain exact island shape and surrounding grey gutters. All skin/hand island positions preserved with natural warm skin matching the face.
Polished painterly game texture, subtle ambient shading and clear small-scale detail, no baked cast shadows, exaggerated pores or added accessories, no new UV islands.
```

Hair/shoulder cleanup prompt:
```text
Use case: precise-object-edit. Image 1 sole target: generated Blaine UV atlas. Targeted cleanup only: the side/back silver hair is still painted as two hard rectangular blocks. Keep the existing bald crown and all facial feature coordinates, but blend every perimeter of those blocks into surrounding natural scalp/ear skin with sparse fine silver wisps over skin, no flat grey background. Upper-left head island u 0.. .625 v 0.. .475. Existing silver side/back band lies approximately u 0.. .23 and u .40.. .625, v .125.. .275. Thin this to sparse short silver strands, with softly feathered transitions over 20-35 pixels at top/bottom/inside corners. Remove all straight horizontal/vertical grey block edges and jagged beige patch artifacts below them; reconstruct natural adjacent warm skin there. Preserve the center face, eyes, brows, nose, lips, ears, wrinkles and bald crown placement; no beard or painted moustache/glasses. Bald crown skin hue should match the forehead naturally without an abrupt orange/peach boundary.
Also remove the accidental grey horizontal garment strip immediately below v .475 on outer coat shoulders (u 0.. .20 and u .43.. .625), blending to adjacent warm-white cotton. Keep the central shirt/tie/collar and lapels unchanged. Preserve all other atlas artwork and normalized islands: trousers, shoes, hands, pen-free coat and central tie. Same square resolution. No text, new islands or redesign.
```

Front torso width .90/right clamp .595; rear plain patch [.075,.64,.008,.15].
Lab-coat plain sleeve mapping includes Blaine and preserves exact body
positions/normals/colors/joint influences. Lower coat color samples (.075,.75)
from the same painting. Shoes use neutral painted uppers and the established
dark upper-heel fade. Separate moustache/glasses remain native. No native
shape change.

SHA-256: `a061c337927a00af634fc9b468c40a2184c833c68bf6b63b58f2eafc76cca64e`.
Actual four-angle production Idle and Run .25 captures were hand-reviewed.
The solid grey band and generic beard shadow are gone; sparse silver hair,
age creases, burgundy tie, lapels, trousers and leather shoes read clearly.
Separate moustache/glasses retain their original fit. Angular crown silhouette,
simplified hand/neck anatomy and separate coat waist gaps remain unfinished.

The first production render exposed burgundy tabs beside the neck. Source
triangle inspection identified collar side faces crossing v=.729.. .752;
they missed the front mapping's whole-face v>=.74 gate and sampled the tie.
A lab-coat-only whole-face mapping now samples plain fabric for faces whose
positions all fall in y=2.40..2.56, abs(x)<.4, original u<.625/v>.60 and any
vertex has v<.74. The mapping is u=.075+.01*x, v=.64+.15*(2.55-y). It applies
to Elm/Oak/Scientist/Blaine and preserves exact positions/normals/colors and
joint influences; neck skin UVs remain untouched. Central front shirt/tie
faces retain the established mapping.

Reviewed final four-angle Idle and Run .25 collar/torso captures for all four
painted coats (32 views). Colored collar tabs are gone, front details stay on
the front, and sleeves/rear remain plain fabric. Existing waist gaps remain.
All five supplied UV regressions pass, including the new plain collar/skin/
weight check with four lab coats and Blue no-op control. Reviewer build and
scoped diff check pass. Owned scratch removed after all owned processes ended.
This is the thirty-fourth individual atlas; full cast unfinished.

### Native bald-crown follow-up — 2026-10-11

No imagegen or raster change. Six hatless bald roles now receive two selective
Loop passes above center.y + .35*half.y; lower face/accessory support and
original UVs/influences stay fixed. Hand-reviewed all six in four-angle Idle
and Run .25 against baseline. Crowns are rounder; Scientist's full Idle
control is pixel-identical. Two supplied head regressions and reviewer build
pass, including 61 Idle/61 Run bounds samples and fixed face/body attributes.
Full details are in character-review.md. Four fallback bald atlases still have
patched/jagged scalp painting and need individual repainting; those artifacts
are more visible after refinement. Atlas count remains 34, full cast unfinished.

## Pharmacist — 2026-10-11

Builtin imagegen edit of the actual exported fallback atlas, inspected before
generation. Initial `exec-339783b1-d3f7-4ba9-a9e6-fdf544fb46f5.png` was rejected
for rectangular side hair/forehead patch outlines and grey garment borders.
Selected repaired source:
`/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-a9dde079-53d4-4bed-b07c-ae4ae364843c.png`.
Installed at `assets/open-models/people/npc-textures/pharmacist.png` without
bitmap postprocessing. The existing individual painting path bypasses the
fallback scalp recoloring; native bald-crown refinement remains active.

Generation prompt:
```text
Use case: precise-object-edit.
Asset type: production stylized 3D character UV color atlas.
Image 1 is the sole edit target: actual exported Pharmacist fallback UV atlas. Repaint within the existing normalized islands. Preserve eye/nose/mouth coordinates, ears, every skin boundary and island position exactly. Square atlas only, no rendered figure, words, labels or watermark.
Face: kindly elderly male pharmacist around seventy, gentle attentive expression, warm dark hazel eyes with small natural highlights, soft silver eyebrows, fine forehead wrinkles and eye-corner creases, subtle full cheeks and a restrained friendly smile. Natural warm skin and gentle painterly shading. Remove brown beard/stubble shadow entirely. Do not paint moustache or glasses: both are already separate fitted 3D geometry. Upper lip and chin underneath stay clean skin. Keep all facial feature coordinates exact.
Bald scalp: upper-left head island u 0.. .625 v 0.. .475. The bald crown must be smooth natural warm skin matched seamlessly to forehead and ears. Existing side/back grey band approximately u 0.. .23 and u .40.. .625, v .125.. .275 must become sparse fine short silver strands OVER NATURAL SKIN, no flat grey background. Soft feathered transitions at all edges, no straight horizontal/vertical grey block edges, jagged beige patches, hair over bald crown or face, scalp spots or square patchwork. Existing scalp/hair placement stays fixed; no new hairstyle.
Clothing island u 0.. .625 v .475..1: clean warm-white long pharmacist robe with subtle cotton weave. Simple folded ivory stand collar centered around u .3125 v .49.. .60; shallow overlapping front closure u .27.. .36 v .57.. .94 with three small pearl buttons. Restrained stitched front seam and soft vertical folds; no lapels, tie, pen, badge, cross, lettering or colored trim. Plain warm-white fabric u .04.. .11 v .60.. .96 for rear/sleeve/lower-robe matching, no details in that patch. Outer shoulder fabric at v .475.. .51, all front edges and every gutter warm white, no grey skin seam, waistband line or hem stripe. All skin above v .475 remains skin.
Lower-right trousers island plain warm-white linen undergarment, subtle seam/folds, no dark waistband. Two upper-right shoes dark charcoal-brown leather with small restrained laces and grey soles, preserve original island shapes and surrounding grey gutters. Preserve all hand/skin island positions and natural warm skin matching face.
Polished painterly game texture, clear small-scale details, subtle ambient shading, no baked cast shadows, exaggerated pores, new accessories, new UV islands or writing.
```

Scalp/garment-border repair prompt:
```text
Use case: precise-object-edit. Image 1 is sole edit target, generated Pharmacist UV atlas. Targeted cleanup only. The upper-left scalp/hair is still split into sharp rectangular blocks with a paper-cut-like outline over the forehead. Remove ALL those straight block edges, their shadows, and the jagged beige ledge below the side hair. The entire bald crown and forehead must read as one continuous natural warm skin surface: no notch outline, cut line, floating patch, rectangle or shadow at their transition. Keep eyes, brows, nose, mouth and ears in exactly the same coordinates and preserve the age creases below the forehead transition. Thin the side/back silver hair (roughly u 0.. .23 and u .40.. .625, v .125.. .275) to sparse short fine silver wisps over adjacent warm scalp skin, with soft feathered boundaries over 25-40 pixels on EVERY perimeter. No solid hair rectangle or grey/brown background; no hair added on the bald crown or face. Preserve the same normalized atlas layout.
Also remove the accidental grey garment frame: at v .475.. .505 across u 0.. .625 except the intended central ivory stand collar u .20.. .40, in outer vertical gutters u 0.. .045 and u .605.. .625, and the pale grey hem stripe near v .97..1. Replace these borders with continuous adjacent warm-white cotton. Remove dark grey/blue outline above the lower-right white trousers island; its waistband should blend naturally to off-white linen. Preserve the collar shape, all front buttons, sewn closure, folds, shoes, hands, face and remaining artwork. No redesign, new details, words, badges, moustache or painted glasses. Same square resolution.
```

Front width .90/right clamp .595, rear plain patch [.075,.64,.008,.15].
The existing white-coat plain sleeve and side-collar mappings include this
white robe, preserving exact positions/normals/colors/joint influences and
neck skin. Lower robe samples matching plain fabric at (.075,.75). Shoes use
neutral painted uppers and the established dark upper-heel fade. Native
glasses/moustache and garment shape remain unchanged.

SHA-256: `855faccecfdba77b91dc910826709f4084de1baad88cb713ab27ec9cdafc0aae`.
Hand-reviewed actual four-angle production Idle and Run .25. The generic
beard and solid grey hair band are gone; softer silver wisps, kind older face,
collar/three buttons and leather shoes read clearly. Front details stay off
sleeves/back and native glasses/moustache retain their fit. Five supplied UV
regressions pass, including Pharmacist torso/skin/weight preservation and
plain sleeve/collar sampling. The lower robe remains plain separate geometry
with an uneven waist join and a stiff hem in running. Hands/neck remain
simplified, and forehead shading still shows patches despite the new painting.

A welded geometric-normal blend across the crown refinement boundary was
rendered on all six bald heads in Idle/Run .25 and rejected: it did not remove
the forehead patches. Its code/test changes were removed. A missing existing
`meadow_noise` import in `mesh/ledge_approach.rs` blocked a rebuild; a one-line
import fix restores that call without changing its behavior. A concurrently
introduced duplicate module qualification was also corrected to that same
imported helper. Final rebuild and both retained head regressions pass after
the normal experiment was removed. Reviewed the retained model in opposite
Run .75 as well. Seven targeted UV/head checks pass in total; scoped diff
check passes. Owned scratch removed after all owned processes completed.
This is the thirty-fifth individual atlas. Full cast unfinished.


## Bald crown boundary follow-up

The selective crown subdivision left T junctions at its boundary. A new welded
edge regression failed on Blaine before repair: a crown edge had one incident
triangle instead of two with opposing winding. `head_surface::bald_crown` now
conforms existing midpoint vertices before subdivision and after each pass,
while retaining separate UV chart copies. New vertices interpolate UV/color
and normalized skin weights; original UVs and skin influences stay exact.
Area-weighted crown normals are rebuilt across welded positions. Lower face
and body positions/normals remain exact.

Both ignored supplied head regressions pass, including all six bald roles,
closed oriented crown edges, valid normals/weights, face preservation, and 61
Idle plus 61 Run animation bounds samples. Reviewer build and scoped diff check
pass. Hand-reviewed all six in four-angle Idle and Run .25 against the baseline.
Scientist Idle remains pixel-identical. Boundary continuity is repaired, but
forehead patches remain visibly present, particularly on the fallback atlases;
this is not a successful visual cleanup claim. Elder/PokefanM/Sage retain hard
hair bands and require individual paintings. No raster asset changed in this
follow-up; 35 individual atlases remain installed. Full cast goal stays active.


## Elder — 2026-10-11

Builtin imagegen, actual fallback atlas edit target exported by the production
reviewer. First output rejected for rectangular scalp skin panels with ochre
edge remnants: exec-73f1a46a-28cc-4663-b764-aa7647624b9f.png.
Selected targeted repair: /Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-94915b0c-32e3-4078-9fff-1401115d5d93.png.
Installed assets/open-models/people/npc-textures/elder.png.
SHA256 c7deab5645d8cc7540dacee436a1b80232e7b967f2469ded024cd33fbf732b74.

Initial prompt:

```text
Use case: style-transfer.
Asset type: production square UV albedo atlas for Elder, an elderly male NPC in a stylized 3D game.
Image 1 sole edit target: actual exported fallback atlas. Preserve EXACT normalized island boundaries and facial feature locations. Output square flat UV texture only.
Head island u0.. .625 v0.. .475: distinctive dignified elderly male face around eighty, thoughtful warm brown eyes, soft silver eyebrows, fine forehead and eye-corner wrinkles, subtle sun-warmed skin, calm closed-mouth expression. Keep eyes/nose/lips/ears coordinates exactly. Bald crown and sparse short silver hair on sides and back. Completely remove the existing flat grey rectangular hair blocks: replace with natural skin carrying softly feathered silver wisps, no hard horizontal or vertical edges, no solid grey backing. Continuous scalp tone throughout top and adjacent forehead, no outlines or patch shadows. Remove painted brown beard and stubble entirely; native fitted silver beard already exists as separate 3D geometry, so cheeks/chin and upper lip underneath must be clean skin.
Clothing island u0.. .625 v.475..1: modest ochre-gold long robe, soft woven cloth, simple shallow overlapping ivory collar centered u.3125 v.49.. .59, restrained central closure and quiet vertical folds. No buttons, belt, lapels, trim, writing or badges. Plain matching ochre cloth u.04.. .11 v.60.. .96 for rear, sleeves and lower robe sampling. All island edges and gutters same ochre fabric, no dark borders, waistband or hem stripe.
Lower-right garment region u.625..1 v.75..1 stays deep muted brown fabric. Shoe islands u.625.. .8125 v.125.. .5 replace white sneakers with plain dark brown soft leather shoes, subtle seams, dark soles, no laces or white patches. Preserve original arm/hand skin island locations and orientation; warm skin without muddy bands.
Clean organized hand-painted game albedo finish, broad soft gradients, restrained wrinkles, even neutral lighting without baked directional shadows, no noisy grain. No model render, portrait, scenery, labels, grid, border, cropping, new islands, painted accessories or watermark. UV alignment highest priority.
```

Targeted repair prompt:

```text
Use case: precise-object-edit. Image 1 sole edit target is the Elder flat square UV atlas. Repair only head island u0.. .625 v0.. .475. The side/back scalp panels still show large flat rectangular peach-brown blocks with hard straight borders and tiny ochre edge remnants, surrounding the center face. Remove ALL panel boundary outlines, straight horizontal/vertical skin-color jumps, ochre pixels and notched paper-cut borders within that head island. Paint continuously across these internal UV areas with the same natural warm scalp skin hue and restrained texture. The crown skin above v.125 and side scalp below v.125 must meet without a straight visible color seam. Retain the sparse feathered silver hairs by ears without solid backing. Preserve EXACT central eyes, brows, nose, mouth, wrinkles, ear positions, head island outer boundary and all feature scale/orientation. Do not add beard, moustache or glasses. Keep all clothing, shoe and arm/hand islands unchanged. Output only square UV atlas, no grid/border/text/scene.
```

Front mapping width .90 with right limit .595; plain rear/sleeve/side-collar
patch keeps front overlap off those surfaces. Lower robe samples ochre fabric
u .075 v .75. Painted shoe uppers retain their neutral palette with dark heel.
Native beard and crown refinement retained. All original body geometry/normals
and skin influences remain unchanged by UV mapping.


Elder visual validation: five supplied UV regressions pass, including exact
geometry/normals/skin-influence preservation and plain ochre sleeve/collar
sampling. Reviewer build and scoped diff check pass. Hand-reviewed actual
production four-angle Idle and Run .25 images against exported baseline.
Hard grey hair band is gone; distinct mature face, feathered silver side wisps,
front ivory collar and ochre robe read clearly. Forehead shading patches remain;
beard is still a flat grey geometric wedge, hands/neck simplified, cane missing,
lower robe plain with uneven waist join and stiff/irregular animated hem.
This approves only the texture improvement, not the finished character. All
owned processes ended before scratch cleanup. 36 individual atlases installed;
PokefanM/Sage still require individual bald paintings. Full 75-role goal active.


## PokefanM — 2026-10-11

Builtin imagegen used actual renderer-exported fallback as sole edit target.
Selected source /Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-24afdf3b-eb43-49fe-9f0d-45453a97b6cc.png.
Installed assets/open-models/people/npc-textures/pokefan_m.png.
SHA256 5793c5bfce7aed5cf9a1d326346ed743ccaafca7501f7c8a00600df037e08707.

Prompt:

```text
Use case: style-transfer. Asset type: production square flat UV albedo atlas for male Pokefan NPC. Image1 sole edit target actual exported fallback atlas. Preserve all normalized island boundaries, and EXACT eye/brow/nose/lip/ear feature coordinates, size and orientation. Upper-left head u0.. .625 v0.. .475: distinct affable middle-aged man around fifty, full cheeks, bright hazel eyes, warm restrained smile, dark chestnut eyebrows, gentle laugh lines. Clean warm skin. No painted moustache, glasses or beard/stubble: separate native moustache and round glasses already exist. Bald crown, very sparse short chestnut hair wisps on sides/back with soft feathered transitions; REMOVE existing flat rectangular dark-brown hair blocks completely and paint natural scalp behind wisps. Continuous skin hue across crown, forehead and side/back chart with NO straight panel borders, no jagged beige patch outline, no dark bands or baked directional shadows.
Lower-left clothing u0.. .625 v.475..1: warm dark chocolate brown short-sleeve polo shirt, neat small folded tan collar at top center u.3125 v.49.. .59, shallow short front placket with two small tan buttons u.31 v.60.. .69. Simple restrained cotton folds, no dense wrinkles or grain. Keep plain solid chocolate cloth u.04.. .11 v.60.. .96 for sleeves/back. All outer shoulder edges and gutters chocolate cloth, no grey frame/waistband/hem line or stripes. No badge, lettering or ball symbol.
Lower-right pants island u.625..1 v.75..1 dark brown trousers with subtle seams, no contrasting waistband. Shoe islands u.625.. .8125 v.125.. .5 soft dark chestnut leather casual shoes with subtle seams and dark soles; no white sneakers or bright laces. Exact original skin arm/hand island locations remain, warm natural peach skin consistent with face.
Clean stylized hand-painted game albedo, broad organized gradients and readable details. Output ONLY flat square atlas, no model render/portrait/scene/grid/text/border/crop/new islands/watermark. UV alignment highest priority.
```

Individual middle-aged face with full cheeks, hazel eyes and short feathered
brown side hair over continuous bald scalp; chocolate polo, tan collar/two
buttons, brown trousers and leather shoes. Existing fitted moustache/glasses
retained. Front mapping .90 width/.595 right, plain rear/sleeve/side-collar
patch. Whole source arm faces split at existing ring: short sleeves on plain
cloth, warm forearms below hand details and above trouser island. UV edits
preserve exact geometry/normals/colors/skin influences. Dark painted shoe
uppers/heel retain their palette. No crown geometry change this repaint.


Five supplied UV regressions pass, including PokefanM exact source geometry,
normals/colors and skin influences, plain collar sides and short-sleeve/skin
sampling. Reviewer build and scoped diff check pass. Hand-reviewed actual
four-angle Idle and Run .25 against baseline: solid brown hair band and white
rear-waist strip are gone; face/collar/buttons/leather footwear are readable.
Forehead shading patches remain, source hands are simplified, wrist color seams
need attention, moustache is a flat surface, body is still thin in side view and
individual ball badge is missing. Sole/heel read grey. This retains the texture
improvement only, not final model approval. All owned processes completed before
scratch cleanup. 37 individual atlases installed. Sage remains the unpainted
hatless bald role. Full 75-role objective stays active.


## Sage — 2026-10-11

Builtin imagegen edited actual exported renderer fallback atlas. First image
exec-a4a860c8-2fa4-49e6-be1d-9b34a5507297.png rejected for straight scalp
panel outlines/skin color borders, notched forehead edge and dark hem band.
Selected repair: /Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-e63873ba-8d4a-4aa3-925c-7214c9b43a11.png.
Installed assets/open-models/people/npc-textures/sage.png.
SHA256 61426bc25a6c0171dcd3aef2b46c8aeb4d3e1b4f70bc556b2294e46bd698a367.

Initial prompt:

```text
Use case: style-transfer. Asset type: production square flat UV albedo atlas for Sage, an older male monk NPC in a stylized 3D game. Image1 sole edit target actual exported fallback atlas. Preserve EXACT normalized island boundaries and original eye/brow/nose/lip/ear coordinates, scale and orientation. Head island u0.. .625 v0.. .475: distinct serene lean male face around sixty-five, thoughtful grey-brown eyes, fine silver eyebrows, gentle age lines and a quiet closed-mouth expression; clean-shaven warm natural skin, no beard, stubble or painted moustache/glasses. Bald scalp with sparse fine silver wisps near ears only. Completely remove the original solid grey side/back hair blocks; natural skin with soft hair wisps, no straight horizontal/vertical panel edges, no abrupt crown/forehead hue transition or jagged patch outlines. Continuous scalp/neck skin with broad soft gradients.
Clothing island u0.. .625 v.475..1: simple warm sand-beige monk robe, small overlapping cream collar centered u.3125 v.49.. .59, narrow understated diagonal closure and vertical fabric folds; no buttons, belt, sash, trim, lapels, badges or writing. Plain sand-beige fabric u.04.. .11 v.60.. .96 for back/sleeves/lower robe. All outer shoulder edges/gutters matching cloth, no grey border/waistband/hem stripe. Do not paint bead necklace: native accessories remain separate 3D details.
Lower-right garment u.625..1 v.75..1 matching sand-beige cloth with restrained folds. Shoe islands u.625.. .8125 v.125.. .5 plain dark taupe-brown cloth slip-on shoes with restrained stitching and dark soles, no white trainers/laces. Exact original arm/hand skin islands remain with warm skin consistent with face.
Clean organized hand-painted game albedo, neutral even lighting without baked directional shadows, readable age details, no dense grain or dirty skin bands. Output ONLY flat square atlas, no model/portrait/scene/text/grid/border/crop/new islands/watermark. UV alignment highest priority.
```

Targeted repair prompt:

```text
Use case: precise-object-edit. Image1 sole edit target Sage flat square UV atlas. Targeted cleanup: in head island u0.. .625 v0.. .475 remove ALL straight borders of rectangular side/back scalp panels, dark outlines, notched paper-cut forehead border, and tiny beige fragments near v.27. Paint natural continuous warm skin across internal crown/side panel boundaries without color jumps or any straight lines. Retain exact eyes/brows/nose/lips/ears, wrinkles, face scale and feathered silver wisps. No beard/stubble/accessories. In clothing island u0.. .625 v.475..1 remove the wide darker horizontal hem band along bottom v.94..1 and all framed borders. Fill with matching sand-beige fabric continuous with above, quiet vertical folds only. Preserve collar and diagonal robe closure exactly. Keep shoes, arm/hand islands and lower-right garment unchanged. Output ONLY square UV atlas no grid/border/text/scene.
```

Front mapping width .90/right .595, plain rear/sleeve/collar-side patch avoids
repeated neckline/diagonal closure. Lower robe matches painting u .075 v .75.
Painted shoes use neutral uppers/dark heel. Original geometry/normals/colors
and skin influences remain exact through UV edits. Previous crown retained.


Five supplied UV regressions pass, including Sage plain sand cloth sampling,
exact geometry/normals/colors and skin-influence preservation. Reviewer build
and scoped diff check pass. Initial test/build attempts failed on disk space;
after external space recovery both were retried successfully without deleting
shared caches. Actual four-angle Idle and Run .25 hand-reviewed against baseline:
hard grey hair band gone, face and overlapping robe collar are readable.
Forehead shading patches persist; hands/neck simplified, lower robe plain with
uneven waist join and animated hem, soles/heels grey and bead necklace missing
(the appearance declares beads but has no geometry implementation). Retain the
texture improvement only, not finished model approval. Owned processes ended
before scratch cleanup. 38 individual atlases installed; all six hatless bald
roles now have individual paintings. Full 75-role objective remains active.


## Bald forehead lighting follow-up

Added opt-in offline `CRYSTAL_PEOPLE_REVIEW_UNLIT=1` albedo-only review.
Production materials remain unchanged outside the reviewer. Actual four-angle
albedo-only renders of Blaine/Elder/Gramps/Pharmacist/PokefanM/Sage removed the
irregular forehead patches while preserving paintings, pointing to normals
rather than UV panels; actors already have NotShadowReceiver. The prior
area-weighted cap normals varied with selective-subdivision triangle areas.
Replaced those normals above the existing cap threshold with a smooth rounded
elliptical cap field derived from supplied source head bounds. Refined positions,
UVs/colors, joints/weights and lower-face/body normals stay exact; topology
repair is retained. This is distinct from the earlier rejected normal blend.

Hand-reviewed all six in four-angle normal-lit Idle and Run .25 against baseline,
plus albedo-only diagnostic: irregular forehead facet shading is visibly reduced
and cap lighting is smooth. Broader painting color transitions and source
contours remain, especially PokefanM's rear scalp and neck. Existing beard,
accessory, garment and hand defects are unchanged and still outstanding.
Both supplied head regressions pass, including an added bound on neighboring
cap normal variation, oriented closed crown edges, unit normals, pinned lower
face, UV/skin-influence preservation and 61 Idle plus 61 Run animation bounds
samples. Reviewer build and scoped diff check pass. No raster assets changed;
38 individual atlases retained. All owned processes ended before scratch cleanup.
Full 75-role objective remains active.


## Sage fitted prayer beads

Added original Rust geometry in beads.rs through the existing chest-attached
accessory slot. 36 rounded wooden beads (20 sides/12 latitude divisions, radius
.022) and a thin closed cord, gentle wood palette variation and smooth normals.
Medial source torso slice closes across the arm junctions with a convex outline;
144 source-fitted path samples resample by arc length to avoid shoulder spikes
and uneven front/back bead spacing. Each final center is fitted with .012 source
units of clearance. No body/texture/skin-influence changes and other roles do not
receive this necklace. The necklace is rigidly attached to Chest; no swing/cloth
simulation is claimed.

First fit regression failed on missing shoulder-root slices. A wider slice
candidate passed bounds/closure but visual review rejected projecting shoulder
beads and crowded rear beads. Revised medial torso outline and arc spacing
remove those defects in actual four-angle Idle, Run .25 and opposite Run .75
renders. Retained regression checks closed oriented bead/cord surfaces, outward
unit normals, round spheres, source outline clearance, bead spacing/extent,
non-Sage exclusion and body bounds through 61 samples over each complete Idle
and Run clip. Scoped regression, reviewer build and diff check pass. Initial
compiles failed from shared disk capacity and an unrelated terrain module error
subsequently corrected in current source; successful final runs disabled Cargo
incremental caching only for those commands. Removed only the specific failed
compiler working directory identified in this task's failure log after own
processes ended; other cache directories preserved. Review scratch/logs removed
once all owned runs completed.

Sage now has the previously missing bead accessory. Hands/neck contours,
robe waist/hem/limited lower cloth details, grey footwear heels and broader
cast defects remain. No raster asset changed; 38 individual atlases retained.
Full 75-role goal remains active.


## Pryce — 2026-10-11

Builtin imagegen edited the actual renderer-exported fallback. First output
exec-3220d99c-46af-4d89-ab81-01060daf86b7.png rejected for solid grey scalp
background, dense knit grain and hem framing. Selected repair:
/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-15de5eef-ca6a-46b8-9535-790d49d6ddf9.png.
Installed assets/open-models/people/npc-textures/pryce.png.
SHA256 74c337433f904ec46c8ffec014cb35bea71959dc1165a04fa480f3c114eaa359.

Initial prompt:

```text
Use case: style-transfer. Asset type: production square flat UV albedo atlas for Pryce, an elderly male trainer NPC in a stylized 3D game. Image1 sole edit target actual renderer-exported fallback atlas. Preserve EXACT normalized island boundaries and original eye/brow/nose/lip/ear feature coordinates, scale and orientation.
Upper-left head u0.. .625 v0.. .475: distinctive lean elderly man around seventy-five, cool grey-blue eyes, firm silver eyebrows, thoughtful serious closed-mouth expression, fine forehead/crow's-feet/cheek age lines, clean-shaven warm natural skin with broad soft gradients. No beard/stubble or painted moustache/glasses. Extremely thinning silver hair: scalp visible across crown, sparse feathered silver wisps at sides/back and a few thin swept silver strands on top. Remove original dense flat gray hair blocks and their sharp notched forehead border. No solid grey cap, rectangular panels, straight horizontal/vertical borders, muddy neck band, color discontinuity between scalp and forehead. Preserve all facial feature coordinates exactly.
Lower-left clothing u0.. .625 v.475..1: ice-blue knitted cardigan with a modest narrow folded blue-grey collar at top center u.3125 v.49.. .59, small ivory undershirt insert and three small muted navy buttons on central front u.31 v.60.. .80. Subtle knit texture, understated seams and a few vertical folds; no dense grain, heavy directional shadows, writing, badges, scarf, lapels, belt or border. Plain matching ice-blue fabric u.04.. .11 v.60.. .96 for sleeves/back. All shoulder/gutter/hem edges match the cloth with no grey frame or dark band.
Lower-right garment u.625..1 v.75..1 muted plum trousers with simple seams and soft folds, no bright waistband. Shoe islands u.625.. .8125 v.125.. .5 dark muted blue leather shoes with restrained stitching and dark soles, no white sneakers/bright laces. Exact original skin arm/hand island positions intact; consistent warm natural skin.
Clean organized hand-painted game albedo, neutral even lighting, readable mature features. Output ONLY flat square atlas, no model/portrait/scene/text/grid/border/crop/new islands/watermark. UV alignment highest priority.
```

Targeted repair prompt:

```text
Use case: precise-object-edit. Image1 sole target Pryce flat square UV atlas. Targeted cleanup:
Head island u0.. .625 v0.. .475: remove ALL solid grey background/cap from side and back scalp, including the entire outer left/right top regions. Replace the grey with continuous warm natural scalp skin matching the central forehead, with only sparse feathered silver strands over skin. No solid grey backing, straight panel borders, paper-cut hairline, abrupt hue jumps or muddy neck shadows. Retain a few thin silver swept strands above forehead. Preserve exact eyes, brows, nose, lips, ears, wrinkles and face scale/orientation; no stubble or accessories.
Clothing u0.. .625 v.475..1: drastically reduce dense repeated knit grain into a subtle low-contrast soft cotton-knit suggestion. Keep ice-blue cardigan, blue-grey collar, ivory insert, central buttons/placket and broad folds exactly aligned. Remove darker horizontal lower hem band and framed side borders; plain blue fabric continuous to every outer edge. Plain patch u.04.. .11 v.60.. .96 must remain without buttons/collar/stripes.
Keep shoe islands, trousers and arm/hand skin coordinates unchanged. Output ONLY square UV atlas, no model/scene/text/grid/border/new islands/crop.
```

Distinct older grey-blue-eyed face, silver thinning wisps over visible scalp,
ice-blue cardigan with collar/ivory insert/buttons, plum trousers and dark blue
leather shoes. Front width .90/right .595, plain rear/sleeve/collar-side patch
prevents repeated details. UV mapping preserves original geometry/normals/colors
and skin influences. Neutral painted shoe uppers/dark heel. Exposed-crown
refinement now includes hatless sparse-haired Pryce, Kurt and KurtOutside;
existing protected lower face, UV/skin-influence preservation and bounds rules
remain. Kurt paintings are still fallback and require individual replacement.


Validation: five supplied painted-torso regressions and two supplied head
regressions passed; reviewer build passed. Hand-reviewed Pryce in all four
angles in Idle and Run .25 against his actual fallback baseline. The elderly
face and thinning silver wisps replace the generic young face and solid grey
cap, cardigan collar/buttons remain on the front, rear white waistband is gone,
and painted dark shoe uppers are readable. Knit grain is still visible at close
range but substantially reduced from the rejected image. Crown lighting is
smoother; faceted rear lower-head/neck contours and scalp color variation remain.
Hands, narrow neck, grey heel/sole regions and missing declared cane/scarf remain
unfinished. No accessory was added in this pass.

Compared Kurt and KurtOutside separately in four-angle head/upper-body crops
before and after refinement in Idle, plus Run .25. Rounded cap silhouettes and
more even cap shading improve; dense grey fallback hair, generic face and
notched hairline still require repainting. Both remain fallback atlases. The
expanded crown regression checks all nine eligible bald/sparse roles, Scientist
exclusion, oriented closed cap edges, bounded neighboring cap normal variation,
unit normals, pinned lower face/body and original UV/color/skin influences,
plus 61 samples across each complete Idle and Run clip. Scoped diff check passed.
All own render processes ended before review scratch/log cleanup.
39 individual painted atlases installed. The full 75-role objective remains active.


## Kurt and KurtOutside — 2026-10-11

Builtin imagegen edited the actual renderer-exported Kurt fallback. First output
exec-4e74671c-fd9d-463a-9750-78a35e2d0077.png rejected for heavy shoulder/outer
fabric seams and straight side scalp transition. Selected repair:
/Users/ryanculligan/.codex/generated_images/01a1277b-19f5-79f1-b0a2-a2dc008920ef/exec-bbef96fc-c45c-4215-8d01-8908a1ae8309.png.
Installed matching kurt.png and kurt_outside.png, preserving one artisan's
identity across indoor and outdoor roles. Both SHA256
 a4f80cac913ebb7544c1672f134d3a3551b68620a2bf15a7b73eaa20a7301717.
This is one new individual painting applied to two roles: 40 distinct repaints,
41 role atlas files. No independently generated outdoor face is claimed.

Initial prompt:

```text
Use case: style-transfer. Asset type: production square flat UV albedo atlas for Kurt, an elderly artisan NPC in a stylized 3D game. Image1 sole edit target actual renderer-exported fallback atlas. Preserve EXACT normalized island boundaries and original eye/brow/nose/lip/ear feature coordinates, scale and orientation. Upper-left head u0.. .625 v0.. .475: distinctive sturdy elderly male artisan around seventy, broad brow, warm brown eyes, silver eyebrows, firm rounded nose, weathered cheeks, gentle crow's-feet, thoughtful slightly stern closed-mouth expression, clean-shaven warm natural skin with broad soft gradients. No painted beard, moustache or glasses. Very sparse silver wisps around temples/ears/back and a few fine swept strands on top over fully visible natural scalp. REMOVE all original solid grey hair blocks. Continuous skin hue from forehead through scalp and side/back, no straight panel borders, jagged hairline, muddy neck band, dark baked shadows or solid grey cap. Preserve facial feature coordinates. Lower-left garment u0.. .625 v.475..1: indigo-blue traditional work robe with a restrained crossed neckline at top center u.3125 v.49.. .59, narrow ivory inner collar, gently overlapping indigo lapel toward upper chest u.30 v.60.. .70, no buttons. Simple seams and broad low-contrast folds, subtly worn soft woven cloth. Keep plain uninterrupted matching indigo cloth u.04.. .11 v.60.. .96 for sleeves/back. All shoulder/gutter/hem edges match plain cloth with no frame, waistband, stripe, dark border or grey band. No apron, belt, pocket, text or badges in atlas; separate accessories may be modeled later. Lower-right garment u.625..1 v.75..1 matching indigo trousers with restrained folds, no bright waist stripe. Shoe islands u.625.. .8125 v.125.. .5 dark warm brown leather slip-on work shoes with understated stitching and dark soles, no white sneakers/laces. Preserve hand/arm skin islands exactly at original positions, consistent natural warm skin. Clean hand-painted neutral evenly lit game albedo. Output ONLY flat square UV atlas, no model/portrait/scene/grid/text/border/new islands/crop/watermark. UV alignment highest priority.
```

Targeted repair prompt:

```text
Use case: precise-object-edit. Image1 sole target Kurt flat square UV atlas. Change ONLY cleanup details: clothing island u0.. .625 v.475..1 remove the entire heavy vertical seam/frame bands at left and right outer edges, and heavy horizontal upper shoulder seam. Replace them with continuous plain indigo cloth matching the central fabric to every outer edge. Keep crossed indigo robe collar and narrow ivory inner collar at exact coordinates unchanged. Keep plain fabric patch u.04.. .11 v.60.. .96 entirely free of seams/lapels/bands/details. Reduce dense repeated woven grain into subtle low-contrast broad cloth texture; preserve soft broad folds. Head u0.. .625 v0.. .475 remove the straight horizontal scalp/neck transition near ear height on left and right outer regions and muddy neck shadows, blend natural skin continuously without panel borders. Preserve exact eyes/brows/nose/lips/ears/wrinkles/face shape and silver wisps, no new hair cap. Keep all shoe/trouser/arm/hand islands, boundaries, orientation, coordinates unchanged. Output ONLY square atlas, no portrait/model/scene/new islands/grid/text/border/crop.
```


Retained wide front .90/right .595, plain rear patch [.075,.64,.008,.15],
plain sleeve and collar-side routing for both Kurt roles. Same indigo-specific
pixel assertions now cover both roles, while exact geometry/normals/colors/
original skin-influence checks remain. Lower robe color samples the selected
plain fabric patch .075/.75 instead of the old palette. Shoe upper tint is
neutral, with dark upper heels; grey heel/sole remnants remain visible.

Hand-reviewed both roles separately in four-angle actual production-mesh Idle
and Run .25 renders (16 views) against fallback baseline. Elderly face, sparse
wisps replacing solid grey cap, crossed neckline/ivory insert and brown leather
uppers improve identity/readability. Rear/sleeves show plain fabric without
repeated collars. Lower robe remains plain and visibly different at the waist,
with existing join/hem limitations. Narrow neck/rear lower-head contours,
simplified hands, wrist skin variation, grey heels and absent declared apron/
outdoor apricorn basket remain. Neither accessory was implemented in this pass.

Five supplied painted-torso regressions pass; 29-roster costume bind-shape,
weight/animation-bounds regression passes; reviewer build and scoped diff check
pass. First UV test compile failed on an unrelated new terrain regression's UV
argument (tuple passed instead of four corners). Corrected only that test call
by expanding its GridGeometry rectangle to four corner UVs; its named regression
then passes. No terrain production change was made by this pass. All owned
processes ended before review scratch and logs were removed.
40 distinct repaints / 41 role atlas files installed. Full 75-role goal remains active.


## Kurt fitted waist apron, pocket and belt

Added original Rust geometry in apron.rs through costume_mesh for Kurt and
KurtOutside only. Warm canvas waist apron with shallow broad folds, rounded
lower outline and raised work-pocket detail, plus a narrow belt around the
whole waist. Each panel is a closed thin shell on a 24-column/32-row grid;
the closed belt has 96 sides and two vertical sections. Total 3,876 added
vertices across apron/pocket/belt. This is colored native geometry; existing
40 distinct repaints / 41 role atlas files are unchanged. Back ties/knot and
KurtOutside's apricorn basket remain absent; other declared apron roles have
not received this targeted accessory implementation.

Front-ray projection fits apron to existing robe and pocket to apron. Radial
horizontal triangle sections fit belt to the full robe circumference. The
outermost supplied cloth triangle supplies barycentric joint weights, combined
and normalized to four skin influences. Thin layers are .008 source units;
base front gap .014, belt .018. The lowest apron region smoothly adds up to
.014 clearance, with the original waist/pocket fit retained. Original robe/body
positions and atlas are preserved. These accessories follow the authored rig;
no cloth simulation, free swinging, or usable pocket storage is claimed.

First compile caught omitted cutaway metadata in the test fixture; corrected
with defaults. Initial closure/unit-normal/interpolated-point bounds regression
passed, but a strengthened check against actual deformed source triangles found
insufficient normal clearance at the inner hem near raised-leg Run phase.
Complete-cycle diagnostic minimum .0025636577 at Run step 21 of 60, apron
vertex 1613, maximum separation .026063377. This remained outside the robe but
failed the retained .004 normal-clearance margin. Added only the local hem
clearance fade instead of lifting all cloth or relaxing the assertions.

Final apron regression passes for both Kurt roles: oriented edge closure,
nondegenerate faces, consistent winding/normals, unit normals/normalized skin
weights, exact appended apron geometry, non-Kurt exclusion, 61 samples over
complete Idle and Run clips, signed clearance above each ACTUALLY deformed
source triangle greater than .004 and separation below .045 for all three
parts, plus render bounds. Existing full 29-roster costume bind/animation
regression also passes (run from the newly built test executable). Reviewer
build and scoped diff check pass. One reviewer compile was stopped by an
unrelated terrain debug trace formatting an unsupported GridGeometry Debug;
the trace had already been corrected in current source when inspected, so
this pass did not edit terrain. Successful build then verified current source.

Hand-reviewed both roles separately in four-angle Idle, Run .25 and opposite
Run .75 production-mesh renders against the no-apron baseline: 24 final views.
Apron follows the curved robe in both leg poses, pocket remains attached,
belt wraps sides/back, and no obvious apron intersections/floating gaps appear
in those views. Waist/hem limitations of the underlying plain robe, hands,
neck/rear lower-head contours, grey footwear heels, back ties/knot and missing
outdoor basket remain unfinished. All owned processes ended before scratch
renders/logs were removed. Full 75-role objective remains active.

## Connected fingertip contour pass

Added native Rust `hand_digits::sculpt` after the existing hand smoothing.
The supplied rigs have one index branch and one thumb branch per hand, rather
than four independent finger chains. This pass shapes four rounded, connected
fingertip lobes into that existing index slab. It does not claim independent
finger articulation or finished hand anatomy.

Three conforming refinement passes (.19, .19, .35 hand-local selection cuts)
split shared geometric edges on neighboring faces as well as selected faces.
Attribute seams retain separate vertices; new UV/color attributes interpolate,
and new skin weights combine the strongest four normalized influences.
Original vertex UVs, colors, joint IDs and weights remain exact. The wrist,
thumb, palm base and noneligible body positions/normals remain unchanged.
Hand-local distal contour subtracts .120 times smooth activation and lateral
groove depth, with modest thickness shaping and welded area-normal rebuilding.
An initial candidate with extra outer-finger shortening failed the required
valley margin; that extra shortening was removed and the valley depth corrected.

The supplied-rig regression passes for human-male, human-female, skater-male
and skater-female: two detected hands each, more than 500 added vertices,
protected attributes/positions/normals, finite unit normals, normalized skin
weights, four sampled tip lobes with valleys at least .035 below both adjacent
tips, oriented welded edge closure in the distal region, and bounds across
61 samples each of relaxed Idle/Run and raw Jump. This is not an animated
self-intersection or face-inversion proof. Reviewer build and scoped diff check
pass. Two intermediate Cargo attempts stopped before compilation from shared
disk exhaustion; successful retries verified the retained source.

Compared Chris, Kris, Kurt and Nurse against old-hand Idle renders and new
Idle/Run .25 renders. Then inspected all 75 roles, four angles each in Idle and
Run .25: 600 final hand-focused views across 25 contact sheets. Wrist attachment
and thumb continuity remain visually intact; no obvious new hand/costume
collision appears in those reviewed views. Fingertips are more readable than
the former broad mitten slab, but still thin and strongly shaded in places;
palms, knuckles, nails, finger proportions and shared articulation remain
simplified. Existing robe/skirt/coat waist joins, plain lower clothing, angular
leg contours and footwear limitations remain visible and unfinished.

No bitmap atlas changed in this pass: 40 distinct repaints / 41 role atlas files
remain installed. Full character quality objective remains active. Owned render
processes completed before the temporary hand review images/logs were removed.

## Kurt lower robe drape and painted fabric

Added native `robe_drape` only for Kurt/KurtOutside when their supplied
painting is present. Both shell layers gain shallow outward longitudinal folds
(maximum .020 source units), faded below y=1.52 over .30 units. The fitted
waist remains exact. The existing plain left indigo atlas island supplies
averaged linear vertex colors (u=.075+.035*sin(angle), v=.62+.32*height);
crossed collar, center seam, hands, shoes and skin are excluded. This is subtle
vertex color variation at the existing mesh resolution, not a new full-resolution
cloth texture. Periodic coordinates and welded normals match duplicate columns.
No raster asset or atlas count changed (40 distinct repaints / 41 role files).

The new drape exposed a failure in the retained apron face/normal agreement
check. Large sloping faces could overpower thin rim faces with area weighting.
Apron, pocket and belt now share corner-angle-weighted normals, preserving
geometry/skin attributes while giving thin rims their proper contribution.
Assertions were retained; failure messages now identify the piece and face.

Validation passes: targeted closed-shell/finite-normal/safe-indigo-island test
with exact waist preservation, .020 displacement bound, periodic seam and
unchanged topology/UVs; supplied apron regression for both roles over 61 Idle
and Run samples, actual deformed triangle clearance and face/normal agreement;
complete 29-role costume bind/animation regression; reviewer build; scoped diff
check. Reviewed both roles separately in four-angle Idle, Run .25 and Run .75
renders: 24 final views, against old Idle baseline. Apron inherits shallow
folds and remains attached; pocket/waist show no obvious new gaps in those views.
Lower robe variation remains subtle, and neck/footwear, underlying garment
joins and hem behavior, back ties/knot and outdoor basket are still unfinished.
Full character objective remains active. All owned processes completed before
temporary drape renders/logs were removed.

## Kimono Girl atlas repaint and kimono integration

Built-in imagegen edited the actual reviewer-exported fallback atlas. First
output `exec-c048e202-3d35-435f-8afa-65e2b08b47d7.png` had beige strips along the
clothing's shoulder edge and a boxed panel border; rejected. Selected repair
`exec-8556ff9a-add3-46c7-8263-a67d427fd54a.png` installed unchanged at
`assets/open-models/people/npc-textures/kimono_girl.png`.
SHA-256: `cc693a502afdac4f3bb978e4d5c537f9556ca005a7c49a388490b4cff1665a88`.
This is the 41st distinct repaint / 42nd role atlas file; Kurt's two roles share
one painting. Clean young adult face, organized dark swept hair, crimson silk
crossed collar/ivory insert, small plum blossom motif and brown footwear.

Native integration uses the existing female front width .56, rear plain
patch centered u=.075/v=.62, plain sleeve and collar-side placement, and the
lower garment's color sampled from u=.075/v=.75. Rounded hanging sleeves and
a fitted cream/gold obi with rear bow now include Kimono Girl in their supported
roster. These are original native meshes; the face/hair/cloth painting remains
the unchanged selected image.

First production views exposed blossom fragments on the shoulders despite
clean shoulder edges in the atlas. Added Kimono-Girl-only whole-face shoulder
mapping for high cloth faces above y=2.30 with mean lateral distance above .18
or a rear-facing center. All three vertices must occupy the cloth island
(u<.625/v>=.475); skin islands and central front collar remain separate.
The first narrower mapping removed large fragments but left one rear seam
fragment; the final selection includes that inner rear shoulder seam.
This remaps UVs without moving vertices or changing normals, winding, colors
or skinning. The broad UV preservation regression's historical torso-only
bounds needed explicit Kimono Girl shoulder/island cases. Other role bounds
remain unchanged; exact geometry and skinning assertions remain in force.

### Initial prompt

Use case: style-transfer. Edit this actual renderer fallback into a clean production flat square UV albedo atlas for Kimono Girl, a young adult Japanese dancer in a crimson silk kimono. Preserve EXACT normalized atlas rectangles and original facial feature positions, scale and orientation. Upper-left head x0..0.625,y0..0.479; original eyes around x=.26 and .36,y=.22, brows/nose/lips/ears remain at their original positions. Paint a graceful expressive adult face, clear warm brown almond eyes, softly arched dark brows, small composed closed smile, smooth warm peach skin and restrained cheek color. Stylized hand-painted game albedo with broad soft planes, NOT a photographic face, no dense pores. Glossy very dark brown hair in organized sweeping locks; keep original swept hairline and entire upper band filled with hair through x=.625. Lower head strip to y=.479 is clean warm neck skin, remove muddy grey neck. Lower-left x0.. .625,y.479..1: elegant deep crimson silk kimono torso, crossed V collar centered u=.3125,v=.48 with narrow ivory inner collar, gentle flowing cloth folds. Small restrained pale pink plum blossom motif only on the outer front chest, around u=.42,v=.65. No painted sash, belt, buttons or lapels. Keep left plain crimson strip u=0.. .14,v=.53..1 for sleeve/rear/lower robe sampling, no motifs, collar or seam there. Lower-right region x.625..1,y.75..1 remains matching crimson cloth with modest folds; no trouser belt. Shoe islands x=.625.. .8125,y=.125.. .5 dark warm brown footwear with neat simple seam and muted sole; preserve exact island placement. Keep right arm/hand/skin islands intact, including tiny finger forms at original coordinates. Output ONLY flat square UV atlas, no model, portrait layout, scene, text, grid, border, crop, rotated islands or new islands. UV alignment and original feature placement are highest priority. Clean organized low contrast albedo, no dramatic lighting, hard highlights or heavy grain.

### Targeted repair prompt

Use case: precise-object-edit. This image is the edit target, a flat game UV atlas. Fix ONLY the lower-left clothing rectangle x0.. .625, y.475..1. Remove ALL beige/skin-colored horizontal bars and exposed beige gutters from the upper clothing edge, specifically the left x0.. .125,y.475.. .50 and right x.49.. .625,y.475.. .52. Fill those areas with matching deep crimson silk, continuous to the exact clothing boundary y=.475. The central V ivory inner collar should remain, but every shoulder and outer edge is crimson fabric. Keep a continuous plain crimson sampling strip at u0.. .14 across v.475..1. Remove the dark vertical boxed-panel border at u=.606; extend cloth smoothly to u=.625. Preserve the face, hair, entire head/neck rectangle above y=.475, right-side shoe/skin/hand islands, blossom motif, central crossed collar and all normalized atlas positions exactly. Keep lower right clothing unchanged. No new borders/frames, seams at patch boundaries, text, model, scene, cropped or rotated islands. Output ONLY flat square atlas.
