# Original Hitmonlee sculpture and kicking skin

The current [three-fighter fidelity audit](hitmon-fidelity.md) supersedes the
earlier proportion and posture review. `tools/hitmonlee_sculpt.py` authors a
broader headless shoulder mass, outward-rising angry eye corners, heavier
nine-rib upright shin coils, wide three-clawed feet and five-rib wrist coils.
Unequal bent guards end in palms with three curled fingers each. The shaped
heel rolls clear of the floor when the attacking leg begins to lift.

`tools/hitmonlee_glb.py` exports and strictly reads the canonical GLB. Its
33 parts contain 15,445 vertices, 25,744 triangles, 10 joints and 164 accessors
in 639,368 bytes. The source digest is
`ac905c2ea1ed344c5428f270e3ca7237dc8e3d4554d47166b5082b9b46861d54`.
Authoring height remains 1.24; the pack retains 1.4986 m in production.
The local normal palette remains ochre (181,115,41), coil gray (123,132,123),
white and black.

The right support foot stays fixed during battle cues. Left hip and knee
rotation lifts the complete attacking foot, claws and fitted side pad. The
upper gaze follows one owner, wrist coils share continuous arm weights, and
palms and fingers follow their wrists. Coil sampling retains 1,568 unique
positions and 3,132 triangles per coil. Idle, attack and hit clips retain
identity endpoints and production cue timing. Exact move choreography remains
outside this anatomical revision; the audit records sampled coverage.

```sh
python3 -m unittest discover -s tools -p test_hitmonlee_sculpt.py
python3 -m unittest discover -s tools -p test_hitmonlee_glb.py
python3 -m unittest discover -s tools -p test_hitmon_fidelity.py
```
