# Partial native review

Selected partial manifest: `manifest-selected.json`. It contains MN_010 Birchit and MN_012 Voyagear only. Root must combine Portabeau and recompute the full-family review hash before canonical integration.

Native output: `target/minnesota-pokemon/proposals/MN_F004-native-v2-center-source/`. Both the native-size board and 8× review were visually inspected. The baby retains paired short teeth and separate eye whites/pupils; the adult retains its dark pupils, black tooth separator and rust muzzle gaps. Its head occupies roughly 20 native pixels while the modest straight paddle stays within the 54-pixel front fit.

Selected policy: `sampling: source_center`, `exterior_contour: source_backed`, with recorded default white/black thresholds 16/32. SourceCenter preserves source-center facial ink without the area-vote white-priority overwrite. Solid was also converted and inspected, but offered no useful improvement. The exact converter was `target/agent-roster-contour/minnesota_sprite_sampling_v5`.

All four frames have exact declared four-color palettes. Fronts are 56×56 with 784 native 2bpp bytes; backs are 48×48 with 576 bytes. Independent decoding of front row-major/back column-major tile planes matches the indexed JSON and every PNG pixel exactly. Metadata records `sampling: source_center` in every view.

Source sheets are 1774×887. Baby crop columns divide at x=910, adult at x=925, both inside clear white gutters. No neighboring-view fragments enter the crops. All v1/v2 sources and conversion alternatives remain saved; no post-generation pixels were manually painted.
