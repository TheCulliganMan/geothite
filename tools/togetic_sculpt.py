#!/usr/bin/env python3
"""Original graceful Togetic with a slender neck and layered swept angel wings."""
import argparse,json,math
from functools import lru_cache
from copy import deepcopy
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube,reverse_front
from wing_insect_sculpt import contour
from togepi_sculpt import ring
from skin_glb import COORDINATES
HEIGHT=.94
WHITE=(.96,.98,.94);RED=(.89,.36,.32);BLUE=(.28,.66,.82);INK=(.12,.10,.13);PINK=(.90,.57,.62);CREASE=(.78,.83,.77)

def head():
    n=72;rows=49;v=[];directions=[(math.sin(a),math.cos(a),amplitude) for a,amplitude in ((-.82,.080),(0,.095),(.82,.080))]
    for k in range(rows):
        phi=.008+(math.pi-.016)*k/(rows-1);uy=math.cos(phi)
        for j in range(n):
            t=j*math.tau/n;ux=math.sin(phi)*math.sin(t);uz=math.sin(phi)*math.cos(t);x=.145*ux;y=1.062+.162*uy;z=.030+.130*uz
            for dx,dy,amplitude in directions:
                bulge=amplitude*math.exp((ux*dx+uy*dy-1)/.035);x+=dx*bulge;y+=dy*bulge
            v.append((x,y,z))
    f=[tuple(reversed(range(n))),tuple((rows-1)*n+j for j in range(n))]
    for k in range(rows-1):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part('Head / rounded ivory face with three continuous crown points',WHITE,v,f,True),40.)

def feather(sign,j):
    if j==0:path=[(sign*.108,.482,-.072),(sign*.243,.657,-.130),(sign*.380,.888,-.174),(sign*.466,1.030,-.202)];r=[.050,.065,.045,.001]
    elif j==1:path=[(sign*.115,.477,-.079),(sign*.285,.602,-.145),(sign*.418,.771,-.190),(sign*.453,.842,-.215)];r=[.050,.054,.027,.001]
    else:path=[(sign*.105,.466,-.073),(sign*.269,.541,-.146),(sign*.354,.647,-.181)];r=[.045,.050,.001]
    p=curved_tube('Wing feather / swept flattened ivory plume '+str(sign)+' '+str(j),WHITE,path,r,28)
    p['positions']=[v*.35-.060 if i%3==2 else v for i,v in enumerate(p['positions'])]
    return crease_normals(p,35.)

