# Individual roster motion review

## Tenth motion batch — 2026-10-12

Bug Catcher, Old Link Receptionist, Red’s Mom and Rocker were rendered silently
at production proportions in Run 0.125 and 0.625. Capture jobs 27829 and 58313
passed. All eight images were individually inspected in all four angles:
32 views. Normal production animation and `CRYSTAL_PREVIEW_MUTE=1` were used.
Artifacts: `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/roster-motion-tenth-{0125,0625}/`.
Reviewer SHA256: `7324e5f6d68bff66dff0dccf691250e303f56cab71833627a748265aef858c11`.
This reviewer predates the three-role gutter candidate; these four roles do not
use that edit. No whole model is approved. These two poses do not establish a
complete gait, bicycle coverage or in-game quality.

| Identity | Direct finding in both sampled Run poses |
| --- | --- |
| bug_catcher | Drawn eyes and shorts/hat read. Thin adult-like child proportions, long angular jaw/nape, thin flat hat brim, plain blue shirt, pale rear/side waist strip, angular shorts/knees, generic hands/blue shoes and absent net remain. |
| old_link_receptionist | Drawn blue eyes read. Rigid pointed ponytail and angular jaw/nape, plain red shirt/dress without receptionist-specific construction, skirt knee scallops/red trouser-like layers and generic hands/dark shoes remain. |
| reds_mom | Drawn face and ivory apron/pocket/waist band read. Rigid bun, angular jaw/nape, plain red top, apron surface banding, skirt knee scallops/red trouser-like underlayers and generic hands/dark shoes remain. |
| rocker | Drawn eyes and orange spike read. Rigid faceted hair and angular jaw/nape, plain green long shirt, pale rear/side waist strip, generic hands/green trainers and absent music props remain. |

Atlas inspection is required before changing the Bug Catcher/Rocker waist UVs.

## Ninth motion batch — 2026-10-12

Subsequent waist correction: Clerk, Outdoorsman and Unused Guy no longer sample gray shirt separator padding. Thirty-six mapped Idle/Run views and a fresh 75-role UV/rig regression passed; engine installed with HTTP verification. Their remaining outfit and anatomy defects stay open. See `staff-waist-uv-review.md`.

Oak, Pryce, Outdoorsman and Unused Guy were rendered silently at production
proportions in Run 0.125 and 0.625. Capture jobs 28264 and 33056 passed. Each
of the eight images was individually inspected in all four angles: 32 views.
Artifacts: `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/roster-motion-ninth-{0125,0625}/`.
Reviewer SHA256: `7324e5f6d68bff66dff0dccf691250e303f56cab71833627a748265aef858c11`.
Normal production animation and `CRYSTAL_PREVIEW_MUTE=1` were used.

| Identity | Direct finding in both sampled Run poses |
| --- | --- |
| oak | Coat, red shirt, lapels and pen read. Soft portrait-like cheek/nose/lip shading and rounded eyes still need cleaner anime direction. Rigid faceted gray hair, angular jaw/nape, abrupt dotted waist seam, coat front panels wrapping around separate thighs, plain rear coat and generic hands/brown shoes remain. |
| pryce | Drawn mature face, gray brows/side hair and cardigan buttons read. Hair is flat pointed temple painting; angular jaw/nape, generic blue cardigan, absent scarf/cane and generic hands/blue trainers remain. |
| outdoorsman | Narrow drawn eyes, beard and backpack read. Hat brim is a thin flat plate; stiff beard, absent vest, plain green shirt/suspender-like straps, pale rear/side waist strip, angular jaw/nape and generic hands/brown shoes remain. |
| unused_guy | Drawn eyes and brown hair read. Suit/teal tie are absent; plain ochre shirt, pale rear/side waist strip, angular jaw/nape and generic hands/black shoes remain. |

No whole model is approved. These two poses do not prove a complete gait,
bicycle coverage or in-game quality. Atlas inspection precedes any UV correction.

## Eighth motion batch — 2026-10-12

