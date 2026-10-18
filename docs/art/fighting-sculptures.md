# Machop, Machoke and Machamp sculptures and rigs

`tools/machop_sculpt.py` retains the distinct juvenile tail, mature muscular
two-arm wrestler and broad four-arm powerhouse silhouettes. All have three
cranial plates, fitted red gaze, curled fingers and thumbs, broad foot pads
and three toes per foot. Curved limb paths and interpolated torso profiles
replace the coarse segment transitions. Shoulder and hip caps sit completely
inside the torso. Eyes follow the actual head surface, Machoke's red arm
markings follow the arms, and Machamp's broad lips root in the actual muzzle.
The mature trunks and gold waist bands share the torso's rings, avoiding the
gray patches that showed through the older independently sampled garments.
No external geometry or rig is imported.

`tools/fighting_glb.py` exports the canonical standard glTF skins:

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Machop | 32 | 4,377 | 7,288 | 13 | 133 | 226,332 |
| Machoke | 41 | 5,621 | 9,264 | 12 | 162 | 271,936 |
| Machamp | 51 | 6,598 | 10,992 | 18 | 199 | 335,256 |

Authoring heights remain 0.85, 1.15 and 1.45. Verified external pack heights
remain 0.7874 m, 1.4986 m and 1.6002 m. Geometry, primitive, accessor and byte
limits remain unchanged. Neutral geometry digests are:

- Machop: `6847f6ddd211164ff5bdcffb5dc94b104dd87fdd3955610b53b611d728b5d71f`
- Machoke: `4b0b6d1f8f12e160831ba3ec746729bf11fcceffffec0fa4c9ef8932bd8313ae`
- Machamp: `c70bd3ba26597bfadc1fafae6d459a9334b41bd63b22f022578d5a6db4793ca6`

Each arm has its own shoulder, elbow and wrist hierarchy; Machamp has four
independent chains. Hands, fingers and thumbs follow their own wrist. Arm
weights use a continuous longitudinal field, avoiding the discontinuities
produced by choosing the nearest segment of a bent elbow. Machoke's markings
share that field. The continuous body blends pelvis, chest, neck and head;
fitted facial features and cranial plates follow the head coherently. Machop
has an independent tail. Idle, attack and hit clips use the existing source
cue timing and return to neutral endpoints. Foot pads and toes remain fixed
in battle; world walking uses two complete support chains for each species.

The production registry, builder and asset readers use these GLBs. The three
obsolete static JSONs are removed. All 12 combined Python checks pass, covering
closed outward geometry, shading, exact source geometry, strict schemas and
weights, attachment roots with a detached negative control, complete buried
shoulder/hip caps, fitted eye and arm-marking clearance through poses, front
and rear garment coverage, floor contact, local joint articulation, winding,
strain and clip contracts. Five focused Rust checks pass for verified-size
GPU activation, runtime owners and articulation, animated culling and two
complete world walking chains through 64 phases. Front, side and rear poses
of all clips were inspected. Blender exports are byte-identical. Shiny checks
are skipped at the user's request. Browser visual acceptance remains open.

Native Route 29 admission passes at original pack size for all three. The
paired 3D/2D views were inspected at preserved state:

| Pair | Frame | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: | ---: |
| Machop / Rattata | 2249 | 343043607 | 2872 | 50.97786054 |
| Machoke / Sentret | 3168 | 2308503285 | 3853 | 70.069877625 |
| Machamp / Sentret | 2244 | 714168474 | 2851 | 49.662108499 |

Complete silhouettes, coherent red gaze, seated shoulders, clean garments and
correct species HUDs remain readable; all three models' feet meet the terrain.
Machamp's four arms and fists are distinct in the production view. No shader
errors were reported. Final ignored evidence uses `machop-normal`,
`machoke-normal` and `machamp-normal` under `target/pokemon-native-review/`.
The existing supported -110-degree angle is selected with normal size/support/
omission and whole-body/frustum checks active. Grass height is untouched.
Browser visual acceptance remains open.
