#!/usr/bin/env python3
"""Check actual stored Spearow geometry and decoded skin/clip behavior.

Uses the existing independent GLB decoder and transform reference, not the
recipe's animation function. No Blender, Cargo, pack, or duplicate mesh input.
"""
import math
import os
from pathlib import Path
import unittest

import spearow_glb as author
from skin_glb_test_support import IDENTITY, raw_accessor
from test_gengar_glb import Model, winding_alignment

ASSET = Path(os.environ.get('SPEAROW_GLB', Path(__file__).resolve().parent.parent /
    'crates/crystal-voxel-view/models/battle_species/spearow.glb'))
EXPECTED_JOINTS = ['root','torso','head','lower_beak','wing_left','flight_feathers_left',
                   'wing_right','flight_feathers_right','tail_fan','crown']
EXPECTED_PARENTS = [None,0,1,2,1,4,1,6,1,2]


def bounds(parts):
    points=[v for p in parts for v in p]
    return ([min(v[a] for v in points) for a in range(3)],
            [max(v[a] for v in points) for a in range(3)])


class SpearowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)

    def test_one_compact_canonical_asset_and_bounded_skin(self):
        m=self.m;d=m.doc
        self.assertEqual(author.export_spearow(),self.blob)
        self.assertLess(len(self.blob),250000)
        self.assertEqual([len(d[k]) for k in ('meshes','skins','scenes','buffers')],[1,1,1,1])
        self.assertEqual(len(m.parts),28)
        self.assertLess(sum(map(len,m.vertices)),4000)
        self.assertLess(sum(len(ix)//3 for ix in m.indices),3000)
        self.assertLessEqual(len(d['accessors']),256)
        self.assertEqual(len(set(m.names)),len(m.names))
        self.assertEqual(d['extras']['source_sha256'],author.geometry_digest(m.neutral_model()))
        self.assertEqual([n['name'] for n in d['nodes'][:-1]],['spearow/'+n for n in EXPECTED_JOINTS])
        self.assertEqual(m.parents[:-1],EXPECTED_PARENTS)
        self.assertEqual(d['nodes'][-1],{'name':'spearow','mesh':0,'skin':0})
        self.assertFalse(d.get('extensionsRequired'))
        self.assertNotIn('uri',d['buffers'][0])
        for mat in d['materials']:
            self.assertTrue(mat['name'].startswith('spearow/'))
            self.assertEqual(mat['pbrMetallicRoughness']['metallicFactor'],0)
            self.assertEqual(mat['pbrMetallicRoughness']['roughnessFactor'],1)

    def test_neutral_height_ground_contact_and_real_winding(self):
        m=self.m;lo,hi=bounds(m.vertices)
        self.assertAlmostEqual(lo[1],0.,places=7)
        self.assertAlmostEqual(hi[1],.75,places=7)
        for p,part in enumerate(m.parts):
            self.assertEqual(set(part['attributes']),{'POSITION','NORMAL','JOINTS_0','WEIGHTS_0'})
            self.assertEqual(m.doc['accessors'][part['indices']]['componentType'],5123)
            for n in m.normals[p]:self.assertAlmostEqual(math.sqrt(sum(x*x for x in n)),1.,places=6)
            for i in range(0,len(m.indices[p]),3):
                cosine,area=winding_alignment(m.vertices[p],m.normals[p],m.indices[p][i:i+3])
                self.assertGreater(cosine,.01,(m.names[p],i//3))
                self.assertGreater(area,1e-11)

    def test_normalized_weights_identity_bind_and_anatomy_ownership(self):
        m=self.m;used=set();blended=0
        for matrix in m.matrices():
            self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),4e-8)
        for p,part in enumerate(m.parts):
            raw=raw_accessor(m.doc,m.binary,part['attributes']['WEIGHTS_0'])
            self.assertTrue(all(sum(raw[i:i+4])==255 for i in range(0,len(raw),4)))
            for ids,ws in zip(m.ids[p],m.weights[p]):
                self.assertAlmostEqual(sum(ws),1)
                for j,w in zip(ids,ws):
                    self.assertTrue(0<=j<10)
                    if w:used.add(j)
                    else:self.assertEqual(j,0)
                blended+=sum(w>0 for w in ws)>1
                if m.names[p].startswith('Feet /'):self.assertEqual(ids,[0,0,0,0])
                if m.names[p].startswith('Eye '):self.assertEqual(ids,[2,0,0,0])
                if m.names[p].startswith('Wing left'):self.assertTrue(set(j for j,w in zip(ids,ws) if w)<=set((4,5)))
                if m.names[p].startswith('Wing right'):self.assertTrue(set(j for j,w in zip(ids,ws) if w)<=set((6,7)))
        self.assertEqual(used,set(range(10)))
        self.assertGreater(blended,50)

    def test_clip_semantics_neutral_endpoints_and_fixed_toes(self):
        m=self.m
        self.assertEqual([a['name'] for a in m.doc['animations']],['spearow.idle','spearow.attack','spearow.hit'])
        feet=[p for p,n in enumerate(m.names) if n.startswith('Feet /')]
        for clip in m.doc['animations']:
            name=clip['name'].split('.')[-1]
            self.assertEqual(clip['extras']['loopSuggested'],name=='idle')
            self.assertEqual(clip['extras']['cueRelative'],name!='idle')
            self.assertEqual(len(clip['channels']),9)
            for channel in clip['channels']:
                self.assertEqual(channel['target']['path'],'rotation')
                self.assertNotEqual(channel['target']['node'],0)
            for t in (0.,1.):
                for matrix in m.matrices(name,t):
                    self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),4e-8)
            for i in range(41):
                matrices=m.matrices(name,i/40)
                for p in feet:
                    self.assertEqual(m.posed(p,matrices),m.vertices[p])
        # The idle loop also approaches the seam with matching velocity.
        # Endpoint equality alone would miss a visible stutter on repetition.
        before=m.matrices('idle',1.-1e-4);after=m.matrices('idle',1e-4)
        for p in range(len(m.parts)):
            for rest,a,b in zip(m.vertices[p],m.posed(p,before),m.posed(p,after)):
                midpoint=[(x+y)*.5 for x,y in zip(a,b)]
                self.assertLess(math.dist(rest,midpoint),1e-6)

    def test_sampled_motion_bounds_no_floor_crossing_or_collapsed_faces(self):
        m=self.m;report={}
        for clip in ('idle','attack','hit'):
            lo=[float('inf')]*3;hi=[float('-inf')]*3
            for i in range(31):
                mats=m.matrices(clip,i/30)
                posed=[m.posed(p,mats) for p in range(len(m.parts))]
                a,b=bounds(posed)
                lo=[min(x,y) for x,y in zip(lo,a)];hi=[max(x,y) for x,y in zip(hi,b)]
                self.assertGreaterEqual(a[1],-1e-8)
                if i in (0,5,12,20,30):
                    for p in range(len(m.parts)):
                        normals=m.posed(p,mats,normal=True)
                        for j in range(0,len(m.indices[p]),3):
                            cosine,area=winding_alignment(posed[p],normals,m.indices[p][j:j+3])
                            self.assertGreater(cosine,0.,(clip,i,m.names[p],j//3))
                            self.assertGreater(area,1e-11)
            self.assertLess(hi[1],.82)
            self.assertLess(max(abs(lo[0]),abs(hi[0])),.52)
            report[clip]=[[round(v,6) for v in lo],[round(v,6) for v in hi]]
        print('\nSpearow decoded neutral bounds:',bounds(m.vertices))
        print('Spearow decoded animated bounds:',report)
        # Stored motion must actually move both wings and the head measurably.
        for prefix,minimum in (('Wing left',.018),('Wing right',.018),('Head /',.035)):
            distance=max(math.dist(v,p) for i,n in enumerate(m.names) if n.startswith(prefix)
                for v,p in zip(m.vertices[i],m.posed(i,m.matrices('attack',.39))))
            self.assertGreater(distance,minimum)

    def test_attack_pecks_on_cue_with_early_wing_balance(self):
        m=self.m
        for step in range(1,26):
            q=m.rotations('attack',step/100)
            self.assertGreater(q[1][0],0.,step)
            self.assertGreater(q[2][0],0.,step)
        peak=max(m.rotations('attack',i/100)[2][0] for i in range(101))
        self.assertGreater(m.rotations('attack',.10)[2][0],peak*.45)
        self.assertGreater(m.rotations('attack',.20)[2][0],peak*.90)
        # Wing balance and beak opening precede the forward peck peak.
        early=m.rotations('attack',.10)
        for joint in (4,6):
            wing_peak=max(abs(m.rotations('attack',i/100)[joint][2]) for i in range(101))
            self.assertGreater(abs(early[joint][2]),wing_peak*.70)
        beak_peak=max(m.rotations('attack',i/100)[3][0] for i in range(101))
        self.assertGreater(early[3][0],beak_peak*.70)


if __name__=='__main__': unittest.main()
