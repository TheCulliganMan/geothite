#!/usr/bin/env python3
"""Original sleepy Gloom bud and five-petalled hollow Vileplume crown."""
import argparse,math
from copy import deepcopy
from functools import lru_cache
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,crease_normals
from bovine_sculpt import oval,curved_tube,smooth
from pitcher_sculpt import bell_fitted as fitted,funnel
from skin_glb import COORDINATES

SPECIES=('gloom','vileplume')
HEIGHTS={'gloom':.85,'vileplume':1.12}
BLUE=(.22,.28,.43);INK=(.035,.035,.055);ORANGE=(.90,.37,.12)
BUD=(.76,.29,.10);RED=(.91,.18,.18);CREAM=(.99,.94,.79)
EYE=(.85,.20,.16)

def turn(p,angle):
    p=deepcopy(p);c,s=math.cos(angle),math.sin(angle)
    for key in ('positions','normals'):
        v=p[key];p[key]=[w for x,y,z in zip(v[::3],v[1::3],v[2::3]) for w in (c*x+s*z,y,-s*x+c*z)]
    return p

def top(p,inverse=False):
    p=deepcopy(p)
    for key in ('positions','normals'):
        v=p[key];p[key]=[w for x,y,z in zip(v[::3],v[1::3],v[2::3]) for w in ((x,z,-y) if inverse else (x,-z,y))]
    return p

def flower_point(point):
    x,y,z=point;c=math.cos(.27);s=math.sin(.27)
    return x,.61+(y-.69)*c-z*s,(y-.69)*s+z*c

def tilt_flower(p):
    p=deepcopy(p);v=p['positions'];p['positions']=[w for point in zip(v[::3],v[1::3],v[2::3]) for w in flower_point(point)]
    v=p['normals'];c=math.cos(.27);s=math.sin(.27);p['normals']=[w for x,y,z in zip(v[::3],v[1::3],v[2::3]) for w in (x,y*c-z*s,y*s+z*c)]
    return p

def petal(name,color,angle,drooping):
    """Closed round-section radial blade; convex section caps never fan across a cup."""
    n=16;rows=19;v=[];faces=[];d=(math.sin(angle),math.cos(angle));e=(math.cos(angle),-math.sin(angle))
    for j in range(rows):
        t=j/(rows-1)
        r=(.06+.34*t) if drooping else (.105+.43*t)
        y=(.555+.040*math.sin(math.pi*t)-.16*t) if drooping else (.685+.045*math.sin(math.pi*t)-.025*t)
        w=(.006+.117*math.sin(math.pi*t)**.8) if drooping else (.028+.177*math.sin(math.pi*(.015+.97*t))**.72)
        h=(.008+.010*math.sin(math.pi*t)) if drooping else (.030+.018*math.sin(math.pi*t))
        if not drooping and t>.85:
            end=math.sqrt(max(0.,1.-((t-.85)/.15)**2))
            w=.002+(w-.002)*end;h=.004+(h-.004)*end
        for k in range(n):
            a=k*math.tau/n;u=w*math.cos(a)
            v.append((r*d[0]+u*e[0],y+h*math.sin(a),r*d[1]+u*e[1]))
    faces=[tuple(reversed(range(n))),tuple((rows-1)*n+k for k in range(n))]
    for j in range(rows-1):
        for k in range(n):
            a=j*n+k;b=j*n+(k+1)%n;faces.append((a,b,b+n,a+n))
    return crease_normals(part(name,color,v,faces,True),45.)

def bud():
    profiles=((.50,.085),(.54,.195),(.63,.255),(.72,.23),(.78,.15),(.81,.025))
    centers,radii=smooth([(0.,y,0.) for y,r in profiles],[r for y,r in profiles],4)
    n=64;v=[]
    for center,r in zip(centers,radii):
        y=center[1]
        for k in range(n):
            a=k*math.tau/n;radius=r*(1.+.22*math.cos(4*a))
            v.append((radius*math.sin(a),y,radius*.92*math.cos(a)))
    faces=[tuple(reversed(range(n))),tuple((len(centers)-1)*n+k for k in range(n))]
    for j in range(len(centers)-1):
        for k in range(n):
            a=j*n+k;b=j*n+(k+1)%n;faces.append((a,b,b+n,a+n))
    return crease_normals(part('Bud / four-lobed orange-brown crown',BUD,v,faces,True),50.)

def arm_path(s,side):
    return ((side*.17,.32,0.),(side*.24,.28 if s=='gloom' else .33,.015),(side*.31,.245 if s=='gloom' else .31,.035))

def leg_path(side):return ((side*.10,.18,0.),(side*.12,.10,.025),(side*.13,.052,.055))

