#!/usr/bin/env python3
"""Original clawed Electabuzz and egg-shaped Elekid with open plug blades."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals
from bovine_sculpt import oval,curved_tube,smooth,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from murkrow_sculpt import bill
from skin_glb import COORDINATES
SPECIES=('electabuzz','elekid');HEIGHTS={'electabuzz':1.28,'elekid':.91}
YELLOW=(.95,.80,.29);INK=(.075,.095,.13);WHITE=(.98,.98,.91);PINK=(.82,.49,.52)

def striped_tube(p,label,points,radii,bands,steps=28):
 base=curved_tube(label,YELLOW,points,radii,32);p.append(base)
 pp,rr=smooth(points,radii,steps)
 for j,(a,b) in enumerate(bands):
  start=max(0,int(a*(len(pp)-1)));end=min(len(pp)-1,int(b*(len(pp)-1)))
  p.append(curved_tube(label.replace(' /',' stripe /')+' '+str(j),INK,pp[start:end+1],[r+.003 for r in rr[start:end+1]],32))
 return base

def plug(s):
 # One closed annular extrusion: the upper contact hole goes right through
 # the yellow blade, with no dark decal or capped opening.
 cx=s*.125;cy=1.153;outline=[];radius=.020
 for x,y,start in ((cx+.050,1.265,0),(cx-.050,1.265,90),(cx-.050,.780,180),(cx+.050,.780,270)):
  for j in range(9):
   a=math.radians(start+j*90/8);outline.append((x+radius*math.cos(a),y+radius*math.sin(a)))
 n=64;outer=[];inner=[]
 for i in range(n):
  a=math.tau*i/n;dx,dy=math.cos(a),math.sin(a);hits=[]
  for u,v in zip(outline,outline[1:]+outline[:1]):
   ex,ey=v[0]-u[0],v[1]-u[1];den=dx*ey-dy*ex
   if abs(den)<1e-12:continue
   qx,qy=u[0]-cx,u[1]-cy;t=(qx*ey-qy*ex)/den;f=(qx*dy-qy*dx)/den
   if t>0 and -.00001<=f<=1.00001:hits.append(t)
  assert hits;t=min(hits);outer.append((cx+t*dx,cy+t*dy));inner.append((cx+.024*dx,cy+.035*dy))
 vv=[(x,y,z) for z in (-.019,.072) for contour in (outer,inner) for x,y in contour];ff=[]
 for i in range(n):
  j=(i+1)%n
  ff.extend(((i,j,2*n+j,2*n+i),(n+j,n+i,3*n+i,3*n+j),(j,i,n+i,n+j),(2*n+i,2*n+j,3*n+j,3*n+i)))
 return crease_normals(part('Plug / rooted yellow blade with actual contact hole '+str(s),YELLOW,vv,ff,True),35)

def gaze(p,parent,baby):
 for s in (-1,1):
  outline=((.028,.620),(.140,.660),(.128,.589),(.084,.566),(.046,.581)) if baby else ((.046,1.040),(.172,1.098),(.151,1.024),(.091,1.006))
  xy=[(s*x,y) for x,y in outline]
  socket=paint('Eye socket / fitted sharp electric recess '+str(s),INK,xy,parent,.005,3);p.append(socket);cx=cy=0.
  for x,y in xy:cx+=x;cy+=y
  cx/=len(xy);cy/=len(xy)
  eye=paint('Eye / fitted ivory angular electric gaze '+str(s),WHITE,[(cx+.88*(x-cx),cy+.85*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  p.append(paint('Pupil / fitted small dark electric stare '+str(s),INK,oval(s*(.096 if baby else .123),.602 if baby else 1.045,.005,.010,24),eye,.003,2))

def elekid():
 body=ellipsoid('Body / continuous yellow baby electric egg',YELLOW,(0,.522,.010),(.280,.350,.224),48,30);p=[body]
 for s in (-1,1):p.append(plug(s))
 gaze(p,body,True)
 p.append(paint('Mouth / fitted pointed baby electric grin',INK,[(-.079,.537),(-.052,.501),(-.033,.520),(.029,.518),(.053,.499),(.078,.538),(.070,.541),(.052,.513),(.032,.531),(-.035,.533),(-.052,.515),(-.071,.541)],body,.004,3))
 profiles=[]
 for y in (.300,.334,.368):
  r=math.sqrt(1-((y-.522)/.350)**2);profiles.append((y,.280*r+.003,.224*r+.003,.010))
 p.append(crease_normals(loft('Belly band / rooted black baby electric waist wrap',INK,curved_profiles(profiles,4),48),40))
 p.append(paint('Bolt / fitted broad baby electric belly zigzag',INK,[(-.066,.466),(.085,.444),(.034,.363),(.091,.367),(.022,.260),(-.002,.329),(-.074,.345),(-.031,.405),(-.076,.415)],body,.007,4))
 for s in (-1,1):
  p.append(paddle('Arm root / slim yellow baby electric connector '+str(s),YELLOW,[(s*.209,.636,.009),(s*.338,.653,.034),(s*.398,.640,.057)],[.042,.038,.045],[.041,.038,.045]))
  striped_tube(p,'Arm / broad yellow baby electric forearm '+str(s),[(s*.354,.650,.045),(s*.447,.644,.061),(s*.556,.631,.075)],[.073,.091,.094],[(.22,.40),(.60,.79)],steps=12)
  hand=ellipsoid('Hand / broad yellow baby electric fist '+str(s),YELLOW,(s*.591,.626,.081),(.094,.097,.102),40,26);p.append(hand)
  for j,(y,z) in enumerate(((.688,.135),(.602,.158),(.595,.018))):
   p.append(curved_tube(f'Hand claw / rooted ivory baby electric point {s} {j}',WHITE,[(s*.615,y,z),(s*.660,y+.006,z+.026),(s*.687,y+.011,z+.033)],[.022,.015,.002],28))
  p.append(paddle('Leg / rooted short yellow baby electric shin '+str(s),YELLOW,[(s*.092,.243,.001),(s*.126,.113,.033),(s*.148,.053,.072)],[.057,.044,.037],[.054,.044,.039]))
  p.append(ellipsoid('Foot / grounded yellow baby electric toe '+str(s),YELLOW,(s*.151,.037,.084),(.061,.037,.086),36,24))
 return p

def electabuzz():
 body=ellipsoid('Body / substantial yellow adult electric pear',YELLOW,(0,.640,-.014),(.260,.337,.205),48,30)
 neck=ellipsoid('Neck / rooted yellow adult electric support',YELLOW,(0,.948,.014),(.121,.119,.109),36,24)
 head=ellipsoid('Head / broad low yellow adult electric skull',YELLOW,(0,1.050,.033),(.280,.159,.190),48,30);p=[body,neck,head]
 gaze(p,head,False)
 p.append(reverse_front(paint('Back stripe / fitted thick adult electric dorsal sweep',INK,[(-.077,.449),(-.133,.507),(-.103,.681),(-.057,.804),(.026,.923),(.107,.933),(.044,.819),(.009,.694),(-.031,.545)],reverse_front(body),.006,5)))
 p.append(paint('Forehead / fitted sharp black adult electric chevron',INK,[(0,1.054),(-.125,1.159),(-.068,1.127),(.023,1.190),(.011,1.138),(.102,1.173),(.039,1.092)],head,.006,4))
 p.append(paint('Bolt / fitted broad adult electric chest lightning',INK,[(-.083,.887),(.106,.861),(.035,.712),(.148,.710),(-.083,.422),(-.044,.633),(-.204,.615),(-.166,.741),(-.108,.729)],body,.007,4))
 for s in (-1,1):
  p.append(paint('Flank stripe / fitted lower adult electric sweep '+str(s),INK,[(s*x,y) for x,y in ((.054,.523),(.237,.651),(.226,.582),(.091,.475),(.132,.509))],body,.005,3))
  p.append(paddle('Crest / rooted pointed yellow adult electric scalp '+str(s),YELLOW,[(s*.017,1.178,.012),(s*.071,1.279,.023),(s*.119,1.389,.031)],[.028,.024,.002],[.029,.018,.002]))
  p.append(curved_tube('Antenna / rooted yellow adult electric stalk '+str(s),YELLOW,[(s*.162,1.145,-.023),(s*.235,1.237,-.027)],[.017,.015],28))
  p.append(ellipsoid('Antenna ball / rooted gold adult electric terminal '+str(s),YELLOW,(s*.241,1.247,-.028),(.031,.035,.032),32,22))
 upper=bill('Jaw / rooted yellow adult electric upper muzzle',YELLOW,[(.157,.169,.986,1.034),(.241,.130,.972,1.012),(.292,.071,.965,.990)])
 lower=bill('Jaw / rooted open yellow adult electric lower muzzle',YELLOW,[(.139,.133,.913,.956),(.221,.112,.861,.900),(.279,.083,.897,.928)])
 p.extend((upper,lower,ellipsoid('Mouth back / rooted dark adult electric oral chamber',INK,(0,.946,.193),(.115,.071,.043),40,26),ellipsoid('Tongue / rooted pink adult electric mouth floor',PINK,(0,.903,.241),(.075,.016,.043),36,24)))
 for s in (-1,1):
  p.append(curved_tube('Fang / rooted ivory adult electric upper tooth '+str(s),WHITE,[(s*.087,.991,.241),(s*.087,.967,.258),(s*.085,.942,.267)],[.020,.016,.002],28))
  p.append(paint('Nostril / fitted small adult electric muzzle slit '+str(s),INK,oval(s*.049,1.006,.010,.005,24),upper,.003,3))
  if s==-1:points=[(s*.144,.861,.008),(s*.285,.946,.015),(s*.349,1.156,.037),(s*.428,1.250,.072)];center=(s*.476,1.279,.114)
  else:points=[(s*.144,.852,.008),(s*.306,.918,.025),(s*.443,.899,.133),(s*.515,.871,.202)];center=(s*.572,.876,.267)
  striped_tube(p,'Arm / rooted bent yellow adult electric forelimb '+str(s),points,[.075,.081,.085,.063],[(.15,.25),(.39,.51),(.67,.79)])
  fist=ellipsoid('Hand / broad clenched yellow adult electric fist '+str(s),YELLOW,center,(.112,.096,.091),40,26);p.append(fist)
  furpaths=[[(s*.406,1.215,.042),(s*.433,1.170,.044)],[(s*.470,1.235,.079),(s*.490,1.190,.096)]] if s==-1 else [[(.490,.894,.180),(.468,.965,.198)],[(.544,.921,.208),(.513,.984,.218)]]
  for j,path in enumerate(furpaths):p.append(curved_tube(f'Wrist fur / rooted pointed yellow adult electric tuft {s} {j}',YELLOW,path,[.029,.002],28))
  for j in range(3):
   x=center[0]+(j-1)*.048
   p.append(ellipsoid(f'Knuckle / rooted yellow adult electric finger {s} {j}',YELLOW,(x,center[1]+.012,center[2]+.060),(.030,.055,.043),32,22))
  p.append(ellipsoid('Thumb / rooted yellow adult electric fist fold '+str(s),YELLOW,(center[0]-s*.076,center[1]-.033,center[2]+.025),(.041,.047,.042),32,22))
  striped_tube(p,'Leg / rooted bent yellow adult electric shin '+str(s),[(s*.119,.425,-.012),(s*.184,.235,.046),(s*.198,.076,.092)],[.078,.061,.048],[(.12,.27),(.64,.81)])
  p.append(ellipsoid('Foot / grounded broad yellow adult electric support '+str(s),YELLOW,(s*.199,.048,.147),(.106,.048,.146),40,26))
  for j in range(2):
   x=s*(.167+.058*j);p.append(curved_tube(f'Toe claw / rooted ivory adult electric point {s} {j}',WHITE,[(x,.053,.246),(x,.036,.301),(x,.024,.338)],[.026,.018,.002],28))
  p.append(curved_tube(f'Toe claw / rooted ivory adult electric point {s} 2',WHITE,[(s*.199,.044,.036),(s*.199,.028,-.046),(s*.199,.017,-.078)],[.021,.015,.002],28))
 striped_tube(p,'Tail / rooted long curved yellow adult electric balance',[(0,.401,-.126),(.053,.314,-.334),(.260,.375,-.528),(.379,.568,-.516)],[.050,.049,.041,.026],[(.13,.23),(.32,.42),(.52,.62),(.73,.83)])
 p.append(ellipsoid('Tail tip / rooted rounded yellow adult electric end',YELLOW,(.379,.568,-.516),(.027,.032,.027),32,22))
 return p

def sculpture(name,normalize=True):
 if name not in SPECIES:raise ValueError(name)
 p=electabuzz() if name=='electabuzz' else elekid();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[name]/(hi-lo)
 if name=='elekid':
  from sculpt_geometry import linear
  for q in p:
   n=q['part'];rgb=(0,0,0) if n.startswith(('Eye socket /','Pupil /','Mouth /','Belly band /','Bolt /','Arm stripe /')) else (255,255,255) if n.startswith(('Eye /','Hand claw /')) else (255,255,41)
   q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':name,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for n in SPECIES:(args.output/(n+'.mesh.json')).write_text(json.dumps(sculpture(n),separators=(',',':')))
if __name__=='__main__':main()
