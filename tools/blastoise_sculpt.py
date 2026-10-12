#!/usr/bin/env python3
"""Original heavy shell turtle with genuine recessed twin cannon bores."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals
from bovine_sculpt import oval,curved_tube,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import sheet,paint
from wartortle_sculpt import ribbon
from chikorita_sculpt import add,sub,mul,dot,cross,unit
from skin_glb import COORDINATES
HEIGHT=1.18;SPECIES=('blastoise',)
BLUE=(.32,.51,.74);TAN=(.88,.79,.56);WHITE=(.97,.97,.91)
BROWN=(.48,.38,.27);SEAM=(.27,.24,.21);SILVER=(.78,.80,.79)
BORE=(.18,.21,.23);INK=(.055,.065,.085);IRIS=(.42,.29,.21)

def frame(root,tip):
 axis=unit(sub(tip,root));side=unit(cross(axis,(0,1,0)));other=cross(axis,side);return axis,side,other

def cannon(p,s):
 root=(s*.26,.765,-.083);tip=(s*.335,1.045,.300);axis,side,other=frame(root,tip);length=math.sqrt(dot(sub(tip,root),sub(tip,root)))
 # The inner wall turns inward at the muzzle and ends well behind its rim.
 rows=[(0,.078),(.060,.078),(length-.047,.068),(length,.071),(length,.048),(length-.165,.046)]
 n=48;v=[]
 for t,r in rows:
  center=add(root,mul(axis,t));v.extend(add(center,add(mul(side,r*math.cos(i*math.tau/n)),mul(other,r*math.sin(i*math.tau/n)))) for i in range(n))
 f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
 for j in range(len(rows)-1):
  for i in range(n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
 p.append(crease_normals(part('Cannon / hollow silver shoulder barrel '+str(s),SILVER,v,f,True),38))
 # A closed annular lining darkens the real interior without filling its void.
 rows=[(length-.163,.0477),(length-.003,.0497),(length-.003,.0467),(length-.163,.0447)];v=[]
 for t,r in rows:
  center=add(root,mul(axis,t));v.extend(add(center,add(mul(side,r*math.cos(i*math.tau/n)),mul(other,r*math.sin(i*math.tau/n)))) for i in range(n))
 f=[]
 for j in range(4):
  for i in range(n):a=j*n+i;b=j*n+(i+1)%n;c=((j+1)%4)*n+(i+1)%n;d=((j+1)%4)*n+i;f.append((a,b,c,d))
 p.append(crease_normals(part('Cannon lining / rooted dark open annulus '+str(s),BORE,v,f,True),38))
 floor=ellipsoid('Cannon floor / recessed dark bore '+str(s),INK,(0,0,0),(.045,.045,.010),32,18)
 center=add(root,mul(axis,length-.163))
 for field in ('positions','normals'):
  old=floor[field];vv=[]
  for i in range(0,len(old),3):
   q=add(add(mul(side,old[i]),mul(other,old[i+1])),mul(axis,old[i+2]));q=add(q,center) if field=='positions' else q;vv.extend(q)
  floor[field]=vv
 p.append(floor)
 # Blue shoulder socket physically roots both the shell and barrel.
 p.append(ellipsoid('Cannon mount / blue rooted shoulder socket '+str(s),BLUE,(s*.265,.744,-.082),(.105,.094,.098),36,24))

def rim():
 n=64;m=16;v=[]
 for i in range(n):
  a=i*math.tau/n
  for j in range(m):
   t=j*math.tau/m;v.append(((.317+.025*math.cos(t))*math.cos(a),.455+(.346+.025*math.cos(t))*math.sin(a),-.015+.025*math.sin(t)))
 f=[(i*m+j,((i+1)%n)*m+j,((i+1)%n)*m+(j+1)%m,i*m+(j+1)%m) for i in range(n) for j in range(m)]
 return crease_normals(part('Shell rim / continuous thick ivory turtle edge',WHITE,v,f,True),45)

def limbs(p):
 for s in (-1,1):
  p.append(paddle('Arm / muscular blue cannon-turtle forelimb '+str(s),BLUE,[(s*.25,.652,.024),(s*.405,.589,.093),(s*.477,.516,.203)],[.110,.115,.104],[.116,.112,.106]))
  p.append(ellipsoid('Hand / substantial blue turtle fist '+str(s),BLUE,(s*.485,.489,.218),(.117,.119,.125),36,24))
  for j,dx in enumerate((-.060,0,.060)):
   p.append(curved_tube(f'Hand claw / three ivory turtle points {s} {j}',WHITE,[(s*.485+dx,.460,.302),(s*.485+dx*.95,.449,.343),(s*.485+dx*.9,.428,.365)],[.024,.019,.002],24))
  p.append(ellipsoid('Haunch / powerful blue cannon-turtle thigh '+str(s),BLUE,(s*.235,.235,.044),(.157,.196,.157),40,26))
  p.append(paddle('Leg / short strong blue turtle ankle '+str(s),BLUE,[(s*.25,.22,.064),(s*.264,.116,.128),(s*.265,.051,.181)],[.085,.082,.086],[.087,.074,.050]))
  p.append(ellipsoid('Foot / grounded broad blue turtle paw '+str(s),BLUE,(s*.265,.035,.187),(.111,.035,.111),36,22))
  for j,dx in enumerate((-.065,0,.065)):
   p.append(curved_tube(f'Toe / three blue turtle digits {s} {j}',BLUE,[(s*.265+dx*.5,.042,.233),(s*.265+dx,.027,.277),(s*.265+dx,.021,.291)],[.035,.026,.020],24))
   p.append(curved_tube(f'Foot claw / three grounded ivory hooks {s} {j}',WHITE,[(s*.265+dx,.024,.278),(s*.265+dx,.017,.312),(s*.265+dx,.002,.331)],[.021,.015,.002],24))

def sculpture(species='blastoise'):
 if species not in SPECIES:raise ValueError('unsupported cannon turtle')
 body=ellipsoid('Body / blue mantle beneath armored turtle plates',BLUE,(0,.455,.011),(.307,.350,.170),48,30)
 shell=ellipsoid('Shell / substantial domed brown turtle armor',BROWN,(0,.455,-.109),(.340,.375,.233),52,32)
 belly=ellipsoid('Belly / gently domed tan turtle breast',TAN,(0,.455,.068),(.287,.335,.146),48,30)
 head=crease_normals(loft('Head / broad angular blue turtle cranium',BLUE,curved_profiles(((.725,.085,.098,.083),(.795,.193,.156,.083),(.91,.223,.175,.074),(.999,.187,.139,.049),(1.055,.106,.080,.020),(1.064,.025,.026,.014)),4),48),45)
 muzzle=ellipsoid('Muzzle / broad low blue turtle snout',BLUE,(0,.830,.221),(.192,.073,.080),44,26)
 jaw=ellipsoid('Jaw / broad cream cannon-turtle chin',TAN,(0,.761,.174),(.158,.069,.095),40,26);p=[body,shell,belly,rim(),head,muzzle,jaw]
 for j,path in enumerate(([(-.237,.596),(-.127,.565),(0,.552),(.127,.565),(.237,.596)],[(-.202,.365),(-.112,.339),(0,.333),(.112,.339),(.202,.365)],[(0,.747),(0,.628),(0,.489),(0,.356),(0,.182)])):
  p.append(paint('Belly seam / fitted plated turtle join '+str(j),SEAM,ribbon(path,.0035),belly,.005,3))
 rear=reverse_front(shell);hexagon=[(0,.744),(-.174,.646),(-.193,.443),(0,.284),(.193,.443),(.174,.646)]
 paths=[[a,b] for a,b in zip(hexagon,hexagon[1:]+hexagon[:1])]+[[(0,.284),(0,.095)],[(-.174,.646),(-.248,.685)],[(.174,.646),(.248,.685)],[(-.193,.443),(-.300,.333)],[(.193,.443),(.300,.333)]]
 for j,path in enumerate(paths):p.append(reverse_front(paint('Shell seam / fitted rear armored scute '+str(j),SEAM,ribbon(path,.0035),rear,.005,3)))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.042,.942),(.143,.970),(.151,.940),(.131,.916),(.069,.916),(.041,.928))]
  socket=paint('Eye socket / fitted narrow cannon-turtle glare '+str(s),INK,xy,head,.008,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted narrow ivory turtle lens '+str(s),WHITE,[(cx+.91*(x-cx),cy+.87*(y-cy)) for x,y in xy],socket,.005,3);p.append(eye)
  iris=paint('Iris / restrained brown turtle gaze '+str(s),IRIS,oval(s*.114,.938,.012,.015,24),eye,.004,3);p.append(iris)
  p.append(paint('Pupil / small dark turtle center '+str(s),INK,oval(s*.115,.938,.006,.011,20),iris,.003,3))
  xy=[(.133,.982),(.178,1.099),(.239,1.058),(.201,.959)]
  ear=sheet('Ear / compact pointed blue turtle ear '+str(s),BLUE,[(s*x,y) for x,y in xy],lambda y,x:.018-.40*(y-.98),.042,3);p.append(ear)
  xy=[(.156,1.007),(.177,1.068),(.213,1.045),(.195,1.007)]
  p.append(paint('Ear inset / fitted dark triangular turtle fold '+str(s),INK,[(s*x,y) for x,y in xy],ear,.005,3))
  p.append(paint('Nostril / small fitted turtle dot '+str(s),INK,oval(s*.050,.850,.004,.003,16),muzzle,.003,2))
 mouth=[(-.144,.817),(-.092,.803),(0,.798),(.092,.803),(.144,.817),(.112,.793),(.057,.784),(0,.782),(-.057,.784),(-.112,.793)]
 p.append(paint('Mouth / fitted dark cannon-turtle smile',INK,mouth,muzzle,.004,3))
 p.append(paddle('Tail / short tapered blue turtle wedge',BLUE,[(0,.265,-.215),(0,.182,-.359),(0,.125,-.459)],[.079,.061,.003],[.067,.043,.003]))
 for s in (-1,1):cannon(p,s)
 limbs(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'blastoise.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
