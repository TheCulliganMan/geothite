#!/usr/bin/env python3
"""Validate the special-environment exports using the shared geometric checks."""
from pathlib import Path
import runpy
ROOT=Path(__file__).resolve().parents[1]
EXPECTED={'ruins_frieze','league_wall','passage_wall','champion_dragon','ship_bulkhead','ship_door','harbor_bollard','dock_railing','hall_of_fame_terminal','puzzle_dais','harbor_ferry'}
if __name__=='__main__':
    validate=runpy.run_path(str(ROOT/'tools/check-dungeon-models.py'),run_name='geometry_checker')['validate']
    paths=sorted((ROOT/'crates/crystal-voxel-view/models/dungeon_extensions').glob('*.mesh.json'))
    assert {p.name.removesuffix('.mesh.json')for p in paths}==EXPECTED
    counts={p.name:validate(p)for p in paths}
    for name,count in counts.items():print(f'{name}: {count} triangles')
    print(f'PASS: {len(counts)} special-environment meshes, {sum(counts.values())} triangles')