Subsequent Nurse correction: twelve mapped Idle/Run views accept narrower anime eyes, clean forearms and a white skirt matching the shirt. Canonical atlas, engine and bundle installed with HTTP verification; remaining geometry defects still apply. See `nurse-anime-uniform-review.md`.

Clerk, Nurse, Receptionist and Link Receptionist were silently rendered at
production proportions in Run 0.125 and 0.625. Each of the eight images was
individually inspected in its four angles: 32 views. Both capture jobs passed.
No whole model is approved; these samples do not establish complete gait,
bicycle or live browser quality.

Artifacts: `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/roster-motion-eighth-{0125,0625}/`.
Reviewer SHA256: `4c465d0083f58a772de038d02675e92d3e953a79cc534e3da6334cc39997a02f`.

| Identity | Direct finding in both Run poses |
| --- | --- |
| clerk | Drawn narrow eyes and green shirt/apron identity read. Apron is a shallow beige front panel without rear ties; pale rear/side waist strip persists. Rigid hair, angular jaw/nape, generic hands and green trainers remain. The waist strip needs comparison with the actual atlas before changing UVs. |
| nurse | Large glossy round eyes and tiny smile still read as a doll. Flat white cap and rigid pink buns, angular jaw/nape and generic hands/white trainers remain. Skirt is visibly cream beside the white shirt and curls around each knee, revealing white trouser-like underlayers. A pink patch appears on the raised forearm in Run 0.625; inspect sleeve/skin mapping before fixing. |
| receptionist | Drawn eyes, collar/tie and navy uniform read. Hair forms a rigid helmet-like bob; long angular jaw/nape, generic hands/black shoes remain. Skirt bends into knee scallops with navy trouser-like underlayers; abrupt waist transition and plain rear costume remain unfinished. |
| link_receptionist | Drawn face and white cap read. Rigid bob and flat cap, angular jaw/nape, generic hands/white shoes remain. Plain pink shirt/skirt lack uniform construction, skirt curls around knees with pink trouser-like underlayers, and waist join is abrupt. |


## Seventh motion batch — 2026-10-12

Subsequent youngster correction: both boys now use reviewed head scale 0.84 and width 0.98; Standing Youngster’s waist UV separator bleed is removed. Twenty-four mapped Idle/Run views and fresh instance/UV regressions passed; engine deployed. See `youngster-proportions-review.md`.

Subsequent face correction: Whitney, Red and Rival now have reviewed drawn anime eyes. Each candidate passed twelve individually inspected Idle/Run views and was deployed in the Rust-validated people bundle. The historical batch findings below describe the prior paintings; geometry defects remain open. See `whitney-red-rival-anime-face-review.md`.

Chris, Kris, Rival, Red, Blue, Youngster, Standing Youngster and Whitney were
silently rendered at production proportions in Run phases 0.125 and 0.625.
Each of the sixteen images was individually inspected in its four angles:
64 views. These two samples are not a complete gait, bicycle or live browser
review. No whole model is approved.

Artifacts: `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/roster-motion-seventh-{0125,0625}/`.
Reviewer SHA256: `4e4c111be97165dc0abc506305328627bf031bddeec0e9ff6a0bc1dc0ddb5966`.
Both capture jobs completed successfully with preview mute enabled. The Standing Youngster atlas was subsequently viewed directly: gray separator padding sits above and beside its blue shirt island, matching the observed waist strip. No gutter correction for this role has yet been applied.

