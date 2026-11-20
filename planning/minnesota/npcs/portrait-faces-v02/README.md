# Gym Leader and professor battle portraits

All eight Gym Leaders and Professor Tamarack now have image-generated waist-up
portraits, converted to the native 56×56 format. The redesign focuses on adult
head-to-shoulder proportions, visible pupils, distinct noses and readable mouths.
The previous full-body set remains rejected history in `portrait-faces-v01`.

This is a complete nine-character **local review candidate**, not user visual
approval or a public deployment. The selected pack and regional asset-plan
sources remain unchanged while this set is reviewed.

## Sources and prompts

Each linked image was created with the built-in image tool. Exact prompts,
native source and per-character conversion metadata are saved together. Native
rows are unedited Rust conversion output; no manual pixel edits follow generation.

| Character | Generated source | Exact final prompt | Native source |
| --- | --- | --- | --- |
| Varel | [Image](varel/image-source-v2.png) | [Prompt](varel/image-prompt-v2.txt) | [56×56](varel/native-pixels-v1.json) |
| Torven | [Image](torven/image-source-v1.png) | [Prompt](torven/image-prompt-v1.txt) | [56×56](torven/native-pixels-v1.json) |
| Avrin | [Image](avrin/image-source-v1.png) | [Prompt](avrin/image-prompt-v1.txt) | [56×56](avrin/native-pixels-v1.json) |
| Brannic | [Image](brannic/image-source-v1.png) | [Prompt](brannic/image-prompt-v1.txt) | [56×56](brannic/native-pixels-v1.json) |
| Saven | [Image](saven/image-source-v3.png) | [Prompt](saven/image-prompt-v3.txt) | [56×56](saven/native-pixels-v1.json) |
| Sivren | [Image](sivren/image-source-v2.png) | [Prompt](sivren/image-prompt-v2.txt) | [56×56](sivren/native-pixels-v1.json) |
| Orlis | [Image](orlis/image-source-v1.png) | [Prompt](orlis/image-prompt-v1.txt) | [56×56](orlis/native-pixels-v1.json) |
| Norek | [Image](norek/image-source-v3.png) | [Prompt](norek/image-prompt-v3.txt) | [56×56](norek/native-pixels-v1.json) |
| Professor Tamarack | [Image](tamarack/image-source-waist-up-v3.png) | [Prompt](tamarack/image-prompt-waist-up-v3.txt) | [56×56](tamarack/native-pixels-waist-up-v3.json) |

`conversion-all-v1.json` and each character's `conversion.json` record exact
sources, original sources, palettes, whole-image sampling phases and fit bounds.
`publication-all-v1.json` is the complete pack publication plan.

Every final native grid was reproduced from its generated image and recorded
conversion settings. All non-battle fields match the original JSON: identity,
palette and all six overworld poses. Encoded PNGs match those regenerated rows.
This audit is `target/trainer-portrait-all/source-provenance.json`.

## Visual review

All nine were reviewed at native size and equal nearest-neighbor enlargement.
An independent pass checked pupil/white separation, nose and mouth visibility,
adult proportions and identity. Varel was regenerated to expose both eye sockets;
Sivren and Saven needed thicker source strokes; Norek needed explicit pupils and
continuous contours; Tamarack's final source separates the far pupil from her
face outline. No sampling phase was used as a substitute for missing source art.

These remain deliberately coarse four-color portraits. Fine eyebrow/ear details
simplify at 56×56, and some thin source edges retain isolated accent pixels.
Tamarack keeps her original warm-gray skin palette. These are style limitations
visible in the native review, not claims of perfect source-image fidelity.

- Complete actual native sheet: `target/trainer-portrait-all/all-nine-native.png`
- Native sheet hashes: `target/trainer-portrait-all/all-nine-native.json`
- Independent review: `target/trainer-portrait-all/independent-visual-review.md`
- Encoded images: `target/trainer-portrait-all/NAME/encoded/`

## Combined review pack

- Parent: `content-packs/minnesota-intro-fair-patrols-v01.crystalpack`
- Parent SHA256: `cfb028aec4080314247063a3c8686fa2f1b361a7c84bc7df3a4f5e3413aab0bd`
- New pack: `content-packs/minnesota-trainer-waist-up-faces-v01.crystalpack`
- SHA256: `778dc512c6559e2fcbae052c329987bf5958d80e58bf5933bf29a84d46d28fde`
- Content identity: `496f64bfc9b64a1ad512f5e02903110b0526e17fd812c98062230829cd2a6a04`

The publisher verifies all nine installed preimages, preserves gameplay/audio,
and reads back the written pack for equality. Exactly 18 resources change: the
nine trainer PNGs and nine 2bpp banks. Palettes, overworld banks, maps, teams,
AI, rewards, progression and unrelated resources remain unchanged. Tamarack's
portrait remains bound to her opening; no professor battle or party was added.

`world.json` records this candidate as `trainer_portrait_waist_up_revision`.
It is unselected and undeployed. Existing saves remain bound to their packs.

## Reproduce and run

With the existing built Rust tools, from the repository root:

```sh
target/debug/examples/pack_trainer_portraits \
  content-packs/minnesota-intro-fair-patrols-v01.crystalpack \
  planning/minnesota/npcs/portrait-faces-v02/publication-all-v1.json \
  content-packs/minnesota-trainer-waist-up-faces-v02.crystalpack
target/debug/geothite play \
  content-packs/minnesota-trainer-waist-up-faces-v01.crystalpack \
  --save target/trainer-portrait-all/player.crystalsave
```

Publication output must be a fresh ignored path; the example uses v02 to retain
the verified v01 pack. Native encoding uses `npc_sprite_candidate --native`.
The optional `trainer_portrait_palette` importer takes an image, fresh output,
original light/accent RGB555 values and `--native PHASE_X PHASE_Y`. Its four
focused tests, focused Clippy and workspace formatting passed during the prior
Tamarack iteration; this expansion changes artwork/data, not Rust code.

## Verification status

The exact combined-pack controller and production-client checks are in progress.
Final results and evidence paths will be recorded here before delivery.
