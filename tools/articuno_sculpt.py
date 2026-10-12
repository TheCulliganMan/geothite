#!/usr/bin/env python3
"""Original ice bird with crystalline crown and a continuous curling streamer."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,part,crease_normals,tube
from bovine_sculpt import smooth,oval
from raptor_bird_sculpt import sheet,paint,vane_xy,feather_depth,feet
from murkrow_sculpt import bill
from chikorita_sculpt import add,sub,mul,dot,cross,unit,linear
from skin_glb import COORDINATES
SPECIES=('articuno',);HEIGHT=1.28
ICE=(.39,.67,.86);FEATHER=(.46,.72,.90);DEEP=(.20,.49,.69)
PALE=(.88,.95,.96);SLATE=(.54,.58,.61);INK=(.07,.09,.12);WHITE=(.98,.99,.97)

def streamer():
 points=[(0,.34,-.155),(0,.23,-.38),(.21,.28,-.69),(.49,.48,-.81),(.65,.57,-.94),(.81,.49,-1.02),(.84,.28,-.99),(.62,.13,-.80),(.27,.11,-.57),(.14,.045,-.60)]
 widths=[.034,.062,.083,.080,.080,.078,.069,.055,.032,.0015]
 pp,ww=smooth(points,widths,7);_,tt=smooth(points,[.011,.012,.011,.010,.009,.008,.007,.006,.004,.0006],7)
 v=[];frames=[];side=(0,1,0);n=20
 for k,p in enumerate(pp):
  tangent=unit(sub(pp[min(k+1,len(pp)-1)],pp[max(0,k-1)]));side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side);frames.append((side,other))
  v.extend(add(p,add(mul(side,ww[k]*math.cos(i*math.tau/n)),mul(other,tt[k]*math.sin(i*math.tau/n)))) for i in range(n))
 f=[tuple(reversed(range(n))),tuple((len(pp)-1)*n+i for i in range(n))]
 for k in range(len(pp)-1):
  for i in range(n):a=k*n+i;b=k*n+(i+1)%n;f.append((a,b,b+n,a+n))
 tail=crease_normals(part('Tail / continuous curling blue streamer',DEEP,v,f,True),45);out=[tail]
 vs=[tuple(tail['positions'][i:i+3]) for i in range(0,len(tail['positions']),3)]
 triangles=[tuple(vs[i] for i in tail['indices'][k:k+3]) for k in range(0,len(tail['indices']),3)]
 def skin(origin,direction):
  hits=[]
  for a,b,c in triangles:
   e=sub(b,a);g=sub(c,a);h=cross(direction,g);den=dot(e,h)
   if abs(den)<1e-12:continue
   rel=sub(origin,a);u=dot(rel,h)/den
   if not -.000001<=u<=1.000001:continue
   q=cross(rel,e);w=dot(direction,q)/den
   if w<-.000001 or u+w>1.000001:continue
   distance=dot(g,q)/den
   if distance>1e-8:hits.append(distance)
  if not hits or min(hits)>.03:raise ValueError('streamer fold has no nearby stored ribbon surface')
  return add(origin,mul(direction,min(hits)))
 for j,offset in enumerate((-.46,0,.46)):
  curve=[];radii=[]
  for k in range(len(pp)-1):
   for step in range(3):
    t=step/3
    if k==0 and step==0:t=.001
    center=add(mul(pp[k],1-t),mul(pp[k+1],t));w=ww[k]*(1-t)+ww[k+1]*t
    side=unit(add(mul(frames[k][0],1-t),mul(frames[k+1][0],t)));other=unit(add(mul(frames[k][1],1-t),mul(frames[k+1][1],t)))
    radius=min(.004,w*.05);point=skin(add(center,mul(side,w*offset)),other)
    curve.append(sub(point,mul(other,radius*.30)));radii.append(radius)
  center=add(mul(pp[-2],.001),mul(pp[-1],.999));side,other=frames[-1];radius=min(.004,ww[-1]*.05)
  curve.append(sub(skin(add(center,mul(side,ww[-1]*offset)),other),mul(other,radius*.30)));radii.append(radius)
  out.append(crease_normals(tube('Tail fold / supported longitudinal channel '+str(j),(.16,.40,.59),curve,radii,10),40))
 return out

def wings(p):
 for sign in (-1,1):
  depth=lambda y,x:-.025-.16*abs(x)+.035*math.sin(y*2.5)
  outline= [(sign*x,y) for x,y in ((.095,.57),(.30,.82),(.52,1.01),(.60,1.29),(.74,1.39),(.72,1.15),(.51,.76),(.28,.59))]
  p.append(sheet('Wing / bent ice-blue upper arm '+str(sign),ICE,outline,depth,.035,2))
  for j in range(9):
   root=(sign*(.21+.056*j),.68+.071*j);tip=(sign*(.41+.069*j),.51+.125*j);xy=vane_xy(root,tip,.051)
   for bank in (-1,1):
    p.append(sheet(f'Primary / curved ice flight vane {sign} {j} {bank}',FEATHER if bank==1 else ICE,xy,feather_depth(depth,root,tip,.051,.025 if bank==1 else -.023,bank),.020,1))
  for j in range(7):
   root=(sign*(.17+.064*j),.67+.066*j);tip=(sign*(.32+.071*j),.55+.070*j);xy=vane_xy(root,tip,.040)
   for bank in (-1,1):p.append(sheet(f'Covert / layered ice shoulder vane {sign} {j} {bank}',FEATHER if bank==1 else ICE,xy,feather_depth(depth,root,tip,.040,.037 if bank==1 else -.029,bank),.023,1))

def sculpture(species='articuno'):
 if species not in SPECIES:raise ValueError('unsupported ice bird')
 rows=curved_profiles(((.182,.028,.038,-.023),(.260,.121,.139,-.023),(.382,.181,.172,-.018),(.514,.160,.153,-.002),(.625,.092,.090,.008),(.678,.030,.035,.016)),4)
 body=crease_normals(loft('Body / slender ice-bird breast and haunch',ICE,rows,40),45)
 neck=crease_normals(loft('Neck / slender upright ice-bird throat',ICE,curved_profiles(((.55,.074,.069,.012),(.69,.067,.065,.012),(.827,.071,.071,.022)),4),32),45)
 head=ellipsoid('Head / calm upright ice-bird cranium',ICE,(0,.877,.020),(.142,.160,.130),40,26);p=[body,neck,head]
 xy=[(-.027,.659),(-.048,.620),(-.118,.533),(-.143,.411),(-.107,.298),(-.073,.307),(-.052,.251),(-.018,.286),(0,.239),(.018,.286),(.052,.251),(.073,.307),(.107,.298),(.143,.411),(.118,.533),(.048,.620),(.027,.659)]
 p.append(paint('Breast / fitted pale feathered bib',PALE,xy,body,.012,4))
 xy=[(-.051,.596),(-.055,.706),(-.044,.792),(0,.818),(.044,.792),(.055,.706),(.051,.596)]
 p.append(paint('Throat / fitted pale feather collar',PALE,xy,neck,.010,4))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.025,.919),(.083,.930),(.112,.909),(.090,.875),(.046,.884))]
  socket=paint('Eye socket / calm fitted ice-bird lens '+str(s),INK,xy,head,.008,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  white=paint('Eye / fitted ivory almond '+str(s),WHITE,[(cx+.88*(x-cx),cy+.88*(y-cy)) for x,y in xy],socket,.006,3);p.append(white)
  pupil=paint('Pupil / quiet dark ice-bird gaze '+str(s),INK,oval(s*.079,.904,.010,.022,24),white,.004,3);p.append(pupil)
  p.append(paint('Glint / tiny ivory ice-bird light '+str(s),WHITE,oval(s*.079-.002,.915,.0025,.004,14),pupil,.003,2))
 p.append(bill('Bill / short grey hooked upper beak',SLATE,[(.121,.047,.842,.877),(.178,.054,.839,.883),(.237,.030,.826,.856),(.256,.002,.817,.831)]))
 p.append(bill('Bill / narrow dark seam',INK,[(.123,.044,.836,.841),(.178,.046,.829,.836),(.226,.020,.824,.830),(.237,.002,.826,.829)]))
 p.append(bill('Bill / short grey lower mandible',SLATE,[(.121,.042,.805,.834),(.180,.046,.800,.830),(.225,.020,.817,.826),(.239,.002,.824,.827)]))
 crowns=[ [(-.030,.962),(-.075,1.076),(.028,1.250),(.038,1.074)],
          [(-.031,.961),(-.100,1.083),(-.111,1.172),(-.031,1.087)],
          [(.003,.960),(.080,1.118),(.244,1.160),(.096,1.038)] ]
 for j,xy in enumerate(crowns):p.append(sheet('Crown / three crystalline blue blades '+str(j),DEEP,xy,lambda y,x:.129-.45*(y-.96),.025,1))
 wings(p);start=len(p);feet(p)
 for q in p[start:]:
  if q['part'].startswith(('Leg /','Foot /','Toe /')):q['base_color']=linear(SLATE)
 p.extend(streamer())
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'articuno.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
