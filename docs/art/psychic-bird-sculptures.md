# Natu and Xatu planted-talon skins

`tools/psychic_bird_sculpt.py` authors the original birds.
`tools/psychic_bird_glb.py` exports their canonical standard skins through the
shared production reader and GPU path. The battle builder directly exports
these GLBs; parallel static JSON is retired.

Natu retains its round orb, calm fitted gaze, rounded bill, swept lance crest,
barred wing shields on both faces and paired tail feathers. Four large leading
wing coatings use two subdivision levels instead of three: vertices fall from
27,993 to 18,853 and triangles from 44,692 to 28,820. All 42 other parts retain
exact original positions, normals and indices. Actual marking vertices and
front triangle clearance pass at the original offsets. The existing vertex
and index limits remain unchanged.

Xatu retains all 49 parts with exact original positions, normals and triangle
order: the tapered torso, green head, hooked bill, jagged collar, chest symbols,
folded ivory sleeves with layered badges/hem bands, swept head ribbons and
rear stripes. Both birds retain supporting ankles, splayed red toes and ivory
talons, with normalized height 1.04 and a grounded origin.

Natu's ten joints articulate its upper orb, rigid wing units, crest, separate
tail feathers and ankle middles. Xatu's eleven joints articulate its head,
rigid sleeves, two independent two-stage streamers and ankle middles. Each
wing owns all fitted markings on both faces. Xatu's torso, collar, chest
symbols and rear stripes share one spatial field into the head. Fitted gaze
and bill share the head. Toes and talons remain on the identity root; ankles
blend into the body. Idle, attack and hit use rotations only, neutral endpoints
and existing source battle cues. Streamer tips follow their bases with delay.

The local normal RGB555 palette is green (10,23,13), red (20,5,6), white and
black for both birds. Natu's bill uses the warm red. Historical `gold` part
names use the source palette's white in this pack; Xatu's bill is also white.
Shiny checks remain skipped at the user's request.

| Asset | Parts | Joints | Vertices | Triangles | Accessors | Bytes | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Natu | 46 | 10 | 18,853 | 28,820 | 189 | 766,412 | 8 = 8 inches = 0.2032 m |
| Xatu | 49 | 11 | 15,289 | 24,060 | 199 | 593,840 | 411 = 4 feet 11 inches = 1.4986 m |

Neutral digests:

- Natu: `42b91ce42d9cbfb2367f3640cdd1641c5da72aedb65da5cc41b412d9d8b09fd1`.
- Xatu: `5196df1ef836e954c4904ec06cf3448e3a701eba946050d5a38a477d116e785b`.

All seven anatomy and seven decoded-skin checks pass. They cover closed outward
surfaces, stored normals, source regeneration, actual support roots, both wing
faces, fitted marking vertices and triangle clearance, canonical bind/budgets,
normal colors, rigid owners, fixed toes/talons, articulation, winding/strain,
posed collar/chest/rear fitting and delayed streamer tips. Both focused Rust
checks pass, proving GPU activation at pack sizes, coherent owners, fixed
supports and animated bounds. Final Blender/native exports are byte-identical;
front/rear software views were inspected. The native build passes. Both normal native reviews pass. Exact sprite-frame motion, field gaits and
browser visual review remain open.

```sh
python3 tools/psychic_bird_glb.py --species natu --output crates/crystal-voxel-view/models/battle_species/natu.glb
python3 tools/psychic_bird_glb.py --species xatu --output crates/crystal-voxel-view/models/battle_species/xatu.glb
python3 -m unittest discover -s tools -p test_psychic_bird_sculpt.py
python3 -m unittest discover -s tools -p test_psychic_bird_glb.py
cargo test --locked -p crystal-voxel-view psychic_bird_family_ --lib
blender -b -t 2 --python tools/build-battle-species.py -- target/psychic-bird-review --only NATU,XATU
```

Builder exports, source-image helpers and capture intermediates remain ignored
verification scratch. Canonical GLBs and authoring code are the deliverables.


Natu's matched normal native 3D/2D review against Pidgey passes. Its small
pack-based scale, planted toes/talons, round body, swept crest and layered wing
markings read clearly against the retained grass. The paired receipt preserves
core frame 2260, hash 1900440669, LCD frame 2941 and time 53.010732125
(`state_preserved=true`). The native build and wider catalog grounding/actual
elapsed-pose regressions pass. Shiny checks remain skipped.


Xatu's matched normal native 3D/2D review against Pidgey passes. Its tall
pack-based silhouette, planted toes/talons, folded sleeves, fitted chest and
wing markings, hooked bill and swept streamers read clearly. The far streamer
and rear bars are partly obscured at this angle; rear software views cover
them. The paired receipt preserves core frame 2043, hash 2373058012, LCD frame
2672 and time 46.734551292 (`state_preserved=true`). Both normal native reviews,
the native build and wider catalog grounding/actual elapsed-pose regressions
pass. Shiny checks remain skipped; exact motion, field gaits, browser review
and the all-species objective remain open.
