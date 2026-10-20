#!/usr/bin/env python3
"""Original three-dimensional A..Z Unown glyphs with a continuous dark eye ring."""
import math
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import ellipsoid,part
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.0
BODY=(57/255,57/255,66/255);WHITE=(1.,1.,1.);INK=(0.,0.,0.)
CENTER=(0.,.55,0.)
# Authored planar centerlines. Each form retains the supplied frontpic's
# characteristic topology; these are original smooth volumetric strokes.
def oval_path(y,rx,ry,segments=24,*,bottom=False):
 return [(rx*math.sin(i*math.tau/segments),y-ry*math.cos(i*math.tau/segments) if bottom else y+ry*math.cos(i*math.tau/segments)) for i in range(segments+1)]

STROKES={
 'a': [[(0,.73),(0,1.02)],[(-.10,.38),(-.18,.04)],[(.10,.38),(.18,.04)],[(-.13,.19),(.13,.19)]],
 'b': [[(.17,.55),(.30,.55)],[(0,.37),(0,.32)],oval_path(.18,.14,.14)],
 'c': [[(.17,.55),(.35,.55),(.30,.76),(.12,.92),(-.12,.92),(-.34,.73),(-.37,.49),(-.26,.27),(0,.13),(0,.02)]],
 'd': [[(0,.73),(0,1.00),(.20,.92),(.36,.74),(.39,.55),(.33,.34),(.16,.18),(0,.16)]],
 'e': [[(-.17,.55),(-.36,.55)],[(.17,.55),(.35,.55)],[(0,.73),(.05,.89),(.28,.89)],[(0,.37),(.05,.21),(.28,.21)]],
 'f': [[(.17,.60),(.43,.60),(.30,.40),(.17,.60)],[(0,.37),(0,.02)],[(-.10,.02),(.12,.02)],[(0,.18),(.15,.18)]],
 'g': [[(0,.73),(0,.85)],oval_path(.96,.18,.095,bottom=True),[(0,.37),(-.17,.21),(.16,.06)]],
 'h': [[(-.17,.55),(-.35,.55)],[(.17,.55),(.35,.55)],[(-.15,.96),(-.33,.77),(-.37,.55),(-.30,.32),(-.10,.14)],[(.15,.96),(.33,.77),(.37,.55),(.30,.32),(.10,.14)]],
 'i': [[(0,.73),(0,1.00)],[(0,.37),(0,.10)]],
 'j': [[(.12,.69),(.22,.83)],[(0,.37),(0,.13),(-.10,.05),(-.24,.07),(-.24,.18)]],
 'k': [[(0,.73),(0,.98)],[(0,.37),(0,.13)],[(.17,.55),(.35,.76)],[(.17,.55),(.35,.34)]],
 'l': [[(-.12,.69),(-.21,.82)],[(0,.37),(0,.12),(.25,.12)]],
 'm': [[(-.12,.69),(-.24,.83)],[(.12,.69),(.24,.83)],[(-.27,.15),(-.36,.32),(-.36,.56),(-.22,.78),(0,.85),(.22,.78),(.36,.56),(.36,.32),(.27,.15)]],
 'n': [[(-.17,.55),(-.36,.73),(-.36,.22)],[(.17,.55),(.36,.37),(.36,.88)]],
 'o': [[(0,.37),(0,.14)],oval_path(.57,.40,.43,32,bottom=True)],
 'p': [[(-.12,.69),(-.22,.82)],[(-.17,.55),(-.17,.13)]],
 'q': [[(.13,.42),(.32,.23),(.40,.23)],[(.27,.29),(.36,.38)]],
 'r': [[(-.17,.55),(-.17,.15)],[(.11,.41),(.26,.30)]],
 's': [[(0,.73),(-.25,.79),(-.08,.96),(0,1.01),(0,1.12)],[(0,.37),(.25,.31),(.08,.14),(0,.09),(0,-.02)]],
 't': [[(0,.73),(0,1.00)],[(-.28,1.00),(.28,1.00)]],
 'u': [[(-.12,.42),(-.18,.25)],[(.12,.42),(.18,.25)],[(-.37,.58),(-.34,.39),(-.20,.23),(0,.18),(.20,.23),(.34,.39),(.37,.58)]],
 'v': [[(0,.73),(-.17,.90),(-.17,1.03),(.17,1.03),(.17,.90),(0,.73)],[(.17,1.03),(.33,1.14)]],
 'w': [[(-.14,.44),(-.30,.65),(-.36,.83)],[(.14,.44),(.30,.65),(.36,.83)],[(0,.73),(0,1.02)]],
 'x': [[(-.12,.69),(-.25,.84)],[(.12,.69),(.25,.84)],[(-.12,.41),(-.25,.26)],[(.12,.41),(.25,.26)]],
 # The stem meets the separate foot at its center; ending at .12 leaves a gap.
 'y': [[(-.12,.69),(-.27,.85)],[(.12,.69),(.27,.85)],[(0,.37),(0,.05)],[(-.12,.05),(.12,.05)]],
 'z': [[(0,.73),(-.25,.93),(-.34,.93)],[(0,.37),(.25,.17),(.34,.17)]],
}

