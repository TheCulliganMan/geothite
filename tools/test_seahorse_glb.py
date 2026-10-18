#!/usr/bin/env python3
"""Independently decode coiled tails, hollow snouts and coherent fin surfaces."""
import copy
import math
import unittest
from pathlib import Path
import seahorse_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces
from sculpt_test_support import indexed_ray_intersections as rays
from bovine_sculpt import reverse_front
from sculpt_geometry import ellipsoid


def surface_hits(part,x,y):
    # One physical crossing can hit both triangles at a shared edge. Pair
    # distinct surfaces, rather than making two zero-width intervals there.
    result=[]
    for z in sorted(rays(part,x,y)):
        if not result or abs(z-result[-1])>1e-9:result.append(z)
    return result


def occupied_overlap(a,b):
    # Same occupied-interval grid/threshold as parts_overlap, with indexed
    # triangle candidates so repeated posed thin-root checks remain cheap.
    xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]))
    yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
    if xh<=xl or yh<=yl:return False
    for i in range(1,19):
        for j in range(1,23):
            x=xl+(xh-xl)*i/19;y=yl+(yh-yl)*j/23
            aa=surface_hits(a,x,y);bb=surface_hits(b,x,y)
            if any(min(ah,bh)-max(al,bl)>.00025 for al,ah in zip(aa[::2],aa[1::2]) for bl,bh in zip(bb[::2],bb[1::2])):return True
    return False


def find(parts,prefix,side=None):
    return next(p for p in parts if p['part'].startswith(prefix) and (side is None or p['part'].split()[-1]==str(side)))


def markings(parts,species):
    for p in parts:
        title = p['part'];side = title.split()[-1];parent = None
        if title.startswith('Belly /'):parent = find(parts,'Body /')
        elif title.startswith('Belly fold /'):parent = find(parts,'Belly /')
        elif title.startswith('Body seam /'):yield reverse_front(p),reverse_front(find(parts,'Body /'));continue
        elif title.startswith('Eye socket /'):parent = find(parts,'Head /')
        elif title.startswith('Eye white /'):parent = find(parts,'Eye socket /',side)
        elif title.startswith('Iris /'):parent = find(parts,'Eye white /',side)
        elif title.startswith('Pupil /'):parent = find(parts,'Eye white /' if species=='seadra' else 'Iris /',side)
        elif title.startswith('Glint /'):parent = find(parts,'Pupil /',side)
        elif title.startswith(('Fin ray /','Fin tip /')):
            parent = find(parts,'Dorsal fin /' if species=='kingdra' else 'Back fin /',title.split()[-2])
        if parent:
            yield p,parent
            if title.startswith('Fin tip /'):yield reverse_front(p),reverse_front(parent)


def inverse_rigid(part,matrix,reverse=False):
    positions = []
    for i in range(0,len(part['positions']),3):
        point = part['positions'][i:i+3]
        if reverse:point = [-point[0],point[1],-point[2]]
        delta = [point[k]-matrix[12+k] for k in range(3)]
        local = [sum(matrix[axis*4+k]*delta[k] for k in range(3)) for axis in range(3)]
        if reverse:local = [-local[0],local[1],-local[2]]
        positions.extend(local)
    return dict(part,positions=positions)


class SeahorseSkinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root = Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
        cls.blobs = {s:(root/(s+'.glb')).read_bytes() for s in author.SPECIES}
        for s,b in cls.blobs.items():
            if b!=author.export_seahorse(s):raise AssertionError('stored seahorse differs from canonical exporter: '+s)
        cls.runtime = {s:Model(b) for s,b in cls.blobs.items()}
        cls.logical = {s:logical_model(m) for s,m in cls.runtime.items()}
        cls.models = {s:m.neutral_model() for s,m in cls.logical.items()}

    def test_exact_logical_geometry_packing_hierarchy_and_existing_limits(self):
        for s,m in self.logical.items():
            self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s])
            self.assertEqual(author.geometry_digest(author.decode_seahorse(self.blobs[s],s)),author.REST_DIGESTS[s])
            self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,pivot in author.JOINTS[s]])
            self.assertLessEqual(sum(len(v) for v in m.vertices),20000)
            self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000)
            self.assertLessEqual(len(self.runtime[s].parts),64)
            self.assertLessEqual(len(m.doc['accessors']),256)
            self.assertLessEqual(len(self.blobs[s]),1024*1024)
            self.assertEqual(len(m.names),47 if s=='kingdra' else 36 if s=='horsea' else 34)
            for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)

    def test_stored_closed_outward_topology_and_actual_shading(self):
        assert_closed_models(self,self.models);assert_model_shading(self,self.models)

    def test_independent_controls_and_original_tail_centerline_weights(self):
        for s,m in self.logical.items():
            # The identity waterline root is structural; these swimmers have
            # no stationary feet/vertices assigned directly to it.
            self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(1,len(author.JOINTS[s]))))
            rotations = m.rotations('attack',.34)
            for j in range(1,len(author.JOINTS[s])):
                isolated = [[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j] = rotations[j];matrices = m.matrices(rotations=isolated)
                self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,matrices))),.0005,(s,j))
            for i,title in enumerate(m.names):
                if title.startswith(('Head /','Snout /','Eye socket /','Eye white /','Iris /','Pupil /','Glint /','Cheek /')):
                    self.assertTrue(all(js==[2,0,0,0] for js in m.ids[i]))
                if title.startswith('Tail /'):
                    self.assertEqual({j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w},{7,8,9})
                    self.assertGreater(sum(sum(w>0 for w in ws)>1 for ws in m.weights[i]),len(m.weights[i])*.20)

    def test_waterline_strain_winding_and_neutral_endpoints(self):
        for s,m in self.logical.items():
            for clip in m.clips:
                for step in range(9):
                    matrices = m.matrices(clip,step/8);low = 1e9
                    self.assertLess(max(abs(a-b) for a,b in zip(matrices[0],IDENTITY)),1e-7)
                    for i,title in enumerate(m.names):
                        posed = m.posed(i,matrices);normals = m.posed(i,matrices,True);low = min(low,min(v[1] for v in posed))
                        self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,title,step))
                        if title.startswith('Tail /'):
                            self.assertLess(max(abs(a[1]-b[1]) for a,b in zip(posed,m.vertices[i])),1e-7)
                        for k in range(0,len(m.indices[i]),3):
                            tri = m.indices[i][k:k+3];cosine,area = winding_alignment(posed,normals,tri)
                            self.assertGreater(cosine,0.,(s,clip,title,step));self.assertGreater(area,1e-11)
                            for a,b in zip(tri,tri[1:]+tri[:1]):
                                ratio = math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])
                                self.assertTrue(.75<ratio<1.3,(s,clip,title,step,ratio))
                    self.assertLess(abs(low),1e-7)
                    if step in (0,8):
                        for matrix in matrices:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)

    def test_roots_and_uncrossed_coils_survive_clips(self):
        for s,m in self.logical.items():
            for clip in m.clips:
                for t in (0.,.34,.67):
                    matrices = m.matrices(clip,t)
                    parts = [dict(p,positions=sum(m.posed(i,matrices),[])) for i,p in enumerate(self.models[s]['primitives'])]
                    body = find(parts,'Body /');head = find(parts,'Head /');tail = find(parts,'Tail /')
                    for a,b in [(body,head),(body,tail),(head,find(parts,'Snout /'))]:
                        self.assertTrue(occupied_overlap(a,b),(s,clip,t,a['part'],b['part']))
                    self.assertIsNone(crossed_faces(tail),(s,clip,t))
                    tail_index = m.names.index(tail['part']);segments = 24 if s=='kingdra' else 40
                    cap = set(m.indices[tail_index][:(segments-2)*3]);self.assertEqual(len(cap),segments)
                    for vertex in cap:
                        x,y,z=tail['positions'][vertex*3:vertex*3+3];hits=surface_hits(body,x,y)
                        self.assertTrue(any(a+.0002<z<b-.0002 for a,b in zip(hits[::2],hits[1::2])),(s,clip,t,'exposed tail cap',vertex))
                        for outside in (hits[0]-.01,hits[-1]+.01):
                            self.assertFalse(any(a+.0002<outside<b-.0002 for a,b in zip(hits[::2],hits[1::2])))
                    for p in parts:
                        title = p['part'];parent = None
                        if title.startswith(('Back fin /','Dorsal fin /','Armor /','Dorsal horn /')):parent = body
                        elif title.startswith(('Head fin /','Neck fin /','Cheek /','Crown /')):parent = head
                        elif title.startswith('Crown branch /'):parent = find(parts,'Crown /',title.split()[-2])
                        elif title.startswith('Dorsal fork /'):parent = find(parts,'Dorsal horn /',title.split()[-1])
                        if parent:self.assertTrue(occupied_overlap(parent,p),(s,clip,t,title))

    def test_fitted_caps_clear_actual_parents_through_pose(self):
        for s,m in self.logical.items():
            original = list(markings(self.models[s]['primitives'],s))
            for clip in m.clips:
                for t in (0.,.34,.67):
                    matrices = m.matrices(clip,t)
                    parts = [dict(p,positions=sum(m.posed(i,matrices),[])) for i,p in enumerate(self.models[s]['primitives'])]
                    for pair,((old,_),(child,parent)) in enumerate(zip(original,markings(parts,s))):
                        pi = m.names.index(parent['part']);ci = m.names.index(child['part']);owner = m.ids[pi][0][0]
                        for i in (pi,ci):self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]))
                        # Back-tip markers/body seams use a reflected front frame.
                        reverse = child['part'].startswith('Body seam /') or child['part'].startswith('Fin tip /') and pair>0 and original[pair-1][0]['part']==child['part']
                        child = inverse_rigid(child,matrices[owner],reverse);parent = inverse_rigid(parent,matrices[owner],reverse)
                        vs = [child['positions'][i:i+3] for i in range(0,len(child['positions']),3)];oldvs = [old['positions'][i:i+3] for i in range(0,len(old['positions']),3)]
                        for k in range(0,len(child['indices']),3):
                            ids = child['indices'][k:k+3];a,b,c = [oldvs[i] for i in ids]
                            if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
                            tri = [vs[i] for i in ids]
                            for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1,0,0),(0,1,0),(0,0,1)):
                                x,y,z = [sum(weights[j]*tri[j][axis] for j in range(3)) for axis in range(3)];hits = rays(parent,x,y)
                                self.assertTrue(hits,(s,child['part']));gap = z-max(hits)
                                self.assertGreater(gap,.0001,(s,clip,t,child['part'],gap));self.assertLess(gap,.018)

    def test_actual_snout_space_stays_empty_with_negative_control(self):
        def clear(parts,species):
            outer = find(parts,'Snout /');zmax = max(outer['positions'][2::3]);front = [outer['positions'][i:i+3] for i in range(0,len(outer['positions']),3) if outer['positions'][i+2]>zmax-.002]
            x,y = [sum(v[axis] for v in front)/len(front) for axis in (0,1)]
            return all((hits:= [z for p in parts for z in rays(p,x+dx,y+dy)]) and max(hits)<zmax-(.20 if species=='kingdra' else .14) for dx in (-.003,0.,.003) for dy in (-.003,0.,.003))
        for s,m in self.logical.items():
            for clip in m.clips:
                for t in (0.,.34,.67):
                    matrices = m.matrices(clip,t)
                    parts = [inverse_rigid(dict(p,positions=sum(m.posed(i,matrices),[])),matrices[2]) for i,p in enumerate(self.models[s]['primitives'])]
                    self.assertTrue(clear(parts,s),(s,clip,t))
            parts = self.models[s]['primitives'];outer = find(parts,'Snout /');zmax = max(outer['positions'][2::3]);front = [outer['positions'][i:i+3] for i in range(0,len(outer['positions']),3) if outer['positions'][i+2]>zmax-.002]
            x,y = [sum(v[axis] for v in front)/len(front) for axis in (0,1)]
            filled = ellipsoid('negative control',(.3,.5,.7),(x,y,zmax-.015),(.015,.015,.010),16,12)
            self.assertFalse(clear(parts+[filled],s))

    def test_source_byte_and_species_mutations_are_rejected(self):
        for s,b in self.blobs.items():
            mutated = bytearray(b);mutated[-4] ^= 1
            with self.assertRaises(ValueError):author.decode_seahorse(bytes(mutated),s)
            source = copy.deepcopy(author.authoring_model(s));source['primitives'][0]['positions'][0] += .001
            with self.assertRaises(ValueError):author.export_seahorse(s,source)
            with self.assertRaises(ValueError):author.decode_seahorse(b,next(n for n in author.SPECIES if n!=s))


if __name__ == '__main__':unittest.main()
