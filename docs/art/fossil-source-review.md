# Fossil Pokémon source and anatomy review

The earlier baseline findings below describe the original shared recipe.
Omanyte and Omastar now have [dedicated nautilus sculptures](nautilus-sculptures.md).
Kabuto and Kabutops now have [dedicated original skins](fossil-scythe-skins.md),
followed by a [full-volume Kabutops/Aerodactyl revision](fossil-pair-fidelity.md).
Consult those reviews for current verification and remaining production checks.

The four canonical fossil models still use the generic `fossils()` batch
recipe. Two paired oblique baseline renders each exited 0 at 1600×824 and
were individually inspected in full. Omanyte and Omastar share a smooth
cream oval shell, prominent brown front coil, blue bead-eyed body and
repeated bent tentacle rods; Omastar mainly adds shell cones. Kabuto has
a faceted brown dome, exposed red bead eyes and straight cream leg cones.
Kabutops has a visibly detached oval head, two head triangles, bead eyes,
an upright toy torso, three raised belly rods, blunt feet and diamond blades.
These baselines are not approved art.

## Official reference

The official pages for [Omanyte](https://sg.portal-pokemon.com/pokedex/0138/),
[Omastar](https://sg.portal-pokemon.com/pokedex/0139/),
[Kabuto](https://sg.portal-pokemon.com/pokedex/0140/) and
[Kabutops](https://sg.portal-pokemon.com/pokedex/0141/) were read.
Their actual main artwork PNGs were fetched with HTTP 200 and individually
inspected in full; the legacy Kabutops page's circle background was a 404
and was not mistaken for character artwork. The main reference paths are:

- Omanyte: `images/pokedex/1f2291efe7407a5585b30792e7b81829cc120a0b.png`
- Omastar: `images/pokedex/c9b99543b14fc3859bb695d3f45d2e38163aacc0.png`
- Kabuto: `images/pokedex/f0757ab6ec3fa6574202a4d1646e2967bbe6fd03.png`
- Kabutops: `images/pokedex/b27f8689027a59c36e875834453df03f4b200ce9.png`

Omanyte needs a ribbed cream shell with a side spiral, soft blue mantle,
large circular white/black gaze and tapered curling tentacles, including
two raised beside the eyes. Omastar is a distinct spiked adult shell and
blue mantle with raised and spreading tentacles, yellow slit eyes and
an actual dark cross-shaped opening among four pale beak segments.
Kabuto needs a broad brown rimmed shell, two tiny upper-shell markings,
a dark recessed underside, round pink-red underside eyes and four hooked
pale claws. Kabutops needs a broad pointed flattened head shield rooted
to the body, narrow gaze, segmented pale chest armor, two long curved
white scythes, bent digitigrade legs, white toe claws, tapered tail and
rear armor fins. The upright toy silhouette and floating head must go.

## Individual external candidates

Public Sketchfab searches were followed by actual individual model API
reads and `/v3/models/<uid>/download` requests. All eight metadata reads
returned HTTP 200; all eight download endpoints returned HTTP 401.

| Candidate | Uploader | Faces | Listed clips | Listed license |
| --- | --- | ---: | ---: | --- |
| [Omanyte](https://sketchfab.com/3d-models/59c42a61405741a1a86b698f08cdd377) | nguyenlouis32 | 4,766 | 0 | CC BY 4.0 |
| [Omanyte doodle](https://sketchfab.com/3d-models/049aca5e33894ed69baf52490ef6b7d1) | won1 | 42,227 | 0 | CC BY 4.0 |
| [Omastar](https://sketchfab.com/3d-models/b894d3e02997467bb3be7dacfa495efb) | nguyenlouis32 | 5,914 | 0 | CC BY 4.0 |
| [Kabuto](https://sketchfab.com/3d-models/db77258448bc4d5fa77642a02968d71e) | nguyenlouis32 | 3,060 | 0 | CC BY 4.0 |
| [Kabuto Pokémon](https://sketchfab.com/3d-models/58f651e036e7476689265eb95dca6073) | greencookieicing | 3,203 | 0 | CC BY 4.0 |
| [Kabutops](https://sketchfab.com/3d-models/3575e52f45054996b80fc5ba23002b89) | nguyenlouis32 | 5,590 | 0 | CC BY 4.0 |
| [3DS Pokémon X/Y Kabutops](https://sketchfab.com/3d-models/a295c25072054a0eafb3baf4678b46db) | Cubone0711 | 5,590 | 0 | CC BY 4.0 |
| [Kabutops](https://sketchfab.com/3d-models/4442830770d84a52bce29eeb7be2df84) | yehun9714 | 24,929 | 0 | Free Standard |

The nguyenlouis32 descriptions give Pokédex-style text rather than mesh
authorship evidence. The won1 description identifies an original Feather
sketch and tutorial. The greencookieicing description identifies Pixelmon
content, and the Cubone0711 title identifies a 3DS game source. The
yehun9714 description is empty and its Free Standard license is distinct
from CC BY. These uploader labels alone do not verify source authorship.
None of the archives, textures, skeletons or animation clips was retrieved
or imported. Zero listed clips does not prove an absent skeleton, and the
failed endpoints do not prove that no accessible rig exists elsewhere.
The paid high-resolution Omanyte listing and unrelated samurai-helmet
search matches do not provide a free Pokémon rig.

Original recipes are the actionable next path with the evidence currently
available. The four fossil canonical meshes were unchanged at that source-review
checkpoint; the nautilus update linked above records the later replacements.
Reference PNGs and baseline images remain ignored and are removed after review.