def body_parts(s):
    body=ellipsoid('Body / round indigo plant bulb',BLUE,(0.,.285,0.),(.225,.23,.185),28,18);p=[body]
    for side in (-1,1):
        p.append(curved_tube('Leg / short rooted blue stem '+str(side),BLUE,leg_path(side),[.054,.049,.038],18))
        p.append(ellipsoid('Foot / planted rounded blue root '+str(side),BLUE,(side*.13,.042,.068),(.085,.042,.105),24,14))
        p.append(curved_tube('Arm / blunt blue plant hand '+str(side),BLUE,arm_path(s,side),[.052,.049,.035],18))
        x=side*.080
        if s=='gloom':
            outline=[(x-.041,.354),(x-.019,.347),(x+.020,.347),(x+.041,.354),(x+.037,.342),(x+.014,.336),(x-.020,.337),(x-.039,.343)]
            p.append(fitted('Eye / fitted sleepy closed lid '+str(side),INK,outline,body,.003))
        else:
            eye=fitted('Eye / fitted red bead '+str(side),EYE,oval(x,.347,.014,.022,24),body,.003);p.append(eye)
    if s=='gloom':
        lip=fitted('Lip / broad orange sleepy lower lip',ORANGE,oval(0.,.269,.080,.032,32),body,.004);p.append(lip)
        p.append(fitted('Mouth / recessed narrow sleepy opening',INK,oval(0.,.279,.060,.010,28),lip,.002))
        # The cream strands start inside the occupied lower lip and stay in
        # front of the bulb instead of disappearing into its curved surface.
        p.append(curved_tube('Drool / connected long cream strand',CREAM,[(.023,.253,.191),(.028,.222,.201),(.021,.186,.199),(.018,.165,.195)],[.016,.013,.011,.018],16))
        p.append(curved_tube('Drool / connected short cream strand',CREAM,[(-.027,.254,.190),(-.031,.232,.200),(-.028,.216,.200)],[.011,.009,.013],16))
    else:
        p.append(fitted('Mouth / tiny happy dark opening',INK,[(-.026,.278),(.026,.278),(.017,.253),(0.,.247),(-.017,.253)],body,.003))
    return p

def gloom():
    p=body_parts('gloom');crown=bud();p.append(crown)
    for j in range(4):
        angle=j*math.tau/4+math.pi/4
        p.append(petal('Sepal / four drooping orange lobes '+str(j),ORANGE,angle,True))
        local=turn(crown,-angle)
        for k,(x,y,rx,ry) in enumerate(((-.053,.696,.021,.017),(.031,.731,.025,.020),(.073,.651,.018,.025))):
            q=fitted(f'Bud spot / fitted pale mottling {j} {k}',CREAM,oval(x,y,rx,ry,20),local,.002)
            p.append(turn(q,angle))
    return p

def vileplume():
    p=body_parts('vileplume');crown_start=len(p)
    p.append(loft('Stem / stout blue flower support',BLUE,((.44,.075,.075,0.),(.60,.090,.090,0.),(.63,.080,.080,0.)),24))
    p.append(funnel('Crown / hollow flared red flower cup',RED,((.59,.072,.072),(.66,.158,.158),(.707,.150,.150)),(0.,0.,0.),True,thickness=.025,segments=40))
    p.append(funnel('Rim / pale annular flower throat',CREAM,((.672,.127,.127),(.713,.145,.145)),(0.,0.,0.),True,thickness=.021,segments=40))
    p.append(ellipsoid('Throat / dark recessed nectar well',INK,(0.,.662,0.),(.142,.007,.142),32,12))
    for j in range(5):
        angle=j*math.tau/5;blade=petal('Petal / five broad scarlet flower lobes '+str(j),RED,angle,False);p.append(blade)
        upper=top(blade)
        for k,(r,u,rx,rz) in enumerate(((.31,-.055,.031,.026),(.40,.040,.035,.030),(.465,-.025,.021,.020))):
            x=r*math.sin(angle)+u*math.cos(angle);z=r*math.cos(angle)-u*math.sin(angle)
            q=fitted(f'Petal spot / fitted ivory mottling {j} {k}',CREAM,oval(x,-z,rx,rz,20),upper,.0025)
            p.append(top(q,True))
    return p[:crown_start]+[tilt_flower(q) for q in p[crown_start:]]

@lru_cache(maxsize=2)
def anatomy(s):
    if s not in SPECIES:raise ValueError('unknown rafflesia species')
    return tuple((gloom if s=='gloom' else vileplume)())

def sculpture(s,normalize=True):
    p=deepcopy(list(anatomy(s)));lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[s]/(hi-lo)
    for q in p:
        if normalize:q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
        for key in ('positions','normals'):q[key]=[0. if v==0 else v for v in q[key]]
    return dict(name=s,version=1,coordinate_system=COORDINATES,primitives=p)

def main():
    from rafflesia_glb import export_rafflesia
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_rafflesia(s))

if __name__=='__main__':main()
