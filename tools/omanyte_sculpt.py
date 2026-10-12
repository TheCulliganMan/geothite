#!/usr/bin/env python3
"""Original ribbed spiral shell and soft blue ten-tentacle mantle."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,part,tube,crease_normals
from bovine_sculpt import oval
from lugia_sculpt import paddle
from goldfish_sculpt import surface
from raptor_bird_sculpt import paint
from cubone_sculpt import skin_shading
from skin_glb import COORDINATES
HEIGHT=.80
CREAM=(.95,.92,.75);GROOVE=(.73,.69,.51);BLUE=(.48,.77,.86)
WHITE=(.98,.98,.96);INK=(.075,.09,.10);GLEAM=(.34,.36,.37)

def anatomy():
 # Rotate the shell's polar axis sideways, so the spiral sits on its flanks.
 vertices=[]
 for i in reversed(range(41)):
  b=math.pi*(i+.025)/(40+.05)
  for j in range(72):
   a=math.tau*j/72;r=.99+.01*math.cos(12*a)
   vertices.append((.245*math.cos(b),.49+.32*math.cos(a)*math.sin(b)*r,-.045+.275*math.sin(a)*math.sin(b)*r))
 faces=[tuple(reversed(range(72))),tuple(40*72+i for i in range(72))]
 for k in range(40):
  for i in range(72):
   a=k*72+i;b=k*72+(i+1)%72;faces.append((a,b,b+72,a+72))
 shell=part('Shell / closed ribbed cream Omanyte nautilus shell',CREAM,vertices,faces,True)
 body=ellipsoid('Body / soft blue Omanyte mantle beneath shell',BLUE,(0,.185,.150),(.230,.130,.205),56,32)
 p=[shell,body]
 # Each thin seam follows an exact meridian of the stored sculpted shell.
 for j in range(12):
  points=[vertices[k*72+j*6+3] for k in range(1,40)]
  p.append(crease_normals(tube('Shell seam / rooted thin Omanyte rib channel '+str(j),GROOVE,points,[.0025]*len(points),12),35))
 for s in (-1,1):
  rotated=dict(shell);rotated['positions']=[v for x,y,z in zip(shell['positions'][::3],shell['positions'][1::3],shell['positions'][2::3]) for v in (-s*z,y,s*x)]
  depth=surface(rotated);points=[]
  for k in range(49):
   t=k/48;a=.30+6.65*t;r=.92-.80*t
   u=s*.045+.192*r*math.cos(a);y=.49+.232*r*math.sin(a);x=depth(y,u)
   points.append((s*x,y,-s*u))
  p.append(crease_normals(tube('Shell spiral / rooted fitted Omanyte side spiral '+str(s),GROOVE,points,[.004*(1-.45*k/48) for k in range(49)],16),35))
  eye=ellipsoid('Eye / rooted round white Omanyte eye '+str(s),WHITE,(s*.100,.265,.322),(.059,.062,.035),40,28);p.append(eye)
  iris=skin_shading(paint('Iris / fitted large black round Omanyte gaze '+str(s),INK,oval(s*.100,.264,.046,.049,36),eye,.003,3),eye);p.append(iris)
  p.append(skin_shading(paint('Eye gleam / fitted soft gray Omanyte eye gleam '+str(s),GLEAM,oval(s*.092,.282,.021,.020,28),iris,.002,3),iris))
  p.append(paddle('Tentacle / rooted raised blue Omanyte side tentacle '+str(s),BLUE,[(s*.160,.170,.170),(s*.290,.220,.170),(s*.310,.320,.190),(s*.250,.420,.200)],[.064,.060,.037,.002],[.040,.035,.025,.0015]))
  for j in range(4):
   root=s*(.045+.040*j);end=s*(.070+.080*j)
   tentacle=paddle(f'Tentacle / rooted grounded blue Omanyte fan tentacle {s} {j}',BLUE,[(root,.175,.200),(s*(.060+.065*j),.100,.310-.015*j),(end,.048,.405-.025*j),(end+s*.030,.065,.455-.040*j)],[.045,.048,.027,.002],[.038,.034,.020,.0015])
   floor=min(tentacle['positions'][1::3]);tentacle['positions']=[v-floor if i%3==1 else v for i,v in enumerate(tentacle['positions'])];p.append(tentacle)
 return p

def sculpture():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'omanyte','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'omanyte.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
