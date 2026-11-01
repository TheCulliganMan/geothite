# Legendary bird source review

The normal Crystal forms of Articuno, Zapdos and Moltres need individual
anatomy. A download label or animation count does not establish a usable rig.
Actual geometry, skeleton, deformation and authoring provenance must be
inspected before importing an external candidate.

The [KiwiGamer450 Zapdos metadata](https://api.sketchfab.com/v3/models/cdbb9ba1ba1c42da95924d75e577161b)
advertises downloadable CC BY 4.0 geometry, 5,752 faces and zero animation
clips. Its description explicitly says “NOT BY ME!” and identifies Pokémon
X/Y on 3DS as its source. Its download endpoint returned HTTP 401 during this
review. No geometry or skeleton was retrieved; the label does not establish
that the uploader authored the geometry or can grant its stated license.
This candidate has not been imported.

The [31schrci Moltres metadata](https://api.sketchfab.com/v3/models/b67d6c39263d443daccb10ed2f6c9472)
advertises downloadable CC BY 4.0 geometry, 6,476 faces and zero animation
clips. Its description describes weaknesses and supplies no authoring
provenance. Its download endpoint returned HTTP 401. No geometry or skeleton
was retrieved, and this candidate has not been imported. Zero clips alone
does not prove the absence of a skeleton.

The [Free3D Moltres listing](https://free3d.com/3d-model/moltres-pokemon-35335.html)
explicitly lists “Rigged No” and “Animated No,” with a Personal Use License.
It does not supply a validated open rig for this project and has not been
imported.

The [MiinhoLOL TurboSquid Zapdos listing](https://www.turbosquid.com/FullPreview/1069628)
labels the Blender 2.7 model rigged and provides a free download entry, but
also explicitly limits the license to editorial uses. Its brief description
identifies the Pokémon, without establishing the authoring source. The
geometry, skeleton and deformation have not been inspected; this listing
has not supplied a validated open rig for inclusion in the game.

These observations exclude none of the other possible open sources. Rig
research remains active. The original Articuno sculpture and its render
verification are documented separately in `articuno-sculpture.md`. Zapdos
now has an original individual model documented in `zapdos-sculpture.md`, with
passing stored-mesh checks and inspected front/oblique/rear software images;
its fresh muted production GPU capture passed and was inspected in full. Moltres now has an original replacement documented in `moltres-sculpture.md`,
with passing candidate mesh checks and inspected three-angle software views;
its installed-source tests also passed. Its fresh muted production GPU capture
passed and was inspected in full, with model fingerprint and executable
identity preserved. The all-251 appearance objective remains
unfinished.


The [Space Bird Ho-Oh metadata](https://api.sketchfab.com/v3/models/0203db23a0ec46c08b8121c5cc56d154)
advertises downloadable CC BY 4.0 geometry, 7,484 faces and zero animation
clips. Its description is only “a model of ho-oh.” The download endpoint
returned HTTP 401 in this review; no model file or skeleton was retrieved.
Zero animation clips do not prove the absence of a rig. This candidate has
not been imported, and Ho-Oh still needs an individual battle replacement.
