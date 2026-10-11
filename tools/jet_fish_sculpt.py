#!/usr/bin/env python3
"""Original tapered Remoraid and eight-armed Octillery anatomy."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals
from abra_sculpt import surface,polygon_caps
from wing_insect_sculpt import contour
from pitcher_sculpt import funnel
from spider_sculpt import upper_skin,dorsal_patch
from skin_glb import COORDINATES
SPECIES=('remoraid','octillery');HEIGHT=1.00
GREEN=(.63,.79,.73);BAR=(.29,.51,.52);FIN=(.91,.95,.88)
RED=(.85,.28,.26);YELLOW=(.97,.82,.26);WELL=(.83,.63,.18)
WHITE=(.97,.96,.88);INK=(.10,.13,.16)

def smooth_path(points,radii,steps=5):
    out=[];rs=[]
    for i in range(len(points)-1):
        a=points[max(i-1,0)];b=points[i];c=points[i+1];d=points[min(i+2,len(points)-1)]
        for j in range(steps):
            t=j/steps
            out.append(tuple(.5*((2*b[k])+(-a[k]+c[k])*t+(2*a[k]-5*b[k]+4*c[k]-d[k])*t*t+(-a[k]+3*b[k]-3*c[k]+d[k])*t*t*t) for k in range(3)))
            rs.append(radii[i]*(1-t)+radii[i+1]*t)
    return out+[points[-1]],rs+[radii[-1]]

def side_rotate(p,side):
    p=dict(p)
    for field in ('positions','normals'):
        p[field]=[v for i in range(0,len(p[field]),3) for v in (side*p[field][i+2],p[field][i+1],-side*p[field][i])]
    return p

def fin(name,outline,axis='x',center=0.,thickness=.010):
    # Curved thin solid. Outline coordinates are rear/forward Z and height Y.
    v=[];n=len(outline)
    for sign in (1,-1):
        for z,y in outline:
            x=center+.018*math.sin((y-.3)*6)+sign*thickness
            v.append((z,y,x))
    caps=polygon_caps(v[:n]);ids=list(range(n))
    if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(outline,outline[1:]+outline[:1]))<0:ids.reverse()
    f=caps+[tuple(j+n for j in reversed(q)) for q in caps]+[(a,a+n,b+n,b) for a,b in zip(ids,ids[1:]+ids[:1])]
    p=crease_normals(part(name,FIN,v,f),35.)
    # Rotate XY drawing into YZ, keeping winding and normals consistent.
    p=side_rotate(p,-1)
    if axis=='z':
        p=side_rotate(p,1)
    return p

def remoraid():
    rows=curved_profiles(((-.38,.020,.030,.41),(-.26,.095,.10,.40),(-.12,.18,.175,.405),(.04,.205,.205,.40),(.20,.16,.16,.395),(.31,.10,.090,.385),(.34,.04,.035,.385)),5)
    v=[];n=40
    for z,w,h,y in rows:
        for i in range(n):a=i*math.tau/n;v.append((w*math.sin(a),y+h*math.cos(a),z))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    for j in range(len(rows)-1):
        for i in range(n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
    body=crease_normals(part('Body / streamlined mint fish',GREEN,v,f),40.);p=[body]
    upper=contour([((-.28,.44),(-.35,.54),(-.53,.82),(-.72,.88)),((-.72,.88),(-.60,.70),(-.51,.57),(-.50,.50)),((-.50,.50),(-.44,.50),(-.39,.45),(-.28,.44))],9)
    lower=contour([((-.30,.38),(-.38,.29),(-.56,.08),(-.72,.025)),((-.72,.025),(-.58,.22),(-.55,.40),(-.64,.47)),((-.64,.47),(-.48,.43),(-.39,.40),(-.30,.38))],9)
    p.extend((fin('Tail / swept upper lobe',upper),fin('Tail / broad scalloped lower lobe',lower)))
    dorsal=contour([((.24,.52),(.22,.56),(.18,.66),(.15,.71)),((.15,.71),(.11,.67),(.12,.57),(.08,.56)),((.08,.56),(.14,.57),(.20,.55),(.24,.52))],7)
    p.append(fin('Dorsal / forward pointed crest',dorsal,thickness=.014))
    pelvic=contour([((.03,.24),(-.01,.22),(-.07,.14),(-.18,.11)),((-.18,.11),(-.20,.16),(-.17,.21),(-.10,.24)),((-.10,.24),(-.07,.23),(-.02,.23),(.03,.24))],7)
    p.append(fin('Pelvic / curved lower fin',pelvic))
    for side in (-1,1):
        p.append(tube('Pectoral / pale tapered paddle '+str(side),FIN,[(side*.16,.32,.13),(side*.24,.29,.07),(side*.32,.25,.025)],[.045,.050,.001],20))
        lateral=side_rotate(body,-side);skin=surface(lateral)
        for j,y in enumerate((.47,.38)):
            # Long, tapered lateral bars lie on the actual stored cheek flank.
            z=-.13;outline=[(-side*z+.105*math.cos(k*math.tau/24),y+.018*math.sin(k*math.tau/24)) for k in range(24)]
            p.append(side_rotate(fitted_patch('Stripe / fitted lateral bar '+str(side)+' '+str(j),BAR,outline,skin,.003),side))
        x=side*.117;y=.45;skin=surface(body)
        eye=ellipsoid('Eye / large ivory oval '+str(side),WHITE,(x,y,skin(y,x)+.01),(.047,.062,.018),24,16);p.append(eye)
        es=surface(eye);pupil=ellipsoid('Eye / dark vertical pupil '+str(side),INK,(x,y,es(y,x)+.002),(.019,.034,.007),20,14);p.append(pupil)
        p.append(ellipsoid('Eye / glint '+str(side),WHITE,(x-.004,y+.013,surface(pupil)(y+.013,x-.004)+.001),(.005,.009,.003),12,8))
    skin=surface(body);points=[]
    for i in range(13):x=(i/12-.5)*.12;y=.372+.021*abs(x/.06);points.append((x,y,skin(y,x)+.002))
    p.append(tube('Mouth / shallow fitted notch',INK,points,[.0023]*len(points),10))
    return p

def octillery():
    body=loft('Body / continuous head neck and arm mantle',RED,curved_profiles(((.13,.22,.20,-.02),(.20,.245,.21,-.025),(.30,.19,.16,-.035),(.40,.125,.12,-.045),(.49,.14,.135,-.025),(.60,.235,.215,0),(.72,.265,.235,-.015),(.84,.235,.225,-.035),(.94,.16,.17,-.035),(.975,.024,.027,-.035)),4),36)
    p=[body]
    for i in range(8):
        a=math.tau*i/8+.20;direction=(math.sin(a),math.cos(a));length=.54 if i%2==0 else .49
        raw=[(direction[0]*r,y,direction[1]*r) for r,y in ((.15,.21),(.27,.13),(.39,.105),(length,.16),(length*.98,.29),(length*.91,.35))]
        points,radii=smooth_path(raw,[.089,.083,.067,.047,.023,.002],4)
        arm=tube('Arm / smooth curling tentacle '+str(i),RED,points,radii,16);p.append(arm)
        top=upper_skin(arm)
        for j,r in enumerate((.345,.445 if length>.5 else .413)):
            x=direction[0]*r;z=direction[1]*r;y=top(z,x)
            p.append(funnel(f'Sucker / raised yellow rim {i} {j}',YELLOW,[(0,.029,.029),(.012,.034,.034),(.018,.030,.030)],(x,y-.004,z),up=True,thickness=.009,segments=16))
            p.append(ellipsoid(f'Sucker / recessed gold well {i} {j}',WELL,(x,y+.001,z),(.021,.004,.021),12,8))
    top=upper_skin(body)
    for i,(x,z) in enumerate(((-.10,-.005),(.10,-.025),(0,-.16))):
        outline=[(x+.048*math.cos(k*math.tau/20),z+.05*math.sin(k*math.tau/20)) for k in range(20)]
        p.append(dorsal_patch('Crown / fitted gold spot '+str(i),YELLOW,outline,top))
    skin=surface(body)
    for side in (-1,1):
        x=side*.165;y=.705
        eye=ellipsoid('Eye / sleepy ivory oval '+str(side),WHITE,(x,y,skin(y,x)+.010),(.056,.074,.019),24,16);p.append(eye)
        es=surface(eye)
        pupil=ellipsoid('Eye / low dark pupil '+str(side),INK,(x,y-.024,es(y-.024,x)+.002),(.020,.029,.008),20,12);p.append(pupil)
        # A red upper lid covers the upper part of the white eye.
        outline=[(x-.051,y+.015),(x+.051,y+.015),(x+.030,y+.052),(x,y+.065),(x-.030,y+.052)]
        p.append(fitted_patch('Eye / fitted heavy red lid '+str(side),RED,outline,es,.007))
    p.append(funnel('Siphon / real hollow upward tube',RED,[(.18,.073,.075),(.25,.080,.079),(.33,.063,.061),(.37,.050,.048)],(0,.77,0),tilt=-.70,thickness=.013,segments=32))
    p.append(funnel('Siphon / dark inner lining',INK,[(.255,.066,.064),(.33,.050,.048),(.365,.037,.035)],(0,.766,0),tilt=-.70,thickness=.004,segments=32))
    return p

def sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported jet fish')
    p=remoraid() if species=='remoraid' else octillery()
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
