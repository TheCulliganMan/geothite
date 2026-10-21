#!/usr/bin/env python3
"""Original small scuffle fighter with a continuous three-pronged head."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals,translate
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from sculpt_geometry import fitted_triangle_contour
from cubone_sculpt import skin_shading
from hitmonchan_sculpt import rounded_cap_cage
from bulldog_sculpt import rounded_loft
from skin_glb import COORDINATES
HEIGHT=.86
LILAC=(.72,.62,.77);FOLD=(.57,.47,.62);BROWN=(.49,.37,.22)
HEM=(.42,.31,.19);CREAM=(.95,.94,.86);GOLD=(.96,.79,.26);INK=(.085,.075,.095)

def paint(name,color,xy,parent,th=.004,sub=1):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),45),parent)

def head():
 # Three short points continue out of a genuinely rounded skull. The planar
 # triangulation is improved before curving both caps, avoiding sliver fans.
 xy=[(0,.715),(-.130,.739),(-.232,.818),(-.278,.933),(-.275,1.062),(-.239,1.147),
     (-.298,1.226),(-.286,1.254),(-.251,1.257),(-.183,1.203),(-.130,1.237),
     (-.091,1.389),(-.055,1.397),(-.033,1.373),(-.016,1.253),(.049,1.235),
     (.107,1.355),(.145,1.352),(.159,1.321),(.157,1.202),(.243,1.134),
     (.279,1.041),(.281,.930),(.237,.817),(.130,.739)]
 xy,caps=rounded_cap_cage(xy);vv=[]
 from skin_glb import smoothstep
 for side in (1,-1):
  for x,y in xy:
   t=smoothstep(1.15,1.27,y);d=.014+.208*math.sqrt(max(0.,1-(x/.292)**2-((y-.985)/.280)**2));d=d*(1-t)+.039*t
   vv.append((x,y,.006+side*d))
 count=len(xy);ff=list(caps)+[tuple(i+count for i in reversed(f)) for f in caps];edges={}
 for f in caps:
  for a,b in zip(f,f[1:]+f[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
 for uses in edges.values():
  if len(uses)==1:
   a,b=uses[0];ff.append((b,a,a+count,b+count))
 return crease_normals(part('Head / continuous lilac three-pronged scuffle crown',LILAC,vv,ff,True),65)

def arm_path(s):
 return [(s*.115,.696,.004),(s*.287,.617,.041),(s*.353,.515,.124)] if s==-1 else [(s*.115,.696,.004),(s*.309,.625,.015),(s*.373,.813,.152)]

def limbs(p):
 for s in (-1,1):
  shorts=translate(crease_normals(loft('Shorts leg / rooted rounded brown scuffle hem '+str(s),BROWN,curved_profiles(((.292,.096,.107,-.024),(.337,.138,.145,-.023),(.419,.142,.150,-.019),(.481,.098,.103,-.014)),3),28),45),s*.123,0,0);p.append(shorts)
  depth=.018 if s==-1 else -.038
  p.append(paddle('Leg / rooted slim lilac bent scuffle shin '+str(s),LILAC,[(s*.132,.335,-.018),(s*.211,.225,depth+.022),(s*.241,.090,depth)],[.074,.071,.051],[.074,.071,.050],segments=20,steps=4))
  shoe=translate(rounded_loft('Shoe / grounded rounded brown scuffle foot '+str(s),BROWN,((0.,.100,.126,depth+.051),(.027,.116,.145,depth+.055),(.079,.107,.130,depth+.053),(.108,.067,.077,depth+.012)),28,.60),s*.242,0,0);p.append(shoe)
  p.append(paint('Shoe seam / fitted brown toe contour '+str(s),HEM,[(s*.242+dx,y) for dx,y in ((-.075,.037),(-.042,.066),(0,.077),(.042,.066),(.075,.037),(.069,.033),(.038,.059),(0,.070),(-.038,.059),(-.069,.033))],shoe,.003,1))
  path=arm_path(s);p.append(paddle('Arm / rooted slim bent lilac scuffle forelimb '+str(s),LILAC,path,[.060,.055,.043],[.061,.055,.044],segments=20,steps=5))
  px,py,pz=(-.381,.488,.158) if s==-1 else (.391,.881,.186)
  p.append(curved_tube('Wrist band / rooted cream scuffle wrap '+str(s),CREAM,[path[-1],(px,py-.016*s,pz-.010)],[.050,.051],24))
  palm=ellipsoid('Palm / open lilac three-finger scuffle hand '+str(s),LILAC,(px,py,pz),(.071,.074,.047),24,18);p.append(palm)
  if s==-1:
   fingers=[([(px+.045,py+.030,pz+.012),(px+.093,py+.065,pz+.025),(px+.097,py+.100,pz+.014)],[.026,.023,.011]),
            ([(px-.043,py-.008,pz+.017),(px-.090,py-.052,pz+.026),(px-.092,py-.086,pz+.013)],[.027,.023,.011]),
            ([(px+.010,py-.046,pz+.021),(px+.012,py-.108,pz+.031),(px+.039,py-.124,pz+.018)],[.028,.023,.011])]
  else:
   fingers=[([(px-.043,py+.009,pz+.014),(px-.093,py+.043,pz+.032),(px-.099,py+.078,pz+.021)],[.026,.023,.011]),
            ([(px-.016,py+.047,pz+.018),(px-.020,py+.115,pz+.025),(px-.002,py+.135,pz+.012)],[.027,.023,.011]),
            ([(px+.040,py+.028,pz+.017),(px+.077,py+.093,pz+.028),(px+.074,py+.120,pz+.015)],[.027,.023,.011])]
  for j,(path,radii) in enumerate(fingers):p.append(curved_tube(f'Finger / rooted curved lilac scuffle digit {s} {j}',LILAC,path,radii,28))

def sculpture(normalize=True):
 torso=crease_normals(loft('Body / compact forward lilac scuffle torso',LILAC,curved_profiles(((.431,.103,.095,-.022),(.512,.125,.111,-.015),(.629,.146,.112,.0),(.706,.119,.086,.013),(.756,.055,.052,.017)),4),32),45)
 neck=ellipsoid('Neck / short rooted lilac scuffle support',LILAC,(0,.748,.016),(.063,.071,.064),24,18);h=head();p=[torso,neck,h]
 p.append(crease_normals(loft('Waist band / continuous cream scuffle belt',CREAM,curved_profiles(((.442,.133,.116,-.020),(.475,.137,.119,-.019),(.506,.126,.114,-.015)),3),32),45))
 p.append(ellipsoid('Shorts / rooted brown scuffle hip bridge',BROWN,(0,.403,-.024),(.191,.111,.142),28,20))
 for s in (-1,1):
  ear=ellipsoid('Ear / rooted broad lilac side disc '+str(s),LILAC,(s*.254,.982,.005),(.096,.102,.125),28,20);p.append(ear)
  p.append(paint('Ear contour / fitted soft lilac disc edge '+str(s),FOLD,[(s*.254+dx,y) for dx,y in ((-.035,.915),(-.065,.951),(-.065,1.009),(-.034,1.059),(-.029,1.054),(-.058,1.006),(-.058,.954),(-.030,.921))],ear,.002,1))
  xy=[(s*x,y) for x,y in ((.057,1.040),(.151,1.066),(.166,1.002),(.147,.940),(.095,.931),(.073,.958))]
  socket=paint('Eye socket / fitted angular dark scuffle recess '+str(s),INK,xy,h,.006,3);p.append(socket);cx=cy=0.
  for x,y in xy:cx+=x;cy+=y
  cx/=len(xy);cy/=len(xy)
  eye=paint('Eye / fitted ivory scuffle lens '+str(s),CREAM,[(cx+.87*(x-cx),cy+.91*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted tall gold scuffle gaze '+str(s),GOLD,oval(s*.119,.991,.021,.043,24),eye,.003,1);p.append(iris)
  pupil=paint('Pupil / fitted dark scuffle slit '+str(s),INK,oval(s*.120,.993,.008,.030,20),iris,.002,2);p.append(pupil)
  p.append(paint('Glint / fitted cream scuffle eye spark '+str(s),CREAM,oval(s*.118,1.007,.003,.007,16),pupil,.0015,1))
 p.append(paint('Mouth / fitted determined scuffle lip',INK,[(-.064,.836),(-.024,.843),(.036,.841),(.067,.827),(.064,.821),(.032,.834),(-.022,.836),(-.060,.829)],h,.003,3))
 limbs(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 from sculpt_geometry import linear
 for q in p:
  n=q['part'];rgb=(0,0,0) if n.startswith(('Eye socket /','Pupil /','Mouth /')) else (255,255,255) if n.startswith(('Eye /','Glint /','Waist band /','Wrist band /')) else (140,57,24) if n.startswith(('Shorts','Shoe','Iris /','Ear contour /')) else (189,123,148)
  q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) or 0. for v in q['normals']]
 return {'name':'tyrogue','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'tyrogue.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
