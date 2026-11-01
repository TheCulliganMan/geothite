#!/usr/bin/env python3
"""Original upright Crystal fire ferret and sloping fire badger with shared-surface coats."""
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

def coat_sweep(name,profiles,axis='z',segments=36,fur_hem=False,mask_edges=None):
 # Two closed half-solids meet on the exact same diameter. Exterior normals
 # come from the unsplit whole, so the color border has no raised shell or seam.
 rows=curved_profiles([(z,w,h,cy) for z,cy,w,h in profiles],3);vertices=[]
 for along,w,h,center in rows:
  edge=0.
  if mask_edges:
   edge=mask_edges[0][1] if along<=mask_edges[0][0] else mask_edges[-1][1]
   for a,b in zip(mask_edges,mask_edges[1:]):
    if a[0]<=along<=b[0]:edge=a[1]+(b[1]-a[1])*(along-a[0])/(b[0]-a[0]);break
  for j in range(segments):
   half=segments//2
   angle=(edge+(math.pi-2*edge)*j/half) if j<half else (math.pi-edge+(math.pi+2*edge)*(j-half)/half)
   x=w*math.cos(angle);other=h*math.sin(angle)
   hem=0.
   if fur_hem:
    fade=max(0.,1.-(along-.05)/.16)
    hem=fade*(.04*max(0.,math.sin(angle))**2
              -.04*max(0.,-math.sin(angle))*(1. if j%2 else .25))
   vertices.append((x,along+hem,center+other) if axis=='y' else (x,center+other,along))
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
 if species=='typhlosion':return [(side*.13,.91,.09),(side*.29,.80,.15),(side*.32,.72,.24)],[.080,.060,.043]
 if row==0:return [(side*.12,.80,.05),(side*.16,.71,.12),(side*.16,.69,.20)],[.055,.042,.027]
 return [(side*.11,.27,-.095),(side*.14,.16,-.085),(side*.15,.052,-.08)],[.090,.070,.050]

def fire_specs(species):
 if species=='quilava':
  return tuple((f'{cluster} {j}',(side*.060,.99 if cluster=='neck' else .20,.04 if cluster=='neck' else -.22),(side*(.18 if cluster=='neck' else .25),1.27 if cluster=='neck' and j==1 else (1.19 if cluster=='neck' else .16),-.20 if cluster=='neck' and j==1 else (-.35 if cluster=='neck' else -.65)),.18,side*22.) for cluster in ('neck','rump') for j,side in enumerate((-1,0,1)))
 return tuple((f'mantle {j}',(x,.99,.015),(x*4.5,1.35+(.13 if abs(x)<.10 else 0.),-.25-abs(x)*.25),.22,side*24.) for j,x in enumerate((-.16,-.105,-.05,0.,.05,.105,.16)) for side in ((-1 if x<0 else 1),))

def ear_wedge(species,side):
 adult=species=='typhlosion'
 triangle=[(side*.09,1.255,.14),(side*.165,1.25,.14),(side*.18,1.375,.08)] if adult else [(side*.05,1.105,.09),(side*.105,1.10,.09),(side*.13,1.20,.035)]
 depth=.022 if adult else .017
 vertices=[(x,y,z+d) for d in (depth,-depth) for x,y,z in triangle]
 faces=[(0,1,2),(5,4,3),(0,3,4,1),(1,4,5,2),(2,5,3,0)]
 if side<0:faces=[tuple(reversed(face)) for face in faces]
 return part('Ear / short rooted triangular midnight pinna '+str(side),COAT,vertices,faces)

