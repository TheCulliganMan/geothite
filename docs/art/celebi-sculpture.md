# Celebi swept-onion fairy and skin

`tools/celebi_sculpt.py` authors an original dedicated fairy sculpture guided by
[the official Celebi artwork](https://sg.portal-pokemon.com/pokedex/0251/).
`tools/celebi_glb.py` exports its canonical standard glTF skin at
`crates/crystal-voxel-view/models/battle_species/celebi.glb`. The common Blender
battle-species builder regenerates the same bytes. The old 22-part generic
sphere/cone/diamond-wing JSON was removed. No reference pixels, third-party
geometry or copied game models are included.

A continuous large green onion bulb sweeps backward into its curled point.
Three shallow folds follow the stored head surface. Two thin curved feelers
have blue distal segments. The eyes use fitted charcoal sockets, ivory lenses,
large blue irises, deep-blue pupils and small white highlights. A fitted smile
sits above the small green torso and pale leaf collar. Reaching slender arms
end in soft palms with three pointed fingers. Two closed pale wing leaves root
behind the shoulders. A darker rounded abdomen leads to short shanks and
rounded green drop-shaped feet. Materials remain matte and opaque.

The initial sculpt's point was too tall and narrow; it was shortened and swept
backward before acceptance. The old boot shape was replaced with rounded
feet. A self-intersection test found a crowded final antenna control interval;
removing that interval resolved the loop without loosening the test. Blue tips
retain actual occupied roots in their green feelers.

The canonical model has 38 named closed parts, 14,678 vertices, 23,056 triangles,
11 joints and 181 accessors in 616,320 bytes, within unchanged renderer limits.
Its source/neutral SHA-256 is
`94a8a577208273651ba7294a48345f701b30142a260b5fc2e920b21e252837fb`.
Authoring height is 0.98 with minimum zero. The verified user-supplied local
pack gives height digits 200, or 0.6096 metres (2 feet), classification TIMETRAVEL.

The identity root fixes both feet. Blended shanks bridge to the yawing torso.
The complete head carries its fitted gaze, mouth and folds. Each hand and all
three fingers share an arm carrier. Each pale wing has a separate shoulder;
each feeler and its blue terminal share an independent antenna carrier.
Idle has six wing-flutter cycles over 3.2 seconds. Attack braces early and
recoil settles through delayed head, hand and antenna motion. All clips contain
rotation keys only, use source cue timing and return literally to neutral.

Ten independent stored-model checks pass for canonical parity/schema, closed
outward solids, normals, fitted front triangle/edge clearance, occupied roots,
a detached-tip negative control, proper ownership, fixed feet, posed floor,
winding/strain, independent motion and neutral endpoints. Rust checks pass for
dedicated-model routing, coherent gaze/fingers/blue tips, fixed feet, independent
carriers, culling bounds and GPU activation at verified pack size. Blender
regeneration is byte-identical. Fixed-camera front/rear neutral, peak flutter,
attack and recoil views were inspected. Shiny checks are skipped as requested.

```sh
python3 tools/celebi_glb.py
python3 tools/test_celebi_glb.py
```


The wider exact-species grounding/volume and elapsed-idle/source-attack timing
regressions passed. The rebuilt production native Route 29 encounter against
Hoppip passes terrain admission (`accepted=true`). Its inspected 3D/2D pair
preserves state at core frame 1525, checksum 2305946238, LCD animation frame
2116 and virtual time 36.53826475. The swept head, dark/blue fitted gaze,
blue feeler tips, small hands and pale wings remain readable beside the source
grass at verified pack size. Rounded feet meet the field; the original forest
and grass remain in place. No shader errors were reported. This verifies the
held native view; browser visual acceptance remains open. Verification files
use prefix `target/pokemon-native-review/celebi-normal` and remain ignored.
