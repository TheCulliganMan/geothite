#!/usr/bin/env python3
"""Prevent auxiliary or encoded geometry duplicates in the canonical asset tree."""
from pathlib import Path

root = Path(__file__).resolve().parents[1] / "crates/crystal-voxel-view/models"
if not root.is_dir():
    raise SystemExit(f"Canonical model directory is missing: {root}")
invalid = sorted(
    p.relative_to(root).as_posix()
    for p in root.rglob("*")
    if p.is_file() and p.suffix in {".stl", ".b64", ".gz"}
)
for glb in sorted(root.rglob("*.glb")):
    for suffix in (".mesh.json", ".rig.json"):
        if glb.with_suffix(suffix).exists():
            invalid.append(f"{glb.relative_to(root)} duplicates {glb.with_suffix(suffix).name}")
if invalid:
    raise SystemExit("Derived/duplicate canonical asset files: " + ", ".join(invalid))
print("Verified canonical model layout: no STL, transport wrappers, or same-asset GLB/JSON copies")
