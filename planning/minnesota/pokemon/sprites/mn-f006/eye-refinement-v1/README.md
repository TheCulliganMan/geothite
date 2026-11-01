# Fortigard eye correction

The original high-resolution front remains the source. At native size the left sclera collapsed into its brow, and the right pupil disappeared.

The selected recipe in `../manifest.json` changes exactly seven pixels in the approved 56×56 front: (21,12), (22,12), (23,12), (27,12), (28,12), (28,13), (28,14). Both eyes now have white sclerae and distinct black pupils. All other front pixels, all rear views, and Agatot/Bandrake are unchanged.

Guarded pixel edits require source-center sampling. The recipe first restores the approved area-sampled body from that deterministic sample, then makes the seven eye corrections. The original manifest is retained as `manifest-before.json`; all source artwork remains intact. No new ImageGen render was needed.

Verified using the existing Rust converter:

```sh
target/agent-minnesota-species/debug/examples/minnesota_sprite_sheet --family MN_F006
```

Inspected the rebuilt front and family contact sheet. Independent family verification passed all six native PNG/indexed JSON/2bpp frames, palettes, source archives, borders, and contact-sheet comparisons. Final pixels were compared against the earlier exports: only the seven specified eye pixels differ.
