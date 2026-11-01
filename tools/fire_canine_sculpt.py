#!/usr/bin/env python3
"""Original striped puppy and long-legged guardian with fitted markings and rooted fur."""
import argparse,json,math
from pathlib import Path
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import ellipsoid,crease_normals,fitted_adaptive_contour,fitted_triangle_contour
from lugia_sculpt import paddle
from bovine_sculpt import oval
from goldfish_sculpt import surface
from cubone_sculpt import skin_shading
from skin_glb import COORDINATES
SPECIES=('growlithe','arcanine');HEIGHTS={'growlithe':.90,'arcanine':1.30}
ORANGE=(.95,.52,.24);ADULT_ORANGE=(.92,.45,.21);CREAM=(.98,.88,.66);INK=(.12,.13,.15);WHITE=(.98,.98,.95);BROWN=(.30,.18,.12);EAR=(.38,.24,.20)
def paint(name,color,outline,parent,th=.003):
 layer=fitted_triangle_contour(name,color,outline,parent,th,precision=7) if name.startswith(('Paw crease /','Back stripe /','Leg stripe /','Flank stripe /','Forehead stripe /','Lip /')) else fitted_adaptive_contour(name,color,outline,None,th,surface_at=surface(parent),initial_subdivisions=1)
 return skin_shading(crease_normals(layer,35.),parent)
def rotate(p,side,inverse=False,roof=False):
 q=dict(p)
 for field in ('positions','normals'):
  q[field]=[]
  for i in range(0,len(p[field]),3):
   x,y,z=p[field][i:i+3]
   q[field].extend((x,z,-y) if inverse and roof else (x,-z,y) if roof else (side*z,y,-side*x) if inverse else (-side*z,y,side*x))
 return q
def side_mark(name,color,outline,parent,side):
 return rotate(paint(name,color,[(-side*z,y) for z,y in outline],rotate(parent,side)),side,True)
def back_mark(name,outline,parent):return rotate(paint(name,INK,[(x,-z) for x,z in outline],rotate(parent,1,roof=True),.0035),1,True,True)
def leg_points(species,side,row):
 if species=='arcanine':return ((side*.09,.61,.21),(side*.195,.45,.25),(side*.21,.27,.30),(side*.21,.075,.30)) if row==0 else ((side*.09,.60,-.46),(side*.22,.43,-.48),(side*.24,.27,-.39),(side*.22,.075,-.50))
 return ((side*.065,.44,.17),(side*.14,.29,.20),(side*.145,.055,.23)) if row==0 else ((side*.065,.43,-.27),(side*.145,.25,-.30),(side*.145,.055,-.29))
def paw_center(species,side,row):
 p=leg_points(species,side,row)[-1];return (p[0],.055 if species=='arcanine' else .047,p[2]+(.035 if species=='arcanine' else .025))
def tail_points(species):
 return ((0,.61,-.55),(.03,.67,-.81),(.08,.85,-.98),(.13,1.08,-.91),(.14,1.17,-.72),(.10,1.12,-.60)) if species=='arcanine' else ((0,.42,-.32),(0,.46,-.48),(.05,.60,-.60),(.11,.75,-.56),(.14,.80,-.44))
def mane_center(species):return (0,.87,.35) if species=='arcanine' else (0,.50,.245)
def head_center(species):return (0,1.02,.46) if species=='arcanine' else (0,.75,.33)
def ear_points(species,side):
 big=species=='arcanine';y0=1.10 if big else .84;z0=.42 if big else .28
 return ((side*.13,y0,z0),(side*.18,y0+.12,z0-.005),(side*.22,1.32 if big else 1.065,z0-.02))
