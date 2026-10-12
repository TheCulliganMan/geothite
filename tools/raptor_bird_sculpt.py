#!/usr/bin/env python3
"""Original swept-crested Pidgeot and long-billed Fearow, with layered vanes."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals,part,fitted_contour
from bovine_sculpt import curved_tube,oval,reverse_front
from wing_insect_sculpt import contour
from goldfish_sculpt import surface,zy
from murkrow_sculpt import bill
from skin_glb import COORDINATES

SPECIES=('pidgeot','fearow')
HEIGHTS={'pidgeot':1.35,'fearow':1.23}
BROWN=(.61,.40,.20);LIGHT_BROWN=(.73,.50,.27);CREAM=(.97,.90,.63)
GOLD=(.94,.76,.31);RED=(.90,.38,.27);PINK=(.78,.49,.56)
INK=(.075,.07,.09);WHITE=(.98,.97,.91);TALON=(.88,.87,.78)

def sheet(name,color,xy,depth,th=.018,sub=2):
 return crease_normals(fitted_contour(name,color,xy,depth,th,sub),35)

def paint(name,color,xy,parent,th=.006,sub=3):
 return sheet(name,color,xy,surface(parent),th,sub)

def vane_xy(root,tip,width):
 dx,dy=tip[0]-root[0],tip[1]-root[1];length=math.hypot(dx,dy)
 nx,ny=-dy/length*width,dx/length*width
 def point(t,w):return (root[0]+t*dx+w*nx,root[1]+t*dy+w*ny)
 return contour([(point(0,-.30),point(.25,-.95),point(.76,-1),point(1,0)),
                 (point(1,0),point(.84,.90),point(.29,.88),point(0,.30)),
                 (point(0,.30),point(-.07,.20),point(-.07,-.20),point(0,-.30))],8)

def feather_depth(base,root,tip,width,offset,bank):
 dx,dy=tip[0]-root[0],tip[1]-root[1];dd=dx*dx+dy*dy
 def at(y,x):
  t=((x-root[0])*dx+(y-root[1])*dy)/dd
  w=((x-root[0])*(-dy)+(y-root[1])*dx)/math.sqrt(dd)
  return base(y,x)+offset+bank*.009*math.sin(math.pi*max(0,min(1,t)))*max(0,1-(w/width)**2)
 return at

def pale_tip(p,feather,xy,root,tip,prefix):
 dx,dy=tip[0]-root[0],tip[1]-root[1];dd=dx*dx+dy*dy
 def along(q):return ((q[0]-root[0])*dx+(q[1]-root[1])*dy)/dd
 clipped=[]
 for a,b in zip(xy,xy[1:]+xy[:1]):
  ta,tb=along(a),along(b)
  if ta>=.70:clipped.append(a)
  if (ta>=.70)!=(tb>=.70):
   t=(.70-ta)/(tb-ta);clipped.append((a[0]+t*(b[0]-a[0]),a[1]+t*(b[1]-a[1])))
 cx=sum(x for x,y in clipped)/len(clipped);cy=sum(y for x,y in clipped)/len(clipped)
 inset=[(cx+.92*(x-cx),cy+.92*(y-cy)) for x,y in clipped]
 for face in (-1,1):
  parent=feather if face==1 else reverse_front(feather)
  patch=paint(prefix+' '+str(face),CREAM,inset if face==1 else [(-x,y) for x,y in inset],parent,.004,2)
  p.append(patch if face==1 else reverse_front(patch))

def wings(p,fear):
 for sign in (-1,1):
  depth=lambda y,x:-.02-.14*abs(x)+.028*math.sin(3*y)
  outline=contour([((.095,.56),(.21,.78),(.35,.98),(.57,1.08)),
    ((.57,1.08),(.70,1.15),(.85,1.26),(.94,1.30)),
    ((.94,1.30),(.89,1.13),(.74,.97),(.60,.84)),
    ((.60,.84),(.43,.68),(.29,.57),(.095,.56))],9)
  wing=sheet('Wing / curved rooted upper spar '+str(sign),BROWN if fear else GOLD,[(sign*x,y) for x,y in outline],depth,.033,2);p.append(wing)
  if not fear:
   cx=sum(x for x,y in outline)/len(outline);cy=sum(y for x,y in outline)/len(outline)
   inset=[(sign*(cx+.93*(x-cx)),cy+.93*(y-cy)) for x,y in outline]
   for face in (1,):
    parent=wing if face==1 else reverse_front(wing)
    patch=paint(f'Wing field / supported cream underside {sign} {face}',CREAM,inset if face==1 else [(-x,y) for x,y in inset],parent,.006,1)
    p.append(patch if face==1 else reverse_front(patch))
  for j in range(8):
   root=(sign*(.21+.059*j),.66+.065*j);tip=(sign*(.43+.085*j),.50+.115*j)
   xy=vane_xy(root,tip,.065 if fear else .072)
   feather=sheet(f'Primary / layered flight vane {sign} {j}',LIGHT_BROWN if fear else CREAM,xy,feather_depth(depth,root,tip,.065 if fear else .072,.027,1),.019,1);p.append(feather)
   rear=sheet(f'Rear primary / layered upper flight vane {sign} {j}',LIGHT_BROWN,xy,feather_depth(depth,root,tip,.065 if fear else .072,-.023,-1),.020,1);p.append(rear)
   if fear:
    # A pale tip is fitted to both faces of the actual closed vane.
    pale_tip(p,feather,xy,root,tip,f'Primary tip / supported pale terminal {sign} {j}')
    pale_tip(p,rear,xy,root,tip,f'Rear primary tip / supported pale terminal {sign} {j}')
  for j in range(6):
   root=(sign*(.20+.073*j),.71+.068*j);tip=(sign*(.37+.076*j),.60+.071*j)
   xy=vane_xy(root,tip,.044)
   p.append(sheet(f'Covert / smaller layered shoulder vane {sign} {j}',LIGHT_BROWN if fear else CREAM,xy,feather_depth(depth,root,tip,.044,.037,1),.020,1))
   p.append(sheet(f'Rear covert / smaller upper shoulder vane {sign} {j}',LIGHT_BROWN,xy,feather_depth(depth,root,tip,.044,-.029,-1),.023,1))

def feet(p):
 for s in (-1,1):
  p.append(curved_tube('Leg / rooted pink shin '+str(s),PINK,[(s*.092,.275,.018),(s*.105,.156,.020),(s*.108,.041,.041)],[.028,.021,.024],20))
  p.append(ellipsoid('Foot / grounded pink knuckle '+str(s),PINK,(s*.108,.023,.057),(.053,.023,.050),26,16))
  for j,dx in enumerate((-.039,0,.039)):
   end=(s*.108+dx,.015,.154 if j==1 else .135)
   p.append(curved_tube(f'Toe / three forward digits {s} {j}',PINK,[(s*.108+dx*.20,.031,.077),(s*.108+dx*.65,.023,.110),end],[.014,.012,.010],16))
   p.append(curved_tube(f'Talon / forward ivory hook {s} {j}',TALON,[(end[0],end[1],end[2]-.005),(end[0],.009,end[2]+.025),(end[0],.002,end[2]+.037)],[.010,.007,.0015],16))
  p.append(curved_tube('Toe / rear gripping digit '+str(s),PINK,[(s*.108,.031,.035),(s*.13,.019,-.025),(s*.14,.014,-.050)],[.013,.011,.008],16))
  p.append(curved_tube('Talon / rear ivory hook '+str(s),TALON,[(s*.14,.014,-.045),(s*.15,.008,-.068),(s*.15,.002,-.082)],[.008,.006,.0015],16))

def tails(p,fear):
 for j in range(5):
  x=(j-2)*.060
  xy=vane_xy((x*.20,.355),(x,.172+abs(j-2)*.012),.043)
  p.append(sheet('Tail / splayed rear feather '+str(j),LIGHT_BROWN if fear else RED,xy,lambda y,x:-.145-2.25*(.355-y),.018,2))

def pidgeot_face(p,head):
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.025,.940),(.110,.953),(.160,.917),(.156,.865),(.111,.790),(.086,.836),(.067,.864))]
  mask=paint('Mask / hooked charcoal cheek stripe '+str(s),INK,xy,head,.010,3);p.append(mask)
  xy=[(s*x,y) for x,y in ((.047,.919),(.112,.927),(.137,.911),(.113,.875),(.065,.889))]
  white=paint('Eye / angular fitted ivory '+str(s),WHITE,xy,mask,.006,3);p.append(white)
  pupil=paint('Pupil / restrained dark gaze '+str(s),INK,oval(s*.104,.901,.014,.023,24),white,.004,3);p.append(pupil)
  p.append(paint('Glint / tiny ivory gaze '+str(s),WHITE,oval(s*.104-.003,.911,.003,.005,16),pupil,.003,2))
 upper=bill('Bill / short hooked pink upper beak',PINK,[(.160,.063,.824,.873),(.230,.079,.822,.877),(.304,.043,.803,.849),(.330,.003,.793,.808)]);p.append(upper)
 p.append(bill('Bill / dark short seam',INK,[(.162,.059,.819,.825),(.236,.060,.815,.821),(.299,.024,.805,.811),(.313,.002,.803,.809)]))
 p.append(bill('Bill / short pink lower beak',PINK,[(.160,.057,.782,.817),(.221,.060,.776,.814),(.287,.030,.795,.807),(.310,.002,.803,.807)]))
 for j in (-1,0,1):
  y=1.025-abs(j)*.018;end=.70 if j==0 else .61
  xy=contour([((-.185,y-.052),(-.10,y+.074),(.28,y+.090),(end,.925-abs(j)*.014)),
              ((end,.925-abs(j)*.014),(.44,.988),(.18,1.010),(-.185,y-.052))],12)
  q=zy(sheet('Crest / long swept ribbon '+str(j),GOLD if j==0 else RED,xy,lambda y,x:j*.047,.024,2));p.append(q)

def fearow_face(p,head):
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.029,.986),(.085,.998),(.110,.977),(.083,.950),(.039,.953))]
  socket=paint('Eye socket / sharp dark bill-side gaze '+str(s),INK,xy,head,.009,3);p.append(socket)
  white=paint('Eye / sharp fitted ivory '+str(s),WHITE,[(s*x,y) for x,y in ((.036,.982),(.083,.990),(.099,.976),(.079,.957),(.044,.959))],socket,.006,3);p.append(white)
  pupil=paint('Pupil / small stern gaze '+str(s),INK,oval(s*.080,.974,.008,.014,20),white,.004,3);p.append(pupil)
 upper=bill('Bill / long tapering pink upper lance',PINK,[(.468,.047,.921,.960),(.566,.054,.898,.940),(.748,.026,.862,.888),(.854,.002,.836,.842)]);p.append(upper)
 p.append(bill('Bill / open lower pink mandible',PINK,[(.469,.038,.890,.916),(.530,.036,.858,.887),(.613,.025,.827,.846),(.667,.002,.856,.860)]))
 crown=[(-.092,.993),(-.103,1.105),(-.054,1.066),(-.040,1.202),(-.012,1.117),(.024,1.242),(.045,1.116),(.075,1.192),(.079,1.088),(.107,1.136),(.086,.999)]
 p.append(sheet('Crown / five pointed upright red feathers',RED,crown,lambda y,x:.403,.037,2))
 mane=[(-.065,.682),(-.125,.813),(-.087,.792),(-.100,.901),(-.056,.858),(-.032,.965),(.001,.885),(.038,.976),(.062,.867),(.100,.928),(.084,.790),(.130,.850),(.075,.680)]
 p.append(sheet('Mane / ragged pale rear neck ruff',CREAM,mane,lambda y,x:.094+.10*(y-.68),.030,2))

def sculpture(species):
 if species not in SPECIES:raise ValueError('unsupported raptor bird')
 fear=species=='fearow'
 rows=curved_profiles(((.188,.045,.045,-.018),(.265,.129,.145,-.025),(.405,.201,.189,-.020),(.525,.186,.167,-.005),(.620,.112,.103,.015),(.692,.039,.041,.036)),4)
 body=crease_normals(loft('Body / tapered individual bird breast',BROWN,rows,40),45);p=[body]
 if fear:
  neck=curved_tube('Neck / long bent narrow fishing neck',BROWN,[(0,.580,.030),(0,.687,.104),(0,.826,.255),(0,.940,.358)],[.077,.055,.043,.041],28)
  head=ellipsoid('Head / narrow long-billed cranium',LIGHT_BROWN,(0,.948,.400),(.126,.097,.105),40,26)
 else:
  neck=crease_normals(loft('Neck / broad crested eagle neck',BROWN,curved_profiles(((.570,.10,.091,.016),(.711,.108,.099,.015),(.859,.110,.11,.020)),4),32),45)
  head=ellipsoid('Head / rounded crested eagle cranium',BROWN,(0,.875,.025),(.174,.175,.154),40,26)
 p.extend((neck,head))
 if fear:
  xy=[(-.145,.548),(-.182,.458),(-.119,.478),(-.118,.408),(-.067,.446),(-.042,.373),(.005,.437),(.041,.385),(.073,.455),(.116,.416),(.112,.481),(.174,.458),(.140,.547)]
  p.append(paint('Breast / fitted ragged ochre feather bib',LIGHT_BROWN,xy,body,.011,3))
 else:
  xy=contour([((0,.223),(-.130,.247),(-.185,.382),(-.145,.534)),
    ((-.145,.534),(-.097,.640),(-.064,.642),(0,.651)),
    ((0,.651),(.064,.642),(.097,.640),(.145,.534)),
    ((.145,.534),(.185,.382),(.130,.247),(0,.223))],9)
  p.append(paint('Breast / fitted broad cream belly',CREAM,xy,body,.012,4))
  xy=[(-.058,.619),(-.077,.694),(-.080,.806),(-.050,.841),(.050,.841),(.080,.806),(.077,.694),(.058,.619)]
  p.append(paint('Throat / fitted continuous cream neck',CREAM,xy,neck,.011,4))
  xy=[(-.075,.737),(-.108,.814),(-.081,.864),(0,.897),(.081,.864),(.108,.814),(.075,.737),(0,.712)]
  p.append(paint('Cheek / fitted cream throat',CREAM,xy,head,.009,3))
 wings(p,fear);feet(p);tails(p,fear)
 (fearow_face if fear else pidgeot_face)(p,head)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
