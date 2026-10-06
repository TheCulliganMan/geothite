#!/usr/bin/env python3
"""Independent decoded geometry, contact, topology, skin and clip regressions.

These assertions operate on the shipped GLB bytes and reference matrix math;
no Blender, sourcepack, runtime rendering, or graphics dependency is needed.
"""
import collections
import math
import os
from pathlib import Path
import unittest

import chikorita_glb as author
import chikorita_sculpt as sculpture
from skin_glb_test_support import IDENTITY, raw_accessor, transform
from test_gengar_glb import Model, winding_alignment

ASSET=Path(os.environ.get('CHIKORITA_GLB',Path(__file__).resolve().parent.parent/'crates/crystal-voxel-view/models/actor_props/battle_chikorita.glb'))
EXPECTED_NAMES=['root','torso','head','foreleg_left','foreleg_right','hindleg_left','hindleg_right',
                'tail_nub','leaf_petiole','leaf_mid_fold','leaf_tip_fold']
EXPECTED_PARENTS=[None,0,1,1,1,1,1,1,2,8,9]


def bounds(parts):
    points=[v for p in parts for v in p]
    return [[f(v[k] for v in points) for k in range(3)] for f in (min,max)]


class ChikoritaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)

    def test_canonical_rebuild_and_bounded_complete_schema(self):
        m=self.m;d=m.doc
        self.assertEqual(author.export_chikorita(),self.blob)
        self.assertEqual(author.export_chikorita(author.decode_chikorita(self.blob)),self.blob)
        self.assertLess(len(self.blob),author.MAX_BYTES)
        self.assertEqual([len(d[k]) for k in ('meshes','skins','scenes','buffers')],[1,1,1,1])
        self.assertEqual(len(m.parts),19);self.assertEqual(len(set(m.names)),19)
        self.assertLessEqual(len(d['accessors']),256)
        self.assertEqual(d['extras']['source_sha256'],author.geometry_digest(m.neutral_model()))
        self.assertEqual([n['name'] for n in d['nodes'][:-1]],['chikorita/'+n for n in EXPECTED_NAMES])
        self.assertEqual(m.parents[:-1],EXPECTED_PARENTS)
        self.assertEqual(d['nodes'][-1],{'name':'chikorita','mesh':0,'skin':0})
        self.assertEqual(d['scenes'][0]['nodes'],[0,11]);self.assertFalse(d.get('extensionsRequired'))
        self.assertNotIn('uri',d['buffers'][0])
        for v in d['bufferViews']:
            self.assertEqual(v['byteOffset']%4,0)
            self.assertLessEqual(v['byteOffset']+v['byteLength'],len(m.binary))
        for p in m.parts:
            self.assertEqual(set(p['attributes']),{'POSITION','NORMAL','JOINTS_0','WEIGHTS_0'})
            self.assertEqual(d['accessors'][p['indices']]['componentType'],5123)
        with self.assertRaises(ValueError):author.decode_chikorita(self.blob[:-4])
        changed=bytearray(self.blob);changed[-40]^=1
        with self.assertRaises(ValueError):author.decode_chikorita(changed)

    def test_anatomy_height_body_center_palette_and_triangle_budget(self):
        m=self.m;lo,hi=bounds(m.vertices);blo,bhi=bounds([m.vertices[0]])
        self.assertEqual(lo[1],0.);self.assertAlmostEqual(hi[1],1.,places=7)
        self.assertAlmostEqual(bhi[1],.688,places=7)
        self.assertAlmostEqual(blo[0],-bhi[0],places=7)
        self.assertEqual(sum(len(i)//3 for i in m.indices),3220)
        self.assertEqual(len(m.indices[-1])//3,92)
        self.assertEqual(sum(len(i)//3 for i,n in zip(m.indices,m.names) if n.startswith('Collar')),160)
        self.assertEqual(m.doc['extras']['reference_span'],1.)
        colors={tuple(mat['pbrMetallicRoughness']['baseColorFactor']) for mat in m.doc['materials']}
        self.assertEqual(colors,{tuple(sculpture.linear(sculpture.PALETTE[n])) for n in ('body','leaf','ink','white')})
        tail=m.vertices[m.names.index(sculpture.TAIL)]
        self.assertLess(min(p[2] for p in tail),-.36)
        self.assertGreater(max(p[1] for p in tail),.28)
        print('\nChikorita neutral bounds:',[lo,hi],'body:',[blo,bhi])

    def test_every_component_is_closed_with_consistent_winding(self):
        m=self.m
        for p,name in enumerate(m.names):
            edges=collections.Counter();directed=collections.Counter();neighbors=collections.defaultdict(set)
            for i in range(0,len(m.indices[p]),3):
                tri=m.indices[p][i:i+3];cosine,area=winding_alignment(m.vertices[p],m.normals[p],tri)
                self.assertGreater(cosine,.99,(name,i//3));self.assertGreater(area,1e-12,(name,i//3))
                points=[tuple(m.vertices[p][j]) for j in tri]
                for a,b in zip(points,points[1:]+points[:1]):
                    edges[tuple(sorted((a,b)))]+=1;directed[(a,b)]+=1;neighbors[a].add(b);neighbors[b].add(a)
            self.assertTrue(all(n==2 for n in edges.values()),name)
            self.assertTrue(all(directed[(b,a)]==1 for a,b in directed),name)
            todo=[next(iter(neighbors))];seen=set()
            while todo:
                point=todo.pop()
                if point not in seen:seen.add(point);todo.extend(neighbors[point]-seen)
            self.assertEqual(len(seen),len(neighbors),name)
            self.assertEqual(len(neighbors)-len(edges)+len(m.indices[p])//3,2,name)
            for normal in m.normals[p]:self.assertAlmostEqual(math.sqrt(sum(x*x for x in normal)),1.,places=6)

    def test_binds_weights_and_all_anatomical_joints_are_used(self):
        m=self.m;used=set();blend=0;coincident={}
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),6e-8)
        for p,part in enumerate(m.parts):
            raw=raw_accessor(m.doc,m.binary,part['attributes']['WEIGHTS_0'])
            self.assertTrue(all(sum(raw[i:i+4])==255 for i in range(0,len(raw),4)))
            for point,ids,ws in zip(m.vertices[p],m.ids[p],m.weights[p]):
                self.assertAlmostEqual(sum(ws),1.);blend+=sum(w>0 for w in ws)>1
                for j,w in zip(ids,ws):
                    self.assertTrue(0<=j<11)
                    if w:used.add(j)
                    else:self.assertEqual(j,0)
                if p==0:
                    if tuple(point) in coincident:self.assertEqual(coincident[tuple(point)],(ids,ws))
                    coincident[tuple(point)]=(ids,ws)
                if m.names[p].startswith(('Eye /','Mouth /')):self.assertEqual((ids,ws),([2,0,0,0],[1.,0.,0.,0.]))
        self.assertEqual(used,set(range(11)));self.assertGreater(blend,400)
        leaf=m.names.index(sculpture.LEAF)
        self.assertEqual({j for ids,ws in zip(m.ids[leaf],m.weights[leaf]) for j,w in zip(ids,ws) if w},{8,9,10})

    def test_four_soles_stay_planted_across_clips_and_blends(self):
        m=self.m;probes=[i for i,p in enumerate(m.vertices[0]) if p[1]<=.038]
        self.assertGreater(len(probes),100)
        quadrants={(p[0]>0,p[2]>-.015) for p in m.vertices[0] if p[1]==0.}
        self.assertEqual(len(quadrants),4)
        for i in probes:self.assertEqual((m.ids[0][i],m.weights[0][i]),([0,0,0,0],[1.,0.,0.,0.]))
        for clip in ('idle','attack','hit'):
            for step in range(33):
                mats=m.matrices(clip,step/32)
                for i in probes:self.assertEqual(m.point(0,i,mats),m.vertices[0][i])
        # glTF rotations can be blended at any cue boundary without relocating
        # the root or peeling a contact vertex away from the floor.
        from skin_glb_test_support import slerp
        for t in (.1,.5,.9):
            a=m.rotations('idle',t);b=m.rotations('hit',.23)
            mats=m.matrices(rotations=[slerp(x,y,.5) for x,y in zip(a,b)])
            for i in probes:self.assertEqual(m.point(0,i,mats),m.vertices[0][i])

    def test_real_stored_clips_neutral_endpoints_and_periodic_idle_velocity(self):
        m=self.m
        self.assertEqual([a['name'] for a in m.doc['animations']],['chikorita.idle','chikorita.attack','chikorita.hit'])
        for clip in m.doc['animations']:
            name=clip['name'].split('.')[-1]
            self.assertEqual(clip['extras']['loopSuggested'],name=='idle')
            self.assertEqual(clip['extras']['cueRelative'],name!='idle')
            self.assertEqual(len(clip['channels']),10)
            for channel in clip['channels']:
                self.assertEqual(channel['target']['path'],'rotation');self.assertNotEqual(channel['target']['node'],0)
            for t in (0.,1.):
                for matrix in m.matrices(name,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),6e-8)
        before=m.matrices('idle',1.-1e-4);after=m.matrices('idle',1e-4)
        for p in range(len(m.parts)):
            for rest,a,b in zip(m.vertices[p],m.posed(p,before),m.posed(p,after)):
                self.assertLess(math.dist(rest,[(x+y)*.5 for x,y in zip(a,b)]),2e-6)
        # The delayed blade must articulate relative to the head, not simply
        # follow the root as a single rigid prop.
        rots=m.rotations('attack',.34)
        self.assertGreater(math.dist(rots[8],rots[9]),.005)
        self.assertGreater(math.dist(rots[9],rots[10]),.005)

    def test_sampled_motion_no_floor_crossings_or_collapsed_triangles(self):
        m=self.m;report={}
        for clip in ('idle','attack','hit'):
            lo=[float('inf')]*3;hi=[float('-inf')]*3
            for i in range(25):
                mats=m.matrices(clip,i/24);posed=[m.posed(p,mats) for p in range(len(m.parts))];a,b=bounds(posed)
                self.assertGreaterEqual(a[1],-1e-8)
                lo=[min(x,y) for x,y in zip(lo,a)];hi=[max(x,y) for x,y in zip(hi,b)]
                if i in (0,4,8,12,18,24):
                    for p in range(len(m.parts)):
                        normals=m.posed(p,mats,normal=True)
                        for j in range(0,len(m.indices[p]),3):
                            cosine,area=winding_alignment(posed[p],normals,m.indices[p][j:j+3])
                            self.assertGreater(cosine,.0,(clip,i,m.names[p],j//3));self.assertGreater(area,1e-12)
            self.assertLess(hi[1],1.12)
            report[clip]=[[round(v,6) for v in lo],[round(v,6) for v in hi]]
        face=m.names.index('Eye / left fitted tall ink');leaf=m.names.index(sculpture.LEAF)
        for p,minimum in ((face,.04),(leaf,.07)):
            movement=max(math.dist(v,pv) for v,pv in zip(m.vertices[p],m.posed(p,m.matrices('attack',.34))))
            self.assertGreater(movement,minimum)
        print('Chikorita decoded animated bounds:',report)

    def test_quantized_weights_do_not_change_visible_pose(self):
        m=self.m;worst=0.
        for clip in ('idle','attack','hit'):
            for t in (.16,.34,.50,.72):
                mats=m.matrices(clip,t)
                for p in (0,m.names.index(sculpture.LEAF)):
                    for i,point in enumerate(m.vertices[p]):
                        ids,ws=author.skin_weights(m.names[p],point);full=[0.,0.,0.]
                        for j,w in zip(ids,ws):
                            q=transform(mats[j],point)
                            full=[a+w*b for a,b in zip(full,q)]
                        worst=max(worst,math.dist(full,m.point(p,i,mats)))
        self.assertLess(worst,.0004)
        print('Chikorita worst decoded UNORM8 skin error:',worst)


if __name__=='__main__':unittest.main()
