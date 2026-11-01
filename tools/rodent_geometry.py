"""Closed material sections and stored-corner shading for original rodent recipes."""
import math,struct
from copy import deepcopy
from sculpt_geometry import part,crease_normals
from skin_glb import COORDINATES


def ring_sections(vertices,segments,rows,sections,*,reverse=False):
    def faces(width,count,sector):
        caps=[tuple(reversed(range(width))),tuple((count-1)*width+i for i in range(width))]
        if sector:
            caps += [(k*width+width-1,k*width,(k+1)*width,(k+1)*width+width-1) for k in range(count-1)]
        outside=[]
        for k in range(count-1):
            for i in range(width-1 if sector else width):
                a=k*width+i;b=k*width+(i+1)%width;outside.append((a,b,b+width,a+width))
        fs=caps+outside
        if reverse:fs=[tuple(reversed(f)) for f in fs]
        return fs,sum(len(f)-2 for f in caps)*3
    fs,_=faces(segments,rows,False)
    whole=part('Body / reference surface',(1.,1.,1.),vertices,fs,True)
    normals={tuple(whole['positions'][i:i+3]):whole['normals'][i:i+3] for i in range(0,len(whole['positions']),3)}
    parts=[]
    for name,color,start,end,angles in sections:
        angles=list(angles);width=len(angles);points=[vertices[k*segments+i] for k in range(start,end+1) for i in angles]
        fs,caps=faces(width,end-start+1,width!=segments)
        p=part(name,color,points,fs)
        for i in set(p['indices'][caps:]):p['normals'][i*3:i*3+3]=normals[tuple(p['positions'][i*3:i*3+3])]
        parts.append(p)
    return parts,whole


def stored_model(name,parts,height,*,normalize=True):
    parts=deepcopy(parts)
    if normalize:
        lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=height/(hi-lo)
        for p in parts:
            values=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])]
            p['positions']=list(struct.unpack('<'+'f'*len(values),struct.pack('<'+'f'*len(values),*values)))
            if not p['part'].startswith(('Body /','Tail /')):p.update(crease_normals(p,40.))
            p['normals']=[round(v,7) or 0. for v in p['normals']]
    return dict(name=name,version=1,coordinate_system=COORDINATES,primitives=parts)
