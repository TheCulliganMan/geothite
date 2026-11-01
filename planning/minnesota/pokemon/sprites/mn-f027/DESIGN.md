# Clinklaw and Trapnack: crayfish redesign

The user liked the crayfish / bait-trap theme and requested stronger, more appealing sprites. The selected source is `mn-067-068.front-back.imagegen-v3.png`, created with built-in ImageGen from `prompt-v3.txt`. Source v1 and the first redesign v2 remain preserved with their prompts. V2 improved expression and tail anatomy but was rejected at native size because its flat profile made the eyes too small; v3 uses larger eye windows and a more compact pose.

Clinklaw has a rounded copper-tan body, big white stalk-eye windows, a clear smile and broad steel-tipped pincers. Trapnack grows massive interlocking claws and a domed carapace with three broad steel arches. Both have low segmented abdomens ending in short fan tails, replacing the upright scorpion-like tails. The broad claws, eye clusters and shell bands carry the silhouette; small rivets and lattice details are removed. The Bug/Steel identities, names, profiles and exact individual four-color palettes remain preserved.

The active recipe uses source-center sampling with solid contours and no native pixel edits. Clinklaw's front fit is 46×44, Trapnack's is 54×54; source aspect ratios remain preserved. Enemy fronts face left; 48×48 player rear portraits show the backs of the eyestalks and carapaces aimed upper right. All four views were inspected after conversion at native size and 8×.

`manifest.json` selects source v3, and `reference-mapping-v2.json` records the inputs. The pre-redesign manifest and v2/v3 settings remain in `redesign-20261027-v1/`. Native alternatives are retained under ignored `target/honeydew-crayfish-redesign-20261027-v1/`; selected native PNG/index/2bpp files and immutable history remain in `target/minnesota-pokemon/native/MN_F027/`.

## Verification

Checked exact original palettes, four inks per frame, 56×56 fronts / 48×48 backs, 784/576-byte tiles and the current review-sheet hash. The real Rust catalog and pack compiler replayed the source sampling and verified PNG/index/2bpp parity through a temporary four-species fixture with the honeydew ant line. Building against the unchanged local `core-modular.browser.crystalpack` produced a verified 255-species pack. The fixture and verification pack are temporary and removed after checks.

Executed the existing converter:

```sh
target/agent-minnesota-species/debug/examples/minnesota_sprite_sheet --family MN_F027
```

This covers authored/native art and pack imports, not website publication or a live gameplay session.

The temporary review pack also passed `verify_crystalpack --audio --desktop`: 388 maps, 255 species, 251 moves and 377 audio programs, with matching PCM hashes, frame counts and loop ranges. No shared pack was replaced.