| Identity | Direct finding in both Run poses |
| --- | --- |
| chris | Narrow drawn eyes and painted collar/zipper/cuffs read; thin flat cap/brim, rigid painted hair, angular jaw/nape, generic hands and shoes remain. Side jacket details have little volume; rear is mostly plain beneath backpack. |
| kris | Drawn eyes and blue outfit/backpack read; rigid teal pigtails, flat cap, long angular jaw/nape and generic hands/shoes remain. Clothing relies on the generic shirt/trouser silhouette. |
| rival | Large round eyes still read as a doll despite angled brows. Red hair has rigid faceted locks; painted red zip/yoke details leave the long rear jacket plain. Angular jaw/nape and generic hands/shoes remain. |
| red | Large round eyes and tiny smile still read as a doll. White undershirt/red vest painting and backpack read; cap is thin and flat, hair/neck angular, hands and white trainers generic. Vest edges have little physical depth. |
| blue | Narrow drawn eyes, brown spikes and pendant painting read; hair has conspicuous polygon-shaped highlights, jaw/nape remain angular, shirt silhouette and hands/shoes generic. Pendant is a small flat painting. |
| youngster | Narrow drawn face reads, but tiny head and thin body resemble a scaled adult rather than a child. Blue shirt/shorts read; rigid hair, angular jaw/nape, knee/shorts transitions and generic hands/shoes remain. |
| standing_youngster | Small narrow face and thin adult-like body remain. Pale rear/side waist strip persists in both poses and needs comparison with the actual atlas. Plain shirt, angular nape, missing held ball, knee/shorts transitions and generic hands/shoes remain. |
| whitney | Large glossy round eyes and tiny smile still read as a doll. Pink pigtails are rigid faceted chunks; white/red shirt is plain at rear, shorts/knee boundaries angular, hands and red trainers generic. |


## Sixth motion batch

Bill, Black Belt, Blaine, Cal, Captain, Daisy, Gym Guide and Lance were silently
rendered at Run 0.125 and 0.625. Both capture jobs exited zero and each produced
all eight expected images. All sixteen images / 64 views were individually
inspected. Native reviewer SHA-256:
`96e48861e5fc249a67e7de283eca118186fdf8985e623da9a83eb13d9eb55899`.
Artifacts: `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/roster-motion-sixth-{0125,0625}/`.
This uses corrected production proportions and animation. Two sampled poses
are not continuous motion validation; no whole model in this batch is approved.

| Role | Findings in both sampled poses |
| --- | --- |
| bill | Drawn eyes and curly brown hair read, but a pale gray side/rear waist strip samples the shirt separator. Plain red long shirt lacks a polo collar/pen; long angular jaw/nape, rigid curls and generic hands/red trainers remain. |
| black_belt | Narrow drawn eyes, headband, wrap and belt read. Diagonal wrap bands remain stiff; white trousers have angular hip/knee shapes and generic pale trainers rather than finished martial footwear. Jaw/nape and hands remain unfinished. |
| blaine | Spectacles, gray side hair, thin moustache, tie and coat read, but rounded smiling toy eyes weaken the elderly face. Hard coat waist seam and independently curled lower coat panels, jaw/nape, hands and shoes remain. |
| cal | Drawn face, blue cap and backpack read, but pale gray side/rear waist strip persists. Plain red shirt lacks jacket construction, cap/brim are thin and simple, and generic hands, angular nape and shoe construction remain. |
| captain | Narrow mature eyes, captain cap, moustache, collar, buttons and belt read. Moustache is thin; large stiff collar, plain jacket rear, angular jaw/nape, hands and generic black shoes remain. |
| daisy | Large round glossy eyes and tiny smiling mouth still read as a doll. Blonde rigid hair curtain, long flat jaw/nape, scalloped moving skirt with blue trouser layers, generic hands and white trainers remain. |
| gym_guide | Drawn face, lapels/tie and blue jacket read, but pale gray side/rear waist strip persists. Plain shirt-like jacket outline, rigid hair, long jaw/nape, generic hands/shoes and absent clipboard remain. |
| lance | Red hair, collar/buttons and broad cape identify the role, but round wide eyes and smiling face read as a child/toy. Cape attachment is thin, hair rigid, jaw/nape long and hands/footwear generic. |

The actual Bill, Cal and Gym Guide atlases were separately inspected and have
gray separator padding immediately above and beside the plain cloth island.
Their new gutter guard and supplied-painting contract are building; fresh
mapped review and deployment are tracked below when completed. Daisy and
Lance face-only imagegen trials are separate, with mapped review required
before replacing either production atlas. These findings precede those trials.

### Sixth-batch gutter correction result

