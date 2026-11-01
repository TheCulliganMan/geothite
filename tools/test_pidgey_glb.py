#!/usr/bin/env python3
"""Independent stored Pidgey topology and articulated anatomy checks."""
import math,os
from pathlib import Path
import unittest
import pidgey_glb as author
from sculpt_test_support import assert_closed_models,assert_model_shading
from skin_glb_test_support import IDENTITY
from test_gengar_glb import Model,winding_alignment
ASSET=Path(os.environ.get('CRYSTAL_PIDGEY_GLB',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/pidgey.glb'))

class PidgeyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_canonical_single_source_and_bounded_schema(self):
        self.assertEqual(self.blob,author.export_pidgey())
        self.assertEqual(len(self.m.parts),44)
        self.assertEqual(sum(len(v) for v in self.m.vertices),8757)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),12600)
        self.assertLess(len(self.blob),1024*1024)
        self.assertLessEqual(len(self.m.doc['accessors']),256)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
    def test_closed_outward_solids_and_smooth_normals(self):
        from pidgeotto_contours import components
        m=self.m.neutral_model();solids={p['part']+'/'+str(i):dict(primitives=[c]) for p in m['primitives'] for i,c in enumerate(components(p))}
        assert_closed_models(self,solids);assert_model_shading(self,solids)
    def test_anatomy_and_physical_height(self):
        for prefix,count in [('Wing /',2),('Flight feather /',4),('Tail /',3),('Crown /',2),('Toe /',6),('Rear toe /',2),('Claw /',6)]:
            self.assertEqual(sum(n.startswith(prefix) for n in self.m.names),count,prefix)
        points=[p for v in self.m.vertices for p in v]
        self.assertEqual(min(p[1] for p in points),0.)
        self.assertAlmostEqual(max(p[1] for p in points),.82,places=6)
        self.assertGreater(max(p[0] for p in points)-min(p[0] for p in points),.45)
    def test_hierarchy_inverse_binds_and_rigid_face(self):
        m=self.m
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['pidgey/'+n for n,_,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for p,name in enumerate(m.names):
            if name.startswith(('Eye /','Head /','Face /','Cheek /','Pupil /','Brow /','Upper beak /','Mouth /')):
                self.assertTrue(all(js==[2,0,0,0] for js in m.ids[p]))
    def test_revised_face_layers_fit_the_actual_closed_parent(self):
        from pidgeotto_sculpt import frame
        from sculpt_test_support import indexed_ray_intersections as hits
        parts=self.m.neutral_model()['primitives']
        def find(prefix,side=None):
            return next(p for p in parts if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+str(side))))
        pairs=[(find('Face /'),find('Head /'),0.),(find('Breast /'),find('Body /'),0.)]
        for side in (-1,1):
            for child,parent in [('Cheek /','Head /'),('Eye /','Cheek /'),('Pupil /','Eye /')]:
                pairs.append((find(child,side),find(parent,side if parent!='Head /' else None),side*math.radians(25)))
        for child,parent,angle in pairs:
            child,parent=frame(child,angle,True),frame(parent,angle,True)
            for i in range(0,len(child['indices']),3):
                a,b,c=[child['positions'][3*j:3*j+3] for j in child['indices'][i:i+3]]
                u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)]
                n=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
                if n[2]<=.5*math.sqrt(sum(x*x for x in n)):continue
                for point in [tuple((a[j]+b[j]+c[j])/3 for j in range(3)),*(tuple((q[j]+r[j])/2 for j in range(3)) for q,r in ((a,b),(b,c),(c,a)))]:
                    backing=hits(parent,*point[:2]);self.assertTrue(backing,(child['part'],i))
                    self.assertGreater(point[2]-max(backing),.0001,(child['part'],i,point))
    def test_complete_head_root_stays_inside_the_body_through_the_clips(self):
        from test_steelix_sculpt import occupied
        m=self.m;head=next(i for i,n in enumerate(m.names) if n.startswith('Head /'))
        body=next(i for i,n in enumerate(m.names) if n.startswith('Body /'))
        bottom=min(p[1] for p in m.vertices[head]);roots=[i for i,p in enumerate(m.vertices[head]) if abs(p[1]-bottom)<1e-7]
        self.assertGreaterEqual(len(roots),28)
        for clip in m.clips:
            for step in range(17):
                matrices=m.matrices(clip,step/16);posed_body=m.posed(body,matrices)
                parent=dict(positions=[v for p in posed_body for v in p],indices=m.indices[body])
                for i in roots:self.assertTrue(occupied(parent,m.point(head,i,matrices)),(clip,step,i))
    def test_planted_soles_floor_and_animation_strain(self):
        m=self.m;fixed=[p for p,n in enumerate(m.names) if n.startswith(('Foot /','Toe /','Rear toe /','Claw /'))]
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
    def test_feather_chain_owners_and_delayed_response(self):
        m=self.m
        for p,name in enumerate(m.names):
            left=sum(v[0] for v in m.vertices[p])<0
            owner=8 if name.startswith('Tail /') else 9 if name.startswith('Crown /') else (5 if left else 7) if name.startswith('Flight feather /') else None
            if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]))
        self.assertGreater(m.rotations('attack',.28)[8][0],m.rotations('attack',.18)[8][0])
    def test_neutral_endpoints_and_early_torso_brace(self):
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
