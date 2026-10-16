#!/usr/bin/env python3
"""Original small scuffle fighter with a continuous three-pronged head."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals,translate
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import sheet,paint
from skin_glb import COORDINATES
HEIGHT=.86
LILAC=(.72,.62,.77);FOLD=(.57,.47,.62);BROWN=(.49,.37,.22)
HEM=(.42,.31,.19);CREAM=(.95,.94,.86);GOLD=(.96,.79,.26);INK=(.085,.075,.095)

def head():
 xy=[(0,.770),(-.135,.790),(-.218,.847),(-.257,.930),(-.262,1.027),(-.236,1.112),(-.283,1.182),(-.246,1.225),(-.190,1.206),(-.161,1.177),(-.138,1.233),(-.161,1.364),(-.113,1.397),(-.081,1.377),(-.027,1.271),(.008,1.245),(.042,1.273),(.069,1.352),(.111,1.329),(.127,1.303),(.134,1.207),(.212,1.163),(.261,1.067),(.267,.965),(.238,.875),(.159,.805)]
 xy=[(-.265*math.sin(1.95*i/14),1.017-.247*math.cos(1.95*i/14)) for i in range(15)]+xy[6:21]+[(.265*math.sin(1.95*i/14),1.017-.247*math.cos(1.95*i/14)) for i in range(14,0,-1)]
 def depth(y,x):return .125-.90*x*x-.45*(y-1.02)**2
 cage=sheet('Head / continuous lilac three-pronged scuffle crown',LILAC,xy,depth,.110,2)
 vv=[]
 for i in range(0,len(cage['positions']),3):
  x,y,z=cage['positions'][i:i+3];d=depth(y,x);t=max(0,min(1,(y-1.17)/.21));fade=1-.65*t*t*(3-2*t);vv.append((x,y,d+(z-d)*fade))
 h=crease_normals(part(cage['part'],LILAC,vv,[tuple(cage['indices'][i:i+3]) for i in range(0,len(cage['indices']),3)],True),40)
 out=dict(h,positions=[],normals=[],indices=[]);lookup={}
 for k in range(0,len(h['indices']),3):
  ids=h['indices'][k:k+3];points=[h['positions'][3*j:3*j+3] for j in ids];offsets=[z-depth(y,x) for x,y,z in points];front=all(q>0 for q in offsets);back=all(q<0 for q in offsets)
  for j,(x,y,z) in zip(ids,points):
   normal=h['normals'][3*j:3*j+3]
   if front or back:
    t=max(0,min(1,(y-1.17)/.21));slope=-.65*6*t*(1-t)/.21;dy=-.90*(y-1.02)+(.110 if front else -.0385)*slope
    normal=(1.80*x,-dy,1) if front else (-1.80*x,dy,-1);length=math.sqrt(sum(q*q for q in normal));normal=tuple(q/length for q in normal)
   key=(x,y,z,*normal)
   if key not in lookup:lookup[key]=len(out['positions'])//3;out['positions'].extend((x,y,z));out['normals'].extend(normal)
   out['indices'].append(lookup[key])
 return out

def limbs(p):
 for s in (-1,1):
  shorts=translate(crease_normals(loft('Shorts leg / rooted rounded brown scuffle hem '+str(s),BROWN,curved_profiles(((.352,.105,.105,-.040),(.391,.128,.137,-.039),(.481,.131,.142,-.024),(.533,.101,.112,-.016)),4),40),45),s*.106,0,0);p.append(shorts)
  p.append(paddle('Leg / rooted slim lilac bent scuffle shin '+str(s),LILAC,[(s*.113,.397,-.025),(s*.170,.230,.035),(s*.204,.088,.087)],[.067,.048,.044],[.065,.051,.042]))
  shoe=ellipsoid('Shoe / grounded rounded brown scuffle foot '+str(s),BROWN,(s*.211,.045,.139),(.102,.045,.130),40,26);p.append(shoe)
  p.append(paint('Shoe seam / fitted brown toe contour '+str(s),HEM,[(s*.211+dx,y) for dx,y in ((-.082,.038),(-.046,.065),(0,.071),(.046,.065),(.082,.038),(.075,.031),(.042,.058),(0,.063),(-.042,.058),(-.075,.031))],shoe,.006,1))
  p.append(paddle('Arm / rooted slim bent lilac scuffle forelimb '+str(s),LILAC,[(s*.065,.695,.020),(s*.154,.696,.013),(s*.257,.562,.057),(s*.221,.465,.178)],[.055,.060,.046,.038],[.055,.061,.045,.039]))
  p.append(curved_tube('Wrist band / rooted cream scuffle wrap '+str(s),CREAM,[(s*.232,.487,.155),(s*.206,.441,.216)],[.046,.048],32))
  palm=ellipsoid('Palm / open lilac three-finger scuffle hand '+str(s),LILAC,(s*.206,.435,.238),(.069,.065,.044),32,24);p.append(palm)
  fingers=[([(s*.160,.465,.243),(s*.127,.514,.251),(s*.132,.546,.250)],[.027,.023,.012]),
           ([(s*.233,.452,.254),(s*.310,.474,.258),(s*.340,.468,.247)],[.025,.022,.010]),
           ([(s*.221,.397,.241),(s*.224,.323,.256),(s*.246,.304,.247)],[.026,.022,.011])]
  for j,(path,radii) in enumerate(fingers):p.append(curved_tube(f'Finger / rooted curved lilac scuffle digit {s} {j}',LILAC,path,radii,28))

def sculpture(normalize=True):
 torso=crease_normals(loft('Body / compact forward lilac scuffle torso',LILAC,curved_profiles(((.450,.111,.095,-.028),(.541,.147,.110,-.030),(.647,.149,.110,-.009),(.732,.116,.090,.030),(.790,.061,.057,.059)),5),44),45)
 neck=ellipsoid('Neck / short rooted lilac scuffle support',LILAC,(0,.789,.086),(.069,.086,.073),24,24);h=head();p=[torso,neck,h]
 p.append(crease_normals(loft('Waist band / continuous cream scuffle belt',CREAM,curved_profiles(((.475,.134,.114,-.022),(.510,.152,.121,-.023),(.550,.153,.119,-.022)),4),44),45))
 p.append(ellipsoid('Shorts / rooted brown scuffle hip bridge',BROWN,(0,.444,-.028),(.165,.108,.120),40,26))
 for s in (-1,1):
  ear=ellipsoid('Ear / rooted broad lilac side disc '+str(s),LILAC,(s*.238,.987,.112),(.071,.100,.044),24,24);p.append(ear)
  p.append(paint('Ear contour / fitted soft lilac disc edge '+str(s),FOLD,[(s*.238+dx,y) for dx,y in ((-.035,.923),(-.053,.965),(-.050,1.020),(-.026,1.067),(-.020,1.062),(-.043,1.016),(-.046,.966),(-.029,.927))],ear,.003,1))
  xy=[(s*x,y) for x,y in ((.050,1.039),(.163,1.078),(.180,.983),(.153,.915),(.095,.903),(.064,.930))]
  socket=paint('Eye socket / fitted angular dark scuffle recess '+str(s),INK,xy,h,.006,3);p.append(socket);cx=cy=0.
  for x,y in xy:cx+=x;cy+=y
  cx/=len(xy);cy/=len(xy)
  eye=paint('Eye / fitted ivory scuffle lens '+str(s),CREAM,[(cx+.87*(x-cx),cy+.91*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted tall gold scuffle gaze '+str(s),GOLD,oval(s*.126,.982,.025,.054,28),eye,.003,1);p.append(iris)
  pupil=paint('Pupil / fitted dark scuffle slit '+str(s),INK,oval(s*.127,.985,.009,.034,24),iris,.003,2);p.append(pupil)
  p.append(paint('Glint / fitted cream scuffle eye spark '+str(s),CREAM,oval(s*.125,.999,.003,.007,20),pupil,.002,1))
 p.append(paint('Mouth / fitted determined scuffle lip',INK,[(-.071,.841),(-.021,.849),(.034,.837),(.074,.818),(.070,.812),(.030,.830),(-.021,.842),(-.067,.834)],h,.004,3))
 limbs(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 from sculpt_geometry import linear
 for q in p:
  n=q['part'];rgb=(0,0,0) if n.startswith(('Eye socket /','Pupil /','Mouth /')) else (255,255,255) if n.startswith(('Eye /','Glint /','Waist band /','Wrist band /')) else (140,57,24) if n.startswith(('Shorts','Shoe','Iris /','Ear contour /')) else (189,123,148)
  q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'tyrogue','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'tyrogue.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
