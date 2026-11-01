#!/usr/bin/env python3
"""Original armored rhinoceros; every dorsal plate seats inside its barrel."""
import argparse,json,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,translate,crease_normals,fitted_triangle_contour
from bovine_sculpt import curved_tube,oval
from abra_sculpt import surface
from bulldog_sculpt import rounded_loft
from cubone_sculpt import skin_shading
from skin_glb import COORDINATES
HEIGHT=1.11
GRAY=(.64,.67,.69);PLATE=(.70,.73,.74);SHADOW=(.47,.51,.54);INK=(.09,.12,.14);IVORY=(.93,.94,.89);RED=(.73,.23,.20)

def longitudinal(name,color,profiles,segments=40,steps=4):
 p=loft(name,color,curved_profiles([(z,rx,ry,-y) for z,y,rx,ry in profiles],steps),segments)
 for key in ('positions','normals'):
  p[key]=[v for i in range(0,len(p[key]),3) for v in (p[key][i],-p[key][i+2],p[key][i+1])]
 return p

def paint(name,color,xy,parent,th=.004):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),45),parent)

def armor(name,z,width,rise,crest,depth):
 # Closed thick shields wrap over the back and down both flanks. Their
 # scalloped longitudinal edges and raised crests belong to one solid band.
 angles=(-2.12,-1.85,-1.58,-1.32,-1.05,-.80,-.52,-.26,0.,.26,.52,.80,1.05,1.32,1.58,1.85,2.12);n=len(angles);v=[]
 sections=(-1.,0.,1.)
 for inside in (False,True):
  for u in sections:
   for i in range(n):
    a=angles[i];factor=.90 if inside else 1.;peak=0. if inside else (1.-abs(u))*crest*max(0.,1-abs(a)/.62)+.065*max(0.,1-abs(abs(a)-.80)/.29)
    # The flank shield has a projecting upper corner and a recessed waist,
    # rather than a constant-width strap around the barrel.
    scallop=.100*max(0.,1-abs(abs(a)-1.05)/.53)-.060*max(0.,1-abs(abs(a)-1.85)/.30)
    flare=1.+(.07 if not inside else .04)*max(0.,1-abs(abs(a)-1.32)/.52)
    zz=z+u*depth+(.275+.725*u)*scallop
    v.append((width*factor*flare*math.sin(a),.385+rise*factor*math.cos(a)+peak,zz))
 f=[]
 layer=len(sections)*n
 for inside in (0,1):
  for row in range(len(sections)-1):
   k=inside*layer+row*n
   for i in range(n-1):
    face=(k+i,k+i+1,k+n+i+1,k+n+i);f.append(tuple(reversed(face)) if inside else face)
 for edge in (0,len(sections)-1):
  k=edge*n
  for i in range(n-1):
   face=(k+i,k+i+layer,k+i+1+layer,k+i+1);f.append(tuple(reversed(face)) if edge else face)
 for row in range(len(sections)-1):
  k=row*n
  f.extend(((k,k+n,k+n+layer,k+layer),(k+n-1,k+n-1+layer,k+2*n-1+layer,k+2*n-1)))
 return crease_normals(part(name,PLATE,v,f,True),28)

