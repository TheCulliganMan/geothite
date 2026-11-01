#!/usr/bin/env python3
"""Original Houndour puppy and lean horned Houndoom canine sculptures."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch as _fitted_patch,crease_normals,translate
from abra_sculpt import surface
from spider_sculpt import upper_skin
from wing_insect_sculpt import membrane
from skin_glb import COORDINATES
SPECIES=('houndour','houndoom');HEIGHT=1.20
COAT=(.23,.24,.25);BONE=(.88,.88,.83);ORANGE=(.80,.43,.27);INK=(.085,.095,.105)
WHITE=(.96,.96,.91);RUBY=(.76,.28,.28);TONGUE=(.80,.43,.47)

def oval(x,y,w,h,n=20):return [(x+w*math.cos(k*math.tau/n),y+h*math.sin(k*math.tau/n)) for k in range(n)]

def fitted_patch(name,color,outline,skin,thickness=.008):
    # Keep the original left-to-right centroid across Python versions.
    outline=list(outline);area=0.
    for a,b in zip(outline,outline[1:]+outline[:1]):area+=a[0]*b[1]-b[0]*a[1]
    if area<0:outline.reverse()
    cx=cy=0.
    for x,y in outline:cx+=x;cy+=y
    return _fitted_patch(name,color,outline,skin,thickness,center=(cx/len(outline),cy/len(outline)))

def dorsal_patch(name,color,outline,skin):
    dense=[]
    for a,b in zip(outline,outline[1:]+outline[:1]):
        dense.extend((a[0]+(b[0]-a[0])*i/5,a[1]+(b[1]-a[1])*i/5) for i in range(5))
    p=fitted_patch(name,color,dense,skin,.007)
    for field in ('positions','normals'):
        p[field]=[v for i in range(0,len(p[field]),3) for v in (p[field][i],p[field][i+2],p[field][i+1])]
    p['indices']=[v for i in range(0,len(p['indices']),3) for v in (p['indices'][i],p['indices'][i+2],p['indices'][i+1])]
    return p

def trunk(big):
    knots=((- .38,.02,.03,.52),(-.28,.125,.15,.52),(-.12,.155,.155,.53),(.07,.14,.16,.53),(.20,.11,.14,.56),(.28,.035,.05,.59)) if big else ((-.32,.015,.025,.38),(-.26,.145,.16,.39),(-.08,.175,.185,.41),(.12,.16,.17,.43),(.22,.105,.13,.47),(.26,.02,.03,.50))
    rows=curved_profiles(knots,4);n=32;v=[]
    for z,w,h,y in rows:
        for k in range(n):a=k*math.tau/n;v.append((w*math.sin(a),y+h*math.cos(a),z))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    for j in range(len(rows)-1):
        for k in range(n):a=j*n+k;b=j*n+(k+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part('Body / continuous long canine trunk',COAT,v,f),40.)

def limb(name,controls,radii):
    rows=curved_profiles([(p[1],r,p[0],p[2]) for p,r in reversed(list(zip(controls,radii)))],4)
    p=tube(name,COAT,[(x,y,z) for y,r,x,z in rows],[r for y,r,x,z in rows],18)
    return crease_normals(p,35.),rows

def center_at(rows,y):
    for a,b in zip(rows,rows[1:]):
        if a[0]<=y<=b[0]:
            t=(y-a[0])/(b[0]-a[0]);return tuple(a[k]*(1-t)+b[k]*t for k in (2,3))
    raise ValueError('cuff outside leg centerline')

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported bone-armored canine')
    big=species=='houndoom';body=trunk(big)
    neck=loft('Neck / contoured upright canine chest',COAT,curved_profiles(((.42,.105,.12,.19),(.56,.095,.11,.24),(.72,.093,.105,.28),(.88,.091,.10,.30)) if big else ((.39,.11,.115,.17),(.50,.10,.115,.22),(.62,.105,.115,.265),(.73,.095,.10,.28)),4),28)
    head=loft('Head / long canine brow and cheek',COAT,curved_profiles(((.775,.048,.065,.34),(.83,.107,.135,.32),(.93,.131,.126,.29),(1.015,.083,.085,.27),(1.037,.015,.02,.265)) if big else ((.625,.036,.055,.345),(.67,.105,.12,.325),(.755,.145,.135,.305),(.83,.115,.10,.29),(.875,.023,.024,.28)),4),28)
    p=[body,neck,head];ns=surface(neck);hs=surface(head)
    y=.87 if big else .69
    muzzle=ellipsoid('Muzzle / projecting orange upper snout',ORANGE,(0,y,.44 if big else .418),(.088 if big else .103,.043 if big else .060,.103 if big else .088),28,16);p.append(muzzle)
    if big:
        p.append(ellipsoid('Jaw / orange lower mandible',ORANGE,(0,.771,.442),(.075,.022,.093),24,14))
        p.append(ellipsoid('Mouth / recessed dark oral cavity',INK,(0,.816,.444),(.063,.043,.078),20,12))
        p.append(ellipsoid('Mouth / pink tongue in lower jaw',TONGUE,(0,.792,.505),(.027,.014,.017),20,10))
        # Forward orange bridge rises between the angled eyes.
        p.append(fitted_patch('Muzzle / fitted orange nasal bridge',ORANGE,[(-.035,.880),(0,.965),(.035,.880)],hs,.004))
    ms=surface(muzzle)
    p.append(ellipsoid('Nose / broad dark canine nose',INK,(0,y+.020,ms(y+.020,0)+.003),(.039,.023,.016),24,14))
    if not big:
        points=[(x,.667+.012*abs(x/.078),ms(.667+.012*abs(x/.078),x)+.002) for x in (-.078,-.052,-.026,0,.026,.052,.078)]
        p.append(tube('Mouth / fitted closed puppy lip',INK,points,[.0025]*len(points),10))
    for side in (-1,1):
        x=side*(.077 if big else .093);ey=.936 if big else .766
        outline=[(x-side*.025,ey+.004),(x+side*.030,ey+.033),(x+side*.031,ey-.025),(x-side*.023,ey-.021)]
        socket=fitted_patch('Eye / fitted dark angled socket '+str(side),INK,outline,hs,.004);p.append(socket)
        cx=sum(a for a,b in outline)/4;cy=sum(b for a,b in outline)/4
        eye=fitted_patch('Eye / fitted ivory canine white '+str(side),WHITE,[(cx+(a-cx)*.80,cy+(b-cy)*.75) for a,b in outline],surface(socket),.0025);p.append(eye)
        iris=fitted_patch('Eye / fitted red narrow iris '+str(side),RUBY,oval(cx,cy,.005,.013),surface(eye),.0015);p.append(iris)
        p.append(fitted_patch('Eye / thin dark pupil '+str(side),INK,oval(cx,cy,.002,.009,12),surface(iris),.001))
        if big:
            points=[(side*.096,.955,.265),(side*.18,1.040,.27),(side*.29,1.10,.225),(side*.35,1.07,.17),(side*.35,.98,.14),(side*.31,.918,.16),(side*.26,.892,.205)]
            radii=[.052,.059,.062,.053,.036,.019,.0015]
            # Dense interpolated circular arc avoids angular tube joints.
            dense=[];rs=[]
            for j in range(len(points)-1):
                a=points[max(j-1,0)];b=points[j];c=points[j+1];d=points[min(j+2,len(points)-1)]
                for k in range(4):
                    t=k/4;dense.append(tuple(.5*(2*b[q]+(-a[q]+c[q])*t+(2*a[q]-5*b[q]+4*c[q]-d[q])*t*t+(-a[q]+3*b[q]-3*c[q]+d[q])*t*t*t) for q in range(3)));rs.append(radii[j]*(1-t)+radii[j+1]*t)
            p.append(tube('Horn / broad hooked ivory crescent '+str(side),BONE,dense+[points[-1]],rs+[radii[-1]],20))
            for j,x in enumerate((side*.046,)):
                p.append(tube('Fang / descending upper canine '+str(side),WHITE,[(x,.852,.502),(x,.820,.526),(x,.802,.530)],[.012,.009,.001],14))
        else:
            ear=[(side*.055,.813),(side*.222,1.045),(side*.194,.837),(side*.091,.795)]
            p.append(membrane('Ear / broad pointed puppy ear '+str(side),COAT,ear,1,root_z=.24,thickness=.027))
            p.append(tube('Fang / small puppy canine '+str(side),WHITE,[(side*.06,.669,.47),(side*.059,.649,.475)],[.009,.001],12))
        for row,z in enumerate((.20,-.235)):
            if big:
                controls=[(side*.125,.53,.16),(side*.14,.34,.17),(side*.14,.085,.24)] if row==0 else [(side*.12,.52,-.25),(side*.145,.37,-.25),(side*.165,.23,-.18),(side*.17,.08,-.245)]
                radii=[.056,.042,.029] if row==0 else [.079,.06,.036,.028]
            else:
                controls=[(side*.115,.415,.18),(side*.12,.27,.21),(side*.12,.08,.24)] if row==0 else [(side*.12,.43,-.24),(side*.15,.27,-.24),(side*.145,.10,-.19)]
                radii=[.059,.047,.040] if row==0 else [.073,.05,.04]
            controls[-1]=(controls[-1][0],.055,controls[-1][2])
            controls.insert(0,(side*.055,controls[0][1]+(.035 if big else .030),controls[0][2]));radii.insert(0,.034)
            leg,rows=limb(f'Leg / sculpted canine support {side} {row}',controls,radii);p.append(leg)
            px=controls[-1][0];pz=controls[-1][2]+.024
            paw=ellipsoid(f'Paw / grounded rounded pad {side} {row}',COAT,(px,.042,pz),(.060,.042,.083),20,12);p.append(paw);skin=surface(paw)
            for j,dx in enumerate((-.021,.021)):
                yy=.055;xx=px+dx
                points=[(xx,yy,skin(yy,xx)+.001),(xx,yy-.022,skin(yy-.022,xx)+.001)]
                p.append(tube(f'Paw crease / fitted toe separation {side} {row} {j}',INK,points,[.0018,.0015],8))
            if big:
                for j,dx in enumerate((-.031,0,.031)):
                    p.append(tube(f'Claw / pale hooked toe {side} {row} {j}',WHITE,[(px+dx,.038,pz+.065),(px+dx,.021,pz+.097),(px+dx,.011,pz+.117)],[.013,.010,.001],12))
            for j,cy in enumerate((.145,.207) if big else (.142,)):
                cx,cz=center_at(rows,cy);width=.047 if big else .055
                p.append(translate(loft(f'Cuff / broad ivory ankle band {side} {row} {j}',BONE,[(cy-.019,width*.89,width*.89,cz),(cy-.011,width,width,cz),(cy+.012,width,width,cz),(cy+.019,width*.88,width*.88,cz)],20),cx,0,0))
    roof=upper_skin(body)
    for j,z in enumerate((.045,-.10,-.235) if big else (-.025,-.205)):
        width=.116 if big else .135
        p.append(dorsal_patch('Armor / broad fitted dorsal bone band '+str(j),BONE,oval(0,z,width,.035 if big else .045,24),roof))
    if big:
        collar=tube('Collar / curved ivory neck bones',BONE,[(-.075,.75,.31),(-.12,.73,.33),(-.14,.68,.33),(-.105,.63,.34),(0,.615,.344),(.105,.63,.34),(.14,.68,.33),(.12,.73,.33),(.075,.75,.31)],[.029,.031,.031,.030,.026,.030,.031,.031,.029],18);p.append(collar)
        charm=ellipsoid('Charm / flat bone skull pendant',BONE,(0,.605,ns(.605,0)+.023),(.033,.044,.018),24,16);p.append(charm)
        cs=surface(charm)
        for side in (-1,1):p.append(fitted_patch('Charm / fitted dark skull eye '+str(side),INK,oval(side*.012,.613,.006,.010,12),cs,.002))
        # Long round tail curls behind the body and ends in a distinct spade.
        points=[(0,.53,-.325),(0,.52,-.43)]
        for k in range(20):
            a=-math.pi/2+(2.3+math.pi/2)*k/19;points.append((.014*math.sin(a),.76+.25*math.sin(a),-.52-.26*math.cos(a)))
        p.append(tube('Tail / long thin curling devil tail',COAT,points,[.026,.023]+[.019-.007*k/19 for k in range(20)],16))
        end=points[-1];outline=[(-.044,end[1]-.016),(0,end[1]-.10),(.044,end[1]-.016),(.016,end[1]+.025),(0,end[1]-.009),(-.016,end[1]+.025)]
        p.append(membrane('Tail / pointed adult spade',COAT,outline,1,root_z=end[2]+.003,thickness=.013))
    else:
        p.append(tube('Tail / short tapered puppy tail',COAT,[(0,.40,-.28),(.015,.43,-.40),(.040,.49,-.54)],[.054,.042,.0015],20))
        cap=fitted_patch('Armor / fitted forehead bone shield',BONE,[(-.055,.790),(-.080,.822),(-.043,.859),(.043,.859),(.080,.822),(.055,.790)],hs,.006);p.append(cap)
        cs=surface(cap)
        p.append(fitted_patch('Armor / dark forehead inset',INK,oval(0,.832,.031,.015),cs,.0015))
    p.append(fitted_patch('Chest / fitted warm orange marking',ORANGE,oval(0,.55 if big else .515,.055,.09,24),ns,.004))
    p=[crease_normals(q,35.) if q['part'].startswith(('Armor / broad','Chest /')) else q for q in p]
    from sculpt_geometry import linear
    for q in p:
        name=q['part']
        rgb=(0,0,0) if name.startswith(('Nose /','Mouth / recessed','Mouth / fitted closed','Eye / fitted dark','Eye / thin','Armor / dark','Charm / fitted','Paw crease /')) else (255,99,0) if name.startswith(('Muzzle /','Jaw /','Chest /','Eye / fitted red','Mouth / pink')) or (not big and name.startswith('Paw /')) else (255,255,255) if name.startswith(('Horn /','Fang /','Claw /','Cuff /','Armor /','Collar /','Charm /','Eye / fitted ivory')) else ((74,33,140) if big else (49,66,82))
        q['base_color']=linear(tuple(v/255 for v in rgb))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    if normalize:
        for q in p:
            q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
            q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
