# Rhydon source and rig review

The [official Rhydon page](https://sg.portal-pokemon.com/pokedex/0112/)
returned 200, and its named primary artwork was individually inspected in a
background image-only tab. It shows a heavy grey biped with a broad muzzle,
cream spiral nasal horn, two tall ears, jagged cheek/neck armor, narrow red-eyed
gaze, open fanged mouth, cream horizontal belly plates, stocky thighs,
clawed arms, two large claws per foot and a thick segmented tapering tail.

The current canonical model's 800×824 oblique software render exited zero and
was fully inspected. It has a small oval head, protruding round cartoon eyes,
plain cone horn, two thin polygon ears, geometric fists without digits, a flat
oval cream belly without plate divisions and flat oval feet without claws.
The wide muzzle, open mouth, neck armor, muscular stocky limbs and segmented
tail silhouette all need individual replacement. No canonical geometry was
changed during this source review.

Specific primary Sketchfab metadata and archive endpoints were inspected:

- [IronGut KM Rhydon](https://api.sketchfab.com/v3/models/77a96e79e6594578a22fc5adb58ea2e1):
  downloadable, CC BY 4.0, 44,420 faces and zero listed animation clips.
  Its Sketchfab archive endpoint returned 401. The creator links Blender,
  FBX and avatar files through [their own asset page](https://irongut1.itch.io/km-rhydon),
  which also explicitly permits reuse. That page identifies a public Drive
  folder; the publicly downloadable `KM Rhydon.zip` returned 200, 13,311,309
  bytes, SHA-256
  `3965afd31bff0dfe5956586bc37b208eb9c66d85c9bc2bbf131998554c5fb08d`.
- [nguyenlouis32 Rhydon](https://api.sketchfab.com/v3/models/c5f0af61e7aa44e7a8324de4f787bbb6):
  downloadable, CC BY 4.0, 5,232 faces and zero listed clips; archive 401.
  Description is species lore rather than verified authorship evidence.
- [Chechorams16 Rhydon](https://api.sketchfab.com/v3/models/66f9b2cf2cc44281935721c9f7bc2e42):
  downloadable, CC BY 4.0, 5,232 faces and zero listed clips; archive 401.
- [exodialeader Rhydon Idle Animation](https://api.sketchfab.com/v3/models/221814ed2d434560869569b55fcada91):
  one listed clip, not downloadable and no exposed license; archive 401.

The accessible creator archive was inspected as data without executing it.
Its FBX 7400 binary contains eight Mesh objects, 97 LimbNode objects, eight
Skin deformers, 592 Cluster deformers, seven BlendShape deformers and 51
BlendShapeChannel deformers. There are no AnimationStack or AnimationCurve
nodes. Thus this is actual accessible rig/skin evidence, not an inference from
the listing or an animation label. Skin weight quality and deformation have
not been rendered or approved.

The creator's full 1920×1080 static model image was individually inspected.
It is a humanoid avatar with large human-like pectoral/arm/leg muscles,
human-like hands, clothing and proportions substantially different from the
official species. The FBX confirms separate underwear, shirt, pants, socks and
adult accessory meshes. This candidate is excluded from the standard roster
for anatomical fit, despite its accessible rig and permission to reuse. No
mesh, texture, rig or animation from it is installed or shipped. Downloaded
verification scratch is removed after this review.

The other checked archive endpoints did not expose model contents. These
specific checks do not establish that no appropriate open rig exists.