def closed_glyph_tube(name,path):
 points=path[:-1];n=len(points);sides=16;vertices=[];faces=[]
 for i,(x,y) in enumerate(points):
  previous=points[(i-1)%n];following=points[(i+1)%n]
  tx=following[0]-previous[0];ty=following[1]-previous[1]
  # Explicit scaled hypot retains the reviewed stroke frames when Python's
  # platform hypot implementation changes its final rounding bit.
  scale=max(abs(tx),abs(ty))
  length=scale*math.sqrt((tx/scale)**2+(ty/scale)**2) if scale else 0.
  if length<1e-9:raise ValueError('closed glyph loop has no tangent')
  nx,ny=ty/length,-tx/length
  for j in range(sides):
   angle=j*math.tau/sides;r=.043*math.cos(angle)
   vertices.append((x+nx*r,y+ny*r,.043*math.sin(angle)))
 for i in range(n):
  for j in range(sides):
   a=i*sides+j;b=((i+1)%n)*sides+j;c=((i+1)%n)*sides+(j+1)%sides;d=i*sides+(j+1)%sides
   faces.append((a,b,c,d))
 return part(name,BODY,vertices,faces,True)

def ring():
 v=[];f=[];segments=48;sides=12
 for i in range(segments):
  a=i*math.tau/segments
  for j in range(sides):
   b=j*math.tau/sides;r=.183+.048*math.cos(b)
   v.append((r*math.cos(a),.55+r*math.sin(a),.048*math.sin(b)))
 for i in range(segments):
  for j in range(sides):
   a=i*sides+j;b=((i+1)%segments)*sides+j;c=((i+1)%segments)*sides+(j+1)%sides;d=i*sides+(j+1)%sides
   f.append((a,b,c,d))
 return part('Ring / continuous thick dark Unown eye socket',BODY,v,f,True)

@lru_cache(maxsize=26)
def _raw_sculpture(letter):
 if letter not in STROKES:raise ValueError('Unown requires an A..Z form')
 eye=ellipsoid('Eye / smooth white Unown lens',WHITE,(0.,.55,.025),(.156,.156,.032),32,18)
 pupil=paint('Pupil / small fitted forward Unown black dot',INK,oval(0.,.55,.043,.043,24),eye,.003,3)
 back=ellipsoid('Back / solid dark Unown eye reverse',BODY,(0.,.55,-.022),(.160,.160,.055),32,18)
 p=[ring(),eye,pupil,back]
 for i,path in enumerate(STROKES[letter]):
  closed=math.dist(path[0],path[-1])<1e-9
  name='Glyph / '+letter.upper()+(' closed loop' if closed else ' authored letter')+' stroke '+str(i)
  p.append(closed_glyph_tube(name,path) if closed else curved_tube(name,BODY,[(x,y,0.) for x,y in path],[.043]*len(path),16))
 return {'name':'unown_'+letter,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def sculpture(letter,*,normalize=True):
 model=deepcopy(_raw_sculpture(letter));p=model['primitives']
 if normalize:
  low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:
   q['positions']=[round((v-(low if j%3==1 else 0.))*scale,7) for j,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model
