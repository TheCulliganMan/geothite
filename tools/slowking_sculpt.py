#!/usr/bin/env python3
"""Original royal pink biped with a flared collar and horned spiral crown."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,part,translate,crease_normals
from bovine_sculpt import curved_tube,oval
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.25
PINK=(.90,.56,.62);CREAM=(.96,.91,.70);WHITE=(.99,.98,.95);INK=(.085,.075,.095);RED=(.87,.31,.24)
SHELL=(.77,.81,.81);LIGHT=(.88,.90,.87);SHADE=(.64,.70,.71);RUBY=(.86,.30,.18)
CROWN_ROWS=[(1.405,.215,.164,-.024),(1.49,.244,.195,-.025),(1.61,.212,.177,-.038),(1.72,.146,.123,-.053),(1.81,.073,.069,-.060),(1.825,.045,.044,-.062)]

def collar_panel(j):
 n=10;theta=math.tau*j/n;span=math.tau/n*.97;verts=[];steps=7
 # The buried inner edge makes every flared cloth sector root into the neck.
 for layer in (-.015,.015):
  for t in (0,.33,.67,1):
   for i in range(steps):
    a=theta+span*(i/(steps-1)-.5);rx=.092+(.344-.092)*t;rz=.088+(.250-.088)*t
    y=1.048-.128*t+.025*math.sin(math.pi*t)-.014*math.cos(a)
    verts.append((rx*math.sin(a),y+layer,rz*math.cos(a)))
 count=len(verts)//2;faces=[]
 for r in range(3):
  for i in range(steps-1):
   a=r*steps+i;b=a+1;faces.extend(((a,b,b+steps,a+steps),(a+count+steps,b+count+steps,b+count,a+count)))
 boundary=list(range(steps))+[r*steps+steps-1 for r in range(1,4)]+list(reversed(range(3*steps,4*steps-1)))+[r*steps for r in (2,1)]
 for a,b in zip(boundary,boundary[1:]+boundary[:1]):faces.append((b,a,a+count,b+count))
 return crease_normals(part('Collar panel / rooted alternating royal ruff '+str(j),RED if j%2==0 else WHITE,verts,faces),35)

def crown_turn(j,start,end):
 rows=curved_profiles(CROWN_ROWS,6);selected=[r for r in rows if start<=r[0]<=end];n=40;verts=[]
 for offset in (.009,-.017):
  for y,rx,rz,z in selected:
   for i in range(n):
    a=math.tau*i/n;yy=y+.018*math.sin(a)+.009*math.cos(a);left,right=next((a,b) for a,b in zip(rows,rows[1:]) if a[0]<=yy<=b[0]);t=(yy-left[0])/(right[0]-left[0]);rx,rz,z=[left[k]+t*(right[k]-left[k]) for k in (1,2,3)]
    verts.append(((rx+offset)*math.sin(a),yy,z+(rz+offset)*math.cos(a)))
 count=len(selected);inner=count*n;faces=[]
 for k in range(count-1):
  for i in range(n):
   a=k*n+i;b=k*n+(i+1)%n;faces.extend(((a,b,b+n,a+n),(a+inner+n,b+inner+n,b+inner,a+inner)))
 for i in range(n):
  a=i;b=(i+1)%n;faces.append((b,a,a+inner,b+inner));a=(count-1)*n+i;b=(count-1)*n+(i+1)%n;faces.append((a,b,b+inner,a+inner))
 return part('Crown turn / fitted slanted overlapping shell turn '+str(j),LIGHT,verts,faces,True)

def anatomy():
 body=loft('Body / slender pink Slowking pear torso and neck',PINK,curved_profiles(((.19,.13,.124,-.028),(.35,.265,.224,-.025),(.53,.281,.222,-.010),(.73,.236,.18,.005),(.89,.155,.132,.01),(1.035,.120,.111,.017),(1.135,.132,.125,.020)),5),40)
 head=ellipsoid('Head / rooted rounded pink Slowking face',PINK,(0,1.239,.020),(.215,.213,.195),40,26)
 muzzle=ellipsoid('Muzzle / rooted broad cream Slowking snout',CREAM,(0,1.168,.220),(.215,.076,.151),40,26)
 p=[body,head,muzzle]
 for j,(low,high,w) in enumerate(((.24,.325,.148),(.337,.435,.211),(.447,.542,.225))):
  p.append(paint('Belly band / fitted cream lower Slowking abdomen '+str(j),CREAM,[(-w*.76,low),(-w,low+.03),(-w,high-.025),(-w*.84,high),(w*.84,high),(w,high-.025),(w,low+.03),(w*.76,low)],body,.011,3))
 for j in range(10):p.append(collar_panel(j))
 for s in (-1,1):
  eye=paint('Eye / fitted tall white Slowking gaze '+str(s),WHITE,oval(s*.113,1.312,.041,.083,36),head,.008,3);p.append(eye)
  p.append(paint('Pupil / fitted tiny black Slowking dot '+str(s),INK,oval(s*.117,1.326,.005,.008,20),eye,.003,2))
  p.append(paint('Nostril / fitted subtle Slowking nostril '+str(s),INK,oval(s*.041,1.209,.0035,.005,16),muzzle,.0025,2))
  p.append(curved_tube('Tooth / rooted small ivory Slowking canine '+str(s),WHITE,[(s*.146,1.126,.270),(s*.147,1.102,.28),(s*.142,1.088,.282)],[.013,.010,.0015],20))
  p.append(ellipsoid('Shoulder / rooted rounded pink Slowking shoulder '+str(s),PINK,(s*.183,.849,.021),(.095,.108,.097),32,22))
  p.append(paddle('Arm / short bent pink Slowking forelimb '+str(s),PINK,[(s*.180,.859,.044),(s*.261,.745,.106),(s*.204,.653,.193)],[.091,.082,.068],[.095,.087,.071]))
  hand=ellipsoid('Hand / rooted rounded pink Slowking palm '+str(s),PINK,(s*.183,.660,.214),(.081,.086,.077),32,22);p.append(hand)
  for j,dx in enumerate((-.046,0,.046)):
   p.append(curved_tube(f'Hand claw / three rooted white Slowking digits {s} {j}',WHITE,[(s*.183+dx*.80,.626,.257),(s*.183+dx,.598,.287),(s*.183+dx,.590,.307)],[.017,.012,.0015],20))
  p.append(ellipsoid('Thigh / rooted round pink Slowking haunch '+str(s),PINK,(s*.221,.289,.025),(.169,.213,.177),36,24))
  p.append(paddle('Leg / short rooted pink Slowking ankle '+str(s),PINK,[(s*.236,.235,.038),(s*.263,.11,.104),(s*.271,.07,.125)],[.090,.087,.084],[.102,.096,.075]))
  p.append(translate(loft('Foot / grounded pink Slowking support '+str(s),PINK,[(0,.082,.087,.126),(.036,.114,.135,.147),(.078,.118,.125,.136),(.116,.091,.087,.109)],32),s*.273,0,0))
  for j,dx in enumerate((-.054,.054)):
   p.append(paddle(f'Foot claw / two rooted white Slowking toes {s} {j}',WHITE,[(s*.273+dx,.05,.224),(s*.273+dx,.027,.282),(s*.273+dx,.012,.300)],[.033,.023,.005],[.023,.016,.0015]))
 seam=[(-.165,1.146),(-.084,1.129),(0,1.123),(.084,1.129),(.165,1.146),(.137,1.134),(.082,1.122),(0,1.115),(-.082,1.122),(-.137,1.134)]
 p.append(paint('Mouth seam / fitted curved Slowking smile',INK,seam,muzzle,.003,3))
 p.append(curved_tube('Tail / rooted thick tapered upturned pink Slowking tail',PINK,[(0,.282,-.158),(.13,.245,-.365),(.285,.29,-.595),(.399,.402,-.784),(.447,.526,-.878)],[.145,.142,.12,.076,.002],36))
 crown=loft('Crown body / rooted grey spiral royal shell',SHELL,curved_profiles(CROWN_ROWS,6),40);p.append(crown)
 p.append(ellipsoid('Crown brim / rooted broad pale shell brim',LIGHT,(0,1.427,-.012),(.28,.073,.214),40,24))
 for j,(a,b) in enumerate(((1.495,1.545),(1.67,1.707))):p.append(crown_turn(j,a,b))
 for s in (-1,1):
  p.append(paddle('Crown horn / rooted curved lateral shell horn '+str(s),LIGHT,[(s*.199,1.44,-.024),(s*.345,1.465,-.054),(s*.421,1.568,-.080),(s*.448,1.691,-.091)],[.090,.082,.053,.002],[.082,.070,.043,.0015]))
 for j,(x,y,z,tx,ty,tz,r) in enumerate(((-.082,1.748,-.050,-.172,1.95,-.055,.054),(.075,1.75,-.052,.193,1.96,-.056,.054),(-.025,1.789,-.060,-.057,2.03,-.056,.056),(.015,1.777,-.079,.076,1.955,-.184,.051),(0,1.771,-.024,.019,1.914,.071,.052))):
  p.append(curved_tube('Crown crest / rooted irregular upper shell point '+str(j),SHELL,[(x,y,z),((x+tx)/2,(y+ty)/2,(z+tz)/2),(tx,ty,tz)],[r,r*.62,.0015],24))
 rim=paint('Jewel rim / fitted pale circular royal setting',LIGHT,oval(0,1.708,.064,.066,40),crown,.010,3);p.append(rim)
 ruby=paint('Jewel / fitted red round Slowking ruby',RUBY,oval(0,1.708,.048,.049,40),rim,.007,3);p.append(ruby)
 p.append(paint('Jewel glint / fitted ivory ruby gleam',WHITE,oval(-.014,1.727,.010,.009,20),ruby,.003,2))
 return p

def sculpture():
 p=anatomy();low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:
  q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'slowking','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'slowking.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
