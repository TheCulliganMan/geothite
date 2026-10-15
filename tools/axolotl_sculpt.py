#!/usr/bin/env python3
"""Distinct original Wooper/Quagsire contours, fitted faces and paddle tails.

Standard-library geometry recipe shared by the canonical amphibian skins.
No game content, source sprites, external meshes or duplicate catalog.
"""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,tube,ellipsoid,curved_profiles,fitted_patch,crease_normals,subdivided_part,translate
from abra_sculpt import surface
from skin_glb import COORDINATES

SPECIES=('wooper','quagsire');HEIGHT={'wooper':.70,'quagsire':1.13}
BLUE=(.38,.70,.77);QUAG=(115/255,173/255,231/255);INK=(.07,.12,.14)
GILL=(.79,.44,.52);BAR=(.25,.44,.61);FIN=(115/255,24/255,165/255)


def oval(x,y,rx,ry,n=16):return [(x+rx*math.cos(k*math.tau/n),y+ry*math.sin(k*math.tau/n)) for k in range(n)]


def pad(name,color,x,steps=1):
    # A shallow sculpted sole and three rounded toe lobes share one closed cage.
    outline=[(-.075,-.058),(-.099,.028),(-.092,.122),(-.060,.166),
             (-.032,.156),(-.004,.181),(.026,.162),(.066,.175),
             (.101,.145),(.097,.060),(.075,-.036),(.029,-.072)]
    v=[(x+xx,.018,z+.061) for xx,z in outline]
    v+=[(x+xx*.75,.065,z*.83+.06) for xx,z in outline]
    n=len(outline);f=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]
    f += [(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
    p=subdivided_part(name,color,v,f,steps,True)
    # The sole becomes planar after the toe/heel chamfer, with an explicit floor.
    low=min(p['positions'][1::3])
    ps=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
    ps=[(a,max(0.,b-low-.004),c) for a,b,c in ps]
    fs=[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)]
    return crease_normals(part(name,color,ps,fs,True),55.)


def paddle(species,color):
    # Broad elliptical sections along backward Z form a real flattened paddle,
    # with a raised tip and a thick root. A circular tube looked like a stump.
    rows=(((.13,.030,.045,.165),(.23,.110,.065,.160),(.36,.149,.076,.180),
           (.50,.174,.065,.243),(.61,.128,.043,.317),(.66,.025,.013,.347))
          if species=='wooper' else
          ((.16,.10,.087,.25),(.31,.149,.102,.257),(.48,.186,.110,.302),
           (.65,.179,.097,.385),(.80,.123,.060,.466),(.88,.023,.016,.502)))
    profiles=curved_profiles(rows,3);segments=24;v=[];f=[]
    for distance,width,thickness,center_y in profiles:
        for k in range(segments):
            angle=k*math.tau/segments
            v.append((width*math.sin(angle),center_y+thickness*math.cos(angle),-distance))
    f=[tuple(reversed(range(segments))),tuple((len(profiles)-1)*segments+i for i in range(segments))]
    for j in range(len(profiles)-1):
        for k in range(segments):
            a=j*segments+k;b=j*segments+(k+1)%segments;f.append((a,b,b+segments,a+segments))
    return crease_normals(part('Tail / broad rising paddle',color,v,f,True),55.)


def dorsal_ribbon(name,color,path,width):
    # A low rounded sagittal fin follows the back into the upper tail. Its
    # surface has height, rather than being a lavender slab laid on the back.
    v=[];f=[];sides=10
    for j,(y,z,rise) in enumerate(path):
        taper=math.sin(math.pi*(.02+.96*j/(len(path)-1)))
        for k in range(sides):
            a=k*math.tau/sides
            v.append((width*taper*math.sin(a),y+rise*(.5+.5*math.cos(a)),z-.024*taper*math.cos(a)))
    f=[tuple(reversed(range(sides))),tuple((len(path)-1)*sides+i for i in range(sides))]
    for j in range(len(path)-1):
        for k in range(sides):a=j*sides+k;b=j*sides+(k+1)%sides;f.append((a,b,b+sides,a+sides))
    return crease_normals(part(name,color,v,f,True),50.)


def sculpture(species,*,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported axolotl species')
    big=species=='quagsire';color=QUAG if big else BLUE;ink=(0.,0.,0.) if big else INK
    profiles=(((.104,.056,.064,.01),(.17,.151,.137,0.),(.29,.192,.159,.002),
               (.40,.17,.141,.009),(.47,.164,.124,.020),(.53,.235,.155,.027),
               (.59,.281,.174,.032),(.655,.278,.169,.030),(.709,.236,.145,.02),
               (.749,.143,.100,.015),(.765,.015,.014,.008))
              if not big else
              ((.10,.076,.075,.005),(.19,.183,.172,-.006),(.32,.266,.226,-.014),
               (.49,.30,.253,-.015),(.67,.29,.243,-.001),(.82,.30,.233,.027),
               (.92,.282,.212,.039),(1.01,.216,.162,.035),
               (1.065,.109,.085,.024),(1.08,.018,.020,.016)))
    # Preserve the head's dense contour while spending more of the adult
    # triangle budget on the pads visible in the production camera.
    dense=(curved_profiles(profiles[:6],2)[:-1]+curved_profiles(profiles[5:],3)
           if big else curved_profiles(profiles,3))
    body=loft('Body / continuous broad head and small pear belly' if not big else 'Body / continuous bulky belly shoulders and broad cranium',color,dense,32)
    p=[body];skin=surface(body)
    eye_y=.650 if not big else .929;eye_x=.139 if not big else .166
    for s in (-1,1):
        p.append(fitted_patch('Eye / tiny fitted black dot '+str(s),ink,oval(s*eye_x,eye_y,.012 if not big else .014,.017 if not big else .016),skin,.003))
    span=.153 if not big else .218;mouth_y=.577 if not big else .799
    smile=[]
    for j in range(15):
        x=-span+2*span*j/14;y=mouth_y+.12*x*x/span
        smile.append((x,y,skin(y,x)+.003))
    p.append(tube('Mouth / fitted broad curved smile',ink,smile,[.0027 if not big else .0034]*len(smile),8))
    for s in (-1,1):
        x=s*(.118 if not big else .192)
        leg=loft('Leg / rounded supporting haunch '+str(s),color,curved_profiles(((.027,.043,.045,.080),(.066,.061,.057,.066),(.112,.066,.067,.033),(.169,.060,.070,.005),(.270 if big else .205,.043,.057,-.006)),3),20)
        p.append(translate(leg,x,0.,0.))
        foot=pad('Foot / grounded three-lobed pad '+str(s),color,x,2 if big else 1)
        if big:
            # Adult pads widen together with the larger supporting ankles.
            ps=[tuple(foot['positions'][i:i+3]) for i in range(0,len(foot['positions']),3)]
            ps=[(x+(xx-x)*1.15,y*2.3,z*1.15) for xx,y,z in ps]
            foot=part(foot['part'],color,ps,[tuple(foot['indices'][i:i+3]) for i in range(0,len(foot['indices']),3)],True)
        p.append(foot)
        if big:
            # Three toes remain one closed pad; shallow folds follow the real
            # upper surface instead of reading as a thin featureless disc.
            upper=dict(foot);upper['positions']=[v for i in range(0,len(foot['positions']),3)
                for v in (foot['positions'][i],foot['positions'][i+2],foot['positions'][i+1])]
            sole=surface(upper)
            for j,offset in enumerate((-.026,.026)):
                tx=x+offset
                p.append(tube('Toe crease / fitted pad fold '+str(s)+' '+str(j),ink,
                    [(tx,sole(z,tx)+.0013,z) for z in (.165,.19,.22)],[.0015]*3,8))
            arm=loft('Arm / fleshy hanging paddle '+str(s),color,curved_profiles(((.39,.018,.026,.093),(.43,.057,.053,.088),(.53,.074,.067,.043),(.65,.076,.078,.014),(.735,.049,.058,.003)),3),20)
            ps=[tuple(arm['positions'][i:i+3]) for i in range(0,len(arm['positions']),3)]
            ps=[(xx+s*(.270+(.72-y)*.19),y,z) for xx,y,z in ps]
            arm=crease_normals(part(arm['part'],color,ps,[tuple(arm['indices'][i:i+3]) for i in range(0,len(arm['indices']),3)],True),55.)
            p.append(arm);palm=surface(arm)
            for j in range(2):
                x=s*(.307+j*.026);y=.443+j*.003
                line=[(x+s*.003*t,y-.018*t) for t in (0.,.5,1.)]
                p.append(tube('Finger crease / short palm fold '+str(s)+' '+str(j),ink,
                    [(xx,yy,palm(yy,xx)+.0013) for xx,yy in line],[.0017]*3,8))
        else:
            stalk=[(s*.254,.602,.022),(s*.34,.607,.018),(s*.43,.612,.012),(s*.469,.621,.007)]
            p.append(tube('Gill / tapered lateral stalk '+str(s),GILL,stalk,[.023,.021,.016,.007],12))
            for j,x in enumerate((.349,.431)):
                for direction in (-1,1):
                    branch=[(s*x,.609,.018),(s*(x+.006),.609+direction*.042,.013),(s*(x+.017),.609+direction*(.079 if j==0 else .066),.010)]
                    p.append(tube('Gill branch / '+str(s)+' '+str(j)+' '+str(direction),GILL,branch,[.014,.011,.0045],10))
    p.append(paddle(species,color))
    if big:
        # Dorsal spine grows from the rear contour and follows the raised tail.
        rear=surface({'positions':[v*(-1 if i%3==2 else 1) for i,v in enumerate(body['positions'])],'indices':body['indices']})
        path=[(y,-rear(y,0),rise) for y,rise in ((.84,.013),(.72,.019),(.58,.023),(.44,.032),(.32,.036))]
        path += [(.401,-.48,.063),(.475,-.65,.060),(.520,-.80,.035),(.519,-.876,.009)]
        p.append(dorsal_ribbon('Dorsal fin / continuous lavender back-to-tail ridge',FIN,path,.054))
        for s in (-1,1):
            p.append(fitted_patch('Nostril / tiny snout inset '+str(s),ink,oval(s*.054,.861,.004,.0025,10),skin,.002))
    else:
        for j,(y,width) in enumerate(((.236,.047),(.298,.065),(.362,.078))):
            outline=[(-width*.92,y-.010),(-width,y),(-width*.85,y+.011),(width*.85,y+.011),(width,y),(width*.92,y-.010)]
            p.append(fitted_patch('Belly marking / fitted blue bar '+str(j),BAR,outline,skin,.0027))
    if normalize:
        low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT[species]/(high-low)
        for q in p:
            q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
            q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))

if __name__=='__main__':main()
