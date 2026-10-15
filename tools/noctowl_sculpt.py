#!/usr/bin/env python3
"""Original Noctowl: swept crown brows, fitted owl eyes and broad grasping feet."""
import math
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import ellipsoid, loft, tube
from bovine_sculpt import oval
from raptor_bird_sculpt import paint
from lugia_sculpt import paddle
from skin_glb import COORDINATES

HEIGHT = 1.30
# Verified normal pack RGB555 (18,12,4) and (11,5,6).
BROWN = (148/255, 99/255, 33/255)
DARK = (90/255, 41/255, 49/255)
WHITE = (1., 1., 1.)
INK = (0., 0., 0.)

@lru_cache(maxsize=1)
def raw_sculpture():
    body = loft('Body / continuous tawny owl head and breast', BROWN, [
        (.17,.11,.11,-.02), (.26,.20,.16,-.02), (.43,.255,.205,0.),
        (.65,.26,.225,0.), (.81,.215,.20,.025), (.92,.225,.175,.045),
        (1.02,.245,.195,.060), (1.13,.270,.20,.065),
        (1.25,.255,.17,.065), (1.32,.13,.105,.055), (1.35,.008,.008,.04),
    ], 32)
    parts = [body]
    for side in (-1,1):
        mask = paint('Eye mask / fitted pale owl disk '+str(side), WHITE,
            oval(side*.111,1.165,.103,.109,24),body,.008,3)
        parts.append(mask)
        ring = paint('Eye ring / fitted dark owl outline '+str(side), DARK,
            oval(side*.111,1.165,.084,.090,24),mask,.004,2)
        parts.append(ring)
        eye = paint('Eye / fitted pale owl lens '+str(side), WHITE,
            oval(side*.111,1.165,.070,.077,24),ring,.004,2)
        parts.append(eye)
        pupil = paint('Pupil / focused dark owl gaze '+str(side), INK,
            oval(side*.101,1.165,.027,.044,20),eye,.003,2)
        parts.append(pupil)
        parts.append(paint('Glint / small pale owl eye '+str(side),WHITE,
            oval(side*.101-.007,1.187,.007,.010,12),pupil,.002,1))
        # The brows join low at the nose and sweep into tall, forked crown horns.
        parts.append(paddle('Brow / sweeping pale crown bridge '+str(side),WHITE,
            [(side*.010,1.115,.265),(side*.135,1.190,.259),
             (side*.230,1.275,.204)], [.014,.029,.047], [.012,.017,.025],
            segments=18,steps=5))
        parts.append(paddle('Crown / broad upright owl horn '+str(side),WHITE,
            [(side*.228,1.245,.185),(side*.244,1.405,.120),
             (side*.256,1.575,.073)], [.061,.075,.003], [.017,.010,.002],
            segments=18,steps=5))
        parts.append(paddle('Crown / outer forked owl horn '+str(side),WHITE,
            [(side*.244,1.370,.127),(side*.292,1.439,.095),
             (side*.304,1.519,.068)], [.031,.025,.002], [.019,.014,.002],
            segments=16,steps=4))
        parts.append(paddle('Wing / long dark folded owl wing '+str(side),DARK,
            [(side*.224,1.020,.004),(side*.313,.680,-.006),
             (side*.227,.280,-.128)], [.055,.091,.012], [.049,.047,.008],
            segments=22,steps=6))
        for j in range(2):
            parts.append(paddle('Flight feather / seated folded owl tip '+str(side)+' '+str(j),DARK,
                [(side*(.275+.014*j),.620,-.032),(side*(.265+.016*j),.403,-.081),
                 (side*(.192+.028*j),.240,-.173)], [.028,.035,.002], [.025,.022,.002],
                segments=16,steps=5))
        parts.append(tube('Leg / stout pale owl shank '+str(side),WHITE,
            [(side*.107,.255,-.005),(side*.110,.129,.022),(side*.110,.055,.059)],
            [.035,.030,.025],16))
        parts.append(ellipsoid('Foot / broad planted owl pad '+str(side),WHITE,
            (side*.110,.041,.080),(.076,.041,.082),20,14))
        for j,dx in enumerate((-.055,0.,.055)):
            parts.append(tube('Toe / grasping forward owl digit '+str(side)+' '+str(j),WHITE,
                [(side*.110+dx*.25,.038,.113),(side*.110+dx,.026,.185),
                 (side*.110+dx*1.2,.020,.229)], [.020,.016,.009],14))
            parts.append(paddle('Claw / curved forward owl talon '+str(side)+' '+str(j),DARK,
                [(side*.110+dx*1.2,.024,.220),(side*.110+dx*1.2,.020,.253),
                 (side*.110+dx*1.2,.004,.279)], [.010,.008,.001], [.012,.007,.001],
                segments=14,steps=4))
        parts.append(tube('Rear toe / opposing owl hallux '+str(side),WHITE,
            [(side*.110,.032,.061),(side*.122,.019,-.005),(side*.137,.013,-.054)],
            [.016,.013,.006],14))
    for row,y in enumerate((.706,.488)):
        for j,x in enumerate((-.155,0.,.155)):
            width=.044 if j else .052
            parts.append(paint('Breast mark / fitted dark owl feather '+str(row)+' '+str(j),DARK,
                [(x-width,y+.051),(x,y-.064),(x+width,y+.051)],body,.005,2))
    parts.append(paddle('Beak / short pale hooked owl bill',WHITE,
        [(0.,1.114,.246),(0.,1.075,.302),(0.,1.030,.301)],
        [.029,.025,.003],[.022,.022,.002],segments=18,steps=5))
    parts.append(paint('Mouth / fitted owl bill line',DARK,
        [(-.045,1.011),(0.,1.003),(.045,1.011),(.040,1.004),
         (0.,.997),(-.040,1.004)],body,.003,2))
    for j,x in enumerate((-.075,0.,.075)):
        parts.append(paddle('Tail / broad layered owl fan '+str(j),DARK,
            [(x*.6,.355,-.168),(x,.268,-.324),(x*1.2,.237,-.490)],
            [.031,.045,.005],[.027,.021,.003],segments=18,steps=5))
    return {'name':'noctowl','version':1,'coordinate_system':COORDINATES,'primitives':parts}

def sculpture(*,normalize=True):
    model=deepcopy(raw_sculpture())
    if normalize:
        parts=model['primitives']
        low=min(v for p in parts for v in p['positions'][1::3])
        high=max(v for p in parts for v in p['positions'][1::3])
        scale=HEIGHT/(high-low)
        for p in parts:
            p['positions']=[round((v-(low if i%3==1 else 0.))*scale,7)
                for i,v in enumerate(p['positions'])]
            p['normals']=[round(v,7) for v in p['normals']]
    return model
