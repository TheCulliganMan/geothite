#!/usr/bin/env python3
"""Original feathered two- and three-headed running birds."""
import argparse,copy,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,fitted_contour,fitted_patch,crease_normals
from bovine_sculpt import oval,curved_tube
from abra_sculpt import surface
from bivalve_sculpt import top_view
from skin_glb import COORDINATES
SPECIES=('doduo','dodrio');HEIGHT=1.22
BROWN=(.67,.48,.32);TAN=(.84,.77,.60);DARK=(.23,.25,.25);INK=(.09,.10,.12);WHITE=(.98,.98,.93);PINK=(.84,.54,.60)
def furry(name,color,center,radii,segments=64,rings=32):
 v=[]
 for k in range(rings+1):
  t=math.pi*(k+.025)/(rings+.05)
  for j in range(segments):
   a=math.tau*j/segments;mod=1+(.003*math.sin(11*a+7*t)+.022*max(0,math.sin(15*a-5*t))**10)*math.sin(t)**.7
   v.append((center[0]+radii[0]*math.sin(t)*math.sin(a)*mod,center[1]+radii[1]*math.cos(t)*mod,center[2]+radii[2]*math.sin(t)*math.cos(a)*mod))
 f=[tuple(reversed(range(segments))),tuple(rings*segments+j for j in range(segments))]
 for k in range(rings):
  for j in range(segments):a=k*segments+j;b=k*segments+(j+1)%segments;f.append((a,b,b+segments,a+segments))
 return crease_normals(part(name,color,v,f,True),40)
def feather(name,color,start,end,width,sharp=False):
 v=[];n=6;count=25;dx,dy,dz=[end[i]-start[i] for i in range(3)];length=math.sqrt(dx*dx+dy*dy+dz*dz);tx,ty,tz=dx/length,dy/length,dz/length
 # Broad side axis lies across the quill, with a thin closed folded section.
 sx,sy,sz=(.6,.8,0) if name.startswith('Tail /') else ((1,0,0) if abs(tx)<.85 else (0,1,0));dot=sx*tx+sy*ty+sz*tz;sx,sy,sz=sx-dot*tx,sy-dot*ty,sz-dot*tz;d=math.sqrt(sx*sx+sy*sy+sz*sz);sx,sy,sz=sx/d,sy/d,sz/d;nx,ny,nz=ty*sz-tz*sy,tz*sx-tx*sz,tx*sy-ty*sx
 for k in range(count):
  t=k/(count-1);w=.002+width*math.sin(math.pi*t)**(1.1 if sharp else .48);thick=.002+.009*math.sin(math.pi*t);cx,cy,cz=[start[i]+(end[i]-start[i])*t for i in range(3)];cy+=.035*math.sin(math.pi*t)
  for j in range(n):a=math.tau*j/n;v.append((cx+sx*w*math.cos(a)+nx*thick*math.sin(a),cy+sy*w*math.cos(a)+ny*thick*math.sin(a),cz+sz*w*math.cos(a)+nz*thick*math.sin(a)))
 f=[tuple(reversed(range(n))),tuple((count-1)*n+j for j in range(n))]
 for k in range(count-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part(name,color,v,f,True),35)
def turn(q,cx,cy,cz,yaw,pitch):
 c,s=math.cos(pitch),math.sin(pitch);a,b=math.cos(yaw),math.sin(yaw)
 def xyz(x,y,z):y,z=y*c+z*s,z*c-y*s;return x*a+z*b,y,z*a-x*b
 q['positions']=[v for i in range(0,len(q['positions']),3) for x,y,z in [q['positions'][i:i+3]] for v in tuple(t+d for t,d in zip(xyz(x,y,z),(cx,cy,cz)))];q['normals']=[v for i in range(0,len(q['normals']),3) for v in xyz(*q['normals'][i:i+3])];return q
