#!/usr/bin/env python3
"""Stored crocodile anatomy, fitted eyes/jaws and distinct evolution profiles."""
import json,unittest
from pathlib import Path
from crocodile_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from chikorita_sculpt import cross,sub,unit,dot
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class CrocodileTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={n:json.loads((ROOT/(n+'.mesh.json')).read_bytes()) for n in ('croconaw','feraligatr')}
    def test_closed_connected_anatomy(self):assert_closed_models(self,self.models)
    def test_actual_stored_triangle_shading(self):assert_model_shading(self,self.models)
    def test_paper_panels_keep_real_face_normals(self):
        for n,m in self.models.items():
            for p in m['primitives']:
                pts=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
                norms=[p['normals'][i:i+3] for i in range(0,len(p['normals']),3)]
                for i in range(0,len(p['indices']),3):
                    a,b,c=p['indices'][i:i+3]
                    face=unit(cross(sub(pts[b],pts[a]),sub(pts[c],pts[a])))
                    for j in (a,b,c):self.assertGreater(dot(face,norms[j]),.99999,(n,p['part'],i))
    def test_recipe_height_and_ground(self):
        for n,m in self.models.items():
            self.assertEqual(m,sculpture(n));names=[p['part'] for p in m['primitives']]
            self.assertEqual(len(names),len(set(names)))
            self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in m['primitives']),22000)
            for side in ('-1','1'):
                feet=[p for p in m['primitives'] if p['part'].startswith('Foot /') and side in p['part'].split()]
                self.assertAlmostEqual(min(v for p in feet for v in p['positions'][1::3]),0.,places=6)
    def test_distinct_stages_have_real_anatomy(self):
        for n,m in self.models.items():
            adult=n=='feraligatr';names=[p['part'] for p in m['primitives']]
            for prefix,count in {'Body /':1,'Eye / fitted':2,'Nostril / fitted':2,'Belly / blue':0 if adult else 4,'Belly / pale V':2 if adult else 0,'Tooth / upper':6 if adult else 4,'Tooth / lower':4 if adult else 0,'Hand / curved':6,'Hand / white':6 if adult else 0,'Foot / splayed':6,'Foot / white':6 if adult else 0,'Armor /':13 if adult else 0,'Jaw / pale mouth':2 if adult else 0}.items():
                self.assertEqual(sum(x.startswith(prefix) for x in names),count,(n,prefix))
            jaw=next(p for p in m['primitives'] if p['part']=='Jaw / broad upper snout')
            ext=[max(jaw['positions'][i::3])-min(jaw['positions'][i::3]) for i in range(3)]
            self.assertGreater(ext[0],ext[1]*2)
            self.assertGreater(ext[2],ext[1]*1.5)
        def body_width_at(m,y):
            p=next(p for p in m['primitives'] if p['part'].startswith('Body /'));points=list(zip(*[iter(p['positions'])]*3));row=min({q[1] for q in points},key=lambda v:abs(v-y))
            return max(abs(x) for x,yy,z in points if yy==row)
        self.assertGreater(body_width_at(self.models['feraligatr'],.72),body_width_at(self.models['feraligatr'],.51)*1.3)
        self.assertGreater(body_width_at(self.models['croconaw'],.38),body_width_at(self.models['croconaw'],.70)*1.3)
    def test_faces_and_armor_follow_actual_stored_skin_triangles(self):
        # Independent +Z barycentric intersections on stored geometry; recipes
        # are not called to supply their surface or placement expectations.
        for n,m in self.models.items():
            body=next(p for p in m['primitives'] if p['part'].startswith('Body /'))
            jaw=next(p for p in m['primitives'] if p['part']=='Jaw / broad upper snout')
            for p in m['primitives']:
                if not p['part'].startswith(('Eye /','Nostril /','Armor /')):continue
                armor=p['part'].startswith('Armor /');back=p['part'].startswith('Armor / dorsal')
                sources=([body] if back else [q for q in m['primitives'] if q['part'].startswith(('Body /','Arm /','Leg /'))]) if armor else [jaw if p['part'].startswith('Nostril /') else body]
                ts=[]
                for source in sources:
                    pts=[source['positions'][i:i+3] for i in range(0,len(source['positions']),3)]
                    ts.extend([pts[j] for j in source['indices'][i:i+3]] for i in range(0,len(source['indices']),3))
                if not armor:self.assertGreater(p['normals'][2],0.)
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];hits=[]
                    for a,b,c in ts:
                        det=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
                        if abs(det)<1e-12:continue
                        u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/det;v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/det
                        if min(u,v,1-u-v)>=-1e-5:hits.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
                    self.assertTrue(hits,(n,p['part'],i))
                    self.assertLess(abs(z-(min(hits) if back else max(hits))),.023,(n,p['part'],i))
if __name__=='__main__':unittest.main()
