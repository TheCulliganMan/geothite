# Native Pokémon review after the Bevy 0.20 migration

The fresh `immersive_battle_3d` reviewer built successfully using the shared
`target-bevy20-verify` cache. This does not establish rendered appearance.

Fresh Shellder, Cloyster and Krabby battle captures each returned a uniform
black image. The screenshot validator correctly rejected them with one color
and zero luminance range. A separate single-process Shellder capture failed
the same way; classic-mode and live/pipelined control captures were also
black. No other pending
species was approved from these failures. No geometry was changed to conceal
this rendering/capture issue.

The reviewed source meshes remain supported by their stored-mesh regressions
and software previews. Fresh native presentation proof was incomplete at that point.
The Rust all-251 loaded-model check completed successfully after the twelve
recent species replacements, including Magnemite, Magneton, Doduo and Dodrio.
Its scope is loading, mesh bounds, grounding and finite unit normals, not
individual appearance. A later all-251 check also completed successfully after
Farfetch'd, Drowzee and Hypno were replaced: the species test passed with zero
failures. Horsea and Seadra were replaced afterwards. Their fresh all-251 run
returned exit 101 before running tests because the separate foliage work does
not compile: `foliage/park_tree.rs:126` calls private `mesh::append_solid_quad`
(E0603), and line 127 passes `Vec3::to_array` to an array map that requires an
owned `Vec3` argument (E0631). Kingdra was also replaced after the last passing
Rust check. A subsequent source inspection showed the private helper call
removed and the array conversion changed to a closure. A fresh all-251 check
including all three completed successfully: the all-251 species test passed
with zero failures in 1.00 second. This proves loaded bounds, grounding and
finite unit normals, rather than individual appearance. No unrelated foliage source was changed as part of these sculptures.
These captures prove neither TUI gameplay
nor MCP input behavior.

The failed images and temporary capture logs are not deliverables. The shared
build cache belongs to the coordinated Bevy migration and is retained while
other consumers use it. The full Pokémon appearance goal remains active.

An explicit image-target capture from the freshly rebuilt reviewer produced
a nonblack HUD-only image: Shellder and Pidgeotto labels, HP bars and commands
were visible, but neither Pokémon nor the arena appeared. The nonblack-image
validator passed this image; visual inspection correctly rejected it.

The battle compositor previously selected only window cameras. Retargeting
the primary HUD camera to the explicit capture image therefore hid the battle
composite. The capture path now marks that camera with explicit battle
composite ownership; unrelated image cameras remain excluded. A new regression
passes through window-visible, unowned-image-hidden, owned-image-visible,
removed-ownership-hidden and restored-window-visible states. Both existing
image capture ownership/resize regressions and the battle resize/F3 regression
also pass. The ordinary window-target ownership test also passes.

After the corrected native reviewer finished building, explicit image-target
GPU captures completed with exit zero for Shellder, Cloyster, Krabby, Kingler,
Voltorb, Electrode, Exeggcute, Exeggutor, Magnemite, Magneton, Doduo, Dodrio,
Farfetch'd, Drowzee, Hypno, Horsea, Seadra and Kingdra. Every resulting
2000×1400 image was individually inspected: each contains its requested
Pokémon, Pidgeotto, the forest arena and the production HUD. This establishes
actual GPU presentation for these eighteen models in that battle preview.
It does not establish all-251 appearance, browser presentation or gameplay.

A fresh window-target Shellder control still returned exit one and a uniform
black screenshot. Window-target capture remains unresolved; the successful
image-target captures are not evidence that it was repaired. A live-window
control ran, but the computer-use tool could not attach to the standalone
executable, so that attempt supplies no visual proof. The control process was
stopped after this check.

Reproduce the working, explicit GPU image capture after building the reviewer:

```sh
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  ./target-bevy20-verify/wasm-dev/examples/immersive_battle_3d \
  content-packs/realtime-clock.browser.crystalpack \
  --enemy-gust --species KINGDRA --size 1000x700 \
  --screenshot target/kingdra-review.png
```

The screenshot is temporary verification output and must remain ignored.
Inspect the actual image for the Pokémon and arena; the generic nonblack
validator alone cannot distinguish the earlier HUD-only failure.

## Full roster GPU capture and visual inspection

The next review enumerated the union of the exact battle model registry and
`EXACT_SHARED_SPECIES`, asserted exactly 251 distinct names, and captured every
name using the explicit image-target command above. No model file was newer
than the reviewer at the start. The capture driver checked the executable's
inode, size and modification time before each launch and would stop if it
changed. All 251 launches completed with exit zero. The completed output
contained 251 species PNGs and 251 matching successful capture logs.

All 251 requested models were visually inspected in 21 temporary close-up
sheets made from the actual GPU images. Gyarados was also inspected at full
frame; Ho-Oh, Lapras, Lugia, Mantine, Onix and Steelix received additional
full-frame inspection because their models extended beyond the close-up crop.
Rhyhorn received full-frame inspection to confirm its detached armor. Every
requested species produced visible geometry and arena scenery in this preview.
This is one normal-palette battle view per species, not all-angle appearance
approval. It does not resolve the separate window-capture failure or prove
browser presentation, animation completeness, gameplay, or usable external rigs.

The actual images contradict completion of the requested appearance goal.
Immediate corrective work includes:

- Rhyhorn originally had its entire dorsal ridge floating above the torso.
  The old `rocks` recipe put ridge centers at authoring height 0.73 above a
  torso centered at 0.33 with vertical radius 0.26. This is now replaced by
  [original rooted armor and anatomy](rhyhorn-sculpture.md), with ten stored-mesh
  tests, three-angle software inspection and a freshly rebuilt production GPU
  image showing attached visible plates and grounded feet. The other findings
  below remain open.
- Aerodactyl's earlier shared dragon shape is replaced by an
  [original fossil flyer](aerodactyl-sculpture.md): swept skull, genuinely open
  toothed mandible, lean chest and neck, scalloped violet sails, seated wing
  fingers and a visible arrow tail. Twelve stored-mesh regressions, final
  three-angle software inspection, a fresh Rust all-251 check and an inspected
  full-frame production GPU capture passed.
- Pidgeot and Fearow: the older shared bird shape lacks the individual head,
  plumage, beak and tail treatment now visible on Pidgeotto.
- Articuno, Zapdos, Moltres, Ho-Oh and Lugia: each renders, but the body and wing
  treatments remain simplified and need individual anatomy and silhouettes.
- Blastoise/Wartortle, the Nidoran evolutions, the Hitmon family, Jynx/Smoochum,
  Electabuzz/Elekid and Magmar/Magby still show the older rounded primitive
  bodies and oversized protruding eye spheres. These are not final-quality
  replacements.
- Goldeen, Seaking and Magikarp show simple fin plates and tubular mouths.
  Goldeen/Seaking remain candidates for the next original fish sculpture work.
- Dratini, Dragonair and Dragonite now have [individual original anatomy](dragon-family-sculptures.md),
  twelve stored-mesh tests, all three software angles, a fresh all-251 Rust
  load check and three inspected production GPU battle captures. Dratini has
  a continuous broad curve and forehead dot; Dragonair has feather fans and
  three rooted pearls; Dragonite has a smooth segmented belly, curved feelers,
  rooted limbs and teal wing membranes. The remaining shared large serpents
  still need individual treatment. Onix's apparent ground clearance also needs
  a closer placement check before appearance approval.

The complete roster capture removes missing-model rendering as an unknown
for this specific preview. It does not remove the substantial modeling work
shown by the images. Temporary PNGs, inspection sheets and logs are removed
after review; no generated capture catalog or pack content is committed.
