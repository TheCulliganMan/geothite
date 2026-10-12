#!/usr/bin/env python3
"""Original boxing fighter with a continuous crown head and pleated lavender kit."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals,translate
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import sheet,paint
from skin_glb import COORDINATES
HEIGHT=1.30
TAN=(.65,.58,.46);LAVENDER=(.68,.65,.80);HEM=(.61,.58,.72)
RED=(.83,.31,.29);CUFF=(.70,.23,.23);INK=(.075,.075,.10)
WHITE=(.98,.98,.96);BLUE=(.26,.44,.54)

def crown_head():
 xy=[(0,1.058),(-.142,1.093),(-.227,1.192),(-.240,1.345),(-.235,1.538),(-.184,1.598),(-.151,1.495),(-.113,1.686),(-.054,1.705),(-.049,1.515),(.054,1.750),(.109,1.730),(.079,1.510),(.183,1.639),(.228,1.598),(.156,1.454),(.267,1.495),(.283,1.439),(.223,1.291),(.199,1.173),(.108,1.095)]
 def depth(y,x):return .115-.070*(y-1.25)
 cage=sheet('Head / continuous angular tan five-point crown',TAN,xy,depth,.120,4)
 vv=[]
 for i in range(0,len(cage['positions']),3):
  x,y,z=cage['positions'][i:i+3];d=depth(y,x);t=max(0,min(1,(y-1.32)/.32));factor=1-.78*t*t*(3-2*t);vv.append((x,y,d+(z-d)*factor))
 head=crease_normals(part(cage['part'],TAN,vv,[tuple(cage['indices'][i:i+3]) for i in range(0,len(cage['indices']),3)],True),40)
 # Skinny ear-clipped cap triangles must share the same smooth surface normal.
 # Keep the original perimeter normals so the crown edges remain crisp.
 out=dict(head,positions=[],normals=[],indices=[]);lookup={}
 for k in range(0,len(head['indices']),3):
  ids=head['indices'][k:k+3];points=[head['positions'][3*j:3*j+3] for j in ids]
  offsets=[z-depth(y,x) for x,y,z in points];front=all(q>0 for q in offsets);back=all(q<0 for q in offsets)
  for j,(x,y,z) in zip(ids,points):
   normal=head['normals'][3*j:3*j+3]
   if front or back:
    t=max(0,min(1,(y-1.32)/.32));slope=-.78*6*t*(1-t)/.32
    dy=-.070+(.120 if front else -.042)*slope
    normal=(0,-dy,1) if front else (0,dy,-1);length=math.sqrt(sum(q*q for q in normal));normal=tuple(q/length for q in normal)
   key=(x,y,z,*normal)
   if key not in lookup:lookup[key]=len(out['positions'])//3;out['positions'].extend((x,y,z));out['normals'].extend(normal)
   out['indices'].append(lookup[key])
 return out

def skirt():
 rows=[(.490,.305,.219),(.552,.269,.198),(.626,.205,.159),(.711,.168,.136)];n=96;vv=[]
 for y,w,d in rows:
  fade=(.711-y)/(.711-.490)
  for i in range(n):
   a=math.tau*i/n;fold=1+.072*fade*math.cos(8*a);vv.append((w*math.sin(a)*fold,y,.015+d*math.cos(a)*fold))
 ff=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
 for j in range(len(rows)-1):
  for i in range(n):a=j*n+i;b=j*n+(i+1)%n;ff.append((a,b,b+n,a+n))
 return crease_normals(part('Skirt / continuous flared eight-pleat lavender hem',LAVENDER,vv,ff,True),35)

def limbs(p):
 for s in (-1,1):
  leg=paddle('Leg / rooted long tan boxing support '+str(s),TAN,[(s*.137,.563,-.018),(s*.171,.319,.043),(s*.177,.069,.111)],[.079,.060,.056],[.084,.067,.037]);p.append(leg)
  boot=ellipsoid('Boot / grounded rounded lavender boxing shoe '+str(s),LAVENDER,(s*.180,.043,.167),(.093,.043,.128),36,24);p.append(boot)
  cuff=translate(crease_normals(loft('Boot cuff / rooted lavender ankle collar '+str(s),HEM,curved_profiles(((.072,.065,.072,.111),(.115,.065,.069,.104),(.153,.052,.055,.096)),3),32),40),s*.177,0,0);p.append(cuff)
  xy=[(s*(.180+dx),y) for dx,y in ((-.075,.043),(-.045,.066),(0,.073),(.045,.066),(.075,.043),(.055,.045),(.032,.060),(0,.064),(-.032,.060),(-.055,.045))]
  p.append(paint('Boot seam / fitted curved lavender toe piping '+str(s),HEM,xy,boot,.003,3))
  shoulder=sheet('Shoulder guard / rooted pointed tan boxing armor '+str(s),TAN,[(s*x,y) for x,y in ((.105,1.090),(.196,1.207),(.291,1.176),(.346,1.078),(.204,.948),(.148,1.000))],lambda y,x:.014,.060,3);p.append(shoulder)
  if s==-1:
   path=[(s*.210,1.069,.023),(s*.374,1.033,.157),(s*.517,1.110,.335)];widths=[.082,.069,.055];depths=[.085,.071,.054];center=(s*.560,1.170,.432);cuffpath=[(s*.505,1.099,.320),(s*.541,1.144,.389)]
  else:
   path=[(s*.209,1.063,.028),(s*.345,.910,.101),(s*.396,.956,.244),(s*.321,1.149,.388)];widths=[.082,.066,.060,.050];depths=[.085,.070,.063,.053];center=(s*.300,1.226,.456);cuffpath=[(s*.328,1.130,.376),(s*.310,1.186,.423)]
  p.append(paddle('Arm / rooted bent tan boxing forelimb '+str(s),TAN,path,widths,depths))
  p.append(curved_tube('Glove cuff / rooted dark red wrist collar '+str(s),CUFF,cuffpath,[.064,.068],32))
  glove=ellipsoid('Glove / rounded red padded boxing mitt '+str(s),RED,center,(.111,.116,.129),40,26);p.append(glove)
  thumb=ellipsoid('Glove thumb / rooted red inner thumb '+str(s),RED,(center[0]-s*.070,center[1]-.045,center[2]+.026),(.052,.060,.069),32,22);p.append(thumb)
  cx,cy,cz=center
  xy=[(cx+x,cy+y) for x,y in ((-.084,-.028),(-.039,-.077),(.039,-.077),(.084,-.028),(.076,-.038),(.034,-.085),(-.034,-.085),(-.076,-.038))]
  p.append(paint('Glove seam / fitted dark lower mitt contour '+str(s),CUFF,xy,glove,.003,3))

def sculpture():
 torso=crease_normals(loft('Tunic / continuous sleeveless lavender boxing torso',LAVENDER,curved_profiles(((.652,.128,.103,.024),(.728,.185,.147,.015),(.877,.239,.177,.009),(1.027,.222,.147,.017),(1.108,.098,.080,.039)),5),48),45)
 neck=ellipsoid('Neck / rooted short tan boxing neck',TAN,(0,1.116,.077),(.088,.110,.080),36,24)
 head=crown_head();p=[torso,neck,head,skirt()]
 p.append(crease_normals(loft('Belt / rooted narrow charcoal boxing waist',INK,curved_profiles(((.680,.166,.139,.019),(.703,.181,.147,.018),(.731,.184,.146,.015)),3),48),40))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.062,1.293),(.189,1.319),(.183,1.240),(.130,1.214),(.093,1.222))]
  socket=paint('Eye socket / fitted angular dark boxing recess '+str(s),INK,xy,head,.006,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted ivory boxing gaze '+str(s),WHITE,[(cx+.88*(x-cx),cy+.87*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted blue boxing stare '+str(s),BLUE,oval(s*.145,1.268,.016,.026,24),eye,.003,2);p.append(iris)
  pupil=paint('Pupil / fitted dark boxing slit '+str(s),INK,oval(s*.145,1.266,.007,.021,20),iris,.003,2);p.append(pupil)
  p.append(paint('Glint / tiny ivory boxing eye point '+str(s),WHITE,oval(s*.145-.002,1.276,.002,.004,16),pupil,.002,2))
 p.append(paint('Mouth / fitted firm boxing jaw crease',INK,[(-.089,1.152),(-.033,1.136),(.031,1.129),(.090,1.139),(.085,1.132),(.029,1.121),(-.034,1.129),(-.084,1.145)],head,.004,3))
 limbs(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'hitmonchan','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'hitmonchan.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
