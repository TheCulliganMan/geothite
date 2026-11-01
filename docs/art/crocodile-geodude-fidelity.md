# Crocodile family and Geodude reference revision

This revision replaces the earlier neutral shapes and poses for Totodile,
Croconaw, Feraligatr and Geodude. Earlier numerical or roster acceptance did
not establish convincing proportions, depth, rear markings or posture.

## Reference and pose review

Reviewed the supplied pack's actual front/back pictures, captured
[X/Y front animations](https://www.pkparaiso.com/xy/sprites_pokemon.php) and
[back animations](https://www.pkparaiso.com/xy/sprites_pokemon_espalda.php), and
eight-angle Battle Revolution turnarounds for
[Totodile](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339470/),
[Croconaw](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339471/),
[Feraligatr](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339472/) and
[Geodude](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339394/).
Reference archives were kept externally for study. Runtime geometry remains
original recipe geometry; no downloaded mesh or texture enters the asset tree.
Extracted T-poses and OBJ material artifacts were distinguished from captured
game poses and markings.

The personal `game-model-fidelity-audit` skill now requires front/back and full
orbit comparison, a separate posture audit, fitted/occupied surfaces, motion
checks and production verification. It also excludes Blender's bone display
mesh from turntable bounds. Its files remain outside this repository.

## Authored changes

- Totodile: longer projected muzzle, shallow resting grin, side-wrapping tall
  eyes, two restrained orbital rises, a fuller tail wedge, broad planted toes,
  thinner dorsal plates, and a chest chevron clipped to the actual skin.
  Lowered elbows, forward palms and a modest forward lean replace straight
  sideways arms. The enlarged low tail root keeps the heel foundation fixed.
- Croconaw: rounder jaw bowl and haunches, relaxed bent arms, forward weight,
  and a low broad tail. Its asymmetric pale hide now wraps both sides and the
  back; four irregular blue fields remain on the front.
- Feraligatr: heavier shoulders, thighs, hands and claws; bent elbows below
  forward wrists; forward body carriage; a thick hinged mandible and a real
  recessed oral cup with tongue. Arm plates fit the occupied visible limb
  surface and retain their original clearance limits. The low tail carries a
  serrated fin instead of a narrow upturned tip.
- Geodude: irregular deep rock volume, geological forehead/brow planes and an
  actual mouth depression. Angular bent arms end in broad open palms, four
  separate splayed fingers and opposing thumbs. Hands sit below the top of
  the boulder instead of being held over its head. Hand pivots follow the new
  pose; source-cue timing and fixed lower boulder support remain intact.

All eight neutral angles were inspected for each revised stored GLB, followed
by idle/attack/hit quarter and low-side views. Pose, rear hide, muzzle depth,
tail girth and fitted details were reviewed separately. These samples do not
establish every animation, shiny palette or browser rendering.

| Model | Vertices | Triangles | Runtime parts | Joints | Accessors | Bytes |
|---|---:|---:|---:|---:|---:|---:|
| Totodile | 5,422 | 10,552 | 30 | 8 | 175 | 293,644 |
| Croconaw | 14,789 | 23,092 | 29 | 14 | 181 | 679,016 |
| Feraligatr | 17,447 | 25,520 | 43 | 14 | 241 | 785,928 |
| Geodude | 11,598 | 13,932 | 28 | 8 | 165 | 515,876 |

Existing loader, byte, geometry, accessor and support limits are unchanged.
The remaining 272 canonical GLBs stayed byte-identical. Content packs and
audio were not regenerated.

## Executed verification

The 44 focused Python checks cover stored source parity, geometry/packing,
closed surfaces, normals/winding, rigid details, joint ownership, fixed feet,
tail support, occupied caps/roots, armor clearance, deformation and cue clips.
The final Totodile support correction was followed by all 16 Totodile checks.
Added regressions reject front-only Croconaw hide, flat oral caps, thin tail
depth, forward-facing Totodile eyes, raised Geodude hands and straight adult
arm posture. Four-species storage/source checks and canonical-layout checks
also passed.

The updated native Totodile Blender file and its own manifest record were
verified, then opened and re-exported in background Blender. Its neutral
geometry reproduced the runtime digest exactly. A separate fresh recipe build
also reproduced every runtime GLB byte, including the final support weights.
The full editable-source
catalog check still encounters the pre-existing `gym-scenery-kit.blend`
identity mismatch in `johto_art_sources.read_source`; that unrelated source
and its record were preserved.

Nine focused Rust checks passed: five crocodile checks (both pack-size GPU
activations, jaw/wrist/tail articulation, 64 phases of walking with supported
feet, and early source-cue response), two Geodude checks (shared canonical
body/fixed boulder support and pack-size GPU activation), the Cyndaquil/Totodile
GPU placement/buffer check, and the complete species schema/count/height check.
The schema check's obsolete Croconaw/Feraligatr rows were updated to the new
stored counts. Further filters used the already built test executable, with
all four exact revised compressed payloads verified inside it, to avoid
recompiling for unrelated concurrent source edits.

Fresh `immersive_battle_3d` and `render_at_location` examples built with
`CARGO_TARGET_DIR=target/pokemon-models`, `--locked --jobs 2`, the
`location-tester` feature and `profile.dev.package.crystal-bevy.opt-level=1`.
All four current raw GLBs were verified against their build metadata and their
exact gzip payloads inside both executables before capturing.

For each model, headless production review used a witnessed Route 29 walking
encounter (`--walking-encounter --walk RALARALA...`) settled at the normal move
menu, and a New Bark walking pair started at `(7,12)` with `--walk RR`, hour 12 and
camera `2 1 35 0`. All eight paired-state receipts reported
`state_preserved=true`; battles listed the intended modeled species and
`fallback=None`, and world receipts identified the corresponding
`PartyFollower` source. Pack heights remain 0.6096 m Totodile, 1.0922 m Croconaw,
2.3114 m Feraligatr and 0.4064 m Geodude.

The actual eight 3D and eight 2D images were inspected. Battle menus hide the
classic player's back picture; the separately reviewed pack backs and studio
rear views establish rear likeness. The classic field omits the presentation
follower, so field pairs prove matching state rather than matching follower
art. Geodude's near hand obscures some face in its battle profile, and
Feraligatr's battle camera is high; their studio views and visible field
quarters provide the additional anatomy coverage. An additional lower orbit
shows Geodude's face and spread hands. The attempted low Feraligatr view was
blocked by foreground foliage and is not accepted as model evidence.
Broad browser, shiny and
every-animation acceptance is not claimed.

The final ignored review output is `target/pokemon-fidelity-review-20261020/`:
four full eight-angle sheets, matching baseline/after comparisons, sampled
motion and the inspected production pairs. Temporary source downloads and
intermediate exports are removed after review. No changes were staged or
committed.
