# Bilateral walking support packing

Lickitung, Mr. Mime, Scizor and Scyther now keep bilateral support details
separate for the production walking rig. The prior GLBs combined left/right
thigh markings, hip/knee shells or toe details into runtime primitives that
walking IK assigns to a single leg. The mantises also combined shin armor and
rigid foot pads under one unrecognized greave category. Four stored-asset
regressions fail against those previous assemblies and pass after the split.

Lickitung's thigh rings and centres and Mr. Mime's hip/knee shells remain
separate per side. Mantis greaves and foot pads retain their own runtime
primitives and walking roles. Scizor's toe caps stay per foot; Scyther's three
claws are grouped within each individual foot. Walking classification now
recognizes hip and greave categories. The shared change is covered by the
entire locomotion test module. Its roster sweep visits 251 species and 26
Unown selectors, checking bind identity, attack/hit motion and finite unit-scale
locomotion poses at five phases. The new four-species support test supplies
the stronger per-vertex floor/bounds and ownership assertions described below.

The new four-species production regression checks the actual gait tree of
every affected support, one complete leg per side, rigid foot/toe carriers,
ground contact and bounds at 64 locomotion phases. Canonical battle rigs and
clips retain their original geometry and ownership. Independent old/new GLB
decoding confirms identical logical f32 geometry/colors, normals, indices,
weights, hierarchy, inverse binds and clip keys, plus identical sampled battle
positions and normals across idle, attack and hit.

| Species | Logical solids | Runtime primitives | Vertices | Triangles | Canonical joints | Accessors | GLB bytes | Reference height |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Lickitung | 33 | 26 | 17,028 | 29,256 | 18 | 167 | 760,140 | 1.18 |
| Mr. Mime | 67 | 34 | 19,663 | 33,212 | 26 | 222 | 911,044 | 1.20 |
| Scizor | 55 | 47 | 18,558 | 27,348 | 17 | 246 | 784,780 | 1.28 |
| Scyther | 57 | 40 | 19,533 | 30,360 | 13 | 201 | 786,488 | 1.18 |

The prior runtime primitive counts were 24, 32, 43 and 36 respectively.
The 20,000-vertex, 40,000-triangle, 64-runtime-part, 64-joint,
256-accessor and 1 MiB limits remain unchanged. Physical-size and terrain
support/admission paths remain unchanged. No game content or audio is changed.

Current packed geometry pins:

- Lickitung: `972f51bfd45847d1dcda324c713ac66537d2eb19a6a9407ae395279d9bc99156`.
- Mr. Mime: `d583990f629188f68ad7098a02d6fca3b854f80187ac1bc00ac935bbf9bd1ab2`.
- Scizor: `c53bc5cf81b225ae9057e6bb3e1f854cf3903411a8c2b30e59b402b565002a73`.
- Scyther: `549a4c15bda5cddc1bb41177a7aac6881f06e09d2e1d245940677f0beb3ea908`.

Current full GLB SHA-256:

- Lickitung: `312a89a4eb72623112b29bb265c2f3e478feef7dda95c147d29100b36e44ea13`.
- Mr. Mime: `b726af3dd2962062035970ae5da1c571d92fc55adc4b199cda7d3c105c7b4ae8`.
- Scizor: `89c57edf4c56d45718b1b8c56d1cd0286dcb0154fb09c0ee2c97b72ff9c73ac4`.
- Scyther: `51ef533f1fce97ea199fd027943dcf098f474e8fee1f86ed2776e55346b1a7cd`.

All 44 Python support-packing and four-family skin checks pass in 116.967
seconds. All 31 Rust locomotion checks pass in 156.70 seconds, including the
new four-species ownership regression and roster-wide checks. Four canonical
skin and four original-size GPU regressions pass. The shared registry and
source-cue/GPU timing regressions also pass. Background Blender reproduces all
four exact GLBs. Native examples rebuild in 44.19 seconds with `crystal-assets`
at optimization level 2.

All eight production 3D/2D pairs and the four Route36 moving frames were
inspected inline. Every launch uses headless image capture, preview mute and
normal coloration. Preflight requires the headless parse guard, mute support
and the exact current gzip payload for all four canonical GLBs in each native
executable. The executable identities remain unchanged after the full batch.
No window, tab or application activation is used.

