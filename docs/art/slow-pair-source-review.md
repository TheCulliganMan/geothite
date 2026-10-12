# Slowbro and Slowking source review

Reviewed 2026-10-12 against the current canonical pair and primary sources.
The paired oblique software baseline was rendered successfully and inspected
at 1600×824. Both existing models have toy-like spherical eyes, undifferentiated
oval muzzles, extruded cream belly disks, mitten arms and plain feet. Slowbro's
shell is a sphere with a decorative spiral and scattered short spikes;
Slowking's crown is an oval with two straight cones and a red oval. These are the inspected baseline defects. Slowbro now has an installed
[original sculpture](slowbro-sculpture.md) with reviewed software views; its
native image-target capture and final-hash Rust check now pass. Slowking
now has its own installed [royal sculpture](slowking-sculpture.md) with
reviewed software views and passing canonical checks. Both final-hash muted
GPU captures and the all-251 Rust loading check now pass.

## Official anatomy references

The official [Slowbro entry](https://sg.portal-pokemon.com/play/pokedex/0080/)
and [Slowking entry](https://sg.portal-pokemon.com/play/pokedex/0199/) both
returned HTTP 200. Their named primary artwork was opened as silent static
images in background browser tabs and inspected in full.

[Slowbro artwork](https://sg.portal-pokemon.com/images/pokedex/4d93d1171a1b9551989b17b4ae6838e4e9e98378.png)
shows a continuous pink pear-shaped torso/head, rounded ears with recessed
inner curls, tall white eyes with tiny black pupils, a broad cream muzzle,
an open pink mouth with two small upper canines, hanging short arms with
white claws, a cream segmented lower belly, heavy thighs and two white toe
claws per foot. The tail-mounted shell is a separate gray-blue creature with
overlapping curved spiral turns, large tapered spines, dark angular eyes and
a toothed opening gripping the pink tail. A featureless ball cannot substitute
for that shell creature.

[Slowking artwork](https://sg.portal-pokemon.com/images/pokedex/31aa203eeae4980dfcce57f2e1316406683a4774.png)
shows a slimmer upright pink body, tall white eyes with tiny pupils, broad
cream muzzle with small lower-edge teeth, short clawed hands, rounded thighs
and a thick tapering upturned pink tail. The substantial collar has alternating
red and pale flared panels. Its gray spiral crown has a fitted brim, two large
curved lateral horns, an irregular pointed upper crest and a red round jewel
with a pale rim. Two straight cones on a plain oval do not capture this anatomy.

## External candidates and actual access

Specific primary model API responses returned HTTP 200 and identify these
candidates as downloadable under CC BY 4.0:

| Candidate | Creator | Faces | Listed animation clips |
| --- | --- | ---: | ---: |
| [Slowbro](https://sketchfab.com/3d-models/34c4cac468d048fd87f762afda1ec009) | nguyenlouis32 | 5,318 | 0 |
| [Slowbro - Pokemon](https://sketchfab.com/3d-models/093feb5a76f94a0c816c1ec78021c01b) | Chechorams16 | 5,318 | 0 |
| [Slowbro](https://sketchfab.com/3d-models/9ab3e8d87e4947a28506c90eeadadb83) | Mrmocom | 6,110 | 0 |
| [Slowking](https://sketchfab.com/3d-models/936e43d115414ac6a40d0df80eed6dbe) | nguyenlouis32 | 950 | 0 |

All four primary `/v3/models/<uid>/download` endpoints returned HTTP 401.
No archive was retrieved, so bones, weights, deformation and actual asset
quality have not been inspected. Zero listed animation clips does not prove
that a model lacks a rig. Search also exposed a Galarian Slowking candidate,
which does not match the Crystal species form. No external mesh, rig, texture
or animation has been imported. The two original replacements preserve the
normal species routes and pass their geometry checks, three inspected final
views, final-hash muted GPU captures and all-251 Rust loading verification.
Live-window, browser, animation and gameplay paths remain outside that proof.
