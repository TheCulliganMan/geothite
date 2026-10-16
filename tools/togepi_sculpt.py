#!/usr/bin/env python3
"""Original joyful Togepi with a real hollow broken shell and open ring markings."""
import argparse,json,math
from functools import lru_cache
from copy import deepcopy
from pathlib import Path
from sculpt_geometry import part,curved_profiles,ellipsoid,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube,reverse_front
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=.94
CREAM=(.98,.91,.69);SHELL=(.97,.97,.91);INK=(.13,.10,.13);RED=(.89,.38,.27);BLUE=(.29,.66,.81);PINK=(.91,.57,.62);WHITE=(.99,.99,.94)

def shell():
    n=80;rows=curved_profiles(((.073,.038,.033,0),(.128,.137,.119,0),(.258,.228,.190,0),(.402,.278,.226,0),(.520,.268,.224,0),(.559,.230,.199,0)),4);v=[]
    for y,w,d,z in rows:v.extend((w*math.sin(j*math.tau/n),y,d*math.cos(j*math.tau/n)) for j in range(n))
    for j in range(n):
        a=j*math.tau/n;wave=2*abs(2*((j*10/n)%1)-1)-1;v.append((.222*math.sin(a),.607+.034*wave,.194*math.cos(a)))
    nr=len(rows)+1;inner=len(v)
    for y,w,d in ((None,.203,.175),(.142,.122,.105)):
        for j in range(n):
            a=j*math.tau/n;wave=2*abs(2*((j*10/n)%1)-1)-1;v.append((w*math.sin(a),.607+.034*wave if y is None else y,d*math.cos(a)))
    f=[tuple(reversed(range(n))),tuple(inner+n+j for j in range(n))]
    for k in range(nr-1):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
    for j in range(n):
        b=(j+1)%n;f.append(((nr-1)*n+j,(nr-1)*n+b,inner+b,inner+j));f.append((inner+j,inner+b,inner+n+b,inner+n+j))
    return crease_normals(part('Shell / continuous hollow ivory egg with ten cracked rim teeth',SHELL,v,f,True),35.)

def head():
    n=96;rows=65;v=[]
    directions=[(math.sin(a),math.cos(a),amplitude) for a,amplitude in ((-math.pi/2,.103),(-.82,.137),(0,.150),(.82,.137),(math.pi/2,.103))]
    for k in range(rows):
        phi=.008+(math.pi-.016)*k/(rows-1);uy=math.cos(phi)
        for j in range(n):
            t=j*math.tau/n;ux=math.sin(phi)*math.sin(t);uz=math.sin(phi)*math.cos(t)
            x=.170*ux;y=.705+(.170 if uy>=0 else .600)*uy;z=.142*uz
            for dx,dy,amplitude in directions:
                bulge=amplitude*math.exp((ux*dx+uy*dy-1)/.027);x+=dx*bulge;y+=dy*bulge
            v.append((x,y,z))
    f=[tuple(reversed(range(n))),tuple((rows-1)*n+j for j in range(n))]
    for k in range(rows-1):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part('Head / continuous rounded five-point cream star',CREAM,v,f,True),40.)

