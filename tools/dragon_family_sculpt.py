#!/usr/bin/env python3
"""Original continuous blue serpents and a friendly orange guardian dragon."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,fitted_contour,fitted_patch
from bovine_sculpt import smooth,curved_tube,oval
from chikorita_sculpt import add,sub,mul,dot,cross,unit
from wing_insect_sculpt import contour,membrane
from abra_sculpt import surface
from skin_glb import COORDINATES
SPECIES=('dratini','dragonair','dragonite');HEIGHTS={'dratini':1.18,'dragonair':1.18,'dragonite':1.40}
BLUE=(.49,.64,.85);DEEP=(.19,.43,.74);ORANGE=(.93,.64,.29);CREAM=(.96,.92,.76);WHITE=(.98,.98,.94);TEAL=(.21,.59,.58);INK=(.08,.09,.12);PURPLE=(.41,.20,.40);FOLD=(.64,.54,.35)

def painted_tube(name,color,points,radii,hint=(0,0,1)):
 pts,rs=smooth(points,radii,8);n=48;v=[];side=None
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
 ids=list(range(-8,9));w=len(ids);skin=[]
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
 skin=surface(head)
 for sign in (-1,1):
  xx=sign*x
  socket=crease_normals(fitted_contour('Eye socket / fitted gentle dark oval '+str(sign),INK,oval(xx,y,w,h,32),skin,.005,subdivisions=2),35);p.append(socket)
  if serpent:
   iris=crease_normals(fitted_contour('Iris / fitted violet serpent oval '+str(sign),PURPLE,oval(xx,y-.005,w*.77,h*.78,32),surface(socket),.0025,subdivisions=2),35);p.append(iris);parent=iris
  else:
   white=crease_normals(fitted_contour('Eye white / fitted ivory guardian oval '+str(sign),WHITE,oval(xx,y,w*.80,h*.84,32),surface(socket),.0025,subdivisions=2),35);p.append(white);parent=white
  pupil=crease_normals(fitted_contour('Pupil / fitted dark round dragon gaze '+str(sign),INK,oval(xx+sign*.003,y-.002,w*.43,h*.58,28),surface(parent),.002,subdivisions=2),35);p.append(pupil)
  p.append(crease_normals(fitted_patch('Glint / small ivory eye reflection '+str(sign),WHITE,oval(xx+sign*.003-.003,y+h*.23,w*.09,h*.17,20),surface(pupil),.0018),35))

def serpents(species):
 big=species=='dragonair'
 pts=[(0,.905,.13),(0,.58,.15),(.03,.27,.25),(.23,.16,.32),(.46,.15,.10),(.42,.11,-.20),(.04,.085,-.39),(-.38,.13,-.35),(-.53,.31,-.26),(-.52,.50,-.24)] if big else [(0,.89,.12),(.055,.64,.12),(.17,.41,.105),(.12713,.25,.080),(.01,.13287,.052),(-.15,.09,.023),(-.40,.105,-.02),(-.65,.18,-.06),(-.85,.22,-.08)]
 rs=[.057,.068,.080,.085,.090,.090,.075,.050,.028,.002] if big else [.102,.112,.115,.112,.106,.092,.060,.029,.0015]
 p=painted_tube('Body / continuous coiled '+species+' serpent',BLUE,pts,rs)
 if big:
  head=loft('Head / slender blue dragon brow and lowered muzzle',BLUE,curved_profiles(((.868,.026,.034,.15),(.92,.102,.095,.15),(1.025,.113,.087,.125),(1.075,.060,.048,.12),(1.09,.009,.012,.12)),4),40)
  muzzle=ellipsoid('Muzzle / rooted slender blue snout',BLUE,(0,.920,.212),(.076,.041,.079),40,24)
 else:
  head=ellipsoid('Head / broad rounded pale-blue hatchling head',BLUE,(0,.955,.12),(.17,.18,.15),48,28)
  muzzle=ellipsoid('Muzzle / broad rooted ivory hatchling nose',WHITE,(0,.897,.250),(.105,.064,.053),40,24)
 p.extend((head,muzzle));eyes(p,head,.072 if big else .096,1.007 if big else .977,.025 if big else .034,.050 if big else .063,True)
 if big:
  p.append(curved_tube('Horn / rooted forward ivory forehead lance',WHITE,[(0,1.049,.177),(0,1.085,.25),(0,1.106,.291)],[.028,.019,.0015],24))
 else:p.append(crease_normals(fitted_patch('Forehead / supported round ivory hatchling dot',WHITE,oval(0,1.064,.018,.018,28),surface(head),.004),35))
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
   p.append(crease_normals(fitted_patch('Orb glint / fitted ivory pearl reflection '+str(j),WHITE,oval(c[0]-.014,c[1]+.020,r[0]*.19,r[1]*.22,24),surface(orb),.002),35))
 return p

def guardian():
 body=loft('Body / continuous orange guardian pear and neck',ORANGE,curved_profiles(((.16,.035,.047,-.025),(.27,.225,.190,-.02),(.45,.290,.235,-.015),(.63,.270,.215,-.015),(.79,.220,.170,0),(.94,.135,.120,.014),(1.08,.095,.087,.015)),4),48)
 head=ellipsoid('Head / rounded gentle guardian brow',ORANGE,(0,1.105,.050),(.165,.177,.145),48,28)
 muzzle=ellipsoid('Muzzle / broad rooted orange guardian snout',ORANGE,(0,1.024,.178),(.134,.075,.116),48,28);p=[body,head,muzzle];eyes(p,head,.080,1.140,.029,.045,False)
 belly=crease_normals(fitted_contour('Belly / supported broad cream segmented breast',CREAM,[(-.066,1.028),(.066,1.028),(.105,.96),(.13,.80),(.215,.50),(.16,.25),(0,.20),(-.16,.25),(-.215,.50),(-.13,.80),(-.105,.96)],surface(body),.012,subdivisions=4),35);p.append(belly);bs=surface(belly)
 for j,(y,w) in enumerate(((.30,.11),(.40,.17),(.50,.19),(.60,.175),(.70,.145),(.80,.113),(.89,.085),(.96,.066))):
  xy=[(-w,y+.006),(-w*.5,y-.001),(0,y-.004),(w*.5,y-.001),(w,y+.006),(w,y+.010),(w*.5,y+.003),(0,y),( -w*.5,y+.003),(-w,y+.010)]
  p.append(crease_normals(fitted_contour('Belly fold / fitted curved cream plate seam '+str(j),FOLD,xy,bs,.004,subdivisions=3),35))
 p.append(curved_tube('Horn / short rooted orange crown point',ORANGE,[(0,1.266,.02),(0,1.333,.04)],[.029,.0015],24))
 ms=surface(muzzle)
 smile=contour([((-.09,1.005),(-.07,.962),(.07,.962),(.09,1.005)),((.09,1.005),(.07,.968),(-.07,.968),(-.09,1.005))],18)
 p.append(crease_normals(fitted_contour('Mouth / supported gentle curved guardian smile',INK,smile,ms,.0025,subdivisions=2),35))
 for sign in (-1,1):
  p.append(crease_normals(fitted_patch('Nostril / tiny supported guardian nasal slit '+str(sign),INK,oval(sign*.044,1.037,.005,.008,24),ms,.0025),35))
  p.append(curved_tube('Antenna / rooted long curved cream guardian feeler '+str(sign),CREAM,[(sign*.074,1.218,-.014),(sign*.18,1.30,-.07),(sign*.28,1.285,-.065),(sign*.36,1.31,-.040),(sign*.39,1.37,-.005),(sign*.375,1.414,.012)],[.014,.013,.012,.011,.009,.004],24))
  arm=curved_tube('Arm / rooted curved orange guardian forelimb '+str(sign),ORANGE,[(sign*.19,.78,.03),(sign*.30,.67,.09),(sign*.36,.54,.15),(sign*.37,.47,.20)],[.073,.068,.065,.053],28);p.append(arm)
  hand=ellipsoid('Hand / rounded orange guardian palm '+str(sign),ORANGE,(sign*.37,.48,.21),(.063,.060,.057),32,20);p.append(hand)
  for j,dx in enumerate((-.033,0,.033)):
   p.append(curved_tube('Hand claw / rooted ivory guardian digit '+str(sign)+' '+str(j),WHITE,[(sign*.37+dx,.475,.250),(sign*.37+dx,.45,.285),(sign*.37+dx,.429,.309)],[.015,.011,.0015],18))
  p.append(ellipsoid('Haunch / rooted broad orange guardian thigh '+str(sign),ORANGE,(sign*.22,.27,.01),(.117,.166,.143),40,24))
  p.append(curved_tube('Leg / rooted short guardian ankle '+str(sign),ORANGE,[(sign*.23,.20,.04),(sign*.245,.10,.07),(sign*.25,.055,.11)],[.066,.057,.036],28))
  foot=ellipsoid('Foot / broad grounded orange guardian pad '+str(sign),ORANGE,(sign*.25,.044,.135),(.105,.044,.134),40,24);p.append(foot)
  for j,dx in enumerate((-.065,0,.065)):
   p.append(curved_tube('Foot claw / curved ivory guardian talon '+str(sign)+' '+str(j),WHITE,[(sign*.25+dx,.045,.225),(sign*.25+dx,.033,.295),(sign*.25+dx,.015,.332)],[.025,.020,.0015],20))
  outline=contour([((.13,.80),(.26,.89),(.37,1.02),(.42,1.14)),((.42,1.14),(.61,1.10),(.75,.84),(.75,.61)),((.75,.61),(.67,.74),(.55,.77),(.45,.64)),((.45,.64),(.34,.76),(.24,.75),(.13,.66)),((.13,.66),(.12,.71),(.12,.76),(.13,.80))],10)
  p.append(membrane('Wing border / rooted orange guardian sail '+str(sign),ORANGE,outline,sign,root_z=-.11,thickness=.012))
  p.append(membrane('Wing / inset teal guardian membrane on both faces '+str(sign),TEAL,outline,sign,root_z=-.11,inset=.91,thickness=.016))
  def depth(x,y):return -.11-.06*abs(x)+.025*(y-.70)**2
  path=[(.13,.80),(.30,.98),(.42,1.14),(.59,1.04),(.73,.83),(.75,.61)]
  p.append(curved_tube('Wing spar / rooted curved orange leading guardian edge '+str(sign),ORANGE,[(sign*x,y,depth(x,y)) for x,y in path],[.026,.024,.022,.018,.013,.003],24))
  for j,(x,y) in enumerate(((.45,.64),(.13,.66))):
   pts=[(.42,1.14),((.42+x)/2,(1.14+y)/2+.01),(x,y)]
   p.append(curved_tube('Wing rib / seated curved guardian support '+str(sign)+' '+str(j),ORANGE,[(sign*a,b,depth(a,b)) for a,b in pts],[.012,.009,.003],18))
 p.extend(painted_tube('Tail / continuous thick swept orange guardian tail',ORANGE,[(0,.28,-.12),(.08,.20,-.38),(.23,.22,-.58),(.38,.33,-.75),(.47,.47,-.84),(.49,.59,-.88)],[.13,.115,.080,.050,.025,.0015],(0,-1,0)))
 return p

def sculpture(species):
 if species not in SPECIES:raise ValueError('unsupported dragon family')
 p=guardian() if species=='dragonite' else serpents(species)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,required=True);args=a.parse_args();args.out.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:
  m=sculpture(s);(args.out/(s+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(s,len(m['primitives']),sum(len(p['indices'])//3 for p in m['primitives']),flush=True)
if __name__=='__main__':main()
