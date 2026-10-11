# Gligar flying scorpion sculpture

`tools/gligar_sculpt.py` authors the original canonical Gligar mesh. The battle
species builder dispatches it before Blender batching. The species keeps its
1.06 presentation height and floor origin. The asset has 32 closed parts and
25,728 triangles.

The rounded pink torso and broad flattened head carry long swept pointed ears,
fitted ivory triangular eyes, narrow dark pupils and tiny ivory catchlights.
A broad fitted mischievous mouth frames a protruding rounded pink tongue with
a fitted central crease and two curved ivory fangs.

The asymmetrical raised arms support thick hooked crescent pincers. Their open
centers and jaws are actual empty space between tapered hooks, replacing the
old balls and small triangular fingers. Two blue scalloped gliding sails attach
to the torso and curved pink upper struts. Bent rear haunches join grounded
rounded pads and sharp curved forward talons. A continuous curled rear tail
joins a darker rounded venom bulb and swept poison barb.

Silhouette and palette were checked against the
[official Gligar artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/207.png).
External research found a CC Attribution
[Gligar listing](https://sketchfab.com/3d-models/gligar-cff27e7a06ce478a9c85a1abda041974),
whose metadata did not establish a usable original rig. An animated
[Nintendo 3DS listing](https://sketchfab.com/3d-models/nintendo-3ds-gligar-a08cd22b43ef419caed61508370fdf5c)
identified game content rather than an original authored asset. No external
mesh or textures were imported. This original static sculpture uses native
base colors; it does not claim rigging or animation.

`python3 -m unittest discover -s tools -p test_gligar_sculpt.py` checks closed
connected topology, outward winding, finite unit normals, deterministic recipe
parity, presentation height, budget, anatomy counts, actual open pincer gaps,
all arm and claw roots, blue sail/strut roots, grounded leg/pad/talon joins,
tongue and fang roots, the complete venom tail, fitted facial vertices and
front triangle clearance. Front and rear software views and a native battle
capture assess model presentation rather than gameplay behavior.

Final verification passed all ten Gligar geometry regressions. Front, rear and
battle-angle software views and the final native battle capture were visually
inspected. The Rust catalog regression passed for the 220-species battle catalog; it checks
grounding and volume rather than visual quality. Layout and storage validation
passed for all 571 JSON assets. Temporary reference artwork, review renders and
logs were removed after inspection.
