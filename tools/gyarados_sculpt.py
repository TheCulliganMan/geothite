#!/usr/bin/env python3
"""Original coiled Gyarados: open fanged mouth, trident crest and rooted fins.

Standard-library authoring, +Y up and +Z forward. No imported geometry.
"""
import argparse
import json
import math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,tube,fitted_patch,crease_normals
from bovine_sculpt import curved_tube,oval,smooth
from abra_sculpt import surface
from chikorita_sculpt import add,sub,mul,dot,cross,unit
from skin_glb import COORDINATES

HEIGHT=1.40
BLUE=(.19,.58,.76);CREAM=(.95,.86,.62);WHITE=(.96,.97,.92)
INK=(.10,.12,.17);MOUTH=(.30,.12,.15);TONGUE=(.78,.43,.49);RED=(.83,.27,.23)
PATH=((.79,.49,-.54),(.61,.37,-.69),(.30,.26,-.62),(-.06,.22,-.44),
      (-.34,.21,-.12),(-.30,.30,.17),(-.10,.49,.24),(0.,.75,.24),
      (0.,1.02,.24),(0.,1.26,.26))
RADII=(.045,.075,.105,.135,.165,.180,.190,.185,.17,.135)


def fin(name,points,thickness=.014):
    # Folded closed fan, with a central ridge instead of a single flat sheet.
    center=tuple(sum(p[k] for p in points)/len(points) for k in range(3))
    v=[(x,y,z-thickness/2) for x,y,z in points]+[(center[0],center[1],center[2]+thickness)]
    v.append((center[0],center[1],center[2]-thickness));n=len(points)
    return part(name,WHITE,v,[(i,(i+1)%n,n) for i in range(n)]+[((i+1)%n,i,n+1) for i in range(n)])


def body_path():
    points,radii=smooth(PATH,RADII)
    # The swollen scale segments taper back into each seam; this changes the
    # blue silhouette rather than painting a generic smooth serpent.
    radii=[r*(1.+.08*math.sin(math.pi*(i%4)/4)**2) for i,r in enumerate(radii)]
    return points,radii


def belly_band(index):
    points,radii=body_path()
    frames=[];side=None
    for k,center in enumerate(points):
        tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]))
        if side is None:side=cross(tangent,(0,1,0) if abs(tangent[1])<.9 else (1,0,0))
        side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
        frames.append((side,other,tangent))
    width=12;ring=width+1;v=[]
    for inset in (.005,-.012):
        for k in range(index*4,index*4+5):
            center=points[k];side,other,tangent=frames[k]
            target=(0.,-.20,1.)
            angle=math.atan2(dot(target,other),dot(target,side))
            for j in range(width+1):
                theta=angle-1.04+2.08*j/width
                direction=add(mul(side,math.cos(theta)),mul(other,math.sin(theta)))
                v.append(add(center,mul(direction,radii[k]+inset)))
    layer=ring*5;f=[]
    for j in range(4):
        for k in range(width):
            n=j*ring+k;f.append((n,n+1,n+ring+1,n+ring))
    f += [tuple(layer+i for i in reversed(face)) for face in list(f)]
    boundary=list(range(ring))+[j*ring+width for j in range(1,5)]+list(range(4*ring+width-1,4*ring-1,-1))+[j*ring for j in (3,2,1)]
    f += [(b,a,a+layer,b+layer) for a,b in zip(boundary,boundary[1:]+boundary[:1])]
    return crease_normals(part('Belly / inset armored band '+str(index),CREAM,v,f),40.)


