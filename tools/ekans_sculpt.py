#!/usr/bin/env python3
"""Original grounded Ekans coil, broad collar and fitted slit-eyed snake head."""
import argparse, math
from copy import deepcopy
from functools import lru_cache
from pathlib import Path
from sculpt_geometry import loft, curved_profiles, ellipsoid, tube, crease_normals, part
from bovine_sculpt import smooth, oval
from dragon_family_sculpt import painted_tube, fitted
from abra_sculpt import surface
from skin_glb import COORDINATES

SPECIES=('ekans',)
HEIGHT=1.18
PURPLE=(.56,.29,.65)
GOLD=(.93,.78,.35)
INK=(.075,.055,.09)
WHITE=(.98,.97,.91)
PATH=((0.,.935,.025),(-.025,.70,-.015),(.015,.43,.025),
      (.32,.20,.055),(.20,.12,.35),(-.24,.12,.35),(-.49,.14,-.05),
      (-.20,.16,-.36),(.28,.18,-.30),(.55,.34,-.16),
      (.58,.55,-.19),(.57,.68,-.22))
RADII=(.103,.104,.080,.080,.095,.110,.100,.095,.083,.060,.039,.003)
STEPS=6

@lru_cache(maxsize=1)
def spine_samples():
    return smooth(PATH,RADII,STEPS)

def collar():
    # Closed complete neck band copied from the actual body rings. Shared
    # ring positions and diagonals avoid a floating necklace or flat decal.
    from chikorita_sculpt import add,sub,mul,dot,cross,unit
    centers,radii=spine_samples();n=32;v=[];side=None;frames=[]
    for k,p in enumerate(centers):
        tangent=unit(sub(centers[min(k+1,len(centers)-1)],centers[max(0,k-1)]))
        side=unit(sub((0,0,1) if side is None else side,mul(tangent,dot((0,0,1) if side is None else side,tangent))))
        frames.append((side,cross(tangent,side)))
    rows=list(range(2,6))
    for offset in (.0025,-.001):
        for k in rows:
            a,b=frames[k];r=radii[k]+offset
            for j in range(n):
                t=j*math.tau/n;v.append(add(centers[k],mul(add(mul(a,math.cos(t)),mul(b,math.sin(t))),r)))
    count=len(rows)*n;f=[]
    for j in range(len(rows)-1):
        for k in range(n):
            a=j*n+k;b=j*n+(k+1)%n
            f.extend(((a,b,b+n,a+n),(a+count+n,b+count+n,b+count,a+count)))
    for k in range(n):
        a=k;b=(k+1)%n;c=(len(rows)-1)*n
        f.extend(((b,a,a+count,b+count),(a+c,b+c,b+c+count,a+c+count)))
    return crease_normals(part('Collar / continuous broad golden neck band',GOLD,v,f,True),40.)

@lru_cache(maxsize=1)
def anatomy():
    p=painted_tube('Body / continuous grounded purple snake coil',PURPLE,PATH,RADII,
                   segments=32,steps=STEPS)
    # Ekans has a golden ventral strip rather than the blue dragons' ivory.
    from chikorita_sculpt import linear
    p[1]['base_color']=linear(GOLD)
    p[1]['part']='Belly / continuous fitted golden ventral scales'
    p.append(collar())
    head=loft('Head / broad tapered purple snake brow and muzzle',PURPLE,
        curved_profiles(((.913,.060,.070,.065),(.94,.144,.137,.092),
                         (.995,.194,.188,.086),(1.065,.180,.156,.057),
                         (1.105,.122,.103,.038),(1.125,.025,.027,.025)),5),40)
    p.append(head)
    for side in (-1,1):
        # The outer upper corner rises, giving the characteristic alert gaze.
        x=side*.107;y=1.055
        eye=fitted('Eye / fitted golden snake gaze '+str(side),GOLD,
                   [(x-side*.038,y-.037),(x+side*.035,y-.026),
                    (x+side*.040,y+.036),(x-side*.027,y+.024)],head,.004)
        p.append(eye)
        pupil=fitted('Pupil / fitted vertical dark snake slit '+str(side),INK,
                     oval(x,y,.008,.027,24),eye,.002)
        p.append(pupil)
        p.append(fitted('Glint / fitted tiny ivory snake reflection '+str(side),WHITE,
                        oval(x-.003,y+.012,.003,.005,16),pupil,.0015))
        p.append(fitted('Nostril / fitted dark snake nostril '+str(side),INK,
                        oval(side*.065,.987,.008,.005,20),head,.002))
    hs=surface(head);smile=[]
    for i in range(41):
        x=-.135+.27*i/40;y=.957+.019*(abs(x)/.135)**2
        smile.append((x,y,hs(y,x)+.0016))
    p.append(tube('Mouth / fitted curved closed snake mouth',INK,smile,[.0022]*41,8))
    # Three overlapping tapered rattles follow the raised tail's true axis.
    for j,(c,r) in enumerate((((.573,.545,-.189),(.049,.046,.047)),
                              ((.578,.602,-.202),(.039,.039,.038)),
                              ((.571,.650,-.214),(.027,.033,.028)))):
        p.append(ellipsoid('Rattle / rooted golden tail segment '+str(j),GOLD,c,r,24,16))
    return tuple(p)

@lru_cache(maxsize=1)
def authoring_transform():
    ps=anatomy();lo=min(v for p in ps for v in p['positions'][1::3]);hi=max(v for p in ps for v in p['positions'][1::3])
    return lo,HEIGHT/(hi-lo)

def sculpture(species='ekans',normalize=True):
    if species not in SPECIES:raise ValueError('unsupported coiled snake')
    ps=deepcopy(anatomy());lo,k=authoring_transform()
    for p in ps:
        if normalize:p['positions']=[round((v-(lo if i%3==1 else 0.))*k,7) or 0. for i,v in enumerate(p['positions'])]
        p['normals']=[round(v,7) or 0. for v in p['normals']]
    return dict(name=species,version=1,coordinate_system=COORDINATES,primitives=list(ps))

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
    from ekans_glb import export_ekans
    args.output.mkdir(parents=True,exist_ok=True);(args.output/'ekans.glb').write_bytes(export_ekans())
if __name__=='__main__':main()
