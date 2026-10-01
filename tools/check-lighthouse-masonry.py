"""Validate geometry, opacity, closed unit bounds, face roles and compact storage."""
from pathlib import Path
from model_asset_storage import read_model_json as decode
from model_asset_storage import stored_model_size
from collections import Counter
import hashlib,json,math,sys

def cross(a,b):return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def minus(a,b):return tuple(x-y for x,y in zip(a,b))
def check(path):
    d=decode(path);edges=Counter();direction=Counter();volume=0.;triangles=0;side_counts=Counter();allpoints=[];materials=set()
    assert d['format']=='geothite-lighthouse-masonry-v1'
    for p in d['primitives']:
        assert p['side'] in range(6);materials.add(p['material']);assert p['base_color'][3]==1
        assert all(0<=v<=1 and math.isfinite(v) for v in p['base_color'])
        pos=list(zip(*[iter(p['positions'])]*3));norm=list(zip(*[iter(p['normals'])]*3));assert len(pos)==len(norm)
        assert all(math.isfinite(v) and -1e-7<=v<=1+1e-7 for xyz in pos for v in xyz);allpoints.extend(pos)
        assert all(abs(sum(v*v for v in n)-1)<1e-5 for n in norm)
        for i in range(0,len(p['indices']),3):
            ids=p['indices'][i:i+3];assert all(j<len(pos) for j in ids)
            a,b,c=[pos[j] for j in ids];n=cross(minus(b,a),minus(c,a));assert sum(v*v for v in n)>1e-15
            assert all(sum(v*w for v,w in zip(n,norm[j]))>1e-10 for j in ids)
            volume+=sum(v*w for v,w in zip(a,cross(b,c)))/6;triangles+=1;side_counts[p['side']]+=1
            for u,v in [(a,b),(b,c),(c,a)]:
                u=tuple(round(t,6) for t in u);v=tuple(round(t,6) for t in v);edge=tuple(sorted([u,v]));edges[edge]+=1;direction[edge]+=1 if u<v else -1
    assert .70<volume<=1,volume
    assert 0<triangles<=1100,triangles
    assert all(v==2 for v in edges.values()),Counter(edges.values())
    assert all(v==0 for v in direction.values()),'inconsistent winding'
    assert {0,1,2,3,4,5}==set(side_counts)
    for axis in range(3):assert min(p[axis] for p in allpoints)==0 and max(p[axis] for p in allpoints)==1
    if 'window' in path.name:assert {'window_reveal','deep_sea_glass','aged_bronze'}<=materials
    # Every side can be omitted without changing the positions of other sides.
    for mask in range(16):
        keep=[p for p in d['primitives'] if p['side']>=4 or mask&(1<<p['side'])]
        assert all(-1e-7<=v<=1+1e-7 for p in keep for v in p['positions'])
    return {'asset':path.stem,'triangles':triangles,'triangles_by_side':dict(side_counts),'welded_edges':len(edges),'signed_volume':volume,'runtime_bytes':stored_model_size(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}

if __name__=='__main__':
    root=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/lighthouse_masonry'
    report=[check(root/(name+'.mesh.json')) for name in ['ashlar','window_single','window_double']]
    print(json.dumps(report,indent=2))
