#!/usr/bin/env python3
"""Independent stored Lapras topology and articulated anatomy checks."""
import math
import hashlib
from pathlib import Path
import unittest
import lapras_glb as author
from sculpt_test_support import assert_closed_models,assert_model_shading
from skin_glb_test_support import IDENTITY
from skin_glb import packed
from sculpt_test_support import indexed_ray_intersections
from test_gengar_glb import Model,winding_alignment
ASSET=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/lapras.glb'

class LaprasTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_canonical_single_source_and_bounded_schema(self):
        self.assertEqual(self.blob,author.export_lapras())
        self.assertEqual(len(self.m.parts),45)
        self.assertEqual(sum(len(v) for v in self.m.vertices),19471)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),23332)
        self.assertLess(len(self.blob),1024*1024)
        self.assertLessEqual(len(self.m.doc['accessors']),256)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
    def test_closed_outward_solids_and_smooth_normals(self):
        m=self.m.neutral_model();assert_closed_models(self,{'lapras':m});assert_model_shading(self,{'lapras':m})
    def test_face_refinement_preserves_the_34_other_source_parts(self):
        digest=hashlib.sha256();count=0
        for p in self.m.neutral_model()['primitives']:
            if p['part'].startswith(('Eye /','Brow /','Nostril /','Mouth /')):continue
            for key,kind in [('positions',5126),('normals',5126),('indices',5125),('base_color',5126)]:
                digest.update(packed(p[key],kind))
            count+=1
        self.assertEqual(count,34)
        self.assertEqual(digest.hexdigest(),'04b4c4324b15652e0a1c9d26b0299374318a80bbf1e02330e50d0cfd77070cba')
    def test_verified_pack_blue_and_tan(self):
        for part,name in zip(self.m.parts,self.m.names):
            if name.startswith(('Body /','Neck /','Head /','Ear /','Horn /','Fore flipper /','Rear flipper /','Tail /')):rgb555=(8,16,28)
            elif name.startswith(('Belly /','Throat /','Jaw /')):rgb555=(28,21,13)
            else:continue
            rgb=[((v<<3)|(v>>2))/255 for v in rgb555]
            expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
            color=self.m.doc['materials'][part['material']]['pbrMetallicRoughness']['baseColorFactor']
            for a,b in zip(color,expected):self.assertAlmostEqual(a,b,places=7,msg=name)
    def test_anatomy_and_physical_height(self):
        for prefix,count in [('Ear /',4),('Fore flipper /',2),('Rear flipper /',2),('Shell spike /',8),('Neck spot /',8),('Horn /',1)]:
            self.assertEqual(sum(n.startswith(prefix) for n in self.m.names),count,prefix)
        points=[p for v in self.m.vertices for p in v]
        self.assertEqual(min(p[1] for p in points),0.)
        self.assertAlmostEqual(max(p[1] for p in points),1.50,places=6)
        self.assertGreater(max(p[0] for p in points)-min(p[0] for p in points),1.6)
    def test_hierarchy_inverse_binds_and_palette_independent_geometry(self):
        m=self.m
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['lapras/'+n for n,_,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for p,name in enumerate(m.names):
            if name.startswith(('Eye /','Head /','Ear /','Horn /','Brow /','Jaw /','Nostril /','Mouth /')):
                self.assertTrue(all(js==[3,0,0,0] for js in m.ids[p]))
    def test_refined_face_fits_the_stored_curved_muzzle(self):
        parts=self.m.neutral_model()['primitives']
        find=lambda prefix:next(p for p in parts if p['part'].startswith(prefix))
        head=find('Head /')
        for mark in parts:
            name=mark['part']
            if name.startswith(('Nostril /','Eye / expressive')):parent=head
            elif name.startswith('Eye / dark'):parent=find('Eye / expressive white almond '+name.split()[-1])
            elif name.startswith('Eye / small'):parent=find('Eye / dark upright pupil '+name.split()[-1])
            else:continue
            for i in range(0,len(mark['positions']),3):
                x,y,z=mark['positions'][i:i+3];hits=indexed_ray_intersections(parent,x,y)
                self.assertTrue(hits,name);self.assertLess(abs(z-max(hits)),.008,name)
            for k in range(0,len(mark['indices']),3):
                a,b,c=[mark['positions'][3*j:3*j+3] for j in mark['indices'][k:k+3]]
                if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
                for q in [tuple((a[j]+b[j]+c[j])/3 for j in range(3)),
                          *[tuple((u[j]+v[j])/2 for j in range(3)) for u,v in [(a,b),(b,c),(c,a)]]]:
                    hits=indexed_ray_intersections(parent,q[0],q[1]);self.assertTrue(hits,name)
                    self.assertGreater(q[2]-max(hits),.0001,(name,q))
        for side in (-1,1):
            eye=find('Eye / expressive white almond '+str(side))
            points=list(zip(*[iter(eye['positions'])]*3))
            inner=min(points,key=lambda p:side*p[0]);outer=max(points,key=lambda p:side*p[0])
            self.assertGreater(outer[1]-inner[1],.020)
        self.assertEqual(sum(p['part'].startswith('Nostril /') for p in parts),2)
        self.assertEqual(sum(p['part'].startswith('Mouth /') for p in parts),1)
        mouth=find('Mouth /');clearance=[]
        for i in range(0,len(mouth['positions']),3):
            x,y,z=mouth['positions'][i:i+3];hits=indexed_ray_intersections(head,x,y)
            self.assertTrue(hits,'mouth leaves the actual muzzle')
            clearance.append(z-max(hits))
        self.assertLess(max(clearance),.006,'mouth crease floats above the face')
        self.assertGreater(max(clearance),.002,'mouth crease is completely buried')
        self.assertGreater(min(clearance),-.004,'mouth crease cuts through the muzzle')
    def test_seated_shell_floor_and_animation_strain(self):
        m=self.m;fixed=[p for p,n in enumerate(m.names) if n.startswith(('Body /','Belly /','Shell'))]
        for clip in m.clips:
            for step in range(17):
                mats=m.matrices(clip,step/16)
                for p in fixed:self.assertEqual(m.posed(p,mats),m.vertices[p])
                for p in range(len(m.parts)):
                    posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,m.names[p]))
                    if step%4==0:
                        normals=m.posed(p,mats,True)
                        for i in range(0,len(m.indices[p]),3):
                            tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
                            self.assertGreater(cosine,0.,(clip,step,m.names[p],i));self.assertGreater(area,1e-12)
                            for a,b in zip(tri,tri[1:]+tri[:1]):
                                ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
                                self.assertTrue(.7<ratio<1.3,(clip,step,m.names[p],ratio))
    def test_neutral_endpoints_and_early_neck_brace(self):
        m=self.m
        for clip in m.clips:
            for t in (0.,1.):
                for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for joint in (1,2):
            values=[m.rotations('attack',i/100)[joint][0] for i in range(101)]
            self.assertGreater(values[10],max(values)*.4);self.assertGreater(values[20],max(values)*.9)
        for animation in m.doc['animations']:
            idle=animation['name'].endswith('.idle')
            self.assertEqual(animation['extras']['loopSuggested'],idle)
            self.assertEqual(animation['extras']['cueRelative'],not idle)
            self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})

if __name__=='__main__':unittest.main()
