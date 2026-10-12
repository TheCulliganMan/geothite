#!/usr/bin/env python3
"""Original flame bird with gold leading wings and closed flowing fire contours."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals
from raptor_bird_sculpt import sheet,paint,feet
from murkrow_sculpt import bill
from bovine_sculpt import oval,reverse_front
from wing_insect_sculpt import contour
from chikorita_sculpt import linear
from skin_glb import COORDINATES
SPECIES=('moltres',);HEIGHT=1.28
GOLD=(.97,.79,.32);LIGHT=(1.,.85,.43);FIRE=(.97,.42,.21)
ORANGE=(1.,.64,.28);HOT=(1.,.80,.34);GREY=(.53,.48,.44);INK=(.09,.08,.10);IVORY=(.98,.96,.87)

def flowing(points,steps=5):
 curves=[]
 for i,p in enumerate(points):
  before=points[i-1];after=points[(i+1)%len(points)]
  a=tuple((x+y)/2 for x,y in zip(before,p));b=tuple((x+y)/2 for x,y in zip(p,after));curves.append((a,p,p,b))
 return contour(curves,steps)

def inner_fire(points):
 # Sample real vertical sections of the authored outline. The warm core is
 # inset from both boundaries instead of shrinking a concave polygon blindly.
 low=min(x for x,y in points);high=max(x for x,y in points);bottom=[];top=[]
 for i in range(1,64):
  x=low+(high-low)*i/64;hits=[]
  for a,b in zip(points,points[1:]+points[:1]):
   if (a[0]<=x<b[0]) or (b[0]<=x<a[0]):hits.append(a[1]+(b[1]-a[1])*(x-a[0])/(b[0]-a[0]))
  hits.sort();ranges=list(zip(hits[::2],hits[1::2]))
  if not ranges:raise ValueError('flame core has no outline section')
  a,b=max(ranges,key=lambda q:q[1]-q[0]);mid=(a+b)/2;half=(b-a)*.34
  bottom.append((x,mid-half));top.append((x,mid+half))
 return bottom+list(reversed(top))

def both_faces(p,parent,xy,color,name,th=.008,sub=3):
 for face in (-1,1):
  actual=parent if face==1 else reverse_front(parent)
  q=paint(name+' '+str(face),color,xy if face==1 else [(-x,y) for x,y in xy],actual,th,sub)
  p.append(q if face==1 else reverse_front(q))

def wings(p):
 depth=lambda y,x:-.034-.12*abs(x)+.035*math.sin(y*2.5)
 for s in (-1,1):
  armxy=contour([((.098,.604),(.27,.731),(.54,.902),(1.02,1.147)),
                 ((1.02,1.147),(.90,.967),(.58,.826),(.30,.657)),
                 ((.30,.657),(.21,.587),(.15,.566),(.098,.604))],12)
  arm=sheet('Wing / slim gold leading arm '+str(s),GOLD,[(s*x,y) for x,y in armxy],depth,.062,2);p.append(arm)
  xy=flowing([(.12,.625),(.20,.83),(.26,.91),(.28,1.045),(.37,.918),(.43,1.17),(.48,1.12),(.51,1.31),(.60,1.16),(.66,1.25),(.69,1.11),(.83,1.36),(.84,1.21),(1.02,1.41),(1.04,1.235),(1.16,1.36),(1.10,1.03),(.91,.867),(.89,.737),(.73,.78),(.73,.605),(.63,.667),(.53,.50),(.50,.619),(.36,.458),(.34,.563),(.20,.536)],6)
  fire=sheet('Wing fire / flowing red-orange flame bank '+str(s),FIRE,[(s*x,y) for x,y in xy],lambda y,x:depth(y,x)-.030+.020*math.sin(5*abs(x))*math.sin(4*(y-.5)),.035,1);p.append(fire)
  # A broad orange core follows the real outer bank, on each visible side.
  corexy=[(s*x,y) for x,y in inner_fire(xy)];cores={}
  for face in (-1,1):
   actual=fire if face==1 else reverse_front(fire)
   q=paint(f'Wing glow / fitted broad orange fire core {s} {face}',ORANGE,corexy if face==1 else [(-x,y) for x,y in corexy],actual,.010,2)
   cores[face]=q if face==1 else reverse_front(q);p.append(cores[face])
  for j,pocket in enumerate(([(.28,.70),(.32,.79),(.35,.81),(.34,.72)],[(.50,.86),(.52,.98),(.55,1.035),(.55,.91)],[(.75,1.00),(.79,1.105),(.81,1.08),(.80,.99)])):
   xyhot=[(s*x,y) for x,y in flowing(pocket,5)]
   for face in (-1,1):
    actual=cores[face] if face==1 else reverse_front(cores[face]);q=paint(f'Wing heart / fitted hot flame pocket {s} {j} {face}',HOT,xyhot if face==1 else [(-x,y) for x,y in xyhot],actual,.008,2);p.append(q if face==1 else reverse_front(q))

def crown(p,head):
 xy=flowing([(-.086,.984),(-.111,1.095),(-.067,1.17),(-.075,1.28),(-.013,1.23),(.018,1.41),(.067,1.32),(.089,1.35),(.097,1.20),(.145,1.26),(.113,1.11),(.069,.990)],6)
 fire=sheet('Crown / flowing three-lick head flame',FIRE,xy,lambda y,x:.057-.20*(y-.99),.046,2);p.append(fire)
 xy=flowing([(-.054,1.04),(-.066,1.12),(-.020,1.19),(.027,1.29),(.041,1.245),(.043,1.16),(.078,1.18),(.056,1.055)],6)
 both_faces(p,fire,xy,ORANGE,'Crown glow / supported orange flame heart',.008,2)
 xy=flowing([(-.028,1.063),(-.026,1.125),(.005,1.18),(.025,1.23),(.033,1.12),(.036,1.068)],6)
 both_faces(p,fire,xy,HOT,'Crown heart / supported gold flame core',.019,2)

def tail(p):
 depth=lambda y,x:-.14-1.90*(.385-y)
 xy=flowing([(-.045,.394),(-.128,.351),(-.20,.26),(-.186,.169),(-.281,.118),(-.20,.090),(-.107,.119),(-.112,.040),(-.022,.079),(.049,.034),(.101,.070),(.115,.103),(.25,.045),(.25,.13),(.167,.258),(.075,.395)],6)
 fire=sheet('Tail / wide trailing closed flame plume',FIRE,xy,depth,.042,2);p.append(fire)
 xy=flowing([(-.024,.365),(-.10,.275),(-.134,.210),(-.072,.23),(-.066,.135),(.012,.191),(.059,.090),(.085,.183),(.143,.106),(.12,.26),(.032,.365)],6)
 both_faces(p,fire,xy,ORANGE,'Tail glow / supported flowing orange flame',.008,2)
 xy=flowing([(-.025,.322),(-.057,.264),(-.036,.251),(-.02,.223),(.022,.265),(.056,.208),(.067,.267),(.018,.329)],6)
 both_faces(p,fire,xy,HOT,'Tail heart / supported small gold flame',.019,2)

def sculpture(species='moltres'):
 if species not in SPECIES:raise ValueError('unsupported flame bird')
 body=crease_normals(loft('Body / slender gold flame-bird breast',GOLD,curved_profiles(((.191,.032,.035,-.027),(.265,.119,.131,-.017),(.407,.181,.154,-.005),(.537,.155,.134,.011),(.633,.092,.078,.022),(.683,.031,.033,.025)),4),40),45)
 neck=crease_normals(loft('Neck / long slender golden throat',GOLD,curved_profiles(((.571,.075,.067,.02),(.721,.061,.058,.028),(.851,.064,.061,.034)),4),32),45)
 head=ellipsoid('Head / narrow alert fire-bird cranium',GOLD,(0,.899,.034),(.134,.140,.115),40,26);p=[body,neck,head]
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.023,.956),(.085,.962),(.110,.933),(.082,.908),(.038,.925))]
  socket=paint('Eye socket / fitted alert black lens '+str(s),INK,xy,head,.009,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  white=paint('Eye / narrow ivory fire-bird gaze '+str(s),IVORY,[(cx+.83*(x-cx),cy+.82*(y-cy)) for x,y in xy],socket,.005,3);p.append(white)
  pupil=paint('Pupil / small dark fire-bird stare '+str(s),INK,oval(s*.075,.940,.007,.016,20),white,.004,3);p.append(pupil)
 p.append(bill('Bill / long grey pointed upper beak',GREY,[(.104,.043,.851,.892),(.178,.052,.836,.894),(.304,.033,.793,.826),(.408,.0015,.750,.755)]))
 p.append(bill('Bill seam / fine charcoal join',INK,[(.105,.037,.847,.853),(.178,.043,.832,.838),(.30,.026,.789,.796),(.4,.001,.753,.756)]))
 p.append(bill('Bill / narrow grey lower beak',GREY,[(.105,.037,.824,.847),(.178,.042,.811,.833),(.3,.026,.785,.790),(.4,.0015,.751,.753)]))
 wings(p);crown(p,head);start=len(p);feet(p)
 for q in p[start:]:
  if q['part'].startswith(('Leg /','Foot /','Toe /')):q['base_color']=linear(GREY)
  elif q['part'].startswith('Talon /'):q['base_color']=linear(IVORY)
 tail(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'moltres.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