def ring(name,color,outline,skin):
    cx=sum(x for x,y in outline)/len(outline);cy=sum(y for x,y in outline)/len(outline);edge=[]
    for a,b in zip(outline,outline[1:]+outline[:1]):edge.extend((a[0]+(b[0]-a[0])*j/10,a[1]+(b[1]-a[1])*j/10) for j in range(10))
    n=len(edge);radii=(.55,.70,.85,1.);v=[]
    for offset in (.009,-.003):
        for r in radii:
            for x,y in edge:
                x=cx+(x-cx)*r;y=cy+(y-cy)*r;v.append((x,y,skin(y,x)+offset))
    f=[];back=4*n
    for k in range(3):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.extend(((a,a+n,b+n,b),(a+back,b+back,b+n+back,a+n+back)))
    for j in range(n):
        b=(j+1)%n;f.extend(((j,b,b+back,j+back),(3*n+b,3*n+j,3*n+j+back,3*n+b+back)))
    return crease_normals(part(name,color,v,f),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
    egg=shell();star=head();p=[egg,star];ss=surface(egg);hs=surface(star)
    designs=[('large blue front triangle',BLUE,[(-.104,.494),(.053,.400),(-.071,.311)],False),('small red front lozenge',RED,[(.129,.509),(.201,.505),(.201,.426),(.146,.419)],False),('lower blue front triangle',BLUE,[(.095,.245),(.181,.339),(.179,.240)],False),('lower red front lozenge',RED,[(-.184,.296),(-.094,.236),(-.113,.193),(-.197,.256)],False),('large blue rear triangle',BLUE,[(-.075,.517),(.089,.426),(-.057,.332)],True),('small red rear lozenge',RED,[(-.199,.345),(-.124,.322),(-.104,.263),(-.174,.271)],True),('small blue rear triangle',BLUE,[(.055,.244),(.137,.299),(.137,.232)],True)]
    for name,color,xy,rear in designs:
        parent=reverse_front(egg) if rear else egg;patch=ring('Shell ring / '+name,color,xy,surface(parent));p.append(reverse_front(patch) if rear else patch)
    for rear,xy in ((False,[(-.209,.514),(-.146,.535),(-.171,.461)]),(True,[(.118,.527),(.190,.484),(.137,.458)])):
        parent=reverse_front(egg) if rear else egg;patch=fitted_contour('Shell fleck / small solid red triangle '+str(int(rear)),RED,xy,surface(parent),.007,subdivisions=3);p.append(reverse_front(patch) if rear else patch)
    for sign in (-1,1):
        x=sign*.061;eye=fitted_patch('Eye / small joyful dark vertical oval '+str(sign),INK,oval(x,.775,.014,.028,32),hs,.005);p.append(eye)
        p.append(fitted_patch('Eye glint / ivory upper sparkle '+str(sign),WHITE,oval(x-.001,.787,.005,.008,20),surface(eye),.002))
        brow=contour([((x-sign*.025,.768),(x-sign*.042,.737),(x-sign*.036,.716),(x-sign*.025,.714)),((x-sign*.025,.714),(x-sign*.031,.735),(x-sign*.029,.746),(x-sign*.025,.768))],9)
        brow=[(x,y+.033) for x,y in brow]
        p.append(fitted_contour('Eye crease / soft cream side expression '+str(sign),(.68,.59,.43),brow,hs,.002))
        p.append(curved_tube('Arm / short raised tapered cream hand '+str(sign),CREAM,[(sign*.208,.543,0),(sign*.286,.594,.036),(sign*.345,.653,.050)],[.043,.043,.003],24))
        p.append(curved_tube('Leg / short cream shell support '+str(sign),CREAM,[(sign*.132,.151,-.010),(sign*.151,.067,.044)],[.035,.034],24))
        foot=ellipsoid('Foot / grounded rounded cream pad '+str(sign),CREAM,(sign*.152,.043,.052),(.095,.043,.081),32,20);p.append(foot)
        fs=surface(foot);crease=[(sign*.156-.004,.063),(sign*.156+.004,.063),(sign*.176+.003,.034),(sign*.176-.003,.034)]
        p.append(fitted_contour('Foot crease / fitted soft toe division '+str(sign),(.68,.59,.43),crease,fs,.0015))
    smile=contour([((-.033,.710),(-.010,.703),(.018,.706),(.038,.716)),((.038,.716),(.021,.644),(-.023,.642),(-.033,.710))],12)
    smile=[(x,y+.033) for x,y in smile]
    mouth=fitted_contour('Mouth / fitted delighted open grin',INK,smile,hs,.007);p.append(mouth)
    tongue=contour([((-.020,.671),(-.009,.676),(.007,.677),(.020,.676)),((.020,.676),(.006,.651),(-.013,.651),(-.020,.671))],10)
    tongue=[(x*.62,y+.051) for x,y in tongue]
    p.append(fitted_contour('Tongue / rosy lower smile',PINK,tongue,surface(mouth),.003))
    p=[crease_normals(q,35.) if q['part'].startswith(('Eye /','Eye glint /','Eye crease /','Foot crease /','Mouth /','Tongue /','Shell fleck /')) else q for q in p]
    return {'name':'togepi','version':1,'coordinate_system':COORDINATES,'primitives':p}
def sculpture(normalize=True):
    model=deepcopy(raw_sculpture())
    if not normalize:return model
    p=model['primitives']
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'togepi','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    from egg_fairy_glb import export_egg_fairy
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'togepi.glb').write_bytes(export_egg_fairy('togepi'))
if __name__=='__main__':main()
