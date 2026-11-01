# Rhydon sculpture and rig

The current [full-turnaround fidelity review](rhino-larva-fidelity.md) corrects
the raised palms and forward-sloping shins. `tools/rhydon_sculpt.py` authors a
heavy upright barrel, lower cream belly bands, large skull, thick pointed ears,
six cheek points, narrow fitted red gaze, forward nasal drill and a backed open
jaw. Unequal lowered hands carry three forward/downward claws each. Short shins
stand over full heels and two claws per foot. Flattened back points and three
restrained gray tail collars complete the original curved tail. Reference game
geometry, textures and rigs are not imported into the asset.

`tools/rhino_glb.py` packs 64 authored solids into 48 runtime material groups:
17,699 vertices, 27,280 triangles, 15 joints, 255 accessors and 755,720 bytes.
Authoring height remains 1.40; pack height remains 1.905 m. The logical digest is
`d83099e8c19e82f3cdadd99bd79bb56bad29655c8a36e12f2bea81d2e715d3b5`;
the packed digest is
`3f9909391c0d4fbb4722e12c76dc927b2302ce579e40a5e6c9c3f6e2e20ea9cb`.

Both arm chains blend by their actual bent paths; distal forearms, palms and
claws meet their wrist carriers. Head, jaw, ears, drill, tail and leg pivots
follow the revised anatomy. Tail and collars share source-arc weights. Pads and
claws retain source-clip floor contact; complete hind supports participate in
world walking. Idle, attack and hit retain source-cue clocks and neutral ends.

The fidelity review records inspected references, geometry and motion checks,
production evidence and coverage limits. Mouth probes derive the empty gap from
the current stored lips, verify occupied backing and follow the head carrier.
No geometry, floor, scale, admission or loader tolerance is relaxed. The regular
Blender builder reproduces the canonical GLB exactly; no pack rebuild is needed.
