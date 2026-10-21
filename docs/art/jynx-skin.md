# Jynx: grounded bell skirt and articulated five-digit hands

The [October 20 fidelity audit](ice-fighter-fidelity.md) records the current
full-orbit, authentic game and front/back sprite comparison. The original
recipe now gives Jynx a broad heavy seven-fold bell skirt, fuller bodice,
rounded face, swept hair curtains and pointed rear mantle. The two open hands
occupy distinct heights and depths following Crystal's idle stance.

The closed head has an actual kiss recess behind a fitted dark lining and
annular pink lips. Eye layers follow the revised skull. Hair markings clip to
the actual parent triangles. Sleeve bends and layered hair roots pass both
self-intersection and occupied-join checks.

The local normal palette stays plum RGB555 (14,2,15), pink (29,6,19), white
and black. Pack height `407` supplies 1.397 m; authoring height stays 1.28.
Physical scaling, collision and admission gates are preserved.

The 22-control rig owns torso, neck, head, two hair curtains, two shoulder/
elbow/wrist chains and ten individual fingers above a fixed hem. Pivots follow
the new anatomy. Gown and throat have continuous spatial fields; face and lips
share the head; each finger blends from its wrist. Clips use rotation-only
source cues with neutral endpoints. Production walking preserves the grounded
hem and canonical upper anatomy.

Coherent hand packing retains all 45 logical anatomy ranges in 35 runtime
parts. The GLB has 18,473 vertices, 30,004 triangles, 220 accessors and 846,100
bytes, within existing limits. Source geometry pin:
`6106c733209f0e0d60a6652f74f4f410e4d15902ff18cae67e0a1ce2e9b6918a`.
Packed geometry pin:
`f9101897d7134dfc74cf912abd9abe6a1936a5a4e84345062c540af1dc837e2b`.
GLB SHA-256:
`7a7a262314186adb5e6d3c8f9a78687c8023a50a6e91b0a94e4af190d7a009ec`.

`tools/ice_dancers_sculpt.py` remains the anatomy source and
`tools/jynx_glb.py` the strict exporter/reader. The regular background builder
reproduces canonical bytes. Nine decoded-skin tests plus shared anatomy and
independent volume regressions check logical ranges, binds, ownership, roots,
fixed hem, posed winding/strain, fitting, palette and mutation rejection.
Smoochum is revised in the same audit; its separate canonical exporter remains
independently verified. See the audit for executed commands and coverage limits.
No external mesh, texture, rig or animation is imported. Nothing is committed.
