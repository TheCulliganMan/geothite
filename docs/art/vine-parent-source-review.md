# Tangela and Kangaskhan source review

The current canonical pair was rendered at 1600×824 and inspected in full.
Tangela is a dark upright egg with repeated blue bent rods, protruding bead
eyes and tall red boot stems. It needs a rounded tangled vine silhouette,
layered loops around a clear dark eye opening, tiny black pupils inside white
eyes, and broad red shoes. Kangaskhan is a brown generic upright toy with
pointed ears, two forehead cones, protruding eyes, a cream oval badge and a
purple baby projecting from that badge. It lacks the reference's heavy
parent anatomy, open toothed muzzle, skull brow, real pouch, broad clawed
hands/feet and articulated-looking shoulder/thigh plates. Neither baseline
is approved for the requested art upgrade.

The [official Tangela page](https://sg.portal-pokemon.com/pokedex/0114/) and
[official Kangaskhan page](https://sg.portal-pokemon.com/pokedex/0115/) were
read. Their main artwork was downloaded with HTTP 200 and inspected in full:
[Tangela artwork](https://sg.portal-pokemon.com/images/pokedex/e6fdba2d3e167f34a8a75eb7ed3965fb3ddd4df8.png)
and [Kangaskhan artwork](https://sg.portal-pokemon.com/images/pokedex/c959a88d4dd9e2007d466535100ae40439d8ff82.png).
Neighboring and Mega-form images were excluded. These references are not
shipped assets.

Tangela's blue vines curl and overlap into a rounded mass, framing a dark
opening and two white oval eyes with tiny high black pupils. Its red shoes
are broad rounded pads; long upright red legs are not the main silhouette.
Kangaskhan has a heavy gray-brown parent body, a large cream belly and actual
cup-like pouch, a small pale-purple baby leaning over its rim, a broad long
toothed open muzzle, a dark skull brow, cream ear interiors, sturdy raised
arms with three white hand claws, heavy thighs with plates, broad white-toed
feet, back ridges and a substantial pointed balancing tail.

Six individual primary Sketchfab APIs returned model metadata. Each listing
advertised CC Attribution / CC BY 4.0 and downloadable status. Each actual
download endpoint returned HTTP 401:

- [Tangela by PrufDesigner](https://sketchfab.com/3d-models/tangela-4744458a88ac4979ada265c776c929d3):
  1,432 faces, zero listed clips, description identifies Pixelmon provenance.
- [Tangela by nguyenlouis32](https://sketchfab.com/3d-models/tangela-e0ba16a9cae94102bdb92507fbc452e2):
  5,382 faces, zero listed clips, Pokédex-text description.
- [Kangaskhan by nguyenlouis32](https://sketchfab.com/3d-models/kangaskhan-f5c2b702c010417c9268b11f6dcf634d):
  6,454 faces, zero listed clips, Pokédex-text description.
- [3DS Pokémon X/Y Kangaskhan by siouxsiepikachu](https://sketchfab.com/3d-models/3ds-pokemon-x-y-115-kangaskhan-dd15c24d711a47cabaf1fe4c7473f374):
  6,454 faces, one listed clip, empty description; the title identifies game
  provenance, and the uploader's license alone does not establish authorship.
- [Kangaskhan ma boi by medothegeek](https://sketchfab.com/3d-models/kangaskhan-ma-boi-cd643befa3c64f13a708d29ca6fe2f99):
  6,280 faces, one listed clip, empty description.
- [Kangaskhan by medothegeek](https://sketchfab.com/3d-models/kangaskhan-be2a8a0d586d4d01887ce8c7460e33d9):
  6,280 faces, 25 listed clips, empty description.

No archive, mesh, texture, skeleton or animation was retrieved or imported.
Listed clips do not verify usable rig data, and zero listed clips does not
prove absence of skinning. Canonical models remain unchanged. Original
anatomy is the available next step unless a suitable accessible rig is
retrieved and verified. Geometry, attachments, three-view art review, muted
final-hash GPU captures and actual Rust integration are still required.
Task-owned ignored reference/baseline scratch stays while work continues.
