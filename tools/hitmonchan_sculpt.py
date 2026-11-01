#!/usr/bin/env python3
"""Original boxer with a rounded crown skull, open pleated hem and planted boots."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals,translate,fitted_contour,fitted_triangle_contour
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from bulldog_sculpt import rounded_loft
from cubone_sculpt import skin_shading
from skin_glb import COORDINATES,smoothstep
HEIGHT=1.30
TAN=(.65,.58,.46);LAVENDER=(.68,.65,.80);HEM=(.61,.58,.72)
RED=(.83,.31,.29);CUFF=(.70,.23,.23);INK=(.075,.075,.10)
WHITE=(.98,.98,.96);BLUE=(.26,.44,.54)

def paint(name,color,xy,parent,th=.006,sub=2):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th),45),parent)

def rounded_cap_cage(outline):
 # Ear clipping alone leaves long sliver fans across the skull. Improve the
 # internal planar diagonals before curving the cap; its outline stays fixed.
 cage=fitted_contour('crown cage',TAN,outline,lambda y,x:0.,.08,3);xy=[];lookup={};faces=[]
 for k in range(0,len(cage['indices']),3):
  tri=[cage['positions'][3*i:3*i+3] for i in cage['indices'][k:k+3]]
  if not all(p[2]>0 for p in tri):continue
  face=[]
  for x,y,z in tri:
   if (x,y) not in lookup:lookup[(x,y)]=len(xy);xy.append((x,y))
   face.append(lookup[(x,y)])
  faces.append(tuple(face))
 def orient(a,b,c):return (xy[b][0]-xy[a][0])*(xy[c][1]-xy[a][1])-(xy[b][1]-xy[a][1])*(xy[c][0]-xy[a][0])
 def positive(face):return face if orient(*face)>0 else (face[0],face[2],face[1])
 faces=[positive(f) for f in faces]
 for step in range(120):
  edges={}
  for i,face in enumerate(faces):
   for a,b,c in (face,face[1:]+face[:1],face[2:]+face[:2]):edges.setdefault(tuple(sorted((a,b))),[]).append((i,c))
  changed=set()
  for (a,b),uses in sorted(edges.items()):
   if len(uses)!=2:continue
   (i,c),(j,d)=uses
   if i in changed or j in changed or tuple(sorted((c,d))) in edges:continue
   if orient(a,b,c)*orient(a,b,d)>=-1e-18 or orient(c,d,a)*orient(c,d,b)>=-1e-18:continue
   if orient(a,b,c)<0:a,b=b,a
   ax,ay=xy[a][0]-xy[d][0],xy[a][1]-xy[d][1];bx,by=xy[b][0]-xy[d][0],xy[b][1]-xy[d][1];cx,cy=xy[c][0]-xy[d][0],xy[c][1]-xy[d][1]
   determinant=(ax*ax+ay*ay)*(bx*cy-by*cx)-(bx*bx+by*by)*(ax*cy-ay*cx)+(cx*cx+cy*cy)*(ax*by-ay*bx)
   if determinant<=1e-14:continue
   faces[i]=positive((c,d,a));faces[j]=positive((d,c,b));changed.update((i,j))
  if not changed:break
 else:raise ValueError('crown cap triangulation did not settle')
 return xy,faces

def crown_head():
 # The five crown points rise out of a rounded skull; the face is not a plate.
 xy=[(0,1.047),(-.125,1.073),(-.209,1.150),(-.248,1.290),(-.245,1.410),(-.227,1.530),(-.190,1.540),(-.143,1.430),(-.119,1.607),(-.078,1.621),(-.051,1.443),(.020,1.649),(.063,1.640),(.079,1.443),(.160,1.593),(.198,1.566),(.166,1.407),(.245,1.493),(.278,1.455),(.232,1.290),(.203,1.156),(.113,1.075)]
 xy,caps=rounded_cap_cage(xy);vv=[]
 for sign in (1,-1):
  for x,y in xy:
   t=smoothstep(1.380,1.510,y);rounded=.180*math.sqrt(max(0.,1-(x/.270)**2-((y-1.270)/.240)**2));depth=.012+(1.-t)*rounded+.023*t;center=.028-.105*t;vv.append((x,y,center+sign*depth))
 count=len(xy);faces=list(caps)+[tuple(i+count for i in reversed(f)) for f in caps];edges={}
 for f in caps:
  for a,b in zip(f,f[1:]+f[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
 for uses in edges.values():
  if len(uses)==1:
   a,b=uses[0];faces.append((b,a,a+count,b+count))
 head=part('Head / continuous rounded tan five-point crown',TAN,vv,faces)
 # Smooth each rounded cap as one surface. The narrow crown walls keep their
 # own normals instead of averaging through the opposing front/back caps.
 from chikorita_sculpt import cross,sub,unit,f32
 points=[tuple(head['positions'][i:i+3]) for i in range(0,len(head['positions']),3)];faces=[];sums={}
 for k in range(0,len(head['indices']),3):
  tri=[points[i] for i in head['indices'][k:k+3]];normal=cross(sub(tri[1],tri[0]),sub(tri[2],tri[0]));group=1 if normal[2]>1e-10 else -1 if normal[2]<-1e-10 else 0;faces.append((tri,normal,group))
  if group:
   for point in tri:
    key=(point,group);sums[key]=tuple(a+b for a,b in zip(sums.get(key,(0.,0.,0.)),normal))
 out=dict(head,positions=[],normals=[],indices=[]);lookup={}
 for tri,normal,group in faces:
  for point in tri:
   n=tuple(f32(v) for v in unit(sums[(point,group)] if group else normal));key=point+n
   if key not in lookup:lookup[key]=len(out['positions'])//3;out['positions'].extend(point);out['normals'].extend(n)
   out['indices'].append(lookup[key])
 return out

def skirt():
 # A closed cloth shell with an actual open hem, rather than a solid cone.
 rows=[(.490,.305,.219),(.552,.269,.198),(.626,.205,.159),(.711,.168,.136)];n=64;vv=[]
 for inside in (False,True):
  for y,w,d in rows:
   fade=(.711-y)/(.711-.490)
   for i in range(n):
    a=math.tau*i/n;fold=1+.072*fade*math.cos(8*a);inset=.016 if inside else 0.;vv.append(((w-inset)*math.sin(a)*fold,y,.015+(d-inset)*math.cos(a)*fold))
 count=n*len(rows);ff=[]
 for inside in (0,1):
  start=inside*count
  for j in range(len(rows)-1):
   for i in range(n):
    a=start+j*n+i;b=start+j*n+(i+1)%n;face=(a,b,b+n,a+n);ff.append(tuple(reversed(face)) if inside else face)
 for i in range(n):
  a=i;b=(i+1)%n;ff.append((b,a,a+count,b+count));a+=(len(rows)-1)*n;b+=(len(rows)-1)*n;ff.append((a,b,b+count,a+count))
 return crease_normals(part('Skirt / continuous open eight-pleat lavender hem',LAVENDER,vv,ff,True),35)

def arm_path(s):
 if s==-1:return [(s*.210,1.069,.023),(s*.360,1.010,.130),(s*.450,1.100,.280)],(s*.480,1.220,.390)
 return [(s*.209,1.063,.028),(s*.400,.945,.130),(s*.370,1.150,.320)],(s*.350,1.260,.390)

def limbs(p):
 for s in (-1,1):
  p.append(paddle('Leg / rooted long tan boxing support '+str(s),TAN,[(s*.100,.625,.010),(s*.174,.340,.010),(s*.184,.102,.010)],[.079,.063,.055],[.082,.065,.048],segments=24,steps=5))
  boot=translate(rounded_loft('Boot / grounded broad lavender boxing shoe '+str(s),LAVENDER,[(0.,.104,.150,.070),(.035,.110,.156,.070),(.092,.103,.143,.067),(.122,.073,.090,.025)],28,.7),s*.185,0,0);p.append(boot)
  cuff=translate(crease_normals(loft('Boot cuff / rooted lavender ankle collar '+str(s),HEM,curved_profiles(((.095,.066,.073,.010),(.151,.066,.070,.010),(.205,.054,.056,.010)),3),28),40),s*.184,0,0);p.append(cuff)
  xy=[(s*(.185+dx),y) for dx,y in ((-.080,.041),(-.045,.061),(0,.069),(.045,.061),(.080,.041),(.055,.045),(.032,.055),(0,.061),(-.032,.055),(-.055,.045))]
  p.append(paint('Boot seam / fitted raised lavender toe piping '+str(s),HEM,xy,boot,.003))
  xy=[(s*x,y) for x,y in ((.125,1.045),(.140,1.180),(.385,1.223),(.300,.982),(.232,.962))]
  guard=fitted_contour('Shoulder guard / rooted pointed tan boxing armor '+str(s),TAN,xy,lambda y,x:.015,.100,2)
  guard['positions']=[.015+(.100 if v>.015 else -.100) if i%3==2 else v for i,v in enumerate(guard['positions'])];p.append(crease_normals(guard,40))
  path,center=arm_path(s);widths=[.082,.070,.055] if s==-1 else [.082,.044,.051]
  p.append(paddle('Arm / rooted bent tan boxing forelimb '+str(s),TAN,path,widths,widths,segments=24,steps=5))
  wrist=path[-1];cuffend=tuple(wrist[k]+.60*(center[k]-wrist[k]) for k in range(3))
  p.append(curved_tube('Glove cuff / rooted dark red wrist collar '+str(s),CUFF,[wrist,cuffend],[.065,.073],24))
  cx,cy,cz=center;rows=[(cy-.125,.065,.070,cz-.040),(cy-.075,.110,.108,cz),(cy+.025,.128,.130,cz+.020),(cy+.105,.108,.112,cz+.022),(cy+.143,.058,.070,cz+.018)]
  glove=translate(rounded_loft('Glove / rounded padded boxing mitt '+str(s),RED,rows,28,.75),cx,0,0);p.append(glove)
  p.append(ellipsoid('Glove thumb / rooted curled boxing fold '+str(s),RED,(cx-s*.091,cy-.058,cz+.037),(.057,.073,.064),24,16))
  xy=[(cx+dx,cy+dy) for dx,dy in ((-.085,-.070),(-.04,-.092),(.03,-.095),(.083,-.073),(.075,-.081),(.03,-.103),(-.04,-.100),(-.080,-.078))]
  p.append(paint('Glove seam / fitted dark lower mitt contour '+str(s),CUFF,xy,glove,.003))

def sculpture(normalize=True):
 torso=crease_normals(loft('Tunic / continuous sleeveless lavender boxing torso',LAVENDER,curved_profiles(((.565,.127,.113,.015),(.675,.145,.131,.015),(.750,.185,.155,.015),(.925,.239,.190,.013),(1.027,.222,.167,.017),(1.108,.098,.080,.039)),4),40),45)
 neck=ellipsoid('Neck / rooted short tan boxing neck',TAN,(0,1.116,.045),(.088,.110,.086),28,18)
 head=crown_head();p=[torso,neck,head,skirt()]
 p.append(crease_normals(loft('Belt / rooted narrow charcoal boxing waist',INK,curved_profiles(((.680,.166,.139,.019),(.703,.181,.147,.018),(.731,.184,.146,.015)),3),40),40))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.045,1.273),(.176,1.312),(.171,1.246),(.113,1.219),(.069,1.228))]
  socket=paint('Eye socket / fitted angular dark boxing recess '+str(s),INK,xy,head,.005);p.append(socket);cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted ivory boxing gaze '+str(s),WHITE,[(cx+.88*(x-cx),cy+.87*(y-cy)) for x,y in xy],socket,.0035);p.append(eye)
  iris=paint('Iris / fitted dark boxing stare '+str(s),BLUE,oval(s*.125,1.264,.014,.024,20),eye,.0025);p.append(iris)
  p.append(paint('Glint / tiny ivory boxing eye point '+str(s),WHITE,oval(s*.123,1.274,.002,.004,12),iris,.0015))
 p.append(paint('Mouth / fitted firm boxing jaw crease',INK,[(-.067,1.146),(.067,1.142),(.065,1.134),(-.065,1.138)],head,.003))
 limbs(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 from sculpt_geometry import linear
 for q in p:
  n=q['part'];rgb=(0,0,0) if n.startswith(('Eye socket /','Iris /','Pupil /','Mouth /','Belt /','Glove seam /')) else (255,255,255) if n.startswith(('Eye /','Glint /')) else (222,24,148) if n.startswith(('Tunic /','Skirt /','Boot /','Boot cuff /','Boot seam /','Glove')) else (173,123,99)
  q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:
   q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'hitmonchan','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'hitmonchan.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
