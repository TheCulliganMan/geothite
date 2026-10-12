#!/usr/bin/env python3
"""Original four-armed boulder with a fitted rock face and blunt stone digits."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,translate
from chikorita_sculpt import sub,cross,dot,unit
from goldfish_sculpt import surface
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.10
STONE=(.61,.62,.59);LIGHT=(.69,.70,.65);SHADE=(.46,.48,.46)
IVORY=(.98,.97,.92);INK=(.075,.08,.075);PINK=(.85,.48,.57)

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
 base=ellipsoid('Body / original irregular grey Graveler boulder',STONE,(0,.50,0),(.40,.42,.325),24,16);vertices=[]
 for i in range(0,len(base['positions']),3):
  x,y,z=base['positions'][i:i+3];a=math.atan2(z,x);q=(y-.50)/.42;factor=1+.032*math.sin(5*a+.9)*max(0,1-q*q)+.019*math.cos(3*a-1.2)*max(0,1-q*q)
  vertices.append((x*factor,y,z*factor))
 body=part(base['part'],STONE,vertices,[tuple(base['indices'][i:i+3]) for i in range(0,len(base['indices']),3)],False);p=[body]
 brow=[(-.04,.866),(.033,.88),(.066,.824),(.111,.87),(.154,.842),(.144,.775),(.222,.809),(.247,.764),(.206,.710),(.237,.682),(.183,.652),(.095,.674),(0,.708),(-.095,.674),(-.183,.652),(-.237,.682),(-.206,.710),(-.247,.764),(-.222,.809),(-.144,.775),(-.154,.842),(-.111,.87),(-.066,.824)]
 p.append(rock_field(paint('Forehead / fitted jagged light Graveler rock field',LIGHT,brow,body,.015,3),body))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.025,.596),(.146,.641),(.170,.581),(.077,.558))]
  socket=paint('Socket / fitted narrow dark Graveler gaze '+str(s),INK,xy,body,.009,3);p.append(socket);cx=sum(x for x,y in xy)/4;cy=sum(y for x,y in xy)/4
  eye=paint('Eye / fitted narrow ivory Graveler stare '+str(s),IVORY,[(cx+.85*(x-cx),cy+.79*(y-cy)) for x,y in xy],socket,.005,3);p.append(eye)
  p.append(paint('Pupil / fitted dark Graveler slit '+str(s),INK,oval(s*.099,.594,.007,.015,20),eye,.003,2))
 smile=[(-.151,.524),(-.092,.510),(.092,.510),(.151,.524),(.105,.468),(0,.448),(-.105,.468)]
 mouth=paint('Mouth / fitted dark curved Graveler grin',INK,smile,body,.008,3);p.append(mouth)
 p.append(paint('Mouth inset / fitted pink Graveler grin',PINK,[(-.127,.509),(-.078,.501),(.078,.501),(.127,.509),(.093,.479),(0,.462),(-.093,.479)],mouth,.004,3))
 for s in (-1,1):
  for level in ('upper','lower'):
   upper=level=='upper';name=str(s)+' '+level
   if upper:
    path=[(s*.292,.70,.0),(s*.47,.735,.03),(s*.605,.623,.115)];ww=[.105,.084,.072];dd=[.107,.086,.069];center=(s*.648,.568,.162);radii=(.108,.101,.098)
   else:
    path=[(s*.28,.34,.05),(s*.40,.235,.15)];ww=[.067,.060];dd=[.071,.065];center=(s*.195,.257,.306);radii=(.093,.087,.078)
   p.append(paddle('Arm / rooted '+level+' grey Graveler arm '+str(s),STONE,path,ww,dd))
   if not upper:p.append(paddle('Forearm / rooted folded lower grey Graveler forearm '+str(s),STONE,[(s*.40,.235,.15),(s*.29,.235,.23),(s*.224,.246,.267)],[.060,.055,.050],[.065,.058,.054]))
   p.append(ellipsoid('Hand / broad '+level+' grey Graveler palm '+str(s),SHADE,center,radii,20,14))
   x,y,z=center
   for j,dx in enumerate((-.065,0,.065)):
    if upper:
     points=[(x+dx*.75,y-.015,z+.045),(x+dx*1.20,y-.022+(.028 if j==1 else 0),z+.132),(x+dx*1.5,y-.064+(.018 if j==1 else 0),z+.184)];widths=[.037,.034,.025];depths=[.040,.037,.026]
    else:
     points=[(x+dx*.64,y-.018,z+.035),(x+dx*.87,y+.023,z+.081),(x+dx*.83,y+.048,z+.109)];widths=[.030,.027,.021];depths=[.033,.028,.021]
    p.append(paddle(f'Finger / three blunt grey Graveler digits {s} {level} {j}',SHADE,points,widths,depths))
  foot=translate(loft('Foot / grounded short grey Graveler support '+str(s),SHADE,[(0,.081,.081,.051),(.048,.102,.115,.051),(.12,.104,.099,.020),(.185,.071,.072,-.01)],10),s*.159,0,0);p.append(foot)
 # Seated small stones break the rear silhouette without duplicating a head.
 for j,(x,y,z,rx,ry,rz) in enumerate(((-.18,.77,-.15,.082,.067,.070),(.15,.80,-.16,.089,.071,.071),(-.28,.59,-.20,.081,.097,.072),(.26,.59,-.22,.080,.091,.066),(-.17,.33,-.28,.071,.081,.061),(.15,.36,-.29,.079,.075,.061))):
  rock=ellipsoid('Rubble / rooted small rear Graveler stone '+str(j),LIGHT,(x,y,z),(rx,ry,rz),10,6);p.append(part(rock['part'],LIGHT,[tuple(rock['positions'][i:i+3]) for i in range(0,len(rock['positions']),3)],[tuple(rock['indices'][i:i+3]) for i in range(0,len(rock['indices']),3)],False))
 return p

def sculpture():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'graveler','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'graveler.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
