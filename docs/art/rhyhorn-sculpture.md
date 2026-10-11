# Rhyhorn sculpture

`tools/rhyhorn_sculpt.py` replaces the old generic quadruped with an original
heavy gray barrel, broad lowered wedge muzzle, rooted nasal horn, pointed ears,
a lower armored jaw, narrow fitted eyes, dark nostrils, four stocky legs,
tapered knee cuffs, broad grounded feet, twelve supported ivory claws and a
short rooted stone tail. Four thick pentagonal dorsal plates intersect the
actual barrel; a central forehead crest intersects the actual head.

The native roster review exposed the old ridge floating above the torso.
The first replacement passed root tests but its smoothed plates were almost
buried in the body. Front, oblique and rear previews guided taller plate peaks
that retain solid roots and express the armored silhouette.

The final stored model has 44 parts and 24,984 triangles, grounds at zero and
preserves authoring height 1.11. The main builder routes Rhyhorn through the
pure Python recipe and rejects its obsolete Blender quadruped route. Other
rock species are preserved. Its SHA-256 prefix is `0d1c4918c5e06ce0`.

All ten stored-mesh tests passed. They cover closed connected outward parts,
finite unit normals and actual face winding, recipe parity, height, grounding,
budget and identity counts, actual solid roots for every plate, head, jaw,
horn, ears, legs, cuffs, feet and tail, supported fitted face/claw details and
front-face clearance and raised armor silhouette. A shifted-plate negative
control catches detached armor.
Front, oblique and rear software previews were inspected. Layout validation and
storage validation for 569 raw JSON model documents passed.

The [official species page](https://sg.portal-pokemon.com/pokedex/0111/)
was consulted for identification; no reference artwork is shipped. The
[3dlogicus listing](https://sketchfab.com/3d-models/rhyhorn-pokemon-d976c6a719be46e3a4efda785c5f45b0)
offered no download or rig information. No external geometry was imported;
this original sculpture is static. The full 251-species appearance goal and
accessible reusable rig search remain active.

The fresh Rust all-251 loaded-model test passed with zero failures. The native
`immersive_battle_3d` example then built successfully. An explicit GPU image
capture using the ignored realtime-clock browser pack exited zero and produced
a 2000×1400 PNG with 43,992 colors. The entire actual image was inspected:
Rhyhorn has seated visible dorsal armor, grounded feet, a clear face and horn,
and shares the visible arena with Pidgeotto and the production battle HUD.
This establishes this battle preview, not gameplay or animation completeness.

Ordinary window captures both before and after the successful fresh build
remained entirely black (one color, zero luminance range; exit one).
Live native-window inspection was attempted through computer control, but the
Mac was locked and could not be unlocked automatically. The independent
image-target capture does not resolve that window-capture failure.


Temporary candidate meshes, previews, native PNGs, logs, the inspection app
wrapper and authoring/test bytecode were removed after the jobs completed.
