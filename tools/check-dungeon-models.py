#!/usr/bin/env python3
"""Validate original dungeon runtime exports without reading a game pack."""
import json, math
from pathlib import Path
from model_asset_storage import read_model_json
ROOT=Path(__file__).resolve().parents[1]
EXPECTED={
 'cave_boulder','dark_boulder','ice_boulder','ice_mass','tower_guardian','alph_guardian',
 'stone_tablet','gym_plaque','warehouse_crate','ship_barrel','ship_stool','ship_rack',
 'ship_bunk','porthole_bulkhead','timber_column','warning_beacon','gym_bin','league_podium','timber_wall',
}
def validate(path):
    data=read_model_json(path);assert data['name']==path.name.removesuffix('.mesh.json')
    triangles=0;positions=[];materials=set()
    for p in data['primitives']:
        assert len(p['positions'])%3==0 and len(p['positions'])==len(p['normals'])
        assert all(math.isfinite(v) for v in p['positions']+p['normals'])
        assert len(p['indices'])%3==0 and p['indices']
        assert all(0<=i<len(p['positions'])//3 for i in p['indices'])
        assert all(math.isfinite(v) and 0<=v<=1 for v in p['base_color'])
        assert p['base_color'][3]==1, 'full opaque volumes, no sprite alpha cutouts'
        xyz=list(zip(*[iter(p['positions'])]*3));norm=list(zip(*[iter(p['normals'])]*3))
        assert all(abs(sum(v*v for v in n)-1)<1e-4 for n in norm)
        for i in range(0,len(p['indices']),3):
            ids=p['indices'][i:i+3];a,b,c=[xyz[j] for j in ids]
            u=[b[k]-a[k] for k in range(3)];v=[c[k]-a[k] for k in range(3)]
            cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
            assert sum(x*x for x in cross)>1e-16, (path,'degenerate triangle',i)
            assert sum(cross[k]*norm[ids[0]][k] for k in range(3))>0, (path,'reversed winding',i)
        materials.add(tuple(p['base_color']));positions.extend(xyz);triangles+=len(p['indices'])//3
    assert 24<=triangles<8000
    assert triangles==data['triangle_count']
    assert len(materials)>=2, 'semantic authored parts need separate materials'
    for axis in range(3):
        low=min(v[axis] for v in positions);high=max(v[axis] for v in positions)
        assert high-low>0.05, 'models must have full three-dimensional volume'
        assert abs(low-data['bounds']['min'][axis])<1e-5
        assert abs(high-data['bounds']['max'][axis])<1e-5
    return triangles
if __name__=='__main__':
    paths=sorted((ROOT/'crates/crystal-voxel-view/models/dungeons').glob('*.mesh.json'))
    assert {p.name.removesuffix('.mesh.json')for p in paths}==EXPECTED
    counts={p.stem:validate(p) for p in paths}
    for name,count in counts.items():print(f'{name}: {count} triangles')
    print(f'PASS: {len(counts)} original full-volume meshes, {sum(counts.values())} triangles')