POSES={'doduo':((-.225,1.095,.035,-.42,.08),(.215,1.230,-.025,.38,.38)),'dodrio':((-.310,1.110,.015,-.62,.10),(0,1.350,-.025,0,.40),(.310,1.135,.025,.64,-.05))}
def head(species,j):
 triple=species=='dodrio';p=[];h=furry('Head / individual scalloped brown feather ball '+str(j),BROWN,(0,0,0),(.146,.159,.137));p.append(h);ss=surface(h)
 for sign in (-1,1):
  x=sign*.071;y=.039
  xy=[(x-sign*.027,y+.018),(x+sign*.032,y+.030),(x+sign*.023,y-.011),(x-sign*.021,y-.017)] if triple else oval(x,y,.019,.028,28)
  eye=fitted_contour('Eye / fitted '+('angular ivory watchful eye ' if triple else 'black oval alert eye ')+str(j)+' '+str(sign),WHITE if triple else INK,xy,ss,.007,subdivisions=3);p.append(eye)
  if triple:p.append(fitted_patch('Pupil / fitted tiny watchful bird point '+str(j)+' '+str(sign),INK,oval(x+sign*.007,y+.002,.003,.009,20),surface(eye),.002))
  else:p.append(fitted_patch('Glint / fitted tiny pale bird eye highlight '+str(j)+' '+str(sign),WHITE,oval(x-.003,y+.010,.005,.007,20),surface(eye),.002))
 upper=top_view(loft('Upper beak / continuous tapered ivory bill '+str(j),TAN,curved_profiles(((.080,.031,.024,-.009),(.125,.043,.026,-.008),(.210,.028,.017,-.009),(.320,.012,.007,-.012),(.380,.0015,.0015,-.016)),4),24));p.append(upper)
 rows=((.075,.026,.018,.027),(.120,.035,.015,.041),(.195,.027,.010,.063),(.255,.003,.002,.068)) if triple else ((.075,.026,.017,.020),(.125,.034,.014,.024),(.230,.019,.008,.025),(.355,.002,.0015,.020))
 p.append(top_view(loft('Lower beak / separate tapered ivory mandible '+str(j),TAN,curved_profiles(rows,4),24)))
 if triple:
  p.append(ellipsoid('Mouth / dark open bill cavity '+str(j),INK,(0,-.025,.115),(.025,.018,.050),24,16));p.append(ellipsoid('Tongue / small pink open bill tongue '+str(j),PINK,(0,-.029,.170),(.014,.006,.060),24,16))
  for k in (0,1):p.append(feather('Crest / rooted sharp dark head quill '+str(j)+' '+str(k),DARK,((k-.5)*.080,.116,-.025),((k-.5)*.14,.385 if k==0 else .342,-.135),.029,True))
 return p

def sculpture(species):
 if species not in SPECIES:raise ValueError(species)
 triple=species=='dodrio';body=furry('Body / rounded scalloped brown feather torso',(.72,.54,.37),(0,.530,0),(.245,.231,.211),72,36);p=[body]
 if triple:p.append(furry('Underbelly / rooted charcoal lower feather apron',DARK,(0,.390,-.005),(.190,.130,.173),64,28))
 for sign in (-1,1):
  leg=curved_tube('Leg / long rooted '+('ivory' if triple else 'charcoal')+' runner shank '+str(sign),TAN if triple else DARK,[(sign*.115,.400,-.030),(sign*.149,.252,.004),(sign*.116,.112,.055),(sign*.151,.061,.090)],[.031,.026,.021,.023],24);p.append(leg)
  foot=ellipsoid('Foot / grounded rounded ivory toe root '+str(sign),TAN,(sign*.152,.043,.099),(.043,.035,.057),28,18);p.append(foot)
  for k,off in enumerate((-.060,0,.060)):
   end=(sign*.152+off,.018,.254-(.023 if k!=1 else 0));p.append(curved_tube('Toe / grounded curved forward runner digit '+str(sign)+' '+str(k),TAN,[(sign*.152,.046,.116),(sign*.152+off*.70,.035,.195),end],[.020,.015,.009],20));p.append(curved_tube('Claw / short rooted ivory toe point '+str(sign)+' '+str(k),WHITE,[(end[0],end[1]+.002,end[2]-.007),(end[0],.018,end[2]+.020),(end[0],.012,end[2]+.032)],[.011,.007,.0018],16))
  end=(sign*.175,.020,-.008);p.append(curved_tube('Hind toe / curved rear runner digit '+str(sign),TAN,[(sign*.155,.040,.073),(sign*.176,.027,.025),end],[.016,.012,.007],18));p.append(curved_tube('Hind claw / small rooted rear ivory point '+str(sign),WHITE,[end,(sign*.182,.016,-.028)],[.008,.0018],16))
 template=head(species,0)
 for j,(cx,cy,cz,yaw,pitch) in enumerate(POSES[species]):
  p.append(curved_tube('Neck / long separate curved rooted bird neck '+str(j),TAN if triple else DARK,[(cx*.36,.688,-.016),(cx*.74,.815,-.020),(cx*.94,cy-.225,cz-.015),(cx,cy-.110,cz)],[.041,.031,.026,.030],24))
  for q in copy.deepcopy(template):
   words=q['part'].split();words[-2 if q['part'].startswith(('Eye /','Pupil /','Glint /','Crest /')) else -1]=str(j);q['part']=' '.join(words);p.append(turn(q,cx,cy,cz,yaw,pitch))
 if triple:
  for j,end in enumerate(((-.155,.545,-.620),(.075,.585,-.687),(.312,.539,-.620))):p.append(feather('Tail / rooted broad rounded pink fan plume '+str(j),PINK,(j*.030-.030,.495,-.151),end,.100))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye /','Pupil /','Glint /')):p[i]=crease_normals(q,35)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);a.add_argument('--only',choices=SPECIES);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in ([args.only] if args.only else SPECIES):
  m=sculpture(s);(args.output/(s+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(s,len(m['primitives']),sum(len(q['indices'])//3 for q in m['primitives']),flush=True)
if __name__=='__main__':main()