@lru_cache(maxsize=1)
def raw_sculpture():
    body=loft('Body / continuous pear torso and long slender ivory neck',WHITE,curved_profiles(((.070,.032,.035,-.020),(.170,.176,.139,-.032),(.330,.231,.177,-.045),(.480,.184,.151,-.022),(.580,.100,.095,.013),(.650,.055,.053,.026),(.750,.044,.045,.032),(.863,.044,.044,.034),(.962,.050,.047,.030)),4),40)
    face=head();p=[body,face];bs=surface(body);hs=surface(face)
    designs=[('upper red front triangle',RED,[(-.080,.505),(.039,.454),(-.059,.376)],False),('lower blue front lozenge',BLUE,[(-.196,.343),(-.119,.316),(-.127,.240),(-.207,.272)],False),('right red front triangle',RED,[(.132,.263),(.219,.296),(.199,.371)],False),('upper red rear triangle',RED,[(-.060,.512),(.087,.430),(-.065,.364)],True),('lower blue rear lozenge',BLUE,[(-.179,.341),(-.103,.311),(-.111,.236),(-.192,.273)],True),('lower red rear triangle',RED,[(.045,.222),(.139,.293),(.141,.216)],True)]
    for name,color,xy,rear in designs:
        parent=reverse_front(body) if rear else body;patch=ring('Body ring / '+name,color,xy,surface(parent));p.append(reverse_front(patch) if rear else patch)
    drop=contour([((.060,.333),(.073,.324),(.093,.290),(.078,.279)),((.078,.279),(.046,.266),(.045,.307),(.060,.333))],10)
    p.append(fitted_contour('Body fleck / fitted small blue teardrop',BLUE,drop,bs,.006))
    for sign in (-1,1):
        for j in range(3):
            wing=feather(sign,j);p.append(wing)
            if j<2:
                xy=[(.156,.548),(.243,.657),(.335,.807),(.416,.940)] if j==0 else [(.179,.522),(.279,.601),(.365,.717),(.412,.785)]
                skin=surface(wing);pts=[(sign*x,y,skin(y,sign*x)+.001) for x,y in xy]
                p.append(tube('Feather crease / fitted pale wing seam '+str(sign)+' '+str(j),CREASE,pts,[.0014]*len(pts),10))
        x=sign*.046;eye=fitted_patch('Eye / small dark vertical happiness oval '+str(sign),INK,oval(x,1.085,.0105,.024,32),hs,.005);p.append(eye)
        p.append(fitted_patch('Eye glint / tiny ivory happiness sparkle '+str(sign),WHITE,oval(x-.001,1.096,.0035,.0065,20),surface(eye),.002))
        brow=contour([((x-.016,1.123),(x-.012,1.140),(x+.010,1.141),(x+.018,1.132)),((x+.018,1.132),(x+.006,1.137),(x-.008,1.136),(x-.016,1.123))],9)
        p.append(fitted_contour('Brow / fitted soft smiling arch '+str(sign),CREASE,brow,hs,.002))
        path=[(-.093,.539,.054),(-.176,.613,.090),(-.245,.632,.102)] if sign<0 else [(.083,.548,.058),(.129,.699,.091),(.145,.773,.100)]
        p.append(curved_tube('Arm / raised tapered ivory leaf hand '+str(sign),WHITE,path,[.036,.041,.001],24))
        p.append(curved_tube('Leg / short supporting ivory root '+str(sign),WHITE,[(sign*.111,.185,-.012),(sign*.132,.067,.052)],[.033,.031],22))
        foot=ellipsoid('Foot / rounded grounded ivory leaf pad '+str(sign),WHITE,(sign*.145,.039,.101),(.061,.039,.085),32,20);p.append(foot)
        p.append(curved_tube('Foot point / long tapered ivory leaf toe '+str(sign),WHITE,[(sign*.151,.042,.146),(sign*.168,.034,.225)],[.028,.001],20))
        crease=[(sign*.148-.002,.061),(sign*.148+.002,.061),(sign*.162+.002,.035),(sign*.162-.002,.035)]
        p.append(fitted_contour('Toe crease / fitted pale foot division '+str(sign),CREASE,crease,surface(foot),.0015))
    smile=contour([((-.044,1.022),(-.014,1.023),(.017,1.028),(.046,1.030)),((.046,1.030),(.029,.953),(-.029,.951),(-.044,1.022))],12)
    mouth=fitted_contour('Mouth / fitted delighted open fairy smile',INK,smile,hs,.006);p.append(mouth)
    cx=sum(x for x,y in smile)/len(smile);cy=sum(y for x,y in smile)/len(smile);inside=[(cx+(x-cx)*.72,cy+(y-cy)*.72) for x,y in smile]
    p.append(fitted_contour('Tongue / fitted rosy smile interior',PINK,inside,surface(mouth),.003))
    p=[crease_normals(q,35.) if q['part'].startswith(('Eye /','Eye glint /','Brow /','Body fleck /','Toe crease /','Mouth /','Tongue /')) else q for q in p]
    return {'name':'togetic','version':1,'coordinate_system':COORDINATES,'primitives':p}
def sculpture(normalize=True):
    model=deepcopy(raw_sculpture())
    if not normalize:return model
    p=model['primitives']
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'togetic','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    from egg_fairy_glb import export_egg_fairy
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'togetic.glb').write_bytes(export_egg_fairy('togetic'))
if __name__=='__main__':main()
