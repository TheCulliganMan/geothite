#!/usr/bin/env python3
"""Distinct Ledyba and Ledian: fitted faces, spotted shells and six insect limbs."""
import math
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import ellipsoid, tube, part
from bovine_sculpt import oval, reverse_front
from raptor_bird_sculpt import paint
from lugia_sculpt import paddle
from skin_glb import COORDINATES

SPECIES=('ledyba','ledian')
HEIGHTS={'ledyba':.95,'ledian':1.05}
RED=(222/255,49/255,41/255)
GOLD={'ledyba':(214/255,156/255,66/255),'ledian':(1.,181/255,33/255)}
WHITE=(1.,1.,1.)
INK=(0.,0.,0.)

def shell_cases(species):
    """Two closed halves share one dome's positions and exterior normals."""
    evolved=species=='ledian'
    cy,cz,rx,ry,rz=(.485,-.073,.157,.220,.115) if evolved else (.450,-.100,.255,.320,.145)
    segments=36;rings=20 if evolved else 22;rows=rings+1
    vertices=[]
    for row in reversed(range(rows)):
        angle=math.pi*(row+.025)/(rings+.05)
        y=cy+ry*math.cos(angle)
        for column in range(segments):
            theta=math.tau*column/segments
            point=(rx*math.sin(angle)*math.sin(theta),y,cz+rz*math.sin(angle)*math.cos(theta))
            vertices.append(tuple(0. if abs(v)<1e-15 else v for v in point))
    faces=[tuple(reversed(range(segments))),tuple((rows-1)*segments+i for i in range(segments))]
    for row in range(rows-1):
        for column in range(segments):
            a=row*segments+column;b=row*segments+(column+1)%segments
            faces.append((a,b,b+segments,a+segments))
    whole=part('Shell / reference continuous red dome',RED,vertices,faces,True)
    normals={tuple(whole['positions'][i:i+3]):whole['normals'][i:i+3]
        for i in range(0,len(whole['positions']),3)}
    cases=[];width=segments//2+1
    for side,start in ((1,0),(-1,segments//2)):
        vs=[vertices[row*segments+(start+column)%segments]
            for row in range(rows) for column in range(width)]
        fs=[tuple(reversed(range(width))),tuple((rows-1)*width+i for i in range(width))]
        for row in range(rows-1):
            for column in range(width-1):
                a=row*width+column;b=a+1
                fs.append((a,b,b+width,a+width))
        boundary=[row*width for row in range(rows)]+[row*width+width-1 for row in reversed(range(rows))]
        # Separate vertices give the hidden cut face its own plane normals.
        cut_start=len(vs);vs.extend(vs[i] for i in boundary)
        fs.append(tuple(range(cut_start,len(vs))))
        q=part('Shell / continuous red '+('Ledian' if evolved else 'Ledyba')+' wing case '+str(side),RED,vs,fs,True)
        cut_indices=(len(boundary)-2)*3
        for i in set(q['indices'][:-cut_indices]):
            q['normals'][i*3:i*3+3]=normals[tuple(q['positions'][i*3:i*3+3])]
        cases.append(q)
    return {int(q['part'].split()[-1]):q for q in cases}

def back_paint(name,color,outline,parent,th=.006,sub=2):
    # A proper 180-degree turn preserves winding while fitting the rear skin.
    return reverse_front(paint(name,color,[(-x,y) for x,y in outline],
        reverse_front(parent),th,sub))

def glove(parts,name,center,radii):
    hand=ellipsoid(name,WHITE,center,radii,20,14)
    parts.append(hand)
    x,y,z=center
    outline=[(x-.018,y+.015),(x-.027,y+.006),(x-.024,y-.010),
        (x-.009,y-.020),(x+.005,y-.013),(x+.002,y-.009),
        (x-.008,y-.014),(x-.019,y-.007),(x-.021,y+.004),(x-.014,y+.011)]
    parts.append(paint('Glove seam / fitted curled finger '+name.split('/')[-1].strip(),
        INK,outline,hand,.003,2))

@lru_cache(maxsize=2)
def raw_sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported ladybird')
    evolved=species=='ledian';gold=GOLD[species];cases=shell_cases(species)
    body=ellipsoid('Abdomen / slender gold ladybird breast' if evolved else
        'Abdomen / round gold ladybird belly',gold,
        (0.,.48 if evolved else .44,.015),
        (.145,.225,.115) if evolved else (.247,.305,.185),32,22)
    head=ellipsoid('Head / rounded red ladybird face',RED,
        (0.,.87 if evolved else .85,.065 if evolved else .04),
        (.190,.170,.160) if evolved else (.212,.191,.165),32,22)
    parts=[body,head]
    if evolved:
        parts.append(back_paint('Head spot / fitted rear Ledian mark',INK,
            oval(0.,.948,.047,.039,24),head,.006,2))
    face=paint('Face / fitted gold lower mask',gold,
        oval(0.,.775 if evolved else .755,.125 if evolved else .160,.071 if evolved else .077,28),
        head,.008,3)
    parts.append(face)
    if evolved:
        parts.append(paint('Mouth / small dark Ledian mouth',INK,
            oval(0.,.750,.013,.008,16),face,.004,2))
    parts.append(paint('Breast seam / fitted central ladybird line',INK,
        [(-.002,.30 if evolved else .19),(-.002,.65 if evolved else .67),
         (.002,.65 if evolved else .67),(.002,.30 if evolved else .19)],body,.004,3))
    for side in (-1,1):
        x=side*(.090 if evolved else .106);y=.907 if evolved else .90
        eye=paint('Eye / fitted white ladybird lens '+str(side),WHITE,
            oval(x,y,.056 if evolved else .066,.072 if evolved else .080,24),head,.008,3)
        parts.append(eye)
        pupil=paint('Pupil / dark ladybird gaze '+str(side),INK,
            oval(x-side*.006,y,.034 if evolved else .035,.050 if evolved else .053,22),eye,.004,2)
        parts.append(pupil)
        parts.append(paint('Glint / bright ladybird eye '+str(side),WHITE,
            oval(x-side*.015,y+.023,.013,.017,16),pupil,.003,2))
        parts.append(paddle('Antenna / tapered bent black feeler '+str(side),INK,
            [(side*.076,1.013,.024),(side*.117,1.115,-.012),
             (side*(.159 if evolved else .141),1.235 if evolved else 1.200,-.017)],
            [.020,.024,.002],[.017,.017,.002],segments=16,steps=5))
        if evolved:
            for row,(root_y,hand_x,hand_y) in enumerate(((.655,.294,.565),(.503,.245,.414))):
                parts.append(tube('Arm / dark jointed Ledian limb '+str(side)+' '+str(row),INK,
                    [(side*.132,root_y,.028),(side*(hand_x-.045),hand_y+.022,.061),
                     (side*hand_x,hand_y,.100)], [.025,.027,.018],16))
                glove(parts,'Hand / white Ledian fist '+str(side)+' '+str(row),
                    (side*hand_x,hand_y,.115),(.056,.062,.053))
            parts.append(tube('Leg / dark weight-bearing Ledian shank '+str(side),INK,
                [(side*.090,.300,.006),(side*.083,.173,.014),(side*.081,.078,.043)],
                [.026,.025,.019],16))
            parts.append(ellipsoid('Foot / planted red Ledian boot '+str(side),RED,
                (side*.081,.061,.060),(.062,.061,.091),22,16))
            case=cases[side]
            parts.append(case)
            for j,spot_y in enumerate((.565,.405)):
                parts.append(back_paint('Shell spot / round Ledian mark '+str(side)+' '+str(j),INK,
                    oval(side*.066,spot_y,.034,.041,24),case,.006,2))
            parts.append(paddle('Wing / folded pale Ledian flight membrane '+str(side),WHITE,
                [(side*.077,.659,-.101),(side*.229,.519,-.149),(side*.330,.433,-.188)],
                [.015,.068,.006],[.010,.013,.002],segments=20,steps=6))
        else:
            for row,(root_y,hand_x,hand_y) in enumerate(((.659,.337,.704),(.465,.324,.476))):
                parts.append(tube('Arm / dark jointed Ledyba limb '+str(side)+' '+str(row),INK,
                    [(side*.187,root_y,.047),(side*(hand_x-.062),hand_y-.012,.086),
                     (side*hand_x,hand_y,.132)],[.021,.025,.018],16))
                glove(parts,'Hand / white Ledyba fist '+str(side)+' '+str(row),
                    (side*hand_x,hand_y,.148),(.061,.060,.058))
            parts.append(tube('Leg / dark weight-bearing Ledyba limb '+str(side),INK,
                [(side*.162,.242,.030),(side*.153,.142,.082),(side*.139,.063,.129)],
                [.022,.023,.018],16))
            foot=ellipsoid('Foot / planted white Ledyba glove '+str(side),WHITE,
                (side*.139,.053,.151),(.065,.053,.065),20,14)
            parts.append(foot)
            case=cases[side]
            parts.append(case)
            for j,spot_y in enumerate((.584,.346)):
                parts.append(back_paint('Shell spot / round Ledyba mark '+str(side)+' '+str(j),INK,
                    oval(side*.099,spot_y,.056,.062,24),case,.008,2))
            # The fifth spot is split across the two independently riggable cases.
            half=[(side*.035*math.sin(math.pi*i/16),.630+.036*math.cos(math.pi*i/16))
                for i in range(17)]
            parts.append(back_paint('Shell spot / fifth joined cap '+str(side),INK,half,case,.008,2))
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':parts}

def sculpture(species,*,normalize=True):
    model=deepcopy(raw_sculpture(species))
    if normalize:
        parts=model['primitives']
        low=min(v for p in parts for v in p['positions'][1::3])
        high=max(v for p in parts for v in p['positions'][1::3])
        scale=HEIGHTS[species]/(high-low)
        for p in parts:
            p['positions']=[round((v-(low if i%3==1 else 0.))*scale,7)
                for i,v in enumerate(p['positions'])]
            p['normals']=[round(v,7) for v in p['normals']]
    return model
