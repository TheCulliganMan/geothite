#!/usr/bin/env python3
"""Original Pichu: broad tipped ears, jagged black collar and lightning tail."""
import math
from sculpt_geometry import ellipsoid,loft,curved_profiles,part,crease_normals,linear
from abra_sculpt import surface
from bovine_sculpt import fitted_patch,oval,curved_tube
from skin_glb import COORDINATES
from skarmory_sculpt import blade
HEIGHT=.62
YELLOW=(1.,.9,.25);INK=(0.,0.,0.);WARM=(.9,.3,.1);WHITE=(1.,1.,1.)
def sculpture(normalize=True):
 body=loft('Body / small yellow pear',YELLOW,curved_profiles(((.085,.050,.050,-.014),(.15,.116,.095,-.014),(.25,.133,.102,-.012),(.33,.105,.085,-.005),(.385,.065,.060,0)),4),32)
 head=ellipsoid('Head / oversized round yellow mouse crown',YELLOW,(0,.535,.017),(.233,.212,.168),36,24);p=[body,head];skin=surface(head)
 # The closed black ruff wraps the neck, with an alternating lower silhouette.
 collar=loft('Collar / continuous jagged black neck ruff',INK,[(.323,.107,.087,-.002),(.371,.105,.087,-.002),(.398,.080,.070,0)],24)
 vs=[tuple(collar['positions'][i:i+3]) for i in range(0,len(collar['positions']),3)]
 vs=[(x,y+(.035 if j%2 else 0) if abs(y-.323)<1e-7 else y,z) for j,(x,y,z) in enumerate(vs)]
 p.append(crease_normals(part(collar['part'],INK,vs,[tuple(collar['indices'][i:i+3]) for i in range(0,len(collar['indices']),3)]),40))
 for s in (-1,1):
  ear=blade('Ear / broad black pointed mouse ear '+str(s),INK,[(s*.102,.665),(s*.177,.861),(s*.350,.932),(s*.303,.725),(s*.220,.635)],lambda x,y:.008,.032);p.append(ear)
  p.append(fitted_patch('Ear inset / fitted yellow triangular center '+str(s),YELLOW,[(s*.145,.693),(s*.192,.824),(s*.301,.868),(s*.268,.739),(s*.218,.671)],surface(ear),.002))
  p.append(curved_tube('Arm / short reaching yellow paw '+str(s),YELLOW,[(s*.093,.294,.018),(s*.165,.258,.047),(s*.185,.227,.073)],[.035,.031,.026],18))
  p.append(curved_tube('Leg / supporting yellow shin '+str(s),YELLOW,[(s*.070,.171,-.006),(s*.087,.104,.010),(s*.102,.048,.037)],[.043,.036,.031],18))
  p.append(ellipsoid('Foot / planted long mouse toe '+str(s),YELLOW,(s*.104,.034,.077),(.062,.034,.101),28,18))
  eye=fitted_patch('Eye / fitted large black mouse oval '+str(s),INK,oval(s*.097,.562,.027,.038,32),skin,.004);p.append(eye)
  p.append(fitted_patch('Eye glint / fitted white mouse sparkle '+str(s),WHITE,oval(s*.097-.006,.575,.009,.010,24),surface(eye),.0015))
  p.append(fitted_patch('Cheek / warm round electric pouch '+str(s),WARM,oval(s*.176,.476,.032,.029,32),skin,.003))
 p.append(fitted_patch('Nose / tiny black mouse button',INK,[(-.014,.500),(.014,.500),(0,.488)],skin,.004))
 mouth=fitted_patch('Mouth / fitted tiny happy mouse smile',INK,[(-.032,.458),(0,.447),(.032,.458),(.021,.430),(0,.420),(-.021,.430)],skin,.003);p.append(mouth)
 p.append(fitted_patch('Tongue / fitted warm smile interior',WARM,[(-.015,.433),(.015,.433),(0,.426)],surface(mouth),.001))
 p.append(curved_tube('Tail / thin rooted black stem',INK,[(0,.198,-.085),(.035,.202,-.190),(.095,.248,-.282)],[.016,.014,.015],14))
 p.append(blade('Tail / broad black lightning flag',INK,[(.079,.239),(.099,.351),(.165,.351),(.148,.305),(.231,.305),(.214,.184),(.151,.184),(.163,.241)],lambda x,y:-.280,.020))
 low=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-low)
 for q in p:
  n=q['part'];rgb=(0,0,0) if n.startswith(('Ear /','Collar /','Eye /','Nose /','Mouth /','Tail /')) else (255,255,255) if n.startswith('Eye glint /') else (214,49,0) if n.startswith(('Cheek /','Tongue /')) else (239,214,41)
  q['base_color']=linear(tuple(v/255 for v in rgb))
  if normalize:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'pichu','version':1,'coordinate_system':COORDINATES,'primitives':p}
