# Giant mosquito pseudo-legendary — MN_151–MN_153

A Minnesota marsh pressure pump: Wrigglet’s breathing siphon becomes Pupump’s twin air trumpets and Mosquitan’s lance-like proboscis. The small reservoir grows into a huge adult abdomen. The smaller Midgebit/Skeeterex vampire pests remain a separate ordinary family.

| Species | Type | Proposed stat total | Evolution |
| --- | --- | ---: | --- |
| Wrigglet | Bug | 300 | Level 30 |
| Pupump | Bug/Poison | 420 | Level 55 |
| Mosquitan | Bug/Poison | 600 | Final |

The family uses Slow growth. Pupump gains protection while losing speed. Full proposed stats remain in the canonical [roster](../../roster.json). It is a three-stage pseudo-legendary; the catalog retains five actual legendaries and one mythical.

Each Pokemon now has its own four-color GBC palette, consistent between front and back. White and black are common entries; its two ink hues differ by species. RGB555 packs `r | (g <<5) | (b <<10)`; channels expand with `(v <<3) | (v >>2)`.

| Species | Light RGB5 / RGB555 / hex | Dark RGB5 / RGB555 / hex |
| --- | --- | --- |
| Wrigglet |20,26,8 /0x2354 /#A5D642 |8,12,5 /0x1588 /#426329 |
| Pupump |14,27,21 /0x576E /#73DEAD |9,5,14 /0x38A9 /#4A2973 |
| Mosquitan |25,27,8 /0x2379 /#CEDE42 |13,4,10 /0x288D /#6B2152 |

The [original front source](mn-151-153.imagegen-v1.png) and [earlier back source](mn-151-153.back.imagegen-v1.png) remain untouched. The original large source includes antialiasing despite the strict prompt; it is not itself a native game asset. Front anatomy is preserved while Rust remaps its old shared source palette into the individual target palettes.

The [v2 back-only source](mn-151-153.back.imagegen-v2.png) supplies the close Gen2 rear portraits for Wrigglet and Pupump. The [exact back prompt](back-prompt-v2.txt) and [hashed reference mapping](back-reference-mapping-v2.json) record anatomy, target palettes and authentic Crystal style references. Every original, previous manifest and native conversion version is retained.

Mosquitan now uses the separate [v4 rear source](mn-153-mosquitan.back.imagegen-v4.png), made with built-in ImageGen and converted with the Rust importer. Its small head separates from the low hunched thorax, two broad white/plum wings have clear enclosed outlines, and sparse bellows bands wrap the large lime reservoir. The lower body remains cropped at the bottom. The [final prompt](mosquitan-back-prompt-v4.txt) and [reference mapping](mosquitan-back-reference-mapping-v4.json) preserve the edit instructions and inputs. The [v3 candidate](mn-153-mosquitan.back.imagegen-v3.png), its [prompt](mosquitan-back-prompt-v3.txt), and both earlier manifests remain saved. Native 48×48/576-byte validation checks PNG, palette-index JSON, 2bpp tiles and the family board; the other five family frames remain byte-identical to v2. The static Pokédex export is checked separately; these source changes do not replace existing game packs.

The [manifest](manifest.json) records source paths, authored crop bounds, individual GBC definitions and visual review hashes. Current native outputs were visually inspected together: larval head and segmented neck, protected pupal case with twin trumpets, then the enormous winged pump adult show clear anatomical growth. Fronts are 56×56 /784bytes; backs 48×48 /576bytes. Both views use each Pokemon’s same four exact GBC palette entries. Static 2bpp outputs and index JSON live under ignored `target/minnesota-pokemon/native/MN_F072/`; the combined review sheet is 1344×896 at 8× nearest-neighbor enlargement. Content-pack registration and animation remain separate.

Rebuild with the Rust importer commands in the [catalog README](../../README.md). Native versions, original source/crops, manifests, prompts and reference mappings are archived by the converter; generated verification artifacts remain ignored.
