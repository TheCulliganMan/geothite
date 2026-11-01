#!/usr/bin/env python3
"""Validate retained authoring data and the read-only source battle renderer."""
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
source = ROOT / "art/battles/arena-palettes.json"
palettes = json.loads(source.read_text())
assert set(palettes) == {"meadow", "forest", "cave", "water", "interior", "ice"}
for name, palette in palettes.items():
    assert set(palette) == {"ground", "stage", "edge", "accent", "sky"}, name
    for key, rgba in palette.items():
        assert len(rgba) == 4, (name, key)
        assert all(isinstance(v, (int, float)) and math.isfinite(v) and 0 <= v <= 1 for v in rgba), (name, key)
        assert rgba[3] == 1, (name, key)
renderer = (ROOT / "crates/crystal-voxel-view/src/battle_view.rs").read_text()
# Immersive arena palettes remain authored data; attacks consume source cues.
assert "const BATTLE_LAYER: usize = 29;" in renderer
assert "art/battles/arena-palettes.json" in renderer
shader = ROOT / "crates/crystal-voxel-view/src/battle_composite.wgsl"
assert shader.is_file(), "embedded source-scroll compositor is missing"
assert '"battle_composite.wgsl"' in renderer
for forbidden in ("crystal_runtime", "crystal_core", ".tick(", "resolve_turn(", "GameButton"):
    assert forbidden not in renderer, forbidden
bridge = (ROOT / "crates/crystal-bevy/src/bevy_shell/battle_3d.rs").read_text().split("/// Opt-in fresh-session native preview fixture.")[0]
for forbidden in ("shell.snapshot(", "shell.presentation_snapshot(", "session_mut(", "state_mut(", ".press("):
    assert forbidden not in bridge, forbidden
print("Validated arena palette sources, source-scroll shader and read-only battle boundary")
