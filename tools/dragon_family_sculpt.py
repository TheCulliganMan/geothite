#!/usr/bin/env python3
"""Original continuous blue serpents and a friendly orange guardian dragon."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,tube,curved_profiles,ellipsoid,crease_normals,fitted_triangle_contour,fitted_adaptive_contour
from bovine_sculpt import smooth,curved_tube,oval
from cubone_sculpt import skin_shading
from chikorita_sculpt import add,sub,mul,dot,cross,unit
from wing_insect_sculpt import contour,membrane
from abra_sculpt import surface
from skin_glb import COORDINATES
SPECIES=('dratini','dragonair','dragonite');HEIGHTS={'dratini':1.18,'dragonair':1.18,'dragonite':1.40}
BLUE=(.49,.64,.85);DEEP=(.19,.43,.74);ORANGE=(241/255,171/255,0.);CREAM=(231/255,227/255,181/255);WHITE=(.98,.98,.94);TEAL=(.21,.59,.58);INK=(.08,.09,.12);PURPLE=(.41,.20,.40);FOLD=(.64,.54,.35)

def fitted(name,color,outline,parent,thickness=.001,subdivisions=2,*,precision=7):
 if name.startswith('Belly fold /'):
  # Weld clipped stripe boundaries before extrusion instead of repeatedly
  # subdividing long narrow adaptive triangles across the whole breast.
  return skin_shading(crease_normals(fitted_triangle_contour(name,color,outline,parent,thickness,precision=6),40),parent)
 if name.startswith('Mouth /'):
  return skin_shading(fitted_adaptive_contour(name,color,outline,parent,thickness,initial_subdivisions=0,max_surface_error=.0006),parent)
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,outline,parent,thickness,precision=precision),35),parent)

def painted_tube(name,color,points,radii,hint=(0,0,1),*,segments=48,steps=8):
 pts,rs=smooth(points,radii,steps);n=segments;v=[];side=None
 for k,p in enumerate(pts):
  t=unit(sub(pts[min(k+1,len(pts)-1)],pts[max(0,k-1)]))
  side=unit(sub(hint if side is None else side,mul(t,dot(hint if side is None else side,t))));other=cross(t,side)
  v.extend(add(p,mul(add(mul(side,math.cos(i*math.tau/n)),mul(other,math.sin(i*math.tau/n))),rs[k])) for i in range(n))
 fs=[tuple(reversed(range(n))),tuple((len(pts)-1)*n+i for i in range(n))]
 for k in range(len(pts)-1):
  for i in range(n):a=k*n+i;b=k*n+(i+1)%n;fs.append((a,b,b+n,a+n))
 body=crease_normals(part(name,color,v,fs,True),40)
 # A closed shallow ribbon uses the SAME body ring vertices and face diagonals,
 # with a tapered outward lift and embedded back. It follows the curved body
 # through its full turn rather than projecting a flat white shape through it.
 ids=list(range(-n//6,n//6+1));w=len(ids);skin=[]
 for front in (True,False):
  for k,p in enumerate(pts):
   delta=min(.002,rs[k]*.12) if front else -min(.001,rs[k]*.07)
   skin.extend(add(p,mul(sub(v[k*n+i%n],p),(rs[k]+delta)/rs[k])) for i in ids)
 count=len(skin)//2;faces=[]
 for k in range(len(pts)-1):
  for j in range(w-1):a=k*w+j;b=a+1;faces.append((a,b,b+w,a+w))
 caps=list(faces);faces.extend(tuple(i+count for i in reversed(f)) for f in caps);edges={}
 for f in caps:
  for a,b in zip(f,f[1:]+f[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
 for uses in edges.values():
  if len(uses)==1:a,b=uses[0];faces.append((b,a,a+count,b+count))
 return [body,crease_normals(part(('Tail underside /' if name.startswith('Tail /') else 'Belly /')+' continuous supported ivory ribbon',CREAM if name.startswith('Tail /') else WHITE,skin,faces,True),40)]

def eyes(p,head,x,y,w,h,serpent):
 skin=head
 for sign in (-1,1):
  xx=sign*x
  socket=fitted('Eye socket / fitted gentle dark oval ' + str(sign), INK, oval(xx, y, w, h, 32), skin, 0.005, subdivisions=2,precision=7 if serpent else 6);p.append(socket)
  if serpent:
   iris=fitted('Iris / fitted violet serpent oval ' + str(sign), PURPLE, oval(xx, y - 0.005, w * 0.77, h * 0.78, 32), socket, 0.0025, subdivisions=2);p.append(iris);parent=iris
  else:
   white=fitted('Eye white / fitted ivory guardian oval ' + str(sign), WHITE, oval(xx, y, w * 0.8, h * 0.84, 32), socket, 0.0025, subdivisions=2,precision=6);p.append(white);parent=white
  pupil=fitted('Pupil / fitted dark round dragon gaze ' + str(sign), INK, oval(xx + sign * 0.003, y - 0.002, w * 0.43, h * 0.58, 28), parent, 0.002, subdivisions=2,precision=7 if serpent else 6);p.append(pupil)
  p.append(fitted('Glint / small ivory eye reflection ' + str(sign), WHITE, oval(xx + sign * 0.003 - 0.003, y + h * 0.23, w * 0.09, h * 0.17, 20), pupil, 0.0018,precision=7 if serpent else 6))

def spine_path(species):
 if species=='dragonite':return [(0.,.36,-.12),(.015,.235,-.41),(.06,.215,-.70),(.13,.22,-1.00),(.21,.25,-1.28),(.24,.28,-1.48)],[.145,.145,.115,.075,.035,.0015],(0,-1,0),24,6
 if species not in ('dratini','dragonair'):raise ValueError('unknown dragon spine')
 big=species=='dragonair'
 pts = [(0, 0.905, 0.13), (0, 0.58, 0.15), (0.03, 0.27, 0.25), (0.23, 0.16, 0.32), (0.46, 0.15, 0.1), (0.42, 0.11, -0.2), (0.04, 0.085, -0.39), (-0.38, 0.13, -0.35), (-0.53, 0.31, -0.26), (-0.52, 0.5, -0.24)] if big else [(0, 0.89, 0.12), (0.055, 0.64, 0.12), (0.17, 0.41, 0.105), (0.12713, 0.25, 0.08), (0.01, 0.13287, 0.052), (-0.15, 0.09, 0.023), (-0.4, 0.105, -0.02), (-0.65, 0.18, -0.06), (-0.85, 0.22, -0.08)]
 rs = [0.057, 0.068, 0.08, 0.085, 0.09, 0.09, 0.075, 0.05, 0.028, 0.002] if big else [0.102, 0.112, 0.115, 0.112, 0.106, 0.092, 0.06, 0.029, 0.0015]
 return pts,rs,(0,0,1),48,8

def serpents(species):
 big=species=='dragonair'
 pts,rs,hint,segments,steps=spine_path(species)

 p=painted_tube('Body / continuous coiled '+species+' serpent',BLUE,pts,rs,hint,segments=segments,steps=steps)
 if big:
  head=loft('Head / slender blue dragon brow and lowered muzzle',BLUE,curved_profiles(((.868,.026,.034,.15),(.92,.102,.095,.15),(1.025,.113,.087,.125),(1.075,.060,.048,.12),(1.09,.009,.012,.12)),4),40)
  muzzle=ellipsoid('Muzzle / rooted slender blue snout',BLUE,(0,.920,.212),(.076,.041,.079),40,24)
 else:
  head=ellipsoid('Head / broad rounded pale-blue hatchling head',BLUE,(0,.955,.12),(.17,.18,.15),48,28)
  muzzle=ellipsoid('Muzzle / broad rooted ivory hatchling nose',WHITE,(0,.897,.250),(.105,.064,.053),40,24)
 p.extend((head,muzzle));eyes(p,head,.072 if big else .096,1.007 if big else .977,.025 if big else .034,.050 if big else .063,True)
 if big:
  p.append(curved_tube('Horn / rooted forward ivory forehead lance',WHITE,[(0,1.049,.177),(0,1.085,.25),(0,1.106,.291)],[.028,.019,.0015],24))
 else:p.append(fitted('Forehead / supported round ivory hatchling dot', WHITE, oval(0, 1.064, 0.018, 0.018, 28), head, 0.004))
 for sign in (-1,1):
  if big:
   wing=contour([((.076,1.014),(.115,1.09),(.15,1.20),(.194,1.264)),((.194,1.264),(.20,1.235),(.207,1.213),(.205,1.178)),((.205,1.178),(.222,1.19),(.24,1.217),(.244,1.211)),((.244,1.211),(.246,1.18),(.236,1.14),(.219,1.106)),((.219,1.106),(.237,1.12),(.251,1.143),(.256,1.138)),((.256,1.138),(.242,1.05),(.187,1.003),(.133,.991)),((.133,.991),(.108,.982),(.080,.990),(.076,1.014))],8)
  else:wing=contour([((.12,1.006),(.17,1.055),(.23,1.119),(.298,1.170)),((.298,1.170),(.310,1.126),(.298,1.084),(.270,1.052)),((.270,1.052),(.239,1.026),(.214,1.016),(.213,.994)),((.213,.994),(.224,.979),(.244,.965),(.268,.938)),((.268,.938),(.214,.939),(.163,.943),(.122,.959)),((.122,.959),(.116,.973),(.116,.993),(.12,1.006))],8)
  ear=membrane('Ear fin / rooted folded ivory '+species+' pinna '+str(sign),WHITE,wing,sign,root_z=.10,thickness=.022 if big else .027);p.append(ear)
  if not big:
   xs=[(.141,1.012),(.182,1.024),(.240,1.043)]
   p.append(curved_tube('Ear fold / supported soft hatchling crease '+str(sign),(.77,.78,.75),[(sign*x,y,.10-.06*x+.025*(y-.70)**2+.024) for x,y in xs],[.003,.003,.001],14))
 if big:
  for j,(c,r) in enumerate((((0,.824,.167),(.061,.061,.049)),((-.529,.329,-.255),(.056,.058,.055)),((-.525,.416,-.245),(.041,.043,.041)))):
   orb=ellipsoid('Orb / rooted polished deep-blue pearl '+str(j),DEEP,c,r,40,24);p.append(orb)
   p.append(fitted('Orb glint / fitted ivory pearl reflection ' + str(j), WHITE, oval(c[0] - 0.014, c[1] + 0.02, r[0] * 0.19, r[1] * 0.22, 24), orb, 0.002))
 return p

def arm_path(side):
 return [(side*.13,.84,.01),(side*.24,.78,.055),(side*.32,.75,.12),(side*.355,.73,.20),(side*.35,.73,.28),(side*.34,.74,.34)],[.078,.075,.070,.065,.060,.055]

def arm_samples(side):return smooth(*arm_path(side))

def guardian():
 body=loft('Body / continuous orange guardian pear and neck',ORANGE,curved_profiles(((.135,.055,.065,-.025),(.26,.285,.255,-.02),(.45,.360,.310,-.015),(.63,.330,.280,-.015),(.79,.260,.220,0),(.94,.170,.150,.014),(1.10,.120,.105,.015)),4),32)
 head=ellipsoid('Head / rounded gentle guardian brow',ORANGE,(0,1.105,.045),(.175,.220,.185),32,20)
 muzzle=ellipsoid('Muzzle / broad rooted orange guardian snout',ORANGE,(0,1.024,.235),(.150,.085,.125),32,20);p=[body,head,muzzle];eyes(p,head,.095,1.155,.031,.050,False)
 belly=fitted('Belly / supported broad cream segmented breast',CREAM,[(-.08,1.028),(.08,1.028),(.13,.96),(.18,.8),(.290,.5),(.21,.25),(0.,.18),(-.21,.25),(-.290,.5),(-.18,.8),(-.13,.96)],body,.012,subdivisions=4);p.append(belly);bs=belly
 for j,(y,w) in enumerate(((.30,.16),(.40,.23),(.50,.26),(.60,.235),(.70,.195),(.80,.15),(.89,.11),(.96,.08))):
  xy=[(-w,y+.006),(-w*.5,y-.001),(0,y-.004),(w*.5,y-.001),(w,y+.006),(w,y+.010),(w*.5,y+.003),(0,y),( -w*.5,y+.003),(-w,y+.010)]
  p.append(fitted('Belly fold / fitted curved cream plate seam ' + str(j), FOLD, xy, bs, 0.004, subdivisions=3))
 p.append(curved_tube('Horn / short rooted orange crown point',ORANGE,[(0,1.292,.02),(0,1.36,.04)],[.029,.0015],18))
 ms=muzzle
 smile=contour([((-.09,1.005),(-.07,.962),(.07,.962),(.09,1.005)),((.09,1.005),(.07,.968),(-.07,.968),(-.09,1.005))],18)
 p.append(fitted('Mouth / supported gentle curved guardian smile', INK, smile, ms, 0.0025, subdivisions=2))
 for sign in (-1,1):
  p.append(fitted('Nostril / tiny supported guardian nasal slit ' + str(sign), INK, oval(sign * 0.044, 1.037, 0.005, 0.008, 24), ms, 0.0025))
  p.append(curved_tube('Antenna / rooted long curved cream guardian feeler '+str(sign),CREAM,[(sign*.075,1.255,-.01),(sign*.14,1.335,-.035),(sign*.235,1.37,-.045),(sign*.30,1.40,-.02),(sign*.29,1.43,.005)],[.015,.013,.011,.008,.003],14))
  ap,ar=arm_samples(sign)
  arm=crease_normals(tube('Arm / rooted curved orange guardian forelimb '+str(sign),ORANGE,ap,ar,18),35);p.append(arm)
  hand=ellipsoid('Hand / rounded orange guardian palm '+str(sign),ORANGE,(sign*.34,.75,.36),(.070,.065,.070),20,14);p.append(hand)
  for j,dx in enumerate((-.033,0,.033)):
   p.append(curved_tube('Hand claw / rooted ivory guardian digit '+str(sign)+' '+str(j),WHITE,[(sign*.34+dx,.73,.41),(sign*.34+dx,.706,.445),(sign*.34+dx,.695,.47)],[.017,.012,.0015],12))
  p.append(ellipsoid('Haunch / rooted broad orange guardian thigh '+str(sign),ORANGE,(sign*.25,.28,.01),(.150,.190,.170),28,16))
  p.append(curved_tube('Leg / rooted short guardian ankle '+str(sign),ORANGE,[(sign*.25,.20,.04),(sign*.255,.10,.045),(sign*.26,.055,.047)],[.066,.057,.036],20))
  foot=ellipsoid('Foot / broad grounded orange guardian pad '+str(sign),ORANGE,(sign*.26,.044,.135),(.115,.044,.160),28,16);p.append(foot)
  for j,dx in enumerate((-.065,0,.065)):
   p.append(curved_tube('Foot claw / curved ivory guardian talon '+str(sign)+' '+str(j),WHITE,[(sign*.26+dx,.045,.225),(sign*.26+dx,.033,.295),(sign*.26+dx,.015,.332)],[.025,.020,.0015],12))
  outline=contour([((.13,.80),(.26,.89),(.37,1.02),(.42,1.14)),((.42,1.14),(.61,1.10),(.75,.84),(.75,.61)),((.75,.61),(.67,.74),(.55,.77),(.45,.64)),((.45,.64),(.34,.76),(.24,.75),(.13,.66)),((.13,.66),(.12,.71),(.12,.76),(.13,.80))],10)
  p.append(membrane('Wing border / rooted orange guardian sail '+str(sign),ORANGE,outline,sign,root_z=-.11,thickness=.012))
  p.append(membrane('Wing / inset teal guardian membrane on both faces '+str(sign),TEAL,outline,sign,root_z=-.11,inset=.91,thickness=.016))
  def depth(x,y):return -.11-.06*abs(x)+.025*(y-.70)**2
  path=[(.13,.80),(.30,.98),(.42,1.14),(.59,1.04),(.73,.83),(.75,.61)]
  p.append(curved_tube('Wing spar / rooted curved orange leading guardian edge '+str(sign),ORANGE,[(sign*x,y,depth(x,y)) for x,y in path],[.026,.024,.022,.018,.013,.003],16))
  for j,(x,y) in enumerate(((.45,.64),(.13,.66))):
   pts=[(.42,1.14),((.42+x)/2,(1.14+y)/2+.01),(x,y)]
   p.append(curved_tube('Wing rib / seated curved guardian support '+str(sign)+' '+str(j),ORANGE,[(sign*a,b,depth(a,b)) for a,b in pts],[.012,.009,.003],12))
 tp,tr,th,tn,ts=spine_path('dragonite')
 p.extend(painted_tube('Tail / continuous thick swept orange guardian tail',ORANGE,tp,tr,th,segments=tn,steps=ts))
 # Compact back-swept wings carry their complete membranes and bones together.
 for q in p:
  if not q['part'].startswith(('Wing /','Wing border /','Wing spar /','Wing rib /')):continue
  sign=int(q['part'].split()[-2] if q['part'].startswith('Wing rib /') else q['part'].split()[-1]);a=sign*math.radians(70.);c=math.cos(a);s=math.sin(a)
  for field in ('positions','normals'):
   values=[]
   for i in range(0,len(q[field]),3):
    x,y,z=q[field][i:i+3]
    if field=='positions':x=(x-sign*.13)*.66;y=(y-.73)*.66;z=(z+.11)*.66
    values.extend((c*x+s*z+(sign*.10 if field=='positions' else 0.),y+(.90 if field=='positions' else 0.),-s*x+c*z+(-.095 if field=='positions' else 0.)))
   q[field]=values
 return p

def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError('unsupported dragon family')
 p=guardian() if species=='dragonite' else serpents(species)
 if not normalize:return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 from chikorita_sculpt import f32
 for q in p:
  fitted_guardian=species=='dragonite' and q['part'].startswith(('Belly /','Belly fold /','Eye','Pupil /','Glint /','Mouth /','Nostril /'))
  q['positions']=[(f32((v-(lo if i%3==1 else 0))*scale) if fitted_guardian else round((v-(lo if i%3==1 else 0))*scale,7)) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) or 0. for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,required=True);args=a.parse_args();args.out.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:
  from dragon_family_glb import export_dragon_family
  blob=export_dragon_family(s);(args.out/(s+'.glb')).write_bytes(blob);print(s,len(blob),'bytes',flush=True)
if __name__=='__main__':main()
