#!/usr/bin/env python3
"""Decoded Bayleef anatomy, topology, planted contact, and animation regression.

Independent glTF matrix playback uses the shipped bytes, stored U8 weights,
local rotations, joint hierarchy and inverse binds. No Blender/pack is needed.
"""
import collections
import copy
import math
import os
from pathlib import Path
import unittest

import bayleef_glb as author
import bayleef_sculpt as sculpture
from skin_glb_test_support import IDENTITY, encode, raw_accessor, slerp, transform
from test_gengar_glb import Model, winding_alignment

ASSET=Path(os.environ.get('BAYLEEF_GLB',Path(__file__).resolve().parent.parent/'crates/crystal-voxel-view/models/battle_species/bayleef.glb'))
NAMES=['root','pelvis','chest','neck_base','head','foreleg_left','foreleg_right','hindleg_left','hindleg_right',
       'tail_base','tail_tip','crown_petiole','crown_mid_fold','crown_tip_fold','collar_left','collar_right']
PARENTS=[None,0,1,2,3,2,2,1,1,1,9,4,11,12,3,3]


def bounds(parts):
    points=[v for part in parts for v in part]
    return [[f(v[k] for v in points) for k in range(3)] for f in (min,max)]


class BayleefTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)

    def test_deterministic_canonical_container_and_rejection(self):
        m=self.m;d=m.doc
        self.assertEqual(author.export_bayleef(),self.blob)
        self.assertEqual(author.export_bayleef(author.decode_bayleef(self.blob)),self.blob)
        self.assertLess(len(self.blob),author.MAX_BYTES)
        self.assertEqual([len(d[k]) for k in ('meshes','skins','scenes','buffers')],[1,1,1,1])
        self.assertEqual(len(m.parts),33);self.assertEqual(len(set(m.names)),33)
        self.assertLessEqual(len(d['accessors']),256)
        self.assertEqual(d['extras']['source_sha256'],author.geometry_digest(m.neutral_model()))
        self.assertFalse(d.get('extensionsRequired'));self.assertNotIn('uri',d['buffers'][0])
        for view in d['bufferViews']:
            self.assertEqual(view['byteOffset']%4,0)
            self.assertLessEqual(view['byteOffset']+view['byteLength'],len(m.binary))
        for part in m.parts:
            self.assertEqual(set(part['attributes']),{'POSITION','NORMAL','JOINTS_0','WEIGHTS_0'})
            self.assertEqual(d['accessors'][part['indices']]['componentType'],5123)
        for edit in (lambda d:d['nodes'][1].update(translation=[0.,2.,0.]),
                     lambda d:d['buffers'][0].update(uri='external.bin'),
                     lambda d:d['animations'][0]['channels'][0]['target'].update(node=0)):
            changed=copy.deepcopy(d);edit(changed)
            with self.assertRaises(ValueError):author.decode_bayleef(encode(changed,m.binary))
        with self.assertRaises(ValueError):author.decode_bayleef(self.blob[:-4])
        with self.assertRaises(ValueError):author.decode_bayleef(self.blob+b'\0\0\0\0')

    def test_species_proportions_palette_notch_and_geometry_budget(self):
        m=self.m;lo,hi=bounds(m.vertices);blo,bhi=bounds([m.vertices[0]])
        self.assertEqual(lo[1],0.);self.assertAlmostEqual(hi[1],1.15,places=6)
        self.assertAlmostEqual(bhi[1],.982,places=6)
        self.assertAlmostEqual(blo[0],-bhi[0],places=7)
        self.assertEqual(sum(len(i)//3 for i in m.indices),3836)
        self.assertEqual(m.doc['extras']['reference_span'],1.15)
        colors={tuple(mat['pbrMetallicRoughness']['baseColorFactor']) for mat in m.doc['materials']}
        self.assertEqual(colors,{tuple(sculpture.linear(sculpture.PALETTE[n])) for n in ('body','leaf','ink','white')})
        # A long, genuinely slender middle neck separates the upper muzzle
        # from the haunch; the canonical bound never depends on a clip pose.
        neck=[p for p in m.vertices[0] if .610<p[1]<.710]
        haunch=[p for p in m.vertices[0] if .270<p[1]<.400]
        self.assertGreater(len(neck),50);self.assertLess(max(abs(p[0]) for p in neck),.115)
        self.assertGreater(max(abs(p[0]) for p in haunch),.225)
        blade=m.vertices[m.names.index(sculpture.LEAF)]
        # Explicit independent probes of the right edge around the true notch.
        for point in ((.241,1.0286,-.231),(.237,1.0235,-.299),(.350,.9499,-.354)):
            self.assertLess(min(math.dist(point,p) for p in blade),1e-6)
        self.assertLess(.237,(.241+.350)/2-.055)
        self.assertGreater(max(p[2] for p in blade)-min(p[2] for p in blade),.75)
        print('\nBayleef neutral bounds:',[lo,hi],'body:',[blo,bhi])

    def test_every_part_is_closed_connected_consistently_wound(self):
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

    def test_fitted_face_stays_clear_of_actual_body_facets(self):
        m=self.m
        for clip,t in ((None,0.),('idle',.25),('attack',.49),('hit',.21)):
            mats=m.matrices(clip,t);body=m.posed(0,mats)
            triangles=[tuple(body[k] for k in m.indices[0][i:i+3]) for i in range(0,len(m.indices[0]),3)]
            def front_depth(x,y):
                hits=[]
                for a,b,c in triangles:
                    if not min(a[0],b[0],c[0])-1e-8<=x<=max(a[0],b[0],c[0])+1e-8:continue
                    if not min(a[1],b[1],c[1])-1e-8<=y<=max(a[1],b[1],c[1])+1e-8:continue
                    det=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
                    if abs(det)<1e-12:continue
                    u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/det
                    v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/det;w=1.-u-v
                    if min(u,v,w)>-1e-7:hits.append(u*a[2]+v*b[2]+w*c[2])
                self.assertTrue(hits)
                return max(hits)
            for p,name in enumerate(m.names):
                if not name.startswith(('Eye /','Muzzle /')):continue
                points=m.posed(p,mats);normals=m.posed(p,mats,normal=True)
                for i in range(0,len(m.indices[p]),3):
                    ids=m.indices[p][i:i+3]
                    if sum(normals[j][2] for j in ids)<.1:continue
                    a,b,c=[points[j] for j in ids]
                    probes=[[sum(q[k] for q in (a,b,c))/3 for k in range(3)]]
                    probes += [[(u[k]+v[k])/2 for k in range(3)] for u,v in ((a,b),(b,c),(c,a))]
                    for x,y,z in probes:self.assertGreater(z-front_depth(x,y),.0006,(clip,name))

    def test_hierarchy_binds_and_normalized_weights(self):
        m=self.m;d=m.doc;used=set();blended=0;coincident={}
        self.assertEqual([n['name'] for n in d['nodes'][:-1]],['bayleef/'+n for n in NAMES])
        self.assertEqual(m.parents[:-1],PARENTS)
        self.assertEqual(d['nodes'][-1],{'name':'bayleef','mesh':0,'skin':0})
        self.assertEqual(d['scenes'][0]['nodes'],[0,16])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),8e-8)
        for p,part in enumerate(m.parts):
            raw=raw_accessor(m.doc,m.binary,part['attributes']['WEIGHTS_0'])
            self.assertTrue(all(sum(raw[i:i+4])==255 for i in range(0,len(raw),4)))
            for point,ids,weights in zip(m.vertices[p],m.ids[p],m.weights[p]):
                self.assertAlmostEqual(sum(weights),1.);blended+=sum(w>0 for w in weights)>1
                for j,w in zip(ids,weights):
                    self.assertTrue(0<=j<16)
                    if w:used.add(j)
                    else:self.assertEqual(j,0)
                if p==0:
                    if tuple(point) in coincident:self.assertEqual(coincident[tuple(point)],(ids,weights))
                    coincident[tuple(point)]=(ids,weights)
                if m.names[p].startswith(('Eye /','Muzzle /')):
                    self.assertEqual((ids,weights),([4,0,0,0],[1.,0.,0.,0.]))
        self.assertEqual(used,set(range(16)));self.assertGreater(blended,1000)
        blade=m.names.index(sculpture.LEAF)
        self.assertEqual({j for ids,ws in zip(m.ids[blade],m.weights[blade]) for j,w in zip(ids,ws) if w},{11,12,13})

    def test_four_soles_and_twelve_claws_stay_fixed_across_clips_and_blends(self):
        m=self.m;probes=[(0,i) for i,p in enumerate(m.vertices[0]) if p[1]<=.060]
        probes += [(p,i) for p,name in enumerate(m.names) if name.startswith('Claw /') for i in range(len(m.vertices[p]))]
        self.assertGreater(len(probes),400)
        self.assertEqual(len({(p[0]>0,p[2]>-.020) for p in m.vertices[0] if p[1]==0.}),4)
        for p,i in probes:self.assertEqual((m.ids[p][i],m.weights[p][i]),([0,0,0,0],[1.,0.,0.,0.]))
        for clip in ('idle','attack','hit'):
            for step in range(33):
                mats=m.matrices(clip,step/32)
                for p,i in probes:self.assertEqual(m.point(p,i,mats),m.vertices[p][i])
        for t in (.1,.5,.9):
            a=m.rotations('idle',t);b=m.rotations('hit',.23)
            mats=m.matrices(rotations=[slerp(x,y,.5) for x,y in zip(a,b)])
            for p,i in probes:self.assertEqual(m.point(p,i,mats),m.vertices[p][i])

    def test_stored_rotation_clips_have_neutral_endpoints_and_smooth_idle(self):
        m=self.m
        self.assertEqual([a['name'] for a in m.doc['animations']],['bayleef.idle','bayleef.attack','bayleef.hit'])
        for clip in m.doc['animations']:
            name=clip['name'].split('.')[-1]
            self.assertEqual(clip['extras']['loopSuggested'],name=='idle');self.assertEqual(clip['extras']['cueRelative'],name!='idle')
            self.assertEqual(len(clip['channels']),15)
            self.assertEqual({c['target']['node'] for c in clip['channels']},set(range(1,16)))
            self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
            self.assertEqual({s['interpolation'] for s in clip['samplers']},{'LINEAR'})
            self.assertEqual(len({s['input'] for s in clip['samplers']}),1)
            for _,times,keys in m.clips[name]:
                self.assertEqual(times,sorted(set(times)))
                self.assertAlmostEqual(times[-1],3.8 if name=='idle' else 1.,places=6)
                self.assertEqual(keys[0],[0.,0.,0.,1.]);self.assertEqual(keys[-1],keys[0])
                self.assertTrue(all(abs(sum(x*x for x in q)-1.)<2e-7 for q in keys))
            for t in (0.,1.):
                for matrix in m.matrices(name,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),8e-8)
        before=m.matrices('idle',1.-1e-4);after=m.matrices('idle',1e-4)
        for p in range(len(m.parts)):
            for rest,a,b in zip(m.vertices[p],m.posed(p,before),m.posed(p,after)):
                self.assertLess(math.dist(rest,[(x+y)*.5 for x,y in zip(a,b)]),2e-6)
        rotations=m.rotations('attack',.49)
        for a,b in ((2,3),(3,4),(11,12),(12,13),(14,15)):
            self.assertGreater(math.dist(rotations[a],rotations[b]),.005)

    def test_sampled_motion_no_floor_crossings_or_collapsed_faces(self):
        m=self.m;report={};edges=set()
        for j in range(0,len(m.indices[0]),3):
            tri=m.indices[0][j:j+3]
            for a,b in zip(tri,tri[1:]+tri[:1]):edges.add(tuple(sorted((a,b))))
        rest_edges=[(a,b,math.dist(m.vertices[0][a],m.vertices[0][b])) for a,b in edges]
        for clip in ('idle','attack','hit'):
            lo=[float('inf')]*3;hi=[float('-inf')]*3
            for i in range(33):
                mats=m.matrices(clip,i/32);posed=[m.posed(p,mats) for p in range(len(m.parts))];a,b=bounds(posed)
                self.assertGreaterEqual(a[1],-1e-8)
                lo=[min(x,y) for x,y in zip(lo,a)];hi=[max(x,y) for x,y in zip(hi,b)]
                for a,b,rest in rest_edges:
                    ratio=math.dist(posed[0][a],posed[0][b])/rest
                    self.assertTrue(.65<ratio<1.40,(clip,i,'body strain',ratio))
                if i in (0,5,10,16,24,32):
                    for p in range(len(m.parts)):
                        normals=m.posed(p,mats,normal=True)
                        for j in range(0,len(m.indices[p]),3):
                            cosine,area=winding_alignment(posed[p],normals,m.indices[p][j:j+3])
                            self.assertGreater(cosine,.0,(clip,i,m.names[p],j//3));self.assertGreater(area,1e-12)
            self.assertLess(hi[1],1.27)
            report[clip]=[[round(v,6) for v in lo],[round(v,6) for v in hi]]
        face=m.names.index('Eye / left fitted upright ink');leaf=m.names.index(sculpture.LEAF)
        for p,minimum in ((face,.06),(leaf,.09)):
            movement=max(math.dist(v,pv) for v,pv in zip(m.vertices[p],m.posed(p,m.matrices('attack',.49))))
            self.assertGreater(movement,minimum)
        print('Bayleef decoded animated bounds:',report)

    def test_quantized_weights_bound_pose_error(self):
        m=self.m;worst=0.
        for clip in ('idle','attack','hit'):
            for t in (.17,.34,.49,.72):
                mats=m.matrices(clip,t)
                for p in (0,m.names.index(sculpture.LEAF),m.names.index(sculpture.TAIL),10,14):
                    for i,point in enumerate(m.vertices[p]):
                        ids,ws=author.skin_weights(m.names[p],point);full=[0.,0.,0.]
                        for j,w in zip(ids,ws):
                            q=transform(mats[j],point);full=[a+w*b for a,b in zip(full,q)]
                        worst=max(worst,math.dist(full,m.point(p,i,mats)))
        self.assertLess(worst,.0004)
        print('Bayleef worst decoded UNORM8 skin error:',worst)


if __name__=='__main__':unittest.main()
