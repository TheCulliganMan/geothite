#!/usr/bin/env python3
"""Original low fire ferret and upright fire badger with shared-surface two-tone coats."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from sculpt_geometry import part,curved_profiles,ellipsoid
from bovine_sculpt import curved_tube,oval
from fire_dragon_sculpt import fitted
from cyndaquil_sculpt import flame
from chikorita_sculpt import linear
from murkrow_sculpt import bill
from skin_glb import COORDINATES
SPECIES=('quilava','typhlosion');HEIGHTS={'quilava':1.20,'typhlosion':1.45}
CREAM=(.98,.88,.62);COAT=(.13,.30,.34);INK=(.075,.09,.10);WHITE=(.99,.97,.86);RED=(.83,.23,.20);PINK=(.87,.46,.44)

def deduplicate(p):
 out=dict(p,positions=[],normals=[],indices=[]);lookup={}
 for old in p['indices']:
  key=tuple(p['positions'][old*3:old*3+3]+p['normals'][old*3:old*3+3])
  if key not in lookup:lookup[key]=len(out['positions'])//3;out['positions']+=list(key[:3]);out['normals']+=list(key[3:])
  out['indices'].append(lookup[key])
 return out

def coat_sweep(name,profiles,axis='z',segments=36):
 # Two closed half-solids meet on the exact same diameter. Exterior normals
 # come from the unsplit whole, so the color border has no raised shell or seam.
 rows=curved_profiles([(z,w,h,cy) for z,cy,w,h in profiles],3);vertices=[]
 for along,w,h,center in rows:
  for j in range(segments):
   x=w*math.cos(math.tau*j/segments);other=h*math.sin(math.tau*j/segments)
   vertices.append((x,along,center+other) if axis=='y' else (x,center+other,along))
 faces=[tuple(reversed(range(segments))),tuple((len(rows)-1)*segments+j for j in range(segments))]
 for k in range(len(rows)-1):
  for j in range(segments):a=k*segments+j;b=k*segments+(j+1)%segments;faces.append((a,b,b+segments,a+segments))
 whole=part(name+' / whole surface reference',CREAM,vertices,faces,True)
 normals={tuple(whole['positions'][i:i+3]):whole['normals'][i:i+3] for i in range(0,len(whole['positions']),3)};result=[]
 for half in (0,1):
  ring=segments//2+1;vs=[vertices[k*segments+(half*segments//2+j)%segments] for k in range(len(rows)) for j in range(ring)]
  fs=[tuple(reversed(range(ring))),tuple((len(rows)-1)*ring+j for j in range(ring))]
  for k in range(len(rows)-1):
   for j in range(ring-1):a=k*ring+j;b=a+1;fs.append((a,b,b+ring,a+ring))
  exterior_end=len(fs)
  for k in range(len(rows)-1):a=k*ring;b=a+ring-1;fs.append((b,a,a+ring,b+ring))
  coat=(half==1 if axis=='y' else half==0);p=part(name+(' / joined midnight dorsal coat' if coat else ' / joined warm cream underside'),COAT if coat else CREAM,vs,fs)
  cap_triangles=2*(ring-2);exterior_triangles=(exterior_end-2)*2
  for old in set(p['indices'][cap_triangles*3:(cap_triangles+exterior_triangles)*3]):
   key=tuple(p['positions'][old*3:old*3+3]);p['normals'][old*3:old*3+3]=normals[key]
  result.append(deduplicate(p))
 return result,whole

def arm_path(species,side,row=0):
 if species=='typhlosion':return [(side*.17,.82,.025),(side*.30,.63,.11),(side*.33,.48,.23)],[.085,.065,.046]
 z=.13 if row==0 else -.45
 return [(side*.11,.29,z-.035),(side*.17,.17,z-.015),(side*.18,.050,z+.025)],[.075,.065,.045]

def fire_specs(species):
 if species=='quilava':
  return tuple((f'{cluster} {j}',(side*.075,.47 if cluster=='neck' else .38,.06 if cluster=='neck' else -.47),(side*(.18 if j!=1 else .05),1.09 if cluster=='neck' else .76,-.06 if cluster=='neck' else -.69),.18,side*22.) for cluster in ('neck','rump') for j,side in enumerate((-1,0,1)))
 return tuple((f'mantle {j}',(x,.94,-.075),(x*2.15,1.34+(.14 if abs(x)<.15 else 0.),-.27-abs(x)*.25),.25,side*24.) for j,x in enumerate((-.21,-.14,-.07,0.,.07,.14,.21)) for side in ((-1 if x<0 else 1),))

@lru_cache(maxsize=2)
def anatomy(species):
 if species not in SPECIES:raise ValueError('unknown fire badger')
 adult=species=='typhlosion';p=[]
 profiles=[(.14,-.025,.075,.09),(.28,-.03,.24,.23),(.48,-.02,.28,.25),(.70,0.,.245,.205),(.88,.015,.20,.16),(1.03,.035,.16,.13),(1.12,.045,.10,.09)] if adult else [(-.65,.21,.02,.04),(-.55,.24,.16,.20),(-.36,.25,.20,.225),(-.12,.28,.19,.23),(.10,.34,.155,.205),(.24,.39,.08,.13)]
 body,whole_body=coat_sweep('Body',profiles,'y' if adult else 'z');p+=body
 hp=[(-.03,1.13,.09,.11),(.08,1.17,.185,.14),(.20,1.17,.19,.13),(.31,1.13,.165,.082),(.41,1.115,.115,.052),(.46,1.11,.030,.026)] if adult else [(.10,.41,.07,.10),(.23,.47,.15,.13),(.34,.48,.16,.13),(.45,.45,.13,.085),(.52,.43,.10,.055),(.56,.43,.042,.035)]
 head,whole_head=coat_sweep('Head',hp);p+=head
 if adult:
  p.append(bill('Jaw / rooted open cream badger mandible',CREAM,[(.055,.075,1.00,1.12),(.19,.12,.965,1.025),(.33,.11,.973,1.027),(.425,.055,1.01,1.055),(.447,.010,1.04,1.06)]))
  p.append(ellipsoid('Tongue / seated pink badger tongue',PINK,(0,1.026,.28),(.061,.012,.065),22,12))
  p.append(ellipsoid('Collar / broad rooted midnight shoulder fire socket',COAT,(0,.94,-.025),(.275,.105,.155),32,18))
 else:
  p.append(fitted('Lip / small closed cream ferret smile',INK,[(-.055,.403),(0.,.394),(.055,.403),(.052,.407),(0.,.399),(-.052,.407)],whole_head,.0025))
 nose=(0,1.124,.453) if adult else (0,.44,.558)
 p.append(ellipsoid('Nose / rooted soft charcoal badger nose',INK,nose,(.027,.017,.020) if adult else (.019,.012,.014),24,14))
 for side in (-1,1):
  cx=side*(.105 if adult else .085);cy=1.218 if adult else .520
  outline=[(cx+side*dx,cy+dy) for dx,dy in ((-.034,-.012),(.030,-.020),(.030,.022),(-.028,.034))]
  socket=fitted('Eye socket / fitted angular badger gaze '+str(side),INK,outline,head[0],.004);p.append(socket)
  ex=sum(x for x,y in outline)/4;ey=sum(y for x,y in outline)/4
  eye=fitted('Eye white / fitted ivory badger glare '+str(side),WHITE,[(ex+(x-ex)*.79,ey+(y-ey)*.68) for x,y in outline],socket,.0025);p.append(eye)
  iris=fitted('Iris / fitted red badger iris '+str(side),RED,oval(ex,ey,.007,.009,20),eye,.0018);p.append(iris)
  pupil=fitted('Pupil / fitted charcoal badger pupil '+str(side),INK,oval(ex,ey,.003,.006,16),iris,.0015);p.append(pupil)
  p.append(fitted('Glint / fitted ivory badger reflection '+str(side),WHITE,oval(ex-side*.0007,ey+.0017,.0013,.0021,12),pupil,.0012))
  root=(side*.115,1.235,.055) if adult else (side*.095,.515,.18)
  p.append(curved_tube('Ear / short rooted triangular midnight pinna '+str(side),COAT,[root,(side*(.16 if adult else .135),root[1]+.050,root[2]-.025),(side*(.18 if adult else .15),root[1]+.055,root[2]-.060)],[.045,.032,.0015],20))
  if adult:
   p.append(curved_tube('Fang / rooted ivory upper badger tooth '+str(side),WHITE,[(side*.075,1.092,.30),(side*.075,1.07,.305),(side*.074,1.046,.313)],[.012,.009,.0015],14))
   path,radii=arm_path(species,side);p.append(curved_tube('Arm / curved complete cream badger forelimb '+str(side),CREAM,path,radii,28));hx,hy,hz=path[-1]
   p.append(ellipsoid('Hand / rooted cream badger palm '+str(side),CREAM,(hx,hy-.012,hz+.014),(.058,.052,.061),24,16))
   for j,dx in enumerate((-.032,0.,.032)):
    p.append(curved_tube('Finger / complete cream badger digit '+str(side)+' '+str(j),CREAM,[(hx+dx,hy-.023,hz+.046),(hx+dx*1.08,hy-.058,hz+.065),(hx+dx*1.1,hy-.069,hz+.085)],[.017,.014,.002],14))
   p.append(ellipsoid('Haunch / strong rooted cream badger thigh '+str(side),CREAM,(side*.19,.20,0.),(.14,.18,.16),28,18))
  for row in ((1,) if adult else (0,1)):
   if adult:x,y,z=side*.21,.050,.14
   else:
    path,radii=arm_path(species,side,row);p.append(curved_tube('Leg / short rooted cream ferret support '+str(side)+' '+str(row),CREAM,path,radii,24));x,y,z=side*.18,.032,path[-1][2]+.025
   label=str(side)+' '+str(row)
   p.append(ellipsoid('Foot / planted broad cream badger pad '+label,CREAM,(x,y,z),(.10,.050,.16) if adult else (.066,.032,.10),28,16))
   for j,dx in enumerate((-.05,0.,.05) if adult else (-.035,0.,.035)):
    p.append(curved_tube('Toe / blunt rooted cream badger digit '+label+' '+str(j),CREAM,[(x+dx,y,z+(.12 if adult else .075)),(x+dx,y*.65,z+(.17 if adult else .105))],[.024,.006] if adult else [.016,.004],14))
 for label,root,tip,width,twist in fire_specs(species):
  for layer,mesh in flame(root,tip,width,.042 if adult else .030,twist,-.10).items():
   color=(.87,.18,.10) if layer=='ember silhouette' else ((1.,.48,.07) if 'orange' in layer else (1.,.84,.20));p.append(dict(mesh,part='Fire / '+label+' / '+layer,base_color=linear(color)))
 return tuple(p)

def sculpture(species,normalize=True):
 from copy import deepcopy
 p=deepcopy(list(anatomy(species)));lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for q in p:
  if normalize:q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
  q['normals']=[round(v,7) for v in q['normals']]
  for field in ('positions','normals'):q[field]=[0. if v==0 else v for v in q[field]]
 return dict(name=species,version=1,coordinate_system=COORDINATES,primitives=p)

def main():
 from fire_badger_glb import export_fire_badger
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_fire_badger(s))
if __name__=='__main__':main()