The world reviews start at Route36 `(20, 8)`, hour 12, and hold ordinary Down
input for 32 review ticks. Each trace stays on Route36 and changes authoritative
position. The moving screenshot requests occur at tick 12; readback is
asynchronous, so these supplement the independent 64-phase ownership and
support checks rather than supplying exact-frame 2D animation comparisons.
The endpoint pairs freeze the same authoritative state across 3D and 2D.
The production `SkinnedMesh` world path samples distance-based locomotion and
publishes the world rig's sampled joint transforms.

Lickitung and Mr. Mime use camera `5 2 30 0`; the mantises use `3 2 40 0`.
The views retain full body silhouettes, original scale, readable material
colors and grounded support poses. Lickitung's thigh markings and Mr. Mime's
hip/knee shells remain aligned with their limbs. Mantis pads and toe details
follow their feet while the greaves follow the lower legs. Side-on overlap
limits visibility of some far limbs; the per-leg numerical regression covers
both sides. Existing fine mouth-rim aliasing and broader camera/follower-spacing
work remain open. No model or terrain scaling workaround is introduced.

| Pair | State frame | State hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- | --- |
| lickitung world | 1358 | 1656429115 | 1364 | 23.11312825 |
| lickitung battle | 1652 | 3116806245 | 2287 | 39.169492041 |
| mr__mime world | 2012 | 3416509050 | 2019 | 34.404780541 |
| mr__mime battle | 2162 | 890474973 | 2788 | 47.599675667 |
| scizor world | 1586 | 2222387518 | 1591 | 26.896026583 |
| scizor battle | 2246 | 3221153577 | 2916 | 49.922411458 |
| scyther world | 1759 | 1598317361 | 1767 | 30.013507291 |
| scyther battle | 2139 | 3258571660 | 2748 | 46.938073583 |

| World trace | Review ticks | Column | Visited rows | Tick-12 interpolated center Y |
| --- | --- | --- | --- | --- |
| lickitung | 32 | 20 | 8–12 | -38.895 |
| mr__mime | 32 | 20 | 8–12 | -33.471 |
| scizor | 32 | 20 | 8–12 | -32.365 |
| scyther | 32 | 20 | 8–12 | -39.176 |

Normal Route29 walking encounters retain named modeled species with
`fallback=None`, zero support offset and `accepted=true reason=None` at the
original external pack heights. Whole-body support, terrain admission and
omission gates remain unchanged. Stage counters describe candidate observations,
not percentages.

| Player / opponent | Pack height (m) | Uniform scale | Reference height | Support Y | Stage checked / unsupported |
| --- | --- | --- | --- | --- | --- |
| lickitung / PIDGEY | 1.1938 | 2.6809916 | 1.18 | 0.0 | 176 / 83 |
| mr__mime / PIDGEY | 1.2954 | 2.8606749 | 1.2 | 0.0 | 181 / 83 |
| scizor / SENTRET | 1.8034 | 3.7336016 | 1.28 | 0.0 | 434 / 178 |
| scyther / SENTRET | 1.4986 | 3.3655002 | 1.18 | 0.0 | 449 / 225 |

Executable SHA-256:

- render_at_location: `18a4c70321bd3ba1e97695e9aac74602052b7d76c76e1e97674fcd1f33a5da5d`.
- immersive_battle_3d: `bfec9997c33cf6e7b93aa0dff39b66752322b4322fcb60f011032863b0ebed08`.

The 85 passing checks comprise 44 Python checks, 31 Rust locomotion checks,
four canonical-skin checks, four original-size GPU checks and two shared
registry/cue checks. Final ignored evidence uses
`target/pokemon-native-review/support-four-`: selected native PNGs, paired-state
receipts, world movement CSVs, logs, old/new decoded battle comparison and
executable identity. Temporary old asset copies, Blender export duplicates and
helper scripts are removed. Browser and broader moving-world acceptance remain
open. Shiny checks are skipped; grass, AA and filtering are untouched.
