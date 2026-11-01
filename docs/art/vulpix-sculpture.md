# Vulpix sculpture and rig

The generic orange fox is replaced by an original russet six-tailed fox with
three curled orange forelock waves, broad dark brown gaze and eye sparks,
pointed ears with dark fitted insets, a small muzzle and smile, a fitted cream
chest bib, rounded haunches and four planted dark paws. Six full orange plumes
rise into a layered curl fan. The visual reference is the
[official Vulpix artwork](https://sg.portal-pokemon.com/pokedex/0037/).
No reference pixels are shipped.

`tools/vulpix_sculpt.py` authors 48 closed parts, 13,624 vertices and 22,736
triangles. `tools/vulpix_glb.py` exports a 597,188-byte canonical GLB with
227 accessors and 17 joints at authoring height 0.85. The neutral digest is
`94d20844527dddce0a41783dddcff88ca7f08abff9b6d2e7aa86aa25dcaa6817`.
The verified local pack supplies height digits 200 (0.6096 metres), FOX.
Original pack size is retained.

The rig provides body/neck/head carriers, independent ears, a shared tail-fan
carrier, six independently blended curl carriers and four softly blended limbs.
Face and forelock remain coherent with the head; the cream bib uses the same
height-based neck blend. Paws and toe creases stay on the neutral root. Each
tail blends continuously from torso to common carrier to its own curl joint.
Idle, attack and hit retain neutral endpoints and source-cue timing.

The first tail candidate was too thin and open; fuller plumes and tighter
coils correct the silhouette. The initial cream bib cap clipped into its
curved parent; conforming local subdivision clears the actual neck triangles.
Nine Python checks pass: canonical bytes/schema limits, closed outward solids,
actual normals, fitted gaze/ears/bib, anatomical owners, occupied roots with a
detached negative control, all six curls articulating independently from the
fan carrier, floor/endpoints/winding/strain/independent motion and cue metadata.
Front, side and rear poses of all three clips were inspected. Blender generation
is byte-identical. Both focused Rust rig/GPU checks pass. Shiny verification
is skipped at the user's request.

Rebuild with `python3 tools/vulpix_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_vulpix_glb.py`.

The rebuilt native Route 29 walking encounter against Pidgey passes terrain
admission at original pack size. The inspected 3D/2D pair preserves core frame
3540, checksum 2552778670, LCD animation frame 4244 and virtual time
77.847123207. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing establishing
angle with normal support, omission, scale and full-body/frustum checks.
The russet body, brown gaze, orange forelock and curl fan, cream bib and grounded
dark paws remain readable with matching VULPIX HUD labels. No shader errors
were reported. Both shared grounded-volumetric/model and exact source-cue
timing regressions pass. Final ignored evidence uses
`target/pokemon-native-review/vulpix-normal`; browser visual acceptance
remains open.
