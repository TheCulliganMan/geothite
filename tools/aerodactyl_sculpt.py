#!/usr/bin/env python3
"""Original fossil flyer with an open toothed skull and supported scalloped sails."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,fitted_contour,fitted_patch,fitted_triangle_contour,crease_normals
from rhyhorn_sculpt import longitudinal
from bovine_sculpt import curved_tube,oval
from wing_insect_sculpt import contour,membrane
from abra_sculpt import surface,polygon_caps
from chikorita_sculpt import cross,unit
from fire_dragon_sculpt import inset_wing
from skin_glb import COORDINATES
HEIGHT=1.30
GRAY=(.64,.64,.71);LIGHT=(.72,.72,.79);PURPLE=(.43,.30,.56);IRIS=(.52,.16,.32);INK=(.10,.075,.12);MOUTH=(.32,.16,.23);TONGUE=(.73,.42,.46);IVORY=(.97,.96,.90)
TAIL=((0.,.28,-.10),(0.,.24,-.40),(.02,.235,-.78),(.035,.24,-1.13))

def wing_point(x,y,z,side):
 a=side*x-.105;depth=-.09-.06*abs(x)+.025*(y-.70)**2;c=z-depth
 b=(y-.685)*1.4*(.25+.75*max(0.,min(1.,a/.4)));a*=1.4
 return side*(.105+.60*a+.80*c),.47+.80*a-.60*c,.03+b

def wing_frame(q,side):
 # The long leading arm rises outward. The broad chord lies in depth, rather
 # than presenting an invented flat bat silhouette only to the front camera.
 # Compress the buried shoulder web; widen the actual chord away from it.
 vertices=[]
 for i in range(0,len(q['positions']),3):
  vertices.append(wing_point(*q['positions'][i:i+3],side))
 faces=[tuple(q['indices'][i:i+3]) for i in range(0,len(q['indices']),3)]
 result=crease_normals(part(q['part'],GRAY,vertices,faces,True),35.)
 result['base_color']=q['base_color']
 return result

def wing_case(outline,side):
 # Triangulate the final plane. Warping long pre-existing cap triangles can
 # reverse their projected winding even when the perimeter remains simple.
 xy=[]
 for x,y in outline:
  z=-.09-.06*x+.025*(y-.70)**2;x,y,z=wing_point(side*x,y,z,side);xy.append((.60*x+side*.80*y,z))
 n=len(xy);vertices=[(x,y,-.198+d) for d in (.013,-.013) for x,y in xy];caps=polygon_caps(vertices[:n]);order=list(range(n))
 if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(xy,xy[1:]+xy[:1]))<0.:order.reverse()
 faces=caps+[tuple(i+n for i in reversed(f)) for f in caps]+[(a,a+n,b+n,b) for a,b in zip(order,order[1:]+order[:1])]
 q=crease_normals(part('Wing back / closed gray fossil sail '+str(side),GRAY,vertices,faces,True),35.)
 for key in ('positions','normals'):
  q[key]=[value for u,v,w in zip(q[key][::3],q[key][1::3],q[key][2::3]) for value in (.60*u+side*.80*w,side*.80*u-.60*w,v)]
 return q

def wing_mark(case,outline,side):
 # Fit after the shoulder warp. Clipped child vertices evaluated through a
 # nonlinear warp can otherwise dip beneath the parent's coarser triangles.
 local=dict(case)
 for key in ('positions','normals'):
  local[key]=[v for x,y,z in zip(case[key][::3],case[key][1::3],case[key][2::3]) for v in (.60*x+side*.80*y,z,side*.80*x-.60*y)]
 xy=[]
 for x,y in inset_wing(outline,.012):
  z=-.09-.06*x+.025*(y-.70)**2;x,y,z=wing_point(side*x,y,z,side);xy.append((.60*x+side*.80*y,z))
 mark=crease_normals(fitted_triangle_contour('Wing / broad scalloped violet fossil membrane '+str(side),PURPLE,xy,local,.003,precision=6),35.)
 for key in ('positions','normals'):
  mark[key]=[v for u,vv,w in zip(mark[key][::3],mark[key][1::3],mark[key][2::3]) for v in (.60*u+side*.80*w,side*.80*u-.60*w,vv)]
 return mark

def turn(p,up):
 q=dict(p)
 for key in ('positions','normals'):
  q[key]=[v for i in range(0,len(p[key]),3) for v in ((p[key][i],-p[key][i+2],p[key][i+1]) if up else (p[key][i],p[key][i+2],-p[key][i+1]))]
 return q

def lateral(p,side,inverse=False):
 q=dict(p)
 for key in ('positions','normals'):
  q[key]=[v for x,y,z in zip(p[key][::3],p[key][1::3],p[key][2::3]) for v in ((side*z,y,-side*x) if inverse else (-side*z,y,side*x))]
 return q

def mouth_patch(name,color,parent,outline,up,thickness):
 skin=turn(parent,up);xy=[(x,-z if up else z) for x,z in outline]
 return turn(crease_normals(fitted_triangle_contour(name,color,xy,skin,thickness,precision=6),35),not up)

def arrow():
 # Build a closed dart in its own plane, then orient it along the actual tail.
 xy=[(-.070,.01),(0,.16),(.070,.01),(.030,.04),(0,-.06),(-.030,.04)]
 v=[(x,y,z) for z in (-.010,.010) for x,y in xy];n=len(xy);caps=polygon_caps(v[n:]);caps=[tuple(i+n for i in f) for f in caps]
 ids=list(range(n));ids.reverse()
 fs=caps+[tuple(i-n for i in reversed(f)) for f in caps]+[(a,b,b+n,a+n) for a,b in zip(ids,ids[1:]+ids[:1])]
 q=crease_normals(part('Tail tip / thick pointed fossil arrowhead',GRAY,v,fs),40)
 ex=(0.,1.,0.);ey=(0.,0.,-1.);ez=cross(ex,ey);origin=TAIL[-1]
 for key in ('positions','normals'):
  q[key]=[round(sum(q[key][i+j]*basis[k] for j,basis in enumerate((ex,ey,ez)))+(origin[k] if key=='positions' else 0),7) for i in range(0,len(q[key]),3) for k in range(3)]
 return q

def sculpture(normalize=True):
 body=loft('Body / continuous lean fossil chest and neck',GRAY,curved_profiles(((.20,.030,.035,-.10),(.29,.160,.145,-.10),(.41,.205,.170,-.045),(.49,.185,.160,.015),(.57,.125,.110,.105),(.67,.095,.085,.18)),4),32)
 head=longitudinal('Skull / swept narrow dinosaur wedge',GRAY,[(.10,.64,.075,.080),(.21,.68,.190,.130),(.34,.70,.163,.083),(.50,.72,.125,.047),(.65,.72,.077,.055),(.70,.665,.013,.045)],segments=32)
 jaw=longitudinal('Jaw / lowered long fossil mandible',GRAY,[(.19,.60,.092,.028),(.31,.56,.132,.029),(.49,.48,.105,.030),(.64,.43,.066,.031),(.69,.45,.012,.025)],segments=32)
 p=[body,head,jaw]
 p.append(mouth_patch('Mouth ceiling / supported dark upper palate',MOUTH,head,[(-.092,.27),(-.10,.40),(-.045,.63),(.045,.63),(.10,.40),(.092,.27)],False,.004))
 p.append(mouth_patch('Mouth floor / supported dark lower lining',MOUTH,jaw,[(-.080,.28),(-.080,.40),(-.045,.61),(.045,.61),(.080,.40),(.080,.28)],True,.004))
 p.append(mouth_patch('Tongue / fitted long rose tongue',TONGUE,jaw,[(-.045,.33),(-.052,.43),(-.032,.60),(.032,.60),(.052,.43),(.045,.33)],True,.008))
 p.append(ellipsoid('Mouth back / recessed occupied fossil throat',MOUTH,(0.,.602,.265),(.10,.057,.032),24,16))
 for sign in (-1,1):
  p.append(ellipsoid('Jaw hinge / rooted rear cheek '+str(sign),GRAY,(sign*.12,.63,.21),(.060,.095,.066),24,16))
  p.append(curved_tube('Skull horn / long swept backward fossil crest '+str(sign),GRAY,[(sign*.113,.755,.17),(sign*.18,.82,.065),(sign*.22,.84,-.03)],[.045,.027,.0015],20))
  p.append(curved_tube('Brow / swept gray fossil orbital ridge '+str(sign),GRAY,[(sign*.13,.74,.19),(sign*.15,.755,.26),(sign*.15,.72,.37)],[.025,.022,.010],20))
  # A profile eye belongs to the rear side of the skull, not its foremost +Z
  # surface. The former projection stretched a white stripe along the snout.
  side_head=lateral(head,sign);xy=[(-sign*z,y) for z,y in ((.21,.765),(.375,.716),(.335,.681),(.235,.697))]
  socket=crease_normals(fitted_triangle_contour('Eye socket / supported stern dark wedge '+str(sign),INK,xy,side_head,.004,precision=6),35)
  cx=sum(a for a,b in xy)/4;cy=sum(b for a,b in xy)/4
  white=crease_normals(fitted_triangle_contour('Eye white / fitted narrow ivory wedge '+str(sign),IVORY,[(cx+(a-cx)*.85,cy+(b-cy)*.76) for a,b in xy],socket,.0025,precision=6),35)
  iris=crease_normals(fitted_triangle_contour('Iris / fitted red-violet fossil gaze '+str(sign),IRIS,oval(-sign*.290,.716,.014,.014,24),white,.0018,precision=6),35)
  pupil=crease_normals(fitted_triangle_contour('Pupil / fitted dark fossil slit '+str(sign),INK,oval(-sign*.295,.715,.006,.010,20),iris,.0015,precision=6),35)
  p.extend(lateral(q,sign,True) for q in (socket,white,iris,pupil))
  p.append(crease_normals(fitted_triangle_contour('Nostril / fitted dark nasal slit '+str(sign),INK,oval(sign*.030,.744,.007,.008,20),head,.003,precision=6),35))
  for upper,parent in ((True,head),(False,jaw)):
   skin=surface(turn(parent,not upper))
   for j,(z,x) in enumerate(((.36,.112),(.46,.098),(.56,.070))):
    yy=skin(-z if not upper else z,sign*x)*(1 if not upper else -1);direction=-1 if upper else 1
    p.append(curved_tube('Tooth / rooted ivory '+('upper' if upper else 'lower')+' '+str(sign)+' '+str(j),IVORY,[(sign*x,yy-direction*.008,z),(sign*x*.95,yy+direction*.018,z+.004),(sign*x*.88,yy+direction*(.047 if j==0 else .037),z+.011)],[.014,.010,.0015],14))
  # Wing skin and leading arm share depth at the rooted wrist.
  def depth(x,y):return -.09-.06*abs(x)+.025*(y-.70)**2
  outline=contour([((.11,.69),(.26,.72),(.42,.83),(.52,.99)),((.52,.99),(.78,1.10),(1.13,1.08),(1.28,.97)),((1.28,.97),(1.01,.93),(.86,.73),(.84,.51)),((.84,.51),(.65,.68),(.52,.66),(.35,.40)),((.35,.40),(.23,.50),(.15,.47),(.11,.43)),((.11,.43),(.09,.51),(.10,.62),(.11,.69))],10)
  case=wing_case(outline,sign)
  p.extend((case,wing_mark(case,outline,sign)))
  leading=[(.105,.685),(.30,.77),(.52,.99),( .91,1.066),(1.28,.97)]
  p.append(wing_frame(curved_tube('Wing arm / continuous bent leading fossil arm '+str(sign),GRAY,[(sign*x,y,depth(x,y)+.006) for x,y in leading],[.050,.038,.027,.022,.003],18),sign))
  for j,tip in enumerate(((.84,.51),(.35,.40))):
   x,y=tip;pts=[(.52,.99),((.52+x)*.5,(.99+y)*.5+.015),tip]
   p.append(wing_frame(curved_tube('Wing finger / seated branching fossil spar '+str(sign)+' '+str(j),GRAY,[(sign*a,b,depth(a,b)+.006) for a,b in pts],[.020,.014,.003],14),sign))
  for j,dx in enumerate((-.033,.009,.045)):
   pts=[(sign*(.52+dx),.990,depth(.52+dx,.99)+.02),(sign*(.525+dx),1.065,depth(.525+dx,1.065)+.031),(sign*(.56+dx),1.080,depth(.56+dx,1.08)+.046)]
   p.append(wing_frame(curved_tube('Wing claw / hooked wrist digit '+str(sign)+' '+str(j),GRAY,pts,[.015,.012,.002],14),sign))
  leg=curved_tube('Leg / bent fossil hind limb '+str(sign),GRAY,[(sign*.11,.28,-.10),(sign*.19,.19,-.070),(sign*.19,.072,-.075)],[.075,.047,.028],20);p.append(leg)
  foot=ellipsoid('Foot / grounded narrow fossil pad '+str(sign),GRAY,(sign*.19,.040,-.065),(.049,.040,.055),24,16);p.append(foot)
  for j,dx in enumerate((-.044,0,.044)):
   p.append(curved_tube('Foot claw / curved pointed gray talon '+str(sign)+' '+str(j),GRAY,[(sign*.19+dx*.55,.043,-.035),(sign*.19+dx,.025,.045),(sign*.19+dx*1.2,.008,.095)],[.018,.012,.0015],14))
 p.append(curved_tube('Tail / continuous slender swept fossil tail',GRAY,TAIL,[.075,.055,.027,.012],24));p.append(arrow())
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for j,q in enumerate(p):
  if normalize:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
  if normalize and q['part'].startswith('Skull /'):
   from chikorita_sculpt import f32
   vertices=[tuple(f32(v) for v in q['positions'][i:i+3]) for i in range(0,len(q['positions']),3)];faces=[tuple(q['indices'][i:i+3]) for i in range(0,len(q['indices']),3)]
   q=crease_normals(part(q['part'],GRAY,vertices,faces,True),40.);p[j]=q
  q['normals']=[round(v,7) or 0. for v in q['normals']]
 return {'name':'aerodactyl','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,required=True);args=a.parse_args();args.out.mkdir(parents=True,exist_ok=True)
 from aerodactyl_glb import export_aerodactyl
 blob=export_aerodactyl();(args.out/'aerodactyl.glb').write_bytes(blob);print('aerodactyl',len(blob),'bytes',flush=True)
if __name__=='__main__':main()
