#!/usr/bin/env python3
"""Original Poliwag round blue tadpole, fitted belly spiral and thin oval tail."""
import math
from sculpt_geometry import ellipsoid,fitted_contour,crease_normals,tube
from goldfish_sculpt import surface
from bovine_sculpt import oval
from skin_glb import COORDINATES
HEIGHT=.90
BLUE=(.35,.53,.74);WHITE=(.98,.98,.94);INK=(.075,.085,.13)
PINK=(.88,.54,.68);PALE=(.79,.87,.92);REFLECTION=(.36,.39,.43)


def paint(name,color,xy,parent,th=.004,subdivisions=2):
    return crease_normals(fitted_contour(name,color,xy,surface(parent),th,subdivisions=subdivisions),45.)


def raw_parts():
    body=ellipsoid('Body / round blue Poliwag tadpole',BLUE,(0.,.306,0.),(.247,.250,.224),32,20)
    belly=paint('Belly / fitted broad ivory spiral field',WHITE,oval(0.,.270,.188,.186,48),body,.012,3)
    p=[body,belly];skin=surface(belly)
    curve=[]
    for i in range(161):
        t=i/160;r=.009+.149*t;a=1.4-math.tau*2.55*t;x=r*math.cos(a);y=.270+r*math.sin(a)
        curve.append((x,y,skin(y,x)+.004))
    p.append(tube('Spiral / continuous fitted dark belly coil',INK,curve,[.0065]*len(curve),12))
    for side in (-1,1):
        p.append(ellipsoid('Foot / planted blue tadpole paddle '+str(side),BLUE,(side*.096,.037,.055),(.047,.093,.074),20,14))
        eye=paint('Eye / large ivory tadpole eye '+str(side),WHITE,oval(side*.095,.462,.045,.056,32),body,.006);p.append(eye)
        pupil=paint('Pupil / large dark tadpole gaze '+str(side),INK,oval(side*.097,.464,.029,.040,28),eye,.004);p.append(pupil)
        p.append(paint('Eye glint / large ivory eye shine '+str(side),WHITE,oval(side*.089,.482,.010,.014,20),pupil,.002))
        p.append(paint('Eye reflection / lower gray eye shine '+str(side),REFLECTION,oval(side*.102,.445,.014,.010,20),pupil,.002))
    lips=ellipsoid('Mouth / round forward pink tadpole lips',PINK,(0.,.444,.198),(.043,.032,.033),24,16);p.append(lips)
    p.append(paint('Mouth opening / small fitted dark lip aperture',INK,oval(0.,.448,.013,.0065,20),lips,.002))
    p.append(tube('Tail stem / rooted blue tadpole tail',BLUE,
        [(0.,.176,-.156),(0.,.171,-.285),(0.,.190,-.472),(0.,.219,-.667),(0.,.235,-.797)],[.032,.026,.018,.009,.002],16))
    fin=ellipsoid('Tail fin / closed thin pale oval tadpole membrane',PALE,(0.,.225,-.583),(.014,.126,.287),32,20);p.append(fin)
    for side in (-1,1):
        rotated=dict(fin);vv=fin['positions'];rotated['positions']=[q for i in range(0,len(vv),3) for q in (vv[i+2],vv[i+1],side*vv[i])]
        fin_skin=surface(rotated);pts=[]
        for j in range(49):
            t=j/48;z=-.336-.480*t;y=.185+.055*t+.009*math.sin(math.pi*t)
            pts.append((side*(fin_skin(y,z)+.0015),y,z))
        p.append(tube('Tail vein / fitted blue membrane centerline '+str(side),BLUE,pts,[.0035]*len(pts),10))
    return p


def authoring_transform(parts=None):
    p=raw_parts() if parts is None else parts
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3])
    return lo,HEIGHT/(hi-lo)


def sculpture():
    p=raw_parts();lo,scale=authoring_transform(p)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'poliwag','version':1,'coordinate_system':COORDINATES,'primitives':p}