def forelock_points(species):return ((0,1.16,.48),(0,1.24,.51),(.075,1.27,.47)) if species=='arcanine' else ((0,.89,.35),(0,.97,.36),(.065,1.00,.325))
@lru_cache(maxsize=2)
def anatomy(species):
 if species not in SPECIES:raise ValueError('unknown striped canine')
 big=species=='arcanine';coat=ADULT_ORANGE if big else ORANGE;cy=.58 if big else .38;cz=-.13 if big else -.035
 body=ellipsoid('Body / distinct long guardian trunk' if big else 'Body / round compact puppy trunk',coat,(0,cy,cz),(.235,.255,.52) if big else (.205,.225,.36),36,22)
 neck=ellipsoid('Neck / rooted rising orange canine chest',coat,(0,.80,.30) if big else (0,.50,.20),(.20,.34,.24) if big else (.16,.23,.17),32,24)
 hc=head_center(species);head=ellipsoid('Head / shaped orange canine brow and cheek',coat,hc,(.185,.185,.18) if big else (.19,.185,.175),48,32)
 my=1.00 if big else .705;mz=.60 if big else .455
 muzzle=ellipsoid('Muzzle / rooted broad cream canine snout',CREAM,(0,my,mz),(.14,.084,.165) if big else (.14,.080,.120),32,22)
 jaw=ellipsoid('Jaw / rooted cream closed lower mandible',CREAM,(0,my-.055,mz+.002),(.124,.028,.140) if big else (.122,.027,.103),24,16)
 nose_z=surface(muzzle)(my+.032,0.)+.011
 nose=ellipsoid('Nose / rooted broad black canine nose',INK,(0,my+.032,nose_z),(.047,.030,.028),24,16)
 p=[body,neck,head,muzzle,jaw,nose]
 line=[(-.092,my-.028),(-.045,my-.043),(0,my-.047),(.045,my-.043),(.092,my-.028),(.084,my-.036),(.043,my-.049),(0,my-.053),(-.043,my-.049),(-.084,my-.036)]
 p.append(paint('Lip / fitted closed canine smile',INK,line,muzzle,.0025))
 for side in (-1,1):
  ex=side*(.102 if big else .100);ey=hc[1]+(.064 if big else .057)
  xy=[(ex-side*.037,ey-.016),(ex-side*.035,ey+.021),(ex+side*.035,ey+.035),(ex+side*.039,ey-.023)] if big else oval(ex,ey,.033,.039,24)
  socket=paint('Eye socket / fitted dark canine almond '+str(side),INK,xy,head,.004);p.append(socket)
  ec=(sum(x for x,y in xy)/len(xy),sum(y for x,y in xy)/len(xy));eye=paint('Eye / fitted ivory canine gaze '+str(side),WHITE,[(ec[0]+(x-ec[0])*.83,ec[1]+(y-ec[1])*.78) for x,y in xy],socket,.0025);p.append(eye)
  pupil=paint('Pupil / fitted dark brown canine pupil '+str(side),BROWN,oval(ex-side*.004,ey+.003,.010 if big else .013,.013 if big else .022,20),eye,.002);p.append(pupil)
  p.append(paint('Eye glint / supported white canine glint '+str(side),WHITE,oval(ex-side*.007,ey+.008,.0025,.0035,12),pupil,.0015))
  y0=1.10 if big else .84;z0=.42 if big else .28;tip=1.32 if big else 1.065
  ear=paddle('Ear / rooted broad pointed orange canine ear '+str(side),coat,ear_points(species,side),[.070,.059,.003],[.038,.029,.002],segments=20,steps=5);p.append(ear)
  outline=[(side*.115,y0+.055),(side*.174,tip-.060),(side*.202,y0+.115),(side*.181,y0+.046)]
  p.append(paint('Ear inset / fitted dark warm canine ear '+str(side),EAR,outline,ear,.003))
  for row in range(2):
   pts=leg_points(species,side,row);radii=[.080,.077,.054,.042] if big and row else [.067,.058,.050,.040] if big else [.069,.058,.043]
   leg=paddle(f'Leg / continuous weight-bearing canine limb {side} {row}',coat,pts,radii,[r*.95 for r in radii],segments=20,steps=5);p.append(leg)
   pc=paw_center(species,side,row);paw=ellipsoid(f'Paw / complete grounded canine pad {side} {row}',CREAM if big else coat,pc,(.090,.055,.140) if big else (.072,.047,.105),24,16);p.append(paw)
   for j in (-1,1):
    x=pc[0]+j*(.028 if big else .022);y=pc[1];p.append(paint(f'Paw crease / fitted toe separation {side} {row} {j}',INK,[(x-.0014,y-.014),(x+.0014,y-.014),(x+.0014,y+.022),(x-.0014,y+.022)],paw,.0016))
   sy=.39 if big else .265;sz=pts[1][2];band=[(sz-.027,sy+.043),(sz+.027,sy+.030),(sz+.019,sy-.028),(sz-.010,sy-.048)]
   p.append(side_mark(f'Leg stripe / fitted dark canine leg marking {side} {row}',INK,band,leg,side))
   if big:
    fc=(pc[0],.15,pc[2]-.035);fur=ellipsoid(f'Ankle fur / rooted cream guardian cuff {side} {row}',CREAM,fc,(.103,.085,.117),18,12);p.append(fur)
    for j,dx in enumerate((-.078,0,.078)):
     root=(fc[0]+dx*.5,.16,fc[2]+.020);end=(fc[0]+dx,.089,fc[2]+(.126 if j==1 else .080))
     p.append(paddle(f'Ankle tuft / rooted cream guardian foot fur {side} {row} {j}',CREAM,[root,end],[.046,.002],[.035,.002],segments=12,steps=3))
  for j,zc in enumerate((-.38,-.10,.15) if big else (-.16,.04)):
   w=.042 if big else .030;top=.205 if big else .18;outline=[(zc-w,cy+top-.03),(zc-.008,cy+top),(zc+w,cy+top-.03),(zc+.014,cy-.025),(zc+.023,cy-.12),(zc+.005,cy-.16),(zc-.019,cy-.06),(zc-.007,cy+.06)]
   p.append(side_mark(f'Flank stripe / fitted tiger marking {side} {j}',INK,outline,body,side))
 for j,zc in enumerate((-.38,-.10,.15) if big else (-.16,.04)):
  p.append(back_mark('Back stripe / fitted dark dorsal marking '+str(j),[(-.17,zc-.025),(-.085,zc-.045),(.085,zc-.045),(.17,zc-.025),(.14,zc+.025),(0,zc+.04),(-.14,zc+.025)],body))
 h=hc[1];p.append(paint('Forehead stripe / fitted pointed dark blaze',INK,[(-.041,h+.142),(.041,h+.142),(.015,h+.065),(0,h+.078),(-.015,h+.065)],head,.003))
 mc=mane_center(species);mr=(.280,.34,.260) if big else (.185,.215,.180)
 mane=ellipsoid('Mane / distinct full cream guardian ruff' if big else 'Mane / soft rounded cream puppy ruff',CREAM,mc,mr,28,20);p.append(mane)
 count=10 if big else 7
 for j in range(count):
  angle=(j+.5)*math.tau/count;ux,uy=math.cos(angle),math.sin(angle);root=(mr[0]*.70*ux,mc[1]+mr[1]*.70*uy,mc[2]);mid=(mr[0]*1.04*ux,mc[1]+mr[1]*1.01*uy,mc[2]-.010);tip=(mr[0]*1.35*ux,mc[1]+mr[1]*1.20*uy,mc[2]-.035)
  p.append(paddle(f'Mane tuft / rooted flowing cream ruff {j}',CREAM,[root,mid,tip],[.070 if big else .054,.080 if big else .060,.002],[.065 if big else .047,.048 if big else .034,.002],segments=14,steps=3))
 fp=forelock_points(species)
 p.append(paddle('Forelock / rooted swept cream forehead tuft',CREAM,fp,[.14,.11,.002] if big else [.095,.080,.002],[.085,.055,.002] if big else [.055,.040,.002],segments=20,steps=5))
 p.append(paddle('Tail / distinct broad curled cream guardian plume' if big else 'Tail / curled fluffy cream puppy tail',CREAM,tail_points(species),[.075,.11,.14,.15,.105,.002] if big else [.065,.09,.10,.08,.002],[.075,.11,.13,.14,.09,.002] if big else [.065,.085,.09,.070,.002],segments=24,steps=6))
 for j,index in enumerate((2,3,4) if big else (2,)):
  root=tail_points(species)[index];mid=(root[0]-.12,root[1]+.025,root[2]-.025);tip=(root[0]-.20 if big else root[0]-.16,root[1]+.06,root[2]-.040)
  p.append(paddle(f'Tail tuft / rooted cream plume strand {j}',CREAM,[root,mid,tip],[.052,.040,.0015],[.045,.028,.0015],segments=12,steps=3))
 return p
@lru_cache(maxsize=4)
def sculpture(species,normalize=True):
 p=deepcopy(anatomy(species));lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 if normalize:
  for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--species',choices=SPECIES,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True);(a.output/(a.species+'.mesh.json')).write_text(json.dumps(sculpture(a.species),separators=(',',':')))
if __name__=='__main__':main()
