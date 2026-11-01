#!/usr/bin/env python3
"""Original flowing lava bodies and a closed coiled stone snail shell."""
import argparse,json,math
from functools import lru_cache
from copy import deepcopy
from pathlib import Path
from sculpt_geometry import part,ellipsoid,tube,curved_profiles
from bovine_sculpt import fitted_patch
from abra_sculpt import surface
from pitcher_sculpt import funnel
from skin_glb import COORDINATES

SPECIES=('slugma','magcargo');HEIGHT=.98
RED=(.84,.20,.13);GLOW=(1.,.59,.16);YELLOW=(.98,.86,.32)
INK=(.085,.065,.063);STONE=(.40,.37,.38);ASH=(.58,.52,.50)

def flowing_body(shelled):
    # Longitudinal cross sections join tail, mantle, neck and face in one skin.
    rows=curved_profiles(((-.58,.009,.012,.025),(-.49,.17,.105,.020),
                         (-.31,.275,.17,.012),(-.10,.265,.245,.008),
                         (.07,.205,.51 if shelled else .57,.008),
                         (.18,.158,.54 if shelled else .60,.008),
                         (.28,.09,.40,.012),(.33,.009,.17,.065)),4)
    v=[];n=32
    for z,w,h,base in rows:
        for i in range(n):
            a=math.tau*i/n
            # The low mantle has scalloped edges, while the neck stays smooth.
            wave=1+.065*math.cos(z*26)*max(0.,1-h/.60)
            v.append((w*math.sin(a)*wave,base+h*max(0.,(math.cos(a)+.85)/1.85),z))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    for k in range(len(rows)-1):
        for i in range(n):
            a=k*n+i;b=k*n+(i+1)%n;f.append((a,b,b+n,a+n))
    return part('Body / continuous flowing mantle neck and tapered tail',RED,v,f,True)

def shell_parts():
    shell=ellipsoid('Shell / broad flattened stone coil',STONE,(0,.445,-.25),(.255,.315,.326),32,20)
    # Intersect the actual stored shell along X, rather than approximating
    # depth with a sphere formula. Both sides carry the complete coiling ridge.
    rotated=dict(shell);rotated['positions']=[value for i in range(0,len(shell['positions']),3)
        for value in (shell['positions'][i+2],shell['positions'][i+1],shell['positions'][i])]
    skin=surface(rotated);p=[shell]
    for side in (-1,1):
        points=[];radii=[]
        for i in range(90):
            t=i/89;a=math.tau*(1.62*t)+.5;r=.275*(1-t)+.027*t
            y=.445+r*math.sin(a);z=-.25+r*math.cos(a)
            points.append((side*(skin(y,z)+.006),y,z));radii.append(.014-.005*t)
        p.append(tube('Shell ridge / complete fitted spiral '+str(side),ASH,points,radii,10))
        # Thin warm seams emerge between the stone layers near the mantle.
        for j,a in enumerate((3.7,4.45,5.15)):
            points=[]
            for k in range(7):
                r=.22+.037*k/6;aa=a+.13*math.sin(k*.7)
                y=.445+r*math.sin(aa);z=-.25+r*math.cos(aa)
                points.append((side*(skin(y,z)+.004),y,z))
            p.append(tube('Shell seam / exposed molten fissure '+str(side)+' '+str(j),GLOW,points,[.004]*7,8))
    chimney=funnel('Chimney / hollow stone vent',STONE,((0,.104,.090),(.095,.080,.071),(.245,.075,.063)),(.11,.655,-.285),up=True,thickness=.024,segments=24)
    p.append(chimney)
    p.append(funnel('Chimney / dark inner lining',INK,((.11,.055,.050),(.238,.047,.035)),(.11,.655,-.285),up=True,thickness=.004,segments=20))
    p.append(ellipsoid('Chimney / recessed molten floor',GLOW,(.11,.805,-.285),(.048,.006,.034),20,8))
    return p

@lru_cache(maxsize=2)
def raw_sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported lava species')
    shelled=species=='magcargo';body=flowing_body(shelled);p=[body];skin=surface(body)
    for side in (-1,1):
        # Rounded asymmetric stalks grow directly from the raised neck.
        ys=(.46,.57,.66,.75) if shelled else (.49,.61,.72,.82)
        points=[(side*x,y,z) for x,y,z in zip((.095,.116,.13,.13),ys,(.135,.137,.130,.130))]
        top=points[-1]
        points.append((top[0],top[1]+.038,top[2]))
        stalk=tube('Eye stalk / rising rounded lava horn '+str(side),RED,points,[.058,.050,.045,.040,.004],20)
        p.append(stalk)
        p.append(ellipsoid('Eye / broad yellow oval '+str(side),YELLOW,(top[0],top[1]-.006,top[2]+.037),(.035,.043,.012),20,12))
        p.append(ellipsoid('Eye / slim black pupil '+str(side),INK,(top[0]+side*.004,top[1]-.003,top[2]+.049),(.011,.027,.003),12,8))
        # Glowing elongated droplets follow the front skin.
        outline=[(side*.123,.22),(side*.149,.246),(side*.160,.218),(side*.144,.196)]
        p.append(fitted_patch('Lava / fitted molten cheek droplet '+str(side),GLOW,outline,skin,.004))
    smile=[(-.065,.370),(-.034,.353),(0,.349),(.034,.353),(.065,.370)]
    p.append(tube('Mouth / shallow curved smile',INK,[(x,y,skin(y,x)+.003) for x,y in smile],[.005]*5,10))
    if shelled:p.extend(shell_parts())
    else:
        # Slugma's flexible molten crest replaces the hard stone shell.
        rotated=dict(body);rotated['positions']=[value for i in range(0,len(body['positions']),3)
            for value in (body['positions'][i],body['positions'][i+2],body['positions'][i+1])]
        top=surface(rotated)
        for side in (-1,1):
            outline=[(side*.10,-.44),(side*.15,-.32),(side*.13,-.16),(side*.10,-.04)]
            p.append(tube('Mantle / fitted sinuous lava fold '+str(side),RED,
                [(x,top(z,x)+.003,z) for x,z in outline],[.012,.015,.014,.010],14))
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def sculpture(species,normalize=True):
    model=deepcopy(raw_sculpture(species))
    if not normalize:return model
    p=model['primitives']
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    from lava_glb import export_lava
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_lava(s))
if __name__=='__main__':main()
