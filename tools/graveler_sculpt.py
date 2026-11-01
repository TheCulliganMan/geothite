#!/usr/bin/env python3
"""Original four-armed boulder with a fitted rock face and blunt stone digits."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,translate,crease_normals
from geodude_sculpt import stone_block
from chikorita_sculpt import sub,cross,dot,unit
from goldfish_sculpt import surface
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.10
STONE=(.58,.59,.53);LIGHT=(.62,.63,.56);SHADE=(.52,.53,.47)
IVORY=(.98,.97,.92);INK=(.075,.08,.075);PINK=(.29,.25,.23)

def rock_field(p,parent):
 at=surface(parent);out=dict(p,positions=[],normals=[],indices=[]);lookup={}
 for k in range(0,len(p['indices']),3):
  ids=p['indices'][k:k+3];vv=[p['positions'][3*i:3*i+3] for i in ids];offset=[z-at(y,x) for x,y,z in vv];side=1 if all(v>.01 for v in offset) else (-1 if all(v<-.001 for v in offset) else 0);fn=unit(cross(sub(vv[1],vv[0]),sub(vv[2],vv[0])))
  for i,(x,y,z) in zip(ids,vv):
   normal=p['normals'][3*i:3*i+3]
   if side:
    normal=unit((side*x/.4**2,side*(y-.5)/.42**2,side*at(y,x)/.325**2));correction=max(0,.01-dot(normal,fn));normal=unit(tuple(a+correction*b for a,b in zip(normal,fn)))
   key=(x,y,z,*normal)
   if key not in lookup:lookup[key]=len(out['positions'])//3;out['positions'].extend((x,y,z));out['normals'].extend(normal)
   out['indices'].append(lookup[key])
 return out

def anatomy():
 # Uneven ledges and changing cross-sections carry the silhouette through the
 # rear and profiles, instead of putting a crown decal on a regular egg.
 base=loft('Body / original irregular grey Graveler boulder',STONE,
  [(.08,.12,.14,0),(.16,.30,.27,0),(.30,.39,.33,-.015),
   (.46,.43,.36,-.015),(.62,.45,.37,-.025),(.76,.43,.36,-.035),
   (.86,.37,.32,-.045),(.92,.30,.26,-.045),(.945,.18,.17,-.045),
   (.96,.055,.055,-.045)],20);vertices=[]
 for i in range(0,len(base['positions']),3):
  x,y,z=base['positions'][i:i+3];a=math.atan2(z,x);q=(y-.53)/.5
  factor=1+(.085*math.sin(5*a+.9)+.055*math.cos(3*a-1.2)+.035*math.sin(7*a+15*y))*max(0,1-q*q)
  yy=y+.035*math.cos(3*a+.4)*max(0,1-q*q)+.045*math.cos(5*a-.3)*max(0,(y-.65)/.36)
  vertices.append((x*factor,yy,z*factor))
 body=part(base['part'],STONE,vertices,[tuple(base['indices'][i:i+3]) for i in range(0,len(base['indices']),3)],False);p=[body]
 brow=[(-.23,.70),(-.17,.76),(-.08,.73),(0,.79),(.08,.73),(.17,.76),(.23,.70),(.20,.63),(.11,.65),(0,.61),(-.11,.65),(-.20,.63)]
 p.append(rock_field(paint('Forehead / fitted jagged light Graveler rock field',STONE,brow,body,.012,3),body))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.025,.566),(.166,.605),(.177,.547),(.045,.531))]
  socket=paint('Socket / fitted narrow dark Graveler gaze '+str(s),INK,xy,body,.009,3);p.append(socket);cx=sum(x for x,y in xy)/4;cy=sum(y for x,y in xy)/4
  eye=paint('Eye / fitted narrow ivory Graveler stare '+str(s),IVORY,[(cx+.85*(x-cx),cy+.79*(y-cy)) for x,y in xy],socket,.005,3);p.append(eye)
  p.append(paint('Pupil / fitted dark Graveler slit '+str(s),INK,oval(s*.115,.565,.008,.015,20),eye,.003,2))
 smile=[(-.177,.474),(-.092,.459),(.092,.459),(.177,.474),(.125,.433),(0,.423),(-.125,.433)]
 mouth=paint('Mouth / fitted dark curved Graveler grin',INK,smile,body,.008,3);p.append(mouth)
 p.append(paint('Mouth inset / fitted pink Graveler grin',PINK,[(-.143,.460),(-.078,.450),(.078,.450),(.143,.460),(.105,.442),(0,.435),(-.105,.442)],mouth,.004,3))
 for s in (-1,1):
  for level in ('upper','lower'):
   upper=level=='upper';name=str(s)+' '+level
   if upper:
    path=[(s*.292,.66,.0),(s*.48,.65,.03),(s*.62,.58,.14)];ww=[.105,.084,.074];dd=[.107,.086,.075];center=(s*.650,.545,.175);radii=(.115,.090,.090)
   else:
    path=[(s*.28,.31,.05),(s*.435,.245,.16)];ww=[.075,.065];dd=[.077,.070];center=(s*.485,.270,.320);radii=(.095,.080,.080)
   p.append(crease_normals(paddle('Arm / rooted '+level+' grey Graveler arm '+str(s),STONE,path,ww,dd,segments=8,steps=2),25))
   if not upper:p.append(crease_normals(paddle('Forearm / rooted folded lower grey Graveler forearm '+str(s),STONE,[(s*.435,.245,.16),(s*.46,.252,.235),(s*.478,.265,.299)],[.065,.061,.058],[.070,.065,.060],segments=8,steps=2),25))
   p.append(stone_block('Hand / broad '+level+' grey Graveler palm '+str(s),STONE,center,radii))
   x,y,z=center
   for j,dx in enumerate((-.069,0,.069)):
    if upper:
     points=[(x+dx*.75,y-.015,z+.035),(x+dx*1.20,y-.026+(.018 if j==1 else 0),z+.112),(x+dx*1.5,y-.105+(.018 if j==1 else 0),z+.148)];widths=[.038,.034,.025];depths=[.037,.032,.024]
    else:
     points=[(x+dx*.64,y-.009,z+.028),(x+dx*.87,y-.024,z+.080),(x+dx*.96,y-.072,z+.110)];widths=[.031,.028,.021];depths=[.032,.027,.021]
    p.append(crease_normals(paddle(f'Finger / three blunt grey Graveler digits {s} {level} {j}',STONE,points,widths,depths,segments=6,steps=2),25))
  foot=translate(loft('Foot / grounded short grey Graveler support '+str(s),SHADE,[(0,.105,.14,.10),(.04,.11,.145,.09),(.095,.095,.10,.035),(.145,.07,.075,0)],10),s*.159,0,0);p.append(foot)
 # Seated small stones break the rear silhouette without duplicating a head.
 for j,(x,y,z,rx,ry,rz) in enumerate(((-.18,.77,-.15,.082,.067,.070),(.15,.80,-.16,.089,.071,.071),(-.28,.59,-.20,.081,.097,.072),(.26,.59,-.22,.080,.091,.066),(-.17,.33,-.28,.071,.081,.061),(.15,.36,-.29,.079,.075,.061))):
  rock=ellipsoid('Rubble / rooted small rear Graveler stone '+str(j),STONE,(x,y,z-.03),(rx*1.4,ry*1.4,rz*1.4),7,4);p.append(part(rock['part'],STONE,[tuple(rock['positions'][i:i+3]) for i in range(0,len(rock['positions']),3)],[tuple(rock['indices'][i:i+3]) for i in range(0,len(rock['indices']),3)],False))
 return p

def sculpture(normalize=True):
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  if normalize:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
  q['normals']=[round(v,7) for v in q['normals']]
  for key in ('positions','normals'):q[key]=[0. if v==0 else v for v in q[key]]
 return {'name':'graveler','version':1,'coordinate_system':COORDINATES,'primitives':p}
def write_canonical(output):
    from stone_pair_glb import export_stone_pair
    (output/'graveler.glb').write_bytes(export_stone_pair('graveler'))


def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);write_canonical(args.output)
if __name__=='__main__':main()
