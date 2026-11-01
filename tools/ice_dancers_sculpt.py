#!/usr/bin/env python3
"""Original distinct flowing-gown Jynx and bob-haired Smoochum sculptures."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals
from bovine_sculpt import oval,curved_tube,reverse_front
from lugia_sculpt import paddle
from sculpt_geometry import fitted_triangle_contour
from cubone_sculpt import skin_shading,recessed
from hitmonchan_sculpt import rounded_cap_cage
from goldfish_sculpt import surface
from skin_glb import COORDINATES
SPECIES=('jynx','smoochum');HEIGHTS={'jynx':1.28,'smoochum':.78}
PURPLE=(.48,.35,.61);PINK=(.78,.44,.61);RED=(.84,.30,.29)
GOLD=(.95,.82,.36);HAIR=(.95,.91,.62);BABY_HAIR=(.96,.89,.45)
CREAM=(.98,.94,.75);WHITE=(.96,.97,.94);INK=(.08,.065,.10)
LIP=(.88,.54,.60);GREEN=(.20,.44,.37)

def paint(name,color,xy,parent,th=.004,sub=1):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),45),parent)

def lips(p,head,cx,cy,rx,ry,baby=False):
 at=surface(head);vv=[];nn=48 if baby else 36;mm=16;minor=.018 if baby else .023
 for i in range(nn):
  a=math.tau*i/nn
  for j in range(mm):
   b=math.tau*j/mm;x=cx+(rx+minor*math.cos(b))*math.cos(a);y=cy+(ry+minor*math.cos(b))*math.sin(a)
   vv.append((x,y,at(y,x)+.010+minor*math.sin(b)))
 ff=[]
 for i in range(nn):
  for j in range(mm):ff.append((i*mm+j,((i+1)%nn)*mm+j,((i+1)%nn)*mm+(j+1)%mm,i*mm+(j+1)%mm))
 lip=crease_normals(part('Lips / rooted pink oval kiss rim',LIP,vv,ff,True),45);p.append(lip)
 p.append(paint('Mouth / fitted dark recessed oval kiss opening',INK,oval(cx,cy,rx-minor+.004,ry-minor+.004,40 if baby else 28),head,.002,1))

def eyes(p,head,baby):
 cy=.739 if baby else 1.177;cx=.116 if baby else .093;rx=.072 if baby else .055;ry=.085 if baby else .068
 for s in (-1,1):
  socket=paint('Eye socket / fitted dark ice-dancer recess '+str(s),INK,oval(s*cx,cy,rx+.006,ry+.006,36 if baby else 24),head,.005,1);p.append(socket)
  eye=paint('Eye / fitted ivory ice-dancer gaze '+str(s),WHITE,oval(s*cx,cy,rx,ry,36 if baby else 24),socket,.004,1);p.append(eye)
  if baby:
   pupil=paint('Pupil / fitted broad dark baby stare '+str(s),INK,oval(s*cx-.006,cy+.003,.045,.059,32),eye,.003,1);p.append(pupil)
   p.append(paint('Glint / fitted ivory baby eye spark '+str(s),WHITE,oval(s*cx-.018,cy+.035,.012,.016,24),pupil,.002,1))
  else:p.append(paint('Pupil / fitted tiny dark dancer stare '+str(s),INK,oval(s*cx,cy,.012,.018,20),eye,.003,1))

def gown():
 rows=[(.009,.490,.333),(.085,.481,.331),(.263,.414,.299),(.440,.313,.247),(.586,.228,.177),(.699,.232,.161),(.837,.241,.164),(.939,.117,.100)];n=64;vv=[]
 rows=curved_profiles([(y,w,d,-.015) for y,w,d in rows],3)
 for y,w,d,z in rows:
  fade=max(0,1-y/.68)
  for i in range(n):
   a=math.tau*i/n;fold=1+.105*fade*math.cos(7*a);vv.append((w*math.sin(a)*fold,y,-.015+d*math.cos(a)*fold))
 ff=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
 for k in range(len(rows)-1):
  for i in range(n):a=k*n+i;b=k*n+(i+1)%n;ff.append((a,b,b+n,a+n))
 return crease_normals(part('Gown / continuous flared seven-fold pink dancer dress',RED,vv,ff,True),40)

def jynx_mark(name,color,xy,parent,th=.003,sub=1):
 # Clip narrow adult grooves to the actual parent triangles; a low-poly cap
 # must not bridge across the curvature of a long hair lock.
 from sculpt_geometry import fitted_triangle_contour
 return crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),35)

def jynx_hair(p):
 back=loft('Hair back / rooted long ivory dancer mantle',HAIR,curved_profiles(((.628,.281,.157,-.094),(.760,.338,.191,-.096),(.979,.331,.203,-.074),(1.189,.294,.204,-.049),(1.336,.212,.164,-.024),(1.421,.066,.064,-.009)),4),48)
 vv=[]
 for i in range(0,len(back['positions']),3):
  x,y,z=back['positions'][i:i+3];a=math.atan2(x,(z+.094)/.157*.281);fade=max(0.,min(1.,(.80-y)/(.80-.628)));vv.append((x,y+fade*.045*math.cos(7*a),z))
 back=crease_normals(part(back['part'],HAIR,vv,[tuple(back['indices'][i:i+3]) for i in range(0,len(back['indices']),3)],True),45);p.append(back)
 dome=ellipsoid('Hair crown / rooted ivory center-parted dancer dome',HAIR,(0,1.348,-.021),(.235,.100,.159),28,18);p.append(dome)
 for j in (-1,0,1):
  x=j*.135;xy=[(x-.004,.813),(x-.014,.976),(x-.016,1.151),(x-.010,1.247),(x-.004,1.247),(x-.010,1.151),(x-.008,.976),(x+.002,.813)]
  strand=jynx_mark('Back hair strand / fitted subtle ivory dancer groove '+str(j),(.84,.80,.52),xy,reverse_front(back),.003,1);p.append(reverse_front(strand))
 for s in (-1,1):
  p.append(paddle('Hair part / rooted arched ivory dancer forelock '+str(s),HAIR,[(0,1.341,.113),(s*.058,1.426,.073),(s*.127,1.445,.040),(s*.215,1.401,.038),(s*.271,1.323,.074)],[.005,.047,.053,.058,.062],[.005,.041,.045,.047,.051],segments=18,steps=4))
  p.append(paddle('Hair curtain / rooted sweeping ivory dancer lock '+str(s),HAIR,[(s*.155,1.379,.042),(s*.267,1.253,.132),(s*.317,1.011,.177),(s*.319,.786,.201),(s*.263,.601,.222)],[.096,.102,.094,.075,.004],[.067,.075,.071,.055,.004],segments=20,steps=4))
  curtain=p[-1];xy=[(s*x,y) for x,y in ((.278,.681),(.318,.855),(.317,1.017),(.274,1.242),(.267,1.242),(.310,1.017),(.311,.855),(.271,.681))]
  p.append(jynx_mark('Hair strand / fitted subtle ivory curtain groove '+str(s),(.84,.80,.52),xy,curtain,.003,1))
  for j in range(2):
   p.append(paddle(f'Hair tip / rooted layered ivory dancer point {s} {j}',HAIR,[(s*(.298+.028*j),.928,.141-.015*j),(s*(.324+.025*j),.752,.103-.069*j),(s*(.311+.040*j),.574+.030*j,.134-.080*j)],[.065,.053,.003],[.056,.042,.003],segments=16,steps=4))

def jynx_arm_path(s):
 return [(-.172,.872,.004),(-.351,.792,.045),(-.463,.899,.136)] if s==-1 else [(.172,.872,.004),(.374,.743,.095),(.293,.699,.290)]

def jynx_hand(s):return (-.489,.949,.158) if s==-1 else (.273,.737,.332)

def jynx_digit_paths(s):
 x,y,z=jynx_hand(s)
 shapes=[((-.044,.007),(-.103,.035),(-.113,.068)),((-.030,.052),(-.053,.129),(-.038,.151)),((.005,.063),(.007,.153),(.026,.169)),((.043,.043),(.095,.107),(.108,.105)),((.056,.004),(.122,.037),(.128,.057))]
 return [([(x+s*dx,y+dy,z+(.008 if k==0 else .032 if k==1 else .018)) for k,(dx,dy) in enumerate(shape)],[.025,.021,.010]) for shape in shapes]

def jynx_limbs(p):
 for s in (-1,1):
  p.append(paddle('Sleeve / rooted bent white dancer arm '+str(s),WHITE,jynx_arm_path(s),[.082,.061,.061],[.085,.060,.060],segments=20,steps=5))
  hand=ellipsoid('Palm / open purple five-finger dancer hand '+str(s),PURPLE,jynx_hand(s),(.075,.082,.051),24,18);p.append(hand)
  paths=jynx_digit_paths(s)
  for j,(path,radii) in enumerate(paths):p.append(curved_tube(f'Finger / rooted purple dancer digit {s} {j}',PURPLE,path,radii,16))

def jynx():
 dress=gown();neck=ellipsoid('Neck / rooted purple dancer throat',PURPLE,(0,.953,.016),(.117,.126,.107),24,18)
 head=recessed(ellipsoid('Head / broad purple dancer face',PURPLE,(0,1.117,.037),(.246,.254,.202),56,40),[(0,1.013,.109,.084,.060)]);p=[dress,neck,head]
 for s in (-1,1):
  plate=ellipsoid('Chest trim / rooted white dancer bodice lobe '+str(s),GOLD,(s*.127,.844,.150),(.139,.124,.104),28,20);p.append(plate)
  p.append(paint('Bodice / fitted pink dancer chest inset '+str(s),RED,oval(s*.127,.844,.112,.094,28),plate,.005,2))
 bridge=ellipsoid('Chest bridge / rooted white dancer central waist trim',GOLD,(0,.801,.175),(.054,.076,.051),24,18);p.append(bridge)
 eyes(p,head,False);lips(p,head,0,1.013,.067,.046);jynx_hair(p);jynx_limbs(p)
 return p

def smoochum_hair(p):
 back=crease_normals(loft('Hair back / rooted short golden baby bob',BABY_HAIR,curved_profiles(((.432,.261,.181,-.076),(.497,.298,.223,-.080),(.702,.304,.246,-.076),(.904,.263,.224,-.044),(1.043,.159,.150,-.006),(1.093,.040,.038,.004)),4),40),50);p.append(back)
 cap=ellipsoid('Hair crown / rooted golden baby dome',BABY_HAIR,(0,.900,-.005),(.303,.210,.244),36,28)
 # The broad crown occupies the rear of the swept fringe at both temples.
 # Recess its lower center so it does not fill the fringe's actual notch.
 from skin_glb import smoothstep
 vv=[]
 for i in range(0,len(cap['positions']),3):
  x,y,z=cap['positions'][i:i+3]
  if z>-.005:z-=min(.110,.70*(z+.005))*max(0.,1-abs(x)/.045)*(1-smoothstep(.880,.925,y))
  vv.append((x,y,z))
 cap=crease_normals(part(cap['part'],BABY_HAIR,vv,[tuple(cap['indices'][i:i+3]) for i in range(0,len(cap['indices']),3)],True),45);p.append(cap)
 xy=[(-.305,.847),(-.278,.940),(-.208,1.030),(-.090,1.091),(.025,1.108),(.167,1.060),(.264,.966),(.305,.858),(.190,.834),(.039,.841),(.012,.909),(-.015,.837),(-.177,.826)]
 xy,caps=rounded_cap_cage(xy);vv=[]
 for side in (1,-1):
  for x,y in xy:
   depth=-.005+.231*math.sqrt(max(0.,1-(x/.321)**2-((y-.852)/.270)**2));vv.append((x,y,depth+side*.025))
 count=len(xy);ff=list(caps)+[tuple(i+count for i in reversed(f)) for f in caps];edges={}
 for f in caps:
  for a,b in zip(f,f[1:]+f[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
 for uses in edges.values():
  if len(uses)==1:
   a,b=uses[0];ff.append((b,a,a+count,b+count))
 p.append(crease_normals(part('Bangs / continuous golden baby fringe with center notch',BABY_HAIR,vv,ff,True),55))
 for s in (-1,1):p.append(paddle('Hair side / rooted short golden baby side lock '+str(s),BABY_HAIR,[(s*.219,.943,-.037),(s*.274,.807,.022),(s*.281,.624,.048),(s*.260,.446,.067),(s*.221,.427,.076)],[.067,.064,.063,.058,.034],[.084,.094,.098,.083,.037],segments=24,steps=4))
 tips=[[(0,1.074,-.004),(-.152,1.173,.006),(-.270,1.222,.004)],[(.010,1.077,-.024),(.163,1.193,-.022),(.271,1.261,-.009)],[(0,1.082,-.003),(-.046,1.220,.010),(-.073,1.291,.003)],[(.010,1.078,-.045),(.040,1.198,-.052),(.072,1.260,-.058)]]
 for j,path in enumerate(tips):p.append(paddle('Hair tuft / rooted golden baby leaf '+str(j),BABY_HAIR,path,[.029,.045 if j<2 else .029,.003],[.022,.025,.003],segments=20,steps=4))

def smoochum_limbs(p):
 for s in (-1,1):
  p.append(paddle('Leg / rooted short cream baby support '+str(s),CREAM,[(s*.088,.139,-.006),(s*.118,.083,-.004),(s*.128,.044,-.002)],[.055,.047,.037],[.053,.044,.035],segments=20,steps=4))
  p.append(ellipsoid('Foot / grounded rounded cream baby toe '+str(s),CREAM,(s*.136,.035,.033),(.066,.035,.083),28,20))
  p.append(paddle('Arm / rooted short curved cream baby forelimb '+str(s),CREAM,[(s*.127,.375,.017),(s*.222,.331,.040),(s*.231,.269,.067)],[.052,.044,.036],[.049,.043,.035],segments=20,steps=4))
  p.append(ellipsoid('Hand / small cream three-finger baby palm '+str(s),CREAM,(s*.226,.250,.074),(.038,.039,.032),20,18))
  for j in range(3):
   x=s*(.204+.021*j);p.append(curved_tube(f'Finger / rooted small cream baby digit {s} {j}',CREAM,[(x,.239,.086),(x+s*.004,.212-.004*(j==1),.101),(x+s*.007,.199,.100)],[.013,.011,.005],20))

def smoochum():
 body=ellipsoid('Body / compact pink baby pear',PINK,(0,.232,-.006),(.176,.153,.143),36,24)
 head=recessed(ellipsoid('Head / broad rounded pink baby face',PINK,(0,.730,.026),(.260,.287,.214),56,40),[(0,.575,.096,.070,.055)])
 # A short throat reaches the large head without an unsupported gap.
 neck=ellipsoid('Neck / short rooted cream baby throat',CREAM,(0,.424,.018),(.096,.076,.094),24,20)
 p=[body,neck,head]
 chest=ellipsoid('Chest / rooted broad cream baby collar',CREAM,(0,.382,.015),(.212,.052,.156),36,24);p.append(chest)
 p.append(paint('Chest bib / fitted pointed cream baby breast',CREAM,[(-.102,.355),(.102,.355),(.112,.319),(.059,.292),(0,.281),(-.059,.292),(-.112,.319)],body,.006,3))
 eyes(p,head,True);lips(p,head,0,.575,.061,.028,True);smoochum_hair(p);smoochum_limbs(p)
 return p

def sculpture(name,normalize=True):
 if name not in SPECIES:raise ValueError(name)
 p=jynx() if name=='jynx' else smoochum();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[name]/(hi-lo)
 if name=='smoochum':
  from sculpt_geometry import linear
  for q in p:
   n=q['part'];rgb=(0,0,0) if n.startswith(('Eye socket /','Pupil /','Mouth /')) else (255,255,255) if n.startswith(('Eye /','Glint /','Neck /','Chest /','Chest bib /','Arm /','Hand /','Finger /','Leg /','Foot /')) else (239,198,33) if n.startswith(('Hair','Bangs /','Lips /')) else (165,8,156)
   q['base_color']=linear(tuple(v/255 for v in rgb))
 if name=='jynx':
  from sculpt_geometry import linear
  for q in p:
   n=q['part'];rgb=(0,0,0) if n.startswith(('Eye socket /','Pupil /','Mouth /')) else (255,255,255) if n.startswith(('Eye /','Hair','Back hair strand /','Chest trim /','Chest bridge /','Sleeve /')) else (239,49,156) if n.startswith(('Gown /','Bodice /','Lips /')) else (115,16,123)
   q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) or 0. for v in q['normals']]
 return {'name':name,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from jynx_glb import export_jynx
 from smoochum_glb import export_smoochum
 for n,export in (('jynx',export_jynx),('smoochum',export_smoochum)):
  (args.output/(n+'.glb')).write_bytes(export());(args.output/(n+'.mesh.json')).unlink(missing_ok=True)
if __name__=='__main__':main()
