# Lickitung source and silhouette review

The previous canonical model was rendered obliquely at 800×824 and inspected
in full. It is a small generic pink body with a separate round head, bead
eyes, short red tube tongue, protruding cream belly, three raised pink
stripes and simple cylinder arms. It lacks a broad open mouth, long flat
pink tongue, cream thigh rings, white limb tips and the heavy curling tail
silhouette. That baseline was rejected for the requested art upgrade.

The [official Singapore Pokémon reference](https://sg.portal-pokemon.com/pokedex/0108/)
was read successfully. Its actual Lickitung image is
[the official artwork](https://sg.portal-pokemon.com/images/pokedex/fe41bccc4b1fd3c2ab748d09a157e3c0c28c7700.png),
retrieved with HTTP 200 and inspected in full. The page also has Lickilicky's
image; it was excluded. The reference shows a stocky pink body and smooth
rounded head, tiny black eyes with white glints, subtle nostrils, a broad
dark mouth, very long broad pink tongue with a center groove, short round
arms with white tips, cream belly bands separated by pink skin, cream rings
on the thighs, white forward toe tips and a large pink tail curving upward
behind the body. These are anatomy references, not assets to ship.

The Sketchfab public search returned two downloadable listings. Each
individual primary API response was checked, including description, license,
face count and animation count. Both advertise CC Attribution / CC BY 4.0:

- [Lickitung by nguyenlouis32](https://sketchfab.com/3d-models/lickitung-36c2dff5b80c44a7ad9f8e1f6e626396):
  5,156 faces, zero listed animation clips. The description contains Pokédex
  text and does not establish an authored rig. The actual download endpoint
  returned HTTP 401, requiring authentication.
- [Lickitung by cristiang](https://sketchfab.com/3d-models/lickitung-c494bd46f158415aa41900e12f6f8c7e):
  4,888 faces, zero listed animation clips and an empty description. The
  actual download endpoint likewise returned HTTP 401.

The primary API for [jeykonunderscore’s Lickitung](https://sketchfab.com/3d-models/lickitung-9e0cc42fc0eb4718aebf914b9f3e1a92)
reports 1,078,952 faces, no downloadable archive and no license object. Its
creator describes a Blender sculpt with no retopology. It is an anatomy
comparison candidate, not a usable open rig.

No archive, mesh, texture, rig or animation was retrieved or imported. Zero
listed clips does not establish absence of bones, and a downloadable listing
alone does not verify usable skinning or provenance. Original geometry is the
available next step: separately shaped stocky anatomy, an actual supported
mouth/tongue, fitted markings and a broad tail. Static software review does
not establish native GPU, ordinary live-window, browser or gameplay behavior.
Task-owned reference and baseline scratch was removed after replacement
verification.


The [original replacement](lickitung-sculpture.md) is now installed. Its
35 closed parts and 108,716 triangles replace the short tube tongue with a
supported broad pink tongue, add a recessed oral lining, cream bands and
thigh rings, white tips and a heavier upward tail. The first detached tongue
and insufficient marking clearance were corrected; all ten final candidate
checks passed in 35.632 seconds. The final three software views were inspected
in full. All ten installed canonical checks passed in 23.016 seconds. The fresh native build and final-hash muted GPU capture exited 0, and the
full 2000×1400 battle image was inspected. Rust integration exited 0 with
the exact model check passing in 3.65 seconds; the frozen final-hash direct
rerun passed in 3.76 seconds. All verification jobs are terminal, and
task-owned ignored scratch was removed after the final proofs. No rig, clip,
ordinary live-window, browser or gameplay approval is claimed.
