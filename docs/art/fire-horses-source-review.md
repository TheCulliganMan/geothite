# Ponyta and Rapidash source review

Reviewed 2026-10-12 against the current canonical pair and primary sources.
The paired oblique software baseline rendered successfully at 1600×824 and
was inspected in full. Both baseline meshes used the same stout toy body,
cylindrical neck and legs, oversized protruding round eyes, pale oval feet
and disconnected-looking triangular flame clusters. Baseline Rapidash added a plain
cone horn but otherwise does not convey its taller athletic proportions.
These are the inspected baseline defects. Both horses now have installed original sculptures:
[Ponyta](ponyta-sculpture.md) has twelve passing canonical checks and a
final-hash muted GPU battle capture inspected in full. [Rapidash](rapidash-sculpture.md)
has fourteen passing candidate and canonical checks, reviewed final software
views and an unchanged exact Rust species route. Its native GPU battle capture was inspected in full, and a direct Rust test
with both final horse SHAs passed once in 3.81 seconds. The native preview
was explicitly muted. These checks cover static models and image-target
capture, not live-window, browser, animation or gameplay behavior.

## Official anatomy references

The official [Ponyta entry](https://sg.portal-pokemon.com/play/pokedex/0077/)
and [Rapidash entry](https://sg.portal-pokemon.com/play/pokedex/0078/) both
returned HTTP 200. Their named primary artwork was opened as silent static
images in background browser tabs and inspected in full.

[Ponyta artwork](https://sg.portal-pokemon.com/images/pokedex/dacab2be1777c14ed7da12824dd85c2cdbd2ebf9.png)
shows a youthful cream pony with a continuous arched neck/chest, compact
horizontal torso, slender four-legged stance, dark blue hooves, long modest
muzzle, large dark fitted eyes with white glints and pointed cream ears with
dark inner faces. It has no horn. The orange/gold forelock sweeps up and back;
its neck mane, tail and lower-leg tufts have curling lobed flame contours,
with brighter yellow inner fire. Separate triangular spikes do not capture
those connected flowing shapes.

[Rapidash artwork](https://sg.portal-pokemon.com/images/pokedex/cb874883fcffc227e8d065e275221e1e05ece46c.png)
shows a taller cream athletic horse with a long arched neck, proportioned
horse head, narrow red gaze, long forehead horn, pointed ears and dark blue
hooves. One foreleg is raised and bent while three support the body. The
orange/yellow mane streams behind the neck and the tail spreads into a long
lobed flame plume. Broad leg fire curls around the lower limbs. Rapidash
needs distinct adult proportions, facial expression and fire volume, rather
than the juvenile mesh with an added horn.

## External candidates and actual access

Specific primary model API responses returned HTTP 200 and identify these
candidates as downloadable under CC BY 4.0:

| Candidate | Creator | Faces | Listed animation clips |
| --- | --- | ---: | ---: |
| [Ponyta - Pokemon](https://sketchfab.com/3d-models/e4e73f5a6b174e2d8fa957d7b720c7aa) | Locur | 17,294 | 0 |
| [Ponyta](https://sketchfab.com/3d-models/32d8c4eb33a84f9bb3ea6dc184632b49) | Artlolo | 14,490 | 0 |
| [Rapidash](https://sketchfab.com/3d-models/1bbe936d79fc4f7e9f6fa7abeb32154f) | nguyenlouis32 | 5,841 | 0 |
| [Rapidash SMOOTH](https://sketchfab.com/3d-models/0004cc40a54c4a709942743dded59165) | oceanhf | 5,668 | 0 |

All four primary `/v3/models/<uid>/download` endpoints returned HTTP 401.
No archive was retrieved, so bones, weights, deformation and actual asset
quality have not been inspected. Zero listed animation clips does not prove
that a model lacks a rig. Other search results included Galarian forms,
block-style Cobblemon assets and print statues; none establishes an accessible
rig matching these Crystal species. No external mesh, rig, texture or clip
has been imported. Original replacements need actual anatomical roots,
closed finite geometry, fitted facial layers, reviewed final views and
final-hash muted GPU verification through the unchanged species routes.
