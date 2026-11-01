# Cubone and Marowak source review

Reviewed 2026-10-12. The paired current canonical oblique software baseline
exited 0 at 1600×824 and was inspected in full. Both models share the same
toy brown body, cream oval belly, spherical skull with a projecting oval
snout, brown protruding oval sockets, plain cone horns, generic limbs and
upright bone club. Only horn length meaningfully differentiates the adult.
Both need distinct skull profiles, recessed eyes, proper paws and a bone
that is physically gripped rather than resting beside a generic round hand.

The official [Cubone](https://sg.portal-pokemon.com/play/pokedex/0104/) and
[Marowak](https://sg.portal-pokemon.com/play/pokedex/0105/) entries returned
HTTP 200. Their named primary artwork was downloaded into ignored scratch
and inspected in full:
[Cubone art](https://sg.portal-pokemon.com/images/pokedex/6c44a0ccb922bbb4345214f3fa0b9436a03c11f2.png)
and [Marowak art](https://sg.portal-pokemon.com/images/pokedex/1a2a79bce9b5c63de4d6db9b895a34ce6bf6733d.png).

Cubone has a compact rounded tan body with a broad cream belly, heavy thighs,
short feet with white toe claws, small bent arms and a short pointed tail.
Its oversized white skull helmet has broad angular orbital cavities exposing
brown skin and a white eye with dark pupil, a broad tapered snout, dark
triangular nostril openings, cheek projections, two outward horns and subtle
bone seams. Its held club has a rounded shaft and bifurcated ends.

Marowak has a longer athletic tan body and cream belly with shallow lower
bands, a thicker long tail with a dorsal point, heavy thighs, larger white
toe claws and distinct posed arms. Its skull is longer and narrower with a
sloping brow, sharp narrow eye opening, long tapered snout and pointed side
horns. The reference has a raised gripped bone and an extended opposite arm;
it is not Cubone's body with a taller pair of cones.

Primary Sketchfab model API responses returned HTTP 200 and identified four
specific downloadable CC BY 4.0 candidates. All four archive endpoints
returned HTTP 401:

| Candidate | Creator | Faces | Listed clips | Evidence |
| --- | --- | ---: | ---: | --- |
| [Realistic Cubone](https://sketchfab.com/3d-models/7a105e5e451549b38d1f53cb5ac3f91c) | themarik | 39,938 | 0 | Creator explicitly describes an armature; archive unavailable |
| [Cubone](https://sketchfab.com/3d-models/b5b937603d5a472d8e00bd77c95ca606) | nguyenlouis32 | 4,998 | 0 | Lore description; archive unavailable |
| [Marowak](https://sketchfab.com/3d-models/ba4d232926a24d99921dcdad1e90d2cc) | nguyenlouis32 | 5,068 | 0 | Lore description; archive unavailable |
| [Marowak](https://sketchfab.com/3d-models/5533a9ea411048ddaf7a5f459d93f278) | Chicken0w0 | 5,616 | 0 | Description also reserves rights to Pokemon/Game Freak; listing is not independent provenance |

The [BlenderKit Cubone](https://www.blendkit.com/asset-gallery-detail/2c86b02e-fc56-4e5d-9ae8-b7b8137a8d44/)
by Eleanie is listed as a free fully rigged 3,001-polygon character with
facial controls and EEVEE-specific stylized shaders. Primary asset metadata
returned 200 and confirmed `royalty_free`, `isFree=true`, and a blend-file
download endpoint. The website's Get-model page returned 403. Its
[license explanation](https://www.blendkit.com/docs/licenses/) and
[licensing FAQ](https://www.blendkit.com/docs/licenses/licensing-faq/)
allow higher-level derivative games but restrict directly extractable assets;
this is not a permissive open-source model license for committing the raw
asset into this repository. The download endpoint first required a `scene_uuid`. Supplying a fresh
task-specific scene identifier returned HTTP 200 and a file URL, but the
actual file host returned HTTP 403 with error 1010. No blend file was
retrieved, and actual bone/skin contents remain unverified. A listing's rig
claim alone is not verified skinning.

No external mesh, texture, rig or clip has been imported. Zero listed clips
does not imply no rig. Original sculpting remains available while access and
redistribution suitability are investigated; neither baseline is approved.


The [original Cubone replacement](cubone-sculpture.md) is now installed.
Its 45 closed parts and 94,992 triangles restore the skull recesses, fitted
eyes, juvenile body, paws, fingers and gripped club. The body was shortened
after visual review, and local parent-matched shading removed the fitted
belly/socket seams. A crossed finger, excess triangle density and an outward
normal at a steep socket edge were rejected by meaningful checks and corrected
without relaxing them. Thirteen final candidate checks passed in 36.772
seconds, and final oblique/front/rear software views were inspected in full.

The helmet uses physically recessed closed surfaces, not a claim of a wholly
hollow removable helmet or through-holes. Thirteen installed checks passed
in 33.094 seconds. The final-hash muted native GPU capture exited 0 and was
inspected at 2000×1400. The exact Rust model check passed in 3.10 seconds,
with a frozen final-hash direct rerun passing in 3.41 seconds.

The [original Marowak replacement](marowak-sculpture.md) is also installed.
Its 52 parts and 69,708 triangles provide a narrow recessed skull, long muzzle,
athletic body, raised gripped bone, extended opposite arm and muscular tail
with a dorsal spine. Thirteen candidate checks passed in 27.677 seconds and
thirteen installed checks in 25.797 seconds; the final three software views
were individually inspected. The initial iris was rejected for crossing its
actual narrow parent and corrected. The fresh native build and final-hash muted GPU battle capture both exited 0;
the full 2000×1400 arena image was inspected. The first Rust run encountered an unrelated cave-test dependency error.
After it terminated, a fresh retry using the already corrected manifest
exited 0 with the exact all-species model check passing in 4.70 seconds. A
frozen executable containing both final canonical hashes passed the exact
rerun in 4.83 seconds. All bone-pair jobs are terminal, and task-owned ignored
scratch was removed after the final proofs.
No ordinary live-window, browser, rig, animation or gameplay approval is claimed.
