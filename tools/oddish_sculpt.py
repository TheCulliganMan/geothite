#!/usr/bin/env python3
"""Original Oddish: round blue bulb, two root feet and five broad cupped leaves."""
import math
from sculpt_geometry import ellipsoid,fitted_contour,crease_normals
from goldfish_sculpt import surface
from bovine_sculpt import oval
from pitcher_sculpt import leaf
from skin_glb import COORDINATES
HEIGHT=1.00
BLUE=(.29,.45,.61);GREEN=(.33,.62,.20);VEIN=(.22,.47,.13)
RED=(.93,.35,.27);WHITE=(.99,.98,.91);MOUTH=(.30,.13,.25);PINK=(.94,.61,.69)
LEAVES=(((-.027,.461,.008),(-.279,.801,.262),.099),
        ((-.012,.472,-.020),(-.239,.803,-.274),.103),
        ((.004,.480,-.017),(.042,1.008,-.025),.103),
        ((.022,.467,.015),(.307,.768,.266),.128),
        ((.041,.445,.022),(.335,.659,-.275),.119))


def paint(name,color,xy,parent,th=.004):
    return crease_normals(fitted_contour(name,color,xy,surface(parent),th,subdivisions=3),45.)


def raw_parts():
    body=ellipsoid('Body / round blue Oddish bulb',BLUE,(0.,.287,0.),(.221,.206,.187),28,18)
    p=[body]
    for side in (-1,1):
        p.append(ellipsoid('Foot / planted blue root foot '+str(side),BLUE,(side*.101,.082,.027),(.059,.083,.068),20,14))
        eye=paint('Eye / small red oval gaze '+str(side),RED,oval(side*.077,.328,.011,.017,20),body,.003);p.append(eye)
        p.append(paint('Eye glint / tiny ivory gaze highlight '+str(side),WHITE,oval(side*.079,.334,.0035,.005,12),eye,.001))
    mouth=paint('Mouth / open happy dark wedge',MOUTH,[(-.026,.293),(.026,.293),(.016,.252),(0.,.239),(-.016,.252)],body,.003);p.append(mouth)
    p.append(paint('Tongue / fitted pink lower mouth',PINK,[(-.018,.265),(-.009,.247),(0.,.243),(.009,.247),(.018,.265),(.010,.273),(-.010,.273)],mouth,.002))
    for j,(start,end,width) in enumerate(LEAVES):
        blade,midrib=leaf('Leaf '+str(j),start,end,width,GREEN,bend=.035 if j%2==0 else -.012)
        # Geometry helper's botanical midrib uses this species' dark green material.
        from chikorita_sculpt import linear
        midrib['base_color']=linear(VEIN)
        p.extend((blade,midrib))
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
    return {'name':'oddish','version':1,'coordinate_system':COORDINATES,'primitives':p}
