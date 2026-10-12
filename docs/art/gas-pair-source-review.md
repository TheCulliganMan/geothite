# Koffing and Weezing source review

The previous canonical pair was rendered at 1600×824 and inspected in full.
Both identities use a faceted purple sphere with protruding oval eyes, a
stick smile, identical cylindrical vents and a small cream skull/crossbones
mark. Weezing has a second smaller copy. These baseline shapes do not
capture the distinct expressions or soft cratered surface in the references.

The [official Koffing page](https://sg.portal-pokemon.com/pokedex/0109/) and
[official Weezing page](https://sg.portal-pokemon.com/pokedex/0110/) were read,
and their actual main images were retrieved with HTTP 200 and inspected:
[Koffing artwork](https://sg.portal-pokemon.com/images/pokedex/fd10b57c1ac95c00638aed70b4a41d679e9af2b6.png)
and [Weezing artwork](https://sg.portal-pokemon.com/images/pokedex/4c76d556bda94f8eb8b73a9e58ef051262e78b7d.png).
Regional-form and neighboring-species images were excluded.

Koffing has a softly irregular lavender body, physical crater-like vents,
broad half-open white eyes, tiny pupils, a cheerful open grin, small downward
canines, pink lower mouth and cream skull/crossbones emblem. Weezing has a
larger lower orb, a smaller upper orb and a small rear node on a stalk. Its
half-open eyes and mouths are strained, and the larger face has prominent
upward canines. The cream skull/crossbones mark belongs to the larger orb;
the smaller one has a skull-shaped ring. Pale gas billows surround the
reference artwork. These images guide anatomy and are not shipped assets.

Primary public model APIs and actual download endpoints were checked for
five relevant CC Attribution / CC BY 4.0 listings:

- [Koffing by himanshusood](https://sketchfab.com/3d-models/koffing-bdd4a5f095f944019f79e94ea928bedf):
  1,908 faces and one listed animation clip; described as a PBR model.
  The clip listing is not verified skeletal skinning.
- [Koffing by CitronLegacy](https://sketchfab.com/3d-models/koffing-pokemon-9fe2aba3db804c08b1ffe639c62a9249):
  1,896 faces, zero listed clips. The creator describes original Blender
  modeling and Substance Painter hand painting.
- [Free HQ Koffing by mo7amed85](https://sketchfab.com/3d-models/free-hq-pokemon-koffing-downloadable-6b1b5db873b244dba206e7dde6716a72):
  719,820 faces and zero listed clips; advertises a free download.
- [Weezing by andreas9343](https://sketchfab.com/3d-models/weezing-eaf8fba8643f4be794268d9aa23af738):
  25,388 faces and zero listed clips; the creator describes a school monster
  modeling assignment and explicitly invites downloading.
- [Weezing by Chechorams16](https://sketchfab.com/3d-models/weezing-pokemon-9b0bfe350fe642d699b80817e71efde7):
  4,924 faces, zero listed clips and an empty description.

Each actual download endpoint returned HTTP 401, requiring authentication.
No archive, mesh, texture, rig or animation was retrieved or imported. Zero
listed clips does not establish absence of a rig, and a listed clip alone
does not establish a usable one. Paid print listings and regional-form
models are not replacements for this Crystal pair.

Original candidates in `tools/gas_pair_sculpt.py` use individually authored
soft radial craters, actual front recesses, fitted half-open eyes, separate
grin/frown shapes and cream emblems. Weezing has actual joining stalks and a
rear node. The first adult fang paint bent across the recessed surface and
read as tear trails; it was rejected visually in favor of solid rooted teeth.
The first dark mouth lining also crossed its recessed parent at a face/edge
sample. Its surface offset was increased without changing the guard. Static
billow experiments with rigid trails and ridged volumes were rejected
visually; two final floating clouds each have a central volume and four
connected rounded lobes.

The [original replacements](gas-pair-sculpture.md) are installed: Koffing has
23 parts and 55,900 triangles; Weezing has 35 parts and 90,924 triangles.
All twelve final candidate checks passed in 47.143 seconds. Final oblique,
front and rear software views for each species were inspected individually.
All twelve installed canonical checks passed in 57.767 seconds. Queued
shared-cache jobs were explicitly canceled while waiting without compiler
children, and fresh jobs ran in an isolated ignored dependency cache. The
native build exited 0. Both final-hash muted GPU captures exited 0 and were
inspected at 2000×1400. Rust integration exited 0 with the exact model check
passing in 3.97 seconds; a frozen executable containing both final hashes
passed the exact rerun in 3.17 seconds. All gas-pair verification jobs are
terminal, and task-owned reference/capture scratch was removed. The isolated
ignored build cache is still needed for the ongoing full species upgrade.
These are original static meshes, not a claim of an imported rig or animated
gas. No ordinary live-window, browser, animation or gameplay approval is claimed.
