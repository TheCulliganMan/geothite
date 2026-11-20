# Trainer portrait redesign: visual review

The user rejected the previous nine-portrait set: the professor looked chibi,
her head was too large, and native faces lacked readable eyes and noses. The
regional asset plan has been restored to its earlier sources. The rejected
`portrait-faces-v01` set is retained as history, not an approved replacement.

This revision starts with Professor Tamarack, an older woman with silver hair,
low reading glasses and a moss field coat. The first two new full-body sources
are rejected experiments: one remained too youthful, and the smaller adult
version lost facial features when reduced to 56×56.

## Current candidate

[The current source](tamarack/image-source-waist-up-v2.png) was created with the
built-in image tool using [this exact prompt](tamarack/image-prompt-waist-up-v2.txt).
Waist-up framing allows an adult head-to-body ratio and more room for facial
features. This framing is a proposed direction; user visual acceptance remains open.

The first waist-up source still lost its fine facial strokes. The second source
uses coarser shapes and places the reading glasses at her collar, leaving the
eyes and nose unobstructed. Its native eyes, nose and upturned smile are readable,
but the far pupil still joins the face outline. This is a review candidate, not
a claim of finished visual polish or user acceptance.

`tamarack/native-pixels-waist-up-v2.json` contains unedited machine-converted
battle rows. The original palette and all six overworld frames are unchanged.
`conversion-waist-up-v2.json` records the source, crop, fit and single global
sampling phase. No face-specific painting or head enlargement follows generation.

The optional Rust `trainer_portrait_palette` example performs whole-image Lab
palette matching and native sampling. The selected comparison uses phase
`0.5 0.25`; nine trial phases remain under ignored
`target/trainer-face-diagnosis/phases-waist-up-v2/`. The other phases are not
alternative accepted portraits. The source and actual native comparison is
`target/trainer-portrait-rebuild/tamarack-waist-up-v2-review.png`.

## Local review pack

The fresh ignored `content-packs/minnesota-tamarack-waist-up-review-v02.crystalpack`
uses `content-packs/minnesota-intro-fair-patrols-v01.crystalpack` as its parent.
Its content identity is
`1edb1a112b27f221759af4d6ab95d6f02f69492f561f0806a635507806d37b2d`.
Its file SHA256 is
`204c63411be8fbfb57f84e62c6cede8a42eb4368501e698276042e61718e0ac2`.
Publication readback passed and exactly two resources changed: Tamarack's battle
PNG and 2bpp bank. Gameplay, audio, maps, palettes and overworld art are preserved.
The pack is unselected and undeployed. Tamarack remains a field professor whose
portrait is also used in the opening; no battle or party was added.

The exact v02 pack passes the production-controller Tamarack opening test,
reaching the bedroom and opening Start. Its rendered 56×56 portrait matches
the encoded candidate pixel-for-pixel. Captures, controller log and verification
are under `target/trainer-portrait-rebuild/waist-up-v2-gameplay/`. These checks
verify integration, not visual approval. The prior waist-up v01 also passed
mechanically but was superseded because its native facial detail failed review.

Reproduce the candidate with the current Rust tools:

```sh
target/trainer-face-diagnosis/tools/trainer_portrait_palette \
  planning/minnesota/npcs/portrait-faces-v02/tamarack/image-source-waist-up-v2.png \
  target/tamarack-fresh-review/native.png 25370 8683 --native 0.5 0.25
target/debug/examples/pack_trainer_portraits \
  content-packs/minnesota-intro-fair-patrols-v01.crystalpack \
  planning/minnesota/npcs/portrait-faces-v02/publication-waist-up-v2.json \
  content-packs/minnesota-tamarack-waist-up-review-v03.crystalpack
```

Output paths must be fresh. The first command illustrates native sampling; the
publication plan uses the recorded native source with preserved field frames.
The optional importer passes four focused tests, the focused example Clippy
check, and workspace formatting. Existing library Clippy warnings remain.

## Remaining scope

Varel, Torven, Avrin, Brannic, Saven, Sivren, Orlis and Norek remain pending a
successful visual direction. No new leader design has been accepted in this
revision. Mechanical checks of the earlier set are not evidence of visual quality.
