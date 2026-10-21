# Original Hitmonchan sculpture and boxing skin

The current [three-fighter fidelity audit](hitmon-fidelity.md) supersedes the
earlier front-led review. `tools/hitmonchan_sculpt.py` authors a rounded skull
behind five crown points, pointed shoulder guards, a fitted tunic, open pleated
hem, upright supporting shafts and substantial boxing boots. Bent arms carry
teardrop mitts with occupied cuffs and curled thumbs. Internal cap diagonals
are improved before curving the crown, keeping the rear skull smooth.

`tools/hitmonchan_glb.py` exports and strictly reads the canonical GLB. Its
34 parts contain 10,981 vertices, 16,084 triangles, 12 joints and 184 accessors
in 487,340 bytes. The source digest is
`b3fad26422a1f21e80521d5d8c6520e78d82631230dd212f3243009466d21150`.
Authoring height remains 1.30; pack height `407` retains 1.397 m in production.
The local normal palette retains tan (173,123,99), pink (222,24,148), white and
black. Pink mitts and kit intentionally follow Crystal's supplied pack palette.

Boots, ankle cuffs and toe piping remain fixed during battle cues. The crown
and fitted gaze share one head owner; each glove, cuff, thumb and seam follows
its wrist. Joint pivots follow the revised arm paths. Idle, attack and hit
clips retain identity endpoints and production cue timing. The audit records
the executed geometry, motion and native checks and their coverage limits.

```sh
python3 -m unittest discover -s tools -p test_hitmonchan_sculpt.py
python3 -m unittest discover -s tools -p test_hitmonchan_glb.py
python3 -m unittest discover -s tools -p test_hitmon_fidelity.py
```
