# Dewdant and Barricant: honeydew redesign

The user requested an appealing honeydew-first design with less resemblance to a steel ant. The selected source is `mn-241-242.front-back.imagegen-v2.png`, created with built-in ImageGen from `prompt-v2.txt`. The original v1 artwork, prompt, reference mapping and the pre-redesign manifest remain preserved.

The dominant shape is now a glossy golden nectar reservoir above a purple lower crescent. Round heads, dew-beaded antennae, small spoon mandibles and curved legs replace the oversized tusks and rigid plated shapes. The mature reservoir grows much larger while the front feet offer sap. The six-limb living-gate behavior and stored sour secretion remain the Bug/Poison theme. The scoped design wording in the roster and guides matches the new anatomy; names, types, profiles and individual palettes were preserved.

Native selection uses source-center sampling with solid silhouettes. Dewdant's front fit is 44×44; Barricant's is 54×54. Actual aspect ratio is preserved inside those bounds. Six guarded Dewdant eye edits and one Barricant eye edit separate the pupils from the surrounding outline. Close rear portraits remain 48×48 and show the reservoir and backs of the heads facing upper right; the fronts face left.

Source-center, area-vote, larger-fit and refined native alternatives and their manifests are retained. The selected recipe is `manifest.json`; `reference-mapping-v2.json` records inputs and selection. `redesign-20261027-v1/manifest-before.json` retains the earlier active recipe. Generated native PNG/index/2bpp files and immutable conversion history remain ignored in `target/minnesota-pokemon/native/MN_F110/`.

## Verification

Inspected all four source/native views at native size and 8×. Verified exact original four-color palettes, 56×56 front / 48×48 rear geometry, 784/576-byte tiles and the selected review-sheet hash. The actual Rust catalog/pack compiler also replays source-center sampling and guarded edits and checks PNG/index/2bpp parity. A temporary four-species fixture containing this line and Clinklaw/Trapnack built successfully from the unchanged local `core-modular.browser.crystalpack`, producing 255 species. The temporary fixture and verification pack are removed after checks; preserved artwork candidates remain.

Executed the existing converter:

```sh
target/agent-minnesota-species/debug/examples/minnesota_sprite_sheet --family MN_F110
```

This validates authored/native art and pack imports; it is not a website deployment or live gameplay claim.

The temporary review pack also passed `verify_crystalpack --audio --desktop`: 388 maps, 255 species, 251 moves and 377 audio programs, with matching PCM hashes, frame counts and loop ranges. No shared pack was replaced.
