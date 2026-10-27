# Native review

The generated source is retained intact. The Rust importer produced four native frames: 56×56 fronts with 784 bytes and 48×48 backs with 576 bytes. Each species has its own white/two-color/black palette. Crumband and Jarbaron use different normal palettes.

At native size, Crumband reads as a compact, shallow cookie-pail body with one C-shaped handle. Jarbaron grows into a deeper, wider pail with a complete high arch, three overflowing cookies, heavier arms and larger baked feet. Back portraits show the upper rim, handle and shoulder mass, with the lower body cut off. The handle progression carries Doughlin’s curl forward.

Source-backed and solid-exterior conversions produced identical native rows for this sheet. The source contains a few extra stepped color shades; the converter maps these strictly to the declared four hardware inks. No gradients survive in native output.

Doughlin was not regenerated or converted. Its six PNG/JSON/2bpp hashes were checked before and after this work and remain exact. Current production LINE090 assets, roster and profiles remain unchanged. This is a proposal for root review.

Source-backed review: target/minnesota-pokemon/proposals/MN_F090-v2/line.review.png

Solid-mode review: target/minnesota-pokemon/proposals/MN_F090-v2-solid/line.review.png

Selected candidate: solid exterior with interior_black_min_samples=8. Comparing8,12and16 atnative size favored8: brow masks, cookie boundaries andhandle joints remain black andlegible without isolated speckles. Both front andback views use8. All parameter variants andtheir renders are retained.

Selected review: target/minnesota-pokemon/proposals/MN_F090-v2-solid-ink8/line.review.png

Use manifest-selected.json for the proposed replacement views. It intentionally excludes Doughlin.