The whole-cloth-face gutter guard now includes Bill, Cal and Gym Guide. The
actual source paintings retain their gray separators; affected cloth triangles
are split and constrained to the painted fabric, preserving position, normal,
color, joint IDs and weights. The supplied-painting regression now covers all
eleven affected roles, samples triangle interiors as well as corners, and
retains unchanged-role and adjacent-island checks across the 75-role roster.
Fresh Cargo invocation completed with exit 0 in 5m15s: one actual character
test passed in 1.83s (1,327 tests filtered out); other selected libraries had
zero matching tests.

Native reviewer build completed with exit 0 in 1m27s, SHA-256
`120f120cdd1bb5f885885ba79330f1f06e6d156c2e2c96450bb7e2148f47183a`.
The three silent capture jobs produced Bill, Cal and Gym Guide at Idle,
Run 0.125 and Run 0.625, each exited zero. All nine images / 36 views were
individually inspected in `captures/sixth-gutter-{idle,run125,run625}/`.
The pale side/rear waist strip is absent in every sampled pose, with the
existing facial features, backpack or lapels/tie still readable. This local
UV correction is accepted; the other whole-model findings above remain open.

Browser build completed with exit 0 in 2m41s and bindgen completed with exit 0.
The served engine SHA-256 is
`3dcb163786a822e58c1e35233b430544bee8fa1efa9fd4892b2e3eb80e030c7b`,
glue `bd7370761067f66de6a6b62c489823260f6dcedffb2b83a30dee9ea89cc61731`.
All three page build references use `3dcb163786a822e5`; startup mute remains
enabled. HTTP response bytes match both generated files and the updated page.
The validated four-rig/75-painting bundle also contains the separately reviewed
Daisy/Lance face replacements. WASM/bundle staging and the temporary face-trial
asset root were removed. No refreshed live-browser frame is claimed.

## Fifth motion batch

