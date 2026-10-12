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
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
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
- Pidgeot and Fearow's earlier shared construction is now replaced by
  [individual original raptor-bird sculptures](raptor-bird-sculptures.md), with
  fourteen candidate and canonical mesh tests, inspected front, oblique and
  rear software images, a fresh all-251 Rust check and two individually
  inspected full-frame production GPU battle captures. One unchanged executable
  used app-level mute; its compiled model fingerprints match the installed
  meshes. This supplies replacement-model proof beyond the earlier roster
  captures. Pidgeotto's existing original production wing rig is preserved.
  Inspected official [Pidgeot](https://sg.portal-pokemon.com/pokedex/0018/)
  and [Fearow](https://sg.portal-pokemon.com/pokedex/0022/) artwork establishes
  different anatomy: Pidgeot needs a long swept gold/red crest, black eye
  stripe, rounded cream breast and layered cream flight feathers; Fearow needs
  a long narrow neck, long pink bill, upright pointed red crown, ragged brown
  breast and brown flight feathers with pale tips. The shared three-leaf crest
  and nearly identical wing outlines do not satisfy either silhouette.
  The inspected [Free3D Pidgeot listing](https://free3d.com/3d-model/pidgeot-pokemon-92380.html)
  explicitly reports no rig or animation and a personal-use license.
  [Rashky's Mega Pidgeot metadata](https://api.sketchfab.com/v3/models/3398ad3b162b4f30b92e5ee2df3f9ab2)
  reports downloads disabled, no license and zero animation clips; it is also
  a different form. Neither establishes a usable open rig for the required
  Crystal model. No external geometry was imported from these candidates.
  Eire's [Pidgeot metadata](https://api.sketchfab.com/v3/models/e0d70c4708404544a2120087bb5e6dd4)
  and [Fearow metadata](https://api.sketchfab.com/v3/models/152b20b94b104fea9f59761f2f3707a0)
  advertise downloadable CC BY 4.0 models with zero animation clips. Both
  authenticated download endpoints returned HTTP 401. Their empty descriptions
  do not establish authoring provenance; their geometry and skeletons have
  not been inspected. Zero clips alone does not prove absence of a rig.
- The earlier legendary-bird captures showed simplified bodies and wings.
  Articuno now has an [original crystalline crown, layered ice vanes and
  continuous curling ribbon](articuno-sculpture.md), with eleven candidate and
  canonical mesh tests and inspected three-angle software views. Its fresh
  muted production GPU capture passed and was inspected in full at 2,000 by
  1,400 pixels; the compiled source fingerprint and executable identity were
  verified. The all-251 Rust regression passed. Zapdos now has an
  [original angular thunder fan and black underfeather model](zapdos-sculpture.md),
  with eleven passing candidate and canonical tests and inspected final front,
  oblique and rear software views. Its fresh muted production GPU capture
  passed and was inspected in full at 2,000 by 1,400 pixels, with the compiled
  model fingerprint and executable identity verified. The fresh all-251 Rust
  regression passed. Moltres now has an
  [original gold leading-wing and closed flame-bank model](moltres-sculpture.md),
  with twelve passing candidate and canonical tests and inspected final front,
  oblique and rear software views. Its fresh muted production GPU capture
  passed and was inspected in full at 2,000 by 1,400 pixels. Lugia now has a
  [dedicated original sea guardian model](lugia-sculpture.md), with thirteen
  passing candidate and canonical tests and inspected front, oblique and rear
  software views. Its Rust dedicated-model/icon-routing check and fresh muted
  production GPU capture passed; the full image was inspected at 2,000 by 1,400.
  Ho-Oh now has an
  [original rainbow phoenix model](ho-oh-sculpture.md), with thirteen passing
  candidate and canonical tests and inspected front, oblique and rear software
  views. Its fresh muted production GPU capture passed and was inspected in
  full at 2,000 by 1,400 pixels. Its dedicated-model/icon-routing Rust check
  also passed. The
  [Zapdos and Moltres source research](legendary-bird-rig-research.md) has not
  yet yielded a validated usable rig. The current all-251 Rust model regression
  passed with all three replacement source hashes verified in its binary.
  Lugia's battle replacement preserves its original `actor_props` icon and
  enters the exact battle registry without duplicating its species entry.
  Ho-Oh's dedicated replacement follows the same preservation policy.
- Wartortle now has an [original feather turtle model](wartortle-sculpture.md),
  with twelve passing candidate and canonical checks and inspected final front,
  oblique and rear software views. Its fresh native build and individually
  inspected full-frame muted GPU battle capture passed, with the current source
  fingerprint verified in one unchanged executable. The all-251 Rust model
  regression passed with both current turtle fingerprints verified in its
  test executable.
  The [turtle source review](turtle-source-review.md) found no accessible
  validated open rig among the candidates checked.
- Blastoise now has an [original armored turtle model](blastoise-sculpture.md),
  with thirteen passing candidate and canonical tests, actual recessed twin
  cannon cavities and inspected final front, oblique and rear software views.
  Its fresh muted GPU battle capture passed and was individually inspected,
  including both cannon openings, using the same verified unchanged executable
  as Wartortle. The all-251 Rust model regression passed with both current turtle
  source fingerprints verified in its test executable.
- Nidorina now has an [original hornless poison-pin sculpture](nidorina-sculpture.md),
  with nine passing candidate and canonical checks, fitted eyes and side spots,
  notched ears, rooted dorsal barbs and corrected grounded feet. Final front,
  oblique and rear software views were individually inspected. A fresh native
  build and individually inspected muted full-frame GPU battle capture passed.
  The full current source fingerprint was verified in its unchanged executable.
  The all-251 Rust model check passed with all four current family fingerprints
  verified in its test executable. The
  [Nidoran source review](nidoran-source-review.md) has not yielded an accessible
  validated open rig among the specific candidates checked.
- Nidorino now has an [original low poison-pin quadruped](nidorino-sculpture.md),
  with nine passing candidate and canonical checks, a long lowered wedge head,
  rooted forward horn and ragged teal-lined ears, four grounded clawed paws,
  dorsal barbs and fitted irregular purple markings. Its final front, oblique
  and rear software images were individually inspected. The fresh native build and individually inspected muted full-frame GPU
  captures passed for both middle evolutions. The all-251 Rust model check
  passed with all four current family fingerprints verified in its test executable. An advertised CC BY animated Nidorino
  rig was checked, but its download returned 401 and linked Maya scene returned
  404; no usable rig was retrieved or imported.
- Female and male Nidoran now have [distinct original small sculptures](small-nidoran-sculptures.md),
  with ten passing candidate and canonical checks and inspected final front, oblique and rear
  software sheets. The female's pale compact body, broad navy/teal ears and cheek
  barbs differ from the male's purple low crouch and tall teal-lined ears.
  Both individually inspected muted full-frame GPU captures passed. All four
  final Nidoran-family fingerprints were verified in one unchanged rebuilt
  executable despite the build being queued before the small pair was installed.
  The all-251 Rust model regression passed in 1.31 seconds with zero failures;
  all four current family fingerprints were verified in its test executable.
- Nidoqueen, Nidoking, Hitmonlee and Hitmonchan now have individually
  authored [queen](nidoqueen-sculpture.md), [king](nidoking-sculpture.md),
  [kicking](hitmonlee-sculpture.md) and [boxing](hitmonchan-sculpture.md)
  anatomy. Their nine, twelve, nine and eleven stored-mesh checks passed,
  and final front, oblique and rear software views were inspected. One
  unchanged rebuilt executable, verified against all four final fingerprints,
  produced four individually inspected 2000×1400 production GPU battle images
  with exit zero. Music and effects were muted before playback. The exact
  all-251 Rust model check passed in 2.19 seconds with all four fingerprints
  verified in its actual test executable. Nidoking's open mouth also passes
  an occupied-volume gap check and rejects a deliberately filled negative.
- Hitmontop now has an [original handstand sculpture](hitmontop-sculpture.md),
  eleven passing candidate and canonical mesh checks and inspected final front,
  oblique and rear software views. Its grounded spin point, broad tan helmet,
  blue body, raised three-spiked feet and curved ball tail replace the generic
  brown fighter. A fresh native build and individually inspected production GPU capture passed
  with its final fingerprint verified in the unchanged executable.
- Tyrogue now has an [original small scuffle sculpture](tyrogue-sculpture.md),
  ten passing candidate and canonical mesh checks and inspected final front,
  oblique and rear software views. A continuous three-pronged head, fitted
  yellow eyes, cream waist/wrist bands, brown shorts and shoes and open
  three-finger hands replace the generic fighter. Its final fingerprint was verified in the unchanged native executable and
  its production GPU capture exited zero and was fully inspected.
- Jynx and Smoochum now have [distinct original ice-dancer sculptures](ice-dancers-sculptures.md),
  eleven passing candidate and canonical mesh checks and individually inspected
  final paired front, oblique and rear software renders. Jynx has a continuous
  gown, fitted gold/red bodice, long parted hair, white sleeves and five-finger
  hands. Smoochum has a pink baby body, cream chest and limbs, fitted green
  eyes, short notched bob and four-leaf tuft. Both have rooted open kiss rims
  with a verified recess. Both final fingerprints were verified in an unchanged native executable;
  their production GPU captures exited zero and were fully inspected.
  All four fighter/ice-dancer captures were muted before playback, and the
  exact all-251 Rust regression passed in 2.57 seconds with their final
  fingerprints verified in the actual unchanged test executable.
- Electabuzz and Elekid now have [distinct original electric sculptures](electric-pair-sculptures.md),
  thirteen passing candidate and canonical checks and three inspected software
  views. Electabuzz has real open fang-bearing jaws, crests, fitted bolts and
  dorsal stripe, clawed feet and a striped tail. Elekid has a continuous egg
  silhouette and contact blades with actual through holes. Both final fingerprints were verified in one unchanged native executable;
  two muted production GPU images exited zero and were fully inspected.
  The exact all-251 Rust check passed in 3.16 seconds with both final
  fingerprints verified in the actual unchanged test executable.
- Magby now has an [original live-coal sculpture](magby-sculpture.md),
  with a low five-lobed crown, fitted gaze, dark collar, open bill aperture,
  clawed hands/feet and a plain pointed tail. Ten candidate checks and three
  individually inspected final software views passed. The final fingerprint was verified in the native executable, and its muted
  production GPU capture exited zero and was fully inspected.
- Magmar now has an [original spitfire sculpture](magmar-sculpture.md),
  twelve passing final candidate and canonical checks and three inspected software
  views. Heavy twin brows and flowing crown flames, open bill/tongue, fitted
  flame-shaped belly, blue bands, yellow joints, jagged orange cuffs, clawed
  hands/feet and a yellow flaming tail replace its generic primitive body.
  Belly clearance and shading were corrected and the collar exposed after
  visual review. Both fire-pair final fingerprints were verified in one unchanged native
  executable and two muted GPU captures exited zero and were fully inspected.
  The exact all-251 Rust check passed in 2.09 seconds with both fingerprints
  verified in its actual unchanged test executable.
- Goldeen, Seaking and Magikarp's earlier simple fins and tubular mouths have
  been replaced by [individual original sculptures](goldfish-sculptures.md).
  All eleven stored-mesh checks and all three software angles passed for the
  installed meshes. A fresh native build, the exact all-251 Rust model check
  and three individually inspected full-frame production GPU battle captures
  passed. The captures used one unchanged executable and app-level mute for
  music and effects. This supplements the earlier roster captures with proof
  of the replacement fish meshes; the separate live-window issue remains open.
- Dratini, Dragonair and Dragonite now have [individual original anatomy](dragon-family-sculptures.md),
  twelve stored-mesh tests, all three software angles, a fresh all-251 Rust
  load check and three inspected production GPU battle captures. Dratini has
  a continuous broad curve and forehead dot; Dragonair has feather fans and
  three rooted pearls; Dragonite has a smooth segmented belly, curved feelers,
  rooted limbs and teal wing membranes. The remaining shared large serpents
  still need individual treatment. The installed [original Onix sculpture](onix-sculpture.md)
  has thirteen rooted boulders, fitted narrow gaze, wedge nose/jaw and three
  grounded tail supports, with eleven passing canonical checks and three
  inspected software views. Dedicated builder routing and the measured body
  reference span are installed. Its silent final-hash GPU capture exited zero
  and was fully inspected, including actual tail contact with the arena floor.
  The exact all-251 Rust check and all 18 battle-layout tests passed with the
  final hash verified and executable identities stable. The installed
  [Steelix sculpture](steelix-sculpture.md) now has a broad angular skull,
  toothed open jaw, red irises, alternating body spikes and pointed tail.
  Thirteen canonical checks and three final software views pass. Its native
  final-hash silent GPU capture passed and was individually inspected, including
  grounded tail segments and the complete head/spikes silhouette. The all-251
  check and all 18 corrected battle-layout tests passed in a stable executable
  containing the final hash. Completed Steelix scratch has been removed.
  The installed [Rhydon sculpture](rhydon-sculpture.md) adds the broad drill
  muzzle, open fanged jaw, red gaze, cream belly plates, rooted claws and fitted
  segmented tail. Fourteen canonical checks and three inspected software views
  pass; its exact all-251 check passed with the final hash verified in a stable
  executable. Its final-hash muted GPU battle capture now exits zero and the
  complete frame has been inspected for appearance and floor contact.
  The installed [Graveler sculpture](graveler-sculpture.md) replaces the toy
  head with a faced irregular boulder, adds differently proportioned upper
  and folded lower arm pairs, rooted stone fingers and supporting feet.
  Ten canonical checks and three inspected final software views pass.
  Its exact all-251 check passed with the final hash verified and executable
  identity stable; its final-hash muted GPU battle capture now exits zero, with
  the complete frame inspected for appearance and floor contact. The installed
  [Golem sculpture](golem-sculpture.md) replaces the scattered stones with 32
  fitted shell plates, a reptilian open jaw, red gaze and clawed limbs.
  Fourteen canonical checks and three inspected software views pass. Its
  final-hash muted GPU battle capture exits zero and the complete frame has
  been inspected. A stable executable containing all three final rock-biped
  hashes passes the exact all-251 loading test in 2.96 seconds. These checks
  verify the native image target, not live-window, browser or animation paths.

The installed [Slowbro sculpture](slowbro-sculpture.md) replaces the baseline
with a continuous pink body, fitted white gaze, rounded open cream jaws,
segmented fitted abdomen, clawed limbs and a separately toothed, spiked
shell creature gripping its tail. Fourteen candidate and fourteen canonical
checks pass, with three inspected final software views. Its final-hash muted
GPU battle capture exits zero and the full frame was inspected for anatomy,
floor contact, arena and HUD. The exact all-251 loading check passed in
2.52 seconds with the final hash and executable identity stable.

The installed [Slowking sculpture](slowking-sculpture.md) replaces the toy
crown with a horned spiral shell, irregular upper crest and fitted round
ruby, adds a flared alternating red/ivory collar, fitted tall white gaze,
clawed limbs and a tapered upturned tail. Twelve candidate and twelve
canonical checks pass, with three inspected final software views. Its
final-hash muted GPU battle capture now exits zero; the complete frame was
inspected for anatomy, floor contact, arena and HUD with executable identity
and canonical hash stable. The exact all-251 loading check passed in
1.39 seconds in a stable executable containing both final pair hashes,
with unchanged canonical bytes. The earlier Slowbro test executable lacked
the newer Slowking hash and was excluded. Completed pair review scratch
has been removed. Live-window, browser and animation remain unapproved.

The installed [Ponyta sculpture](ponyta-sculpture.md) replaces the toy body
with a slender cream horse stance, dark grounded hooves, fitted youthful
face and pointed ears, arched neck and seven rooted flowing fire banks with
two-sided yellow cores. Twelve candidate checks pass and three final
software views were inspected. All twelve installed canonical checks also
pass. Matching native/all-251 integration jobs are running; final-hash Rust
and muted GPU
verification remain required. Rapidash still needs its adult replacement.

The complete roster capture removes missing-model rendering as an unknown
for this specific preview. It does not remove the substantial modeling work
shown by the images. Temporary PNGs, inspection sheets and logs are removed
after review; no generated capture catalog or pack content is committed.
