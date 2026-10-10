#!/usr/bin/env python3
"""Stored papercraft water-rabbit surfaces, anatomy and actual attachment checks."""
import copy,json,unittest
from pathlib import Path
from aqua_rabbit_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from chikorita_sculpt import cross,sub,unit,dot
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'


def points(p):return [tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
def triangles(parts):
    out=[]
    for p in parts:
        ps=points(p);out.extend([ps[j] for j in p['indices'][i:i+3]] for i in range(0,len(p['indices']),3))
    return out

def hits(ts,x,y):
    zs=[]
    for a,b,c in ts:
        det=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
        if abs(det)<1e-12:continue
        u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/det;v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/det
        if min(u,v,1-u-v)>=-1e-5:zs.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
    return zs

class AquaRabbitTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={n:json.loads((ROOT/(n+'.mesh.json')).read_bytes()) for n in ('marill','azumarill')}
    def test_closed_anatomy_including_joined_material_regions(self):
        models=copy.deepcopy(self.models)
        for m in models.values():
            a,b=m['primitives'][:2]
            joined={'part':'Body / welded material surface','positions':a['positions']+b['positions'],'normals':a['normals']+b['normals'],'indices':a['indices']+[i+len(a['positions'])//3 for i in b['indices']]}
            m['primitives']=[joined]+m['primitives'][2:]
        assert_closed_models(self,models)
    def test_curved_skin_normals_agree_with_actual_triangles(self):
        assert_model_shading(self,self.models)
        for n,m in self.models.items():
            blended=0
            for p in m['primitives'][:2]:
                ps=points(p);ns=[p['normals'][i:i+3] for i in range(0,len(p['normals']),3)]
                for i in range(0,len(p['indices']),3):
                    a,b,c=p['indices'][i:i+3];face=unit(cross(sub(ps[b],ps[a]),sub(ps[c],ps[a])))
                    if min(dot(face,ns[j]) for j in (a,b,c))<.999:blended+=1
            self.assertGreater(blended,100,n)
    def test_recipe_ground_and_scale(self):
        for n,m in self.models.items():
            self.assertEqual(m,sculpture(n));names=[p['part'] for p in m['primitives']]
            self.assertEqual(len(names),len(set(names)))
            self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0.)
            self.assertLess(sum(len(p['indices'])//3 for p in m['primitives']),12000)
            for p in m['primitives']:
                if p['part'].startswith('Foot /'):self.assertEqual(min(p['positions'][1::3]),0.)
    def test_species_proportions_and_identity(self):
        ratios=[]
        for n,m in self.models.items():
            adult=n=='azumarill';names=[p['part'] for p in m['primitives']]
            for prefix,count in {'Body /':2,'Arm / flattened':2,'Foot /':2,'Ear / round':0 if adult else 2,'Ear / bent':2 if adult else 0,'Belly / fitted white spot':6 if adult else 0,'Tail /':2,'Eye / fitted':2,'Mouth /':2}.items():
                self.assertEqual(sum(s.startswith(prefix) for s in names),count,(n,prefix))
            ps=[v for p in m['primitives'][:2] for v in points(p)]
            ratios.append((max(v[1] for v in ps)-min(v[1] for v in ps))/(max(v[0] for v in ps)-min(v[0] for v in ps)))
        self.assertGreater(ratios[1],ratios[0]*1.25)
    def test_pale_belly_is_a_welded_material_seam(self):
        for n,m in self.models.items():
            a,b=m['primitives'][:2];seam=set(points(a))&set(points(b))
            self.assertEqual(len(seam),40);self.assertNotEqual(a['base_color'],b['base_color'])
            if n=='azumarill':self.assertGreater(max(v[1] for v in seam)-min(v[1] for v in seam),.020)
    def test_fitted_details_and_real_foot_tail_attachments(self):
        for n,m in self.models.items():
            body=triangles(m['primitives'][:2])
            for p in m['primitives']:
                name=p['part']
                if name.startswith(('Eye /','Mouth /','Nose /','Belly /')):skin=body
                elif name.startswith('Ear / fitted'):
                    side=name.split()[-1];skin=triangles([q for q in m['primitives'] if q['part'].startswith(('Ear / round','Ear / bent')) and q['part'].split()[-1]==side])
                else:continue
                self.assertGreater(p['normals'][2],0.,(n,name))
                for x,y,z in points(p):
                    zs=hits(skin,x,y);self.assertTrue(zs,(n,name,x,y))
                    self.assertLess(abs(z-max(zs)),.014,(n,name,x,y))
            for p in m['primitives']:
                if p['part'].startswith('Foot /'):
                    inside=[(x,y,z) for x,y,z in points(p) if (lambda zs:zs and min(zs)<=z<=max(zs))(hits(body,x,y))]
                    self.assertTrue(inside,(n,p['part'],'floating foot'))
            cord=next(p for p in m['primitives'] if p['part'].startswith('Tail / continuous'))
            buoy=next(p for p in m['primitives'] if p['part'].startswith('Tail / blue buoy'))
            center=[(min(buoy['positions'][i::3])+max(buoy['positions'][i::3]))/2 for i in range(3)]
            self.assertLess(min(sum((v[i]-center[i])**2 for i in range(3))**.5 for v in points(cord)),.013)
if __name__=='__main__':unittest.main()
