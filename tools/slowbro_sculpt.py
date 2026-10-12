#!/usr/bin/env python3
"""Original pink shell-tailed biped with a separate toothed spiral creature."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,part,translate
from bovine_sculpt import curved_tube,oval
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from murkrow_sculpt import bill
from skin_glb import COORDINATES
HEIGHT=1.20
PINK=(.90,.56,.64);INNER=(.73,.39,.49);CREAM=(.95,.90,.68);WHITE=(.99,.98,.94);INK=(.08,.07,.10);TONGUE=(.85,.49,.64)
SHELL=(.65,.73,.77);RIDGE=(.73,.80,.82);SHADE=(.44,.53,.61)
SHELL_ROWS=[(.48,.19,.17,-.54),(.57,.25,.215,-.56),(.73,.225,.205,-.59),(.87,.175,.165,-.62),(1.01,.125,.125,-.65),(1.13,.075,.08,-.68),(1.20,.018,.024,-.69)]
SHELL_X=.37

def shell_rim(j,start,end):
 rows=curved_profiles(SHELL_ROWS,6);selected=[r for r in rows if start<=r[0]<=end];n=40;vv=[]
 for offset in (.009,-.017):
  for y,rx,rz,z in selected:
   for i in range(n):
    angle=math.tau*i/n;yy=y+.028*math.sin(angle)+.016*math.cos(angle)
    left,right=next((a,b) for a,b in zip(rows,rows[1:]) if a[0]<=yy<=b[0]);t=(yy-left[0])/(right[0]-left[0]);rrx,rrz,zz=[left[k]+t*(right[k]-left[k]) for k in (1,2,3)]
    vv.append((SHELL_X+(rrx+offset)*math.sin(angle),yy,zz+(rrz+offset)*math.cos(angle)))
 count=len(selected);inner=count*n;ff=[]
 for k in range(count-1):
  for i in range(n):
   a=k*n+i;b=k*n+(i+1)%n;ff.extend(((a,b,b+n,a+n),(a+inner+n,b+inner+n,b+inner,a+inner)))
 for i in range(n):
  a=i;b=(i+1)%n;ff.append((b,a,a+inner,b+inner));a=(count-1)*n+i;b=(count-1)*n+(i+1)%n;ff.append((a,b,b+inner,a+inner))
 return part('Shell coil / fitted overlapping grey-blue turn '+str(j),RIDGE,vv,ff,True)

def anatomy():
 body=loft('Body / continuous pink Slowbro head neck and pear torso',PINK,curved_profiles(((.19,.14,.14,-.015),(.35,.31,.255,-.015),(.57,.345,.265,0),(.80,.29,.235,.015),(1.00,.235,.218,.030),(1.19,.275,.24,.048),(1.35,.258,.22,.050),(1.47,.15,.14,.050),(1.49,.025,.026,.050)),5),48)
 upper=ellipsoid('Muzzle / broad rounded cream Slowbro upper lip',CREAM,(0,1.155,.345),(.267,.087,.179),40,26)
 jaw=ellipsoid('Jaw / open rounded cream Slowbro lower lip',CREAM,(0,.967,.326),(.218,.065,.166),40,26)
 back=ellipsoid('Mouth back / recessed dark Slowbro oral chamber',INK,(0,1.045,.270),(.167,.113,.038),32,22)
 tongue=ellipsoid('Tongue / rooted pink Slowbro tongue',TONGUE,(0,1.015,.382),(.131,.019,.083),32,20)
 p=[body,upper,jaw,back,tongue]
 for j,(low,high,w) in enumerate(((.23,.325,.164),(.337,.435,.255),(.447,.548,.279),(.560,.67,.274))):
  p.append(paint('Belly band / fitted segmented cream Slowbro abdomen '+str(j),CREAM,[(-w*.75,low),(-w,low+.032),(-w,high-.025),(-w*.85,high),(w*.85,high),(w,high-.025),(w,low+.032),(w*.75,low)],body,.013,3))
 for s in (-1,1):
  ear=ellipsoid('Ear / rooted round pink Slowbro ear '+str(s),PINK,(s*.209,1.444,.022),(.098,.094,.067),32,22);p.append(ear)
  inset=paint('Ear inset / fitted recessed pink pinna '+str(s),INNER,oval(s*.214,1.448,.049,.050,32),ear,.005,3);p.append(inset)
  p.append(paint('Ear curl / fitted curved inner ear fold '+str(s),PINK,[(s*.181,1.425),(s*.190,1.458),(s*.219,1.471),(s*.241,1.452),(s*.232,1.447),(s*.217,1.457),(s*.199,1.446),(s*.193,1.424)],inset,.004,3))
  eye=paint('Eye / fitted tall white Slowbro stare '+str(s),WHITE,oval(s*.147,1.302,.054,.094,40),body,.010,3);p.append(eye)
  p.append(paint('Pupil / fitted tiny black Slowbro dot '+str(s),INK,oval(s*.151,1.319,.0055,.009,20),eye,.003,2))
  p.append(paint('Nostril / fitted subtle Slowbro nostril '+str(s),INNER,oval(s*.052,1.193,.005,.009,20),upper,.003,2))
  p.append(curved_tube('Fang / rooted small upper Slowbro canine '+str(s),WHITE,[(s*.179,1.110,.386),(s*.180,1.057,.400),(s*.174,1.037,.406)],[.016,.012,.0015],20))
  p.append(ellipsoid('Shoulder / rooted rounded pink Slowbro shoulder '+str(s),PINK,(s*.255,.817,.084),(.117,.124,.111),32,22))
  p.append(paddle('Arm / hanging rooted pink Slowbro forelimb '+str(s),PINK,[(s*.238,.854,.085),(s*.329,.718,.15),(s*.281,.617,.245)],[.111,.104,.083],[.115,.106,.087]))
  hand=ellipsoid('Hand / short rounded clawed Slowbro palm '+str(s),PINK,(s*.270,.613,.260),(.095,.114,.091),32,22);p.append(hand)
  for j,dx in enumerate((-.052,0,.052)):
   p.append(curved_tube(f'Hand claw / three rooted white Slowbro digits {s} {j}',WHITE,[(s*.270+dx*.80,.566,.312),(s*.270+dx,.529,.347),(s*.270+dx,.515,.372)],[.021,.016,.0015],20))
  thigh=ellipsoid('Thigh / rooted heavy pink Slowbro haunch '+str(s),PINK,(s*.277,.318,.034),(.193,.236,.199),36,24);p.append(thigh)
  p.append(paddle('Leg / short rooted Slowbro ankle '+str(s),PINK,[(s*.291,.250,.04),(s*.310,.125,.107),(s*.31,.073,.141)],[.104,.10,.095],[.121,.111,.083]))
  p.append(translate(loft('Foot / grounded broad pink Slowbro foot '+str(s),PINK,[(0,.105,.112,.149),(.035,.139,.156,.168),(.078,.14,.147,.15),(.125,.105,.105,.113)],32),s*.31,0,0))
  for j,dx in enumerate((-.070,.070)):
   p.append(paddle(f'Foot claw / two rooted white Slowbro toes {s} {j}',WHITE,[(s*.31+dx,.060,.262),(s*.31+dx,.035,.338),(s*.31+dx,.014,.365)],[.047,.035,.008],[.031,.021,.002]))
 tail=curved_tube('Tail / rooted thick pink Slowbro tail gripped by shell',PINK,[(0,.30,-.163),(.17,.245,-.340),(.30,.355,-.440),(.37,.436,-.510)],[.147,.115,.080,.077],36);p.append(tail)
 shell=translate(loft('Shell body / separate tapering grey-blue Shellder spiral',SHELL,curved_profiles(SHELL_ROWS,6),40),SHELL_X,0,0);p.append(shell)
 for j,(a,b) in enumerate(((.56,.64),(.76,.84),(.95,1.015))):p.append(shell_rim(j,a,b))
 # The lower lip is physically separate from the cone's upper biting lip.
 shell_jaw=translate(bill('Shell jaw / rooted dark grey-blue lower biting lip',SHADE,[(-.69,.12,.29,.51),(-.52,.19,.267,.332),(-.35,.15,.294,.347),(-.30,.068,.327,.363)]),SHELL_X,0,0);p.append(shell_jaw)
 p.append(ellipsoid('Shell rear wall / rooted grey-blue rear biting shell',SHELL,(SHELL_X,.429,-.692),(.155,.135,.040),32,22))
 p.append(ellipsoid('Shell mouth back / recessed dark shell oral chamber',INK,(SHELL_X,.429,-.650),(.145,.142,.048),28,20))
 for j,dx in enumerate((-.13,-.065,.065,.13)):
  p.append(curved_tube('Shell upper tooth / rooted shell fang '+str(j),WHITE,[(SHELL_X+dx,.508,-.410),(SHELL_X+dx,.459,-.397),(SHELL_X+dx,.432,-.387)],[.022,.015,.0015],20))
  p.append(curved_tube('Shell lower tooth / rooted shell fang '+str(j),WHITE,[(SHELL_X+dx,.326,-.375),(SHELL_X+dx,.367,-.368),(SHELL_X+dx,.391,-.371)],[.021,.014,.0015],20))
 for s in (-1,1):
  outline=[(SHELL_X+s*x,y) for x,y in ((.023,.596),(.153,.641),(.164,.586),(.067,.567))]
  socket=paint('Shell socket / fitted angular dark shell gaze '+str(s),INK,outline,shell,.009,3);p.append(socket);cx=sum(x for x,y in outline)/4;cy=sum(y for x,y in outline)/4
  eye=paint('Shell eye / fitted narrow white shell glare '+str(s),WHITE,[(cx+.8*(x-cx),cy+.72*(y-cy)) for x,y in outline],socket,.005,3);p.append(eye)
  p.append(paint('Shell pupil / fitted tiny dark shell slit '+str(s),INK,oval(cx,cy,.004,.008,16),eye,.003,2))
 for j,(x,y,z,tx,ty,tz,r) in enumerate(((.58,.61,-.57,.74,.64,-.61,.052),(.57,.76,-.63,.75,.84,-.68,.057),(.50,.93,-.65,.64,1.03,-.72,.047),(.36,1.13,-.67,.39,1.37,-.72,.061),(.19,.76,-.62,.07,.85,-.67,.051),(.22,.94,-.66,.13,1.06,-.73,.047),(.38,.69,-.77,.38,.77,-.98,.061))):
  p.append(curved_tube('Shell spine / rooted tapered grey-blue shell point '+str(j),RIDGE,[(x,y,z),((x+tx)/2,(y+ty)/2,(z+tz)/2),(tx,ty,tz)],[r,r*.65,.0015],24))
 return p

def sculpture():
 p=anatomy();low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:
  q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'slowbro','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'slowbro.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