def sculpture():
    points,radii=body_path()
    p=[crease_normals(tube('Body / continuous coiled sea serpent',BLUE,points,radii,24),35.)]
    p += [belly_band(i) for i in range(len(PATH)-1)]
    head=ellipsoid('Head / broad angular crowned skull',BLUE,(0.,1.30,.34),(.225,.145,.205),32,18);p.append(head);hs=surface(head)
    p.append(ellipsoid('Mouth / recessed open throat',MOUTH,(0.,1.047,.425),(.172,.190,.035),28,20))
    jaw=ellipsoid('Jaw / forward lower mandible',BLUE,(0.,.853,.445),(.171,.064,.154),28,14);p.append(jaw)
    p.append(ellipsoid('Tongue / lower mouth floor',TONGUE,(0.,.928,.466),(.111,.036,.035),24,12))
    rim=[(-.173,1.197,.490),(-.172,1.07,.512),(-.160,.89,.533),(0.,.836,.557),(.160,.89,.533),(.172,1.07,.512),(.173,1.197,.490)]
    p.append(curved_tube('Lip / continuous cream open-mouth rim',CREAM,rim,[.020]*len(rim),16))
    p.append(curved_tube('Lip / cream upper muzzle',CREAM,[(-.173,1.197,.490),(0.,1.210,.546),(.173,1.197,.490)],[.021,.023,.021],16))
    for s in (-1,1):
        p.append(tube('Fang / upper ivory canine '+str(s),WHITE,[(s*.118,1.205,.516),(s*.122,1.11,.527)],[.022,.0018],12))
        p.append(tube('Fang / lower ivory canine '+str(s),WHITE,[(s*.100,.87,.557),(s*.105,.936,.541)],[.017,.0018],12))
        outline=[(s*.049,1.299),(s*.151,1.341),(s*.177,1.301),(s*.064,1.277)]
        eye=fitted_patch('Eye / slanted ivory glare '+str(s),WHITE,outline,hs,.006);p.append(eye)
        iris=fitted_patch('Eye / narrow red iris '+str(s),RED,oval(s*.108,1.310,.014,.010,16),surface(eye),.003);p.append(iris)
        p.append(fitted_patch('Eye / vertical ink pupil '+str(s),INK,oval(s*.108,1.310,.004,.008,12),surface(iris),.002))
        p.append(curved_tube('Crest / outward rooted trident horn '+str(s),BLUE,[(s*.045,1.37,.40),(s*.159,1.48,.445),(s*.241,1.577,.482)],[.047,.026,.0018],12))
        p.append(fin('Cheek fin / rooted three-point fan '+str(s),[(s*.184,1.30,.30),(s*.390,1.396,.293),(s*.311,1.263,.30),(s*.423,1.262,.30),(s*.330,1.157,.30),(s*.397,1.093,.30),(s*.192,1.105,.30)]))
        p.append(curved_tube('Barbel / curling blue whisker '+str(s),BLUE,[(s*.173,1.063,.514),(s*.236,.90,.520),(s*.260,.73,.529),(s*.235,.680,.551),(s*.196,.718,.570)],[.012,.011,.009,.006,.0018],12))
    p.append(curved_tube('Crest / central rooted trident spear',BLUE,[(0.,1.395,.360),(0.,1.560,.409),(0.,1.690,.435)],[.048,.025,.0018],12))
    for j,(x,y,z,w) in enumerate(((0.,1.26,.18,.10),(-.29,.32,-.03,.095),(.34,.30,-.60,.07))):
        p.append(fin('Dorsal fin / white serrated sail '+str(j),[(x-w,y,z),(x-w*.75,y+.19,z-.01),(x-w*.17,y+.12,z),(x+w*.08,y+.31,z-.02),(x+w*.38,y+.16,z),(x+w,y+.21,z),(x+w*.76,y,z)]))
    for s in (-1,1):
        p.append(fin('Tail fin / forked white terminal sail '+str(s),[(.76,.49,-.55),(.80+s*.18,.72,-.55),(.81+s*.12,.54,-.55),(.80+s*.22,.35,-.55),(.78,.42,-.55)],.020))
        p.append(curved_tube('Tail fin / blue leading border '+str(s),BLUE,[(.76,.49,-.55),(.80+s*.18,.72,-.55)],[.021,.002],10))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        # Reviewed belly centerline coordinates use positive zero. Tiny frame
        # cancellation changes its sign across Python's sum implementations.
        if q['part'].startswith('Belly /'):
            q['positions']=[0. if v==0. else v for v in q['positions']]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'gyarados','version':1,'coordinate_system':COORDINATES,'primitives':p}


def reference_length(model=None):
    """Tail-to-mouth centerline in canonical units; excludes crest and sails."""
    points,radii=body_path();raw=tube('body reference',BLUE,points,radii,24)
    stored=(sculpture() if model is None else model)['primitives'][0]
    raw_width=max(raw['positions'][::3])-min(raw['positions'][::3])
    width=max(stored['positions'][::3])-min(stored['positions'][::3])
    length=sum(math.dist(a,b) for a,b in zip(points,points[1:]))
    length+=math.dist(points[-1],(0.,1.047,.546))
    return length*width/raw_width


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();model=sculpture();args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(model,separators=(',',':')))
    print(len(model['primitives']),sum(len(p['positions'])//3 for p in model['primitives']),sum(len(p['indices'])//3 for p in model['primitives']),f'centerline {reference_length(model):.7f}')


if __name__=='__main__':main()