@lru_cache(maxsize=1)
def raw_sculpture():
 from lugia_sculpt import paddle
 body=ellipsoid('Body / continuous heavy gray rhinoceros barrel',GRAY,(0,.385,-.230),(.355,.330,.500),40,26)
 head=longitudinal('Head / lowered broad armored wedge',PLATE,[(-.06,.515,.165,.140),(.09,.535,.255,.177),(.27,.500,.283,.177),(.46,.425,.232,.126),(.66,.362,.142,.060),(.760,.357,.027,.031)],segments=32)
 p=[body,head]
 p.append(longitudinal('Jaw / rooted broad lower armored muzzle',SHADOW,[(.09,.350,.139,.083),(.28,.278,.205,.051),(.47,.251,.216,.028),(.67,.265,.132,.022),(.748,.310,.027,.020)],segments=28))
 p.append(ellipsoid('Mouth back / rooted dark rhino oral chamber',INK,(0,.310,.350),(.155,.055,.065),24,16))
 p.append(curved_tube('Horn / rooted swept stone nasal lance',PLATE,[(0,.423,.604),(0,.504,.699),(0,.599,.779),(0,.657,.812)],[.070,.057,.028,.002],24))
 for j,(z,w,r,c,d) in enumerate(((-.590,.250,.260,.190,.125),(-.370,.355,.340,.150,.140),(-.150,.370,.350,.140,.140),(.045,.325,.300,.190,.120))):
  p.append(armor('Back armor / seated pentagonal dorsal plate '+str(j),z,w,r,c,d))
 p.append(paint('Brow armor / seated central forehead crest',PLATE,[(0,.680),(-.180,.612),(-.178,.507),(0,.408),(.178,.507),(.180,.612)],head,.017))
 for sign in (-1,1):
  ear=paddle('Ear / rooted pointed stone pinna '+str(sign),GRAY,[(sign*.220,.622,.055),(sign*.292,.763,.012),(sign*.280,.836,-.020)],[.074,.050,.005],[.047,.028,.003],segments=16,steps=4);p.append(ear)
  p.append(paint('Ear inset / fitted dark rhino ear opening '+str(sign),INK,[(sign*.258,.734),(sign*.282,.793),(sign*.303,.730)],ear,.002))
  x=sign*.217;y=.466
  xy=[(x+sign*dx,y+dy) for dx,dy in ((-.041,.010),(.033,.035),(.040,-.006),(-.034,-.019))]
  socket=paint('Eye socket / fitted angular black recess '+str(sign),INK,xy,head,.005);p.append(socket)
  cx=sum(a for a,b in xy)/len(xy);cy=sum(b for a,b in xy)/len(xy)
  white=paint('Eye white / narrow watchful ivory '+str(sign),IVORY,[(cx+(a-cx)*.80,cy+(b-cy)*.70) for a,b in xy],socket,.003);p.append(white)
  iris=paint('Iris / fitted red rhino gaze '+str(sign),RED,oval(cx-sign*.004,cy,.009,.011,20),white,.002);p.append(iris)
  p.append(paint('Pupil / fitted dark rhino slit '+str(sign),INK,oval(cx-sign*.004,cy,.0035,.008,16),iris,.0015))
  p.append(paint('Nostril / fitted dark nasal opening '+str(sign),INK,oval(sign*.078,.376,.013,.006,20),head,.003))
  p.append(curved_tube('Fang / rooted small ivory rhino tooth '+str(sign),IVORY,[(sign*.117,.329,.603),(sign*.116,.302,.617),(sign*.112,.277,.621)],[.018,.014,.002],16))
 for sign in (-1,1):
  for front in (False,True):
   z=.105 if front else -.445;name=str(sign)+' '+('front' if front else 'rear')
   leg=translate(rounded_loft('Leg / stocky rooted armored pillar '+name,GRAY,curved_profiles(((.035,.067,.073,z+.010),(.110,.093,.100,z+.008),(.220,.111,.118,z),(.340,.098,.111,z-.008),(.390,.065,.074,z-.012)),3),24,.75),sign*.275,0,0);p.append(leg)
   p.append(translate(rounded_loft('Leg armor / rooted tapered knee cuff '+name,PLATE,[(.135,.082,.092,z+.004),(.220,.113,.123,z),(.295,.103,.113,z-.006)],24,.70),sign*.275,0,0))
   foot=translate(rounded_loft('Foot / broad grounded gray rhino pad '+name,GRAY,[(0.,.104,.126,z+.039),(.029,.120,.143,z+.044),(.077,.109,.127,z+.038),(.098,.073,.078,z+.012)],28,.65),sign*.275,0,0);p.append(foot)
   for j,dx in enumerate((-.064,0,.064)):
    p.append(paint('Claw / supported blunt ivory toe '+name+' '+str(j),IVORY,oval(sign*.275+dx,.044,.025,.025,24),foot,.006))
 p.append(curved_tube('Tail / short rooted rear stone point',GRAY,[(0,.365,-.630),(0,.371,-.748),(.014,.397,-.822)],[.074,.047,.003],24))
 return {'name':'rhyhorn','version':1,'coordinate_system':COORDINATES,'primitives':p}
def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());p=m['primitives']
 if not normalize:return m
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:
  q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) or 0. for v in q['normals']]
 return m

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,required=True);args=a.parse_args();args.out.mkdir(parents=True,exist_ok=True)
 from rhino_glb import export_rhyhorn
 blob=export_rhyhorn();(args.out/'rhyhorn.glb').write_bytes(blob);print(len(blob),'bytes')
if __name__=='__main__':main()
