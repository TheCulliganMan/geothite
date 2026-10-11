#!/usr/bin/env python3
"""Original angular virtual duck with authored wedge panels and polygonal eyes."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,fitted_patch,fitted_contour
from abra_sculpt import surface
from bovine_sculpt import oval
from skin_glb import COORDINATES
HEIGHT=1.05
RED=(.88,.39,.47);BLUE=(.40,.75,.79);INK=(.09,.11,.13);WHITE=(.95,.97,.95)

def sections(name,color,rows):
 """Closed eight-corner transverse cages: z, width, bottom, top."""
 v=[];n=8
 for z,w,lo,hi in rows:
  h=hi-lo
  v.extend((x,y,z) for x,y in ((-.70*w,lo),(.70*w,lo),(w,lo+.24*h),(w,hi-.24*h),(.60*w,hi),(-.60*w,hi),(-w,hi-.24*h),(-w,lo+.24*h)))
 f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return part(name,color,v,f)

def upright(name,color,rows):
 v=[];n=4
 for y,w,d,z in rows:v.extend((x,y,zz) for x,zz in ((-w,z-d),(w,z-d),(w,z+d),(-w,z+d)))
 f=[(3,2,1,0),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return part(name,color,v,f)

def foot(sign):
 p=sections('Foot / grounded cyan angular paddle '+str(sign),BLUE,((-.09,.087,.025,.205),(.115,.125,0.,.165),(.320,.103,.016,.118)))
 c=math.cos(sign*.22);s=math.sin(sign*.22)
 for field in ('positions','normals'):
  values=[]
  for i in range(0,len(p[field]),3):
   x,y,z=p[field][i:i+3];values.extend((c*x+s*z+(sign*.232 if field=='positions' else 0),y,-s*x+c*z))
  p[field]=values
 return p

def side_view(p,sign,inverse=False):
 q=dict(p)
 for field in ('positions','normals'):
  q[field]=[v for i in range(0,len(p[field]),3) for x,y,z in [p[field][i:i+3]] for v in ((sign*z,y,-sign*x) if inverse else (-sign*z,y,sign*x))]
 return q

def sculpture():
 body=sections('Body / authored rose angular virtual-duck hull',RED,((-.30,.100,.16,.40),(-.21,.245,.13,.49),(.06,.270,.105,.44),(.23,.190,.13,.32)))
 belly=sections('Breast / seated cyan sloping chest and lower hull',BLUE,((-.235,.161,.124,.213),(.08,.264,.096,.458),(.285,.204,.106,.324)))
 neck=upright('Neck / short angular rose connector',RED,((.385,.109,.102,.06),(.51,.078,.083,.12),(.665,.094,.101,.165)))
 head=sections('Head / long sloping rose polygonal cranium',RED,((-.045,.104,.760,1.035),(.095,.197,.650,1.025),(.290,.142,.620,.890),(.390,.090,.615,.755)))
 bill=sections('Bill / tapered cyan downward-sloping prism',BLUE,((.325,.108,.607,.775),(.500,.103,.555,.650),(.610,.048,.535,.584)))
 tail=upright('Tail / narrow rooted cyan triangular blade',BLUE,((.335,.028,.020,-.258),(.49,.082,.022,-.34),(.99,.020,.011,-.50),(1.075,.004,.005,-.53)))
 p=[body,belly,neck,head,bill,tail,foot(-1),foot(1)];hs=surface(head)
 for sign in (-1,1):
  # Fit the flat six-sided eye to the actual side panel rather than folding it
  # across the head's forward silhouette bevels.
  x=-sign*.145;y=.803;skin=surface(side_view(head,sign));xy=[(x+.063*math.cos(math.tau*j/6),y+.073*math.sin(math.tau*j/6)) for j in range(6)]
  border=fitted_contour('Eye border / fitted dark hexagonal socket '+str(sign),INK,xy,skin,.005,subdivisions=2)
  white=fitted_contour('Eye white / fitted ivory polygonal panel '+str(sign),WHITE,[(x+(xx-x)*.92,y+(yy-y)*.92) for xx,yy in xy],surface(border),.003,subdivisions=2)
  pupil=fitted_patch('Pupil / fitted small round digital eye '+str(sign),INK,oval(x,y,.014,.018,24),surface(white),.003)
  glint=fitted_patch('Eye glint / tiny fitted ivory digital sparkle '+str(sign),WHITE,oval(x-.002,y+.005,.003,.004,16),surface(pupil),.001)
  p.extend(side_view(q,sign,True) for q in (border,white,pupil,glint))
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'porygon','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'porygon.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
