#!/usr/bin/env python3
"""Original stern leek-carrying duck with folded grasp and webbed feet."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,fitted_contour,fitted_patch,crease_normals,tube,subdivided_part
from bovine_sculpt import oval,curved_tube
from abra_sculpt import surface,polygon_caps
from dodo_sculpt import feather
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.10
BROWN=(.61,.56,.53);LIGHT=(.70,.65,.60);CREAM=(.91,.91,.80);WHITE=(.98,.98,.93);GOLD=(.96,.77,.31);INK=(.12,.11,.12);GREEN=(.43,.63,.27);PALE=(.74,.83,.51)
def webfoot(sign):
 xy=[(-.047,-.026),(.047,-.026),(.060,.071),(.090,.140),(.036,.112),(.020,.173),(-.012,.125),(-.047,.163),(-.046,.102),(-.091,.120),(-.056,.047)]
 xy=[(sign*.105+x,z) for x,z in xy];n=len(xy);caps=polygon_caps([(x,z,0) for x,z in xy]);v=[(x,.013,z) for x,z in xy]+[(x,.061-.022*max(0,z)/.173,z) for x,z in xy];f=list(caps)+[tuple(n+i for i in reversed(t)) for t in caps]
 f += [(i,i+n,(i+1)%n+n,(i+1)%n) for i in range(n)]
 return crease_normals(subdivided_part('Foot / grounded three-point golden web '+str(sign),GOLD,v,f,1,True),40)
def sculpture():
 body=loft('Body / rounded gray-brown upper duck torso',BROWN,curved_profiles(((.305,.237,.181,-.020),(.395,.255,.207,-.020),(.530,.219,.180,-.022),(.644,.144,.116,-.013),(.704,.051,.050,-.007)),4),40);p=[body]
 p.append(loft('Underbody / rooted rounded cream duck underside',CREAM,curved_profiles(((.115,.044,.040,-.015),(.155,.165,.133,-.015),(.255,.242,.182,-.020),(.329,.251,.201,-.020)),4),40))
 head=ellipsoid('Head / rounded upright gray-brown duck hood',BROWN,(0,.795,.005),(.178,.204,.156),40,28);p.append(head);hs=surface(head)
 for sign in (-1,1):
  x=sign*.076;y=.816;xy=contour((((x-sign*.034,y-.064),(x-sign*.040,y+.015),(x-sign*.019,y+.079),(x+sign*.007,y+.077)),((x+sign*.007,y+.077),(x+sign*.034,y+.054),(x+sign*.040,y-.022),(x+sign*.025,y-.061)),((x+sign*.025,y-.061),(x+sign*.002,y-.075),(x-sign*.025,y-.074),(x-sign*.034,y-.064))),10)
  border=fitted_contour('Eye border / fitted fine stern duck socket '+str(sign),INK,xy,hs,.005,subdivisions=3);p.append(border);center=(x,y)
  white=fitted_contour('Eye white / fitted tall ivory duck eye '+str(sign),WHITE,[(x+(a-x)*.91,y+(b-y)*.93) for a,b in xy],surface(border),.003,subdivisions=3);p.append(white)
  iris=fitted_patch('Pupil / fitted long dark duck pupil '+str(sign),INK,oval(x+sign*.010,y+.008,.013,.041,28),surface(white),.002);p.append(iris)
  p.append(fitted_patch('Glint / fitted small ivory duck eye light '+str(sign),WHITE,oval(x+sign*.010-.003,y+.030,.005,.011,20),surface(iris),.0015))
 brow=[(-.098,.925),(-.030,.878),(0,.851),(.098,.934),(.067,.878),(0,.825),(-.052,.881)]
 p.append(fitted_contour('Brow / fitted dark angular forehead chevron',INK,brow,hs,.006,subdivisions=3))
 beak=ellipsoid('Bill / broad rounded flattened golden duck beak',GOLD,(0,.730,.181),(.124,.038,.122),40,26);p.append(beak);ss=surface(beak)
 seam=contour((((-.102,.724),(-.048,.709),(.044,.709),(.102,.724)),((.102,.724),(.048,.715),(-.045,.715),(-.102,.724))),12)
 p.append(fitted_contour('Bill seam / fitted curved dark lower beak division',INK,seam,ss,.0025,subdivisions=3))
 for j,(start,end,w) in enumerate((((-.041,.939,-.012),(-.117,1.098,-.010),.029),((0,.968,-.035),(.024,1.196,-.060),.048),((.045,.945,-.018),(.146,1.095,-.020),.032))):p.append(feather('Crest / rooted three-pronged brown crown '+str(j),BROWN,start,end,w,True))
 for sign in (-1,1):
  p.append(curved_tube('Leg / short rooted golden duck shin '+str(sign),GOLD,[(sign*.108,.192,-.011),(sign*.105,.104,.004),(sign*.105,.051,.020)],[.033,.030,.033],24));p.append(webfoot(sign));foot=p[-1]
  for j,dx in enumerate((-.039,.0,.044)):
   # A small darker ridge sits inside each web, not at a detached toe tip.
   skin=surface({'positions':[v for i in range(0,len(foot['positions']),3) for x,y,z in [foot['positions'][i:i+3]] for v in (x,-z,y)],'indices':foot['indices']})
   pts=[]
   for k in range(7):z=.050+.050*k/6;x=sign*.105+dx*k/6;pts.append((x,skin(-z,x)+.0015,z))
   p.append(crease_normals(tube('Web crease / supported fine golden toe fold '+str(sign)+' '+str(j),(.79,.57,.21),pts,[.001]*len(pts),8),35))
 raised=curved_tube('Raised wing / rooted curved left feather shoulder',LIGHT,[(-.171,.561,-.015),(-.272,.694,-.034),(-.370,.790,-.015)],[.074,.079,.070],28);p.append(raised)
 for j,end in enumerate(((-.453,1.046,.015),(-.540,.928,.035),(-.594,.811,.050))):p.append(feather('Raised feather / rooted broad left wing fan '+str(j),LIGHT,(-.365,.784,-.012),end,.055))
 grasp=curved_tube('Holding wing / broad continuous cross-body folded grasp',BROWN,[(.176,.565,.035),(.166,.542,.172),(0,.505,.238),(-.185,.515,.245),(-.335,.555,.225)],[.082,.093,.102,.080,.051],32);p.append(grasp)
 for j,y in enumerate((.574,.536,.498)):
  p.append(ellipsoid('Holding feather / rooted rounded folded wing finger '+str(j),BROWN,(-.087,y,.294),(.209,.029,.064),36,24))
 for j,y in enumerate((.526,.581)):p.append(ellipsoid('Grip / rounded gray-brown feather curl around leek '+str(j),BROWN,(-.326,y,.198),(.046,.029,.050),28,18))
 stem=curved_tube('Leek stem / continuous curved pale white stalk',CREAM,[(-.250,.118,.189),(-.292,.398,.194),(-.339,.656,.176),(-.355,.818,.157)],[.044,.038,.036,.039],28);p.append(stem)
 p.append(curved_tube('Leek neck / pale green rooted leaf sheath',PALE,[(-.346,.714,.171),(-.357,.820,.155),(-.367,.889,.145)],[.043,.045,.034],28))
 for j,end in enumerate(((-.489,1.127,.115),(-.369,1.163,.097),(-.243,1.104,.143))):p.append(feather('Leek leaf / closed broad curved green blade '+str(j),GREEN,(-.359,.830,.152),end,.041))
 for j,end in enumerate(((-.083,.334,-.403),(.066,.371,-.425),(.213,.405,-.370))):p.append(feather('Tail / rooted short brown duck feather '+str(j),BROWN,(.036,.274,-.146),end,.039,True))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye border /','Eye white /','Pupil /','Glint /','Brow /','Bill seam /')):p[i]=crease_normals(q,35)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'farfetch_d','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);m=sculpture();(args.output/'farfetch_d.mesh.json').write_text(json.dumps(m,separators=(',',':')));print(len(m['primitives']),sum(len(q['indices'])//3 for q in m['primitives']),flush=True)
if __name__=='__main__':main()
