#!/usr/bin/env python3
"""Canonical tree crowns preserve triangle positions, colors, bounds and trunks."""
import hashlib,json,unittest
from pathlib import Path
from foliage_contours import prepare
from sculpt_test_support import assert_model_shading
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models'
EXPECTED={'johto/tree_lod':'e980670ee0c3f727c764b808cf871460f84ef8abaf3876b9bf2d6c3b672f9c39','new_bark/tree':'30447e92962298935288c831338c80a141cd6aced0062d41df9bfa3afcd831ff'}

class FoliageContourTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={n:json.loads((ROOT/(n+'.mesh.json')).read_text()) for n in EXPECTED}
    def test_original_triangles_and_colors_are_exact(self):
        for n,m in self.models.items():
            f=hashlib.sha256()
            for p in m['primitives']:
                ps=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)];values=[c for j in p['indices'] for c in ps[j]]
                f.update(json.dumps([p['material'],p['base_color'],values],separators=(',',':')).encode())
            self.assertEqual(f.hexdigest(),EXPECTED[n])
    def test_regeneration_bounds_and_lod_budget(self):
        for n,m in self.models.items():
            self.assertEqual(m,prepare(m))
            ps=[p['positions'][i:i+3] for p in m['primitives'] for i in range(0,len(p['positions']),3)]
            self.assertEqual(m['bounds'],{'min':[min(v[k] for v in ps) for k in range(3)],'max':[max(v[k] for v in ps) for k in range(3)]})
            self.assertEqual(sum(len(p['indices'])//3 for p in m['primitives']),m['triangle_count'])
        a,b=self.models['new_bark/tree'],self.models['johto/tree_lod']
        self.assertEqual(a['bounds'],b['bounds']);self.assertLess(b['triangle_count']*2,a['triangle_count']);self.assertLess(b['triangle_count'],300)
        self.assertLess(sum(len(p['positions'])//3 for p in b['primitives'])*2,sum(len(p['positions'])//3 for p in a['primitives']))
    def test_crown_normals_are_valid_and_blend_across_materials(self):
        from chikorita_sculpt import unit,cross,sub,dot
        for n,m in self.models.items():
            parts=[];shared={};blended=0
            for p in m['primitives']:
                q=dict(p);q['part']=q['name'];parts.append(q)
                if not p['material'].startswith('leaf'):continue
                ps=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
                ns=[tuple(p['normals'][i:i+3]) for i in range(0,len(p['normals']),3)]
                for v,normal in zip(ps,ns):shared.setdefault((v,normal),set()).add(p['material'])
                for i in range(0,len(p['indices']),3):
                    a,b,c=p['indices'][i:i+3];face=unit(cross(sub(ps[b],ps[a]),sub(ps[c],ps[a])))
                    if min(dot(face,ns[j]) for j in (a,b,c))<.999:blended+=1
            assert_model_shading(self,{n:{'primitives':parts}})
            self.assertGreater(blended,100,n)
            self.assertGreater(sum(len(materials)>1 for materials in shared.values()),20,n)
if __name__=='__main__':unittest.main()