@lru_cache(maxsize=2)
def anatomy(species):
 if species not in SPECIES:raise ValueError('unknown fire badger')
 adult=species=='typhlosion';p=[]
 profiles=[(.05,-.06,.32,.17),(.16,-.09,.31,.22),(.36,-.07,.27,.24),(.61,-.015,.22,.22),(.83,.055,.19,.205),(.99,.11,.20,.18),(1.12,.16,.145,.14)] if adult else [(.07,-.08,.09,.09),(.16,-.10,.21,.18),(.34,-.11,.23,.19),(.54,-.07,.19,.17),(.75,.01,.145,.14),(.93,.07,.12,.12),(1.03,.08,.08,.08)]
 body,whole_body=coat_sweep('Body',profiles,'y',fur_hem=adult);p+=body
 hp=[(.015,1.15,.115,.12),(.12,1.205,.20,.145),(.245,1.215,.215,.11),(.34,1.14,.14,.065),(.39,1.075,.070,.026),(.415,1.065,.012,.014)] if adult else [(-.015,.99,.07,.08),(.06,1.045,.115,.115),(.16,1.06,.14,.11),(.25,1.055,.12,.08),(.31,1.04,.085,.052),(.34,1.038,.025,.025)]
 mask_edges=list(zip([v[0] for v in hp],[.45,.80,.85,.70,.38,.10] if adult else [.35,.70,.85,.75,.40,.10]))
 head,whole_head=coat_sweep('Head',hp,mask_edges=mask_edges);p+=head
 if adult:
  p.append(bill('Jaw / rooted open cream badger mandible',CREAM,[(.08,.08,1.02,1.14),(.19,.12,.975,1.055),(.29,.105,.98,1.035),(.36,.055,1.015,1.045),(.405,.010,1.04,1.055)]))
  p.append(ellipsoid('Tongue / seated pink badger tongue',PINK,(0,1.030,.29),(.050,.007,.040),22,12))
 else:
  p.append(fitted('Lip / small closed cream ferret smile',INK,[(-.050,1.015),(0.,1.005),(.050,1.015),(.046,1.020),(0.,1.012),(-.046,1.020)],whole_head,.0025))
 nose=(0,1.069,.414) if adult else (0,1.044,.341)
 p.append(ellipsoid('Nose / rooted soft charcoal badger nose',INK,nose,(.020,.012,.016) if adult else (.015,.010,.012),24,14))
 for side in (-1,1):
  cx=side*(.145 if adult else .083);cy=1.247 if adult else 1.095
  outline=[(cx+side*dx,cy+dy) for dx,dy in (((-.038,-.012),(.036,.003),(.033,.031),(-.031,.009)) if adult else ((-.032,-.010),(.029,.002),(.027,.027),(-.025,.009)))]
  socket=fitted('Eye socket / fitted angular badger gaze '+str(side),INK,outline,whole_head,.004);p.append(socket)
  ex=sum(x for x,y in outline)/4;ey=sum(y for x,y in outline)/4
  eye=fitted('Eye white / fitted ivory badger glare '+str(side),WHITE,[(ex+(x-ex)*.79,ey+(y-ey)*.68) for x,y in outline],socket,.0025);p.append(eye)
  iris=fitted('Iris / fitted red badger iris '+str(side),RED,oval(ex,ey,.005,.005,20),eye,.0018);p.append(iris)
  pupil=fitted('Pupil / fitted charcoal badger pupil '+str(side),INK,oval(ex,ey,.002,.003,16),iris,.0015);p.append(pupil)
  p.append(fitted('Glint / fitted ivory badger reflection '+str(side),WHITE,oval(ex-side*.0004,ey+.0009,.0008,.0012,12),pupil,.0012))
  p.append(ear_wedge(species,side))
  if adult:
   p.append(curved_tube('Fang / rooted ivory upper badger tooth '+str(side),WHITE,[(side*.070,1.13,.30),(side*.070,1.105,.305),(side*.069,1.087,.31)],[.010,.007,.0015],14))
   p.append(ellipsoid('Haunch / strong rooted cream badger thigh '+str(side),CREAM,(side*.24,.16,-.015),(.12,.14,.13),28,18))
  path,radii=arm_path(species,side,0);p.append(curved_tube('Arm / curved complete cream badger forelimb '+str(side),CREAM,path,radii,28));hx,hy,hz=path[-1]
  hand_scale=.85 if adult else .60
  p.append(ellipsoid('Hand / rooted cream badger palm '+str(side),CREAM,(hx,hy-.012*hand_scale,hz+.014*hand_scale),tuple(v*hand_scale for v in (.058,.052,.061)),24,16))
  for j,dx in enumerate((-.032,0.,.032)):
   p.append(curved_tube('Finger / complete cream badger digit '+str(side)+' '+str(j),CREAM,[(hx+dx*hand_scale,hy-.023*hand_scale,hz+.046*hand_scale),(hx+dx*1.08*hand_scale,hy-.058*hand_scale,hz+.065*hand_scale),(hx+dx*1.1*hand_scale,hy-.069*hand_scale,hz+.085*hand_scale)],[v*hand_scale for v in (.017,.014,.002)],14))
  for row in (1,):
   if adult:x,y,z=side*.26,.045,.12
   else:
    path,radii=arm_path(species,side,row);p.append(curved_tube('Leg / short rooted cream ferret support '+str(side)+' '+str(row),CREAM,path,radii,24));x,y,z=side*.15,.032,-.015
   label=str(side)+' '+str(row)
   p.append(ellipsoid('Foot / planted broad cream badger pad '+label,CREAM,(x,y,z),(.10,.045,.15) if adult else (.075,.032,.12),28,16))
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
