# Murkrow sculpture

`tools/murkrow_sculpt.py` authors the original canonical Murkrow mesh. The main
species builder dispatches this recipe before Blender batching and rejects the
legacy generic bird path for Murkrow. The explicit species height override of
0.78 and floor origin remain unchanged; other birds keep their existing paths.

The rounded breast, narrow neck, sly head and folded wings use dark blue crow
plumage throughout, removing the former brown breast. A broad shallow feather
brim slopes over the ivory and red eyes. Three swept crest feathers join the
brim and taper in thickness toward their pointed tips. A broad hooked golden
upper bill, short lower bill, dark seam and subtle nostrils replace the former
simple cone. Four pointed throat feathers bridge the head and breast.

The folded wings have rounded volumes and three overlapping feather vanes per
side. Slender golden shins join grounded toe junctions; each foot has three
forward toes, one rear toe and four white tips. Toe/talon junctions overlap
rather than merely touching at their caps. A short rear stem and narrow red
binding support a broad jagged broom tail fan, with fine radiating creases
fitted to the actual fan surface.

All parts are closed solids, using the native static mesh and base-color path.
The asset does not introduce animation or a rig. No copied meshes or game
textures are included. Silhouette and palette were checked against the
[official Murkrow artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/198.png).
A [CC Attribution Murkrow listing](https://sketchfab.com/3d-models/murkrow-0ead5dd71f5d422a95330fe7433e046c)
was found, but it did not establish a usable original rig for import.

`python3 -m unittest discover -s tools -p test_murkrow_sculpt.py` checks stored
closed topology, winding, normals, exact regeneration, height, triangle budget,
physical support for the body/head/hat/crest/throat, layered wings, bill, tail
stem/binding/fan, grounded feet and every toe/talon junction, plus fitted eye
layers and nostrils. Multi-angle software renders and a native battle capture
check model presentation; the capture is not proof of gameplay behavior.

The final asset has 62 closed parts and 16,616 triangles. All nine sculpture
regressions passed, together with the Rust test covering grounded volumetric
bounds for the 220-species battle catalog, the canonical layout check and storage validation
of 571 JSON documents. Front, rear and oblique software views and the rebuilt
native battle renderer were inspected with the local external content pack.