Eight roles were silently rendered in four angles at Run 0.125 and 0.625.
Both capture processes (11666 and 52900) exited zero. All sixteen images /
64 views were individually inspected. These processes loaded the prior
reviewer SHA256 `75aa8c2d7291b2de516d526eb21ab15adb0a1fb3adb6b6dc60d8e81827ac42c6`,
before native build 19381 replaced it. None is an apron role; the pending
apron normal change does not affect these findings. Artifacts:
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/roster-next-run{125,625}/`.
This extends motion inspection; no whole model is approved.

| Role | Findings in both sampled poses |
| --- | --- |
| beauty | Drawn adult eyes and wrap neckline read in both poses; rigid hair curtain, long angular jaw/nape, blue trouser-like legs, scalloped skirt hem and generic white trainers remain. |
| cooltrainer_f | Zipped blue top and ponytail read; round doll eyes and long pointed face, stiff ponytail, scalloped skirt around raised knee, generic hands/navy shoes and absent satchel remain. |
| cooltrainer_m | Narrow drawn eyes and spiked hair read; pale side/rear waist strip is conspicuous in both poses. Plain long shirt lacks jacket/wristband; angular jaw/nape, generic hands and blocky navy shoes remain. |
| pharmacist | Glasses/moustache and white robe read; large rounded toy face, painted side hair, generic hands/shoes and absent medicine bag remain. Robe hem bends around knee/foot with white under-hem leg layers and plain rear. |
| pokefan_m | Glasses/moustache and brown polo read; rounded smiling doll face, sparse painted scalp wedges, generic broad hands, brown trousers/shoes and missing ball badge remain. |
| sage | Beads and tan wrap read; round childlike bald face, sparse painted side hair, generic hands/shoes and plain rear remain. Robe curls around lifted leg, revealing tan trouser-like layers. |
| scientist | Glasses, lapels, blue shirt and pen read; smiling round doll eyes, rigid hair, angular jaw/nape, generic hands/shoes remain. Coat has an abrupt waist seam and lower panels curl separately around thighs. |
| super_nerd | Glasses, collar and painted suspenders read; rounded doll eyes, rigid short hair, long jaw/nape, generic hands and pale shoe soles remain. Rear suspenders are absent and painted sleeve bands do not alter the generic shirt silhouette. |

## Fourth motion batch

Six roles were rendered silently in four angles at each of Run phases 0.125
and 0.625. Both capture processes exited zero; all twelve images / 48 views
were individually inspected. Reviewer SHA256:
`627f074f60c5342c6ff0324049209f88917d790bf4345b3f38d0873ad651f0b1`.
Artifacts: `/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/roster-motion-fourth-{0125,0625}/`.
The requested `boarder` and `firebreather` filter names produced no images;
they are excluded from the review count. This executable predates the pending
short-skirt experiment. None of these six roles is approved as a whole model.

| Role | Findings in both sampled poses |
| --- | --- |
| elder | Painted beard/side hair and mustard wrap read, but huge bald doll head, thin upright body, generic open hands/shoes remain. Long robe hem scallops around raised knee/foot with a visible underside; rear is a plain shirt/robe without folds. |
| elm | Glasses, lapels/tie and white coat read. Coat waist has a hard bright seam; lower panels curl around thighs. Round face, rigid short hair, long angular nape, plain rear coat and generic hands/shoes remain. |
| fisher | Red cap and narrowed drawn face read; plain blue shirt has no vest or rod. Angular jaw/nape, weak hands, knee shapes and blocky brown shoes remain. |
| fishing_guru | Hat, narrow mature face and painted moustache read. Pale side/rear waist strip is conspicuous in both poses. Plain blue shirt has no vest or rod; hands, jaw/nape and shoes remain generic. |
| gameboy_kid | Shorts and blue shirt read, but pale side/rear waist strip persists. Thin adult-shaped limbs, long pointed face, rigid hair and shoes remain; handheld absent. |
| biker | Narrow drawn eyes/stubble read, but painted black vest is flat on a generic long shirt. Pale side/rear waist strip persists. Slender body, generic hands/shoes and angular jaw/nape do not convey the role. |

Individual inspection of all three actual atlases confirms gray separator
padding above and beside the blue/dark cloth rectangle. The existing whole-face
UV clamp is now extended to Fishing Guru, Gameboy Kid and Biker in source. The
supplied-painting regression includes them while retaining all-roster checks
that unaffected geometry, UV islands and skin influences stay unchanged.
Cargo regression session 73066 is queued; no passing test, fresh corrected
mapped capture or browser deployment is claimed yet.

The isolated current reviewer build subsequently passed (exit 0, 1m40s),
SHA256 `46ee1858821815e2d31c0761fc344eff3ad0bd58736881d6dd14ec4b60e25704`.
All three roles were recaptured silently in four angles of Idle and Run phases
0.125/0.625 in `captures/three-gutter-fixed-{idle,run125,run625}`. All three
batches exited zero; all nine images / 36 views were individually inspected.
The pale waist strips are gone in each pose. Generic hands, shoes, body shape
and incomplete role outfits remain; no whole-model approval. The freshly
compiled isolated test binary's supplied-painting regression passed directly:
one actual test, exit 0, 2.26s. Browser engine deployment is still pending.
The queued isolated Cargo invocation also completed successfully: one actual
painting test passed in 1.36s, total queue/build time 8m59s. The browser build
is now actively compiling in session 39606. Startup mute remains required.
The WASM build completed with exit 0 in 2m30s; bindgen also completed with
exit 0. The rebuilt engine/glue are now served locally and HTTP bytes match
the generated files exactly. Engine SHA256:
`8581005e2bec024213b734875b5e29e12c73cc324cfe9a2ebaf38c9254937944`;
glue SHA256:
`fb2bc98f88e47eba1e8619574c5b964470856e4a631241efc348b2617cce0fc5`.
All three served cache references use the new engine prefix. Startup mute is
preserved and the existing four-rig/75-painting bundle remains installed.
Owned WASM staging was removed after verification. This verifies delivery,
not a refreshed live browser frame or gameplay. The unrelated corrected
garment-normal regression remains failing on Kurt, as recorded separately in
short-skirt-motion-trial.md; the production skirt weighting was not changed.
Successful captures establish the evidence above, not full-cycle, bicycle,
game-context or whole-roster approval.

## First motion batch

Silent mapped production renders, with `Person::update` and current head/hand
scales. Reviewer SHA256
`b339b0c606b103c0b02700e5896bf4680feb1a7bb0bace9b5e1c9576f571b36e`.
This executable predates the new Chuck/swimmer torso profiles; neither role
is included in this batch. The eight reviewed roles have unchanged model code.

Capture batch completed with exit 0. Each role's four views at Run phases
0.125 and 0.625 was individually inspected (64 views total). Artifacts are in
`/Users/ryanculligan/.codex/artifacts/anime-character-review/pending-model-review/captures/roster-motion-0125/`
and `roster-motion-0625/`, named by role. These are two sampled poses, not a
complete animation-cycle, bicycle, or in-game-context review.

| Role | Findings in both sampled poses |
| --- | --- |
| bugsy | Green shirt/shorts and purple bob read, but silhouette is thin and adult-like; long angular jaw, rigid hair, weak knees/hands, generic sneakers, absent net/belt. |
| twin | Pink dress and pigtails read; hem pulls into a pronounced scallop around the raised leg, with a dark underside edge. Thin adult-like body, long jaw, plain trouser-like legs/shoes, absent ribbon detail remain. |
| teacher | Front cardigan/button painting reads, but skirt hem pulls around each raised leg with a pronounced scallop/underside edge. Generic legs/shoes, long jaw and angular nape, weak hands and rigid bun remain. |
| will | Narrower drawn face and pointed purple hair read, but red tee/trousers do not express the formal costume; mask/gloves absent. Pale lateral/rear waist strip is discontinuous at the front. Jaw/nape, hands and shoes remain generic. |
| sabrina | Maroon seam painting reads; rigid brown bob, round eyes and generic long shirt/trousers dominate. Silver shoe/heel patches are conspicuous in side/rear running views. Hands, long jaw/nape and intended outfit silhouette remain weak. |
| surge | Green tee/trousers and flat blond hair read; pale lateral/rear waist strip is conspicuous. Tank/vest, dog tags and distinct combat boots absent; angular jaw/nape and generic hands remain. |
| sailor | Cap, navy collar/red tie and rear flap read, but collar sits like a large flat patch. Angular jaw/nape, open rigid hands and generic shoes remain. No whole-model acceptance. |
| officer | Cap badge, front pockets/buttons and belt read in motion. Rear remains a generic plain shirt with flat belt stripe; angular jaw/nape, hands and generic shoes remain unfinished. |

None of these models is approved. The skirt deformation and pale waist strips
require source/UV investigation; artifact existence and successful capture do
not establish visual quality.

Inspection of the actual Will and Surge atlases found gray padding above and
beside their lower-left cloth rectangles. Source shirt gutter vertices lie
at heights 1.4143–1.4636 and 2.5112–2.5576. A targeted production UV correction
now splits whole cloth faces for these two roles, clamping them inside
u=0.005–0.595, v=0.51–0.99. Geometry and skin influences are copied exactly;
neck, hand and trouser islands retain their original samples. Other roles
are excluded. A supplied-painting regression checks triangle interior samples
for red/green cloth and attribute preservation across the roster. Its build
is live; neither a test pass nor removal of the visible strips is yet proven.
Fresh four-angle Idle/Run captures remain required before deployment.

The supplied-painting gutter regression subsequently passed via the freshly
linked native test binary: one actual test, exit 0, 1.03 seconds. Four-angle
Idle and Run 0.125/0.625 renders of Will and Surge in `torso-gutter-*` were
individually inspected (24 views): pale lateral/rear waist strips are gone.
Reviewer SHA256 `ecab0ee247affb69673bf8947e6f4ed4c1dce633fc66f667f080d4397c21c5dc`.
This verifies the specific mapping correction; their generic outfits, missing
accessories, angular heads, hands and footwear remain unapproved. Browser
engine deployment of this UV change remains pending.

The queued Cargo gutter test subsequently completed with exit 0 in 23m15s
including the shared build queue: one actual character test passed in 0.87s.
The other selected workspace libraries ran zero matching tests; those are not
additional character passes. The stale room test fixture correction compiled.

## Second motion batch

Reviewer `bc966f29488371e068564c548ea83a94deea5fb732c3fbade6e804fabaa9b0a8`
rendered eight further roles at Run 0.125 and 0.625. Both muted batches exited
zero, and all sixteen images / 64 views were individually inspected. Artifacts:
`captures/roster-motion-second-0125/` and `captures/roster-motion-second-0625/`
under the same external review root. The temporary frozen reviewer was removed.

| Role | Findings in both poses |
| --- | --- |
| mom | Collar/buttons and apron read, but apron edge has dark segmented lines; skirt pulls into a pronounced scallop around the raised leg. Rigid bob, angular jaw/nape, red trouser-like legs and generic shoes/hands remain. |
| lass | Ponytail and red school outfit read; skirt scallops around the raised knee, thin adult-like child silhouette and long jaw remain. Schoolbag absent in these views; generic dark shoes/hands. |
| granny | Glasses, gray bun and front shawl painting read, but shawl becomes a plain short-sleeved shirt at rear. Skirt curls/scallops at raised leg, with generic knees/shoes/hands and little age-specific body shape. |
| gentleman | Bow tie/lapel painting and gray moustache read; hat has a rounded bowler-like crown rather than a top hat. Cane absent. Pale lateral/rear waist strip, generic suit silhouette/shoes/hands and long angular jaw remain. |
| gramps | Glasses, moustache and cardigan seam/buttons read, but body is generic and upright; painted scalp boundary, angular jaw/nape and generic hands/shoes remain. |
| kimono_girl | Wrap, obi/bow and hanging sleeves read, but hem pulls around the raised foot with a dark underside. Small light mark on rear shoulder, rigid bun, round eyes, absent fan/hairpin and generic dark footwear/hands remain. |
| rocket | Red R and cap read, but pale lateral/rear waist strip persists. Intended gloves/boots absent; generic shirt/trousers, long angular jaw/nape and open hands remain. |
| rocket_girl | Red R, cap and magenta hair read, but hair forms a rigid squared bob; skirt scallops around raised leg. Intended gloves/boots absent, with generic dark legs/shoes and long angular jaw/nape. |

No whole model is approved. Actual Gentleman/Rocket atlas inspection confirms
gray separator padding above the navy cloth rectangle. The existing targeted
gutter correction now includes those two roles; its supplied-painting
regression checks navy triangle samples in addition to Will red and Surge
green, and still requires all other roles to remain unchanged. New test and
mapped render evidence for this extension remain pending; it is not deployed.

The mesh-diagnostic reviewer build and queued bare-body Cargo run failed with
exit 101 on concurrent unqualified dungeon `Model` references. Current source
already qualifies them as `crate::dungeon_models::Model`; a replacement
reviewer build is running. The earlier direct three-test bare-body pass remains
the latest successful character contract result, not a pass for those failed
Cargo commands.

Gentleman's rounded dome hat is replaced in source by a head-fitted, slightly
flared tall crown with a flat top, rounded brim edge and contrasting ribbon.
It uses the existing smooth 48-sided cap builder and the production head
attachment, rather than increasing the old ellipsoid's height. Other hats
remain on their previous paths. Existing cap regressions now include Gentleman
for forehead clearance, bounded fit, finite unit normals and outward face
winding. The cap test process and latest reviewer build are live; visual
acceptance and deployment of this silhouette change remain pending.

The extended gutter Cargo regression completed successfully: one actual
character test passed in 1.16s; total command time was 14m06s including the
shared queue and unrelated client build. The other selected libraries ran
zero matching tests. The freshly linked native test executable also passed
both cap regressions directly (two actual tests, 0.02s), including Gentleman.
The separately queued Cargo cap command remains live; it is not reported as
completed from the direct result.

A temporary reviewer linked directly to the newly compiled character library
completed all three `hat-gutter-pose-{idle,run125,run625}` capture batches,
exit zero; reviewer SHA256
`947c1f4e61cb8d2e1a9ebce399f6ef0a4c7d1c54189ec8f92e59af619719db89`.
All 24 Gentleman/Rocket views were individually inspected. The pale waist
strips are gone in both roles; the taller crown/flat top/ribbon read as a top
hat and stay anchored in these poses. These specific changes are accepted
incrementally. Generic hands, shoes, jaw/nape, jacket/boot construction and
Gentleman's missing cane still prevent whole-model approval. Browser
deployment remains pending the exposed-shoulder revision. The ordinary Cargo
reviewer build also completed with exit zero in 14m47s; its earlier pending
status is superseded by that result.

## Hat contract follow-up

The queued Cargo uniform-cap checks completed with exit 0 after 23m06s including
the shared build queue: two actual tests, 0.01 seconds. This confirms the
Gentleman top-hat fit and smooth surface contracts already reviewed in mapped
captures. It does not approve the whole character or establish browser deployment.

## Third eight-role motion pass

Jasmine, Karen, Clair, Erika, Janine, Koga, Morty and Falkner were rendered at
Run phases 0.125 and 0.625. Both batches exited zero and all sixteen images /
64 views were individually inspected. Captures are in
`captures/roster-motion-third-{0125,0625}` under the external pending-model-review
directory. Reviewer SHA256
`5a6b3085b2c7b3dd2d041b5031bd7ba2977bb061756c0051511998c5e578f0a8`.
It predates the latest exposed-arm and swimmer-foot edits; none of these eight
roles use those edits. This is a two-pose audit, not complete motion validation.

| Role | Findings in both sampled poses |
| --- | --- |
| Jasmine | Rigid brown bob; skirt hem scallops around moving knees with a dark inner edge; pale trouser-like legs and generic sneakers rather than a finished dress silhouette. |
| Karen | Grey hair is a rigid bob; large friendly eyes lack the intended mature expression; skirt is pulled into knee scallops; navy legs and generic grey sneakers undermine the outfit. |
| Clair | Blue hair and outfit identify the role by color, but the rigid bob, plain trouser suit and sneakers omit the distinctive cape, boots and hair silhouette. |
| Erika | Kimono wrap, obi and flower read; long hem rises with the knees and shows the underside; generic shoes/hands and angular neck remain. |
| Janine | Purple ponytail and red scarf read; generic suit, sneakers, hands and pale rear heel surfaces remain; wrist/boot shapes are unfinished. |
| Koga | Painted wrap and scarf read but body and hands remain generic; large round eyes, simplified hair and sneaker silhouette weaken the adult ninja identity. |
| Morty | Oversized round eyes and baby smile are inappropriate; headband/scarf read; generic body, hands and sneakers remain. Subsequent face-only revision is documented separately. |
| Falkner | Belt/wrap read; long hem follows knee bends and the sleeve is too simple; generic shoes/hands and symmetric short hair miss the role silhouette. |

No whole model in this batch is approved.


## Cooltrainer M waist-strip correction

The actual atlas was individually inspected: gray separator padding lies
above and beside the blue cloth island. The whole-cloth-face gutter clamp
now includes `cooltrainer_m`. The supplied-painting regression includes that
role while retaining unaffected-role and non-UV attribute checks across all
75 source roles. Fresh direct test 83463 passed (one actual test, 2.32s).

Native reviewer build 96239 passed; SHA256
e44307d6258fddab00b75cc6a0a057d7151a414320bf4a95615217cdf23b63a5.
Capture batch 88142 silently produced Idle, Run 0.125 and 0.625. All three
images / twelve views were individually inspected in
`captures/cooltrainer-gutter-{idle,run125,run625}`. The pale side/rear waist
strip is gone in every sampled pose. Missing jacket/wristband, generic hands
and shoes, angular jaw/nape and plain rear outfit remain. No whole-model
approval. Browser update accompanies the validated apron correction.

Validated engine and glue installed in the local served root; HTTP bytes
match both files. Engine SHA256:
0784e8847bafa3990d6e08a8d0cdbc223504fbe8ba5a7d22a423b265e973601f.
Glue SHA256:
d98b8d5814a24b6d33e8b1162927d9c25db66b290688b714850ceb9f69da0a03.
All three served cache references use 0784e8847bafa399. Startup mute is
preserved. The four-rig/75-painting bundle remains installed with the
Pharmacist face replacement. Temporary WASM staging removed. No live browser
refresh was performed; this proves served bytes, not a rendered browser frame.
