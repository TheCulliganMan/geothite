# Aerodactyl sculpture

`tools/aerodactyl_sculpt.py` replaces the shared generic dragon with an original
lean fossil flyer. A continuous chest and slender neck support a swept wedge
skull, backward horns and a lowered long mandible with rooted cheek hinges.
Twelve ivory teeth surround actual empty jaw space, rather than a painted mouth.
The dark palate and lower lining fit the stored jaw surfaces; a long rose tongue
follows the mandible. Narrow fitted ivory eyes and nasal slits replace protruding
spherical eyes.

Two broad violet scalloped sails attach to the torso and gray bent leading arms.
Each sail has branching seated fingers and three hooked wrist digits. Bent hind
limbs join grounded narrow pads and six ivory talons. A continuous swept slender
tail terminates in a thick arrowhead oriented for a visible front/oblique/rear
silhouette. The first arrowhead appeared edge-on and was revised after inspection.
The first eye fit crossed a skull fold; lateral placement and finer fitting now
clear the actual stored skull triangles.

The model contains 56 parts and 31,516 triangles. It preserves presentation height
1.30 and grounds at zero. The final stored candidate passed all twelve tests in
17.536 seconds: closed connected outward anatomy, finite unit normals and actual
face winding, recipe parity, grounding, height, budget and unique species parts,
real skull/jaw/cheek/horn roots, actual open-jaw clearance across all stored
parts plus a closed-mouth
negative control, tooth roots, fitted face/palate/tongue support and front-face
clearance, sail/arm/finger/wrist digit roots, grounded leg/pad/talon joins,
continuous tail/arrow joins and projected arrowhead area. Final front, oblique
and rear software previews were inspected. The canonical bytes have SHA-256
prefix `4117d1fae62a9502`. The main builder dispatches the pure recipe and rejects the
obsolete Aerodactyl dragon route; Dragonite's existing construction is preserved.

The [official artwork](https://sg.portal-pokemon.com/images/pokedex/0fa47ef9ac7be2f3c447e538152c69e9e2bd3720.png)
was inspected in the browser as a temporary reference; it is not shipped.
The [nguyenlouis32 model](https://sketchfab.com/3d-models/aerodactyl-553b955228aa4546a6050842ffda4e6f)
reports downloadable CC Attribution geometry, 5,488 faces and zero animation
clips through its [public metadata](https://api.sketchfab.com/v3/models/553b955228aa4546a6050842ffda4e6f). Zero clips does not establish absence of a
skeleton. Its download endpoint returned HTTP 401, so geometry, skeleton and
source provenance could not be inspected. No external model was imported. This
original sculpture is static; the full 251-species appearance and usable rig
search remain active.

Layout and storage validation passed for all 569 raw JSON model documents.
The installed canonical mesh passed all twelve tests again in 11.063 seconds
after strengthening the empty-mouth check to inspect every stored part.
The native example built successfully, then an explicit GPU image capture using
the ignored realtime-clock browser pack exited zero. The actual 2000×1400 PNG
was inspected at full frame: the long skull, open toothed jaw, rose tongue,
scalloped violet sails, gray wing fingers, arrow tail, grounded feet, opposing
Pidgeotto, arena scenery and production battle HUD are visible. It contains
38,175 colors. This is production battle-preview evidence, not gameplay or
animation completeness.

The fresh Rust loaded-model regression passed for exactly 251 species, with
one actual test passed and zero failures. Its scope is loaded bounds, grounding
and normals; visual approval uses the inspected images above. Live native-window
inspection was attempted with a temporary application wrapper, but computer
control reported the Mac remains locked and cannot be unlocked automatically.
The successful image capture does not resolve the separate ordinary window
capture failure documented in the native review.


Temporary candidates, software previews, native PNGs, logs, the inspection
application wrapper and task-generated authoring/test bytecode were removed
after all verification jobs completed. The shared build cache is retained.

The later [canonical skin publication](aerodactyl-skin.md) preserves this source
geometry exactly and adds skull, jaw, wing, hind-limb and tail controls. Current
captures use the headless image path; the historical window limitations above
do not describe that path.
