#!/usr/bin/env python3
"""Stored sea anatomy, shared wing seams and fitted facial geometry."""
import json,unittest
from pathlib import Path
from sea_sculpt import sculpture,SPECIES,HEIGHTS,underside
from mantine_glb import read_mantine,geometry_digest
from corsola_glb import read_corsola
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import ray_intersections,parts_overlap
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class SeaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:read_mantine(ROOT/'mantine.glb') if s=='mantine' else read_corsola(ROOT/'corsola.glb') for s in SPECIES}
    def test_closed_oriented_parts(self):assert_closed_models(self,self.models)
    def test_stored_normals(self):assert_model_shading(self,self.models)
    def test_regeneration_bounds_and_budget(self):
        for s,m in self.models.items():
            self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));ps=m['primitives']
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHTS[s],places=6)
            self.assertEqual(len(ps),len({p['part'] for p in ps}))
            self.assertLess(sum(len(p['indices'])//3 for p in ps),40000 if s=='mantine' else 18000)
    def test_curved_ray_wings_share_material_seams(self):
        ps=self.models['mantine']['primitives'];body=ps[0]
        for side in (-1,1):
            a=next(p for p in ps if p['part']=='Wing / curved blue back '+str(side));b=next(p for p in ps if p['part']=='Wing / pale curved underside '+str(side))
            aa={tuple(a['positions'][i:i+3]) for i in range(0,len(a['positions']),3)};bb={tuple(b['positions'][i:i+3]) for i in range(0,len(b['positions']),3)}
            self.assertGreater(len(aa&bb),45)
            self.assertGreater(max(a['positions'][1::3])-min(a['positions'][1::3]),.4)
            self.assertGreater(max(a['positions'][::3])-min(a['positions'][::3]),1.3)
            x=side*.20;y=.20;hits=ray_intersections(a,x,y)+ray_intersections(b,x,y);core=ray_intersections(body,x,y)
            self.assertTrue(hits and core);self.assertGreater(min(max(hits),max(core))-max(min(hits),min(core)),.1)
    def test_fitted_smiles_and_coral_flecks(self):
        for m in self.models.values():
            body=m['primitives'][0]
            for p in m['primitives']:
                if not p['part'].startswith(('Mouth /','Belly fleck /')):continue
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];hits=ray_intersections(body,x,y)
                    self.assertTrue(hits);self.assertLess(abs(z-max(hits)),.023)
    def test_dorsal_mantle_follows_stored_ray_body(self):
        ps=self.models['mantine']['primitives'];body=dict(ps[0])
        body['positions']=[v for i in range(0,len(body['positions']),3) for v in (body['positions'][i],body['positions'][i+2],body['positions'][i+1])]
        patch=next(p for p in ps if p['part'].startswith('Back /'))
        for i in range(0,len(patch['positions']),3):
            x,y,z=patch['positions'][i:i+3];hits=ray_intersections(body,x,z)
            self.assertTrue(hits);self.assertLess(abs(y-max(hits)),.025)
    def test_coral_feet_contact_white_bowl(self):
        ps=self.models['corsola']['primitives'];belly=ps[1]
        self.assertEqual(sum(p['part'].startswith('Coral / thick lateral fork') for p in ps),5)
        for p in ps:
            if not p['part'].startswith('Foot /'):continue
            self.assertEqual(min(p['positions'][1::3]),0)
            x=(min(p['positions'][::3])+max(p['positions'][::3]))/2;overlap=False
            for k in range(1,16):
                y=max(p['positions'][1::3])*k/16;a=ray_intersections(p,x,y);b=ray_intersections(belly,x,y)
                if a and b and min(max(a),max(b))-max(min(a),min(b))>.005:overlap=True
            self.assertTrue(overlap,p['part'])
    def test_ray_paddles_tail_and_remora_have_actual_physical_joins(self):
        ps=self.models['mantine']['primitives'];find=lambda prefix:next(p for p in ps if p['part'].startswith(prefix));overlap=parts_overlap
        body=ps[0];tail=find('Tail /');leaf=find('Tail leaf /');fish=find('Remora / attached');nose=find('Remora nose /');fork=find('Remora tail /');stalk=find('Remora / slender');attachment=find('Remora / dorsal')
        self.assertTrue(overlap(body,tail));self.assertTrue(overlap(tail,leaf));self.assertTrue(overlap(leaf,find('Tail leaf back /')))
        for p in ps:
            if p['part'].startswith('Cephalic fin /'):self.assertTrue(overlap(body,p),p['part'])
        self.assertTrue(overlap(fish,nose));self.assertTrue(overlap(fish,stalk));self.assertTrue(overlap(stalk,fork));self.assertTrue(overlap(fish,attachment));self.assertTrue(overlap(attachment,find('Wing / pale curved underside 1')))
    def test_zigzag_wing_borders_and_remora_details_fit_stored_surfaces(self):
        ps=self.models['mantine']['primitives'];find=lambda prefix:next(p for p in ps if p['part'].startswith(prefix))
        for p in ps:
            parent=None;mark=p;name=p['part'];sign=name.split()[-1]
            if name.startswith('Wing border /'):mark=underside(p);parent=underside(find('Wing / pale curved underside '+sign))
            elif name.startswith('Remora eye /'):parent=find('Remora / attached')
            elif name.startswith('Remora glint /'):parent=find('Remora eye / fitted tiny dark oval '+sign)
            elif name.startswith('Remora mouth /'):parent=find('Remora nose /')
            if not parent:continue
            for i in range(0,len(mark['positions']),3):
                x,y,z=mark['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,(name,x,y));self.assertLess(abs(z-max(hits)),.017)
            for i in range(0,len(mark['indices']),3):
                a,b,c=[mark['positions'][3*j:3*j+3] for j in mark['indices'][i:i+3]]
                if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
                x,y,z=[sum(v[q] for v in (a,b,c))/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(name,i//3,z-max(hits)))
    def test_terminal_tail_retains_two_seated_color_layers(self):
        ps=self.models['mantine']['primitives'];leaf=next(p for p in ps if p['part'].startswith('Tail leaf /'));cap=next(p for p in ps if p['part'].startswith('Tail leaf back /'))
        self.assertNotEqual(leaf['base_color'],cap['base_color']);self.assertGreater(max(leaf['positions'][::3])-min(leaf['positions'][::3]),.65)
        transform=lambda p:dict(p,positions=[v for i in range(0,len(p['positions']),3) for v in (-p['positions'][i],p['positions'][i+2],p['positions'][i+1])])
        parent=transform(leaf);mark=transform(cap)
        for i in range(0,len(mark['indices']),3):
            a,b,c=[mark['positions'][3*j:3*j+3] for j in mark['indices'][i:i+3]]
            if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
            x,y,z=[sum(v[q] for v in (a,b,c))/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0005)
if __name__=='__main__':unittest.main()
