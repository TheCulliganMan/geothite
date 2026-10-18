#!/usr/bin/env python3
"""Original distinct flowing-gown Jynx and bob-haired Smoochum sculptures."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals
from bovine_sculpt import oval,curved_tube,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import sheet,paint
from goldfish_sculpt import surface
from skin_glb import COORDINATES
SPECIES=('jynx','smoochum');HEIGHTS={'jynx':1.28,'smoochum':.78}
PURPLE=(.48,.35,.61);PINK=(.78,.44,.61);RED=(.84,.30,.29)
GOLD=(.95,.82,.36);HAIR=(.95,.91,.62);BABY_HAIR=(.96,.89,.45)
CREAM=(.98,.94,.75);WHITE=(.96,.97,.94);INK=(.08,.065,.10)
LIP=(.88,.54,.60);GREEN=(.20,.44,.37)

def lips(p,head,cx,cy,rx,ry,baby=False):
 at=surface(head);vv=[];nn=48 if baby else 28;mm=20 if baby else 12;minor=.023 if ry>.07 else .020
 for i in range(nn):
  a=math.tau*i/nn
  for j in range(mm):
   b=math.tau*j/mm;x=cx+(rx+minor*math.cos(b))*math.cos(a);y=cy+(ry+minor*math.cos(b))*math.sin(a)
   vv.append((x,y,at(y,x)+.010+minor*math.sin(b)))
 ff=[]
 for i in range(nn):
  for j in range(mm):ff.append((i*mm+j,((i+1)%nn)*mm+j,((i+1)%nn)*mm+(j+1)%mm,i*mm+(j+1)%mm))
 lip=crease_normals(part('Lips / rooted pink oval kiss rim',LIP,vv,ff,True),45);p.append(lip)
 p.append(paint('Mouth / fitted dark recessed oval kiss opening',INK,oval(cx,cy,rx-minor+.005,ry-minor+.005,40 if baby else 24),head,.004,1))

def eyes(p,head,baby):
 cy=.972 if baby else 1.102;cx=.110 if baby else .089;rx=.063 if baby else .048;ry=.079 if baby else .058
 for s in (-1,1):
  socket=paint('Eye socket / fitted dark ice-dancer recess '+str(s),INK,oval(s*cx,cy,rx+.006,ry+.006,36 if baby else 24),head,.005,1);p.append(socket)
  eye=paint('Eye / fitted ivory ice-dancer gaze '+str(s),WHITE,oval(s*cx,cy,rx,ry,36 if baby else 24),socket,.004,1);p.append(eye)
  if baby:
   pupil=paint('Pupil / fitted broad dark baby stare '+str(s),INK,oval(s*cx-.008,cy+.006,.039,.056,32),eye,.003,1);p.append(pupil)
   iris=paint('Iris / fitted green lower baby gaze '+str(s),GREEN,oval(s*cx-.008,cy-.018,.032,.025,28),pupil,.003,1);p.append(iris)
   p.append(paint('Glint / fitted ivory baby eye spark '+str(s),WHITE,oval(s*cx-.016,cy+.037,.010,.014,24),pupil,.004,1))
  else:p.append(paint('Pupil / fitted tiny dark dancer stare '+str(s),INK,oval(s*cx,cy,.012,.018,20),eye,.003,1))

def gown():
 rows=[(.009,.418,.293),(.085,.381,.279),(.263,.313,.239),(.476,.218,.174),(.664,.170,.140),(.798,.201,.151),(.912,.109,.093)];n=64;vv=[]
 rows=curved_profiles([(y,w,d,-.015) for y,w,d in rows],3)
 for y,w,d,z in rows:
  fade=max(0,1-y/.68)
  for i in range(n):
   a=math.tau*i/n;fold=1+.078*fade*math.cos(7*a);vv.append((w*math.sin(a)*fold,y,-.015+d*math.cos(a)*fold))
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
 back=crease_normals(loft('Hair back / rooted long ivory dancer mantle',HAIR,curved_profiles(((.640,.134,.065,-.080),(.720,.234,.110,-.105),(.860,.284,.151,-.113),(1.020,.280,.163,-.100),(1.170,.253,.166,-.085),(1.300,.196,.144,-.060),(1.380,.069,.061,-.020)),3),32),45);p.append(back)
 dome=ellipsoid('Hair crown / rooted ivory center-parted dancer dome',HAIR,(0,1.316,-.055),(.222,.104,.126),28,18);p.append(dome)
 for j in (-1,0,1):
  x=j*.135;xy=[(x-.004,.813),(x-.014,.976),(x-.016,1.151),(x-.010,1.247),(x-.004,1.247),(x-.010,1.151),(x-.008,.976),(x+.002,.813)]
  strand=jynx_mark('Back hair strand / fitted subtle ivory dancer groove '+str(j),(.84,.80,.52),xy,reverse_front(back),.003,1);p.append(reverse_front(strand))
 for s in (-1,1):
  p.append(paddle('Hair part / rooted arched ivory dancer forelock '+str(s),HAIR,[(0,1.286,.097),(s*.061,1.373,.050),(s*.119,1.395,.018),(s*.183,1.368,-.002),(s*.228,1.311,.022)],[.003,.041,.043,.046,.050],[.003,.035,.037,.041,.044],segments=18,steps=4))
  p.append(paddle('Hair curtain / rooted sweeping ivory dancer lock '+str(s),HAIR,[(s*.132,1.346,.023),(s*.238,1.226,.102),(s*.280,.996,.138),(s*.258,.795,.192),(s*.146,.646,.247)],[.090,.091,.091,.078,.008],[.061,.065,.064,.057,.007],segments=18,steps=4))
  curtain=p[-1];xy=[(s*x,y) for x,y in ((.181,.697),(.252,.865),(.279,1.017),(.246,1.214),(.239,1.214),(.272,1.017),(.245,.865),(.174,.697))]
  p.append(jynx_mark('Hair strand / fitted subtle ivory curtain groove '+str(s),(.84,.80,.52),xy,curtain,.003,1))
  for j in range(2):
   p.append(paddle(f'Hair tip / rooted layered ivory dancer point {s} {j}',HAIR,[(s*(.246+.035*j),.878,.151-.015*j),(s*(.260+.034*j),.715,.122-.051*j),(s*(.208+.064*j),.605+.017*j,.185-.044*j)],[.057,.050,.003],[.045,.037,.003],segments=16,steps=4))

def jynx_arm_path(s):return [(s*.130,.821,-.012),(s*.282,.714,.026),(s*.421,.771,.094)]

def jynx_digit_paths(s):
 return [([(s*.411,.836,.132),(s*.369,.890,.146),(s*.358,.915,.145)],[.024,.022,.011]),
         ([(s*.436,.869,.128),(s*.435,.946,.137),(s*.449,.966,.131)],[.022,.020,.010]),
         ([(s*.475,.872,.126),(s*.505,.947,.137),(s*.522,.960,.132)],[.023,.020,.010]),
         ([(s*.501,.834,.125),(s*.572,.869,.139),(s*.589,.861,.133)],[.022,.019,.009]),
         ([(s*.495,.789,.126),(s*.562,.783,.138),(s*.578,.793,.136)],[.022,.019,.009])]

def jynx_limbs(p):
 for s in (-1,1):
  p.append(paddle('Sleeve / rooted bent white dancer arm '+str(s),WHITE,jynx_arm_path(s),[.074,.078,.056],[.081,.076,.055],segments=18,steps=4))
  hand=ellipsoid('Palm / open purple five-finger dancer hand '+str(s),PURPLE,(s*.457,.811,.126),(.071,.079,.048),24,16);p.append(hand)
  paths=jynx_digit_paths(s)
  for j,(path,radii) in enumerate(paths):p.append(curved_tube(f'Finger / rooted purple dancer digit {s} {j}',PURPLE,path,radii,16))

def jynx():
 dress=gown();neck=ellipsoid('Neck / rooted purple dancer throat',PURPLE,(0,.937,.018),(.108,.118,.099),24,16)
 head=ellipsoid('Head / broad purple dancer face',PURPLE,(0,1.087,.044),(.219,.221,.180),36,24);p=[dress,neck,head]
 for s in (-1,1):
  plate=ellipsoid('Chest trim / rooted white dancer bodice lobe '+str(s),GOLD,(s*.099,.814,.128),(.112,.109,.086),28,18);p.append(plate)
  p.append(paint('Bodice / fitted pink dancer chest inset '+str(s),RED,oval(s*.099,.814,.087,.080,24),plate,.006,2))
 bridge=ellipsoid('Chest bridge / rooted white dancer central waist trim',GOLD,(0,.774,.140),(.053,.074,.047),24,16);p.append(bridge)
 eyes(p,head,False);lips(p,head,0,.985,.055,.035);jynx_hair(p);jynx_limbs(p)
 return p

def smoochum_hair(p):
 back=ellipsoid('Hair back / rooted short golden baby bob',BABY_HAIR,(0,1.049,-.078),(.287,.226,.194),28,28);p.append(back)
 cap=ellipsoid('Hair crown / rooted golden baby dome',BABY_HAIR,(0,1.183,.007),(.275,.147,.207),28,28);p.append(cap)
 xy=[(-.293,1.034),(-.255,1.145),(-.190,1.251),(-.078,1.310),(.040,1.322),(.181,1.261),(.259,1.174),(.290,1.085),(.190,1.052),(.042,1.049),(.012,1.123),(-.014,1.044),(-.173,1.024)]
 p.append(sheet('Bangs / continuous golden baby fringe with center notch',BABY_HAIR,xy,lambda y,x:.215-.70*x*x-.22*(y-1.16)**2,.033,3))
 for s in (-1,1):p.append(paddle('Hair side / rooted short golden baby side lock '+str(s),BABY_HAIR,[(s*.185,1.158,-.039),(s*.255,1.051,.013),(s*.264,.966,.023),(s*.242,.885,.041),(s*.184,.824,.072),(s*.135,.808,.080)],[.045,.055,.057,.047,.024,.003],[.044,.048,.049,.041,.021,.003],segments=20,steps=4))
 tips=[[(0,1.283,-.003),(-.153,1.375,.001),(-.259,1.434,.000)],[(.012,1.290,-.021),(.179,1.421,-.018),(.259,1.499,-.004)],[(0,1.299,-.003),(-.041,1.401,.008),(-.054,1.446,.000)],[(.010,1.293,-.045),(.036,1.420,-.039),(.044,1.466,-.027)]]
 for j,path in enumerate(tips):p.append(paddle('Hair tuft / rooted golden baby leaf '+str(j),BABY_HAIR,path,[.025,.040,.002],[.019,.021,.002],segments=28 if j==1 else 20,steps=6 if j==1 else 4))

def smoochum_limbs(p):
 for s in (-1,1):
  p.append(paddle('Leg / rooted short cream baby support '+str(s),CREAM,[(s*.092,.212,-.013),(s*.112,.110,.018),(s*.138,.041,.046)],[.047,.041,.034],[.047,.039,.032]))
  p.append(ellipsoid('Foot / grounded rounded cream baby toe '+str(s),CREAM,(s*.145,.035,.060),(.052,.035,.074),28,24))
  p.append(paddle('Arm / rooted short curved cream baby forelimb '+str(s),CREAM,[(s*.112,.533,.017),(s*.217,.488,.042),(s*.227,.413,.080)],[.054,.047,.040],[.049,.047,.038]))
  p.append(ellipsoid('Hand / small cream three-finger baby palm '+str(s),CREAM,(s*.219,.398,.088),(.039,.041,.033),20,22))
  for j in range(3):
   x=s*(.196+.021*j);p.append(curved_tube(f'Finger / rooted small cream baby digit {s} {j}',CREAM,[(x,.387,.099),(x+s*.004,.359-.004*(j==1),.110),(x+s*.007,.346,.110)],[.013,.011,.005],24))

def smoochum():
 body=ellipsoid('Body / compact pink baby pear',PINK,(0,.365,-.006),(.194,.230,.154),44,28)
 head=ellipsoid('Head / broad rounded pink baby face',PINK,(0,.976,.028),(.237,.234,.181),48,30)
 # A short throat reaches the large head without an unsupported gap.
 neck=ellipsoid('Neck / short rooted cream baby throat',CREAM,(0,.628,.034),(.091,.090,.090),24,24)
 p=[body,neck,head]
 chest=ellipsoid('Chest / rooted broad cream baby collar',CREAM,(0,.536,.022),(.216,.093,.152),44,28);p.append(chest)
 p.append(paint('Chest bib / fitted pointed cream baby breast',CREAM,[(-.122,.533),(.122,.533),(.128,.468),(.068,.436),(0,.425),(-.068,.436),(-.128,.468)],body,.008,3))
 eyes(p,head,True);lips(p,head,0,.869,.037,.060,True);smoochum_hair(p);smoochum_limbs(p)
 for q in p:
  if q['part'].startswith(('Head /','Eye socket /','Eye /','Pupil /','Iris /','Glint /','Lips /','Mouth /','Hair','Bangs /')):q['positions']=[v-(.110 if i%3==1 else 0) for i,v in enumerate(q['positions'])]
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
  for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':name,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from jynx_glb import export_jynx
 from smoochum_glb import export_smoochum
 for n,export in (('jynx',export_jynx),('smoochum',export_smoochum)):
  (args.output/(n+'.glb')).write_bytes(export());(args.output/(n+'.mesh.json')).unlink(missing_ok=True)
if __name__=='__main__':main()
