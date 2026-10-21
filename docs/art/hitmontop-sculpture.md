# Original Hitmontop sculpture and headstand skin

The current [three-fighter fidelity audit](hitmon-fidelity.md) supersedes the
earlier posture review. Captured Crystal and BW sprites confirm the inverted
idle. `tools/hitmontop_sculpt.py` preserves it with a deep helmet, two swept
side flanges and three fitted folds on each flange, a fuller compact blue body,
shorter raised limbs, angular three-spiked boots, wrapped fists and a separate
curved ball tail. Exported reference UV artifacts are excluded from the design.

`tools/hitmontop_glb.py` exports and strictly reads the canonical GLB. Its
41 parts contain 10,934 vertices, 17,604 triangles, 11 joints and 191 accessors
in 447,532 bytes. The source digest is
`f10c1cc4e0482bf05e4a25ba7f3624f04bd517d678f095f4da531de0af07e68b`.
Authoring height remains 1.25; pack height `407` retains 1.397 m in production.
The local normal palette remains gold (198,156,33), blue (82,82,173), white
and black.

The root fixes the spin point. Helmet, flanges, folds and gaze share one owner;
body, belly and neck share continuous weights. Each raised leg carries its wrap,
foot, spikes and pad together. Tail joints follow the shortened curve and keep
the ball and spike coherent. The 3.2-second idle completes one forward turn,
slows near the loop boundary and wraps through shortest-arc quaternions. Small
leg lifts preserve raised-foot clearance through idle, attack and recoil.
Production cues own action timing; the audit records sampled coverage.
World travel preserves this complete skin rather than rebinding the raised
legs through walking IK. Its turn centers on the actual crown contact.

```sh
python3 -m unittest discover -s tools -p test_hitmontop_sculpt.py
python3 -m unittest discover -s tools -p test_hitmontop_glb.py
python3 -m unittest discover -s tools -p test_hitmon_fidelity.py
```
