#!/usr/bin/env python3
"""Original rounded baby elephant with an upturned trunk and drooping ears."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import fitted_patch,oval,curved_tube
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.02
# Local normal RGB555: blue (17,20,26), warm marking (23,9,6).
BLUE=(140/255,165/255,214/255);RED=(189/255,74/255,49/255);INK=(0.,0.,0.);WHITE=(1.,1.,1.);PINK=RED
def ear(sign):
 p=ellipsoid('Ear / rounded drooping blue leaf '+str(sign),BLUE,(0,0,0),(.151,.236,.079),40,28)
 a=sign*.24;c=math.cos(a);s=math.sin(a)
 p['positions']=[v for i in range(0,len(p['positions']),3) for x,y,z in [p['positions'][i:i+3]] for v in (c*x-s*y+sign*.281,s*x+c*y+.680,z+.132)]
 return crease_normals(p,35.)
def sculpture(normalize=True):
 body=ellipsoid('Body / rounded horizontal baby elephant torso',BLUE,(0,.386,-.045),(.275,.270,.370),40,28)
 head=ellipsoid('Head / broad rounded blue cranium',BLUE,(0,.714,.177),(.269,.242,.242),48,32)
 p=[body,head];hs=surface(head)
 trunk=curved_tube('Trunk / short continuous upward curling blue nose',BLUE,[(0,.720,.333),(0,.657,.415),(0,.652,.525),(0,.706,.598)],[.110,.099,.087,.081],36);p.append(trunk)
 tip=ellipsoid('Nose / rounded raised blue trunk tip',BLUE,(0,.708,.601),(.086,.077,.072),36,24);p.append(tip);ns=surface(tip)
 p.append(fitted_patch('Forehead / fitted warm saddle marking',RED,oval(0,.811,.106,.055,48),hs,.007))
 for sign in (-1,1):
  e=ear(sign);p.append(e);es=surface(e)
  patches=[[((.371,.612),(.406,.597),(.399,.520),(.366,.494)),((.366,.494),(.343,.470),(.351,.534),(.371,.612))],[((.275,.612),(.316,.581),(.335,.521),(.319,.486)),((.319,.486),(.285,.490),(.267,.542),(.275,.612))]]
  for j,curves in enumerate(patches):
   xy=[(sign*x,y) for x,y in contour(curves,12)]
   p.append(fitted_contour('Ear tip / fitted warm rounded lobe '+str(sign)+' '+str(j),RED,xy,es,.007,subdivisions=2))
  x=sign*.143;eye=fitted_patch('Eye / fitted vertical dark baby eye '+str(sign),INK,oval(x,.783,.014,.034,36),hs,.005);p.append(eye)
  p.append(fitted_patch('Eye glint / ivory vertical sparkle '+str(sign),WHITE,oval(x-.002,.797,.005,.010,24),surface(eye),.002))
  p.append(fitted_patch('Nostril / small dark trunk opening '+str(sign),INK,oval(sign*.034,.713,.007,.017,28),ns,.003))
 # Three planted supports and one playful lifted foreleg.
 for sign,z,raised in ((-1,-.256,False),(1,-.256,False),(1,.205,False),(-1,.241,True)):
  name=str(sign)+' '+('front' if z>0 else 'rear')
  if raised:
   leg=curved_tube('Leg / bent lifted foreleg '+name,BLUE,[(sign*.183,.361,.165),(sign*.213,.253,.245),(sign*.242,.285,.364)],[.089,.083,.076],32)
   center=(sign*.246,.284,.379);radii=(.082,.073,.090)
  else:
   leg=loft('Leg / planted rounded pillar '+name,BLUE,curved_profiles(((.031,.046,.050,z+.019),(.091,.075,.083,z+.017),(.211,.082,.086,z),(.326,.088,.085,z-.009),(.460,.010,.012,z-.024)),3),32)
   leg['positions']=[v+sign*.178 if i%3==0 else v for i,v in enumerate(leg['positions'])]
   center=(sign*.178,.047,z+.027);radii=(.080,.047,.097)
  p.append(leg);foot=ellipsoid('Foot / '+('lifted' if raised else 'grounded')+' rounded blue pad '+name,BLUE,center,radii,32,20);p.append(foot)
  x,y,z= center
  p.append(fitted_patch('Toenail / fitted ivory oval '+name,WHITE,oval(x,y-.016 if not raised else y,.041,.019,32),surface(foot),.004))
 p.append(curved_tube('Tail / short tapered blue rear flick',BLUE,[(0,.420,-.350),(.025,.432,-.431),(.061,.453,-.487)],[.038,.026,.001],24))
 smile=contour([((-.094,.595),(-.060,.589),(.063,.589),(.094,.595)),((.094,.595),(.070,.505),(-.057,.503),(-.094,.595))],12)
 mouth=fitted_contour('Mouth / fitted happy open baby smile',INK,smile,hs,.006);p.append(mouth)
 cx=cy=0.
 for x,y in smile:cx+=x;cy+=y
 cx/=len(smile);cy/=len(smile)
 p.append(fitted_contour('Tongue / fitted rosy smile interior',PINK,[(cx+(x-cx)*.78,cy+(y-cy)*.78) for x,y in smile],surface(mouth),.003))
 p=[crease_normals(q,35.) if q['part'].startswith(('Ear tip /','Forehead /','Eye /','Eye glint /','Nostril /','Toenail /','Mouth /','Tongue /')) else q for q in p]
 if not normalize:return {'name':'phanpy','version':1,'coordinate_system':COORDINATES,'primitives':p}
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:
  q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'phanpy','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 (args.output/'phanpy.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
