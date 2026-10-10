#!/usr/bin/env python3
"""Refine original authored Pidgeotto contours, retaining its production rig."""
import argparse,copy
from pathlib import Path
from sculpt_geometry import loft,translate,crease_normals,curved_profiles
from pidgeotto_glb import decode_pidgeotto,export_pidgeotto
TARGETS={'Bird body','Bird head','Light breast'}


def components(p):
    ps=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
    triangles=[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)]
    incident={}
    for i,t in enumerate(triangles):
        for j in t:incident.setdefault(ps[j],set()).add(i)
    pending=set(range(len(triangles)));out=[]
    while pending:
        seed=min(pending);seen=set();queue=[seed]
        while queue:
            i=queue.pop()
            if i in seen:continue
            seen.add(i)
            for j in triangles[i]:queue.extend(incident[ps[j]]-seen)
        pending-=seen;ids=[j for i in sorted(seen) for j in triangles[i]];used=sorted(set(ids));remap={j:i for i,j in enumerate(used)}
        q=dict(p)
        for key in ('positions','normals'):q[key]=[c for j in used for c in p[key][j*3:j*3+3]]
        q['indices']=[remap[j] for j in ids];out.append(q)
    return out


def refine(p):
    rows={}
    for i in range(0,len(p['positions']),3):
        x,y,z=p['positions'][i:i+3];rows.setdefault(round(y,6),[]).append((x,z))
    # Canonical output is already refined; do not repeatedly subdivide it.
    if len(rows)!=6:return p
    profiles=[];centers=[]
    for y,points in sorted(rows.items()):
        xs,zs=zip(*points);cx=(min(xs)+max(xs))/2;cz=(min(zs)+max(zs))/2
        centers.append(cx);profiles.append((y,(max(xs)-min(xs))/2,(max(zs)-min(zs))/2,cz))
    if max(centers)-min(centers)>1e-5:raise ValueError('unexpected non-axial authored bird contour')
    q=crease_normals(translate(loft(p['part'],(1,1,1),curved_profiles(profiles),40),sum(centers)/len(centers),0,0))
    q['base_color']=p['base_color'];return q


def prepare(model):
    """Used by the original batch authoring source before canonical GLB export."""
    model=copy.deepcopy(model)
    for index,p in enumerate(model['primitives']):
        if p['part'] not in TARGETS:continue
        pieces=components(p);largest=max(range(len(pieces)),key=lambda i:len(pieces[i]['indices']))
        pieces[largest]=refine(pieces[largest]);joined=dict(p);joined['positions']=[];joined['normals']=[];joined['indices']=[]
        for q in pieces:
            offset=len(joined['positions'])//3
            joined['positions'].extend(q['positions']);joined['normals'].extend(q['normals']);joined['indices'].extend(j+offset for j in q['indices'])
        model['primitives'][index]=joined
    return model


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('path',type=Path);a=a.parse_args()
    model=prepare(decode_pidgeotto(a.path.read_bytes()))
    a.path.write_bytes(export_pidgeotto(model))
    print('Pidgeotto: refined body/head/breast, preserved 34-part rig and wing/foot geometry')
if __name__=='__main__':main()
