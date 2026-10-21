#!/usr/bin/env python3
"""Original clawed Electabuzz and egg-shaped Elekid with open plug blades."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals,fitted_adaptive_contour
from cubone_sculpt import skin_shading
from goldfish_sculpt import surface
from bovine_sculpt import oval,curved_tube,smooth,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from murkrow_sculpt import bill
from skin_glb import COORDINATES
SPECIES=('electabuzz','elekid');HEIGHTS={'electabuzz':1.28,'elekid':.91}
YELLOW=(.95,.80,.29);INK=(.075,.095,.13);WHITE=(.98,.98,.91);PINK=(.82,.49,.52)
PLUG_HOLE_Y=1.105

def baby_arm(s):
 # Crystal's alternating fists: a bent raised arm and a low forward arm.
 if s==-1:
  return [(s*.245,.630,.010),(s*.353,.632,.020),(s*.403,.697,.035)],[(s*.371,.666,.026),(s*.419,.749,.045),(s*.432,.819,.063)],(s*.434,.853,.074)
 return [(s*.245,.615,.010),(s*.345,.585,.024),(s*.390,.523,.047)],[(s*.365,.562,.033),(s*.415,.491,.065),(s*.428,.434,.097)],(s*.431,.403,.112)

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
 cx=s*.125;cy=PLUG_HOLE_Y;outline=[];radius=.020
 for x,y,start in ((cx+.050,1.190,0),(cx-.050,1.190,90),(cx-.050,.780,180),(cx+.050,.780,270)):
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

def gaze(p,parent,baby,paint_at=paint):
 paint=paint_at
 for s in (-1,1):
  outline=((.030,.647),(.155,.670),(.139,.621),(.088,.612),(.046,.625)) if baby else ((.046,1.040),(.172,1.098),(.151,1.024),(.091,1.006))
  xy=[(s*x,y) for x,y in outline]
  socket=paint('Eye socket / fitted sharp electric recess '+str(s),INK,xy,parent,.005,3);p.append(socket);cx=cy=0.
  for x,y in xy:cx+=x;cy+=y
  cx/=len(xy);cy/=len(xy)
  eye=paint('Eye / fitted ivory angular electric gaze '+str(s),WHITE,[(cx+.88*(x-cx),cy+.85*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  p.append(paint('Pupil / fitted small dark electric stare '+str(s),INK,oval(s*(.103 if baby else .123),.638 if baby else 1.045,.005,.009,24),eye,.003,2))

def elekid():
 paint=adult_paint
 body=ellipsoid('Body / continuous yellow baby electric egg',YELLOW,(0,.522,.010),(.330,.340,.265),40,28);p=[body]
 for s in (-1,1):p.append(plug(s))
 gaze(p,body,True,paint_at=paint)
 p.append(paint('Mouth / fitted pointed baby electric grin',INK,[(-.079,.537),(-.052,.501),(-.033,.520),(.029,.518),(.053,.499),(.078,.538),(.070,.541),(.052,.513),(.032,.531),(-.035,.533),(-.052,.515),(-.071,.541)],body,.004,3))
 for j,(a,b) in enumerate(((.303,.338),(.401,.436))):
  profiles=[]
  for y in (a,(a+b)/2,b):
   r=math.sqrt(1-((y-.522)/.340)**2);profiles.append((y,.330*r+.003,.265*r+.003,.010))
  p.append(crease_normals(loft('Belly band / rooted black baby electric waist wrap '+str(j),INK,curved_profiles(profiles,3),40),40))
 p.append(paint('Bolt / fitted broad baby electric belly zigzag',INK,[(-.066,.466),(.085,.444),(.034,.363),(.091,.367),(.022,.260),(-.002,.329),(-.074,.345),(-.031,.405),(-.076,.415)],body,.007,4))
 for s in (-1,1):
  root,fore,center=baby_arm(s)
  p.append(paddle('Arm root / slim yellow baby electric connector '+str(s),YELLOW,root,[.046,.039,.048],[.044,.040,.048],segments=24,steps=4))
  striped_tube(p,'Arm / broad yellow baby electric forearm '+str(s),fore,[.070,.096,.098],[(.18,.38),(.57,.77)],steps=12)
  hand=ellipsoid('Hand / broad yellow baby electric fist '+str(s),YELLOW,center,(.099,.100,.101),28,18);p.append(hand)
  tangent=[center[k]-fore[-2][k] for k in range(3)];length=math.sqrt(sum(v*v for v in tangent));d=[v/length for v in tangent];u=(-d[1],d[0],0);ul=math.hypot(u[0],u[1]);u=[v/ul for v in u]
  for j,a in enumerate((0.,math.tau/3,2*math.tau/3)):
   start=[center[k]+.067*d[k]+.042*math.cos(a)*u[k]+(.042*math.sin(a) if k==2 else 0) for k in range(3)]
   tip=[start[k]+.045*d[k] for k in range(3)]
   p.append(curved_tube(f'Hand claw / rooted ivory baby electric point {s} {j}',WHITE,[start,tip],[.017,.002],20))
  p.append(paddle('Leg / rooted short dark baby electric shin '+str(s),INK,[(s*.115,.250,.015),(s*.143,.142,.015),(s*.151,.057,.015)],[.054,.045,.043],[.052,.047,.045],segments=24,steps=4))
  p.append(ellipsoid('Foot / grounded yellow baby electric toe '+str(s),YELLOW,(s*.153,.043,.051),(.083,.043,.108),28,18))
 return adult_shading([crease_normals(q,45.) for q in p])

def adult_paint(name,color,xy,parent,th=.006,sub=3):
 return fitted_adaptive_contour(name,color,xy,None,th,surface_at=surface(parent),initial_subdivisions=1)
def adult_round(name,color,center,radii,segments=28,rings=18):
 if name.startswith(('Knuckle /','Thumb /','Antenna ball /','Tail tip /','Tongue /')):cap=(20,12)
 elif name.startswith(('Neck /','Mouth back /','Hand /','Foot /')):cap=(24,16)
 else:cap=(28,18)
 return ellipsoid(name,color,center,radii,min(segments,cap[0]),min(rings,cap[1]))
def adult_tube(name,color,points,radii,segments=24):return curved_tube(name,color,points,radii,min(segments,24))
def adult_striped_tube(p,label,points,radii,bands,steps=12):
 base=adult_tube(label,YELLOW,points,radii);p.append(base)
 pp,rr=smooth(points,radii,steps)
 for j,(a,b) in enumerate(bands):
  start=max(0,int(a*(len(pp)-1)));end=min(len(pp)-1,int(b*(len(pp)-1)))
  p.append(adult_tube(label.replace(' /',' stripe /')+' '+str(j),INK,pp[start:end+1],[r+.003 for r in rr[start:end+1]],20))
 return base
def adult_shading(parts):
 def find(prefix,side=None):return next(p for p in parts if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+side)))
 for i,p in enumerate(parts):
  n=p['part'];side=n.split()[-1];parent=None;back=False
  for child,owner in (('Eye socket /','Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /'),('Back stripe /','Body /'),('Forehead /','Head /'),('Bolt /','Body /'),('Flank stripe /','Body /'),('Nostril /','Jaw / rooted yellow')):
   if n.startswith(child):parent=find('Body /' if owner=='Head /' and not any(q['part'].startswith('Head /') for q in parts) else owner,side if owner in ('Eye socket /','Eye /') else None);back=child=='Back stripe /'
  if n.startswith('Mouth /'):parent=find('Body /')
  if parent:parts[i]=reverse_front(skin_shading(reverse_front(p),reverse_front(parent))) if back else skin_shading(p,parent)
 return parts

ADULT_TAIL_POINTS=((0,.401,-.126),(.053,.314,-.334),(.260,.375,-.528),(.379,.568,-.516))
def adult_arm(s):
 if s==-1:return [(s*.211,.852,.003),(s*.397,.825,.027),(s*.472,.961,.076),(s*.483,1.066,.111)],(s*.488,1.115,.146)
 return [(s*.211,.842,.003),(s*.398,.791,.035),(s*.475,.921,.111),(s*.490,1.016,.166)],(s*.493,1.065,.205)

def electabuzz():
 ellipsoid=adult_round;curved_tube=adult_tube;striped_tube=adult_striped_tube;paint=adult_paint
 body=ellipsoid('Body / substantial yellow adult electric pear',YELLOW,(0,.620,-.014),(.310,.337,.250),48,30)
 neck=ellipsoid('Neck / rooted yellow adult electric support',YELLOW,(0,.948,.014),(.121,.119,.109),36,24)
 head=crease_normals(loft('Head / broad low yellow adult electric skull',YELLOW,curved_profiles(((.933,.173,.117,.025),(.979,.294,.188,.025),(1.042,.251,.190,.025),(1.126,.179,.148,.021),(1.185,.080,.071,.017),(1.200,.018,.022,.017)),3),40),45);p=[body,neck,head]
 gaze(p,head,False,paint_at=paint)
 p.append(reverse_front(paint('Back stripe / fitted thick adult electric dorsal sweep',INK,[(-.077,.449),(-.133,.507),(-.103,.681),(-.057,.804),(.026,.923),(.107,.933),(.044,.819),(.009,.694),(-.031,.545)],reverse_front(body),.006,5)))
 p.append(paint('Forehead / fitted sharp black adult electric chevron',INK,[(0,1.054),(-.125,1.159),(-.068,1.127),(.023,1.190),(.011,1.138),(.102,1.173),(.039,1.092)],head,.006,4))
 p.append(paint('Bolt / fitted broad adult electric chest lightning',INK,[(-.083,.887),(.106,.861),(.035,.712),(.148,.710),(-.083,.422),(-.044,.633),(-.204,.615),(-.166,.741),(-.108,.729)],body,.007,4))
 for s in (-1,1):
  p.append(paint('Flank stripe / fitted lower adult electric sweep '+str(s),INK,[(s*x,y) for x,y in ((.054,.523),(.237,.651),(.226,.582),(.091,.475),(.132,.509))],body,.005,3))
  p.append(paddle('Crest / rooted pointed yellow adult electric scalp '+str(s),YELLOW,[(s*.025,1.175,.012),(s*.042,1.276 if s==-1 else 1.246,.013),(s*.050,1.398 if s==-1 else 1.335,.009)],[.025,.016,.0015],[.022,.013,.0015],segments=20,steps=4))
  p.append(curved_tube('Antenna / rooted yellow adult electric stalk '+str(s),YELLOW,[(s*.141,1.125,-.023),(s*.235,1.237,-.027)],[.017,.015],28))
  p.append(ellipsoid('Antenna ball / rooted gold adult electric terminal '+str(s),YELLOW,(s*.241,1.247,-.028),(.031,.035,.032),32,22))
 upper=bill('Jaw / rooted yellow adult electric upper muzzle',YELLOW,[(.140,.212,.986,1.034),(.241,.181,.972,1.012),(.311,.101,.965,.990)])
 lower=bill('Jaw / rooted open yellow adult electric lower muzzle',YELLOW,[(.120,.180,.913,.956),(.221,.154,.861,.900),(.292,.112,.897,.928)])
 p.extend((upper,lower,ellipsoid('Mouth back / rooted dark adult electric oral chamber',INK,(0,.946,.185),(.161,.071,.043),40,26),ellipsoid('Tongue / rooted pink adult electric mouth floor',PINK,(0,.903,.254),(.093,.016,.043),36,24)))
 for s in (-1,1):
  p.append(curved_tube('Fang / rooted ivory adult electric upper tooth '+str(s),WHITE,[(s*.087,.991,.241),(s*.087,.967,.258),(s*.085,.942,.267)],[.020,.016,.002],28))
  p.append(paint('Nostril / fitted small adult electric muzzle slit '+str(s),INK,oval(s*.049,1.006,.010,.005,24),upper,.003,3))
  points,center=adult_arm(s)
  striped_tube(p,'Arm / rooted bent yellow adult electric forelimb '+str(s),points,[.070,.086,.106,.087],[(.12,.22),(.41,.53),(.69,.81)])
  fist=ellipsoid('Hand / broad clenched yellow adult electric fist '+str(s),YELLOW,center,(.121,.103,.106),40,26);p.append(fist)
  wrist=points[-1];furpaths=[[(wrist[0]+s*.067,wrist[1]-.030,wrist[2]),(wrist[0]+s*.112,wrist[1]-.105,wrist[2]-.012)],[(wrist[0]-s*.055,wrist[1]-.036,wrist[2]-.027),(wrist[0]-s*.081,wrist[1]-.100,wrist[2]-.037)]]
  for j,path in enumerate(furpaths):p.append(curved_tube(f'Wrist fur / rooted pointed yellow adult electric tuft {s} {j}',YELLOW,path,[.029,.002],28))
  for j in range(3):
   x=center[0]+(j-1)*.048
   p.append(ellipsoid(f'Knuckle / rooted yellow adult electric finger {s} {j}',YELLOW,(x,center[1]+.012,center[2]+.060),(.030,.055,.043),32,22))
  p.append(ellipsoid('Thumb / rooted yellow adult electric fist fold '+str(s),YELLOW,(center[0]-s*.076,center[1]-.033,center[2]+.025),(.041,.047,.042),32,22))
  striped_tube(p,'Leg / rooted upright yellow adult electric shin '+str(s),[(s*.145,.401,.006),(s*.201,.236,.006),(s*.213,.081,.006)],[.082,.064,.054],[(.15,.30),(.65,.83)])
  p.append(ellipsoid('Foot / grounded broad yellow adult electric support '+str(s),YELLOW,(s*.214,.053,.079),(.108,.053,.147),40,26))
  for j in range(2):
   x=s*(.182+.062*j);p.append(curved_tube(f'Toe claw / rooted ivory adult electric point {s} {j}',WHITE,[(x,.052,.178),(x,.032,.232),(x,.020,.269)],[.026,.018,.002],24))
  p.append(curved_tube(f'Toe claw / rooted ivory adult electric point {s} 2',WHITE,[(s*.214,.044,-.029),(s*.214,.028,-.098),(s*.214,.017,-.130)],[.021,.015,.002],24))
 striped_tube(p,'Tail / rooted long curved yellow adult electric balance',ADULT_TAIL_POINTS,[.050,.049,.041,.026],[(.13,.23),(.32,.42),(.52,.62),(.73,.83)])
 p.append(ellipsoid('Tail tip / rooted rounded yellow adult electric end',YELLOW,(.379,.568,-.516),(.027,.032,.027),32,22))
 return adult_shading([crease_normals(q,45.) for q in p])

def sculpture(name,normalize=True):
 if name not in SPECIES:raise ValueError(name)
 p=electabuzz() if name=='electabuzz' else elekid();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[name]/(hi-lo)
 if name=='elekid':
  from sculpt_geometry import linear
  for q in p:
   n=q['part'];rgb=(0,0,0) if n.startswith(('Eye socket /','Pupil /','Mouth /','Belly band /','Bolt /','Arm stripe /','Leg /')) else (255,255,255) if n.startswith(('Eye /','Hand claw /')) else (255,255,41)
   q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 if name=='electabuzz':
  for q in p:q['positions']=[v or 0. for v in q['positions']]
 return {'name':name,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for n in SPECIES:(args.output/(n+'.mesh.json')).write_text(json.dumps(sculpture(n),separators=(',',':')))
if __name__=='__main__':main()
